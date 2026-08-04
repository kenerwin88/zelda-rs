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

use snes::{DmaChannel, DmaState, PpuState, WRAM_SIZE};

use crate::config::config_value_bytes;
#[cfg(test)]
use crate::game_state::constants::messaging::MODULE as MESSAGING_MODULE;
use crate::game_state::constants::nmi::{
    BG_CHAR_BUFFER_1 as NMI_BG_CHAR_BUFFER_1, BG_CHAR_HALF_BUFFER as NMI_BG_CHAR_HALF_BUFFER,
};
use crate::game_state::constants::{
    ANIMATED_TILE_VRAM_ADDR, BG2_X_SCROLL, BG2_Y_SCROLL, CRYSTAL_ROTATION_COUNTER,
    HDMA_TABLE_DYNAMIC, MESSAGING_BUF_LOAD_GFX, MOVING_WALL_REPLACEMENT_BUFFER,
    OVERWORLD_SCROLL_X_END, OVERWORLD_SCROLL_X_START, OVERWORLD_SCROLL_Y_END, RESERVED_HDMA_TABLE,
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
    SystemSignalsState, SystemWorkArea, TagalongSlotRead, TowerSealOrbitState,
    TowerSealSparkleState, WeatherVaneDebrisSlotState,
};
use crate::raster_timing::{
    attract_map_projection_current_word_is_visible, SpriteMainTimingWorkload,
    ATTRACT_MAP_PROJECTION_WORDS,
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
const DUNGEON_LANDING_HDMA_RESET_PREFIX_SCANLINES: usize = 4;
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
const SNES9X_INTRO_SPRITE_ANIMATION_START_DELAY: u8 = 1;
const SNES9X_POLY_UPLOAD_DEFER_UNTIL_FRAME_COUNTER: u8 = 0x42;
const SNES9X_NMI_POLY_UPLOAD_DEFER_FRAMES: u8 = 3;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct NmiActiveDisplayBlanking {
    prefix_scanlines: u8,
    suffix_start_scanline: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ActiveDisplayBlankingScanout {
    suffix_start_scanline: Option<u8>,
    retain_prior_surface: bool,
}

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

const fn hud_tilemap_nmi_forced_blank_prefix(upload_consumed: bool) -> u8 {
    // NMI subroutine 1 uploads the complete $800-byte tilemap staging buffer.
    // With ordinary core updates ahead of it, instrumented Snes9x reaches the
    // DMA at ROM $008cdb on V=249 and does not restore INIDISP until V=1,
    // H=870. The ROM asserted forced blank at NMI entry, so scanline zero of
    // the snapshot captured immediately before this NMI is black.
    if upload_consumed {
        1
    } else {
        0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct StripeUploadWork {
    packets: usize,
    transfer_bytes: usize,
    fixed_source_packets: usize,
    vertical_packets: usize,
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

const fn rom_intro_poly_thread_is_active(main_module: u8, submodule: u8) -> bool {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GraphicsDmaGeneration {
    HostBoundaryBeforeMain,
    LiveAfterMain,
}

impl GraphicsDmaGeneration {
    const fn resolve_live_override(self, publish_live_generation: bool) -> Self {
        if publish_live_generation {
            Self::LiveAfterMain
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OamScanoutSource {
    RetainCapturedBeforeNmi,
    /// Keep the OAM already resident in the captured PPU. Unlike the ordinary
    /// retained cadence, do not replace it with an adjacent published shadow:
    /// an interrupted native workload can leave the completed hardware DMA and
    /// the next software shadow on opposite sides of the host boundary.
    RetainResidentPpuOam,
    ComposePublishedShadowDma,
    ComposeLiveAfterNmi,
    /// Publish the OAM shadow authored by this main slice. Dungeon big-item
    /// pickup enters Link's hold-item handler before the boundary that starts
    /// the receipt sequence, so the live shadow wins even though the resident
    /// native PPU still owns the preceding DMA generation.
    ComposeLivePlayerOamAfterMain,
}

impl From<GraphicsDmaGeneration> for OamScanoutSource {
    fn from(generation: GraphicsDmaGeneration) -> Self {
        match generation {
            GraphicsDmaGeneration::HostBoundaryBeforeMain => Self::RetainCapturedBeforeNmi,
            GraphicsDmaGeneration::LiveAfterMain => Self::ComposeLiveAfterNmi,
        }
    }
}

impl OamScanoutSource {
    const fn resolve_live_override(self, publish_live_after_nmi: bool) -> Self {
        if publish_live_after_nmi {
            Self::ComposeLiveAfterNmi
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum DialogueOamPublicationPhase {
    #[default]
    Idle,
    PublishedShadow,
    Completed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogueOamCpuBoundary {
    Ordinary,
    // Module0E authors sprite OAM before Text_Render. When state 3 accepts the
    // end-message command and enters state 4, the leading NMI's active display
    // still owns entry OAM, while that NMI's DMA commits live OAM for the next
    // display. Keep this a scanout boundary rather than delaying the DMA.
    MessageFinishedAfterLeadingNmi,
}

const fn dialogue_oam_cpu_boundary(
    main_module: u8,
    submodule: u8,
    entry_text_render_state: u8,
    exit_text_render_state: u8,
) -> DialogueOamCpuBoundary {
    if main_module == 14
        && submodule == 2
        && entry_text_render_state == 3
        && exit_text_render_state == 4
    {
        DialogueOamCpuBoundary::MessageFinishedAfterLeadingNmi
    } else {
        DialogueOamCpuBoundary::Ordinary
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ObjScanoutGenerations {
    oam: OamScanoutSource,
    link_obj: GraphicsDmaGeneration,
    link_obj_sources: GraphicsDmaGeneration,
}

impl ObjScanoutGenerations {
    const fn coherent(generation: GraphicsDmaGeneration) -> Self {
        Self {
            oam: match generation {
                GraphicsDmaGeneration::HostBoundaryBeforeMain => {
                    OamScanoutSource::RetainCapturedBeforeNmi
                }
                GraphicsDmaGeneration::LiveAfterMain => OamScanoutSource::ComposeLiveAfterNmi,
            },
            link_obj: generation,
            link_obj_sources: generation,
        }
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
        ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x14 | 0x22 } => {
            GraphicsDmaGeneration::LiveAfterMain
        }
        _ => GraphicsDmaGeneration::HostBoundaryBeforeMain,
    };
    let oam = match continuation {
        ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x14 } => {
            OamScanoutSource::ComposeLivePlayerOamAfterMain
        }
        ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { .. } => {
            OamScanoutSource::ComposePublishedShadowDma
        }
        ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. } => {
            OamScanoutSource::ComposeLiveAfterNmi
        }
    };
    ObjScanoutGenerations {
        oam,
        link_obj,
        link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum AnimatedBgScanoutGeneration {
    #[default]
    HostBoundaryBeforeNmi,
    LiveAfterNmi,
}

impl AnimatedBgScanoutGeneration {
    const fn resolve_live_override(self, publish_live_after_nmi: bool) -> Self {
        if publish_live_after_nmi {
            Self::LiveAfterNmi
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GraphicsDmaPlan {
    oam_operands: GraphicsDmaGeneration,
    oam_scanout: OamScanoutSource,
    link_obj_scanout: GraphicsDmaGeneration,
    link_obj_operands: GraphicsDmaGeneration,
    animated_bg_operands: GraphicsDmaGeneration,
    animated_bg_scanout: AnimatedBgScanoutGeneration,
}

const fn rom_graphics_dma_plan(main_module: u8, submodule: u8) -> GraphicsDmaPlan {
    // These phase rules describe where the hardware NMI falls relative to the
    // native main-thread slice. OAM, Link OBJ CHR, and animated-BG DMA do not
    // always use the same generation, so keep the domains explicit while
    // deriving the complete plan in one place.
    let dungeon_entrance_nmi_precedes_main = main_module == 0x11 && submodule == 7;
    // Both intra-room stair directions run their Link movement/animation
    // update after the vblank that uploads OBJ CHR. The source words authored
    // by that main-thread work therefore belong to the following NMI.
    let dungeon_intra_room_stairs_nmi_precedes_link_animation =
        main_module == 7 && matches!(submodule, 8 | 0x10);
    let dungeon_spiral_stairs_oam_nmi_precedes_main = main_module == 7 && submodule == 0x0e;
    // Module0E's dungeon-map handler authors its OAM shadow during main. The
    // trailing NMI publishes that shadow for the following scanout, so the
    // active map frame still displays the host-boundary OAM generation.
    let dungeon_map_oam_scanout_uses_host_boundary = main_module == 14 && submodule == 3;
    let dungeon_main_nmi_precedes_main = main_module == 7 && submodule == 0;
    // Shutter control runs after the frame's leading OAM DMA. Its CPU slice can
    // prepare Link's next shadow while the independently scheduled Link-CHR
    // upload remains on the ordinary live generation.
    let dungeon_shutter_oam_nmi_precedes_main = main_module == 7 && submodule == 5;
    let player_link_obj_scanout_uses_host_boundary = (submodule == 0
        && matches!(main_module, 9 | 11))
        || (main_module == 9 && matches!(submodule, 1 | 6..=8 | 0x0a));
    // Module09_LoadAuxGFX suspends its caller suffix, but the intervening NMI
    // still completes the OAM-shadow DMA before active display. Keep Link's
    // animation sources on the host-boundary generation while allowing that
    // independently published OAM image to become visible.
    let player_oam_scanout_uses_host_boundary =
        player_link_obj_scanout_uses_host_boundary && !(main_module == 9 && submodule == 1);
    let oam_scanout_uses_host_boundary = dungeon_entrance_nmi_precedes_main
        || player_oam_scanout_uses_host_boundary
        || dungeon_intra_room_stairs_nmi_precedes_link_animation
        || dungeon_spiral_stairs_oam_nmi_precedes_main
        || dungeon_map_oam_scanout_uses_host_boundary;
    let animated_bg_nmi_precedes_scanout =
        dungeon_entrance_nmi_precedes_main || dungeon_main_nmi_precedes_main;

    GraphicsDmaPlan {
        oam_operands: if dungeon_main_nmi_precedes_main || dungeon_shutter_oam_nmi_precedes_main {
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
            || dungeon_intra_room_stairs_nmi_precedes_link_animation
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
    if frame.main_module == 7 && frame.submodule == 2 && frame.subsubmodule == 8 {
        // Each supertile-scroll iteration authors the next shadow OAM after
        // the active scanout's DMA. Display the shadow published by the prior
        // iteration while BG scroll registers use the current step.
        plan.oam_scanout = OamScanoutSource::ComposePublishedShadowDma;
    } else if rom_dungeon_transition_oam_scanout_uses_host_boundary(
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
    // The entry slice and the first steady slice straddle different sides of
    // the Link-animation update.  Once subsubmodule $02 begins, main authors
    // the next Link OBJ source words after the NMI has already consumed the
    // host-boundary generation.  Unchanged animation frames make the two
    // generations look identical, so keep this keyed to the actual phase
    // instead of the visible four-frame animation cadence.
    let dungeon_spiral_stairs_nmi_precedes_link_animation =
        exit.main_module == 7 && exit.submodule == 0x0e && exit.subsubmodule >= 2;
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
    {
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    } else {
        exit_operands
    }
}

const fn dungeon_supertile_entry_uses_mixed_obj_scanout(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
) -> bool {
    entry.main_module == 7
        && entry.submodule == 0
        && exit.main_module == 7
        && exit.submodule == 2
        && exit.subsubmodule == 0
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

const fn oam_scanout_across_main(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
    entry_scanout: OamScanoutSource,
    screen_transition: u8,
) -> OamScanoutSource {
    // Module 7 enters Dungeon_PrepExitWithSpotlight after the leading NMI has
    // consumed the host-boundary OAM shadow. The atomic native main slice then
    // advances to module $0f, but its newly authored shadow belongs to the
    // following NMI. Publish the exact shadow sampled at entry for this active
    // scanout; once module $0f advances to submodule 1, the ordinary live
    // override publishes the next generation.
    let dungeon_exit_spotlight_publishes_entry_shadow = entry.main_module == 7
        && entry.submodule == 0
        && exit.main_module == 0x0f
        && exit.submodule == 0;
    // Item-receipt dismissal similarly crosses the leading NMI after its OAM
    // shadow has hidden the receipt sprites, before the main slice advances to
    // the dungeon submodule-$0a handoff.
    let dungeon_submodule_handoff_publishes_entry_shadow = entry.main_module == 7
        && entry.submodule == 0
        && exit.main_module == 7
        && matches!(exit.submodule, 1 | 2 | 4 | 5 | 0x0a | 0x0e);
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
    let dungeon_supertile_state3_publishes_entry_shadow = entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 2
        && exit.main_module == 7
        && exit.submodule == 2
        && exit.subsubmodule == 3
        && screen_transition == 0;
    if dungeon_exit_spotlight_publishes_entry_shadow
        || dungeon_submodule_handoff_publishes_entry_shadow
        || dungeon_dialogue_entry_publishes_entry_shadow
        || dungeon_subtile_scroll_publishes_entry_shadow
        || dungeon_supertile_scroll_publishes_entry_shadow
        || dungeon_supertile_state3_publishes_entry_shadow
    {
        OamScanoutSource::ComposePublishedShadowDma
    } else {
        entry_scanout
    }
}

const fn link_obj_scanout_across_main(
    entry: crate::game_state::FrameState,
    exit: crate::game_state::FrameState,
    entry_scanout: GraphicsDmaGeneration,
    _screen_transition: u8,
) -> GraphicsDmaGeneration {
    // The subtile landing tail enters room-load/shutter control after the
    // leading NMI. That scanout keeps the Link tiles resident at host entry,
    // even though the independently scheduled animated-BG upload is live.
    if entry.main_module == 7
        && entry.submodule == 1
        && matches!(entry.subsubmodule, 3..=7)
        && exit.main_module == 7
        && exit.submodule == 5
    {
        return GraphicsDmaGeneration::HostBoundaryBeforeMain;
    }
    // Entering a supertile transition happens after the leading NMI has
    // already uploaded Link's host-boundary source words. The state-0 CPU
    // slice may select the next animation sources, but they belong to the
    // following scanout.
    if dungeon_supertile_entry_uses_mixed_obj_scanout(entry, exit) {
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
    } else if entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 2
        && exit.main_module == 7
        && exit.submodule == 2
        && exit.subsubmodule == 3
    {
        GraphicsDmaGeneration::HostBoundaryBeforeMain
    } else {
        entry_scanout
    }
}

fn dialogue_oam_scanout_transition(
    phase: DialogueOamPublicationPhase,
    module_scanout: OamScanoutSource,
    publish_dialogue_completion_shadow: bool,
    published_shadow_dma: Option<&[u16]>,
    resident_ppu_oam: &[u16],
) -> (OamScanoutSource, DialogueOamPublicationPhase) {
    if !publish_dialogue_completion_shadow {
        return (module_scanout, DialogueOamPublicationPhase::Idle);
    }
    match phase {
        DialogueOamPublicationPhase::Idle
            if published_shadow_dma.is_some_and(|published| published != resident_ppu_oam) =>
        {
            (
                OamScanoutSource::ComposePublishedShadowDma,
                DialogueOamPublicationPhase::PublishedShadow,
            )
        }
        DialogueOamPublicationPhase::PublishedShadow => {
            (module_scanout, DialogueOamPublicationPhase::Completed)
        }
        phase => (module_scanout, phase),
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

fn oam_operands_for_nmi(
    module_operands: GraphicsDmaGeneration,
    publication_phase: DialogueOamPublicationPhase,
) -> GraphicsDmaGeneration {
    if publication_phase == DialogueOamPublicationPhase::PublishedShadow {
        GraphicsDmaGeneration::LiveAfterMain
    } else {
        module_operands
    }
}

const fn oam_scanout_for_cpu_boundary(
    module_scanout: OamScanoutSource,
    cpu_boundary: DialogueOamCpuBoundary,
) -> OamScanoutSource {
    if matches!(
        cpu_boundary,
        DialogueOamCpuBoundary::MessageFinishedAfterLeadingNmi
    ) {
        OamScanoutSource::RetainCapturedBeforeNmi
    } else {
        module_scanout
    }
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

const fn rom_overworld_entry_to_submodule6_publishes_live_animated_bg(
    published: crate::game_state::FrameState,
    captured: crate::game_state::FrameState,
) -> bool {
    // Module09 submodule 5 completes the overlay load, and the following NMI
    // uploads the animated background used by the first submodule-6 scanout.
    // Identify that CPU transition directly instead of inferring it from a
    // BG1-only rain-scroll delta; once scroll publication is coherent, that
    // delta legitimately disappears while the animated DMA boundary remains.
    published.main_module == 9
        && published.submodule == 5
        && captured.main_module == 9
        && captured.submodule == 6
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
    entry.main_module == 7
        && entry.submodule == 1
        && matches!(entry.subsubmodule, 3..=7)
        && captured.main_module == 7
        && captured.submodule == 5
}

const fn rom_dungeon_supertile_filter_entry_publishes_live_animated_bg(
    entry: crate::game_state::FrameState,
    captured: crate::game_state::FrameState,
) -> bool {
    // State 7 finishes the room palette/filter call and reaches vblank before
    // the first state-8 scroll iteration. That NMI's animated-BG transfer has
    // completed by active scanout: Snes9x displays the post-NMI dungeon CHR
    // while OAM and the scroll iteration retain their independent cadence.
    entry.main_module == 7
        && entry.submodule == 2
        && entry.subsubmodule == 7
        && captured.main_module == 7
        && captured.submodule == 2
        && captured.subsubmodule == 8
}

const fn rom_dungeon_supertile_filter_return_resumes_first_scroll_after_nmi(
    entry: crate::game_state::FrameState,
    filtered: crate::game_state::FrameState,
    room: u8,
) -> bool {
    // Most room transitions finish state 7 before vblank, emit the first
    // state-8 image at that NMI, and then execute the first scroll iteration
    // before the host call returns. Rooms $71/$72 have separately measured
    // interrupted-filter continuations and must keep their scheduler path.
    rom_dungeon_supertile_filter_entry_publishes_live_animated_bg(entry, filtered)
        && !matches!(room, 0x71 | 0x72)
}

const fn rom_dungeon_supertile_scroll_runs_after_leading_nmi(
    frame: crate::game_state::FrameState,
    room: u8,
) -> bool {
    // Once state 8 is active, vblank publishes the scroll authored by the
    // preceding iteration and the CPU then prepares the next 4-pixel step.
    // Rooms $71/$72 use their explicit interrupted-return scheduler instead.
    frame.main_module == 7
        && frame.submodule == 2
        && frame.subsubmodule == 8
        && !matches!(room, 0x71 | 0x72)
}

const fn rom_spiral_stairs_suspended_animated_bg_source_address(
    frame: crate::game_state::FrameState,
    host_main_prefix_did_not_advance: bool,
    countdown: u16,
    current_source: usize,
) -> Option<usize> {
    const FIRST_SOURCE: usize = 0xa680;
    const SOURCE_CYCLE_BYTES: usize = 0x0c00;
    if frame.main_module == 7
        && frame.submodule == 0x0e
        && host_main_prefix_did_not_advance
        && countdown == 1
        && current_source >= FIRST_SOURCE
        && current_source < FIRST_SOURCE + SOURCE_CYCLE_BYTES
    {
        Some(FIRST_SOURCE + (current_source - FIRST_SOURCE + 0x400) % SOURCE_CYCLE_BYTES)
    } else {
        None
    }
}

const fn rom_overworld_transition_half_color_is_live(
    snapshot_main_module: u8,
    snapshot_submodule: u8,
    live_main_module: u8,
    live_submodule: u8,
    snapshot_half_color: bool,
    live_half_color: bool,
) -> bool {
    // On the rainy screen-$2b transition, instrumented Snes9x records
    // WritePpuRegisters changing CGADSUB from $72 to $32 at V=257. The active
    // scanout therefore uses the post-NMI half-color bit even though its VRAM
    // and remaining controls retain their existing publication cadence.
    snapshot_main_module == 9
        && live_main_module == 9
        && live_submodule == 3
        // Depending on whether capture owns the interrupted ROM call or its
        // returned caller, the immutable CPU generation is either the
        // preceding transition step or submodule 3 itself.
        && matches!(snapshot_submodule, 2 | 3)
        && snapshot_half_color != live_half_color
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
        // item ID. Both $5b and $5c cross four NMI boundaries; keep the
        // separately packed $5d path immediate until its ROM cost is measured.
        0x5b | 0x5c => ITEM_RECEIPT_STANDARD_ANIMATED_GFX_NMI_SLICES,
        _ => 0,
    }
}

const SPOTLIGHT_ITERATION_SUFFIX_NMI_SLICES: u8 = 1;
// After each active dungeon-exit circle build, the ROM spends one host frame
// returning through the caller and main-loop suffix before beginning the next
// interruptible build.
const DUNGEON_EXIT_SPOTLIGHT_INTER_ITERATION_HOLD_FRAMES: u8 = 1;
const PRE_OVERWORLD_PROPERTIES_NMI_SLICES: u8 = 40;
const PRE_OVERWORLD_OVERLAYS_NMI_SLICES: u8 = 6;
const WORLD_MAP_LIGHT_LOAD_NMI_SLICES: u8 = 5;
// Attract_DramatizeWorldMap enters the tilemap erase immediately after
// vblank, but the clear crosses the next scanout boundary before the caller
// can advance the attract sequence.
const ATTRACT_WORLD_MAP_EXIT_NMI_SLICES: u8 = 1;
const OVERWORLD_SPRITE_RECORD_TIMING_UNITS: usize = 3;
const OVERWORLD_SPRITE_RELOAD_SAME_FRAME_BUDGET_UNITS: usize = 39;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OverworldAuxGraphicsWorkload {
    background_packs_to_decompress: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OverworldAuxGraphicsTiming {
    load_nmi_slices: u8,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OverworldMapGraphicsWorkload {
    map32_definition_changes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OverworldMapAndSpriteGraphicsTiming {
    quadrant_load_nmi_slices: u8,
    screen_map_and_sprite_gfx_tail_nmi_slices: u8,
}

const fn overworld_map_and_sprite_graphics_timing(
    workload: OverworldMapGraphicsWorkload,
) -> OverworldMapAndSpriteGraphicsTiming {
    // The four quadrants always expand 1,024 map32 cells. The expensive branch
    // reloads twelve map16 definition bytes whenever the aligned definition
    // changes. PC/V-counter traces measure 670 changes on screen $1b as 13
    // boundaries and 796 changes on screen $2b as 14. Keep the invariant in
    // work units so later screens refine the calibration without route IDs.
    OverworldMapAndSpriteGraphicsTiming {
        quadrant_load_nmi_slices: 8 + (workload.map32_definition_changes / 128) as u8,
        screen_map_and_sprite_gfx_tail_nmi_slices: 4,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OverworldSpriteReloadWorkload {
    sprite_records: usize,
    in_bounds_proximity_checks: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverworldSpriteReloadEntryPhase {
    OrdinaryModuleIteration,
    VblankEdgeAfterGraphicsTail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreMainNmiResume {
    OverworldAuxGraphicsReturn,
    OverworldSpriteReloadReturn {
        scanout: OverworldSpriteReloadResumeScanout,
    },
    DungeonSupertileQuadrantBuildReturn,
    DungeonSupertileQuadrantBuildPublishedReturn,
    DungeonSupertileQuadrantUploads,
}

/// Caller suffixes that resume before the next fresh module iteration.
///
/// These continuations are mutually exclusive on the 65816 call stack. Keep
/// that invariant explicit instead of representing one program counter with
/// several booleans that could all be true at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreMainCallerContinuation {
    DialogueVwfReturn,
    FileSelectCheckerboardUpload,
    NamePlayerTilemapUpload,
    SpiralStairsSecondPaletteFilter,
    SpiralStairsSecondGrayscalePaletteFilter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NmiPhase {
    BeforeNmi,
    AfterNmi,
}

impl NmiPhase {
    const fn return_bg_scroll_generation(self) -> DisplayBgScrollGeneration {
        match self {
            Self::BeforeNmi => DisplayBgScrollGeneration::ComposeLiveAfterNmi,
            Self::AfterNmi => DisplayBgScrollGeneration::RetainCapturedBeforeNmi,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PreMainNmiScanoutGenerations {
    publication: DisplaySnapshotPublication,
    vram: DisplayVramGeneration,
    animated_bg: Option<AnimatedBgScanoutGeneration>,
    bg_scroll: DisplayBgScrollGeneration,
    obj: Option<ObjScanoutGenerations>,
}

impl PreMainNmiResume {
    const fn scanout_generations(self) -> PreMainNmiScanoutGenerations {
        match self {
            Self::OverworldAuxGraphicsReturn => PreMainNmiScanoutGenerations {
                publication: DisplaySnapshotPublication::PublishCaptured,
                vram: DisplayVramGeneration::ComposeLiveAfterNmi,
                animated_bg: None,
                bg_scroll: DisplayBgScrollGeneration::ComposeLiveAfterNmi,
                obj: None,
            },
            Self::OverworldSpriteReloadReturn { scanout } => scanout.generations(),
            Self::DungeonSupertileQuadrantBuildReturn
            | Self::DungeonSupertileQuadrantBuildPublishedReturn
            | Self::DungeonSupertileQuadrantUploads => PreMainNmiScanoutGenerations {
                publication: if matches!(self, Self::DungeonSupertileQuadrantBuildPublishedReturn) {
                    DisplaySnapshotPublication::RetainPublished
                } else {
                    DisplaySnapshotPublication::PublishCaptured
                },
                vram: if matches!(
                    self,
                    Self::DungeonSupertileQuadrantBuildReturn
                        | Self::DungeonSupertileQuadrantBuildPublishedReturn
                ) {
                    // The resumed state-5 build reaches VRAM during this NMI,
                    // after the active scanout has already selected its
                    // tilemap generation. The following continuation captures
                    // the newly uploaded quadrant for the next image.
                    DisplayVramGeneration::RetainCapturedBeforeNmi
                } else {
                    DisplayVramGeneration::ComposeLiveAfterNmi
                },
                // This continuation captures before a leading hardware NMI.
                // Its animated-CHR DMA and BG scroll-register writes therefore
                // belong to the active scanout, unlike the ordinary
                // main-then-next-NMI cadence.
                animated_bg: Some(AnimatedBgScanoutGeneration::LiveAfterNmi),
                bg_scroll: DisplayBgScrollGeneration::ComposeLiveAfterNmi,
                obj: Some(ObjScanoutGenerations {
                    oam: if matches!(
                        self,
                        Self::DungeonSupertileQuadrantBuildReturn
                            | Self::DungeonSupertileQuadrantBuildPublishedReturn
                    ) {
                        // Main has not yet reached state 6's sprite staging at
                        // this leading boundary. Keep the populated resident
                        // table instead of the freshly cleared shadow.
                        OamScanoutSource::RetainResidentPpuOam
                    } else {
                        OamScanoutSource::ComposeLiveAfterNmi
                    },
                    // Link's CHR DMA runs in the same leading NMI as the
                    // animated-BG transfer. The resumed main thread does not
                    // make that upload belong to the following scanout.
                    link_obj: GraphicsDmaGeneration::LiveAfterMain,
                    link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
                }),
            },
        }
    }

    const fn continues_after_main(self, frame: crate::game_state::FrameState) -> bool {
        matches!(
            self,
            Self::DungeonSupertileQuadrantBuildReturn
                | Self::DungeonSupertileQuadrantBuildPublishedReturn
                | Self::DungeonSupertileQuadrantUploads
        ) && frame.main_module == 7
            && ((frame.submodule == 2 && matches!(frame.subsubmodule, 5..=15))
                || (frame.submodule == 0x0e && matches!(frame.subsubmodule, 8..=15)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverworldSpriteReloadBg1Generation {
    RetainBeforePrepublishedRain,
    ComposeAtTransitionReturn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverworldSpriteReloadResumeScanout {
    ByReturnPhase(NmiPhase),
    CpuSliceEntry {
        scroll: BgScrollRegisterScanout,
        bg1_generation: OverworldSpriteReloadBg1Generation,
    },
}

impl OverworldSpriteReloadResumeScanout {
    const fn generations(self) -> PreMainNmiScanoutGenerations {
        match self {
            Self::ByReturnPhase(return_phase) => PreMainNmiScanoutGenerations {
                publication: match return_phase {
                    NmiPhase::BeforeNmi => DisplaySnapshotPublication::PublishCaptured,
                    NmiPhase::AfterNmi => DisplaySnapshotPublication::RetainPublished,
                },
                vram: DisplayVramGeneration::RetainCapturedBeforeNmi,
                animated_bg: None,
                bg_scroll: return_phase.return_bg_scroll_generation(),
                obj: None,
            },
            Self::CpuSliceEntry { scroll, .. } => PreMainNmiScanoutGenerations {
                publication: DisplaySnapshotPublication::PublishCaptured,
                vram: DisplayVramGeneration::ComposeLiveAfterNmi,
                // Animated-BG DMA is still owned by its ordinary NMI cadence.
                // The later 09/05 -> 09/06 boundary publishes the uploaded
                // tiles explicitly when that generation reaches scanout.
                animated_bg: None,
                bg_scroll: DisplayBgScrollGeneration::RetainCpuSliceEntry(scroll),
                obj: None,
            },
        }
    }

    fn complete_transition_return(self, returned: BgScrollRegisterScanout) -> Self {
        let Self::CpuSliceEntry {
            mut scroll,
            bg1_generation,
        } = self
        else {
            return self;
        };
        if bg1_generation == OverworldSpriteReloadBg1Generation::ComposeAtTransitionReturn {
            scroll.offsets[0] = returned.offsets[0];
        }
        scroll.offsets[1] = returned.offsets[1];
        Self::CpuSliceEntry {
            scroll,
            bg1_generation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverworldSpriteReloadResumeBoundary {
    ByReturnPhase(NmiPhase),
    CpuSliceEntryNmiRegisters,
}

impl OverworldSpriteReloadResumeBoundary {
    fn capture_scanout(
        self,
        state: &ZeldaState,
        bg1_generation: OverworldSpriteReloadBg1Generation,
    ) -> OverworldSpriteReloadResumeScanout {
        match self {
            Self::ByReturnPhase(return_phase) => {
                OverworldSpriteReloadResumeScanout::ByReturnPhase(return_phase)
            }
            Self::CpuSliceEntryNmiRegisters => OverworldSpriteReloadResumeScanout::CpuSliceEntry {
                scroll: state.bg_scroll_scanout_from_nmi_register_mirrors(),
                bg1_generation,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OverworldSpriteReloadTiming {
    load_nmi_slices: u8,
    post_return_hold_nmi_slices: u8,
    return_phase: NmiPhase,
    epilogue_phase: NmiPhase,
    resume_boundary: OverworldSpriteReloadResumeBoundary,
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
// Module $09/$20 begins on the following frame and leaves
// overworld_screen_index set to the temporary rain overlay ($9f) while
// LoadOverworldOverlay crosses six NMI boundaries (17785..17790).
const WORLD_MAP_OVERLAY_RELOAD_NMI_SLICES: u8 = 6;
// Module $09/$21 then spends four NMI boundaries converting the restored main
// Map16 page before it publishes INIDISP=0 and advances to fade submodule $22.
const WORLD_MAP_AMBIENT_MAP8_NMI_SLICES: u8 = 4;
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
const DUNGEON_SUPERTILE_FILTERING_RETURN_NMI_SLICES: u8 = 1;
// Module07_0E's room-loader stack is longer than the ordinary supertile path:
// the spiral adjustment, full room parse, auxiliary graphics, custom tile
// attributes, and follower initialization remain interrupted through nineteen
// subsequent host boundaries before the caller can advance to state 4.
const DUNGEON_SPIRAL_ROOM_INITIALIZATION_NMI_SLICES: u8 = 19;
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

const fn rom_dungeon_exit_spotlight_entry_build_crosses_vblank(vertical_center: u16) -> bool {
    // With the maximal 239-row table, the module-15 sub-0 call's first
    // IrisSpotlight_ConfigureTable build is interrupted by vblank: the ROM
    // ticks the frame counter and clears OAM, then finishes the build and
    // writes the submodule on the following frame (Snes9x wram trace:
    // module written run N, submodule=1 written run N+2 on the Link's-house
    // exit). Smaller tables complete inside the entry frame.
    spotlight_table_row_pairs(vertical_center) >= SPOTLIGHT_MAX_TABLE_ROW_PAIRS
}

// IrisSpotlight_ConfigureTable waits for V=192, copies its 448-byte table, and
// only then writes the next radius at $00:f3cf. PC/V-counter traces place the
// write producing $3f after vblank and the write producing $38 before vblank.
// The crossing iteration can therefore return and begin the next main slice
// before the host's trailing NMI instead of consuming a return-only host frame.
const SPOTLIGHT_CLOSE_RADIUS_UPDATE_BEFORE_NMI_MAX: u16 = 0x38;

const fn spotlight_close_next_radius(radius: u16) -> u16 {
    radius.saturating_sub(load_gfx::SPOTLIGHT_RADIUS_STEP)
}

const fn rom_dungeon_exit_spotlight_radius_update_crosses_before_nmi(
    radius: u16,
    vertical_center: u16,
) -> bool {
    // The crossing was measured on long-table closes (the 189-row dungeon
    // landing and the 239-row Link's-house exit). On the castle-entry close's
    // smaller table the oracle keeps the split return through $3f->$38
    // ($067c/$0c00d trace: rad 56 at run 11461, prep at 11462); firing the
    // merge there ended the close one frame early and shifted the whole
    // castle load (video 11461..11477, RNG schedule ±1 at f11561).
    spotlight_table_has_long_nmi_workload(vertical_center)
        && radius > SPOTLIGHT_CLOSE_RADIUS_UPDATE_BEFORE_NMI_MAX
        && spotlight_close_next_radius(radius) <= SPOTLIGHT_CLOSE_RADIUS_UPDATE_BEFORE_NMI_MAX
}

const fn rom_long_close_iteration_prep_returns_with_main(
    radius_after_shrink: u16,
    vertical_center: u16,
) -> bool {
    // Oracle $0c00d/$067c write traces on the maximal-table close (vertical
    // center 238, Link's-house exit): once the interrupted large-radius
    // builds are behind it, each mid-close iteration finishes its table and
    // reaches Main_PrepSpritesForNmi in the SAME host frame as its radius
    // write (radius values 105 down to 63 pair rad+prep on one run), until
    // the $3f->$38 crossing restores the split return. The interrupted
    // builds themselves ($7e/$77, and $70 at the maximal table) still prep
    // on their following return frame.
    spotlight_table_has_long_nmi_workload(vertical_center)
        && radius_after_shrink > SPOTLIGHT_CLOSE_RADIUS_UPDATE_BEFORE_NMI_MAX
        && !rom_dungeon_exit_spotlight_table_needs_entry_slice(radius_after_shrink, vertical_center)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DungeonSupertileTransitionWork {
    RoomLoad,
    AuxiliarySpriteGraphics,
    SpriteConversion,
    RoomLoadCallerResume,
    SpriteConversionCallerResume,
    QuadrantTilemapBuild,
    SpiralRoomInitialization,
    SpiralRoomCallerResume,
    SpiralBgCharacters34,
    SpiralSpriteGraphics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpiralStaircasePaletteTail {
    BuildQuadrantForVram,
    PrepareNextQuadrant,
}

impl DungeonSupertileTransitionWork {
    const fn nmi_slices(self) -> u8 {
        match self {
            Self::RoomLoad => DUNGEON_SUPERTILE_ROOM_LOAD_NMI_SLICES,
            Self::AuxiliarySpriteGraphics => DUNGEON_SUPERTILE_AUX_SPRITE_GFX_NMI_SLICES,
            Self::SpriteConversion => DUNGEON_SUPERTILE_SPRITE_CONVERSION_NMI_SLICES,
            Self::RoomLoadCallerResume | Self::SpriteConversionCallerResume => {
                DUNGEON_SUPERTILE_CALLER_RESUME_NMI_SLICES
            }
            Self::QuadrantTilemapBuild => DUNGEON_SUPERTILE_QUADRANT_TILEMAP_NMI_SLICES,
            Self::SpiralRoomInitialization => DUNGEON_SPIRAL_ROOM_INITIALIZATION_NMI_SLICES,
            Self::SpiralRoomCallerResume => DUNGEON_SUPERTILE_CALLER_RESUME_NMI_SLICES,
            Self::SpiralBgCharacters34 => DUNGEON_SPIRAL_BG_CHARACTERS_34_NMI_SLICES,
            Self::SpiralSpriteGraphics => DUNGEON_SPIRAL_SPRITE_GRAPHICS_NMI_SLICES,
        }
    }

    const fn next_module_resumes_after_pre_main_nmi(self) -> bool {
        matches!(self, Self::SpriteConversionCallerResume)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DungeonFallingEntranceWork {
    RoomAndTilesets,
    SpriteGraphics,
}

impl DungeonFallingEntranceWork {
    const fn nmi_slices(self) -> u8 {
        match self {
            Self::RoomAndTilesets => DUNGEON_FALLING_ENTRANCE_ROOM_LOAD_NMI_SLICES,
            Self::SpriteGraphics => DUNGEON_FALLING_ENTRANCE_SPRITE_GFX_NMI_SLICES,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ItemReceiptCaller {
    AtomicCaller,
    UnclePassage { sprite_slot: u8 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GameCallStatus {
    Returned,
    Suspended,
}

impl GameCallStatus {
    pub(super) const fn is_suspended(self) -> bool {
        matches!(self, Self::Suspended)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ItemReceiptReturn {
    pub(super) ancilla_slot: u8,
    pub(super) item: u8,
    pub(super) chest_position: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DungeonSpriteMainReturn {
    bg2_x: u16,
    bg2_y: u16,
    bg1_x: u16,
    bg1_y: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ItemReceiptGraphicsContinuation {
    /// The translated caller is still atomic. Its suffix has already run, so
    /// the measured graphics delay only owns the ordinary main-loop epilogue.
    CallerAlreadyCompleted { gfx: u8 },
    /// `DecodeAnimatedSpriteTile_variable` interrupted this real ROM call
    /// stack. Resume each typed caller suffix after the graphics routine
    /// returns instead of publishing selected side effects early.
    ResumeUnclePassage {
        receipt: ItemReceiptReturn,
        sprite_slot: u8,
        dungeon: DungeonSpriteMainReturn,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameWorkContinuation {
    FinishAttractWorldMap,
    FinishAttractWorldMapExit,
    FinishWorldMapLightLoad,
    FinishAttractThroneRoom,
    FinishAttractZeldaPrison,
    FinishAttractMaidenWarp,
    FinishAttractEndOfStory,
    FinishDungeonFallingEntrance {
        work: DungeonFallingEntranceWork,
    },
    FinishDungeonSupertileTransition {
        work: DungeonSupertileTransitionWork,
    },
    FinishDungeonSupertileFilteringReturn,
    HoldDungeonSupertileFilteringReturn,
    FinishPreDungeonEntranceLoad,
    FinishPreDungeonSongBankTransfer,
    FinishItemReceiptGraphics {
        continuation: ItemReceiptGraphicsContinuation,
    },
    FinishBigKeyDropGraphics {
        sprite_slot: u8,
        dungeon: DungeonSpriteMainReturn,
    },
    FinishDungeonMapGraphicsPreparation,
    FinishDungeonMapRoomDrawing,
    FinishDungeonMapRecovery,
    FinishDungeonSubtilePaletteFilter,
    FinishSpiralStaircasePaletteFilter {
        tail: SpiralStaircasePaletteTail,
    },
    FinishDungeonExitSpotlightEntry,
    FinishSpotlightIteration {
        iteration: SpotlightIteration,
    },
    FinishPreOverworldProperties {
        overworld_screen: u8,
        animated_tiles: u8,
    },
    FinishPreOverworldOverlays,
    FinishPreOverworldScreenBuild,
    FinishWorldMapExitTilesets,
    FinishWorldMapOverlayReload,
    FinishWorldMapAmbientMap8,
    FinishOverworldAuxGraphics,
    FinishOverworldMapQuadrants {
        screen_map_and_sprite_gfx_tail_nmi_slices: u8,
    },
    FinishOverworldScreenMapAndSpriteGraphicsTail,
    FinishOverworldSpriteReloadTail {
        post_return_hold_nmi_slices: u8,
        return_phase: NmiPhase,
        epilogue_phase: NmiPhase,
        resume_scanout: OverworldSpriteReloadResumeScanout,
    },
    HoldOverworldSpriteReloadReturn,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct GameWorkCompletionPublication {
    bg_scroll: Option<DisplayBgScrollGeneration>,
    obj: Option<ObjScanoutGenerations>,
}

impl GameWorkContinuation {
    const fn completion_publication(
        self,
        cpu_slice_entry: BgScrollRegisterScanout,
    ) -> GameWorkCompletionPublication {
        match self {
            Self::FinishOverworldAuxGraphics | Self::HoldOverworldSpriteReloadReturn => {
                GameWorkCompletionPublication {
                    bg_scroll: Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
                    obj: None,
                }
            }
            Self::FinishOverworldSpriteReloadTail { return_phase, .. } => {
                GameWorkCompletionPublication {
                    bg_scroll: Some(match return_phase {
                        NmiPhase::BeforeNmi => DisplayBgScrollGeneration::ComposeLiveAfterNmi,
                        NmiPhase::AfterNmi => {
                            DisplayBgScrollGeneration::RetainCpuSliceEntry(cpu_slice_entry)
                        }
                    }),
                    // The reload returns during the NMI that begins the next
                    // hardware frame. Its new OAM and overlapping Link/BG CHR
                    // upload therefore belong to the following scanout.
                    obj: Some(ObjScanoutGenerations::coherent(
                        GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    )),
                }
            }
            _ => GameWorkCompletionPublication {
                bg_scroll: None,
                obj: None,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpotlightIterationPhase {
    /// The entry table is still being calculated when the next scanout starts.
    CloseEntryBeforeTablePublication,
    /// The entry table reaches HDMA before the next scanout starts.
    CloseEntryAfterTablePublication,
    /// HDMA consumes one complete table generation for the scanout.
    WholeTable,
    /// The circle calculation finishes early enough for HDMA to consume the
    /// completed table before the remaining display domains publish.
    WholeTableAfterTablePublication,
    /// The close stages a published prefix with newly authored final lines.
    MixedTailAfterReturn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpotlightDirection {
    Opening { completes_goal_transition: bool },
    Closing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SpotlightIteration {
    direction: SpotlightDirection,
    phase: SpotlightIterationPhase,
}

impl SpotlightIteration {
    pub(super) const fn opening(completes_goal_transition: bool) -> Self {
        Self {
            direction: SpotlightDirection::Opening {
                completes_goal_transition,
            },
            phase: SpotlightIterationPhase::WholeTable,
        }
    }

    pub(super) const fn closing(phase: SpotlightIterationPhase) -> Self {
        Self {
            direction: SpotlightDirection::Closing,
            phase,
        }
    }

    pub(super) const fn is_closing(self) -> bool {
        matches!(self.direction, SpotlightDirection::Closing)
    }

    const fn in_flight_publication(self) -> DisplaySnapshotPublication {
        DisplaySnapshotPublication::AdvanceStaged
    }

    const fn completion_publication(self) -> DisplaySnapshotPublication {
        match self.direction {
            SpotlightDirection::Opening {
                completes_goal_transition: true,
            } => DisplaySnapshotPublication::PublishCaptured,
            SpotlightDirection::Opening {
                completes_goal_transition: false,
            } => DisplaySnapshotPublication::AdvanceStaged,
            SpotlightDirection::Closing => self.phase.close_completion_publication(),
        }
    }

    const fn publishes_completed_hdma_table_to_active_scanout(self) -> bool {
        matches!(
            (self.direction, self.phase),
            (
                SpotlightDirection::Closing,
                SpotlightIterationPhase::WholeTable
                    | SpotlightIterationPhase::WholeTableAfterTablePublication
            )
        )
    }

    const fn projects_following_table_tail_on_completion(self) -> bool {
        matches!(
            (self.direction, self.phase),
            (
                SpotlightDirection::Closing,
                SpotlightIterationPhase::WholeTable | SpotlightIterationPhase::MixedTailAfterReturn
            )
        )
    }
}

impl SpotlightIterationPhase {
    const fn for_close_iteration(submodule: u8, radius: u16, vertical_center: u16) -> Self {
        if submodule == 0 {
            if spotlight_table_has_long_nmi_workload(vertical_center) {
                Self::CloseEntryBeforeTablePublication
            } else {
                Self::CloseEntryAfterTablePublication
            }
        } else if !spotlight_table_has_long_nmi_workload(vertical_center) {
            Self::WholeTableAfterTablePublication
        } else if radius != 0
            && spotlight_close_next_radius(radius) <= SPOTLIGHT_CLOSE_RADIUS_UPDATE_BEFORE_NMI_MAX
        {
            // Snes9x PC/V-counter traces show the next circle write reaching
            // HDMA at scanline 221 once the close has reached this CPU phase.
            Self::MixedTailAfterReturn
        } else {
            Self::WholeTable
        }
    }

    const fn close_completion_publication(self) -> DisplaySnapshotPublication {
        match self {
            Self::CloseEntryAfterTablePublication => DisplaySnapshotPublication::PublishCaptured,
            Self::WholeTable | Self::WholeTableAfterTablePublication => {
                DisplaySnapshotPublication::RetainPublished
            }
            Self::CloseEntryBeforeTablePublication | Self::MixedTailAfterReturn => {
                DisplaySnapshotPublication::AdvanceStaged
            }
        }
    }
}

const DUNGEON_SUBTILE_PALETTE_FILTER_RETURN_NMI_SLICES: u8 = 1;

const fn rom_dungeon_landing_wipe_is_active(main_module: u8, submodule: u8) -> bool {
    main_module == 7 && submodule == 15
}

// Cartridge-state sweeps and Snes9x PC/V-counter traces place the exact
// CPU/NMI workload crossover at 184 generated row pairs, symmetrically at
// vertical centers 41/42 and 182/183. Below it, the table and its $80:f427
// goal reset reach the next hardware publication boundary; at and above it,
// the calculation remains on the preceding table generation for one more
// scanout.
const SPOTLIGHT_LONG_NMI_WORKLOAD_MIN_ROW_PAIRS: u16 = 184;
// With the dungeon landing's 189-row workload, Snes9x PC/V-counter traces
// show the opening calculation reaching the scanline-221 HDMA read through
// radius $3f. The following ROM step ($46) misses that consumer, so the tail
// remains on the preceding table generation.
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

const fn dungeon_landing_wipe_return_slices(
    vertical_center: u16,
    completes_goal_transition: bool,
) -> u8 {
    if completes_goal_transition && spotlight_table_has_long_nmi_workload(vertical_center) {
        2
    } else {
        1
    }
}

const fn dungeon_landing_goal_reset_preserves_scanout_prefix(vertical_center: u16) -> bool {
    !spotlight_table_has_long_nmi_workload(vertical_center)
}

const fn rom_spotlight_goal_transition_waits_for_iteration_return(
    main_module: u8,
    submodule: u8,
) -> bool {
    rom_dungeon_landing_wipe_is_active(main_module, submodule)
        || (main_module == 16 && submodule == 1)
}

const fn rom_dialogue_initialization_nmi_slices(
    main_module: u8,
    messaging_module: u8,
    attract_sequence: u8,
) -> u8 {
    if messaging_module != 0 {
        0
    } else if main_module == 14 || (main_module == 20 && matches!(attract_sequence, 3 | 4)) {
        5
    } else {
        0
    }
}

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

pub(super) fn configured_intro_sprite_animation_start_delay() -> u8 {
    env::var("ZELDA3_SNES9X_INTRO_SPRITE_ANIMATION_START_DELAY")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(SNES9X_INTRO_SPRITE_ANIMATION_START_DELAY)
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
const DOOR_DEBRIS_X: usize = 0x728;
const DOOR_DEBRIS_Y: usize = 0x732;
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

pub struct LoadFuncState<'a> {
    p: &'a [u8],
    pos: usize,
}

impl<'a> LoadFuncState<'a> {
    pub fn new(p: &'a [u8]) -> Self {
        Self { p, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.p.len().saturating_sub(self.pos)
    }
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

#[derive(Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DialogueTextScanout {
    vram: Vec<u16>,
    glyph_runs: Vec<Bg3VwfGlyphRun>,
    glyph_run_dialogue_offsets: Vec<u16>,
    dialogue_msg_read_pos: u16,
    dialogue_message_id: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DialogueTextGeneration {
    PublishedDisplay,
    CurrentRenderBuffer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub(crate) struct DialogueScanoutOwnership(u8);

impl DialogueScanoutOwnership {
    const SNAPSHOT: Self = Self(0);
    const FROZEN_SCROLL_COPY: Self = Self(1);
    const COMPLETION_PENDING: Self = Self(2);
    const COMPLETED_SCROLL: Self = Self(3);
    const COMPLETION_STAGED_AFTER_FROZEN: Self = Self(4);
    const COMPLETION_STAGED_AFTER_SNAPSHOT: Self = Self(5);

    const fn is_snapshot(self) -> bool {
        self.0 == Self::SNAPSHOT.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DialogueScrollCompletionTiming {
    AfterReturnBoundary,
    BeforeNextVblank,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BgScrollRegisterScanout {
    offsets: [[u16; 2]; 4],
}

impl BgScrollRegisterScanout {
    fn capture(ppu: &PpuState) -> Self {
        Self {
            offsets: std::array::from_fn(|index| {
                [ppu.bg_layer[index].h_scroll, ppu.bg_layer[index].v_scroll]
            }),
        }
    }

    fn publish_to(self, ppu: &mut PpuState) {
        for (layer, [h_scroll, v_scroll]) in ppu.bg_layer.iter_mut().zip(self.offsets) {
            layer.h_scroll = h_scroll;
            layer.v_scroll = v_scroll;
        }
    }

    fn after_nmi_writes(ppu: &PpuState, register_bytes: [[u8; 4]; 3]) -> Self {
        let mut scanout = Self::capture(ppu);
        let mut previous = ppu.scroll_prev;
        let mut previous2 = ppu.scroll_prev2;
        for (layer, [h_low, h_high, v_low, v_high]) in register_bytes.into_iter().enumerate() {
            for value in [h_low, h_high] {
                scanout.offsets[layer][0] = ((u16::from(value)) << 8)
                    | (u16::from(previous) & 0xf8)
                    | (u16::from(previous2) & 0x07);
                previous = value;
                previous2 = value;
            }
            for value in [v_low, v_high] {
                scanout.offsets[layer][1] = ((u16::from(value)) << 8) | u16::from(previous);
                previous = value;
            }
        }
        scanout
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DisplayVramGeneration {
    #[default]
    ComposeLiveAfterNmi,
    RetainCapturedBeforeNmi,
}

impl DisplayVramGeneration {
    const fn resolve_for_scanout(self, retain_previous_nmi_display_memory: bool) -> Self {
        if retain_previous_nmi_display_memory || matches!(self, Self::RetainCapturedBeforeNmi) {
            Self::RetainCapturedBeforeNmi
        } else {
            Self::ComposeLiveAfterNmi
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DisplaySnapshotPublication {
    #[default]
    /// Publish this capture immediately and discard any staged generation.
    PublishCaptured,
    /// Stage this capture while publishing the generation staged previously.
    AdvanceStaged,
    /// Keep both the currently published and staged generations unchanged.
    RetainPublished,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum DisplayHdmaTableGeneration {
    #[default]
    Captured,
    /// The spotlight circle has been projected before HDMA starts, while OAM,
    /// VRAM, and the remaining display domains still own the older snapshot.
    SpotlightPublishedAheadOfSnapshot { active_table: Vec<u8> },
    /// The CPU is still projecting the new table while HDMA consumes it.
    /// Words that miss their scanline retain the pre-projection generation.
    AttractMapProjectionDuringScanout { before_projection: Vec<u8> },
    /// The spotlight circle builder crosses HDMA at scanline 221. The active
    /// scanout therefore owns the preceding table above that boundary and the
    /// newly projected table from that boundary downward.
    SpotlightProjectionDuringScanout {
        before_projection: [Vec<u8>; 2],
        after_projection: [Vec<u8>; 2],
        live_tail_start: usize,
    },
}

impl DisplayHdmaTableGeneration {
    fn compose_into(&self, ram: &mut [u8]) {
        match self {
            Self::Captured => {}
            Self::SpotlightPublishedAheadOfSnapshot { active_table } => {
                let byte_count = active_table.len().min(ZeldaState::HDMA_DYNAMIC_TABLE_LEN);
                ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + byte_count]
                    .copy_from_slice(&active_table[..byte_count]);
            }
            Self::AttractMapProjectionDuringScanout { before_projection } => {
                for scanline in 0..ATTRACT_MAP_PROJECTION_WORDS {
                    if !attract_map_projection_current_word_is_visible(scanline) {
                        let offset = scanline * 2;
                        ram[HDMA_TABLE_DYNAMIC + offset..HDMA_TABLE_DYNAMIC + offset + 2]
                            .copy_from_slice(&before_projection[offset..offset + 2]);
                    }
                }
            }
            Self::SpotlightProjectionDuringScanout {
                before_projection,
                after_projection,
                live_tail_start,
            } => {
                let byte_start = live_tail_start * 2;
                for ((table_base, before), after) in [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE]
                    .into_iter()
                    .zip(before_projection)
                    .zip(after_projection)
                {
                    let byte_count = before
                        .len()
                        .min(after.len())
                        .min(ZeldaState::HDMA_DYNAMIC_TABLE_LEN);
                    let split = byte_start.min(byte_count);
                    ram[table_base..table_base + split].copy_from_slice(&before[..split]);
                    ram[table_base + split..table_base + byte_count]
                        .copy_from_slice(&after[split..byte_count]);
                }
            }
        }
    }
}

fn spotlight_hdma_tables_from_ram(ram: &[u8]) -> [Vec<u8>; 2] {
    [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE]
        .map(|table_base| ram[table_base..table_base + ZeldaState::HDMA_DYNAMIC_TABLE_LEN].to_vec())
}

#[derive(Clone, Debug)]
struct LiveSpotlightScanout {
    windowsel: u32,
    screen_windowed: [u8; 2],
    hdma_enable_mask: u8,
    dma_channels: [DmaChannel; 2],
    hdma_tables: [Vec<u8>; 2],
}

impl LiveSpotlightScanout {
    fn capture(state: &ZeldaState) -> Self {
        let windowsel = u32::from(state.ram[crate::game_state::constants::W12SEL_COPY])
            | (u32::from(state.ram[crate::game_state::constants::W34SEL_COPY]) << 8)
            | (u32::from(state.ram[crate::game_state::constants::WOBJSEL_COPY]) << 16);
        Self {
            windowsel,
            screen_windowed: [
                state.ram[crate::game_state::constants::TMW_COPY],
                state.ram[crate::game_state::constants::TSW_COPY],
            ],
            hdma_enable_mask: state.ram[crate::game_state::constants::HDMAEN_COPY],
            dma_channels: [state.dma.channel[6], state.dma.channel[7]],
            hdma_tables: spotlight_hdma_tables_from_ram(&state.ram),
        }
    }

    fn compose_hdma_into(&self, ram: &mut [u8], dma: &mut DmaState) {
        ram[crate::game_state::constants::HDMAEN_COPY] = self.hdma_enable_mask;
        dma.channel[6..8].copy_from_slice(&self.dma_channels);
        for (table_base, table) in [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE]
            .into_iter()
            .zip(&self.hdma_tables)
        {
            let byte_count = table.len().min(ZeldaState::HDMA_DYNAMIC_TABLE_LEN);
            ram[table_base..table_base + byte_count].copy_from_slice(&table[..byte_count]);
        }
    }

    fn compose_into(&self, ram: &mut [u8], ppu: &mut PpuState, dma: &mut DmaState) {
        ppu.windowsel = self.windowsel;
        ppu.screen_windowed = self.screen_windowed;
        self.compose_hdma_into(ram, dma);
    }
}

#[derive(Clone, Debug)]
enum SpotlightScanoutGeneration {
    CapturedBeforeNmi,
    ComposeLiveAfterNmi(LiveSpotlightScanout),
}

impl SpotlightScanoutGeneration {
    fn compose_hdma_into(&self, ram: &mut [u8], dma: &mut DmaState) {
        if let Self::ComposeLiveAfterNmi(live) = self {
            live.compose_hdma_into(ram, dma);
        }
    }

    fn compose_into(&self, ram: &mut [u8], ppu: &mut PpuState, dma: &mut DmaState) {
        if let Self::ComposeLiveAfterNmi(live) = self {
            live.compose_into(ram, ppu, dma);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DisplayBgScrollGeneration {
    #[default]
    RetainCapturedBeforeNmi,
    /// A scheduled CPU continuation returned after NMI, so the active scanout
    /// owns the scroll registers from the start of that interrupted host slice
    /// rather than the snapshot captured after its caller suffix completed.
    RetainCpuSliceEntry(BgScrollRegisterScanout),
    ComposeLiveAfterNmi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DisplayedBgScrollSource {
    CapturedBeforeNmi,
    CpuSliceEntry(BgScrollRegisterScanout),
    LiveAfterNmi,
    LiveBg1AfterNmi,
}

impl DisplayedBgScrollSource {
    fn resolve(
        captured_generation: DisplayBgScrollGeneration,
        dungeon_exit_crosses_nmi_boundary: bool,
        publish_live_overworld_bad_weather_scroll: bool,
        attract_map_retains_display_memory: bool,
    ) -> Self {
        if captured_generation == DisplayBgScrollGeneration::ComposeLiveAfterNmi
            || dungeon_exit_crosses_nmi_boundary
            || attract_map_retains_display_memory
        {
            Self::LiveAfterNmi
        } else if publish_live_overworld_bad_weather_scroll {
            Self::LiveBg1AfterNmi
        } else if let DisplayBgScrollGeneration::RetainCpuSliceEntry(scroll) = captured_generation {
            Self::CpuSliceEntry(scroll)
        } else {
            Self::CapturedBeforeNmi
        }
    }

    fn compose_into(self, shown: &mut PpuState, live: &PpuState) {
        match self {
            Self::CapturedBeforeNmi => {}
            Self::CpuSliceEntry(scroll) => scroll.publish_to(shown),
            Self::LiveAfterNmi => {
                for (shown, live) in shown.bg_layer.iter_mut().zip(&live.bg_layer) {
                    shown.h_scroll = live.h_scroll;
                    shown.v_scroll = live.v_scroll;
                }
            }
            Self::LiveBg1AfterNmi => {
                shown.bg_layer[0].h_scroll = live.bg_layer[0].h_scroll;
                shown.bg_layer[0].v_scroll = live.bg_layer[0].v_scroll;
            }
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum DisplayObjGeneration {
    #[default]
    FollowModuleCadence,
    RetainCapturedOam {
        oam: Vec<u16>,
    },
    RetainCapturedMemory {
        oam: Vec<u16>,
        vram: Vec<u16>,
    },
}

impl DisplayObjGeneration {
    fn retained_oam(&self) -> Option<&[u16]> {
        match self {
            Self::FollowModuleCadence => None,
            Self::RetainCapturedOam { oam } | Self::RetainCapturedMemory { oam, .. } => Some(oam),
        }
    }

    fn retained_vram(&self) -> Option<&[u16]> {
        match self {
            Self::FollowModuleCadence | Self::RetainCapturedOam { .. } => None,
            Self::RetainCapturedMemory { vram, .. } => Some(vram),
        }
    }
}

fn retain_captured_oam_for_scanout(
    obj_generation: &DisplayObjGeneration,
    scanout_source: OamScanoutSource,
) -> bool {
    obj_generation.retained_oam().is_some()
        || matches!(
            scanout_source,
            OamScanoutSource::RetainCapturedBeforeNmi | OamScanoutSource::RetainResidentPpuOam
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DisplayPublicationSignals {
    retain_previous_nmi_display_memory: bool,
    module_oam_publication_is_deferred: bool,
    dungeon_exit_crosses_nmi_boundary: bool,
    publish_live_overworld_bad_weather_scroll: bool,
    publish_live_overworld_transition_half_color: bool,
    attract_map_retains_display_memory: bool,
    world_map_fade_display: bool,
    world_map_mode7_brightness_is_early_published: bool,
    dungeon_item_hold_publishes_live_scroll: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DisplayPublicationPlan {
    vram_generation: DisplayVramGeneration,
    compose_live_cgram: bool,
    oam_scanout_source: OamScanoutSource,
    retain_captured_oam: bool,
    link_obj_scanout_generation: GraphicsDmaGeneration,
    link_obj_source_generation: GraphicsDmaGeneration,
    animated_bg_scanout_generation: AnimatedBgScanoutGeneration,
    bg_scroll_source: DisplayedBgScrollSource,
    publish_live_overworld_transition_half_color: bool,
    world_map_fade_display: bool,
    world_map_mode7_brightness_is_early_published: bool,
}

impl DisplayPublicationPlan {
    fn resolve(snapshot: &DisplaySnapshot, signals: DisplayPublicationSignals) -> Self {
        let explicit_oam_generation = matches!(
            snapshot.oam_scanout_source,
            OamScanoutSource::RetainResidentPpuOam
                | OamScanoutSource::ComposePublishedShadowDma
                | OamScanoutSource::ComposeLivePlayerOamAfterMain
        );
        let module_oam_scanout_source =
            if signals.module_oam_publication_is_deferred && !explicit_oam_generation {
                OamScanoutSource::RetainCapturedBeforeNmi
            } else {
                snapshot.oam_scanout_source
            };
        let oam_scanout_source = module_oam_scanout_source
            .resolve_live_override(signals.dungeon_exit_crosses_nmi_boundary);
        let link_obj_scanout_generation = snapshot
            .link_obj_scanout_generation
            .resolve_live_override(signals.dungeon_exit_crosses_nmi_boundary);
        let link_obj_source_generation = snapshot
            .link_obj_source_generation
            .resolve_live_override(signals.dungeon_exit_crosses_nmi_boundary);
        let atomic_item_graphics_return_publishes_live_vram = matches!(
            snapshot.link_obj_scanout_generation,
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        ) && matches!(
            snapshot.link_obj_source_generation,
            GraphicsDmaGeneration::LiveAfterMain
        ) && snapshot.vram_generation == DisplayVramGeneration::ComposeLiveAfterNmi;
        Self {
            vram_generation: if atomic_item_graphics_return_publishes_live_vram {
                DisplayVramGeneration::ComposeLiveAfterNmi
            } else {
                snapshot
                    .vram_generation
                    .resolve_for_scanout(signals.retain_previous_nmi_display_memory)
            },
            compose_live_cgram: !signals.retain_previous_nmi_display_memory,
            oam_scanout_source,
            retain_captured_oam: retain_captured_oam_for_scanout(
                &snapshot.obj_generation,
                oam_scanout_source,
            ),
            link_obj_scanout_generation,
            link_obj_source_generation,
            animated_bg_scanout_generation: snapshot
                .animated_bg_scanout_generation
                .resolve_live_override(signals.publish_live_overworld_bad_weather_scroll),
            bg_scroll_source: if signals.dungeon_item_hold_publishes_live_scroll {
                DisplayedBgScrollSource::LiveAfterNmi
            } else {
                DisplayedBgScrollSource::resolve(
                    snapshot.bg_scroll_generation,
                    signals.dungeon_exit_crosses_nmi_boundary,
                    signals.publish_live_overworld_bad_weather_scroll,
                    signals.attract_map_retains_display_memory,
                )
            },
            publish_live_overworld_transition_half_color: signals
                .publish_live_overworld_transition_half_color,
            world_map_fade_display: signals.world_map_fade_display,
            world_map_mode7_brightness_is_early_published: signals
                .world_map_mode7_brightness_is_early_published,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub(crate) struct DialogueScrollContinuation(u8);

impl DialogueScrollContinuation {
    const IDLE: Self = Self(0);
    const RETURN_ONLY: Self = Self(1);
    const COPY_REMAINING_PIXELS_THEN_RETURN: Self = Self(2);
    const COPY_REMAINING_PIXELS_BEFORE_VBLANK: Self = Self(3);
    const COMPLETION_PENDING_PUBLICATION: Self = Self(4);

    pub(crate) fn begin(completion_timing: DialogueScrollCompletionTiming) -> Self {
        match completion_timing {
            DialogueScrollCompletionTiming::AfterReturnBoundary => {
                Self::COPY_REMAINING_PIXELS_THEN_RETURN
            }
            DialogueScrollCompletionTiming::BeforeNextVblank => {
                Self::COPY_REMAINING_PIXELS_BEFORE_VBLANK
            }
        }
    }

    pub(crate) fn is_idle(self) -> bool {
        self == Self::IDLE
    }

    fn is_copying_remaining_pixels(self) -> bool {
        matches!(
            self,
            Self::COPY_REMAINING_PIXELS_THEN_RETURN | Self::COPY_REMAINING_PIXELS_BEFORE_VBLANK
        )
    }

    fn completion_timing(self) -> DialogueScrollCompletionTiming {
        debug_assert!(self.is_copying_remaining_pixels());
        if self == Self::COPY_REMAINING_PIXELS_BEFORE_VBLANK {
            DialogueScrollCompletionTiming::BeforeNextVblank
        } else {
            DialogueScrollCompletionTiming::AfterReturnBoundary
        }
    }

    fn is_return_only(self) -> bool {
        self == Self::RETURN_ONLY
    }

    fn is_completion_pending_publication(self) -> bool {
        self == Self::COMPLETION_PENDING_PUBLICATION
    }

    fn finish_remaining_pixels(&mut self) {
        debug_assert!(self.is_copying_remaining_pixels());
        *self = match self.completion_timing() {
            DialogueScrollCompletionTiming::AfterReturnBoundary => Self::RETURN_ONLY,
            DialogueScrollCompletionTiming::BeforeNextVblank => {
                Self::COMPLETION_PENDING_PUBLICATION
            }
        };
    }

    fn publish_early_completion(&mut self) {
        debug_assert!(self.is_completion_pending_publication());
        *self = Self::IDLE;
    }

    fn finish_return(&mut self) {
        debug_assert!(self.is_return_only());
        *self = Self::IDLE;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogueScrollPhase {
    Idle,
    CopyingRemainingPixels {
        completion_timing: DialogueScrollCompletionTiming,
    },
    ReturnOnly,
    CompletionPendingPublication,
    CompletionStagedAfterFrozenScanout,
    CompletionStagedAfterSnapshot,
    CompletedScroll,
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
        debug_assert_eq!(publication, DialogueScanoutOwnership::COMPLETED_SCROLL);
        return DialogueScrollPhase::CompletedScroll;
    }
    debug_assert!(publication.is_snapshot());
    DialogueScrollPhase::Idle
}

/// Atomic transition surface for the dialogue scroll's CPU and scanout phases.
///
/// The referenced fields remain separate only because their positions are part
/// of the existing `ZeldaState` bincode layout. Production code must mutate
/// them through this machine so invalid execution/publication combinations
/// cannot be assembled one field at a time.
struct DialogueScrollMachineMut<'a> {
    continuation: &'a mut DialogueScrollContinuation,
    frozen_scanout: &'a mut Option<DialogueTextScanout>,
    publication: &'a mut DialogueScanoutOwnership,
    completion_scanout: &'a mut Option<DialogueTextScanout>,
    staged_completion: &'a mut Option<DialogueTextScanout>,
}

impl DialogueScrollMachineMut<'_> {
    fn phase(&self) -> DialogueScrollPhase {
        dialogue_scroll_phase(
            *self.continuation,
            *self.publication,
            self.frozen_scanout.is_some(),
            self.completion_scanout.is_some(),
            self.staged_completion.is_some(),
        )
    }

    fn begin_scroll(
        &mut self,
        frozen_scanout: DialogueTextScanout,
        completion_timing: DialogueScrollCompletionTiming,
    ) {
        let entry_phase = self.phase();
        assert!(
            matches!(
                entry_phase,
                DialogueScrollPhase::Idle | DialogueScrollPhase::CompletedScroll
            ),
            "dialogue scroll began from invalid phase {entry_phase:?}",
        );
        *self.continuation = DialogueScrollContinuation::begin(completion_timing);
        *self.frozen_scanout = Some(frozen_scanout);
        *self.publication = DialogueScanoutOwnership::FROZEN_SCROLL_COPY;
        *self.completion_scanout = None;
        debug_assert_eq!(
            self.phase(),
            DialogueScrollPhase::CopyingRemainingPixels { completion_timing }
        );
    }

    fn finish_remaining_pixels(&mut self) -> DialogueScrollCompletionTiming {
        let DialogueScrollPhase::CopyingRemainingPixels { completion_timing } = self.phase() else {
            panic!("dialogue scroll copy completed outside its copy phase");
        };
        self.continuation.finish_remaining_pixels();
        *self.publication = match completion_timing {
            DialogueScrollCompletionTiming::AfterReturnBoundary => {
                DialogueScanoutOwnership::FROZEN_SCROLL_COPY
            }
            DialogueScrollCompletionTiming::BeforeNextVblank => {
                DialogueScanoutOwnership::COMPLETION_PENDING
            }
        };
        completion_timing
    }

    fn finish_return(&mut self) {
        debug_assert_eq!(self.phase(), DialogueScrollPhase::ReturnOnly);
        self.continuation.finish_return();
        *self.publication = DialogueScanoutOwnership::SNAPSHOT;
        *self.frozen_scanout = None;
    }

    fn stage_completion_after_return(&mut self, completed_scanout: DialogueTextScanout) {
        debug_assert_eq!(self.phase(), DialogueScrollPhase::Idle);
        *self.staged_completion = Some(completed_scanout);
        *self.publication = DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_SNAPSHOT;
        debug_assert_eq!(
            self.phase(),
            DialogueScrollPhase::CompletionStagedAfterSnapshot
        );
    }

    fn stage_early_completion(&mut self, completed_scanout: DialogueTextScanout) {
        debug_assert_eq!(
            self.phase(),
            DialogueScrollPhase::CompletionPendingPublication
        );
        self.continuation.publish_early_completion();
        *self.staged_completion = Some(completed_scanout);
        *self.publication = DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_FROZEN;
        debug_assert_eq!(
            self.phase(),
            DialogueScrollPhase::CompletionStagedAfterFrozenScanout
        );
    }

    fn advance_display_boundary(&mut self) {
        match self.phase() {
            DialogueScrollPhase::CompletionStagedAfterFrozenScanout
            | DialogueScrollPhase::CompletionStagedAfterSnapshot => {
                *self.completion_scanout = self.staged_completion.take();
                *self.publication = DialogueScanoutOwnership::COMPLETED_SCROLL;
                *self.frozen_scanout = None;
            }
            DialogueScrollPhase::CompletedScroll => {
                *self.completion_scanout = None;
                *self.publication = DialogueScanoutOwnership::SNAPSHOT;
                *self.frozen_scanout = None;
            }
            DialogueScrollPhase::CopyingRemainingPixels { .. }
            | DialogueScrollPhase::ReturnOnly => {
                *self.publication = DialogueScanoutOwnership::FROZEN_SCROLL_COPY;
            }
            DialogueScrollPhase::CompletionPendingPublication => {
                *self.publication = DialogueScanoutOwnership::COMPLETION_PENDING;
            }
            DialogueScrollPhase::Idle => {
                *self.publication = DialogueScanoutOwnership::SNAPSHOT;
                *self.frozen_scanout = None;
            }
        }
    }

    fn restore_transient_after_checkpoint(&mut self, completed_scanout: DialogueTextScanout) {
        if !self.continuation.is_idle()
            || self.completion_scanout.is_some()
            || self.staged_completion.is_some()
        {
            return;
        }
        match *self.publication {
            DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_FROZEN => {
                *self.staged_completion = Some(completed_scanout);
            }
            DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_SNAPSHOT => {
                *self.staged_completion = Some(completed_scanout);
                *self.frozen_scanout = None;
            }
            DialogueScanoutOwnership::COMPLETED_SCROLL => {
                *self.completion_scanout = Some(completed_scanout);
                *self.frozen_scanout = None;
            }
            // Compatibility with checkpoints produced before staged
            // publication received its own serialized ownership value.
            DialogueScanoutOwnership::COMPLETION_PENDING => {
                *self.publication = DialogueScanoutOwnership::COMPLETION_STAGED_AFTER_FROZEN;
                *self.staged_completion = Some(completed_scanout);
            }
            DialogueScanoutOwnership::FROZEN_SCROLL_COPY => {
                *self.publication = DialogueScanoutOwnership::SNAPSHOT;
                *self.frozen_scanout = None;
            }
            DialogueScanoutOwnership::SNAPSHOT => {}
            _ => {
                *self.publication = DialogueScanoutOwnership::SNAPSHOT;
                *self.frozen_scanout = None;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ColorMathRegisterScanout {
    windowsel: u32,
    clip_mode: u8,
    prevent_math_mode: u8,
    add_subscreen: bool,
    subtract_color: bool,
    half_color: bool,
    math_enabled: u8,
    fixed_color: [u8; 3],
    screen_enabled: [u8; 2],
    screen_windowed: [u8; 2],
}

impl ColorMathRegisterScanout {
    fn publish_to(self, ppu: &mut PpuState) {
        ppu.windowsel = self.windowsel;
        ppu.clip_mode = self.clip_mode;
        ppu.prevent_math_mode = self.prevent_math_mode;
        ppu.add_subscreen = self.add_subscreen;
        ppu.subtract_color = self.subtract_color;
        ppu.half_color = self.half_color;
        ppu.math_enabled = self.math_enabled;
        ppu.fixed_color_r = self.fixed_color[0];
        ppu.fixed_color_g = self.fixed_color[1];
        ppu.fixed_color_b = self.fixed_color[2];
        ppu.screen_enabled = self.screen_enabled;
        ppu.screen_windowed = self.screen_windowed;
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
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
    #[serde(default)]
    dialogue_oam_publication_phase: DialogueOamPublicationPhase,
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
    pub wanted_zelda_features: u32,
    pub state_recorder: StateRecorder,
    dialogue_blk_index: usize,
    dialogue_font_blk_index: usize,
    dialogue_flags: u8,
    #[serde(default)]
    #[serde(skip)]
    rom_startup_timing: bool,
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
    // Set on a frame whose ROM main thread runs PAST the next vblank (a lag
    // frame, Bank00 Vector_NMI `LDA $12 : BNE .skip`): the NMI then skips
    // NMI_DoUpdates entirely, so the $7E0800->$2104 OAM DMA does not happen and
    // the scanout keeps the PREVIOUS frame's OAM. Snes9x-verified for the
    // module-0x0F dungeon-exit prep frame (route frame 14661: the bed-sheet
    // ancilla's OAM entries stay displayed one frame after the shadow hid
    // them). Consumed (taken) in nmi_do_updates_from.
    #[serde(skip)]
    rom_lag_frame_skip_oam_dma: bool,
    #[serde(skip)]
    intro_initialization_work_frames_pending: u8,
    #[serde(skip)]
    intro_initialization_reset_obj_control_pending: bool,
    #[serde(skip)]
    rom_reset_frame_delay: u8,
    #[serde(skip)]
    intro_memory_darken_frame_delay: u8,
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
    #[serde(skip)]
    next_display_vram_generation: DisplayVramGeneration,
    #[serde(skip)]
    publish_live_hud_vram_on_next_capture: bool,
    #[serde(skip)]
    next_display_animated_bg_scanout_generation: Option<AnimatedBgScanoutGeneration>,
    #[serde(skip)]
    next_display_bg_scroll_generation: DisplayBgScrollGeneration,
    #[serde(skip)]
    next_display_obj_scanout_generation: Option<ObjScanoutGenerations>,
    #[serde(skip)]
    next_display_obj_memory_generation: Option<DisplayObjGeneration>,
    #[serde(skip)]
    next_display_interrupted_item_receipt_obj_cache: bool,
    #[serde(skip)]
    link_obj_dma_completed_this_frame: bool,
    #[serde(skip)]
    next_display_spotlight_scanout: Option<LiveSpotlightScanout>,
    #[serde(skip)]
    active_display_obj_generation: DisplayObjGeneration,
    /// The room-$72 state-10 quadrant builder returned through one complete
    /// main-loop prefix before the next ordinary module iteration. Its two
    /// room-sprite entries remain four lines ahead until that interrupted
    /// sorted OAM generation retires from the captured PPU table.
    #[serde(skip)]
    room_72_interrupted_main_prefix_oam_offset_active: bool,
    #[serde(skip)]
    next_overworld_sprite_reload_entry_phase: Option<OverworldSpriteReloadEntryPhase>,
    #[serde(skip)]
    joypad_sampled_before_main: bool,
    #[serde(skip)]
    audio_nmi_processed_before_main: bool,
    #[serde(skip)]
    dungeon_landing_wipe_return_slices_remaining: u8,
    #[serde(skip)]
    dungeon_exit_spotlight_table_delay: u8,
    #[serde(skip)]
    dungeon_exit_spotlight_resume_module: bool,
    #[serde(skip)]
    iris_spotlight_goal_transition_pending: bool,
    #[serde(skip)]
    normal_dialogue_initialization_phase: u8,
    #[serde(skip)]
    hud_tilemap_nmi_publication_phase: u8,
    #[serde(skip)]
    intro_poly_upload_delay: u8,
    #[serde(skip)]
    intro_sprite_animation_start_delay: u8,
    #[serde(skip)]
    display_snapshot: Option<Box<DisplaySnapshot>>,
    #[serde(skip)]
    visible_display_snapshot: Option<Box<DisplaySnapshot>>,
    /// OAM evaluated for the most recently rendered scanout. Interrupted CPU
    /// graphics work retains this exact hardware generation while NMI-owned
    /// register domains continue advancing.
    #[serde(skip)]
    last_presented_oam: Option<Vec<u16>>,
    /// OBJ tile data used by the most recently rendered scanout. Some
    /// dungeon-transition states suppress Link's upload while the live PPU
    /// has already advanced to the next source generation.
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
    /// Animated-BG VRAM as it existed when the host entered this frame, before
    /// the frame's NMI could upload a newly selected animation phase.
    #[serde(skip)]
    pre_nmi_animated_bg_scanout: Option<PreNmiAnimatedBgScanout>,
    #[serde(default)]
    nmi_forced_blank_scanlines_pending: u8,
    nmi_forced_blank_from_scanline_pending: Option<u8>,
    #[serde(default)]
    nmi_active_display_blanking_candidate: NmiActiveDisplayBlanking,
    #[serde(skip)]
    active_display_force_blank_event: Option<u8>,
    /// Work performed by the most recent `Sprite_Main` call in this host
    /// frame. Consumers use it only through measured raster timing models.
    #[serde(skip)]
    last_sprite_main_timing_workload: Option<SpriteMainTimingWorkload>,
    spotlight_hdma_reset_prefix: Option<[u16; DUNGEON_LANDING_HDMA_RESET_PREFIX_SCANLINES]>,
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
    #[serde(skip)]
    replay_reload_file_select_stall: u8,
    #[serde(skip)]
    replay_reopened_lamp_prompt: bool,
    ending_coords: sprite::PrepOamCoordsRet,
    #[serde(skip)]
    intro_poly_vram_history: Vec<(u8, Vec<u16>, Vec<u16>)>,
    #[serde(skip)]
    intro_poly_presented_vram: Option<(u8, Vec<u16>)>,
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

#[derive(Clone)]
struct DisplaySnapshot {
    ram: Vec<u8>,
    ppu: PpuState,
    dma: DmaState,
    vram_chr_source: crate::chr_source::VramChrSourceTable,
    vram_chr_preview_source: crate::chr_source::VramChrSourceTable,
    hdma_table_generation: DisplayHdmaTableGeneration,
    vram_generation: DisplayVramGeneration,
    hud_vram_generation: DisplayVramGeneration,
    hud_vram_destination: usize,
    link_obj_scanout_generation: GraphicsDmaGeneration,
    link_obj_source_generation: GraphicsDmaGeneration,
    oam_scanout_source: OamScanoutSource,
    dungeon_item_hold_entry_scanout: bool,
    dungeon_item_hold_entry_bg2_scroll: Option<(u16, u16)>,
    published_shadow_oam_dma: Option<Vec<u16>>,
    room_72_interrupted_main_prefix_oam_offset_active: bool,
    animated_bg_scanout_generation: AnimatedBgScanoutGeneration,
    bg_scroll_generation: DisplayBgScrollGeneration,
    spotlight_scanout_generation: SpotlightScanoutGeneration,
    obj_generation: DisplayObjGeneration,
    interrupted_item_receipt_obj_cache: bool,
    published_bg3_vwf_glyph_runs: Vec<Bg3VwfGlyphRun>,
    published_bg3_vwf_glyph_run_dialogue_offsets: Vec<u16>,
    published_dialogue_msg_read_pos: u16,
    published_dialogue_message_id: u16,
    intro_poly_upload_delay: u8,
    intro_sprite_animation_start_delay: u8,
    rom_reset_frame_delay: u8,
    intro_memory_darken_frame_delay: u8,
    nmi_poly_upload_deferred: u8,
    obj_vram_latch_generation: u64,
    snes9x_poly_scheduler_counter: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PublishedDialogueMetadata {
    glyph_runs: Vec<Bg3VwfGlyphRun>,
    glyph_run_dialogue_offsets: Vec<u16>,
    message_read_position: u16,
    message_id: u16,
}

impl PublishedDialogueMetadata {
    fn from_scanout(scanout: &DialogueTextScanout) -> Self {
        Self {
            glyph_runs: scanout.glyph_runs.clone(),
            glyph_run_dialogue_offsets: scanout.glyph_run_dialogue_offsets.clone(),
            message_read_position: scanout.dialogue_msg_read_pos,
            message_id: scanout.dialogue_message_id,
        }
    }

    fn from_snapshot(snapshot: &DisplaySnapshot) -> Self {
        Self {
            glyph_runs: snapshot.published_bg3_vwf_glyph_runs.clone(),
            glyph_run_dialogue_offsets: snapshot
                .published_bg3_vwf_glyph_run_dialogue_offsets
                .clone(),
            message_read_position: snapshot.published_dialogue_msg_read_pos,
            message_id: snapshot.published_dialogue_message_id,
        }
    }

    fn from_live_state(state: &ZeldaState) -> Self {
        Self {
            glyph_runs: state.published_bg3_vwf_glyph_runs.clone(),
            glyph_run_dialogue_offsets: state.published_bg3_vwf_glyph_run_dialogue_offsets.clone(),
            message_read_position: state.published_dialogue_msg_read_pos,
            message_id: state.published_dialogue_message_id,
        }
    }

    fn replace_in(self, state: &mut ZeldaState) -> Self {
        Self {
            glyph_runs: std::mem::replace(&mut state.published_bg3_vwf_glyph_runs, self.glyph_runs),
            glyph_run_dialogue_offsets: std::mem::replace(
                &mut state.published_bg3_vwf_glyph_run_dialogue_offsets,
                self.glyph_run_dialogue_offsets,
            ),
            message_read_position: std::mem::replace(
                &mut state.published_dialogue_msg_read_pos,
                self.message_read_position,
            ),
            message_id: std::mem::replace(
                &mut state.published_dialogue_message_id,
                self.message_id,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CaptureDisplayDiagnostics {
    attract_timeline: bool,
    frame_boundary: bool,
}

impl CaptureDisplayDiagnostics {
    fn from_env() -> Self {
        Self {
            attract_timeline: env::var_os("ZELDA3_DEBUG_ATTRACT_TIMELINE").is_some(),
            frame_boundary: env::var_os("ZELDA3_DEBUG_FRAME_BOUNDARY").is_some(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DisplayDiagnostics {
    capture: CaptureDisplayDiagnostics,
    display_oam: bool,
    nmi_latch: bool,
    scroll_retain: bool,
}

impl DisplayDiagnostics {
    fn from_env() -> Self {
        Self {
            capture: CaptureDisplayDiagnostics::from_env(),
            display_oam: env::var_os("ZELDA3_DEBUG_DISPLAY_OAM").is_some(),
            nmi_latch: env::var_os("ZELDA3_DEBUG_NMI_LATCH").is_some(),
            scroll_retain: env::var_os("ZELDA3_DEBUG_SCROLL_RETAIN").is_some(),
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

#[derive(Clone)]
struct PreMainAnimatedTileDma {
    source_address: usize,
    destination_address: usize,
    data: Vec<u8>,
}

#[derive(Clone)]
struct PreNmiAnimatedBgScanout {
    destination_address: usize,
    vram: Vec<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RetainedVramRegion {
    destination: usize,
    words: Vec<u16>,
}

impl RetainedVramRegion {
    fn capture(vram: &[u16], destination: usize, word_count: usize) -> Option<Self> {
        let end = destination.checked_add(word_count)?;
        Some(Self {
            destination,
            words: vram.get(destination..end)?.to_vec(),
        })
    }

    fn publish_to(&self, vram: &mut [u16]) {
        let Some(end) = self.destination.checked_add(self.words.len()) else {
            return;
        };
        if let Some(destination) = vram.get_mut(self.destination..end) {
            destination.copy_from_slice(&self.words);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct NmiCopyPacketScanout {
    words: Vec<(usize, u16)>,
}

impl NmiCopyPacketScanout {
    fn capture(vram: &[u16], packet_bytes: &[u8]) -> Self {
        let mut words = Vec::new();
        for packet in nmi::nmi_vram_copy_packets(packet_bytes) {
            let word_count = packet.data.len().div_ceil(2);
            let stride = match packet.direction {
                nmi::NmiVramCopyDirection::Horizontal => 1,
                nmi::NmiVramCopyDirection::Vertical => 32,
            };
            for index in 0..word_count {
                let address = packet.destination + index * stride;
                if let Some(&value) = vram.get(address) {
                    words.push((address, value));
                }
            }
        }
        Self { words }
    }

    fn publish_to(&self, vram: &mut [u16]) {
        for &(address, value) in &self.words {
            if let Some(word) = vram.get_mut(address) {
                *word = value;
            }
        }
    }
}

#[derive(Clone)]
struct PreMainGraphicsDma {
    entry_frame: crate::game_state::FrameState,
    entry_plan: GraphicsDmaPlan,
    entry_dialogue_text_render_state: u8,
    entry_link_handler_state: u8,
    animated_tile: Option<PreMainAnimatedTileDma>,
    link_operands: PreMainLinkDmaOperands,
    link_obj_vram: Vec<u16>,
    oam_shadow: Vec<u8>,
}

const LINK_DMA_EXPANDED_HIGH_PLANES_START: usize = 0xbd40;
const LINK_DMA_EXPANDED_HIGH_PLANES_LEN: usize = 0x80;
const LINK_DMA_EXPANDED_HIGH_PLANES_HALF_LEN: usize = LINK_DMA_EXPANDED_HIGH_PLANES_LEN / 2;

#[derive(Clone, Copy)]
struct PreMainLinkDmaOperands {
    sources: LinkDmaSources,
    expanded_high_planes: [u8; LINK_DMA_EXPANDED_HIGH_PLANES_LEN],
}

impl PreMainLinkDmaOperands {
    fn capture(ram: &[u8]) -> Self {
        let mut expanded_high_planes = [0; LINK_DMA_EXPANDED_HIGH_PLANES_LEN];
        let end = LINK_DMA_EXPANDED_HIGH_PLANES_START + LINK_DMA_EXPANDED_HIGH_PLANES_LEN;
        if let Some(bytes) = ram.get(LINK_DMA_EXPANDED_HIGH_PLANES_START..end) {
            expanded_high_planes.copy_from_slice(bytes);
        }
        Self {
            sources: LinkDmaSources::load_from_ram(ram),
            expanded_high_planes,
        }
    }
}

pub type ZeldaRunFrameFunc = fn(&mut ZeldaState, u16, i32);
pub type ZeldaSyncAllFunc = fn(&mut ZeldaState);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Bg3VwfGlyphRun {
    pub glyph_code: u16,
    pub origin_tile_number: u16,
    pub x: i16,
    pub y: i16,
    pub width: u8,
}

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

fn compose_published_link_oam(oam: &mut [u16], published_shadow_oam: Option<&[u16]>) {
    compose_published_oam_entries(oam, published_shadow_oam, LINK_OAM_ENTRIES);
}

fn supertile_state3_oam_entry_is_on_screen(oam: &[u16], entry: usize) -> bool {
    let high_word = 256 + entry / 8;
    let high_shift = (entry % 8) * 2;
    let x_high = (oam[high_word] >> high_shift) & 1;
    oam_entry_bytes(oam, entry)[1] != 0xf0 && x_high == 0
}

fn compose_visible_published_oam(oam: &mut [u16], published_shadow_oam: Option<&[u16]>) {
    let Some(published) = published_shadow_oam.filter(|published| published.len() == oam.len())
    else {
        return;
    };
    for entry in 0..128 {
        if supertile_state3_oam_entry_is_on_screen(oam, entry)
            && supertile_state3_oam_entry_is_on_screen(published, entry)
        {
            // The sorted shadow contributes the already-evaluated entry body,
            // while the resident high table has independently advanced to the
            // scanout's size/X generation. Copying the packed high pair here
            // would also regress same-slot sprites whose size changed during
            // the transition.
            let start = entry * 2;
            oam[start..start + 2].copy_from_slice(&published[start..start + 2]);
        }
    }
}

fn room_71_supertile_return_retains_link_vram(
    entry: crate::game_state::FrameState,
    following: crate::game_state::FrameState,
    dungeon_room_index: u8,
) -> bool {
    dungeon_room_index == 0x71
        && following.main_module == 7
        && ((following.submodule == 2 && matches!(following.subsubmodule, 13 | 14))
            || (entry.main_module == 7
                && entry.submodule == 2
                && entry.subsubmodule == 15
                && following.submodule == 5))
}

fn room_71_supertile_return_uses_published_link_oam(
    following: crate::game_state::FrameState,
    dungeon_room_index: u8,
) -> bool {
    dungeon_room_index == 0x71
        && following.main_module == 7
        && ((following.submodule == 2 && matches!(following.subsubmodule, 13 | 14))
            || following.submodule == 5)
}

fn room_71_supertile_room_load_uses_composed_obj_cache(
    following: crate::game_state::FrameState,
    dungeon_room_index: u8,
) -> bool {
    dungeon_room_index == 0x71 && following.main_module == 7 && following.submodule == 5
}

fn dungeon_supertile_pre_scroll_oam(
    live_oam: &[u16],
    published_shadow_oam: Option<&[u16]>,
) -> Vec<u16> {
    let mut oam = live_oam.to_vec();
    // LinkOam_Main returns on the host-boundary generation even though the
    // other sprites have already published their live pre-scroll image.
    compose_published_link_oam(&mut oam, published_shadow_oam);
    if published_shadow_oam.is_none_or(|published| published.len() != oam.len()) {
        return oam;
    }
    // The interrupted LinkOam_Main tail projects the two visible body pieces
    // one pixel above the already-authored shadow coordinates on both the
    // return scanout and the first state-8 scanout.
    for entry in [102usize, 107] {
        let xy = &mut oam[entry * 2];
        *xy = xy.wrapping_sub(0x0100);
    }
    oam
}

fn dungeon_supertile_interrupted_filter_oam(
    live_oam: &[u16],
    previously_presented_oam: Option<&[u16]>,
) -> Vec<u16> {
    let mut oam = live_oam.to_vec();
    // Vblank interrupts the filtering return after ordinary sprites have
    // authored their next shadow, but before LinkOam_Main publishes its tail.
    compose_published_link_oam(&mut oam, previously_presented_oam);
    oam
}

fn dungeon_supertile_first_state8_oam(resident_oam: &[u16]) -> Vec<u16> {
    let mut oam = resident_oam.to_vec();
    // The active room sprites finish their interrupted upward motion before
    // the caller reaches the first state-8 scroll iteration.
    for entry in [92usize, 93] {
        oam[entry * 2] = oam[entry * 2].wrapping_sub(0x0100);
    }
    oam
}

fn dungeon_supertile_second_state8_oam(first_state8_oam: &[u16]) -> Vec<u16> {
    let mut oam = first_state8_oam.to_vec();
    for entry in [92usize, 93] {
        oam[entry * 2] = oam[entry * 2].wrapping_sub(0x0200);
    }
    oam
}

fn adjust_room_72_published_sprite_oam(oam: &mut [u16], pixels: u8) {
    for entry in [92usize, 93] {
        oam[entry * 2] = oam[entry * 2].wrapping_sub(u16::from(pixels) << 8);
    }
}

fn adjust_room_72_state13_link_oam(oam: &mut [u16], pixels: u8) {
    for entry in LINK_OAM_ENTRIES {
        oam[entry * 2] = oam[entry * 2].wrapping_add(u16::from(pixels) << 8);
    }
}

fn adjust_room_72_state13_body_oam(oam: &mut [u16], live_oam: &[u16], link_y: u16) {
    for (entry, interrupted_gap) in [(102usize, 17u8), (107, 13)] {
        let [_, live_y] = live_oam[entry * 2].to_le_bytes();
        if (link_y as u8).wrapping_sub(live_y) >= interrupted_gap {
            oam[entry * 2] = oam[entry * 2].wrapping_add(0x0100);
        }
    }
}

fn adjust_room_72_state14_link_oam(oam: &mut [u16]) {
    for entry in [103usize, 110, 111] {
        oam[entry * 2] = oam[entry * 2].wrapping_add(0x0200);
    }
}

fn rebase_room_72_sprite_oam_after_interrupted_main_prefix(oam: &mut [u16]) {
    for entry in [92usize, 93] {
        oam[entry * 2] = oam[entry * 2].wrapping_sub(0x0400);
    }
}

impl ZeldaState {
    pub(crate) fn compatibility_state_len(&self) -> usize {
        self.ram.len()
    }

    fn replay_trace_col(&self, label: &str) {
        let Some(target) = env::var("ZELDA3_REPLAY_TRACE_COL_FRAME")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
        else {
            return;
        };
        if self.frame_ctr_dbg != target {
            return;
        }
        let frame = &self.game_state.frame;
        eprintln!(
            "replay-col frame={} {label} main={} sub={} subsub={} col=0x{:02x},0x{:02x} door=0x{:02x} last=0x{:02x} dlast=0x{:02x} speed=0x{:02x}/0x{:02x} dir=0x{:02x} state=0x{:02x} x=0x{:04x} y=0x{:04x}",
            self.frame_ctr_dbg,
            frame.main_module,
            frame.submodule,
            frame.subsubmodule,
            self.game_state.player.tile_detection.tile_collision_bits_primary(),
            self.game_state.player.tile_detection.tile_collision_bits_secondary(),
            self.game_state.dungeon.doors.door_open_counter_low(),
            self.game_state.player.follower_link.last_direction(),
            self.game_state.player.follower_link.swim_direction_flags(),
            self.game_state.player.follower_link.speed_setting(),
            self.game_state.player.follower_link.speed_modifier(),
            self.game_state.player.follower_link.direction(),
            self.game_state.player.follower_link.handler_state(),
            self.game_state.player.follower_link.x(),
            self.game_state.player.follower_link.y(),
        );
    }

    /// Native↔RAM coherence guard. With `ZELDA3_ASSERT_NATIVE_COHERENT` set, report (or
    /// `=panic` to abort on) any native sub-state that has drifted out of sync with RAM
    /// at this labeled step — the signature of a stale-native-field or RAM-written-
    /// without-native-sync bug. Optionally scope to one frame with
    /// `ZELDA3_ASSERT_COHERENT_FRAME=<n>` to keep the (heavy) check cheap.
    /// True at the labeled checkpoint of the target frame, whether the run started from
    /// frame 0 (`frame_ctr_dbg`) or resumed from a `--load-state` checkpoint
    /// (`replay_frame_counter`, which IS restored while `frame_ctr_dbg` counts from load).
    /// Use for every trace-frame gate so the diagnostics work with checkpoint resume.
    fn trace_frame_matches(&self, target: u32) -> bool {
        self.frame_ctr_dbg == target || self.state_recorder.replay_frame_counter == target
    }

    /// The committed provenance-clean CGRAM mirror for the renderer (the
    /// zero-CGRAM color source; see `zelda3_palette`).
    pub fn cgram_provenance_snapshot(&self) -> zelda3_palette::CgramProvenanceSnapshot {
        self.game_state
            .display
            .palette_provenance
            .0
            .cgram_snapshot()
    }

    /// Audit the mirror's committed CGRAM image (what the renderer substitutes) against the live
    /// PPU CGRAM. Used at render-capture points to catch a stale committed image between upload
    /// commits (the main-vs-shadow audit cannot see it).
    pub fn audit_cgram_mirror(&self, ppu_cgram: &[u16]) -> zelda3_palette::BankAudit {
        self.game_state
            .display
            .palette_provenance
            .0
            .audit_cgram(ppu_cgram)
    }

    /// Serialize the provenance mirror for a checkpoint trailer. The mirror is
    /// `#[serde(skip)]` in the state snapshot (a restore reconstitutes it from the
    /// shadow, tagged `Copied`); a checkpoint that carries these bytes can instead
    /// restore the mirror exactly as-derived — true provenance tags and no live-CGRAM
    /// read at the boundary. See [`Self::restore_palette_mirror_from_bytes`].
    pub fn palette_mirror_snapshot_bytes(&self) -> Vec<u8> {
        bincode::serialize(&self.game_state.display.palette_provenance.0)
            .expect("palette mirror is a fixed-size POD struct and always serializes")
    }

    /// Install a provenance mirror captured by [`Self::palette_mirror_snapshot_bytes`],
    /// overwriting whatever the snapshot restore reconstituted from the shadow.
    pub fn restore_palette_mirror_from_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mirror = bincode::deserialize(bytes).map_err(|e| e.to_string())?;
        self.game_state.display.palette_provenance.0 = mirror;
        Ok(())
    }

    /// Diagnostic: compact per-bank source-tag histogram of the provenance mirror. Confirms a
    /// checkpoint-restored mirror carries its true derivation tags (asset/constant/computed)
    /// rather than an all-`Copied` shadow reconstitution.
    pub fn palette_mirror_tag_histogram(&self) -> String {
        self.game_state
            .display
            .palette_provenance
            .0
            .tag_histogram_line()
    }

    /// Snapshot the palette-provenance mirror at a CGRAM upload (the mirror's
    /// equivalent of `memcpy(cgram, main_palette_buffer)`), and under
    /// `ZELDA3_PALETTE_PROVENANCE_CHECK=1|panic` audit the mirror's main bank
    /// against the WRAM shadow the upload actually read. The audit line is
    /// rate-limited to changes in the (mismatch, unknown) counts.
    pub(crate) fn commit_palette_provenance_cgram(&mut self) {
        self.game_state.display.palette_provenance.0.commit_cgram();
        let Some(mode) = crate::game_state::palette_provenance_check_mode() else {
            return;
        };
        let shadow = &self.ram[crate::game_state::constants::MAIN_PALETTE_BUFFER
            ..crate::game_state::constants::MAIN_PALETTE_BUFFER + 0x200];
        let audit = self
            .game_state
            .display
            .palette_provenance
            .0
            .audit_bank(zelda3_palette::Bank::Main, shadow);
        // Also audit the committed CGRAM image (what the renderer actually consumes) against the
        // live PPU CGRAM. The main-vs-shadow audit alone provably misses a stale CGRAM image (a
        // restore that bulk-loads ppu.cgram without a following upload — see
        // `reconstitute_palette_mirror_from_shadow`). At an upload commit these agree by
        // construction; the audit guards the upload/restore paths against regressing.
        let cgram_audit = self
            .game_state
            .display
            .palette_provenance
            .0
            .audit_cgram(&self.ppu.cgram);
        use std::sync::atomic::{AtomicU64, Ordering};
        static LAST: AtomicU64 = AtomicU64::new(u64::MAX);
        let packed = ((audit.mismatches.len() as u64) << 48)
            | ((audit.unknown.len() as u64) << 32)
            | ((cgram_audit.mismatches.len() as u64) << 16)
            | cgram_audit.unknown.len() as u64;
        if LAST.swap(packed, Ordering::Relaxed) != packed {
            let first_mismatch = audit.mismatches.first().map(|w| {
                format!(
                    " first_mismatch=idx{}:mirror={:04x}:actual={:04x}",
                    w.index,
                    w.mirror.unwrap_or(0),
                    w.actual
                )
            });
            let first_unknown = audit
                .unknown
                .first()
                .map(|w| format!(" first_unknown=idx{}", w.index));
            eprintln!(
                "palette_provenance_coherence frame={} mismatches={} unknown={} cgram_mismatches={} cgram_unknown={}{}{}",
                self.frame_ctr_dbg,
                audit.mismatches.len(),
                audit.unknown.len(),
                cgram_audit.mismatches.len(),
                cgram_audit.unknown.len(),
                first_mismatch.unwrap_or_default(),
                first_unknown.unwrap_or_default(),
            );
        }
        if mode == crate::game_state::ProvenanceCheckMode::Panic
            && (!audit.is_clean() || !cgram_audit.is_clean())
        {
            panic!(
                "palette provenance mirror diverged at frame {} (main: mismatches={} unknown={}; \
                 cgram: mismatches={} unknown={})",
                self.frame_ctr_dbg,
                audit.mismatches.len(),
                audit.unknown.len(),
                cgram_audit.mismatches.len(),
                cgram_audit.unknown.len(),
            );
        }
    }

    fn replay_assert_native_coherent(&self, label: &str) {
        let Ok(mode) = std::env::var("ZELDA3_ASSERT_NATIVE_COHERENT") else {
            return;
        };
        if let Some(frame) = Self::parse_trace_env_u32("ZELDA3_ASSERT_COHERENT_FRAME") {
            if !self.trace_frame_matches(frame) {
                return;
            }
        }
        let mut bad = self.game_state.report_incoherent_with_ram(&self.ram);
        // Some states legitimately diverge mid-frame (gated/mode-reuse projections, the
        // cached-sprite shadow). Pass a comma-separated allow-list in
        // ZELDA3_ASSERT_COHERENT_IGNORE to suppress that baseline so `=panic` aborts only
        // on a genuinely-unexpected drift.
        if let Ok(ignore) = std::env::var("ZELDA3_ASSERT_COHERENT_IGNORE") {
            let ignore: Vec<&str> = ignore.split(',').map(|s| s.trim()).collect();
            bad.retain(|name| !ignore.contains(name));
        }
        if bad.is_empty() {
            return;
        }
        let f = &self.game_state.frame;
        let msg = format!(
            "native-incoherent frame={} m=0x{:02x} sm=0x{:02x} ssm=0x{:02x} after '{label}': {:?}",
            self.frame_ctr_dbg, f.main_module, f.submodule, f.subsubmodule, bad
        );
        if mode == "panic" {
            panic!("{msg}");
        } else {
            eprintln!("{msg}");
        }
    }

    fn replay_trace_ram_watch(&self, label: &str) {
        self.replay_assert_native_coherent(label);
        let Some(target) = Self::parse_trace_env_u32("ZELDA3_REPLAY_RAM_WATCH_FRAME") else {
            return;
        };
        if !self.trace_frame_matches(target) {
            return;
        }
        let watched_addr = Self::parse_trace_env_u32("ZELDA3_REPLAY_RAM_WATCH_ADDR")
            .and_then(|addr| self.ram.get(addr as usize).map(|value| (addr, *value)));
        let frame = &self.game_state.frame;
        eprintln!(
            "ram-watch frame={} {label} fc=0x{:02x} main={} sub={} subsub={} watch={} d340={:02x} d341={:02x} d342={:02x} d343={:02x} d344={:02x} d345={:02x} d346={:02x} d347={:02x} deep=0x{:04x} normal=0x{:04x} inwater=0x{:02x} link=0x{:04x}/0x{:04x} state=0x{:02x}",
            self.frame_ctr_dbg,
            frame.frame_counter,
            frame.main_module,
            frame.submodule,
            frame.subsubmodule,
            watched_addr
                .map(|(addr, value)| format!("0x{addr:05x}=0x{value:02x}"))
                .unwrap_or_else(|| "none".to_string()),
            self.game_state.player.follower_link.swim_direction_flags(),
            self.game_state.player.tile_detection.deepwater() as u8,
            self.game_state.player.tile_detection.deepwater_high(),
            self.game_state.player.tile_detection.normal_tiles() as u8,
            self.game_state.player.tile_detection.normal_tiles_high(),
            self.game_state.player.follower_link.deep_water_state(),
            self.game_state.player.follower_link.palette_bits_of_oam(),
            self.game_state.player.tile_detection.palette_bits_high(),
            self.game_state.player.tile_detection.deepwater(),
            self.game_state.player.tile_detection.normal_tiles(),
            self.game_state.player.follower_link.deep_water_state(),
            self.game_state.player.follower_link.x(),
            self.game_state.player.follower_link.y(),
            self.game_state.player.follower_link.handler_state(),
        );
    }

    #[track_caller]
    pub(super) fn replay_trace_sfx(&self, func: &str, k: Option<usize>, raw: u8, out: u8) {
        let Some(target) = Self::parse_trace_env_u32("ZELDA3_REPLAY_SFX_TRACE_FRAME") else {
            return;
        };
        if self.frame_ctr_dbg != target && self.state_recorder.replay_frame_counter != target {
            return;
        }
        let caller = std::panic::Location::caller();
        let frame = &self.game_state.frame;
        eprintln!(
            "sfx-trace frame={} local={} fc=0x{:02x} func={} caller={}:{} k={} raw=0x{:02x} out=0x{:02x} se=0x{:02x}/0x{:02x}/0x{:02x} cf8=0x{:02x}",
            self.state_recorder.replay_frame_counter,
            self.frame_ctr_dbg,
            frame.frame_counter,
            func,
            caller.file(),
            caller.line(),
            k.map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string()),
            raw,
            out,
            self.game_state.system_signals.ambient_sound_effect(),
            self.game_state.system_signals.sound_effect_1(),
            self.game_state.system_signals.sound_effect_2(),
            self.game_state.system_signals.raw_sfx_pan_value(),
        );
    }

    fn parse_trace_env_u32(name: &str) -> Option<u32> {
        let value = env::var(name).ok()?;
        if let Some(hex) = value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
        {
            u32::from_str_radix(hex, 16).ok()
        } else {
            value.parse::<u32>().ok()
        }
    }

    fn replay_trace_filter_matches_current_frame(&self) -> bool {
        let Some(target) = Self::parse_trace_env_u32("ZELDA3_REPLAY_TRACE_SUB_FRAME") else {
            return false;
        };
        let frame = &self.game_state.frame;
        if frame.frame_counter as u32 != target {
            return false;
        }
        if let Some(target) = Self::parse_trace_env_u32("ZELDA3_REPLAY_TRACE_SUB_MAIN") {
            if frame.main_module as u32 != target {
                return false;
            }
        }
        if let Some(target) = Self::parse_trace_env_u32("ZELDA3_REPLAY_TRACE_SUB_OW") {
            if u32::from(self.game_state.world.location.overworld_screen()) != target {
                return false;
            }
        }
        true
    }

    pub(crate) fn selected_save_slot_x2(&self) -> u16 {
        read_le_u16(&self.sram, SELECTED_SAVE_SLOT_X2)
    }

    pub(crate) fn selected_save_slot_byte(&self) -> u8 {
        self.selected_save_slot_x2() as u8
    }

    pub(crate) fn selected_save_slot_index(&self) -> usize {
        ((self.selected_save_slot_x2() >> 1).wrapping_sub(1)) as usize
    }

    pub(crate) fn selected_save_slot_offset(&self) -> usize {
        self.selected_save_slot_index() * 0x500
    }

    pub(crate) fn selected_save_slot_source_offset(&self) -> u16 {
        ((u16::from(self.selected_save_slot_byte()) >> 1).wrapping_sub(1)).wrapping_mul(0x500)
    }

    pub(crate) fn set_selected_save_slot_x2(&mut self, value: u16) {
        write_le_u16(&mut self.sram, SELECTED_SAVE_SLOT_X2, value);
    }

    pub(crate) fn set_selected_save_slot_from_cursor(&mut self, cursor: u8) {
        self.set_selected_save_slot_x2(u16::from(cursor) * 2 + 2);
    }

    pub(crate) fn clear_selected_save_slot(&mut self) {
        self.set_selected_save_slot_x2(0);
    }

    fn replay_trace_submodule(&self, label: &str) {
        if !self.replay_trace_filter_matches_current_frame() {
            return;
        }
        let frame = &self.game_state.frame;
        let world_location = &self.game_state.world.location;
        eprintln!(
            "replay-sub frame={} {label} main={} sub={} subsub={} state=0x{:02x} nearpit=0x{:02x} pit=0x{:02x} water=0x{:04x} deep=0x{:04x} flippers=0x{:02x} bunny=0x{:02x} pearl=0x{:02x} indoors={} ow=0x{:04x} vis=0x{:02x} x=0x{:04x} y=0x{:04x} subpix=0x{:02x}/0x{:02x} vel=0x{:02x}/0x{:02x} yvel=0x{:02x} dir=0x{:02x} last=0x{:02x} dlast=0x{:02x} r14=0x{:04x} r12=0x{:04x} normal=0x{:04x} vledge=0x{:02x} stair=0x{:02x} drag=0x{:02x} hp=0x{:02x}",
            frame.frame_counter,
            frame.main_module,
            frame.submodule,
            frame.subsubmodule,
            self.game_state.player.follower_link.handler_state(),
            self.game_state.player.follower_link.near_pit_state(),
            self.game_state.player.tile_detection.pit_tile(),
            self.game_state.player.tile_detection.water_staircase(),
            self.game_state.player.tile_detection.deepwater(),
            self.game_state.player.follower_link.flippers(),
            self.game_state.player.follower_link.is_bunny_mirror() as u8,
            self.game_state.player.follower_link.moon_pearl(),
            world_location.indoor_flag(),
            world_location.overworld_screen(),
            self.game_state.player.follower_link.visibility_status(),
            self.game_state.player.follower_link.x(),
            self.game_state.player.follower_link.y(),
            self.game_state.player.follower_link.x_subpixel(),
            self.game_state.player.follower_link.y_subpixel(),
            self.game_state.player.follower_link.actual_x_velocity(),
            self.game_state.player.follower_link.actual_y_velocity(),
            self.game_state.player.follower_link.y_velocity(),
            self.game_state.player.follower_link.direction(),
            self.game_state.player.follower_link.last_direction(),
            self.game_state.player.follower_link.last_direction_moved_towards(),
            self.game_state.player.tile_detection.collision_bits(),
            self.game_state.player.tile_detection.slope_collision_bits(),
            self.game_state.player.tile_detection.normal_tiles(),
            self.game_state.player.tile_detection.vertical_ledge(),
            self.game_state.player.tile_detection.stair_tile(),
            self.game_state.player.follower_link.defense_flags(),
            self.game_state.inventory.player_resources.current_health(),
        );
    }

    pub fn intro_poly_upload_delay(&self) -> u8 {
        self.intro_poly_upload_delay
    }

    pub fn debug_nmi_poly_upload_deferred(&self) -> u8 {
        self.nmi_poly_upload_deferred
    }

    pub fn debug_nmi_poly_upload_started(&self) -> bool {
        self.nmi_poly_upload_started
    }

    pub fn debug_snes9x_poly_scheduler_counter(&self) -> u8 {
        self.snes9x_poly_scheduler_counter
    }

    pub fn debug_snes9x_hold_intro_step_this_frame(&self) -> bool {
        self.snes9x_hold_intro_step_this_frame
    }

    pub fn debug_snes9x_intro_step_carry_phase_active(&self) -> bool {
        self.snes9x_intro_step_carry_phase_active
    }

    pub fn debug_last_poly_work(&self) -> PolyWorkMetrics {
        self.last_poly_work
    }

    pub fn debug_snes9x_intro_step_hold_alternate(&self) -> bool {
        self.snes9x_intro_step_hold_alternate
    }

    pub(crate) fn player_state(&self) -> RamPlayerStateView<'_> {
        RamPlayerStateView::new(&self.ram)
    }

    pub(crate) fn player_state_mut(&mut self) -> RamPlayerStateViewMut<'_> {
        RamPlayerStateViewMut::new(&mut self.ram)
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
            game_state.effects.door_debris
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
        pub(crate) fn sprite_workspace_mut() -> NativeSpriteWorkspaceBridgeMut {
            game_state.sprites.workspace
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

    pub(crate) fn sync_follower_link_state_from_ram(&mut self) {
        self.game_state
            .player
            .sync_follower_link_from_ram(&self.ram);
    }

    pub(crate) fn follower_link_state(&self) -> &FollowerLinkState {
        &self.game_state.player.follower_link
    }

    pub(crate) fn palette_swap_enabled(&self) -> bool {
        self.game_state.sprites.follower_runtime.palette_swap_flag() != 0
    }

    pub(crate) fn set_music_control(&mut self, value: u8) {
        self.system_signals_mut().set_music_control(value);
    }

    pub(crate) fn set_current_music_control(&mut self, value: u8) {
        self.system_signals_mut().set_current_music_control(value);
    }

    pub(crate) fn set_last_music_control(&mut self, value: u8) {
        self.system_signals_mut().set_last_music_control(value);
    }

    pub(crate) fn set_queued_music_control(&mut self, value: u8) {
        self.system_signals_mut().set_queued_music_control(value);
    }

    pub(crate) fn resident_song_bank_is_dungeon(&self) -> bool {
        self.game_state
            .system_signals
            .resident_song_bank_is_dungeon()
    }

    pub(crate) fn select_overworld_song_bank(&mut self) {
        self.system_signals_mut().select_overworld_song_bank();
    }

    pub(crate) fn select_dungeon_song_bank(&mut self) {
        self.system_signals_mut().select_dungeon_song_bank();
    }

    pub(crate) fn set_ambient_sound_effect(&mut self, value: u8) {
        self.system_signals_mut().set_ambient_sound_effect(value);
    }

    pub(crate) fn set_sound_effect_1(&mut self, value: u8) {
        self.system_signals_mut().set_sound_effect_1(value);
    }

    pub(crate) fn set_sound_effect_2(&mut self, value: u8) {
        self.system_signals_mut().set_sound_effect_2(value);
    }

    pub(crate) fn set_apui00(&mut self, value: u8) {
        self.system_signals_mut().set_apui00(value);
    }

    pub(crate) fn set_msu_volume(&mut self, value: u8) {
        self.system_signals_mut().set_msu_volume(value);
    }

    pub(crate) fn set_msu_resume_info(&mut self, slot: MsuResumeSlot, info: MsuResumeInfoState) {
        self.system_signals_mut().set_msu_resume_info(slot, info);
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

    pub(crate) fn increment_cgram_update_flag(&mut self) -> u8 {
        self.system_signals_mut().increment_cgram_update_flag()
    }

    pub(crate) fn clear_cgram_update_flag(&mut self) {
        self.system_signals_mut().clear_cgram_update_flag();
    }

    pub(crate) fn set_bugs_fixed(&mut self, value: u8) {
        self.system_signals_mut().set_bugs_fixed(value);
    }

    pub(crate) fn save_current_music_as_last(&mut self) {
        self.system_signals_mut().save_current_music_as_last();
    }

    pub(crate) fn save_ambient_sound_effect_as_last(&mut self) {
        self.system_signals_mut()
            .save_ambient_sound_effect_as_last();
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

    pub(crate) fn set_raw_sfx_pan_value(&mut self, value: u8) {
        self.system_signals_mut().set_raw_sfx_pan_value(value);
    }

    pub(crate) fn set_game_over_check_flag(&mut self, value: u8) {
        self.system_signals_mut().set_game_over_check_flag(value);
    }

    pub(crate) fn increment_game_over_check_flag(&mut self) {
        self.system_signals_mut().increment_game_over_check_flag();
    }

    pub(crate) fn set_death_backup_current_music(&mut self, value: u8) {
        self.system_signals_mut()
            .set_death_backup_current_music(value);
    }

    pub(crate) fn set_death_backup_ambient_sound(&mut self, value: u8) {
        self.system_signals_mut()
            .set_death_backup_ambient_sound(value);
    }

    fn compatibility_ram_range(&self, offset: usize, len: usize) -> &[u8] {
        CompatibilityBytesView::new(&self.ram).range(offset, len)
    }

    fn set_compatibility_ram_byte(&mut self, offset: usize, value: u8) {
        CompatibilityBytesViewMut::new(&mut self.ram).set_byte_at(offset, value);
    }

    pub(crate) fn set_sound_effect_1_with_link_pan(&mut self, effect: u8) {
        let sound_effect = self.link_calculate_sfx_pan() | effect;
        self.set_sound_effect_1(sound_effect);
    }

    pub(crate) fn set_sound_effect_2_with_link_pan(&mut self, effect: u8) {
        let sound_effect = self.link_calculate_sfx_pan() | effect;
        self.set_sound_effect_2(sound_effect);
    }

    pub(crate) fn set_sound_effect_1_with_ancilla_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.ancilla_calculate_sfx_pan(slot) | effect;
        self.set_sound_effect_1(sound_effect);
    }

    pub(crate) fn set_sound_effect_2_with_ancilla_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.ancilla_calculate_sfx_pan(slot) | effect;
        self.set_sound_effect_2(sound_effect);
    }

    pub(crate) fn set_sound_effect_1_with_sprite_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.sprite_calculate_sfx_pan(slot) | effect;
        self.set_sound_effect_1(sound_effect);
    }

    pub(crate) fn set_sound_effect_2_with_sprite_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.sprite_calculate_sfx_pan(slot) | effect;
        self.set_sound_effect_2(sound_effect);
    }

    pub(crate) fn bg1_move_calc(&self) -> &Bg1MovementAccumulatorState {
        &self.game_state.player.bg1_movement_accumulator
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

    /// Write-through backup of the map/death palette into MAPBAK_PALETTE. This is NOT done
    /// via the scroll-copy sync (PpuScrollCopyState::write_to_ram no longer projects
    /// mapbak_palette): a scroll-register sync would otherwise re-run that projection every
    /// frame and clobber a freshly-written overworld palette backup (f335672). Mirrors the
    /// old projection's RAM effect (the native field is fill-padded to MAPBAK_PALETTE_BYTES).
    pub(crate) fn copy_mapbak_palette_from(
        &mut self,
        palette: &[u8],
        source: crate::game_state::PaletteSliceSource,
    ) {
        self.ppu_scroll_copy_mut().copy_mapbak_palette_from(palette);
        // The write-through above updated RAM[MAPBAK_PALETTE] and the scroll-copy
        // model but not the provenance mirror; carry the source provenance into
        // the mirror's Backup bank so later mapbak reads resolve clean.
        let len = palette.len().min(0x200);
        self.palette_buffer_mut()
            .tag_backup_bank_from(palette, len, source);
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

    pub(crate) fn increment_submodule(&mut self) {
        self.frame_state_mut().increment_submodule();
    }

    pub(crate) fn decrement_submodule(&mut self) {
        self.frame_state_mut().decrement_submodule();
    }

    pub(crate) fn set_subsubmodule(&mut self, value: u8) {
        self.frame_state_mut().set_subsubmodule(value);
    }

    pub(crate) fn increment_subsubmodule(&mut self) {
        self.frame_state_mut().increment_subsubmodule();
    }

    pub(crate) fn decrement_subsubmodule(&mut self) {
        self.frame_state_mut().decrement_subsubmodule();
    }

    pub(crate) fn set_frame_counter(&mut self, value: u8) {
        self.frame_state_mut().set_frame_counter(value);
    }

    pub(crate) fn increment_frame_counter(&mut self) {
        self.frame_state_mut().increment_frame_counter();
    }

    pub(crate) fn set_saved_module_for_menu(&mut self, value: u8) {
        self.frame_state_mut().set_saved_module_for_menu(value);
    }

    pub(crate) fn clear_saved_module_for_menu(&mut self) {
        self.frame_state_mut().clear_saved_module_for_menu();
    }

    pub(crate) fn save_main_module_for_menu(&mut self) {
        self.frame_state_mut().save_main_module_for_menu();
    }

    pub(crate) fn save_submodule_for_menu(&mut self) {
        self.frame_state_mut().save_submodule_for_menu();
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

    pub(crate) fn set_dungeon_room(&mut self, value: u16) {
        self.world_location_mut().set_dungeon_room(value);
    }

    pub(crate) fn set_dungeon_room_index(&mut self, value: u8) {
        self.world_location_mut().set_dungeon_room_index(value);
    }

    pub(crate) fn increment_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        self.world_location_mut()
            .increment_dungeon_room_index_by(value)
    }

    pub(crate) fn decrement_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        self.world_location_mut()
            .decrement_dungeon_room_index_by(value)
    }

    pub(crate) fn set_overworld_screen(&mut self, value: u8) {
        self.world_location_mut().set_overworld_screen(value);
    }

    pub(crate) fn set_overworld_screen_word(&mut self, value: u16) {
        self.world_location_mut().set_overworld_screen_word(value);
    }

    pub(crate) fn set_indoor_flag(&mut self, value: u8) {
        self.world_location_mut().set_indoor_flag(value);
    }

    // BG scroll copy2 (0xe0/0xe2/0xe6/0xe8) is owned solely by PpuScrollCopyState; these
    // legacy `set_bgN_{x,y}` names delegate to it so the ~80 callers stay unchanged.
    pub(crate) fn set_bg1_x(&mut self, value: u16) {
        self.set_bg1_h_copy2(value);
    }

    pub(crate) fn set_bg1_x_low(&mut self, value: u8) {
        let hi = self.game_state.display.ppu_scroll_copy.bg1_h_copy2() & 0xff00;
        self.set_bg1_h_copy2(hi | u16::from(value));
    }

    pub(crate) fn set_bg1_y(&mut self, value: u16) {
        self.set_bg1_v_copy2(value);
    }

    pub(crate) fn set_bg1_y_low(&mut self, value: u8) {
        let hi = self.game_state.display.ppu_scroll_copy.bg1_v_copy2() & 0xff00;
        self.set_bg1_v_copy2(hi | u16::from(value));
    }

    pub(crate) fn set_bg2_x(&mut self, value: u16) {
        self.set_bg2_h_copy2(value);
    }

    pub(crate) fn set_bg2_y(&mut self, value: u16) {
        self.set_bg2_v_copy2(value);
    }

    pub(crate) fn set_bg1_x_offset(&mut self, value: u16) {
        self.world_scroll_mut().set_bg1_x_offset(value);
    }

    pub(crate) fn set_bg1_y_offset(&mut self, value: u16) {
        self.world_scroll_mut().set_bg1_y_offset(value);
    }

    pub(crate) fn set_overworld_offset_base_y(&mut self, value: u16) {
        self.world_scroll_mut().set_overworld_offset_base_y(value);
    }

    pub(crate) fn set_overworld_offset_base_x(&mut self, value: u16) {
        self.world_scroll_mut().set_overworld_offset_base_x(value);
    }

    pub(crate) fn set_overworld_offset_mask_y(&mut self, value: u16) {
        self.world_scroll_mut().set_overworld_offset_mask_y(value);
    }

    pub(crate) fn set_overworld_offset_mask_x(&mut self, value: u16) {
        self.world_scroll_mut().set_overworld_offset_mask_x(value);
    }

    pub(crate) fn world_camera_boundaries_mut(
        &mut self,
    ) -> NativeWorldCameraBoundariesBridgeMut<'_> {
        NativeWorldCameraBoundariesBridgeMut::new(
            &mut self.game_state.world.camera_boundaries,
            &mut self.ram,
        )
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

    pub(crate) fn set_overworld_area_index(&mut self, value: u8) {
        self.world_region_mut().set_overworld_area_index(value);
    }

    pub(crate) fn set_overworld_area_index_word(&mut self, value: u16) {
        self.world_region_mut().set_overworld_area_index_word(value);
    }

    pub(crate) fn set_current_area_of_player_word(&mut self, value: u16) {
        self.world_region_mut()
            .set_current_area_of_player_word(value);
    }

    pub(crate) fn set_flag_overworld_area_changed(&mut self, value: u8) {
        self.world_region_mut()
            .set_flag_overworld_area_changed(value);
    }

    pub(crate) fn clear_flag_overworld_area_changed(&mut self) {
        self.world_region_mut().clear_flag_overworld_area_changed();
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

    pub(crate) fn save_spexit_area_index(&mut self) {
        self.world_region_mut().save_spexit_area_index();
    }

    pub(crate) fn restore_spexit_area_index(&mut self) {
        self.world_region_mut().restore_spexit_area_index();
    }

    pub(crate) fn save_exit_area_index(&mut self) {
        self.world_region_mut().save_exit_area_index();
    }

    pub(crate) fn restore_exit_area_index(&mut self) {
        self.world_region_mut().restore_exit_area_index();
    }

    pub(crate) fn set_ow_entrance_value(&mut self, value: u16) {
        self.world_region_mut().set_ow_entrance_value(value);
    }

    pub(crate) fn ow_entrance_value(&self) -> u16 {
        self.game_state.world.region.ow_entrance_value()
    }

    pub(crate) fn set_room_transitioning_flags(&mut self, value: u8) {
        self.world_transient_mut()
            .set_room_transitioning_flags(value);
    }

    pub(crate) fn clear_custom_spell_animation(&mut self) {
        self.world_transient_mut().clear_custom_spell_animation();
    }

    pub(crate) fn set_custom_spell_animation_active(&mut self) {
        self.world_transient_mut()
            .set_custom_spell_animation_active();
    }

    pub(crate) fn set_allow_scroll_z(&mut self, value: u8) {
        self.world_transient_mut().set_allow_scroll_z(value);
    }

    pub(crate) fn set_cached_room_bounds(
        &mut self,
        y_start: u16,
        y_end: u16,
        x_start: u16,
        x_end: u16,
    ) {
        self.world_transient_mut()
            .set_cached_room_bounds(y_start, y_end, x_start, x_end);
    }

    pub(crate) fn set_standing_in_doorway_cached(&mut self, value: u8) {
        self.world_transient_mut()
            .set_standing_in_doorway_cached(value);
    }

    pub(crate) fn cache_standing_in_doorway(&mut self, doorway_state: u8) {
        self.world_transient_mut()
            .cache_standing_in_doorway(doorway_state);
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

    pub(crate) fn set_door_animation_step(&mut self, value: u8) {
        self.world_transient_mut().set_door_animation_step(value);
    }

    pub(crate) fn set_door_animation_step_word(&mut self, value: u16) {
        self.world_transient_mut()
            .set_door_animation_step_word(value);
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

    pub(crate) fn restore_quadrant_fullsize_from_cached(&mut self) {
        self.world_transient_mut()
            .restore_quadrant_fullsize_from_cached();
    }

    pub(crate) fn set_quadrant_fullsize_x(&mut self, value: u8) {
        self.world_transient_mut().set_quadrant_fullsize_x(value);
    }

    pub(crate) fn set_quadrant_fullsize_y(&mut self, value: u8) {
        self.world_transient_mut().set_quadrant_fullsize_y(value);
    }

    pub(crate) fn set_fullsize_overworld_quadrants(&mut self) {
        self.world_transient_mut()
            .set_fullsize_overworld_quadrants();
    }

    pub(crate) fn set_horizontal_room_fullsize_state(&mut self, value: u8) {
        self.world_transient_mut()
            .set_horizontal_room_fullsize_state(value);
    }

    pub(crate) fn set_vertical_room_fullsize_state(&mut self, value: u8) {
        self.world_transient_mut()
            .set_vertical_room_fullsize_state(value);
    }

    pub(crate) fn apply_dungeon_layout_quadrant_fullsize(
        &mut self,
        layout_flags: u8,
        horizontal_mask: u8,
        vertical_mask: u8,
        blast_wall_x_open: bool,
        blast_wall_y_open: bool,
    ) {
        self.world_transient_mut()
            .apply_dungeon_layout_quadrant_fullsize(
                layout_flags,
                horizontal_mask,
                vertical_mask,
                blast_wall_x_open,
                blast_wall_y_open,
            );
    }

    pub(crate) fn apply_dungeon_layout_horizontal_fullsize(
        &mut self,
        layout_flags: u8,
        horizontal_mask: u8,
        blast_wall_x_open: bool,
    ) {
        self.world_transient_mut()
            .apply_dungeon_layout_horizontal_fullsize(
                layout_flags,
                horizontal_mask,
                blast_wall_x_open,
            );
    }

    pub(crate) fn apply_dungeon_layout_vertical_fullsize(
        &mut self,
        layout_flags: u8,
        vertical_mask: u8,
        blast_wall_y_open: bool,
    ) {
        self.world_transient_mut()
            .apply_dungeon_layout_vertical_fullsize(layout_flags, vertical_mask, blast_wall_y_open);
    }

    pub(crate) fn apply_reset_xy_quadrant_overrides(&mut self, reset_xy_flags: u16) {
        self.world_transient_mut()
            .apply_reset_xy_quadrant_overrides(reset_xy_flags);
    }

    pub(crate) fn force_horizontal_fullsize_for_blast_wall(&mut self) {
        self.world_transient_mut()
            .force_horizontal_fullsize_for_blast_wall();
    }

    pub(crate) fn force_vertical_fullsize_for_blast_wall(&mut self) {
        self.world_transient_mut()
            .force_vertical_fullsize_for_blast_wall();
    }

    pub(crate) fn save_spexit_tm_copy(&mut self) {
        self.world_transient_mut().save_spexit_tm_copy();
    }

    pub(crate) fn restore_spexit_layer_masks(&mut self) {
        self.world_transient_mut().restore_spexit_layer_masks();
    }

    pub(crate) fn save_exit_tm_copy(&mut self) {
        self.world_transient_mut().save_exit_tm_copy();
    }

    pub(crate) fn restore_exit_layer_masks(&mut self) {
        self.world_transient_mut().restore_exit_layer_masks();
    }

    pub(crate) fn set_world_transient_map_backup_subscreen_layer(&mut self, value: u8) {
        self.world_transient_mut().set_mapbak_ts(value);
    }

    pub(crate) fn set_world_transient_map_backup_main_layer(&mut self, value: u8) {
        self.world_transient_mut().set_mapbak_tm(value);
    }

    pub(crate) fn increment_move_overlay_ctr(&mut self) -> u8 {
        self.world_transient_mut().increment_move_overlay_ctr()
    }

    pub(crate) fn set_overworld_hole_scan_step(&mut self, value: u8) {
        self.world_transient_mut()
            .set_overworld_hole_scan_step(value);
    }

    pub(crate) fn set_overworld_peg_puzzle_progress(&mut self, value: u16) {
        self.world_transient_mut()
            .set_overworld_peg_puzzle_progress(value);
    }

    pub(crate) fn set_overworld_hole_tilemap_pos(&mut self, value: u16) {
        self.world_transient_mut()
            .set_overworld_hole_tilemap_pos(value);
    }

    pub(crate) fn set_overworld_bomb_tile_sweep_x(&mut self, value: u16) {
        self.world_transient_mut()
            .set_overworld_bomb_tile_sweep_x(value);
    }

    pub(crate) fn set_overworld_bomb_tile_sweep_y_end(&mut self, value: u16) {
        self.world_transient_mut()
            .set_overworld_bomb_tile_sweep_y_end(value);
    }

    pub(crate) fn set_big_key_door_message_triggered(&mut self, value: u16) {
        self.world_transient_mut()
            .set_big_key_door_message_triggered(value);
    }

    pub(crate) fn set_savegame_has_master_sword_flags(&mut self, value: u16) {
        self.world_transient_mut()
            .set_savegame_has_master_sword_flags(value);
    }

    pub(crate) fn set_dung_replacement_tile_state(&mut self, index: usize, value: u16) {
        self.world_transient_mut()
            .set_dung_replacement_tile_state(index, value);
    }

    pub(crate) fn decrement_milestone_item_gfx_swap_countdown(&mut self) {
        self.world_transient_mut()
            .decrement_milestone_item_gfx_swap_countdown();
    }

    pub(crate) fn overworld_map_state(&self) -> u8 {
        self.game_state.world.overworld.map_ui.map_state()
    }

    pub(crate) fn overworld_map_state_word(&self) -> u16 {
        self.game_state.world.overworld.map_ui.map_state_word()
    }

    pub(crate) fn set_overworld_map_state(&mut self, value: u8) {
        self.overworld_map_ui_mut().set_map_state(value);
        self.sync_dungeon_chest_cursor_with_map_state();
    }

    pub(crate) fn set_overworld_map_state_word(&mut self, value: u16) {
        self.overworld_map_ui_mut().set_map_state_word(value);
        self.sync_dungeon_chest_cursor_with_map_state();
    }

    pub(crate) fn increment_overworld_map_state(&mut self) {
        self.overworld_map_ui_mut().increment_map_state();
        self.sync_dungeon_chest_cursor_with_map_state();
    }

    // OVERWORLD_MAP_STATE (0x200) is SNES byte-reused: OverworldMapUiState.map_state (the HUD
    // redraw counter) and DungeonRoomItemState.chest_reveal_cursor_x2 both model/project it.
    // Both native states persist and sync independently, so a HUD-counter change made via
    // map_ui is clobbered when the (stale) dungeon chest-cursor bridge syncs the same byte
    // afterward — leaving OVERWORLD_MAP_STATE stuck (e.g. =5) across a dungeon room transition,
    // which kept hud_refill_logic gated off and skewed rupee/heart-drain timing (~rf 126k-132k).
    // Keep the dungeon model in sync with the HUD counter so neither clobbers the other.
    fn sync_dungeon_chest_cursor_with_map_state(&mut self) {
        let value = self.game_state.world.overworld.map_ui.map_state;
        self.dungeon_room_items_mut()
            .set_chest_reveal_cursor_x2(value);
    }

    pub(crate) fn overworld_map_flags(&self) -> u8 {
        self.game_state.world.overworld.map_ui.map_flags
    }

    pub(crate) fn set_overworld_map_flags(&mut self, value: u8) {
        self.overworld_map_ui_mut().set_map_flags(value);
    }

    pub(crate) fn and_overworld_map_flags(&mut self, value: u8) {
        self.overworld_map_ui_mut().and_map_flags(value);
    }

    pub(crate) fn or_overworld_map_flags(&mut self, value: u8) {
        self.overworld_map_ui_mut().or_map_flags(value);
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

    pub(crate) fn set_mode7_zoom_step_counter(&mut self, value: u8) {
        self.overworld_map_zoom_mut().set_step_counter(value);
    }

    pub(crate) fn mode7_zoom_timer(&self) -> u8 {
        self.game_state.world.overworld.map_zoom.timer
    }

    pub(crate) fn set_mode7_zoom_timer(&mut self, value: u8) {
        self.overworld_map_zoom_mut().set_timer(value);
    }

    pub(crate) fn overworld_is_big_area_word(&self) -> u16 {
        self.game_state
            .world
            .overworld
            .screen_size
            .is_big_area_word()
    }

    pub(crate) fn overworld_is_big_area(&self) -> bool {
        self.game_state.world.overworld.screen_size.is_big_area()
    }

    pub(crate) fn overworld_right_bottom_scroll_bound(&self) -> u16 {
        self.game_state
            .world
            .overworld
            .screen_size
            .right_bottom_bound_word()
    }

    pub(crate) fn clear_overworld_big_area_high(&mut self) {
        self.overworld_screen_size_mut().clear_big_area_high();
    }

    pub(crate) fn set_overworld_big_area_low(&mut self, value: u8) {
        self.overworld_screen_size_mut().set_big_area_low(value);
    }

    pub(crate) fn backup_overworld_big_area_low(&mut self) {
        self.overworld_screen_size_mut().backup_big_area_low();
    }

    pub(crate) fn set_overworld_right_bottom_bound_low(&mut self, value: u8) {
        self.overworld_screen_size_mut()
            .set_right_bottom_bound_low(value);
    }

    pub(crate) fn set_overworld_right_bottom_bound_high(&mut self, value: u8) {
        self.overworld_screen_size_mut()
            .set_right_bottom_bound_high(value);
    }

    pub(crate) fn overworld_vertical_scroll_delta_low(&self) -> u8 {
        self.game_state
            .world
            .overworld
            .scroll_delta
            .vertical_delta_low_byte()
    }

    pub(crate) fn overworld_horizontal_scroll_delta_low(&self) -> u8 {
        self.game_state
            .world
            .overworld
            .scroll_delta
            .horizontal_delta_low_byte()
    }

    pub(crate) fn overworld_vertical_scroll_delta(&self) -> u16 {
        self.game_state
            .world
            .overworld
            .scroll_delta
            .vertical_delta_word()
    }

    pub(crate) fn set_overworld_vertical_scroll_delta_low(&mut self, value: u8) {
        self.overworld_scroll_delta_mut()
            .set_vertical_delta_low_byte(value);
    }

    pub(crate) fn set_overworld_horizontal_scroll_delta_low(&mut self, value: u8) {
        self.overworld_scroll_delta_mut()
            .set_horizontal_delta_low_byte(value);
    }

    pub(crate) fn set_overworld_vertical_scroll_delta(&mut self, value: u16) {
        self.overworld_scroll_delta_mut()
            .set_vertical_delta_word(value);
    }

    pub(crate) fn set_overworld_horizontal_scroll_delta(&mut self, value: u16) {
        self.overworld_scroll_delta_mut()
            .set_horizontal_delta_word(value);
    }

    pub(crate) fn clear_overworld_vertical_scroll_delta_low(&mut self) {
        self.overworld_scroll_delta_mut()
            .clear_vertical_delta_low_byte();
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

    pub(crate) fn overworld_transition_countdown(&self) -> u8 {
        self.game_state.world.overworld.transition.countdown()
    }

    pub(crate) fn set_overworld_transition_countdown(&mut self, value: u8) {
        self.overworld_transition_mut().set_countdown(value);
    }

    pub(crate) fn decrement_overworld_transition_countdown(&mut self) -> u8 {
        self.overworld_transition_mut().decrement_countdown()
    }

    pub(crate) fn save_previous_screen_transition_direction_bits(&mut self) {
        self.overworld_transition_mut()
            .save_previous_direction_bits();
    }

    pub(crate) fn restore_previous_screen_transition_direction_bits(&mut self) {
        self.overworld_transition_mut()
            .restore_previous_direction_bits();
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

    pub(crate) fn attract_vram_destination_high_is_clear(&self) -> bool {
        self.game_state
            .display
            .attract_vram_destination_high_is_clear()
    }

    pub(crate) fn attract_vram_destination_page_offset(&self) -> u8 {
        self.game_state
            .display
            .attract_vram_destination_page_offset()
    }

    pub(crate) fn attract_vram_destination_address(&self) -> u16 {
        self.game_state.display.attract_vram_destination_address
    }

    pub(crate) fn set_attract_vram_destination_address(&mut self, value: u16) {
        self.attract_vram_destination_bridge_mut()
            .set_address(value);
    }

    pub(crate) fn clear_attract_vram_destination_address(&mut self) {
        self.attract_vram_destination_bridge_mut().clear_address();
    }

    pub(crate) fn set_attract_vram_destination_page_offset(&mut self, value: u8) {
        self.attract_vram_destination_bridge_mut()
            .set_page_offset(value);
    }

    pub(crate) fn decrement_attract_vram_destination_page_offset(&mut self) {
        self.attract_vram_destination_bridge_mut()
            .decrement_page_offset();
    }

    pub(crate) fn decrement_attract_vram_destination_address(&mut self) -> u16 {
        self.attract_vram_destination_bridge_mut()
            .decrement_address()
    }

    pub(crate) fn loaded_room_data_word(&self, offset: usize, index: usize) -> u16 {
        loaded_room_data_word(&self.ram, offset, index)
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

    pub(crate) fn latch_nmi_update(&mut self) {
        self.display_core_mut().latch_nmi_update();
    }

    pub(crate) fn clear_nmi_update_latch(&mut self) {
        self.display_core_mut().clear_nmi_update_latch();
    }

    pub(super) fn suspend_dungeon_subtile_palette_filter_if_return_crosses_nmi(&mut self) {
        let filter = &self.game_state.display.palette_filter;
        let return_crosses_nmi = filter.countdown() != 0 || !filter.is_darkening();
        if self.rom_startup_timing() && return_crosses_nmi {
            // An unfinished color loop always crosses the following vblank.
            // On its terminal step the direction has already toggled: a
            // completed darkening pass (now lightening) still returns after
            // that boundary, while a completed lightening pass (now
            // darkening) returns before it.
            self.game_execution_scheduler.schedule_work(
                GameWorkContinuation::FinishDungeonSubtilePaletteFilter,
                DUNGEON_SUBTILE_PALETTE_FILTER_RETURN_NMI_SLICES,
            );
        }
    }

    pub(super) fn suspend_spiral_staircase_palette_filter(
        &mut self,
        tail: SpiralStaircasePaletteTail,
    ) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        // The interrupted palette walk keeps NMI_DoUpdates gated until the
        // translated caller returns. In particular, the animated-BG DMA must
        // not consume the source advanced by the suspended spiral-staircase
        // main slice.
        self.set_core_update_disable_flag(1);
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishSpiralStaircasePaletteFilter { tail },
            DUNGEON_SUBTILE_PALETTE_FILTER_RETURN_NMI_SLICES,
        );
        true
    }

    fn stage_spiral_stairs_second_grayscale_nmi(&mut self) -> GraphicsDmaGeneration {
        // The final quadrant-upload NMI has returned, so the atomic ROM path
        // exposes $0710 = 0 to this caller. Its core DMA is allowed to run.
        // The resident animated batch remains visible for this image. When
        // this suffix consumes countdown 1 it advances $0adc before NMI, so
        // the DMA must use the live operand even though its result belongs to
        // the following scanout.
        self.clear_core_update_disable_flag();
        self.next_display_animated_bg_scanout_generation =
            Some(AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi);
        if self.game_state.display.bg_tile_animation_countdown == 1 {
            GraphicsDmaGeneration::LiveAfterMain
        } else {
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        }
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

    pub(crate) fn set_pending_nmi_subroutine(&mut self, value: u8) {
        self.display_core_mut().set_pending_nmi_subroutine(value);
    }

    pub(crate) fn clear_pending_nmi_subroutine(&mut self) {
        self.display_core_mut().clear_pending_nmi_subroutine();
    }

    pub(crate) fn take_pending_nmi_subroutine(&mut self) -> u8 {
        self.display_core_mut().take_pending_nmi_subroutine()
    }

    pub(crate) fn set_bg_vram_load_mode(&mut self, value: u8) {
        self.display_core_mut().set_bg_vram_load_mode(value);
    }

    pub(crate) fn queue_tilemap_update(&mut self, destination_page: u8, source_offset: u16) {
        self.display_core_mut()
            .queue_tilemap_update(destination_page, source_offset);
    }

    pub(crate) fn clear_pending_tilemap_update_destination(&mut self) {
        self.display_core_mut()
            .clear_pending_tilemap_update_destination();
    }

    pub(crate) fn set_bg_mode(&mut self, value: u8) {
        self.display_core_mut().set_bg_mode(value);
    }

    fn mirror_display_layer_masks_to_world_transient(&mut self) {
        let layer_masks = self.game_state.display.layer_masks_word();
        self.game_state
            .world
            .transient
            .set_tilemap_layer_copy(layer_masks);
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

    pub(crate) fn set_bg12_window_selection(&mut self, value: u8) {
        self.display_core_mut().set_bg12_window_selection(value);
    }

    pub(crate) fn set_bg34_window_selection(&mut self, value: u8) {
        self.display_core_mut().set_bg34_window_selection(value);
    }

    pub(crate) fn set_object_color_window_selection(&mut self, value: u8) {
        self.display_core_mut()
            .set_object_color_window_selection(value);
    }

    pub(crate) fn set_main_screen_window_layers(&mut self, value: u8) {
        self.display_core_mut().set_main_screen_window_layers(value);
    }

    pub(crate) fn set_sub_screen_window_layers(&mut self, value: u8) {
        self.display_core_mut().set_sub_screen_window_layers(value);
    }

    pub(crate) fn set_window_layer_masks(
        &mut self,
        bg12_window_selection: u8,
        bg34_window_selection: u8,
        object_color_window_selection: u8,
        main_screen_window_layers: u8,
        sub_screen_window_layers: u8,
    ) {
        self.display_core_mut().set_window_layer_masks(
            bg12_window_selection,
            bg34_window_selection,
            object_color_window_selection,
            main_screen_window_layers,
            sub_screen_window_layers,
        );
    }

    pub(crate) fn clear_window_layer_masks(&mut self) {
        self.display_core_mut().clear_window_layer_masks();
    }

    pub(crate) fn clear_window_main_sub_masks(&mut self) {
        self.display_core_mut().clear_window_main_sub_masks();
    }

    pub(crate) fn clear_bg_vram_load_mode(&mut self) {
        self.display_core_mut().clear_bg_vram_load_mode();
    }

    pub(crate) fn set_nmi_copy_packets_request(&mut self, value: u8) {
        self.display_core_mut().set_nmi_copy_packets_request(value);
    }

    pub(crate) fn request_nmi_copy_packets(&mut self) {
        self.display_core_mut().request_nmi_copy_packets();
    }

    pub(crate) fn clear_nmi_copy_packets_request(&mut self) {
        self.display_core_mut().clear_nmi_copy_packets_request();
    }

    pub(crate) fn request_polyhedral_nmi_update(&mut self) {
        self.display_core_mut().request_polyhedral_nmi_update();
    }

    pub(crate) fn clear_pending_polyhedral_update(&mut self) {
        self.display_core_mut().clear_pending_polyhedral_update();
    }

    pub(crate) fn set_chr_halfslot_request(&mut self, value: u8) {
        self.display_core_mut().set_chr_halfslot_request(value);
    }

    pub(crate) fn clear_chr_halfslot_request(&mut self) {
        self.display_core_mut().clear_chr_halfslot_request();
    }

    pub(crate) fn increment_chr_halfslot_request(&mut self) -> u8 {
        self.display_core_mut().increment_chr_halfslot_request()
    }

    pub(crate) fn activate_nmi_thread(&mut self) {
        self.display_core_mut().activate_nmi_thread();
    }

    pub(crate) fn deactivate_nmi_thread(&mut self) {
        self.display_core_mut().deactivate_nmi_thread();
    }

    pub(crate) fn set_nmi_thread_stack_pointer(&mut self, value: u16) {
        self.display_core_mut().set_nmi_thread_stack_pointer(value);
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

    pub(crate) fn set_sprite_dma_head_pointer(&mut self, value: u8) {
        self.display_core_mut().set_sprite_dma_head_pointer(value);
    }

    pub(crate) fn set_sprite_dma_body_pointer(&mut self, value: u8) {
        self.display_core_mut().set_sprite_dma_body_pointer(value);
    }

    pub(crate) fn set_hdma_enable_mask(&mut self, value: u8) {
        self.display_core_mut().set_hdma_enable_mask(value);
    }

    pub(crate) fn clear_hdma_enable_mask(&mut self) {
        self.display_core_mut().clear_hdma_enable_mask();
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

    pub(crate) fn set_nmi_load_target_page(&mut self, value: u8) {
        self.display_core_mut().set_nmi_load_target_page(value);
    }

    pub(crate) fn set_nmi_load_target_address(&mut self, value: u16) {
        self.display_core_mut().set_nmi_load_target_address(value);
    }

    pub(crate) fn reset_incremental_vram_upload_counter(&mut self) {
        self.display_core_mut()
            .reset_incremental_vram_upload_counter();
    }

    pub(crate) fn increment_vram_upload_counter(&mut self) -> u8 {
        self.display_core_mut().increment_vram_upload_counter()
    }

    pub(crate) fn set_link_body_dma_sources(&mut self, top: u16, bottom: u16) {
        self.display_core_mut()
            .set_link_body_dma_sources(top, bottom);
    }

    pub(crate) fn set_link_head_dma_sources(&mut self, top: u16, bottom: u16) {
        self.display_core_mut()
            .set_link_head_dma_sources(top, bottom);
    }

    pub(crate) fn set_link_hand_dma_sources(&mut self, left: u16, right: u16) {
        self.display_core_mut()
            .set_link_hand_dma_sources(left, right);
    }

    pub(crate) fn set_link_sword_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_sword_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_shield_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_shield_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_aux_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_aux_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_push_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_push_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_animated_tile_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_animated_tile_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_head_pointer_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_head_pointer_dma_sources(upper, lower);
    }

    pub(crate) fn set_link_body_pointer_dma_sources(&mut self, upper: u16, lower: u16) {
        self.display_core_mut()
            .set_link_body_pointer_dma_sources(upper, lower);
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

    pub(crate) fn clear_star_tile_restore_phase(&mut self) {
        self.display_core_mut().clear_star_tile_restore_phase();
    }

    pub(crate) fn dungeon_star_tile_restore_source_offsets(&self) -> (usize, usize) {
        // 0x4bc is mode-reused: DisplayState owns the overworld star-tile restore
        // phase, while dungeon room effects own the dungeon interpretation. Match
        // C's live byte read here so a stale overworld projection cannot choose
        // the wrong dungeon graphics source half.
        if self.ram[crate::game_state::constants::STAR_TILE_RESTORE_PHASE] != 0 {
            (32, 0)
        } else {
            (0, 32)
        }
    }

    pub(crate) fn set_animated_tile_data_source_address(&mut self, value: u16) {
        self.display_core_mut()
            .set_animated_tile_data_source_address(value);
    }

    pub(crate) fn set_animated_tile_vram_destination_address(&mut self, value: u16) {
        self.display_core_mut()
            .set_animated_tile_vram_destination_address(value);
    }

    pub(crate) fn overworld_tile_attribute_word(&self, index: usize) -> u16 {
        self.game_state
            .display
            .overworld_tile_attribute_word(&self.ram, index)
    }

    pub(crate) fn set_overworld_tile_attribute_word(&mut self, index: usize, value: u16) {
        let address = crate::game_state::constants::nmi::OVERWORLD_TILE_ATTR_BUFFER + index * 2;
        write_le_u16(&mut self.ram, address, value);
        debug_assert_eq!(
            self.game_state
                .display
                .overworld_tile_attribute_word(&self.ram, index),
            value
        );
    }

    pub(crate) fn set_overworld_tile_upload_word(&mut self, index: usize, value: u16) {
        let address = crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF + index * 2;
        write_le_u16(&mut self.ram, address, value);
        debug_assert_eq!(
            self.game_state
                .display
                .overworld_tile_upload_word(&self.ram, index),
            value
        );
    }

    pub(crate) fn terminate_overworld_tile_upload_words(&mut self, index: usize) {
        self.set_overworld_tile_upload_word(index, 0xffff);
    }

    pub(crate) fn copy_tilemap_upload_stripe_bytes(&mut self, bytes: &[u8]) {
        let start = crate::game_state::constants::nmi::TILEMAP_UPLOAD_BUFFER;
        let len = bytes.len().min(self.ram.len().saturating_sub(start));
        self.ram[start..start + len].copy_from_slice(&bytes[..len]);
        let cursor = self
            .game_state
            .display
            .apply_tilemap_upload_prefix_to_vram_cursor(&bytes[..len]);
        write_le_u16(
            &mut self.ram,
            crate::game_state::constants::nmi::VRAM_UPLOAD_OFFSET,
            cursor,
        );
        debug_assert_eq!(
            self.game_state.display.vram_upload_cursor,
            read_le_u16(
                &self.ram,
                crate::game_state::constants::nmi::VRAM_UPLOAD_OFFSET
            )
        );
    }

    pub(crate) fn set_message_dma_destination_address(&mut self, value: u16) {
        self.display_core_mut()
            .set_message_dma_destination_address(value);
    }

    pub(crate) fn set_message_dma_tile_base(&mut self, value: u16) {
        self.display_core_mut().set_message_dma_tile_base(value);
    }

    pub(crate) fn set_message_dma_tile_limit(&mut self, value: u16) {
        self.display_core_mut().set_message_dma_tile_limit(value);
    }

    pub(crate) fn set_message_dma_tile_sentinel(&mut self, value: u16) {
        self.display_core_mut().set_message_dma_tile_sentinel(value);
    }

    pub(crate) fn set_overworld_fixed_color_adjustment(&mut self, value: u8) {
        // OVERWORLD_FIXED_COLOR_PLUSMINUS (0xc017) is owned/projected by
        // dungeon.room_effects.fixed_color_plusminus. Keep that field in sync so the
        // owner re-projects the value we just wrote; the display copy no longer projects.
        self.dungeon_room_effects_mut()
            .set_fixed_color_plusminus_value_only(value);
        self.display_core_mut()
            .set_overworld_fixed_color_adjustment(value);
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

    pub(crate) fn dungeon_tile_attribute(&self, tile: usize) -> u8 {
        self.game_state
            .dungeon
            .bg2_attributes
            .attr_for_tile(&self.ram, tile)
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

    pub(crate) fn increment_dungeon_map_init_state(&mut self) {
        self.dungeon_map_mut().increment_dungmap_init_state();
    }

    pub(crate) fn clear_dungeon_map_init_state(&mut self) {
        self.dungeon_map_mut().clear_dungmap_init_state();
    }

    pub(crate) fn set_dungeon_map_current_floor(&mut self, value: u16) {
        self.dungeon_map_mut().set_dungmap_cur_floor(value);
    }

    pub(crate) fn decrement_dungeon_map_current_floor_byte(&mut self) {
        self.dungeon_map_mut().decrement_dungmap_cur_floor_byte();
    }

    pub(crate) fn increment_dungeon_map_current_floor_byte(&mut self) {
        self.dungeon_map_mut().increment_dungmap_cur_floor_byte();
    }

    pub(crate) fn clear_dungeon_map_floor_scroll_step(&mut self) {
        self.dungeon_map_mut().clear_dungmap_floor_scroll_step();
    }

    pub(crate) fn increment_dungeon_map_floor_scroll_step(&mut self) {
        self.dungeon_map_mut().increment_dungmap_floor_scroll_step();
    }

    pub(crate) fn set_dungeon_map_scroll_draw_offset(&mut self, value: u16) {
        self.dungeon_map_mut().set_scroll_draw_offset(value);
    }

    pub(crate) fn set_dungeon_map_scroll_input(&mut self, value: u16) {
        self.dungeon_map_mut().set_scroll_input(value);
    }

    pub(crate) fn set_dungeon_map_scroll_target_y(&mut self, value: u16) {
        self.dungeon_map_mut().set_dungmap_scroll_target_y(value);
    }

    pub(crate) fn clear_dungeon_map_scroll_state(&mut self) {
        self.dungeon_map_mut().clear_scroll_state();
    }

    pub(crate) fn set_dungeon_map_idx(&mut self, value: u16) {
        self.dungeon_map_mut().set_dungmap_idx(value);
    }

    pub(crate) fn clear_dungeon_map_idx(&mut self) {
        self.dungeon_map_mut().clear_dungmap_idx();
    }

    pub(crate) fn set_dungeon_map_player_marker_x(&mut self, value: u16) {
        self.dungeon_map_mut().set_dungmap_player_marker_x(value);
    }

    pub(crate) fn set_dungeon_map_player_marker_y(&mut self, value: u16) {
        self.dungeon_map_mut().set_dungmap_player_marker_y(value);
    }

    pub(crate) fn set_dungeon_map_location_marker_base_y(&mut self, value: u8) {
        self.dungeon_map_mut().set_location_marker_base_y(value);
    }

    pub(crate) fn reset_dungeon_map_marker_offsets(&mut self) {
        self.dungeon_map_mut().reset_marker_offsets();
    }

    pub(crate) fn shift_dungeon_map_marker_x_left(&mut self) -> u16 {
        self.dungeon_map_mut().shift_marker_x_left()
    }

    pub(crate) fn reset_dungeon_map_marker_x_and_shift_marker_y_low_up(&mut self) {
        self.dungeon_map_mut()
            .reset_marker_x_and_shift_marker_y_low_up();
    }

    pub(crate) fn set_dungeon_map_marker_y_offset(&mut self, value: u16) {
        self.dungeon_map_mut().set_marker_y_offset(value);
    }

    pub(crate) fn add_dungeon_map_marker_y_offset_signed(&mut self, value: i16) -> u16 {
        self.dungeon_map_mut().add_marker_y_offset_signed(value)
    }

    pub(crate) fn set_overworld_event_bits(&mut self, screen: usize, mask: u8) {
        self.overworld_event_info_mut().set_event_bits(screen, mask);
    }

    pub(crate) fn set_overworld_event_info(&mut self, screen: usize, value: u8) {
        self.overworld_event_info_mut()
            .set_event_info(screen, value);
    }

    pub(crate) fn clear_overworld_event_bits(&mut self, screen: usize, mask: u8) {
        self.overworld_event_info_mut()
            .clear_event_bits(screen, mask);
    }

    pub(crate) fn overworld_config_table(&self) -> OverworldConfigTableRead<'_> {
        OverworldConfigTableRead::new(
            &self.game_state.world.overworld.config_table,
            usize::from(self.game_state.world.location.overworld_screen_index()),
        )
    }

    pub(crate) fn copy_overworld_music_primary(&mut self, data: &[u8]) {
        self.overworld_config_table_mut().copy_music_primary(data);
    }

    pub(crate) fn copy_overworld_music_secondary(&mut self, data: &[u8]) {
        self.overworld_config_table_mut().copy_music_secondary(data);
    }

    pub(crate) fn set_overworld_music(&mut self, screen: usize, value: u8) {
        self.overworld_config_table_mut().set_music(screen, value);
    }

    pub(crate) fn copy_overworld_sprite_graphics_range(
        &mut self,
        dst: usize,
        data: &[u8],
        src: usize,
        len: usize,
    ) {
        self.overworld_config_table_mut()
            .copy_sprite_graphics_range(dst, data, src, len);
    }

    pub(crate) fn copy_overworld_sprite_palette_range(
        &mut self,
        dst: usize,
        data: &[u8],
        src: usize,
        len: usize,
    ) {
        self.overworld_config_table_mut()
            .copy_sprite_palette_range(dst, data, src, len);
    }

    pub(crate) fn clear_aux_visible_subpalettes(&mut self) {
        self.palette_buffer_mut().clear_aux_visible_subpalettes();
    }

    pub(crate) fn clear_main_visible_subpalettes(&mut self) {
        self.palette_buffer_mut().clear_main_visible_subpalettes();
    }

    pub(crate) fn clear_aux_sprite_subpalettes(&mut self) {
        self.palette_buffer_mut().clear_aux_sprite_subpalettes();
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

    pub(crate) fn clear_overworld_aux_or_main_offset(&mut self) {
        self.palette_buffer_mut()
            .clear_overworld_aux_or_main_offset();
    }

    pub(crate) fn select_overworld_aux_palette_offset(&mut self) {
        self.palette_buffer_mut()
            .select_overworld_aux_palette_offset();
    }

    pub(crate) fn keep_overworld_aux_or_main_low_byte(&mut self) {
        self.palette_buffer_mut()
            .keep_overworld_aux_or_main_low_byte();
    }

    pub(crate) fn clear_main_full(&mut self) {
        self.palette_buffer_mut().clear_main_full();
    }

    pub(crate) fn initialize_palette_mirror_from_zeroed_buffers(&mut self) {
        self.palette_buffer_mut()
            .initialize_mirror_from_zeroed_buffers();
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
    pub(crate) fn backup_overworld_palette_from(&mut self, palette: &[u8]) {
        self.palette_buffer_mut()
            .backup_overworld_palette_from(palette);
    }

    pub(crate) fn backup_overworld_palette_from_tagged(
        &mut self,
        palette: &[u8],
        source: crate::game_state::PaletteSliceSource,
    ) {
        self.palette_buffer_mut()
            .backup_overworld_palette_from_tagged(palette, source);
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

    #[track_caller]
    pub(crate) fn copy_main_palette_bytes(&mut self, src: &[u8], len: usize) {
        self.palette_buffer_mut().copy_main_palette_bytes(src, len);
    }

    pub(crate) fn copy_main_palette_bytes_tagged(
        &mut self,
        src: &[u8],
        len: usize,
        source: crate::game_state::PaletteSliceSource,
    ) {
        self.palette_buffer_mut()
            .copy_main_palette_bytes_tagged(src, len, source);
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

    pub(crate) fn set_palette_main_indoors(&mut self, value: u8) {
        self.palette_buffer_mut().set_palette_main_indoors(value);
    }

    pub(crate) fn set_hud_palette(&mut self, value: u8) {
        self.palette_buffer_mut().set_hud_palette(value);
    }

    pub(crate) fn set_sp6r_indoors(&mut self, value: u8) {
        self.palette_buffer_mut().set_sp6r_indoors(value);
    }

    pub(crate) fn set_overworld_palette_aux2_hi(&mut self, value: u8) {
        self.palette_buffer_mut()
            .set_overworld_palette_aux2_hi(value);
    }

    pub(crate) fn set_overworld_palette_aux3_lo(&mut self, value: u8) {
        self.palette_buffer_mut()
            .set_overworld_palette_aux3_lo(value);
    }

    pub(crate) fn set_bg_tile_animation_countdown(&mut self, value: u16) {
        self.palette_buffer_mut()
            .set_bg_tile_animation_countdown(value);
    }

    pub(crate) fn set_overworld_palette_mode(&mut self, value: u8) {
        self.palette_buffer_mut().set_overworld_palette_mode(value);
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

    pub(crate) fn set_color_window_selection(&mut self, value: u8) {
        self.palette_filter_mut().set_color_window_selection(value);
    }

    pub(crate) fn set_color_window_and_math_word(&mut self, value: u16) {
        self.palette_filter_mut()
            .set_color_window_and_math_word(value);
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

    fn debug_assert_hud_tilemap_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.display.hud_tilemap,
            HudTilemapState::load_from_ram(&self.ram)
        );
    }

    pub(crate) fn set_super_bomb_indicator_timer(&mut self, value: u8) {
        self.hud_mut().set_super_bomb_indicator_timer(value);
    }

    pub(crate) fn set_super_bomb_indicator_counter(&mut self, value: u8) {
        self.hud_mut().set_super_bomb_indicator_counter(value);
    }

    pub(crate) fn set_rupee_sfx_sound_delay(&mut self, value: u8) {
        self.hud_mut().set_rupee_sfx_sound_delay(value);
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

    pub(crate) fn start_shared_message_timer(&mut self, value: u16) {
        self.shared_message_timer_bridge_mut().start(value);
    }

    pub(crate) fn clear_shared_message_timer(&mut self) {
        self.shared_message_timer_bridge_mut().clear();
    }

    pub(crate) fn tick_shared_message_timer(&mut self) -> u16 {
        self.shared_message_timer_bridge_mut().tick()
    }

    pub(crate) fn pause_intro_triangle_motion(&mut self) {
        self.intro_scene_bridge_mut().pause_triangle_motion();
    }

    pub(crate) fn resume_intro_triangle_motion(&mut self) {
        self.intro_scene_bridge_mut().resume_triangle_motion();
    }

    pub(crate) fn reset_intro_sprite_oam_cursor(&mut self) {
        self.intro_scene_bridge_mut().set_sprite_oam_cursor(0x0800);
    }

    pub(crate) fn allocate_intro_sprite_oam_entries(&mut self, entry_count: usize) -> usize {
        self.intro_scene_bridge_mut()
            .allocate_oam_entries(entry_count)
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

    pub(crate) fn graphics_primary_decompression_buffer(&self, len: usize) -> Vec<u8> {
        GraphicsDecompressionScratch::primary_buffer(&self.ram, len)
    }

    pub(crate) fn graphics_combined_decompression_buffers(&self) -> Vec<u8> {
        GraphicsDecompressionScratch::combined_buffers(&self.ram)
    }

    pub(crate) fn copy_to_primary_decompression_buffer(&mut self, data: &[u8]) {
        GraphicsDecompressionScratch::copy_to_primary_buffer(&mut self.ram, data);
    }

    pub(crate) fn copy_decompressed_graphics_to(&mut self, dst: usize, data: &[u8]) -> usize {
        GraphicsDecompressionScratch::copy_to_buffer(&mut self.ram, dst, data)
    }

    pub(crate) fn rotate_animated_dungeon_tile_planes(&mut self) {
        for i in 0..256 {
            let base = 0x9000 + i * 2;
            let x = read_le_u16(&self.ram, base + 0x1880);
            let a = read_le_u16(&self.ram, base + 0x1c80);
            let b = read_le_u16(&self.ram, base + 0x1e80);
            let c = read_le_u16(&self.ram, base + 0x1a80);
            write_le_u16(&mut self.ram, base + 0x1880, a);
            write_le_u16(&mut self.ram, base + 0x1c80, b);
            write_le_u16(&mut self.ram, base + 0x1e80, c);
            write_le_u16(&mut self.ram, base + 0x1a80, x);
        }
    }

    #[track_caller]
    pub(crate) fn write_expanded_graphics_tile_row(
        &mut self,
        dst: usize,
        low_plane: u8,
        high_plane: u8,
        upper_plane: u8,
        composite_plane: u8,
    ) {
        crate::types::ww_check(dst, 2, "expand_gfx_tile_row[lo/hi]", low_plane as u32);
        crate::types::ww_check(
            dst + 0x10,
            2,
            "expand_gfx_tile_row[up/comp]",
            upper_plane as u32,
        );
        self.ram[dst] = low_plane;
        self.ram[dst + 1] = high_plane;
        self.ram[dst + 0x10] = upper_plane;
        self.ram[dst + 0x11] = composite_plane;
    }

    pub(crate) fn set_dungeon_line_pointer_row0(&mut self, index: usize, value: u16) {
        write_le_u16(&mut self.ram, DUNG_LINE_PTRS_ROW0 + index * 2, value);
    }

    pub(crate) fn copy_graphics_message_rows(
        &mut self,
        dst: usize,
        src0: usize,
        src1: usize,
        len: usize,
    ) {
        self.messaging_render_buffer_mut()
            .copy_rows_from_ram(dst, src0, src1, len);
    }

    pub(crate) fn copy_peg_tile_graphics_to_message_buffer(&mut self, first: usize, second: usize) {
        for i in 0..64 {
            let color = read_le_u16(&self.ram, PEG_TILE_GFX_BUFFER + (first >> 1) * 2 + i * 2);
            write_le_u16(&mut self.ram, MESSAGING_BUF_LOAD_GFX + i * 2, color);
        }
        for i in 0..64 {
            let color = read_le_u16(&self.ram, PEG_TILE_GFX_BUFFER + (second >> 1) * 2 + i * 2);
            write_le_u16(&mut self.ram, MESSAGING_BUF_LOAD_GFX + (64 + i) * 2, color);
        }
    }

    pub(crate) fn clear_agahnim_palette_settings(&mut self, len: usize) {
        self.ram[AGAHNIM_PAL_SETTING..AGAHNIM_PAL_SETTING + len].fill(0);
    }

    pub(crate) fn agahnim_palette_word(&self, index: usize) -> u16 {
        read_le_u16(&self.ram, AGAHNIM_PAL_SETTING + index * 2)
    }

    pub(crate) fn set_agahnim_palette_word(&mut self, index: usize, value: u16) {
        write_le_u16(&mut self.ram, AGAHNIM_PAL_SETTING + index * 2, value);
    }

    pub(crate) fn graphics_sprite_decompression_buffer_tail(&self) -> Vec<u8> {
        GraphicsDecompressionScratch::sprite_buffer_tail(&self.ram)
    }

    pub(crate) fn staged_bg_and_sprite_decompression_buffers(&self) -> Vec<u8> {
        GraphicsDecompressionScratch::staged_bg_and_sprite_buffers(&self.ram)
    }

    pub(crate) fn overworld_map16_decode(&self) -> OverworldMap16Decode<'_> {
        OverworldMap16Decode::new(&self.ram)
    }

    pub(crate) fn copy_overworld_map16_decode_source_from(&mut self, data: &[u8]) {
        OverworldMap16DecodeScratch::copy_source_from(&mut self.ram, data);
    }

    pub(crate) fn copy_overworld_map16_scratch_to_source_words_high(&mut self, len: usize) {
        OverworldMap16DecodeScratch::copy_scratch_to_source_words_high(&mut self.ram, len);
    }

    pub(crate) fn copy_overworld_map16_scratch_to_source_words_low(&mut self, len: usize) {
        OverworldMap16DecodeScratch::copy_scratch_to_source_words_low(&mut self.ram, len);
    }

    pub(crate) fn write_overworld_map16_decompressed_byte(&mut self, dst: usize, value: u8) {
        OverworldMap16DecodeScratch::write_decompressed_byte(&mut self.ram, dst, value);
    }

    pub(crate) fn copy_overworld_map16_decompressed_byte(
        &mut self,
        dst_org: usize,
        dst: usize,
        offset: usize,
    ) {
        OverworldMap16DecodeScratch::copy_decompressed_byte(&mut self.ram, dst_org, dst, offset);
    }

    pub(crate) fn fill_overworld_map16_decode_block(&mut self, dst: usize, table: &[u8], x: usize) {
        OverworldMap16DecodeScratch::decode_block_fill(&mut self.ram, dst, table, x);
    }

    pub(crate) fn set_overworld_map16_decode_last(&mut self, value: u16) {
        OverworldMap16DecodeScratch::set_decode_last(&mut self.ram, value);
    }

    pub(crate) fn set_overworld_map16_decode_tmp(&mut self, value: u16) {
        OverworldMap16DecodeScratch::set_decode_tmp(&mut self.ram, value);
    }

    pub(crate) fn write_decoded_overworld_map32_to_bg2_tilemap(&mut self, dst: usize, idx: usize) {
        OverworldMap16DecodeScratch::write_decoded_map32_to_bg2_tilemap(&mut self.ram, dst, idx);
        // The decode wrote the BG2 tilemap as raw RAM, bypassing the live
        // dungeon tilemap cache. Mirror the four written words back so overworld
        // readers and the frame-end projection stay coherent with RAM.
        self.game_state
            .dungeon
            .room_tilemaps
            .mirror_decoded_map32_from_ram(&self.ram, dst);
    }

    pub(crate) fn vram_upload_buffer_word(&self, offset: usize) -> u16 {
        self.game_state
            .display
            .vram_upload_buffer_word(&self.ram, offset)
    }

    pub(crate) fn vram_upload_tilemap_word(&self, offset: usize) -> u16 {
        self.game_state
            .display
            .vram_upload_tilemap_word(&self.ram, offset)
    }

    pub(crate) fn vram_upload_buffer_byte(&self, offset: usize) -> u8 {
        self.game_state
            .display
            .vram_upload_buffer_byte(&self.ram, offset)
    }

    pub(crate) fn vram_upload_buffer_remaining(&self) -> &[u8] {
        self.game_state
            .display
            .vram_upload_buffer_remaining(&self.ram)
    }

    pub(crate) fn vram_upload_buffer_remaining_len(&self) -> usize {
        self.ram
            .len()
            .saturating_sub(self.game_state.display.vram_upload_buffer_base())
    }

    pub(crate) fn animated_tile_dma_source_bytes(&self) -> &[u8] {
        self.game_state
            .display
            .animated_tile_dma_source_bytes(&self.ram)
    }

    pub(crate) fn message_dma_tile_indices(&self) -> &[u8] {
        self.game_state.display.message_dma_tile_indices(&self.ram)
    }

    pub(crate) fn sprite_oam_shadow_buffer(&self) -> &[u8] {
        self.game_state.display.sprite_oam_shadow_buffer(&self.ram)
    }

    pub(crate) fn tilemap_upload_stripe_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .tilemap_upload_stripe_buffer(&self.ram)
    }

    pub(crate) fn secondary_stripe_upload_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .secondary_stripe_upload_buffer(&self.ram)
    }

    pub(crate) fn pending_tilemap_update_source_data(&self) -> &[u8] {
        self.game_state
            .display
            .pending_tilemap_update_source_data(&self.ram)
    }

    pub(crate) fn nmi_vram_packet_buffer(&self) -> &[u8] {
        self.game_state.display.nmi_vram_packet_buffer(&self.ram)
    }

    pub(crate) fn dungeon_bg2_attribute_table(&self) -> &[u8] {
        self.game_state
            .display
            .dungeon_bg2_attribute_table(&self.ram)
    }

    pub(crate) fn dungeon_bg1_attribute_table(&self) -> &[u8] {
        self.game_state
            .display
            .dungeon_bg1_attribute_table(&self.ram)
    }

    pub(crate) fn arbitrary_tilemap_destination(&self, slot: usize) -> u16 {
        self.game_state
            .display
            .arbitrary_tilemap_destination(&self.ram, slot)
    }

    pub(crate) fn bg1_wall_top_tilemap_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .bg1_wall_top_tilemap_buffer(&self.ram)
    }

    pub(crate) fn bg1_wall_bottom_tilemap_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .bg1_wall_bottom_tilemap_buffer(&self.ram)
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

    pub(crate) fn game_over_text_tile_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .game_over_text_tile_buffer(&self.ram)
    }

    pub(crate) fn game_over_text_tail_tile_buffer(&self) -> &[u8] {
        self.game_state
            .display
            .game_over_text_tail_tile_buffer(&self.ram)
    }

    pub(crate) fn polyhedral_tile_buffer(&self) -> &[u8] {
        self.game_state.display.polyhedral_tile_buffer(&self.ram)
    }

    pub(crate) fn vram_dma_source_bytes(&self, source_addr: usize, len: usize) -> &[u8] {
        self.game_state
            .display
            .vram_dma_source_bytes(&self.ram, source_addr, len)
    }

    #[track_caller]
    pub(crate) fn write_vram_upload_buffer_byte(&mut self, offset: usize, value: u8) {
        self.vram_upload_mut().write_buffer_byte(offset, value);
    }

    #[track_caller]
    pub(crate) fn write_vram_upload_buffer_word(&mut self, offset: usize, value: u16) {
        self.vram_upload_mut().write_buffer_word(offset, value);
    }

    #[track_caller]
    pub(crate) fn write_vram_upload_tilemap_word(&mut self, offset: usize, value: u16) {
        self.vram_upload_mut().write_tilemap_word(offset, value);
    }

    pub(crate) fn write_overworld_vram_word(&mut self, word_index: usize, value: u16) {
        self.vram_upload_mut()
            .write_overworld_vram_word(word_index, value);
    }

    #[track_caller]
    pub(crate) fn write_vram_upload_absolute_byte(&mut self, address: usize, value: u8) {
        self.vram_upload_mut().write_absolute_byte(address, value);
    }

    #[track_caller]
    pub(crate) fn write_vram_upload_absolute_word(&mut self, address: usize, value: u16) {
        self.vram_upload_mut().write_absolute_word(address, value);
    }

    pub(crate) fn copy_vram_upload_buffer_bytes(&mut self, offset: usize, data: &[u8]) {
        self.vram_upload_mut().copy_buffer_bytes(offset, data);
    }

    pub(crate) fn terminate_vram_upload_buffer_at(&mut self, offset: usize) {
        self.vram_upload_mut().terminate_buffer_at(offset);
    }

    pub(crate) fn write_vram_upload_level_label_tiles(
        &mut self,
        left: &[u8; 14],
        right: &[u8; 14],
    ) {
        self.vram_upload_mut().write_level_label_tiles(left, right);
    }

    pub(crate) fn write_vram_upload_map16_update_packet(
        &mut self,
        address: usize,
        vram_pos: u16,
        tiles: [u16; 4],
    ) {
        self.vram_upload_mut()
            .write_map16_update_packet(address, vram_pos, tiles);
    }

    pub(crate) fn write_vram_upload_single_tile_stripe_packet(
        &mut self,
        address: usize,
        stripe: u16,
        tile: u16,
    ) {
        self.vram_upload_mut()
            .write_single_tile_stripe_packet(address, stripe, tile);
    }

    pub(crate) fn write_vram_upload_tile_stripe_sentinel(&mut self, address: usize) {
        self.vram_upload_mut().write_tile_stripe_sentinel(address);
    }

    pub(crate) fn set_vram_upload_cursor(&mut self, value: u16) {
        self.vram_upload_mut().set_offset(value);
    }

    pub(crate) fn clear_vram_upload_cursor(&mut self) {
        self.vram_upload_mut().clear_offset();
    }

    pub(crate) fn advance_vram_upload_cursor_by(&mut self, value: u16) -> u16 {
        self.vram_upload_mut().advance_offset_by(value)
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

    pub(crate) fn bombos_fire_column(&self, slot: usize) -> BombosFireColumnState {
        self.game_state.effects.bombos_spell.fire_column(slot)
    }

    pub(crate) fn bombos_fire_column_mut(
        &mut self,
        slot: usize,
    ) -> NativeBombosFireColumnBridgeMut<'_> {
        NativeBombosFireColumnBridgeMut::new(
            &mut self.game_state.effects.bombos_spell,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn bombos_blast(&self, slot: usize) -> BombosBlastState {
        self.game_state.effects.bombos_spell.blast(slot)
    }

    pub(crate) fn bombos_blast_mut(&mut self, slot: usize) -> NativeBombosBlastBridgeMut<'_> {
        NativeBombosBlastBridgeMut::new(
            &mut self.game_state.effects.bombos_spell,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn tower_seal_orbit(&self, slot: usize) -> TowerSealOrbitState {
        self.game_state.effects.tower_seal.orbit(slot)
    }

    pub(crate) fn tower_seal_orbit_mut(
        &mut self,
        slot: usize,
    ) -> NativeTowerSealOrbitBridgeMut<'_> {
        NativeTowerSealOrbitBridgeMut::new(
            &mut self.game_state.effects.tower_seal,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn tower_seal_sparkle(&self, slot: usize) -> TowerSealSparkleState {
        self.game_state.effects.tower_seal.sparkle(slot)
    }

    pub(crate) fn tower_seal_sparkle_mut(
        &mut self,
        slot: usize,
    ) -> NativeTowerSealSparkleBridgeMut<'_> {
        NativeTowerSealSparkleBridgeMut::new(
            &mut self.game_state.effects.tower_seal,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn blast_wall_explosion(&self, slot: usize) -> BlastWallExplosionSlotState {
        self.game_state
            .effects
            .entrance_effects
            .blast_wall_explosion_slot(slot)
    }

    pub(crate) fn blast_wall_explosion_mut(
        &mut self,
        slot: usize,
    ) -> NativeBlastWallExplosionBridgeMut<'_> {
        NativeBlastWallExplosionBridgeMut::new(
            &mut self.game_state.effects.entrance_effects,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn blast_wall_fragment(&self, slot: usize) -> BlastWallFragmentSlotState {
        self.game_state
            .effects
            .entrance_effects
            .blast_wall_fragment_slot(slot)
    }

    pub(crate) fn blast_wall_fragment_mut(
        &mut self,
        slot: usize,
    ) -> NativeBlastWallFragmentBridgeMut<'_> {
        NativeBlastWallFragmentBridgeMut::new(
            &mut self.game_state.effects.entrance_effects,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn blast_wall_fireball(&self, slot: usize) -> BlastWallFireballSlotState {
        self.game_state
            .effects
            .entrance_effects
            .blast_wall_fireball_slot(slot)
    }

    pub(crate) fn blast_wall_fireball_mut(
        &mut self,
        slot: usize,
    ) -> NativeBlastWallFireballBridgeMut<'_> {
        NativeBlastWallFireballBridgeMut::new(
            &mut self.game_state.effects.entrance_effects,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn blast_wall_direction(&self) -> u8 {
        self.game_state
            .effects
            .entrance_effects
            .blast_wall_direction()
    }

    pub(crate) fn skull_woods_fire(&self, slot: usize) -> SkullWoodsFireSlotState {
        self.game_state
            .effects
            .entrance_effects
            .skull_woods_fire_slot(slot)
    }

    pub(crate) fn skull_woods_fire_mut(
        &mut self,
        slot: usize,
    ) -> NativeSkullWoodsFireSlotBridgeMut<'_> {
        NativeSkullWoodsFireSlotBridgeMut::new(
            &mut self.game_state.effects.entrance_effects,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn skull_woods_fire_has_started_entrance_opening(&self) -> bool {
        self.game_state
            .effects
            .entrance_effects
            .skull_woods_fire_has_started_entrance_opening()
    }

    pub(crate) fn skull_woods_fire_inner_x(&self) -> u16 {
        self.game_state
            .effects
            .entrance_effects
            .skull_woods_fire_inner_x()
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

    pub(crate) fn set_weather_vane_countdown(&mut self, value: u16) {
        self.weather_vane_bridge_mut().set_countdown(value);
    }

    pub(crate) fn tick_weather_vane_countdown(&mut self) -> u16 {
        self.weather_vane_bridge_mut().tick_countdown()
    }

    pub(crate) fn weather_vane_music_latch(&self) -> u8 {
        self.game_state.world.overworld.weather_vane.music_latch
    }

    pub(crate) fn set_weather_vane_music_latch(&mut self, value: u8) {
        self.weather_vane_bridge_mut().set_music_latch(value);
    }

    pub(crate) fn set_weather_vane_source_slot(&mut self, value: u8) {
        self.weather_vane_bridge_mut().set_source_slot(value);
    }

    pub(crate) fn reset_weather_vane_oam_offset(&mut self) {
        self.weather_vane_bridge_mut().reset_oam_offset();
    }

    pub(crate) fn advance_weather_vane_oam_offset(&mut self, value: u8) {
        self.weather_vane_bridge_mut().advance_oam_offset(value);
    }

    pub(crate) fn weather_vane_debris(&self, slot: usize) -> WeatherVaneDebrisSlotState {
        self.game_state.effects.weather_vane_debris.debris(slot)
    }

    pub(crate) fn weather_vane_debris_mut(
        &mut self,
        slot: usize,
    ) -> NativeWeatherVaneDebrisBridgeMut<'_> {
        NativeWeatherVaneDebrisBridgeMut::new(
            &mut self.game_state.effects.weather_vane_debris,
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

    pub(crate) fn swamola_target(&self, slot: usize) -> HistoryPositionState {
        self.game_state
            .effects
            .sprite_histories
            .swamola_target(slot)
    }

    pub(crate) fn swamola_target_mut(&mut self, slot: usize) -> NativeSwamolaTargetBridgeMut<'_> {
        NativeSwamolaTargetBridgeMut::new(
            &mut self.game_state.effects.sprite_histories,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn swamola_history(&self, slot: usize) -> HistoryPositionState {
        self.game_state
            .effects
            .sprite_histories
            .swamola_history(slot)
    }

    pub(crate) fn swamola_history_mut(&mut self, slot: usize) -> NativeSwamolaHistoryBridgeMut<'_> {
        NativeSwamolaHistoryBridgeMut::new(
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

    pub(crate) fn set_vwf_next_glyph_advance_prefix_sum(&mut self, index: usize, value: u8) {
        self.vwf_render_mut()
            .set_next_glyph_advance_prefix_sum(index, value);
        // C writes `vwf_arr[index + 1] = value` as raw g_ram, even past the modeled buffer
        // (the credits render lines whose glyph cursor runs beyond it). For those overflow
        // bytes the native Vec projection above does not reach RAM, so write them directly.
        let buf_len = self
            .game_state
            .messaging
            .vwf_render
            .glyph_advance_buffer_len();
        let addr = VWF_ARR + index + 1;
        if index + 1 >= buf_len && addr < self.ram.len() {
            self.ram[addr] = value;
        }
    }

    /// C: `arrval = vwf_arr[index]` — raw g_ram. In-bounds indices read the modeled buffer
    /// (kept RAM-coherent); indices past it read RAM directly, matching C's unbounded access.
    pub(crate) fn vwf_glyph_advance_prefix_sum(&self, index: usize) -> u8 {
        let vwf = &self.game_state.messaging.vwf_render;
        if index < vwf.glyph_advance_buffer_len() {
            vwf.glyph_advance_prefix_sum(index)
        } else {
            self.ram.get(VWF_ARR + index).copied().unwrap_or(0)
        }
    }

    pub(crate) fn set_vwf_glyph_cursor(&mut self, value: u16) {
        self.vwf_render_mut().set_glyph_cursor(value);
    }

    pub(crate) fn clear_vwf_glyph_cursor(&mut self) {
        self.vwf_render_mut().clear_glyph_cursor();
    }

    pub(crate) fn increment_vwf_glyph_cursor(&mut self) -> u16 {
        self.vwf_render_mut().increment_glyph_cursor()
    }

    pub(crate) fn request_vwf_next_line(&mut self, value: u16) {
        self.vwf_render_mut().request_next_line(value);
    }

    pub(crate) fn clear_vwf_next_line_request(&mut self) {
        self.vwf_render_mut().clear_next_line_request();
    }

    pub(crate) fn set_vwf_current_line(&mut self, value: u16) {
        self.vwf_render_mut().set_current_line(value);
    }

    pub(crate) fn set_vwf_line_render_offset(&mut self, value: u16) {
        self.vwf_render_mut().set_line_render_offset(value);
    }

    pub(crate) fn set_vwf_tile_word_at_byte_offset(&mut self, byte_offset: usize, value: u16) {
        self.vwf_render_mut()
            .set_tile_word_at_byte_offset(byte_offset, value);
    }

    pub(crate) fn set_select_file_choice(&mut self, index: usize, value: u8) {
        self.select_file_menu_mut().set_choice(index, value);
    }

    pub(crate) fn set_select_file_cursor(&mut self, value: u8) {
        self.select_file_menu_mut().set_cursor(value);
    }

    pub(crate) fn clear_select_file_cursor(&mut self) {
        self.select_file_menu_mut().clear_cursor();
    }

    pub(crate) fn clear_select_file_transition_scratch(&mut self) {
        self.select_file_menu_mut().clear_transition_scratch();
    }

    pub(crate) fn increment_select_file_cursor(&mut self) -> u8 {
        self.select_file_menu_mut().increment_cursor()
    }

    pub(crate) fn decrement_select_file_cursor(&mut self) -> u8 {
        self.select_file_menu_mut().decrement_cursor()
    }

    pub(crate) fn clear_select_file_remembered_cursor(&mut self) {
        self.select_file_menu_mut().clear_remembered_cursor();
    }

    pub(crate) fn remember_select_file_cursor(&mut self) {
        self.select_file_menu_mut().remember_current_cursor();
    }

    pub(crate) fn restore_select_file_remembered_cursor(&mut self) {
        self.select_file_menu_mut().restore_remembered_cursor();
    }

    pub(crate) fn set_select_file_target_word(&mut self, value: u16) {
        self.select_file_menu_mut().set_target_word(value);
    }

    pub(crate) fn set_select_file_copy_source_slot(&mut self, slot: u8) {
        self.select_file_menu_mut().set_copy_source_slot(slot);
    }

    pub(crate) fn set_select_file_name_scroll_x(&mut self, value: u16) {
        self.select_file_menu_mut().set_name_scroll_x(value);
    }

    pub(crate) fn clear_select_file_name_entry_state(&mut self) {
        self.select_file_menu_mut().clear_name_entry_state();
    }

    pub(crate) fn set_select_file_name_column(&mut self, value: u8) {
        self.select_file_menu_mut().set_name_column(value);
    }

    pub(crate) fn set_select_file_name_cursor_y(&mut self, value: u8) {
        self.select_file_menu_mut().set_name_cursor_y(value);
    }

    pub(crate) fn step_select_file_name_cursor_y_toward(&mut self, target_y: u8) -> bool {
        self.select_file_menu_mut()
            .step_name_cursor_y_toward(target_y)
    }

    pub(crate) fn move_select_file_name_slot_left_wrapped(&mut self) -> u8 {
        self.select_file_menu_mut().move_name_slot_left_wrapped()
    }

    pub(crate) fn move_select_file_name_slot_right_wrapped(&mut self) -> u8 {
        self.select_file_menu_mut().move_name_slot_right_wrapped()
    }

    pub(crate) fn set_select_file_name_scroll_x_step(&mut self, value: u8) {
        self.select_file_menu_mut().set_name_scroll_x_step(value);
    }

    pub(crate) fn advance_select_file_name_scroll_x_step_by(&mut self, value: u8) -> u8 {
        self.select_file_menu_mut()
            .advance_name_scroll_x_step_by(value)
    }

    pub(crate) fn clear_select_file_name_scroll_y_step(&mut self) {
        self.select_file_menu_mut().clear_name_scroll_y_step();
    }

    pub(crate) fn increment_select_file_name_scroll_y_step(&mut self) -> u8 {
        self.select_file_menu_mut().increment_name_scroll_y_step()
    }

    pub(crate) fn set_select_file_name_row(&mut self, value: u8) {
        self.select_file_menu_mut().set_name_row(value);
    }

    pub(crate) fn set_select_file_name_scroll_x_direction(&mut self, value: u8) {
        self.select_file_menu_mut()
            .set_name_scroll_x_direction(value);
    }

    pub(crate) fn mark_select_file_save_slot_present(&mut self, slot: usize) {
        self.select_file_menu_mut().mark_save_slot_present(slot);
    }

    pub(crate) fn clear_select_file_save_slot_flag(&mut self, slot: usize) {
        self.select_file_menu_mut().clear_save_slot_flag(slot);
    }

    pub(crate) fn clear_select_file_save_slot_flags(&mut self) {
        self.select_file_menu_mut().clear_save_slot_flags();
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

    pub(crate) fn alt_sprite_slot_mut(&mut self, slot: usize) -> NativeCachedSpriteBridgeMut<'_> {
        NativeCachedSpriteBridgeMut::new(
            &mut self.game_state.sprites.cached_sprites,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn cached_sprite_slot(&self, slot: usize) -> CachedSpriteRead {
        self.game_state.sprites.cached_sprites.slot(slot)
    }

    pub(crate) fn cached_sprite_slot_mut(
        &mut self,
        slot: usize,
    ) -> NativeCachedSpriteBridgeMut<'_> {
        NativeCachedSpriteBridgeMut::new(
            &mut self.game_state.sprites.cached_sprites,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn tagalong_slot(&self, slot: usize) -> TagalongSlotRead<'_> {
        TagalongSlotRead::new(&self.game_state.sprites.tagalong_trail, slot)
    }

    pub(crate) fn tagalong_slot_mut(&mut self, slot: usize) -> NativeTagalongSlotBridgeMut<'_> {
        NativeTagalongSlotBridgeMut::new(
            &mut self.game_state.sprites.tagalong_trail,
            &mut self.ram,
            slot,
        )
    }

    pub(crate) fn ancilla_spawn_scratch_mut(
        &mut self,
    ) -> NativeFailedSpinSparkleSpawnBridgeMut<'_> {
        NativeFailedSpinSparkleSpawnBridgeMut::new(
            &mut self.game_state.sprites.failed_spin_sparkle_spawn,
            &mut self.ram,
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

    pub(crate) fn sprite_slot_view(&self, slot: usize) -> NativeSpriteSlotView<'_> {
        self.game_state.sprites.sprite_slots.slot(slot)
    }

    pub(crate) fn sprite_slot_view_mut(&mut self, slot: usize) -> NativeSpriteSlotBridgeMut<'_> {
        self.game_state
            .sprites
            .sprite_slots
            .slot_mut(&mut self.ram, slot)
    }

    pub(crate) fn overlord_slot_view(&self, slot: usize) -> NativeOverlordSlotView<'_> {
        self.game_state.sprites.overlord_slots.slot(slot)
    }

    pub(crate) fn overlord_slot_view_mut(
        &mut self,
        slot: usize,
    ) -> NativeOverlordSlotBridgeMut<'_> {
        self.game_state
            .sprites
            .overlord_slots
            .slot_mut(&mut self.ram, slot)
    }

    pub(crate) fn ancilla_slot_view(&self, slot: usize) -> NativeAncillaSlotView<'_> {
        self.game_state.sprites.ancilla_slots.slot(slot)
    }

    pub(crate) fn ancilla_slot_view_mut(&mut self, slot: usize) -> NativeAncillaSlotBridgeMut<'_> {
        self.game_state
            .sprites
            .ancilla_slots
            .slot_mut(&mut self.ram, slot)
    }

    pub(crate) fn garnish_slot_view(&self, slot: usize) -> NativeGarnishSlotView<'_> {
        self.game_state.sprites.garnish_slots.slot(slot)
    }

    pub(crate) fn garnish_slot_view_mut(&mut self, slot: usize) -> NativeGarnishSlotBridgeMut<'_> {
        self.game_state
            .sprites
            .garnish_slots
            .slot_mut(&mut self.ram, slot)
    }

    pub(crate) fn sprite_system_mut(&mut self) -> NativeSpriteSystemBridgeMut<'_> {
        NativeSpriteSystemBridgeMut::new(
            &mut self.game_state.sprites.system,
            &mut self.game_state.sprites.sprite_slots,
            &mut self.ram,
        )
    }

    pub(crate) fn set_overworld_sprite_presence_marker(&mut self, index: usize, value: u8) {
        self.overworld_sprite_presence_mut()
            .set_marker(index, value);
    }

    pub(crate) fn clear_overworld_sprite_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
        self.overworld_sprite_loaded_mut()
            .clear_loaded_mask(block, loaded_mask);
    }

    pub(crate) fn clear_overworld_sprite_loaded_mask_wrapped(
        &mut self,
        block: u16,
        loaded_mask: u8,
    ) {
        // C computes the SNES address `addr = 0xEF80 + (blk >> 3)` in 16-bit
        // (so it wraps mod 0x10000), THEN dereferences `&g_ram[addr + 0x10000]`.
        // The bank add happens AFTER the 16-bit wrap, so a large `blk` whose
        // 16-bit address wraps below 0xEF80 lands in 0x10000..=0x10F7F (the BG
        // char buffer), not in low WRAM. Replicate that exact two-step math:
        // mask the 16-bit address first, then add the 0x10000 bank.
        let addr16 = 0xEF80u16.wrapping_add(block >> 3);
        let address = 0x10000 + usize::from(addr16);
        self.ram[address] &= !loaded_mask;
        let in_table = match address.checked_sub(OVERWORLD_SPRITE_WAS_LOADED) {
            Some(index) if index < crate::game_state::OVERWORLD_SPRITE_FLAG_COUNT => {
                self.game_state
                    .sprites
                    .overworld_sprite_loaded
                    .clear_loaded_mask(block, loaded_mask);
                true
            }
            _ => false,
        };
        // Legacy parity quirk: for a large `block` (e.g. a killed dungeon sprite whose
        // load block is near 0xffff) the wrapped address can spill OUT of the
        // overworld-sprite-loaded table and land deep in low WRAM modeled by the
        // live sprite slots (SPRITE_FLAGS4 at 0xf60 == OVERWORLD_SPRITE_WAS_LOADED +
        // 0x1fe0 wrapped). The raw `&=` above already wrote that byte, but the
        // sprite-slot native model didn't see it, so its bulk projection would
        // re-stamp the stale value on a later slot's sync. Resync the live slots
        // from RAM so the direct write sticks (matches the legacy baseline raw RAM).
        if !in_table {
            self.sprite_system_mut().reload_live_slots_from_ram();
        }
    }

    pub(crate) fn set_overworld_sprite_loaded_mask(&mut self, block: u16, loaded_mask: u8) {
        self.overworld_sprite_loaded_mut()
            .set_loaded_mask(block, loaded_mask);
    }

    pub(crate) fn clear_all_overworld_sprite_loaded_masks(&mut self) {
        self.overworld_sprite_loaded_mut().clear_all();
    }

    pub(crate) fn set_trinexx_red_shell_palette_delay(&mut self, value: u8) {
        self.trinexx_palette_bridge_mut().set_red_shell_delay(value);
    }

    pub(crate) fn set_trinexx_blue_shell_palette_delay(&mut self, value: u8) {
        self.trinexx_palette_bridge_mut()
            .set_blue_shell_delay(value);
    }

    pub(crate) fn set_trinexx_red_shell_palette_step(&mut self, value: u8) {
        self.trinexx_palette_bridge_mut().set_red_shell_step(value);
    }

    pub(crate) fn set_trinexx_blue_shell_palette_step(&mut self, value: u8) {
        self.trinexx_palette_bridge_mut().set_blue_shell_step(value);
    }

    pub(crate) fn decrement_trinexx_red_shell_palette_delay(&mut self) {
        self.trinexx_palette_bridge_mut()
            .decrement_red_shell_delay();
    }

    pub(crate) fn decrement_trinexx_blue_shell_palette_delay(&mut self) {
        self.trinexx_palette_bridge_mut()
            .decrement_blue_shell_delay();
    }

    pub(crate) fn increment_trinexx_red_shell_palette_step(&mut self) -> u8 {
        self.trinexx_palette_bridge_mut().increment_red_shell_step()
    }

    pub(crate) fn increment_trinexx_blue_shell_palette_step(&mut self) -> u8 {
        self.trinexx_palette_bridge_mut()
            .increment_blue_shell_step()
    }

    pub(crate) fn set_spotlight_y_lower(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_y_lower(value);
    }

    pub(crate) fn set_spotlight_y_upper(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_y_upper(value);
    }

    pub(crate) fn set_spotlight_window_x_center(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_window_x_center(value);
    }

    pub(crate) fn set_spotlight_window_state(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_window_state(value);
    }

    pub(crate) fn set_spotlight_window_radius(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_window_radius(value);
    }

    pub(crate) fn set_spotlight_window_y_buffer(&mut self, value: u16) {
        self.spotlight_hdma_mut().set_window_y_buffer(value);
    }

    pub(crate) fn decrement_spotlight_window_y_buffer(&mut self) -> u16 {
        self.spotlight_hdma_mut().decrement_window_y_buffer()
    }

    pub(crate) fn set_spotlight_window_radius_byte(&mut self, value: u8) {
        self.spotlight_hdma_mut().set_window_radius_byte(value);
    }

    pub(crate) fn set_spotlight_window_state_byte(&mut self, value: u8) {
        self.spotlight_hdma_mut().set_window_state_byte(value);
    }

    pub(crate) fn set_spotlight_window_y_buffer_byte(&mut self, value: u8) {
        self.spotlight_hdma_mut().set_window_y_buffer_byte(value);
    }

    pub(crate) fn increment_spotlight_window_y_buffer_byte(&mut self) {
        self.spotlight_hdma_mut().increment_window_y_buffer_byte();
    }

    pub(crate) fn shr_spotlight_window_radius_byte(&mut self, shift: u8) {
        self.spotlight_hdma_mut().shr_window_radius_byte(shift);
    }

    pub(crate) fn add_spotlight_window_radius_byte(&mut self, value: u8) {
        self.spotlight_hdma_mut().add_window_radius_byte(value);
    }

    pub(crate) fn spotlight_hdma_table_dynamic_entry(&self, index: usize) -> u16 {
        self.game_state
            .display
            .spotlight_hdma
            .hdma_table_dynamic_entry(index)
    }

    pub(crate) fn set_spotlight_hdma_table_dynamic_entry(&mut self, index: usize, value: u16) {
        self.spotlight_hdma_mut()
            .set_hdma_table_dynamic_entry(index, value);
    }

    pub(crate) fn clear_spotlight_hdma_table_dynamic(&mut self, count: usize) {
        self.spotlight_hdma_mut().clear_hdma_table_dynamic(count);
    }

    pub(crate) fn clear_spotlight_hdma_table_dynamic_range(&mut self, start: usize, count: usize) {
        self.spotlight_hdma_mut()
            .clear_hdma_table_dynamic_range(start, count);
    }

    fn restore_spotlight_hdma_from_saveload_buffer(&mut self) {
        // C copies SAVELOAD_HDMA_TABLE[0..224] -> HDMA_TABLE_DYNAMIC[0..224] as a raw ram copy,
        // leaving the off-screen entries 224-239 at their loaded-snapshot value. Routing through
        // the native spotlight table instead re-projects ITS stale 224-239 (0xff00) over the
        // snapshot's zeros -> a 1390-frame parity divergence in the off-screen HDMA scanlines
        // (page 0x1dc00). Do the raw ram copy; load_snes_state's following
        // sync_native_game_state_from_ram reloads the native table from this ram.
        self.spotlight_hdma_mut()
            .copy_saveload_buffer_to_dynamic_table_ram(224);
    }

    fn backup_spotlight_hdma_to_saveload_buffer(&mut self) {
        self.spotlight_hdma_mut()
            .backup_dynamic_table_to_saveload_buffer(224);
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

    /// Single-byte HDMA scratch (0x654) that the C-style snapshot save/restore
    /// also leaves divergent on resume (a known tiny artifact). Restoring it keeps
    /// resume byte-faithful; it is otherwise transient HDMA scratch.
    /// Returns the pristine scratch bytes: the contiguous 0x1b00 region followed by
    /// one trailing byte for 0x654.
    pub fn saveload_hdma_scratch_bytes(&self) -> Vec<u8> {
        SpotlightHdmaState::saveload_scratch_bytes(&self.ram)
    }

    pub fn restore_saveload_hdma_scratch_bytes(&mut self, bytes: &[u8]) {
        SpotlightHdmaState::restore_saveload_scratch_bytes(&mut self.ram, bytes);
    }

    /// Byte extent of the live spotlight HDMA dynamic table (0x1dba0, 240 words).
    /// The C-style saveload reconstructs the native spotlight dynamic table from
    /// the LOSSY SAVELOAD_HDMA_TABLE projection (only 224 words round-trip, and the
    /// native sync re-derives entries), so a resumed run's native spotlight backing
    /// can differ from a continuous run. We capture the live table bytes and, on
    /// load, both restore them to RAM and re-sync the native model directly from
    /// them, making the spotlight backing byte-faithful.
    pub const HDMA_DYNAMIC_TABLE_LEN: usize = SpotlightHdmaState::DYNAMIC_TABLE_LEN;

    pub fn hdma_dynamic_table_bytes(&self) -> Vec<u8> {
        SpotlightHdmaState::dynamic_table_bytes(&self.ram)
    }

    pub fn restore_hdma_dynamic_table_bytes(&mut self, bytes: &[u8]) {
        self.spotlight_hdma_mut().restore_dynamic_table_bytes(bytes);
    }

    pub(crate) fn project_spotlight_dynamic_hdma_table_to_reserved(&mut self, count: usize) {
        self.spotlight_hdma_mut()
            .project_dynamic_table_to_reserved_hdma_table(count);
    }

    pub(crate) fn overworld_palette_backup_mut(
        &mut self,
    ) -> NativeOverworldPaletteBackupBridgeMut<'_> {
        NativeOverworldPaletteBackupBridgeMut::new(
            &mut self.game_state.display.overworld_palette_backup,
            &mut self.ram,
        )
    }

    pub(crate) fn set_overworld_main_indoors_palette_backup(&mut self, value: u8) {
        self.overworld_palette_backup_mut()
            .set_main_indoors_backup(value);
    }

    pub(crate) fn set_overworld_aux3_bg_palette_7_backup(&mut self, value: u8) {
        self.overworld_palette_backup_mut()
            .set_aux3_bg_palette_7_backup(value);
    }

    pub(crate) fn set_overworld_main_indoors_copy_palette_backup(&mut self, value: u8) {
        self.overworld_palette_backup_mut()
            .set_main_indoors_copy_backup(value);
    }

    pub fn set_overworld_map16_load_state(&mut self, state: OverworldMap16LoadState) {
        self.overworld_map16_mut().set_active_load(state);
    }

    pub fn overworld_prev_map16_load_state(&self) -> OverworldMap16LoadState {
        self.game_state.world.overworld.map16.previous_load
    }

    pub fn set_overworld_prev_map16_load_state(&mut self, state: OverworldMap16LoadState) {
        self.overworld_map16_mut().set_previous_load(state);
    }

    pub fn overworld_spexit_map16_src_off(&self) -> u16 {
        self.game_state.world.overworld.map16.special_exit_src_off
    }

    pub fn set_overworld_spexit_map16_src_off(&mut self, src_off: u16) {
        self.overworld_map16_mut().set_special_exit_src_off(src_off);
    }

    pub fn overworld_exit_map16_src_off(&self) -> u16 {
        self.game_state.world.overworld.map16.exit_src_off
    }

    pub fn set_overworld_exit_map16_src_off(&mut self, src_off: u16) {
        self.overworld_map16_mut().set_exit_src_off(src_off);
    }

    pub fn small_overworld_map16_scroll_backup_state(
        &self,
    ) -> SmallOverworldMap16ScrollBackupState {
        self.game_state.world.overworld.map16.small_scroll_backup
    }

    pub fn set_small_overworld_map16_scroll_backup_state(
        &mut self,
        state: SmallOverworldMap16ScrollBackupState,
    ) {
        self.overworld_map16_mut().set_small_scroll_backup(state);
    }

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

    #[track_caller]
    pub(crate) fn assert_native_frame_state_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.frame,
            crate::game_state::FrameState::load_from_ram(&self.ram),
            "native frame state diverged from compatibility RAM",
        );
    }

    #[track_caller]
    pub(crate) fn assert_native_world_location_state_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.world.location,
            crate::game_state::WorldLocationState::load_from_ram(&self.ram),
            "native world location state diverged from compatibility RAM",
        );
    }

    #[track_caller]
    pub(crate) fn assert_native_display_state_matches_ram(&self) {
        self.game_state
            .display
            .debug_assert_core_matches_ram(&self.ram);
    }

    #[track_caller]
    pub(crate) fn assert_native_messaging_render_buffer_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.messaging.render_buffer,
            crate::game_state::MessagingRenderBufferState::load_from_ram(&self.ram),
            "native messaging render buffer diverged from compatibility RAM",
        );
    }

    #[track_caller]
    pub(crate) fn assert_native_select_file_menu_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.messaging.select_file_menu,
            crate::game_state::SelectFileMenuState::load_from_ram(&self.ram),
            "native select-file menu state diverged from compatibility RAM",
        );
    }

    #[track_caller]
    pub(crate) fn assert_native_dungeon_map_display_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.dungeon_map_display,
            crate::game_state::DungeonMapDisplayState::load_from_ram(&self.ram),
            "native dungeon-map display state diverged from compatibility RAM",
        );
    }

    #[track_caller]
    pub(crate) fn assert_native_vwf_render_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.messaging.vwf_render,
            crate::game_state::VwfRenderState::load_from_ram(&self.ram),
            "native VWF render state diverged from compatibility RAM",
        );
    }

    #[track_caller]
    pub(crate) fn assert_native_save_progress_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.inventory.save_progress,
            crate::game_state::SaveProgressState::load_from_ram(&self.ram),
            "native save-progress state diverged from compatibility RAM",
        );
    }

    pub fn sync_overworld_map16_state_from_ram(&mut self) {
        self.overworld_map16_mut().sync_from_ram();
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
            dialogue_fast_forward_hold_active: false,
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
            dialogue_oam_publication_phase: DialogueOamPublicationPhase::Idle,
            dma: DmaState::new(),
            frame_ctr_dbg: 0,
            previous_host_controller_input: 0,
            rom: Vec::new(),
            assets: None,
            gloves_color: default_gloves_color(),
            initialized: false,
            apply_links_movement_to_camera_called: false,
            wanted_zelda_features: 0,
            state_recorder: StateRecorder::default(),
            dialogue_blk_index: 0,
            dialogue_font_blk_index: 0,
            dialogue_flags: 0,
            rom_startup_timing: false,
            rom_load_partial_nmi_this_frame: false,
            rom_lag_frame_skip_oam_dma: false,
            intro_initialization_work_frames_pending: 0,
            intro_initialization_reset_obj_control_pending: false,
            rom_reset_frame_delay: 0,
            intro_memory_darken_frame_delay: 0,
            intro_poly_thread_initialization_phase: 0,
            attract_init_graphics_phase: 0,
            attract_first_story_render_delay: 0,
            game_execution_scheduler: GameExecutionScheduler::default(),
            active_dungeon_sprite_main_return: None,
            next_display_vram_generation: DisplayVramGeneration::default(),
            publish_live_hud_vram_on_next_capture: false,
            next_display_animated_bg_scanout_generation: None,
            next_display_bg_scroll_generation: DisplayBgScrollGeneration::default(),
            next_display_obj_scanout_generation: None,
            next_display_obj_memory_generation: None,
            next_display_interrupted_item_receipt_obj_cache: false,
            link_obj_dma_completed_this_frame: false,
            next_display_spotlight_scanout: None,
            active_display_obj_generation: DisplayObjGeneration::default(),
            room_72_interrupted_main_prefix_oam_offset_active: false,
            next_overworld_sprite_reload_entry_phase: None,
            joypad_sampled_before_main: false,
            audio_nmi_processed_before_main: false,
            dungeon_landing_wipe_return_slices_remaining: 0,
            dungeon_exit_spotlight_table_delay: 0,
            dungeon_exit_spotlight_resume_module: false,
            iris_spotlight_goal_transition_pending: false,
            normal_dialogue_initialization_phase: 0,
            hud_tilemap_nmi_publication_phase: 0,
            intro_poly_upload_delay: 0,
            intro_sprite_animation_start_delay: 0,
            display_snapshot: None,
            visible_display_snapshot: None,
            last_presented_oam: None,
            last_presented_obj_vram: None,
            presented_history_host_frame: None,
            staged_presented_oam: None,
            staged_presented_obj_vram: None,
            last_presented_vram_chr_source: None,
            last_presented_vram_chr_preview_source: None,
            staged_presented_vram_chr_source: None,
            staged_presented_vram_chr_preview_source: None,
            deferred_display_snapshot: None,
            attract_map_hdma_projection_before: None,
            pre_main_graphics_dma: None,
            pre_nmi_animated_bg_scanout: None,
            nmi_forced_blank_scanlines_pending: 0,
            nmi_forced_blank_from_scanline_pending: None,
            nmi_active_display_blanking_candidate: NmiActiveDisplayBlanking::default(),
            active_display_force_blank_event: None,
            last_sprite_main_timing_workload: None,
            spotlight_hdma_reset_prefix: None,
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
            replay_reload_file_select_stall: 0,
            replay_reopened_lamp_prompt: false,
            ending_coords: sprite::PrepOamCoordsRet::default(),
            intro_poly_vram_history: Vec::new(),
            intro_poly_presented_vram: None,
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
        self.intro_poly_thread_initialization_phase = 0;
        self.attract_init_graphics_phase = 0;
        self.attract_first_story_render_delay = 0;
        self.game_execution_scheduler.reset();
        self.joypad_sampled_before_main = false;
        self.audio_nmi_processed_before_main = false;
        self.dungeon_landing_wipe_return_slices_remaining = 0;
        self.dungeon_exit_spotlight_table_delay = 0;
        self.dungeon_exit_spotlight_resume_module = false;
        self.iris_spotlight_goal_transition_pending = false;
        self.normal_dialogue_initialization_phase = 0;
        self.hud_tilemap_nmi_publication_phase = 0;
        self.intro_sprite_animation_start_delay = 0;
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
        self.intro_poly_presented_vram = None;
        self.sync_overworld_map16_state_from_ram();
        self.display_snapshot = None;
        self.visible_display_snapshot = None;
        self.last_presented_oam = None;
        self.last_presented_obj_vram = None;
        self.presented_history_host_frame = None;
        self.staged_presented_oam = None;
        self.staged_presented_obj_vram = None;
        self.last_presented_vram_chr_source = None;
        self.last_presented_vram_chr_preview_source = None;
        self.staged_presented_vram_chr_source = None;
        self.staged_presented_vram_chr_preview_source = None;
        self.deferred_display_snapshot = None;
        self.spotlight_hdma_reset_prefix = None;
        self.emu_synchronize_whole_state();
    }

    pub fn set_rom_startup_timing(&mut self, enabled: bool) {
        self.rom_startup_timing = enabled;
        self.zelda_set_rom_startup_audio_phase(enabled);
        if !enabled {
            self.rom_reset_frame_delay = 0;
            self.intro_initialization_work_frames_pending = 0;
            self.intro_initialization_reset_obj_control_pending = false;
            self.intro_memory_darken_frame_delay = 0;
            self.intro_poly_thread_initialization_phase = 0;
            self.attract_init_graphics_phase = 0;
            self.attract_first_story_render_delay = 0;
            self.game_execution_scheduler.reset();
            self.joypad_sampled_before_main = false;
            self.audio_nmi_processed_before_main = false;
            self.dungeon_landing_wipe_return_slices_remaining = 0;
            self.dungeon_exit_spotlight_table_delay = 0;
            self.dungeon_exit_spotlight_resume_module = false;
            self.iris_spotlight_goal_transition_pending = false;
            self.normal_dialogue_initialization_phase = 0;
            self.hud_tilemap_nmi_publication_phase = 0;
            self.intro_sprite_animation_start_delay = 0;
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
            self.intro_poly_presented_vram = None;
            self.display_snapshot = None;
            self.visible_display_snapshot = None;
            self.last_presented_oam = None;
            self.last_presented_obj_vram = None;
            self.presented_history_host_frame = None;
            self.staged_presented_oam = None;
            self.staged_presented_obj_vram = None;
            self.last_presented_vram_chr_source = None;
            self.last_presented_vram_chr_preview_source = None;
            self.staged_presented_vram_chr_source = None;
            self.staged_presented_vram_chr_preview_source = None;
            self.deferred_display_snapshot = None;
            self.spotlight_hdma_reset_prefix = None;
        } else if !self.game_state.display.has_animated_tile_data_source() {
            self.rom_reset_frame_delay = configured_rom_reset_frame_delay();
        }
    }

    /// Restores the live-ROM frame scheduling mode after deserializing a
    /// checkpoint without resetting the already-restored audio sequencer.
    ///
    /// `rom_startup_timing` is host runtime policy rather than game state, so
    /// it is intentionally omitted from `ZeldaState`'s serialized payload.
    /// Calling `set_rom_startup_timing(true)` here would also reapply the SPC
    /// bootstrap phase and corrupt the checkpoint's exact audio position.
    pub fn restore_live_rom_timing_after_checkpoint(&mut self) {
        let completed_scanout = self.dialogue_text_scanout_from_render_buffer();
        self.dialogue_scroll_machine_mut()
            .restore_transient_after_checkpoint(completed_scanout);
        self.rom_startup_timing = true;
    }

    /// Paired emulator checkpoints may only be captured between translated
    /// game calls. The execution scheduler represents a suspended 65816 call
    /// stack and is intentionally absent from ordinary playable save states.
    pub fn paired_resume_cpu_boundary_is_quiescent(&self) -> bool {
        self.game_execution_scheduler.is_idle() && self.active_dungeon_sprite_main_return.is_none()
    }

    pub(super) fn rom_startup_timing(&self) -> bool {
        self.rom_startup_timing
    }

    pub(super) fn schedule_spotlight_iteration_return(&mut self, iteration: SpotlightIteration) {
        if !self.rom_startup_timing() {
            return;
        }
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishSpotlightIteration { iteration },
            SPOTLIGHT_ITERATION_SUFFIX_NMI_SLICES,
        );
    }

    pub(super) fn stage_spotlight_scanout_for_next_display(&mut self) {
        self.next_display_spotlight_scanout = Some(LiveSpotlightScanout::capture(self));
    }

    /// With the maximal spotlight table, the module-15 sub-0 call's first
    /// IrisSpotlight_ConfigureTable build is interrupted by vblank. The entry
    /// frame runs only the PrepExit prefix; the table copy, first radius
    /// write, submodule advance, and Link/OAM suffix complete on the next
    /// host frame through the scheduled continuation.
    pub(super) fn begin_dungeon_exit_spotlight_entry(&mut self, vertical_center: u16) -> bool {
        if !self.rom_startup_timing()
            || !rom_dungeon_exit_spotlight_entry_build_crosses_vblank(vertical_center)
        {
            return false;
        }
        self.game_execution_scheduler
            .schedule_work(GameWorkContinuation::FinishDungeonExitSpotlightEntry, 1);
        true
    }

    pub(super) fn schedule_dungeon_landing_wipe_return(&mut self, nmi_slices: u8) {
        if !self.rom_startup_timing() {
            return;
        }
        self.dungeon_landing_wipe_return_slices_remaining = nmi_slices;
        // Module07_0F authored Link's OAM and OBJ tiles after the active
        // scanout's DMA boundary. Retain the captured hardware generation
        // while its continuation is waiting; the completion path runs
        // NMI_PrepareSprites and publishes the live generation explicitly.
        self.next_display_vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
        self.next_display_obj_scanout_generation = Some(ObjScanoutGenerations::coherent(
            GraphicsDmaGeneration::HostBoundaryBeforeMain,
        ));
    }

    #[cold]
    #[inline(never)]
    fn resume_dungeon_landing_wipe_return(
        &mut self,
        input: u16,
        oam_dma_source: Option<&[u8]>,
    ) -> bool {
        if !self.rom_startup_timing() || self.dungeon_landing_wipe_return_slices_remaining == 0 {
            return false;
        }

        let vertical_center = spotlight_vertical_center(
            self.game_state.player.follower_link.y(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        let long_workload = spotlight_table_has_long_nmi_workload(vertical_center);
        // Only a long goal calculation spans a second return slice. That
        // waiting scanout keeps the last published circle; the completion
        // below publishes the fully returned module state normally.
        let waiting_publication = (self.iris_spotlight_goal_transition_pending && long_workload)
            .then_some(DisplaySnapshotPublication::RetainPublished);
        self.dungeon_landing_wipe_return_slices_remaining -= 1;
        if self.dungeon_landing_wipe_return_slices_remaining != 0 {
            self.capture_display_snapshot_with_override(waiting_publication);
            self.interrupt_nmi(input, oam_dma_source, false);
            return true;
        }
        if self.iris_spotlight_goal_transition_pending {
            self.iris_spotlight_goal_transition_pending = false;
            if dungeon_landing_goal_reset_preserves_scanout_prefix(vertical_center) {
                self.spotlight_hdma_reset_prefix = Some(std::array::from_fn(|index| {
                    self.spotlight_hdma_table_dynamic_entry(index)
                }));
            }
            self.complete_iris_spotlight_goal_transition();
            self.complete_module07_0f_operate_spotlight_suffix();
        }
        self.complete_module07_dungeon_after_submodule();
        self.nmi_prepare_sprites();
        self.clear_nmi_update_latch();
        self.capture_display_snapshot();
        self.interrupt_nmi(input, oam_dma_source, false);
        true
    }

    pub(super) fn begin_pre_overworld_properties_work(
        &mut self,
        overworld_screen: u8,
        animated_tiles: u8,
    ) -> bool {
        if !self.rom_startup_timing() || self.game_state.frame.main_module != 8 {
            return false;
        }
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishPreOverworldProperties {
                overworld_screen,
                animated_tiles,
            },
            PRE_OVERWORLD_PROPERTIES_NMI_SLICES,
        );
        true
    }

    pub(super) fn begin_pre_overworld_overlays_work(&mut self) -> bool {
        if !self.rom_startup_timing() || self.game_state.frame.main_module != 8 {
            return false;
        }
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishPreOverworldOverlays,
            PRE_OVERWORLD_OVERLAYS_NMI_SLICES,
        );
        true
    }

    pub(super) fn begin_pre_overworld_screen_build_work(&mut self) -> bool {
        if !self.rom_startup_timing() || self.game_state.frame.main_module != 8 {
            return false;
        }
        let timing =
            overworld_map_and_sprite_graphics_timing(self.overworld_map_graphics_workload());
        self.game_execution_scheduler
            .schedule_work_before_trailing_nmi(
                GameWorkContinuation::FinishPreOverworldScreenBuild,
                timing.quadrant_load_nmi_slices + timing.screen_map_and_sprite_gfx_tail_nmi_slices,
            );
        true
    }

    pub(super) fn begin_selected_game_load(&mut self) {
        self.enable_force_blank();
        self.game_execution_scheduler.schedule_selected_game_load();
        // The ROM starts the heavy save-file load on this frame; its NMI is
        // PARTIAL (no Main_PrepSpritesForNmi — Snes9x holds 0xc00d here while
        // rust's game loop otherwise decrements once more on this entry frame,
        // the single event that left the BG-tile animation phase one step ahead
        // for the rest of the route, surfacing at frame 14661).
        self.rom_load_partial_nmi_this_frame = true;
    }

    #[doc(hidden)]
    pub fn zelda_debug_selected_game_load_remaining_nmi_slices(&self) -> u8 {
        self.game_execution_scheduler
            .selected_game_load_remaining_nmi_slices()
    }

    pub(super) fn begin_item_receipt_graphics_work(
        &mut self,
        gfx: u8,
        receipt: ItemReceiptReturn,
        caller: ItemReceiptCaller,
    ) -> GameCallStatus {
        if !self.rom_startup_timing() {
            return GameCallStatus::Returned;
        }
        let nmi_slices = rom_item_receipt_graphics_nmi_slices(gfx);
        if nmi_slices == 0 {
            return GameCallStatus::Returned;
        }
        match self.game_state.player.follower_link.item_receipt_method() {
            // The two ROM entry paths whose decompression timing has been
            // measured. Their different entry boundaries are scheduled by
            // their semantic callers, not stored on the continuation.
            0 | 1 => {}
            // Other entry paths remain atomic until their ROM boundary is
            // measured.
            _ => return GameCallStatus::Returned,
        }
        let continuation = match caller {
            ItemReceiptCaller::AtomicCaller => {
                ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx }
            }
            ItemReceiptCaller::UnclePassage { sprite_slot } => {
                let dungeon = self
                    .active_dungeon_sprite_main_return
                    .take()
                    .expect("uncle passage item receipt must suspend a Module 7 sprite loop");
                ItemReceiptGraphicsContinuation::ResumeUnclePassage {
                    receipt,
                    sprite_slot,
                    dungeon,
                }
            }
        };
        let call_status = if matches!(
            continuation,
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { .. }
        ) {
            GameCallStatus::Returned
        } else {
            GameCallStatus::Suspended
        };
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishItemReceiptGraphics { continuation },
            nmi_slices,
        );
        call_status
    }

    pub(super) fn begin_big_key_drop_graphics_work(&mut self, sprite_slot: usize) -> bool {
        if !self.rom_startup_timing() || !self.game_execution_scheduler.is_idle() {
            return false;
        }
        let Some(dungeon) = self.active_dungeon_sprite_main_return.take() else {
            // Big-key graphics can also be prepared by atomic setup paths.
            // Only a live Module 7 sprite loop has a measured resumable caller.
            return false;
        };
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishBigKeyDropGraphics {
                sprite_slot: sprite_slot as u8,
                dungeon,
            },
            BIG_KEY_DROP_GRAPHICS_NMI_SLICES,
        );
        // The entry main slice has already crossed the OAM DMA that published
        // the host-boundary shadow, but it will not reach another sprite-prep
        // epilogue before the decompressor is interrupted. Publish that prior
        // shadow once; the scheduled slices retain it until the call returns.
        self.next_display_obj_scanout_generation = Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposePublishedShadowDma,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        });
        true
    }

    pub(super) fn begin_dungeon_falling_entrance_work(
        &mut self,
        work: DungeonFallingEntranceWork,
    ) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        debug_assert_eq!(self.game_state.frame.main_module, 0x11);
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonFallingEntrance { work },
            work.nmi_slices(),
        );
        true
    }

    pub(super) fn begin_dungeon_supertile_transition_work(
        &mut self,
        work: DungeonSupertileTransitionWork,
    ) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        debug_assert_eq!(self.game_state.frame.main_module, 7);
        if matches!(
            work,
            DungeonSupertileTransitionWork::SpiralRoomInitialization
                | DungeonSupertileTransitionWork::SpiralRoomCallerResume
                | DungeonSupertileTransitionWork::SpiralBgCharacters34
                | DungeonSupertileTransitionWork::SpiralSpriteGraphics
        ) {
            debug_assert_eq!(self.game_state.frame.submodule, 0x0e);
        } else if work == DungeonSupertileTransitionWork::QuadrantTilemapBuild {
            if self.game_state.frame.submodule == 0x0e {
                return false;
            }
            debug_assert_eq!(self.game_state.frame.submodule, 2);
            if self.game_state.world.location.dungeon_room_index() != 0x72 {
                // The ordinary supertile quadrant builder returns within the
                // current host frame. Only room $72 has the measured
                // interrupted state-5/state-10 returns handled by the
                // continuation below; suspending every room delayed state 6
                // and its NMI-owned Link animation by one frame.
                return false;
            }
        } else {
            debug_assert_eq!(self.game_state.frame.submodule, 2);
        }
        let mut nmi_slices = work.nmi_slices();
        if work == DungeonSupertileTransitionWork::RoomLoad
            && self.game_state.world.location.dungeon_room_index() == 0x82
        {
            // Room $82's measured room-object workload crosses one more
            // vblank than the neighboring room-$81 load, despite both using
            // layout $1c.
            nmi_slices = nmi_slices.saturating_add(1);
        }
        if work == DungeonSupertileTransitionWork::RoomLoadCallerResume
            && self.game_state.world.location.dungeon_room_index() == 0x72
        {
            // Room $72's suspended caller returns one vblank later than the
            // default path. The guard-prep suffix, including its two ROM RNG
            // reads, therefore belongs to execution frame 23195 rather than
            // the preceding state-2 frame.
            nmi_slices = nmi_slices.saturating_add(1);
        }
        if work == DungeonSupertileTransitionWork::SpiralSpriteGraphics
            && self.game_state.world.location.dungeon_room_index() == 0x70
            && matches!(
                (
                    self.game_state.dungeon.stair_movement.staircase_index(),
                    self.game_state
                        .dungeon
                        .stair_movement
                        .staircase_lower_level_status(),
                ),
                (0x30, 0) | (0x34, 2)
            )
        {
            // These room-$70 staircase entries reuse a resident sprite set;
            // their conversion completes within the vblank that interrupts
            // the call instead of spanning the default three slices.
            nmi_slices = 1;
        }
        if work == DungeonSupertileTransitionWork::SpiralSpriteGraphics
            && self.game_state.world.location.dungeon_room_index() == 0x80
            && self.game_state.dungeon.stair_movement.staircase_index() == 0x35
        {
            // The reciprocal staircase into room $80 crosses five measured
            // vblanks while loading its sprite set.
            nmi_slices = 5;
        }
        if work == DungeonSupertileTransitionWork::SpiralSpriteGraphics
            && self.game_state.world.location.dungeon_room_index() == 0x71
            && self.game_state.dungeon.stair_movement.staircase_index() == 0x30
            && self
                .game_state
                .dungeon
                .stair_movement
                .staircase_lower_level_status()
                == 0
        {
            // Room $71's matching staircase reuses part of the resident set,
            // but still crosses one more vblank than the room-$70 endpoint.
            nmi_slices = 2;
        }
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonSupertileTransition { work },
            nmi_slices,
        );
        true
    }

    pub(super) fn stage_dungeon_supertile_quadrant_upload_obj_scanout(&mut self) {
        if !self.rom_startup_timing() {
            return;
        }
        self.next_display_obj_memory_generation = Some(DisplayObjGeneration::RetainCapturedOam {
            oam: self.ppu.oam.clone(),
        });
        self.next_display_obj_scanout_generation = Some(ObjScanoutGenerations {
            oam: OamScanoutSource::RetainResidentPpuOam,
            link_obj: GraphicsDmaGeneration::LiveAfterMain,
            link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
        });
    }

    pub(super) fn stage_room_72_supertile_scroll_obj_scanout(&mut self) {
        if !self.rom_startup_timing() || self.game_state.world.location.dungeon_room_index() != 0x72
        {
            return;
        }
        self.next_display_obj_scanout_generation = Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposePublishedShadowDma,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        });
    }

    pub(super) fn stage_room_72_supertile_landing_obj_scanout(&mut self) {
        if !self.rom_startup_timing() || self.game_state.world.location.dungeon_room_index() != 0x72
        {
            return;
        }
        self.next_display_obj_scanout_generation = Some(ObjScanoutGenerations {
            oam: OamScanoutSource::ComposeLiveAfterNmi,
            link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
            link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
        });
    }

    pub(super) fn begin_dungeon_supertile_filtering_return(&mut self) -> bool {
        let room = self.game_state.world.location.dungeon_room_index();
        if !self.rom_startup_timing()
            || self.game_state.frame.main_module != 7
            || self.game_state.frame.submodule != 2
            || !matches!(room, 0x71 | 0x72)
        {
            return false;
        }
        if room == 0x72 {
            // In the north-to-south room-$72 transition the vblank lands in
            // the filtering call itself, before it publishes state 8. Keep
            // the caller-visible state and the already scanned Link OAM until
            // the interrupted return completes on the next host frame.
            self.set_subsubmodule(7);
            if let Some(previously_presented_oam) = self
                .staged_presented_oam
                .clone()
                .or_else(|| self.last_presented_oam.clone())
            {
                let oam = dungeon_supertile_interrupted_filter_oam(
                    &self.ppu.oam,
                    Some(&previously_presented_oam),
                );
                self.next_display_obj_memory_generation =
                    Some(DisplayObjGeneration::RetainCapturedOam { oam });
            }
        }
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonSupertileFilteringReturn,
            DUNGEON_SUPERTILE_FILTERING_RETURN_NMI_SLICES,
        );
        true
    }

    pub(super) fn begin_pre_dungeon_entrance_load_work(&mut self) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        debug_assert_eq!(self.game_state.frame.main_module, 6);
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishPreDungeonEntranceLoad,
            PRE_DUNGEON_ENTRANCE_LOAD_NMI_SLICES,
        );
        true
    }

    pub(super) fn begin_pre_dungeon_song_bank_transfer_work(&mut self) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        debug_assert_eq!(self.game_state.frame.main_module, 7);
        debug_assert_eq!(self.game_state.frame.submodule, 15);
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishPreDungeonSongBankTransfer,
            PRE_DUNGEON_SONG_BANK_TRANSFER_NMI_SLICES,
        );
        true
    }

    pub(super) fn begin_attract_throne_room_work(&mut self) {
        let retained_sprite_subset_2 = self.game_state.sprites.workspace.graphics_subset(2);
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishAttractThroneRoom,
            attract_throne_room_nmi_slices(retained_sprite_subset_2),
        );
    }

    pub(super) fn begin_attract_world_map_work(&mut self) {
        // Snes9x executing the original ROM reaches attract state 4 on host
        // frame 5651. The work starts at frame 5646, so exactly five NMI
        // slices elapse before the world-map continuation runs. Seven slices
        // delayed the live scene by two frames and made the source-native
        // renderer faithfully draw the wrong state.
        self.game_execution_scheduler
            .schedule_work(GameWorkContinuation::FinishAttractWorldMap, 5);
    }

    pub(super) fn begin_attract_world_map_exit_work(&mut self) {
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishAttractWorldMapExit,
            ATTRACT_WORLD_MAP_EXIT_NMI_SLICES,
        );
    }

    pub(super) fn begin_world_map_light_load_work(&mut self) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        // The original CPU enters WorldMap_LoadLightWorldMap after host frame
        // 5934 and does not return until frame 5940. The entry frame performs
        // the first portion of the ROM work; five later NMI slices elapse
        // before the state increment and NMI-7 request become observable.
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishWorldMapLightLoad,
            WORLD_MAP_LIGHT_LOAD_NMI_SLICES,
        );
        true
    }

    pub(super) fn begin_attract_zelda_prison_work(&mut self) {
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishAttractZeldaPrison,
            ATTRACT_ZELDA_PRISON_NMI_SLICES,
        );
    }

    pub(super) fn begin_attract_maiden_warp_work(&mut self) {
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishAttractMaidenWarp,
            ATTRACT_MAIDEN_WARP_NMI_SLICES,
        );
    }

    pub(super) fn begin_attract_end_of_story_work(&mut self) {
        self.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishAttractEndOfStory,
            ATTRACT_END_OF_STORY_NMI_SLICES,
        );
    }

    fn publish_bg_scroll_for_following_scanout(&mut self, scroll: BgScrollRegisterScanout) {
        scroll.publish_to(&mut self.ppu);
    }

    pub(crate) fn dialogue_text_scanout_from_render_buffer(&self) -> DialogueTextScanout {
        let buffer = self.background_character_buffer();
        DialogueTextScanout {
            vram: (0..0x3f0)
                .map(|index| read_word_from_slice(buffer, index * 2))
                .collect(),
            glyph_runs: self.bg3_vwf_glyph_runs.clone(),
            glyph_run_dialogue_offsets: self.bg3_vwf_glyph_run_dialogue_offsets.clone(),
            dialogue_msg_read_pos: self.game_state.messaging.runtime.dialogue_msg_read_pos(),
            dialogue_message_id: self.bg3_vwf_glyph_run_dialogue_message_id,
        }
    }

    pub(crate) fn dialogue_text_scanout_from_published_display(&self) -> DialogueTextScanout {
        DialogueTextScanout {
            vram: self.ppu.vram[0x7c00..0x7ff0].to_vec(),
            glyph_runs: self.published_bg3_vwf_glyph_runs.clone(),
            glyph_run_dialogue_offsets: self.published_bg3_vwf_glyph_run_dialogue_offsets.clone(),
            dialogue_msg_read_pos: self.published_dialogue_msg_read_pos,
            dialogue_message_id: self.published_dialogue_message_id,
        }
    }

    fn dialogue_scroll_machine_mut(&mut self) -> DialogueScrollMachineMut<'_> {
        DialogueScrollMachineMut {
            continuation: &mut self.dialogue_scroll_continuation,
            frozen_scanout: &mut self.dialogue_scroll_frozen_scanout,
            publication: &mut self.dialogue_scanout_ownership,
            completion_scanout: &mut self.dialogue_scroll_completion_scanout,
            staged_completion: &mut self.dialogue_scroll_completion_staged,
        }
    }

    fn dialogue_scroll_phase(&self) -> DialogueScrollPhase {
        dialogue_scroll_phase(
            self.dialogue_scroll_continuation,
            self.dialogue_scanout_ownership,
            self.dialogue_scroll_frozen_scanout.is_some(),
            self.dialogue_scroll_completion_scanout.is_some(),
            self.dialogue_scroll_completion_staged.is_some(),
        )
    }

    pub(crate) fn dialogue_scroll_cpu_is_idle(&self) -> bool {
        self.dialogue_scroll_continuation.is_idle()
    }

    fn dialogue_scroll_is_copying_remaining_pixels(&self) -> bool {
        matches!(
            self.dialogue_scroll_phase(),
            DialogueScrollPhase::CopyingRemainingPixels { .. }
        )
    }

    fn dialogue_scroll_is_return_only(&self) -> bool {
        self.dialogue_scroll_phase() == DialogueScrollPhase::ReturnOnly
    }

    fn dialogue_scroll_is_completion_pending_publication(&self) -> bool {
        self.dialogue_scroll_phase() == DialogueScrollPhase::CompletionPendingPublication
    }

    pub(crate) fn dialogue_scroll_holds_nmi_registers(&self) -> bool {
        matches!(
            self.dialogue_scroll_phase(),
            DialogueScrollPhase::CopyingRemainingPixels { .. } | DialogueScrollPhase::ReturnOnly
        )
    }

    fn schedule_pre_main_caller_continuation(&mut self, continuation: PreMainCallerContinuation) {
        self.game_execution_scheduler
            .schedule_pre_main_caller_continuation(continuation);
    }

    fn pre_main_caller_continuation_is(&self, continuation: PreMainCallerContinuation) -> bool {
        self.game_execution_scheduler
            .pre_main_caller_continuation_is(continuation)
    }

    fn finish_pre_main_caller_continuation(&mut self, expected: PreMainCallerContinuation) {
        self.game_execution_scheduler
            .finish_pre_main_caller_continuation(expected);
    }

    pub(crate) fn begin_dialogue_scroll(
        &mut self,
        generation: DialogueTextGeneration,
        completion_timing: DialogueScrollCompletionTiming,
    ) {
        let frozen_scanout = match generation {
            DialogueTextGeneration::PublishedDisplay => {
                self.dialogue_text_scanout_from_published_display()
            }
            DialogueTextGeneration::CurrentRenderBuffer => {
                self.dialogue_text_scanout_from_render_buffer()
            }
        };
        if std::env::var_os("ZELDA3_DEBUG_SCROLL_RETAIN").is_some() {
            let vram_sum = frozen_scanout
                .vram
                .iter()
                .map(|&word| u64::from(word & 0xff) + u64::from(word >> 8))
                .sum::<u64>();
            eprintln!(
                "scroll_freeze host={} generation={generation:?} vram_sum={vram_sum}",
                self.frame_ctr_dbg,
            );
        }
        self.dialogue_scroll_machine_mut()
            .begin_scroll(frozen_scanout, completion_timing);
    }

    fn finish_dialogue_scroll_remaining_pixels(&mut self) -> DialogueScrollCompletionTiming {
        self.dialogue_scroll_machine_mut().finish_remaining_pixels()
    }

    fn finish_dialogue_scroll_return(&mut self) {
        self.dialogue_scroll_machine_mut().finish_return();
    }

    fn stage_dialogue_scroll_completion_after_return(
        &mut self,
        completed_scanout: DialogueTextScanout,
    ) {
        self.dialogue_scroll_machine_mut()
            .stage_completion_after_return(completed_scanout);
    }

    fn stage_early_dialogue_scroll_completion(&mut self, completed_scanout: DialogueTextScanout) {
        self.dialogue_scroll_machine_mut()
            .stage_early_completion(completed_scanout);
    }

    fn advance_dialogue_scroll_display_boundary(&mut self) {
        self.dialogue_scroll_machine_mut()
            .advance_display_boundary();
    }

    fn color_math_scanout_from_nmi_register_mirrors(&self) -> ColorMathRegisterScanout {
        let color_window = self
            .game_state
            .display
            .palette_filter
            .color_window_selection();
        let color_math = self.game_state.display.palette_filter.color_math_control();
        let mut fixed_color = [
            self.ppu.fixed_color_r,
            self.ppu.fixed_color_g,
            self.ppu.fixed_color_b,
        ];
        for value in [
            self.game_state.display.palette_filter.fixed_color_red(),
            self.game_state.display.palette_filter.fixed_color_green(),
            self.game_state.display.palette_filter.fixed_color_blue(),
        ] {
            if value & 0x20 != 0 {
                fixed_color[0] = value & 0x1f;
            }
            if value & 0x40 != 0 {
                fixed_color[1] = value & 0x1f;
            }
            if value & 0x80 != 0 {
                fixed_color[2] = value & 0x1f;
            }
        }
        ColorMathRegisterScanout {
            windowsel: u32::from(self.game_state.display.bg12_window_selection)
                | (u32::from(self.game_state.display.bg34_window_selection) << 8)
                | (u32::from(self.game_state.display.object_color_window_selection) << 16),
            clip_mode: (color_window & 0xc0) >> 6,
            prevent_math_mode: (color_window & 0x30) >> 4,
            add_subscreen: color_window & 0x02 != 0,
            subtract_color: color_math & 0x80 != 0,
            half_color: color_math & 0x40 != 0,
            math_enabled: color_math & 0x3f,
            fixed_color,
            screen_enabled: [
                self.game_state.display.main_screen_layers,
                self.game_state.display.sub_screen_layers,
            ],
            screen_windowed: [
                self.game_state.display.main_screen_window_layers,
                self.game_state.display.sub_screen_window_layers,
            ],
        }
    }

    fn bg_scroll_scanout_from_nmi_register_mirrors(&self) -> BgScrollRegisterScanout {
        let scroll = &self.game_state.display.ppu_scroll_copy;
        BgScrollRegisterScanout::after_nmi_writes(
            &self.ppu,
            [
                [
                    scroll.bg1_h_copy_low(),
                    scroll.bg1_h_high(),
                    scroll.bg1_v_copy_low(),
                    scroll.bg1_v_high(),
                ],
                [
                    scroll.bg2_h_copy_low(),
                    scroll.bg2_h_high(),
                    scroll.bg2_v_copy_low(),
                    scroll.bg2_v_high(),
                ],
                [
                    scroll.bg3_h_copy2_low(),
                    scroll.bg3_h_high(),
                    scroll.bg3_v_copy2_low(),
                    scroll.bg3_v_high(),
                ],
            ],
        )
    }

    pub(super) fn capture_display_snapshot(&mut self) {
        self.capture_display_snapshot_with_override(None);
    }

    fn capture_display_snapshot_with_override(
        &mut self,
        publication_override: Option<DisplaySnapshotPublication>,
    ) {
        // The native frame identifies the CPU publication phase. It can
        // deliberately lag a direct WRAM module handoff until NMI resumes.
        let frame = self.game_state.frame;
        let publication = publication_override.unwrap_or_else(|| {
            if rom_attract_world_map_display_is_one_frame_deferred(
                frame.main_module,
                frame.submodule,
                self.game_state.ending.attract_scene.sequence(),
                self.game_state.ending.attract_scene.state(),
            ) {
                DisplaySnapshotPublication::AdvanceStaged
            } else {
                rom_display_snapshot_publication(frame.main_module, frame.submodule)
            }
        });
        let spotlight_iteration = self.game_execution_scheduler.spotlight_iteration();
        let spotlight_vertical_center = spotlight_vertical_center(
            self.game_state.player.follower_link.y(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        let spotlight_live_tail_start = spotlight_mixed_scanout_live_tail_start(
            spotlight_vertical_center,
            self.game_state.display.spotlight_hdma.window_radius(),
        );
        let spotlight_table_is_still_building = spotlight_iteration.is_some_and(|iteration| {
            iteration.is_closing()
                && self.game_state.frame.main_module == 15
                && rom_dungeon_exit_spotlight_table_needs_entry_slice(
                    self.game_state.display.spotlight_hdma.window_radius(),
                    spotlight_vertical_center,
                )
        });
        let hdma_table_generation = spotlight_iteration
            .filter(|iteration| {
                iteration.publishes_completed_hdma_table_to_active_scanout()
                    && !spotlight_table_is_still_building
            })
            .map(
                |_| DisplayHdmaTableGeneration::SpotlightPublishedAheadOfSnapshot {
                    active_table: self.hdma_dynamic_table_bytes(),
                },
            );
        let mixed_spotlight_after_projection = spotlight_iteration
            .filter(|iteration| iteration.phase == SpotlightIterationPhase::MixedTailAfterReturn)
            .map(|_| spotlight_hdma_tables_from_ram(&self.ram));
        let landing_spotlight_after_projection =
            (self.dungeon_landing_wipe_return_slices_remaining != 0
                && spotlight_opening_projects_live_tail_before_hdma(
                    self.game_state.display.spotlight_hdma.window_radius(),
                    spotlight_vertical_center,
                ))
            .then(|| spotlight_hdma_tables_from_ram(&self.ram));
        self.capture_display_snapshot_with_publication(publication);
        if let (Some(generation), Some(display)) =
            (hdma_table_generation, self.display_snapshot.as_mut())
        {
            display.hdma_table_generation = generation;
        }
        if let (Some(after_projection), Some(display)) = (
            mixed_spotlight_after_projection,
            self.display_snapshot.as_mut(),
        ) {
            display.hdma_table_generation =
                DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
                    before_projection: spotlight_hdma_tables_from_ram(&display.ram),
                    after_projection,
                    live_tail_start: spotlight_live_tail_start,
                };
        }
        if let (Some(after_projection), Some(display)) = (
            landing_spotlight_after_projection,
            self.display_snapshot.as_mut(),
        ) {
            display.hdma_table_generation =
                DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
                    before_projection: spotlight_hdma_tables_from_ram(&display.ram),
                    after_projection,
                    live_tail_start: spotlight_live_tail_start,
                };
        }
    }

    fn project_following_spotlight_tail_to_active_scanout(
        &mut self,
        phase: SpotlightIterationPhase,
    ) {
        let live_tables = spotlight_hdma_tables_from_ram(&self.ram);
        let before_projection = if phase == SpotlightIterationPhase::MixedTailAfterReturn {
            self.display_snapshot
                .as_ref()
                .map(|display| spotlight_hdma_tables_from_ram(&display.ram))
                .unwrap_or_else(|| live_tables.clone())
        } else {
            live_tables.clone()
        };
        let mut after_projection = live_tables;
        let vertical_center = spotlight_vertical_center(
            self.game_state.player.follower_link.y(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        let radius = self.game_state.display.spotlight_hdma.window_radius();
        let live_tail_start = spotlight_mixed_scanout_live_tail_start(vertical_center, radius);
        if phase == SpotlightIterationPhase::MixedTailAfterReturn {
            // The fixed 448-byte copy crosses HDMA at scanline 221. HDMA has
            // already consumed the published table above that line; from the
            // crossing onward it reads the table which just completed in WRAM.
        } else {
            // Follow the ROM builder's paired lower/upper cursors exactly. The
            // lower cursor starts at max(2*center, 224), so its radial operand
            // is not equivalent to abs(scanline-center) at the bottom edge.
            let mut lower_cursor = vertical_center.wrapping_mul(2).max(224);
            let mut upper_cursor = vertical_center.wrapping_mul(2).wrapping_sub(lower_cursor);
            let y_upper = vertical_center.wrapping_add(radius);
            let mut radial_operand = radius;
            loop {
                let value = if lower_cursor < y_upper {
                    let operand = radial_operand as u8;
                    radial_operand = radial_operand.saturating_sub(1);
                    self.iris_spotlight_calculate_circle_value(operand)
                } else {
                    0x00ff
                };
                for scanline in [upper_cursor, lower_cursor] {
                    let scanline = scanline as usize;
                    if (live_tail_start..224).contains(&scanline) {
                        for table in &mut after_projection {
                            let offset = scanline * 2;
                            table[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
                        }
                    }
                }
                if upper_cursor == vertical_center {
                    break;
                }
                upper_cursor = upper_cursor.wrapping_add(1);
                lower_cursor = lower_cursor.wrapping_sub(1);
            }
        }
        if let Some(display) = self.display_snapshot.as_mut() {
            display.hdma_table_generation =
                DisplayHdmaTableGeneration::SpotlightProjectionDuringScanout {
                    before_projection,
                    after_projection,
                    live_tail_start,
                };
        }
    }

    fn capture_display_snapshot_with_publication(
        &mut self,
        publication: DisplaySnapshotPublication,
    ) {
        if publication != DisplaySnapshotPublication::RetainPublished {
            self.commit_retiring_display_window_latches();
        }
        let diagnostics = CaptureDisplayDiagnostics::from_env();
        self.ppu.refresh_brightness_cache();
        // The upcoming NMI may latch a fresh pre-upload CGRAM image; the one
        // from the previous frame has been consumed by that frame's renders.
        self.cgram_upload_latch = None;
        // Advance the coherent CPU/scanout pair once per display boundary. A
        // staged completion becomes visible for exactly one scanout, while an
        // in-flight copy retains its frozen hardware generation.
        self.advance_dialogue_scroll_display_boundary();
        let frame = self.game_state.frame;
        if diagnostics.attract_timeline && (5640..=5700).contains(&self.frame_ctr_dbg) {
            let trace = format!(
                "attract_display_capture host={} state={} seq={} zoom={:02x} timer={} brightness={} c0={:04x} c1={:04x} fixed={:02x},{:02x},{:02x} math={:02x}/{:02x} table0={:04x} ppu_a={:04x}",
                self.frame_ctr_dbg,
                self.game_state.ending.attract_scene.state(),
                self.game_state.ending.attract_scene.sequence(),
                self.game_state.ending.attract_scene.mode7_zoom_timer(),
                self.game_state.ending.attract_scene.scene_timer(),
                self.game_state.display.screen_brightness,
                self.ppu.cgram[0],
                self.ppu.cgram[1],
                self.ppu.fixed_color_r,
                self.ppu.fixed_color_g,
                self.ppu.fixed_color_b,
                self.ppu.math_enabled,
                self.ppu.prevent_math_mode,
                self.spotlight_hdma_table_dynamic_entry(0),
                self.ppu.m7_matrix[0] as u16,
            );
            eprintln!("{trace}");
            append_parity_trace("attract-display-timeline.trace", &trace);
        }
        if diagnostics.frame_boundary {
            eprintln!(
                "frame_boundary_before host={} main={:02x} sub={:02x} frame_counter={:02x} work={:?} caller={:?} dialogue_init={} dialogue_scroll={:?} next_obj={:?} bg1=({:04x},{:04x}) link_dma_countdown={:04x} latch={} pending={} target={:04x} disable={:02x} dialogue_runs=authored:{}/published:{}/display:{}",
                self.frame_ctr_dbg,
                frame.main_module,
                frame.submodule,
                frame.frame_counter,
                self.game_execution_scheduler.current_work(),
                self.game_execution_scheduler.pre_main_caller_continuation(),
                self.normal_dialogue_initialization_phase,
                self.dialogue_scroll_phase(),
                self.next_display_obj_scanout_generation,
                self.ppu.bg_layer[0].h_scroll,
                self.ppu.bg_layer[0].v_scroll,
                read_le_u16(&self.ram, LINK_DMA_COUNTDOWN),
                self.game_state.display.nmi_update_is_latched(),
                self.game_state.display.pending_nmi_subroutine,
                self.game_state.display.nmi_load_target_address,
                self.game_state.display.core_update_disable_flag,
                self.bg3_vwf_glyph_runs.len(),
                self.published_bg3_vwf_glyph_runs.len(),
                self.display_snapshot
                    .as_ref()
                    .map_or(0, |snapshot| snapshot.published_bg3_vwf_glyph_runs.len()),
            );
        }
        // Unlike the CPU phase above, transition identity belongs to the WRAM
        // generation copied into this snapshot. Keep both meanings explicit:
        // collapsing them regresses the later staged landing-wipe cadence.
        let captured_frame = crate::game_state::FrameState::load_from_ram(&self.ram);
        let captured_messaging = crate::game_state::MessagingState::load_from_ram(&self.ram);
        let published_frame = self
            .display_snapshot
            .as_ref()
            .map(|published| crate::game_state::FrameState::load_from_ram(&published.ram));
        let publish_live_animated_bg_on_submodule6_entry =
            published_frame.is_some_and(|published| {
                rom_overworld_entry_to_submodule6_publishes_live_animated_bg(
                    published,
                    captured_frame,
                )
            });
        let transition_entry_obj = self.display_snapshot.as_ref().and_then(|published| {
            let published_frame = published_frame?;
            rom_dungeon_falling_entry_retains_published_obj_generation(
                published_frame.main_module,
                published_frame.submodule,
                captured_frame.main_module,
                captured_frame.submodule,
            )
            .then(|| {
                (
                    published.ppu.oam.clone(),
                    published.ppu.vram[0x4000..0x4400].to_vec(),
                )
            })
        });
        let obj_generation = self
            .next_display_obj_memory_generation
            .take()
            .or_else(|| {
                transition_entry_obj
                    .map(|(oam, vram)| DisplayObjGeneration::RetainCapturedMemory { oam, vram })
            })
            .unwrap_or_else(|| self.active_display_obj_generation.clone());
        let interrupted_item_receipt_obj_cache =
            std::mem::take(&mut self.next_display_interrupted_item_receipt_obj_cache);
        let entry_graphics_dma_plan = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_plan)
            .unwrap_or_else(|| {
                rom_graphics_dma_plan(captured_frame.main_module, captured_frame.submodule)
            });
        let entry_frame = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_frame)
            .unwrap_or(captured_frame);
        let module_oam_scanout_source = oam_scanout_across_main(
            entry_frame,
            captured_frame,
            entry_graphics_dma_plan.oam_scanout,
            self.screen_transition(),
        );
        let obj_scanout_generation = self.next_display_obj_scanout_generation.take();
        let link_obj_scanout_generation = obj_scanout_generation
            .map(|generation| generation.link_obj)
            .unwrap_or_else(|| {
                link_obj_scanout_across_main(
                    entry_frame,
                    captured_frame,
                    entry_graphics_dma_plan.link_obj_scanout,
                    self.screen_transition(),
                )
            });
        let link_obj_source_generation = obj_scanout_generation
            .map(|generation| generation.link_obj_sources)
            .unwrap_or_else(|| {
                link_obj_scanout_across_main(
                    entry_frame,
                    captured_frame,
                    entry_graphics_dma_plan.link_obj_scanout,
                    self.screen_transition(),
                )
            });
        let oam_dma_byte_len = self.ppu.oam.len() * 2;
        let dialogue_holds_published_oam = published_frame.is_some_and(|published| {
            dialogue_text_frame_holds_published_oam(
                published,
                captured_frame,
                captured_messaging.runtime.text_render_state(),
            )
        });
        let previously_published_shadow_oam_dma =
            self.display_snapshot.as_ref().and_then(|published| {
                dialogue_holds_published_oam
                    .then(|| published.published_shadow_oam_dma.clone())
                    .flatten()
                    .or_else(|| {
                        published
                            .ram
                            .get(OAM_BUF..OAM_BUF + oam_dma_byte_len)
                            .map(|bytes| {
                                bytes
                                    .chunks_exact(2)
                                    .map(|word| u16::from_le_bytes([word[0], word[1]]))
                                    .collect::<Vec<_>>()
                            })
                    })
            });
        let host_boundary_shadow_oam_dma = self
            .pre_main_graphics_dma
            .as_ref()
            .and_then(|graphics| graphics.oam_shadow.get(..oam_dma_byte_len))
            .map(|bytes| {
                bytes
                    .chunks_exact(2)
                    .map(|word| u16::from_le_bytes([word[0], word[1]]))
                    .collect::<Vec<_>>()
            });
        let (dialogue_oam_scanout_source, next_dialogue_oam_phase) =
            dialogue_oam_scanout_transition(
                self.dialogue_oam_publication_phase,
                module_oam_scanout_source,
                captured_frame.main_module == 14
                    && captured_frame.submodule == 2
                    && captured_messaging.runtime.text_render_state() == 4,
                previously_published_shadow_oam_dma.as_deref(),
                &self.ppu.oam,
            );
        let entry_text_render_state = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_dialogue_text_render_state)
            .unwrap_or_else(|| captured_messaging.runtime.text_render_state());
        let dialogue_cpu_boundary = dialogue_oam_cpu_boundary(
            captured_frame.main_module,
            captured_frame.submodule,
            entry_text_render_state,
            captured_messaging.runtime.text_render_state(),
        );
        let dialogue_oam_scanout_source =
            oam_scanout_for_cpu_boundary(dialogue_oam_scanout_source, dialogue_cpu_boundary);
        let dungeon_item_hold_entry_scanout = is_dungeon_item_hold_entry(
            entry_frame,
            captured_frame,
            self.pre_main_graphics_dma
                .as_ref()
                .map(|graphics| graphics.entry_link_handler_state)
                .unwrap_or_else(|| self.game_state.player.follower_link.handler_state()),
            self.game_state.player.follower_link.handler_state(),
        );
        let item_hold_entry_oam_scanout_source = oam_scanout_for_dungeon_item_hold_entry(
            dialogue_oam_scanout_source,
            dungeon_item_hold_entry_scanout,
        );
        self.dialogue_oam_publication_phase = next_dialogue_oam_phase;
        let oam_scanout_source = obj_scanout_generation
            .map(|generation| generation.oam)
            .unwrap_or(item_hold_entry_oam_scanout_source);
        let published_shadow_oam_dma =
            if module_oam_scanout_source == OamScanoutSource::ComposePublishedShadowDma {
                host_boundary_shadow_oam_dma
            } else {
                previously_published_shadow_oam_dma
            };
        let bg_scroll_generation = std::mem::take(&mut self.next_display_bg_scroll_generation);
        let suspended_spiral_animated_bg_dma_crosses_scanout = entry_frame.frame_counter
            == captured_frame.frame_counter
            && captured_frame.main_module == 7
            && captured_frame.submodule == 0x0e
            && self.game_state.display.bg_tile_animation_countdown == 1;
        let mut snapshot = Box::new(DisplaySnapshot {
            ram: self.ram.clone(),
            ppu: self.ppu.clone(),
            dma: self.dma.clone(),
            vram_chr_source: self.vram_chr_source.clone(),
            vram_chr_preview_source: self.vram_chr_preview_source.clone(),
            hdma_table_generation: self
                .attract_map_hdma_projection_before
                .take()
                .map(|before_projection| {
                    DisplayHdmaTableGeneration::AttractMapProjectionDuringScanout {
                        before_projection,
                    }
                })
                .unwrap_or_default(),
            vram_generation: std::mem::take(&mut self.next_display_vram_generation),
            hud_vram_generation: if std::mem::take(&mut self.publish_live_hud_vram_on_next_capture)
                || !self.game_state.system_signals.should_update_hud()
            {
                DisplayVramGeneration::ComposeLiveAfterNmi
            } else {
                DisplayVramGeneration::RetainCapturedBeforeNmi
            },
            hud_vram_destination: self
                .game_state
                .display
                .message_dma_destination_address_usize(),
            link_obj_scanout_generation,
            link_obj_source_generation,
            oam_scanout_source,
            dungeon_item_hold_entry_scanout,
            dungeon_item_hold_entry_bg2_scroll: dungeon_item_hold_entry_scanout
                .then_some((self.ppu.bg_layer[1].h_scroll, self.ppu.bg_layer[1].v_scroll)),
            published_shadow_oam_dma,
            room_72_interrupted_main_prefix_oam_offset_active: self
                .room_72_interrupted_main_prefix_oam_offset_active,
            animated_bg_scanout_generation: self
                .next_display_animated_bg_scanout_generation
                .take()
                .unwrap_or_else(|| {
                    if suspended_spiral_animated_bg_dma_crosses_scanout {
                        // The projected upload completes at this vblank, after
                        // the already-captured scanout generation. Keep the
                        // resident animated tiles for this image; the live PPU
                        // carries the completed DMA into the following image.
                        return AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi;
                    }
                    let generation = animated_bg_scanout_across_main(
                        entry_graphics_dma_plan,
                        rom_graphics_dma_plan_at_host_boundary(captured_frame),
                    );
                    if publish_live_animated_bg_on_submodule6_entry
                        || rom_dungeon_item_hold_to_dialogue_publishes_live_animated_bg(
                            entry_frame,
                            captured_frame,
                        )
                        || rom_dungeon_subtile_return_publishes_live_animated_bg(
                            entry_frame,
                            captured_frame,
                        )
                        || rom_dungeon_supertile_filter_entry_publishes_live_animated_bg(
                            entry_frame,
                            captured_frame,
                        )
                    {
                        AnimatedBgScanoutGeneration::LiveAfterNmi
                    } else {
                        generation
                    }
                }),
            bg_scroll_generation,
            spotlight_scanout_generation: self
                .next_display_spotlight_scanout
                .take()
                .map(SpotlightScanoutGeneration::ComposeLiveAfterNmi)
                .unwrap_or(SpotlightScanoutGeneration::CapturedBeforeNmi),
            obj_generation,
            interrupted_item_receipt_obj_cache,
            published_bg3_vwf_glyph_runs: self.published_bg3_vwf_glyph_runs.clone(),
            published_bg3_vwf_glyph_run_dialogue_offsets: self
                .published_bg3_vwf_glyph_run_dialogue_offsets
                .clone(),
            published_dialogue_msg_read_pos: self.published_dialogue_msg_read_pos,
            published_dialogue_message_id: self.published_dialogue_message_id,
            intro_poly_upload_delay: self.intro_poly_upload_delay,
            intro_sprite_animation_start_delay: self.intro_sprite_animation_start_delay,
            rom_reset_frame_delay: self.rom_reset_frame_delay,
            intro_memory_darken_frame_delay: self.intro_memory_darken_frame_delay,
            nmi_poly_upload_deferred: self.nmi_poly_upload_deferred,
            obj_vram_latch_generation: self.obj_vram_latch_generation,
            snes9x_poly_scheduler_counter: self.snes9x_poly_scheduler_counter,
        });
        let nmi_forced_blank_scanlines =
            std::mem::take(&mut self.nmi_forced_blank_scanlines_pending);
        snapshot.ppu.forced_blank_scanlines = nmi_forced_blank_scanlines;
        snapshot.ppu.forced_blank_from_scanline = self
            .nmi_forced_blank_from_scanline_pending
            .take()
            .filter(|_| snapshot.ppu.forced_blank);
        snapshot.ppu.retain_active_display_history =
            snapshot.ppu.forced_blank_from_scanline.is_some();
        if frame.main_module == 0
            && frame.submodule == 1
            && self.intro_initialization_reset_obj_control_pending
        {
            // OBSEL is still at its reset value for the interrupted initial
            // display slice. Keep this in the immutable control snapshot; the
            // live native PPU remains configured for the normal Zelda OBJ CHR
            // base used once initialization resumes.
            snapshot.ppu.obj_tile_adr1 = 0;
            snapshot.ppu.obj_tile_adr2 = 0;
        }
        // A high-bit V-counter request is a one-frame raster event. Publish it
        // in this immutable display snapshot, then advance live simulation to
        // the consumed state. Rendering must not decide whether game state
        // advances, and replaying the same snapshot must reproduce the same
        // scanlines.
        if self.game_state.display.irq_control_has_vcounter_marker() {
            self.clear_irq_control_flag();
        }
        if let Some(prefix) = self.spotlight_hdma_reset_prefix.take() {
            // The reset is reached after HDMA has already consumed the first
            // few scanlines of the landing-wipe table. Preserve that consumed
            // prefix in the published snapshot while live state remains fully
            // reset for the next frame.
            for (index, value) in prefix.into_iter().enumerate() {
                write_le_u16(&mut snapshot.ram, HDMA_TABLE_DYNAMIC + index * 2, value);
            }
        }
        if rom_intro_poly_thread_is_active(frame.main_module, frame.submodule) {
            self.intro_poly_vram_history.push((
                frame.frame_counter,
                self.ppu.vram[0x5800..0x5c00].to_vec(),
                self.ppu.oam.to_vec(),
            ));
            if self.intro_poly_vram_history.len() > 16 {
                self.intro_poly_vram_history.remove(0);
            }
        } else {
            self.intro_poly_vram_history.clear();
        }
        let retires_room_72_interrupted_main_prefix_oam = publication
            != DisplaySnapshotPublication::RetainPublished
            && snapshot.room_72_interrupted_main_prefix_oam_offset_active
            && snapshot.ppu.oam[12 * 2].to_le_bytes()[1] == 0xf0;
        self.visible_display_snapshot = None;
        match publication {
            DisplaySnapshotPublication::AdvanceStaged => {
                let previous = self.deferred_display_snapshot.replace(snapshot);
                self.display_snapshot = previous.or_else(|| self.deferred_display_snapshot.clone());
            }
            DisplaySnapshotPublication::PublishCaptured => {
                self.deferred_display_snapshot = None;
                self.display_snapshot = Some(snapshot);
            }
            DisplaySnapshotPublication::RetainPublished => {
                if self.display_snapshot.is_none() {
                    self.display_snapshot =
                        self.deferred_display_snapshot.clone().or(Some(snapshot));
                } else if let Some(published) = self.display_snapshot.as_mut() {
                    // The entry snapshot owns one scanout with the pre-pickup
                    // camera. Item graphics then retain that same OAM/VRAM
                    // generation across subsequent vblanks, but BG2 scroll is
                    // independently republished from the live camera starting
                    // with the first retained boundary.
                    published.dungeon_item_hold_entry_scanout = false;
                }
            }
        }
        if retires_room_72_interrupted_main_prefix_oam {
            self.room_72_interrupted_main_prefix_oam_offset_active = false;
        }
    }

    fn compose_display_registers(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
    ) {
        // The ROM's iris setup authors window controls, HDMA channel state,
        // enable state, and both indirect tables as one scanout generation.
        // Never combine live controls with the captured (still-open) circle.
        following.spotlight_scanout_generation.compose_into(
            &mut self.ram,
            &mut self.ppu,
            &mut self.dma,
        );
        // A table projection can complete after that coupled iris generation
        // is staged but before HDMA consumes the next scanout. Apply this
        // scanout-local table generation last so it refines, rather than gets
        // overwritten by, the coherent controls/channels/tables baseline.
        following.hdma_table_generation.compose_into(&mut self.ram);
        plan.bg_scroll_source
            .compose_into(&mut self.ppu, &following.ppu);
        if plan.publish_live_overworld_transition_half_color {
            self.ppu.half_color = following.ppu.half_color;
        }
    }

    fn atomic_item_graphics_holds_following_nmi(&self) -> bool {
        matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishItemReceiptGraphics {
                continuation: ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { .. },
            })
        )
    }

    fn item_receipt_graphics_return_uses_ordinary_module_epilogue(
        &self,
        continuation: ItemReceiptGraphicsContinuation,
    ) -> bool {
        if !matches!(
            continuation,
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x21 }
        ) {
            return false;
        }
        self.game_state.player.follower_link.handler_state() != 21
    }

    fn stage_atomic_item_graphics_return_obj_scanout(
        &mut self,
        continuation: ItemReceiptGraphicsContinuation,
    ) {
        let scanout = atomic_item_graphics_return_obj_scanout(continuation);
        let retained_display_memory = !matches!(
            continuation,
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x22 }
        );
        let live_animated_bg = !retained_display_memory
            || matches!(
                continuation,
                ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x24 }
            );
        if (matches!(
            continuation,
            ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. }
        ) && scanout.link_obj == GraphicsDmaGeneration::HostBoundaryBeforeMain)
            || matches!(
                continuation,
                ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x24 }
            )
        {
            self.next_display_interrupted_item_receipt_obj_cache = true;
        }
        // RetainPublished deliberately keeps the already-visible snapshot
        // object. Annotate that retained generation with split Link pixel and
        // provenance ownership now, then carry the same split into the next
        // captured boundary as well.
        for snapshot in [
            self.display_snapshot.as_mut(),
            self.visible_display_snapshot.as_mut(),
        ]
        .into_iter()
        .flatten()
        {
            snapshot.oam_scanout_source = match continuation {
                ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { .. } => {
                    OamScanoutSource::ComposePublishedShadowDma
                }
                ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. } => scanout.oam,
            };
            snapshot.vram_generation = if retained_display_memory {
                DisplayVramGeneration::RetainCapturedBeforeNmi
            } else {
                DisplayVramGeneration::ComposeLiveAfterNmi
            };
            snapshot.hud_vram_generation = if retained_display_memory {
                DisplayVramGeneration::RetainCapturedBeforeNmi
            } else {
                DisplayVramGeneration::ComposeLiveAfterNmi
            };
            snapshot.animated_bg_scanout_generation = if live_animated_bg {
                AnimatedBgScanoutGeneration::LiveAfterNmi
            } else {
                AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
            };
            snapshot.link_obj_scanout_generation = scanout.link_obj;
            snapshot.link_obj_source_generation = scanout.link_obj_sources;
        }
        if matches!(
            continuation,
            ItemReceiptGraphicsContinuation::CallerAlreadyCompleted { gfx: 0x24 }
        ) {
            // This receipt finishes after the current retained scanout has
            // already selected its OAM, but Main_PrepSpritesForNmi has prepared
            // the complete shadow consumed by the following vblank. Capture
            // that exact next OAM generation explicitly so an older retained
            // OBJ-memory generation cannot keep the packed size bits stale.
            let shadow = self.sprite_oam_shadow_buffer();
            let oam = shadow
                .chunks_exact(2)
                .take(self.ppu.oam.len())
                .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
                .collect();
            self.next_display_obj_memory_generation =
                Some(DisplayObjGeneration::RetainCapturedOam { oam });
        }
        self.publish_live_hud_vram_on_next_capture = true;
        self.next_display_vram_generation = if retained_display_memory {
            DisplayVramGeneration::RetainCapturedBeforeNmi
        } else {
            DisplayVramGeneration::ComposeLiveAfterNmi
        };
        self.next_display_animated_bg_scanout_generation = Some(if live_animated_bg {
            AnimatedBgScanoutGeneration::LiveAfterNmi
        } else {
            AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi
        });
        self.next_display_obj_scanout_generation = Some(scanout);
    }

    fn stage_live_animated_bg_scanout(&mut self) {
        if std::env::var_os("ZELDA3_TRACE_DISPLAY_VRAM").is_some() {
            eprintln!(
                "TRACE_STAGE_LIVE_ANIMATED display={} visible={} deferred={}",
                self.display_snapshot.is_some(),
                self.visible_display_snapshot.is_some(),
                self.deferred_display_snapshot.is_some(),
            );
        }
        for snapshot in [
            self.display_snapshot.as_mut(),
            self.visible_display_snapshot.as_mut(),
        ]
        .into_iter()
        .flatten()
        {
            snapshot.animated_bg_scanout_generation = AnimatedBgScanoutGeneration::LiveAfterNmi;
        }
        self.next_display_animated_bg_scanout_generation =
            Some(AnimatedBgScanoutGeneration::LiveAfterNmi);
    }

    fn compose_display_vram(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
        retained_full_tilemap_vram: Option<&RetainedVramRegion>,
    ) {
        // The polygon worker publishes through its NMI handshake at the start
        // of the frame. Preserve that completed pre-NMI buffer rather than a
        // job that may have finished later in the current CPU slice.
        let presented_poly = self.selected_intro_poly_display_buffer();
        let entry_frame = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_frame)
            .unwrap_or_else(|| crate::game_state::FrameState::load_from_ram(&self.ram));
        let following_frame = crate::game_state::FrameState::load_from_ram(&following.ram);
        let mixed_supertile_entry_obj =
            dungeon_supertile_entry_uses_mixed_obj_scanout(entry_frame, following_frame);
        let host_boundary_link_obj_vram = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.link_obj_vram.clone())
            .unwrap_or_else(|| self.ppu.vram[0x4000..0x4400].to_vec());
        let entry_link_obj_vram = (matches!(
            plan.link_obj_scanout_generation,
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        ) && matches!(
            plan.link_obj_source_generation,
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        ) && !mixed_supertile_entry_obj)
            .then(|| host_boundary_link_obj_vram.clone());
        let retained_supertile_entry_link_obj_vram = mixed_supertile_entry_obj.then(|| {
            [
                host_boundary_link_obj_vram[0x000..0x050].to_vec(),
                host_boundary_link_obj_vram[0x100..0x150].to_vec(),
            ]
        });
        let following_room = crate::game_state::WorldLocationState::load_from_ram(&following.ram)
            .dungeon_room_index();
        let retained_doorway_link_obj_vram = room_71_supertile_return_retains_link_vram(
            entry_frame,
            following_frame,
            following_room,
        )
        .then(|| {
            [
                self.ppu.vram[0x4000..0x4050].to_vec(),
                self.ppu.vram[0x4100..0x4150].to_vec(),
            ]
        });
        let retained_hud_vram = matches!(
            following.hud_vram_generation,
            DisplayVramGeneration::RetainCapturedBeforeNmi
        )
        .then(|| {
            RetainedVramRegion::capture(
                &self.ppu.vram,
                following.hud_vram_destination,
                HUD_TILEMAP_NMI_WORDS,
            )
        })
        .flatten();
        let retained_nmi_copy_packet_vram = (self.ram[NMI_COPY_PACKETS_FLAG] != 0).then(|| {
            NmiCopyPacketScanout::capture(
                &self.ppu.vram,
                &self.ram[crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF..],
            )
        });
        // Animated-BG DMA configures the next active frame. Retain the
        // host-boundary VRAM generation at whichever destination the current
        // tileset selected ($3b00 indoors, $3c00 outdoors). The resumed
        // bad-weather tail is the measured exception that publishes the live
        // post-NMI generation.
        let animated_bg_destination = read_le_u16(&self.ram, ANIMATED_TILE_VRAM_ADDR) as usize;
        if std::env::var_os("ZELDA3_TRACE_DISPLAY_VRAM").is_some()
            && following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 8
        {
            eprintln!(
                "TRACE_DISPLAY_VRAM entry={:02x}/{:02x}/{:02x} following={:02x}/{:02x}/{:02x} vram={:?} animated={:?} destination=0x{animated_bg_destination:04x} captured_3c20=0x{:04x} following_3c20=0x{:04x}",
                entry_frame.main_module,
                entry_frame.submodule,
                entry_frame.subsubmodule,
                following_frame.main_module,
                following_frame.submodule,
                following_frame.subsubmodule,
                plan.vram_generation,
                plan.animated_bg_scanout_generation,
                self.ppu.vram[0x3c20],
                following.ppu.vram[0x3c20],
            );
        }
        let previous_animated_bg_vram = (plan.animated_bg_scanout_generation
            == AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi)
            .then(|| {
                self.pre_nmi_animated_bg_scanout
                    .as_ref()
                    .filter(|scanout| {
                        scanout.destination_address == animated_bg_destination
                            && scanout.destination_address + scanout.vram.len()
                                <= self.ppu.vram.len()
                    })
                    .map(|scanout| (scanout.destination_address, scanout.vram.clone()))
            })
            .flatten();

        if plan.vram_generation == DisplayVramGeneration::ComposeLiveAfterNmi {
            self.ppu.vram.clone_from(&following.ppu.vram);
            if let Some((destination, animated_bg_vram)) = previous_animated_bg_vram {
                self.ppu.vram[destination..destination + animated_bg_vram.len()]
                    .copy_from_slice(&animated_bg_vram);
            }
            if let Some(entry_link_obj_vram) = entry_link_obj_vram {
                self.ppu.vram[0x4000..0x4400].copy_from_slice(&entry_link_obj_vram);
            }
            if let Some([upper, lower]) = retained_supertile_entry_link_obj_vram {
                self.ppu.vram[0x4000..0x4050].copy_from_slice(&upper);
                self.ppu.vram[0x4100..0x4150].copy_from_slice(&lower);
            }
            if let Some([upper, lower]) = retained_doorway_link_obj_vram {
                self.ppu.vram[0x4000..0x4050].copy_from_slice(&upper);
                self.ppu.vram[0x4100..0x4150].copy_from_slice(&lower);
            }
            if let Some(retained_hud_vram) = retained_hud_vram.as_ref() {
                retained_hud_vram.publish_to(&mut self.ppu.vram);
            }
            if let Some(scanout) = retained_nmi_copy_packet_vram.as_ref() {
                scanout.publish_to(&mut self.ppu.vram);
            }
            self.ppu.vram[0x5800..0x5c00].copy_from_slice(&presented_poly);
        } else if matches!(
            plan.link_obj_scanout_generation,
            GraphicsDmaGeneration::LiveAfterMain
        ) {
            if matches!(
                plan.oam_scanout_source,
                OamScanoutSource::ComposePublishedShadowDma
                    | OamScanoutSource::ComposeLivePlayerOamAfterMain
            ) {
                self.ppu.vram[0x4000..0x4400]
                    .copy_from_slice(&following.ppu.vram[0x4000..0x4400]);
            } else if !self.atomic_item_graphics_holds_following_nmi() {
                // A long NMI can retain the pre-upload BG/tilemap image while its
            // OBJ DMA still completes before the sprites are scanned. Keep
            // this domain independent: Snes9x's completed tile cache can own
            // the early Link body/head/hand batch while the rest of visible
            // VRAM, including later Link OBJ uploads, remains captured.
                for range in [0x4000..0x4050, 0x4100..0x4150] {
                    self.ppu.vram[range.clone()].copy_from_slice(&following.ppu.vram[range]);
                }
            } else {
                // Resolve that DMA from the scanout snapshot's own source words.
                // The following coarse host slice can already have latched a long
                // item-graphics call and skipped NMI_DoUpdates, leaving its PPU
                // bytes one upload behind even though the snapshot owns the next
                // hardware generation (oracle frame 4582).
                let live_sources = LinkDmaSources::load_from_ram(&following.ram);
                let link_graphics = self.asset_raw(57).map(Vec::from);
                for (destination, source, len) in EARLY_LINK_OBJ_DMA_TRANSFERS {
                    let source_address = usize::from(live_sources.source(source));
                    let source_offset = source_address.saturating_sub(0x8000);
                    let destination_end = destination + len / 2;
                    if let Some(link_graphics) = link_graphics.as_deref().filter(|graphics| {
                        source_address >= 0x8000 && source_offset + len <= graphics.len()
                    }) {
                        self.copy_asset_bytes_to_vram(
                            destination,
                            link_graphics,
                            source_address,
                            len,
                        );
                    } else {
                        self.ppu.vram[destination..destination_end]
                            .copy_from_slice(&following.ppu.vram[destination..destination_end]);
                    }
                }
            }
        }
        if plan.vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi
            && plan.animated_bg_scanout_generation == AnimatedBgScanoutGeneration::LiveAfterNmi
        {
            // A pending stripe can retain the general VRAM generation while
            // the leading-NMI animated-tile DMA independently completes for
            // this scanout. Compose that $400-byte domain explicitly; merely
            // selecting LiveAfterNmi is otherwise a no-op in the retained
            // whole-VRAM branch above.
            let destination = read_le_u16(&following.ram, ANIMATED_TILE_VRAM_ADDR) as usize;
            const ANIMATED_BG_NMI_WORDS: usize = 0x200;
            if let (Some(live), Some(presented)) = (
                following
                    .ppu
                    .vram
                    .get(destination..destination + ANIMATED_BG_NMI_WORDS),
                self.ppu
                    .vram
                    .get_mut(destination..destination + ANIMATED_BG_NMI_WORDS),
            ) {
                presented.copy_from_slice(live);
            }
        }
        if plan.vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi
            && following.hud_vram_generation == DisplayVramGeneration::ComposeLiveAfterNmi
        {
            // Item-receipt return retains the room/tile animation generation,
            // but the HUD packet has completed and is independently visible.
            // Publish just that packet's destination instead of promoting the
            // entire post-NMI VRAM image.
            let destination = following.hud_vram_destination;
            if let (Some(live), Some(presented)) = (
                following
                    .ppu
                    .vram
                    .get(destination..destination + HUD_TILEMAP_NMI_WORDS),
                self.ppu
                    .vram
                    .get_mut(destination..destination + HUD_TILEMAP_NMI_WORDS),
            ) {
                presented.copy_from_slice(live);
            }
        }
        let completed_item_return_publishes_live_tilemap =
            (plan.oam_scanout_source == OamScanoutSource::ComposeLivePlayerOamAfterMain
                && plan.link_obj_scanout_generation == GraphicsDmaGeneration::LiveAfterMain
                && plan.link_obj_source_generation == GraphicsDmaGeneration::LiveAfterMain
                && following.hud_vram_generation == DisplayVramGeneration::ComposeLiveAfterNmi)
                || (following.interrupted_item_receipt_obj_cache
                    && plan.oam_scanout_source == OamScanoutSource::ComposePublishedShadowDma);
        if plan.vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi
            && completed_item_return_publishes_live_tilemap
        {
            // The item-receipt return crosses the tilemap DMA before the
            // caller's held-item OAM is scanned. BG1/BG2 map words therefore
            // belong to the live post-NMI image even though their CHR and the
            // rest of general VRAM retain the host-boundary generation.
            self.ppu.vram[0x0000..0x2000]
                .copy_from_slice(&following.ppu.vram[0x0000..0x2000]);
        }
        if let Some(retained_full_tilemap_vram) = retained_full_tilemap_vram {
            retained_full_tilemap_vram.publish_to(&mut self.ppu.vram);
        }
    }

    fn compose_display_chr_sources(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
    ) {
        let entry_frame = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.entry_frame)
            .unwrap_or_else(|| crate::game_state::FrameState::load_from_ram(&self.ram));
        let following_frame = crate::game_state::FrameState::load_from_ram(&following.ram);
        let mixed_supertile_entry_obj =
            dungeon_supertile_entry_uses_mixed_obj_scanout(entry_frame, following_frame);
        let dungeon_supertile_state3_retains_presented_link_sources = following_frame.main_module
            == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 3;
        let retained_link_obj_sources = ((matches!(
            plan.link_obj_source_generation,
            GraphicsDmaGeneration::HostBoundaryBeforeMain
        )
            || dungeon_supertile_state3_retains_presented_link_sources)
            && !mixed_supertile_entry_obj)
            .then(|| {
                (
                    self.vram_chr_source.clone(),
                    self.vram_chr_preview_source.clone(),
                )
            });
        let retained_supertile_entry_link_obj_sources = mixed_supertile_entry_obj.then(|| {
            (
                self.vram_chr_source.clone(),
                self.vram_chr_preview_source.clone(),
            )
        });

        if plan.vram_generation == DisplayVramGeneration::ComposeLiveAfterNmi {
            self.vram_chr_source.clone_from(&following.vram_chr_source);
            self.vram_chr_preview_source
                .clone_from(&following.vram_chr_preview_source);
            if let Some((logical, preview)) = retained_link_obj_sources.as_ref() {
                self.vram_chr_source
                    .copy_word_range_from(logical, 0x4000..0x4400);
                self.vram_chr_preview_source
                    .copy_word_range_from(preview, 0x4000..0x4400);
            }
            if let Some((logical, preview)) = retained_supertile_entry_link_obj_sources.as_ref() {
                for range in [0x4000..0x4050, 0x4100..0x4150] {
                    self.vram_chr_source
                        .copy_word_range_from(logical, range.clone());
                    self.vram_chr_preview_source
                        .copy_word_range_from(preview, range);
                }
            }
        } else if matches!(
            plan.link_obj_source_generation,
            GraphicsDmaGeneration::LiveAfterMain
        ) {
            if matches!(
                plan.oam_scanout_source,
                OamScanoutSource::ComposePublishedShadowDma
                    | OamScanoutSource::ComposeLivePlayerOamAfterMain
            ) {
                self.vram_chr_source
                    .copy_word_range_from(&following.vram_chr_source, 0x4000..0x4400);
                self.vram_chr_preview_source.copy_word_range_from(
                    &following.vram_chr_preview_source,
                    0x4000..0x4400,
                );
            } else if !self.atomic_item_graphics_holds_following_nmi()
                && !dungeon_supertile_state3_retains_presented_link_sources
            {
                for range in [0x4000..0x4050, 0x4100..0x4150] {
                    self.vram_chr_source
                        .copy_word_range_from(&following.vram_chr_source, range.clone());
                    self.vram_chr_preview_source
                        .copy_word_range_from(&following.vram_chr_preview_source, range);
                }
            } else if !dungeon_supertile_state3_retains_presented_link_sources {
                let captured_sources = LinkDmaSources::load_from_ram(&self.ram);
                let link_graphics_len = self.asset_raw(57).map(<[u8]>::len);
                let link_pack = read_le_u16(
                    &self.ram,
                    crate::game_state::constants::LINK_DMA_GRAPHICS_INDEX,
                ) >> 1;
                for (destination, source, len) in EARLY_LINK_OBJ_DMA_TRANSFERS {
                    let source_address = usize::from(captured_sources.source(source));
                    let source_offset = source_address.saturating_sub(0x8000);
                    if source_address >= 0x8000
                        && link_graphics_len
                            .is_some_and(|asset_len| source_offset + len <= asset_len)
                    {
                        let tile_count = (len / 2).div_ceil(16);
                        let base_offset = (source_offset >> 5) as u16;
                        self.vram_chr_source.record_tiles_from(
                            destination,
                            tile_count,
                            crate::chr_source::CHR_KIND_LINK,
                            link_pack,
                            base_offset,
                        );
                        self.vram_chr_preview_source.record_tiles_from(
                            destination,
                            tile_count,
                            crate::chr_source::CHR_KIND_LINK,
                            link_pack,
                            base_offset,
                        );
                    } else {
                        let range = destination..destination + len / 2;
                        self.vram_chr_source
                            .copy_word_range_from(&following.vram_chr_source, range.clone());
                        self.vram_chr_preview_source
                            .copy_word_range_from(&following.vram_chr_preview_source, range);
                    }
                }
            }
        }
        if dungeon_supertile_state3_retains_presented_link_sources {
            if let Some(logical) = self.last_presented_vram_chr_source.as_ref() {
                self.vram_chr_source
                    .copy_word_range_from(logical, 0x4000..0x4400);
            }
            if let Some(preview) = self.last_presented_vram_chr_preview_source.as_ref() {
                self.vram_chr_preview_source
                    .copy_word_range_from(preview, 0x4000..0x4400);
            }
        }
        if plan.vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi
            && plan.animated_bg_scanout_generation == AnimatedBgScanoutGeneration::LiveAfterNmi
        {
            let destination = read_le_u16(&following.ram, ANIMATED_TILE_VRAM_ADDR) as usize;
            let range = destination..destination + 0x200;
            self.vram_chr_source
                .copy_word_range_from(&following.vram_chr_source, range.clone());
            self.vram_chr_preview_source
                .copy_word_range_from(&following.vram_chr_preview_source, range);
        }
        if plan.vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi
            && following.hud_vram_generation == DisplayVramGeneration::ComposeLiveAfterNmi
        {
            // HUD tile bytes can already match the resident PPU image while
            // their modern CHR ownership was authored by the just-completed
            // item-receipt return. Keep the raw retained scanout, but publish
            // the matching BG3 source identities with it; otherwise the modern
            // renderer treats an exact HUD icon as transparent.
            let bg3_chr_start = usize::from(following.ppu.bg_layer[2].tile_adr);
            let bg3_chr_end = bg3_chr_start.saturating_add(0x1000).min(0x8000);
            if bg3_chr_start < bg3_chr_end {
                let range = bg3_chr_start..bg3_chr_end;
                self.vram_chr_source
                    .copy_word_range_from(&following.vram_chr_source, range.clone());
                self.vram_chr_preview_source
                    .copy_word_range_from(&following.vram_chr_preview_source, range);
            }
        }
    }

    fn compose_display_cgram(
        &mut self,
        following: &DisplaySnapshot,
        plan: &DisplayPublicationPlan,
    ) {
        if !plan.compose_live_cgram {
            return;
        }

        // The NMI's main-palette-buffer upload is only visible on the next
        // scanout. Direct CGRAM writes outside that upload stay same-frame
        // visible through the following image.
        if plan.world_map_fade_display {
            // The world-map fade publishes Mode 7 memory and INIDISP now, but
            // CGRAM remains on the generation consumed by the preceding NMI.
        } else if let Some(latch) = self.cgram_upload_latch.as_ref() {
            self.ppu.cgram.copy_from_slice(latch);
        } else {
            self.ppu.cgram.clone_from(&following.ppu.cgram);
        }
    }

    fn compose_display_oam(&mut self, following: &DisplaySnapshot, plan: &DisplayPublicationPlan) {
        let following_frame = crate::game_state::FrameState::load_from_ram(&following.ram);
        let resident_dungeon_landing_oam = (following_frame.main_module == 7
            && following_frame.submodule == 0x0f
            && following_frame.subsubmodule == 1)
            .then(|| self.ppu.oam.clone())
            .filter(|resident| resident[116 * 2].to_le_bytes()[1] != 0xf0);
        let resident_link_body_xy = self.ppu.oam[102 * 2];
        let resident_first_sorted_entry_y = self.ppu.oam[12 * 2].to_le_bytes()[1];
        let debug_subtile_oam = env::var_os("ZELDA3_DEBUG_DISPLAY_OAM").is_some()
            && following_frame.main_module == 7
            && following_frame.submodule == 1
            && following_frame.subsubmodule == 6;
        let debug_entries = [12, 13, 92, 93, 102, 103, 107, 110, 111, 112, 113];
        let captured_entries = debug_subtile_oam
            .then(|| debug_entries.map(|entry| oam_entry_bytes(&self.ppu.oam, entry)));
        if !plan.retain_captured_oam {
            self.ppu.oam.clone_from(&following.ppu.oam);
        }
        let published_shadow_oam = match plan.oam_scanout_source {
            OamScanoutSource::RetainCapturedBeforeNmi
            | OamScanoutSource::ComposePublishedShadowDma => {
                following.published_shadow_oam_dma.as_deref()
            }
            OamScanoutSource::RetainResidentPpuOam
            | OamScanoutSource::ComposeLiveAfterNmi
            | OamScanoutSource::ComposeLivePlayerOamAfterMain => None,
        };
        if matches!(
            plan.oam_scanout_source,
            OamScanoutSource::RetainCapturedBeforeNmi | OamScanoutSource::ComposePublishedShadowDma
        ) {
            // The snapshot's PPU OAM includes main-thread shadow writes that
            // hardware has not necessarily DMAed yet. When scanout retains the
            // pre-NMI generation, use the shadow image published by the prior
            // completed OAM DMA rather than those newly authored coordinates.
            if let Some(published_shadow_oam) =
                published_shadow_oam.filter(|oam| oam.len() == self.ppu.oam.len())
            {
                self.ppu.oam.clone_from_slice(published_shadow_oam);
            }
        }
        if plan.oam_scanout_source == OamScanoutSource::ComposeLivePlayerOamAfterMain {
            // LinkOam_Main owns entries 12..17 on this unsorted dungeon slice.
            // The remaining sprite table was already consumed from the entry
            // shadow and must not be advanced with Link's hold-item handoff.
            // Link's live coordinates were authored against the live camera,
            // while this scanout still owns the independently captured scroll
            // registers. Rebase the six player entries onto that display
            // generation instead of mixing camera generations.
            const PLAYER_OAM_WORDS: std::ops::Range<usize> = 24..36;
            let byte_start = OAM_BUF + PLAYER_OAM_WORDS.start * 2;
            let byte_end = OAM_BUF + PLAYER_OAM_WORDS.end * 2;
            if let Some(live_shadow) = following.ram.get(byte_start..byte_end) {
                for (word, bytes) in self.ppu.oam[PLAYER_OAM_WORDS]
                    .iter_mut()
                    .zip(live_shadow.chunks_exact(2))
                {
                    *word = u16::from_le_bytes([bytes[0], bytes[1]]);
                }
            }
            // The mixed OAM generation remains resident while the camera can
            // independently advance on later retained scanouts. Rebase against
            // the camera that authored this OAM DMA, not the scroll registers
            // that happen to be live for the current scanout.
            let (oam_camera_x, oam_camera_y) = following
                .dungeon_item_hold_entry_bg2_scroll
                .unwrap_or((self.ppu.bg_layer[1].h_scroll, self.ppu.bg_layer[1].v_scroll));
            let x_delta =
                read_le_u16(&following.ram, BG2_X_SCROLL).wrapping_sub(oam_camera_x) as u8;
            let y_delta =
                read_le_u16(&following.ram, BG2_Y_SCROLL).wrapping_sub(oam_camera_y) as u8;
            for entry in 12..18 {
                let [x, y] = self.ppu.oam[entry * 2].to_le_bytes();
                self.ppu.oam[entry * 2] =
                    u16::from_le_bytes([x.wrapping_add(x_delta), y.wrapping_add(y_delta)]);
            }
        }
        if let Some(oam) = following.obj_generation.retained_oam() {
            self.ppu.oam.clone_from_slice(oam);
        }
        if let Some(resident_oam) = resident_dungeon_landing_oam.as_deref() {
            // The landing main slice authors the following OAM shadow after
            // hardware has already published these entries. Keep the resident
            // Link-body size bit and four landing pieces from the completed
            // DMA generation, matching the v1.0.0 scanout boundary.
            compose_published_oam_entries(
                &mut self.ppu.oam,
                Some(resident_oam),
                [102, 116, 117, 118, 119],
            );
        }
        if let Some(vram) = following.obj_generation.retained_vram() {
            self.ppu.vram[0x4000..0x4400].copy_from_slice(vram);
        }
        let following_room = crate::game_state::WorldLocationState::load_from_ram(&following.ram)
            .dungeon_room_index();
        if following_room == 0x72
            && following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 7
            && plan.oam_scanout_source == OamScanoutSource::RetainResidentPpuOam
            && following.ppu.oam[92 * 2].to_le_bytes()[1] != 0xf0
        {
            // The state-5 quadrant return keeps the populated resident table
            // across two scanouts. The room sprites were evaluated two lines
            // later than the generation selected by the second return.
            adjust_room_72_published_sprite_oam(&mut self.ppu.oam, 2);
        }
        if following_room == 0x72
            && following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 8
            && plan.oam_scanout_source == OamScanoutSource::ComposeLiveAfterNmi
            && plan.retain_captured_oam
            && following.ppu.oam[92 * 2].to_le_bytes()[1] != 0xf0
        {
            // The first state-8 capture releases the two-return hold after it
            // has selected the resident room-sprite table. Rebase those two
            // entries onto the live state-8 generation selected for scanout.
            adjust_room_72_published_sprite_oam(&mut self.ppu.oam, 2);
        }
        if following_room == 0x72
            && following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 8
            && plan.oam_scanout_source == OamScanoutSource::ComposePublishedShadowDma
            && !plan.retain_captured_oam
        {
            // The southward scroll's resumed sprite tail is three scanlines
            // ahead of the completed shadow DMA used for ordinary state-8
            // publication. Correct only the two active room-sprite entries;
            // Link and the remaining table already match that DMA generation.
            let bg2_y = read_le_u16(&following.ram, BG2_Y_SCROLL);
            let pixels = if bg2_y > 0x0fe0 {
                3
            } else if bg2_y == 0x0fe0 {
                2
            } else {
                0
            };
            adjust_room_72_published_sprite_oam(&mut self.ppu.oam, pixels);
        }
        if following_room == 0x72
            && following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 13
        {
            // State 12 completes Link's southward landing before the OAM DMA
            // visible at the state-13 boundary. The translated host frame has
            // already rebased those five entries to the following camera; move
            // only Link back onto the scanout's three-line-earlier generation.
            let first_landing_scanout = read_le_u16(&self.ram, LINK_DMA_COUNTDOWN) == 1;
            let pixels = if first_landing_scanout { 3 } else { 2 };
            adjust_room_72_state13_link_oam(&mut self.ppu.oam, pixels);
            if !first_landing_scanout {
                adjust_room_72_state13_body_oam(
                    &mut self.ppu.oam,
                    &following.ppu.oam,
                    read_le_u16(&following.ram, LINK_Y_COORD),
                );
            }
        }
        if following_room == 0x72
            && following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 14
        {
            // The landing frame publishes the body pieces before the final
            // head and hand tail crosses OAM DMA at the filtering boundary.
            adjust_room_72_state14_link_oam(&mut self.ppu.oam);
        }
        if following_room == 0x72
            && following_frame.main_module == 7
            && following_frame.submodule == 0
            && following_frame.subsubmodule == 0
            && following.room_72_interrupted_main_prefix_oam_offset_active
            && resident_first_sorted_entry_y != 0xf0
            && self.ppu.oam[102 * 2] != resident_link_body_xy
        {
            // The early state-11 return already crossed a main-loop prefix,
            // while these two room-sprite entries are still evaluated against
            // the pre-return camera generation. Keep that four-line offset
            // until that interrupted sorted table retires from resident OAM.
            rebase_room_72_sprite_oam_after_interrupted_main_prefix(&mut self.ppu.oam);
        }
        let room_71_return_uses_published_link_oam =
            room_71_supertile_return_uses_published_link_oam(following_frame, following_room);
        if room_71_return_uses_published_link_oam {
            // The room-$71 return crosses OAM DMA during doorway movement,
            // filtering, and the first room-load scanout. Snes9x therefore
            // presents Link's five entries from the completed shadow generation;
            // all other sprites remain from the live OAM image selected above.
            compose_published_link_oam(
                &mut self.ppu.oam,
                following.published_shadow_oam_dma.as_deref(),
            );
        }
        if let Some(captured_entries) = captured_entries {
            let published = following.published_shadow_oam_dma.as_deref();
            eprintln!(
                "display_subtile_oam host={} phase={:02x}/{:02x}/{:02x} source={:?} link={:?}/{:?} retain={} captured={:02x?} following={:02x?} published={:02x?} composed={:02x?}",
                self.frame_ctr_dbg,
                following_frame.main_module,
                following_frame.submodule,
                following_frame.subsubmodule,
                plan.oam_scanout_source,
                plan.link_obj_scanout_generation,
                plan.link_obj_source_generation,
                plan.retain_captured_oam,
                captured_entries,
                debug_entries.map(|entry| oam_entry_bytes(&following.ppu.oam, entry)),
                published.map(|oam| debug_entries.map(|entry| oam_entry_bytes(oam, entry))),
                debug_entries.map(|entry| oam_entry_bytes(&self.ppu.oam, entry)),
            );
        }
        self.ppu.obj_vram_latch = None;
        if following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 3
            && plan.link_obj_scanout_generation == GraphicsDmaGeneration::HostBoundaryBeforeMain
        {
            // The state-3 entry scanout evaluates Link before the next pose's
            // upload becomes resident. Resolve the host-boundary DMA operands
            // after the transition OBJ bundle has been composed.
            if matches!(following_room, 0x70 | 0x71) {
                let live_sources = LinkDmaSources::load_from_ram(&following.ram);
                let link_graphics = self.asset_raw(57).map(Vec::from);
                for (destination, source) in [
                    (0x4020, LinkDmaSourceSlot::HeadTop),
                    (0x4120, LinkDmaSourceSlot::HeadBottom),
                ] {
                    let len = 0x40;
                    let source_address =
                        usize::from(live_sources.source(source).wrapping_sub(0x40));
                    let source_offset = source_address.saturating_sub(0x8000);
                    let destination_end = destination + len / 2;
                    let Some(source_bytes) = link_graphics.as_deref().and_then(|graphics| {
                        (source_address >= 0x8000 && source_offset + len <= graphics.len())
                            .then_some(&graphics[source_offset..source_offset + len])
                    }) else {
                        if let Some(previous_obj_vram) = self.ppu.obj_previous_frame_vram.as_ref() {
                            self.ppu.vram[destination..destination_end]
                                .copy_from_slice(&previous_obj_vram[destination..destination_end]);
                        }
                        continue;
                    };
                    for (index, bytes) in source_bytes.chunks_exact(2).enumerate() {
                        self.ppu.vram[destination + index] =
                            u16::from_le_bytes([bytes[0], bytes[1]]);
                    }
                }
            } else if let Some(previous_obj_vram) = self.last_presented_obj_vram.as_deref() {
                self.ppu.vram[0x4000..0x4050].copy_from_slice(&previous_obj_vram[0x000..0x050]);
                self.ppu.vram[0x4100..0x4150].copy_from_slice(&previous_obj_vram[0x100..0x150]);
                // The transition main slice has authored Link and the sorted
                // sprite table, so active display evaluates the live OAM image.
                // Entries visible in both the completed shadow and the live
                // sorted table have already entered Snes9x's evaluated OBJ
                // list. Overlay those same-slot entries while allowing newly
                // visible live entries to replace offscreen shadow sentinels;
                // retired shadow-only slots must not be duplicated.
                self.ppu.oam.clone_from(&following.ppu.oam);
                compose_visible_published_oam(
                    &mut self.ppu.oam,
                    following.published_shadow_oam_dma.as_deref(),
                );
            }
        }
        if following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 2
            && plan.link_obj_scanout_generation == GraphicsDmaGeneration::LiveAfterMain
        {
            // State 2 suppresses the ordinary Link DMA, so the early body,
            // head, and hand ranges remain on the image completed in state 1.
            // The live source words already describe the following upload and
            // would advance these ranges one scanout too early.
            if let Some(completed_obj_vram) =
                self.last_presented_obj_vram.as_deref().or_else(|| {
                    self.pre_main_graphics_dma
                        .as_ref()
                        .map(|graphics| graphics.link_obj_vram.as_slice())
                })
            {
                self.ppu.vram[0x4000..0x4050].copy_from_slice(&completed_obj_vram[0x000..0x050]);
                self.ppu.vram[0x4100..0x4150].copy_from_slice(&completed_obj_vram[0x100..0x150]);
            }

            if self.screen_transition() == 1 {
                // This transition direction retains the raw Link image while
                // Snes9x's decoded cache has already consumed the current head
                // operands. Direction 2 instead keeps the prior decoded head
                // generation alongside the retained raw image.
                let live_sources = LinkDmaSources::load_from_ram(&following.ram);
                let link_graphics = self.asset_raw(57).map(Vec::from);
                let mut obj_cache_vram = self.ppu.vram.clone();
                for (destination, source) in [
                    (0x4020, LinkDmaSourceSlot::HeadTop),
                    (0x4120, LinkDmaSourceSlot::HeadBottom),
                ] {
                    let len = 0x40;
                    let source_address = usize::from(live_sources.source(source));
                    let source_offset = source_address.saturating_sub(0x8000);
                    let destination_end = destination + len / 2;
                    let Some(source_bytes) = link_graphics.as_deref().and_then(|graphics| {
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
                self.ppu.obj_vram_latch = Some(obj_cache_vram);
            }
        }
        let dungeon_supertile_first_scroll_uses_live_obj_cache = following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 1
            && plan.oam_scanout_source == OamScanoutSource::RetainResidentPpuOam;
        if plan.link_obj_source_generation == GraphicsDmaGeneration::LiveAfterMain
            && !dungeon_supertile_first_scroll_uses_live_obj_cache
            && (self.atomic_item_graphics_holds_following_nmi()
                || following.interrupted_item_receipt_obj_cache)
        {
            // Only typed interrupted returns own this decoded-cache split. A
            // coarse host/live Link generation elsewhere must not inherit a
            // previous decoded head generation.
            let mut obj_cache_vram = self.ppu.vram.clone();
            let captured_sources = LinkDmaSources::load_from_ram(&self.ram);
            let link_graphics = self.asset_raw(57).map(Vec::from);
            for (destination, source, len) in EARLY_LINK_OBJ_DMA_TRANSFERS {
                let source_address = usize::from(captured_sources.source(source));
                let source_offset = source_address.saturating_sub(0x8000);
                let destination_end = destination + len / 2;
                let Some(source_bytes) = link_graphics.as_deref().and_then(|graphics| {
                    (source_address >= 0x8000 && source_offset + len <= graphics.len())
                        .then_some(&graphics[source_offset..source_offset + len])
                }) else {
                    obj_cache_vram[destination..destination_end]
                        .copy_from_slice(&following.ppu.vram[destination..destination_end]);
                    continue;
                };
                for (index, bytes) in source_bytes.chunks_exact(2).enumerate() {
                    obj_cache_vram[destination + index] = u16::from_le_bytes([bytes[0], bytes[1]]);
                }
            }
            if following.interrupted_item_receipt_obj_cache
                && plan.oam_scanout_source == OamScanoutSource::ComposeLiveAfterNmi
            {
                // The uncle-passage return completes the WRAM-backed shield
                // upload before scanout, then a later held NMI restores the
                // resident raw VRAM words. Snes9x therefore retains these four
                // newly decoded cache tiles even though its visible raw VRAM
                // still matches the preceding boundary.
                for (destination, source) in [
                    (0x4070, LinkDmaSourceSlot::ShieldUpper),
                    (0x4170, LinkDmaSourceSlot::ShieldLower),
                ] {
                    const LEN: usize = 0x40;
                    let source_address = usize::from(captured_sources.source(source));
                    let destination_end = destination + LEN / 2;
                    if let Some(source_bytes) =
                        self.ram.get(source_address..source_address + LEN)
                    {
                        for (word, bytes) in obj_cache_vram[destination..destination_end]
                            .iter_mut()
                            .zip(source_bytes.chunks_exact(2))
                        {
                            *word = u16::from_le_bytes([bytes[0], bytes[1]]);
                        }
                    }
                }
            }
            for range in [0x4240..0x4250, 0x4340..0x4350] {
                obj_cache_vram[range.clone()].copy_from_slice(&following.ppu.vram[range]);
            }
            // The two right-hand quadrants of the 16x16 receipt sprite were
            // not decoded by the interrupted transfer. Snes9x leaves those
            // cache entries transparent instead of exposing their resident
            // VRAM contents.
            obj_cache_vram[0x4250..0x4260].fill(0);
            obj_cache_vram[0x4350..0x4360].fill(0);
            self.ppu.obj_vram_latch = Some(obj_cache_vram);
        } else {
            let dungeon_subtile_palette_filter_scanout = following_frame.main_module == 7
                && following_frame.submodule == 1
                && matches!(following_frame.subsubmodule, 6 | 7)
                && plan.oam_scanout_source == OamScanoutSource::RetainResidentPpuOam;
            let dungeon_subtile_palette_filter_uses_live_obj_cache =
                dungeon_subtile_palette_filter_scanout
                    && plan.link_obj_scanout_generation == GraphicsDmaGeneration::LiveAfterMain
                    && plan.link_obj_source_generation == GraphicsDmaGeneration::LiveAfterMain;
            let dungeon_subtile_palette_filter_holds_decoded_obj_cache =
                dungeon_subtile_palette_filter_scanout
                    && plan.link_obj_scanout_generation
                        == GraphicsDmaGeneration::HostBoundaryBeforeMain
                    && plan.link_obj_source_generation
                        == GraphicsDmaGeneration::HostBoundaryBeforeMain;
            let interrupted_spiral_stairs_first_palette_pass = following_frame.main_module == 7
                && following_frame.submodule == 0x0e
                && following_frame.subsubmodule == 2
                && self.pre_main_caller_continuation_is(
                    PreMainCallerContinuation::SpiralStairsSecondPaletteFilter,
                );
            if dungeon_subtile_palette_filter_uses_live_obj_cache {
                // The palette loop can hold NMI_DoUpdates for a host frame.
                // Its captured source words still identify the decoded cache
                // generation already owned by this scanout, including the
                // generation made visible by the preceding boundary.
                // The ensuing NMI can consume live Link operands without
                // invalidating the tiles Snes9x already decoded for this
                // scanout. Keep the whole decoded Link batch on the captured
                // pre-main source generation; the next boundary owns the new
                // pose.
                let captured_sources = self
                    .pre_main_graphics_dma
                    .as_ref()
                    .map(|graphics| graphics.link_operands.sources)
                    .unwrap_or_else(|| LinkDmaSources::load_from_ram(&self.ram));
                let obj_cache_vram = compose_early_link_obj_cache(
                    &self.ppu.vram,
                    captured_sources,
                    self.asset_raw(57),
                );
                if env::var_os("ZELDA3_DEBUG_DISPLAY_OAM").is_some() {
                    eprintln!(
                        "display_subtile_cache host={} dma={} last={} head_source={:04x} raw_head={:04x} cache_head={:04x}",
                        self.frame_ctr_dbg,
                        self.link_obj_dma_completed_this_frame,
                        self.last_presented_obj_vram.is_some(),
                        captured_sources.source(LinkDmaSourceSlot::HeadTop),
                        self.ppu.vram[0x4020],
                        obj_cache_vram[0x4020],
                    );
                }
                self.ppu.obj_vram_latch = Some(obj_cache_vram);
            } else if dungeon_subtile_palette_filter_holds_decoded_obj_cache {
                // A held NMI does not decode another Link batch. Raw OBJ VRAM
                // still belongs to its independently retained generation, so
                // carry forward the cache that the preceding scanout actually
                // presented instead of falling back to those raw words.
                if let Some(previous) = self.last_presented_obj_vram.as_deref() {
                    let mut obj_cache_vram = self.ppu.vram.clone();
                    obj_cache_vram[0x4000..0x4400].copy_from_slice(previous);
                    self.ppu.obj_vram_latch = Some(obj_cache_vram);
                }
            } else if dungeon_supertile_first_scroll_uses_live_obj_cache {
                // The first steady supertile-scroll scanout retains the raw
                // host-boundary Link tiles, but Snes9x's renderer cache has
                // already decoded the live NMI upload. Keep that cache-only
                // generation independent from the retained visible VRAM.
                let live_sources = LinkDmaSources::load_from_ram(&following.ram);
                let link_graphics = self.asset_raw(57).map(Vec::from);
                let mut obj_cache_vram = self.ppu.vram.clone();
                for (destination, source, len) in EARLY_LINK_OBJ_DMA_TRANSFERS {
                    let source_address = usize::from(live_sources.source(source));
                    let source_offset = source_address.saturating_sub(0x8000);
                    let destination_end = destination + len / 2;
                    let Some(source_bytes) = link_graphics.as_deref().and_then(|graphics| {
                        (source_address >= 0x8000 && source_offset + len <= graphics.len())
                            .then_some(&graphics[source_offset..source_offset + len])
                    }) else {
                        obj_cache_vram[destination..destination_end]
                            .copy_from_slice(&following.ppu.vram[destination..destination_end]);
                        continue;
                    };
                    for (index, bytes) in source_bytes.chunks_exact(2).enumerate() {
                        obj_cache_vram[destination + index] =
                            u16::from_le_bytes([bytes[0], bytes[1]]);
                    }
                }
                self.ppu.obj_vram_latch = Some(obj_cache_vram);
            } else if room_71_supertile_room_load_uses_composed_obj_cache(
                following_frame,
                following_room,
            ) {
                self.ppu.obj_vram_latch = Some(self.ppu.vram.clone());
            } else if interrupted_spiral_stairs_first_palette_pass {
                // The first palette walk crosses vblank after Link's early
                // body/head/hand DMA has updated Snes9x's OBJ tile cache, but
                // before that NMI's raw VRAM generation owns active scanout.
                // Resolve the renderer-only cache from the live source words
                // while retaining the independently selected raw VRAM image.
                let live_sources = LinkDmaSources::load_from_ram(&following.ram);
                let link_graphics = self.asset_raw(57).map(Vec::from);
                let mut obj_cache_vram = self.ppu.vram.clone();
                for (destination, source, len) in EARLY_LINK_OBJ_DMA_TRANSFERS {
                    let source_address = usize::from(live_sources.source(source));
                    let source_offset = source_address.saturating_sub(0x8000);
                    let destination_end = destination + len / 2;
                    let Some(source_bytes) = link_graphics.as_deref().and_then(|graphics| {
                        (source_address >= 0x8000 && source_offset + len <= graphics.len())
                            .then_some(&graphics[source_offset..source_offset + len])
                    }) else {
                        obj_cache_vram[destination..destination_end]
                            .copy_from_slice(&following.ppu.vram[destination..destination_end]);
                        continue;
                    };
                    for (index, bytes) in source_bytes.chunks_exact(2).enumerate() {
                        obj_cache_vram[destination + index] =
                            u16::from_le_bytes([bytes[0], bytes[1]]);
                    }
                }
                self.ppu.obj_vram_latch = Some(obj_cache_vram);
            }
        }
        if room_71_supertile_room_load_uses_composed_obj_cache(following_frame, following_room) {
            // Room-load scanout keeps Link's published OAM coordinates while
            // decoding the already-composed current Link tile generation.
            self.ppu.obj_vram_latch = Some(self.ppu.vram.clone());
        }
        if following_frame.main_module == 7
            && following_frame.submodule == 2
            && following_frame.subsubmodule == 3
            && !matches!(following_room, 0x70 | 0x71)
        {
            // The raw Link ranges remain on the prior completed DMA image, but
            // Snes9x has already decoded the post-NMI OBJ page used by this
            // scanout. Preserve that renderer-only generation independently.
            let mut obj_cache_vram = following.ppu.vram.clone();
            if self.screen_transition() == 1 {
                // Direction 1's state-3 scanout has decoded the same current
                // head operands as the preceding state-2 scanout even though
                // the native PPU snapshot has moved to a different raw Link
                // generation. Recompose only those cache entries; direction 2
                // already matches the following PPU cache verbatim.
                let captured_sources = self
                    .pre_main_graphics_dma
                    .as_ref()
                    .map(|graphics| graphics.link_operands.sources)
                    .unwrap_or_else(|| LinkDmaSources::load_from_ram(&self.ram));
                let link_graphics = self.asset_raw(57).map(Vec::from);
                for (destination, source) in [
                    (0x4020, LinkDmaSourceSlot::HeadTop),
                    (0x4120, LinkDmaSourceSlot::HeadBottom),
                ] {
                    let len = 0x40;
                    let source_address = usize::from(captured_sources.source(source));
                    let source_offset = source_address.saturating_sub(0x8000);
                    let destination_end = destination + len / 2;
                    let Some(source_bytes) = link_graphics.as_deref().and_then(|graphics| {
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
            }
            self.ppu.obj_vram_latch = Some(obj_cache_vram);
        }
        self.ppu.obj_previous_frame_vram = following.ppu.obj_previous_frame_vram.clone();
    }

    fn compose_display_raster(
        &mut self,
        live_forced_blank: bool,
        mut live_forced_blank_from_scanline: Option<u8>,
        live_retain_active_display_history: bool,
        captured_screen_brightness: u8,
        plan: &DisplayPublicationPlan,
    ) {
        let captured_frame = crate::game_state::FrameState::load_from_ram(&self.ram);
        if live_forced_blank
            && live_forced_blank_from_scanline.is_none()
            && captured_frame.main_module == 14
            && captured_frame.submodule == 3
            && self.ram[OVERWORLD_MAP_STATE] == 1
            && self.ppu.brightness == 1
        {
            // Dungeon-map setup's terminal fade overruns into active display:
            // the HUD-height prefix scans at brightness 1, then the late
            // INIDISP write blanks the playfield from scanline 27 onward.
            live_forced_blank_from_scanline = Some(27);
        }
        // A force-blank write published by NMI takes effect before the next
        // active scanline even when other domains retain the pre-NMI snapshot.
        self.ppu.forced_blank |= live_forced_blank;
        if live_forced_blank {
            let scanout = resolve_active_display_blanking_scanout(
                self.ppu.retain_active_display_history,
                live_forced_blank_from_scanline,
                live_retain_active_display_history,
            );
            self.ppu.forced_blank_from_scanline = scanout.suffix_start_scanline;
            self.ppu.retain_active_display_history = scanout.retain_prior_surface;
        }
        self.ppu.mode7_scanout_brightness_override = plan
            .world_map_mode7_brightness_is_early_published
            .then_some(captured_screen_brightness);
    }

    fn displayed_dialogue_scanout(&self) -> Option<DialogueTextScanout> {
        match self.dialogue_scroll_phase() {
            DialogueScrollPhase::CopyingRemainingPixels { .. }
            | DialogueScrollPhase::ReturnOnly
            | DialogueScrollPhase::CompletionPendingPublication
            | DialogueScrollPhase::CompletionStagedAfterFrozenScanout => {
                self.dialogue_scroll_frozen_scanout.clone()
            }
            DialogueScrollPhase::CompletedScroll => self.dialogue_scroll_completion_scanout.clone(),
            DialogueScrollPhase::Idle | DialogueScrollPhase::CompletionStagedAfterSnapshot => None,
        }
    }

    fn resolve_displayed_dialogue_metadata(
        &self,
        pristine_snapshot: &DisplaySnapshot,
        dialogue_scanout: Option<&DialogueTextScanout>,
        vram_generation: DisplayVramGeneration,
    ) -> PublishedDialogueMetadata {
        if let Some(scanout) = dialogue_scanout {
            PublishedDialogueMetadata::from_scanout(scanout)
        } else if vram_generation == DisplayVramGeneration::RetainCapturedBeforeNmi {
            PublishedDialogueMetadata::from_snapshot(pristine_snapshot)
        } else {
            PublishedDialogueMetadata::from_live_state(self)
        }
    }

    /// Runs a renderer capture against the coherent pre-NMI display state while
    /// leaving the live post-NMI simulation untouched.
    ///
    /// This is the shared publication boundary for both the scanline renderer
    /// and the modern asset/GPU renderer. The returned value must own anything
    /// it borrows from `game`, because live state is restored before returning.
    pub fn with_display_snapshot<R>(&mut self, capture: impl FnOnce(&mut ZeldaState) -> R) -> R {
        let from_display_slot = self.display_snapshot.is_some();
        let Some(mut display) = self
            .display_snapshot
            .take()
            .or_else(|| self.visible_display_snapshot.take())
        else {
            return capture(self);
        };
        if self.presented_history_host_frame != Some(self.frame_ctr_dbg) {
            if self.presented_history_host_frame.is_some() {
                if let Some(oam) = self.staged_presented_oam.take() {
                    self.last_presented_oam = Some(oam);
                }
                if let Some(vram) = self.staged_presented_obj_vram.take() {
                    self.last_presented_obj_vram = Some(vram);
                }
                if let Some(source) = self.staged_presented_vram_chr_source.take() {
                    self.last_presented_vram_chr_source = Some(source);
                }
                if let Some(source) = self.staged_presented_vram_chr_preview_source.take() {
                    self.last_presented_vram_chr_preview_source = Some(source);
                }
            }
            self.presented_history_host_frame = Some(self.frame_ctr_dbg);
        }
        // Capture must be side-effect-free on live game state. The native
        // game_state is re-derived from the snapshot RAM below so capture
        // closures see coherent native views; restore the ORIGINAL native
        // state afterwards instead of re-deriving it from live RAM — a
        // RAM-derived rebuild rewinds every native field whose RAM projection
        // is stale mid-frame (write-through fields, animation countdowns),
        // which made per-frame video capture perturb game behavior.
        let saved_game_state = self.game_state.clone();
        // Compose mutates the snapshot side (VRAM/OAM/CGRAM composition,
        // latch clears). Store back the PRISTINE snapshot so repeated captures
        // and later consumers see exactly what NMI published.
        let pristine_snapshot = display.clone();
        let diagnostics = DisplayDiagnostics::from_env();

        let snapshot_frame = crate::game_state::FrameState::load_from_ram(&display.ram);
        let snapshot_attract_scene =
            crate::game_state::AttractSceneState::load_from_ram(&display.ram);
        let snapshot_messaging = crate::game_state::MessagingState::load_from_ram(&display.ram);
        let pending_main_thread_stripe = display.ram[NMI_LOAD_BG_FROM_VRAM] == 1;
        let pending_full_tilemap_upload =
            display.ram[crate::game_state::constants::NMI_SUBROUTINE_INDEX] == 1;
        let live_pending_main_thread_stripe = self.ram[NMI_LOAD_BG_FROM_VRAM] == 1;
        // RenderText_Draw_Finish authors the fixed-source stripe that replaces
        // the dialogue box with tile 0x387f, then returns to the saved module.
        // When live NMI has consumed that exact packet, Snes9x scans out the
        // cleared BG3 tilemap immediately. The ordinary menu-stripe cadence
        // remains deferred even after its live flag is cleared.
        let consumed_dialogue_box_clear = pending_main_thread_stripe
            && !live_pending_main_thread_stripe
            && stripe_upload_clears_dialogue_box(
                &display.ram[crate::game_state::constants::VRAM_UPLOAD_DATA..],
            );
        let retained_full_tilemap_vram = rom_full_tilemap_scanout_retains_uploaded_region(
            pending_full_tilemap_upload,
            display.ppu.forced_blank_scanlines,
        )
        .then(|| {
            let target_page = display.ram[crate::game_state::constants::NMI_LOAD_TARGET_ADDR];
            let (destination, word_count) = full_tilemap_nmi_vram_region(target_page)?;
            RetainedVramRegion::capture(&display.ppu.vram, destination, word_count)
        })
        .flatten();
        let retain_previous_nmi_display_memory = (rom_display_memory_publication_is_deferred(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
            snapshot_messaging.runtime.text_render_state(),
            pending_main_thread_stripe,
        ) && !consumed_dialogue_box_clear)
            || (snapshot_frame.main_module == 20
                && snapshot_frame.submodule == 0
                // Snes9x retains the pre-NMI attract image through the sequence-1
                // load/fade-out. Mode 7 begins publishing immediately once that
                // sequence has entered its fade-in state.
                && !(snapshot_attract_scene.sequence() == 1
                    && snapshot_attract_scene.state() >= 4));
        let active_display_nmi_overrun = display.ppu.forced_blank_scanlines != 0;
        let module_oam_publication_is_deferred = rom_display_oam_publication_is_deferred(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
            snapshot_messaging.runtime.text_render_state(),
            active_display_nmi_overrun,
            pending_main_thread_stripe,
        );
        let dungeon_exit_crosses_nmi_boundary = rom_dungeon_exit_entry_crosses_nmi_boundary(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
            self.game_state.frame.main_module,
            self.game_state.frame.submodule,
            matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishDungeonExitSpotlightEntry)
            ),
        );
        let world_map_fade_display = snapshot_frame.main_module == 20
            && snapshot_frame.submodule == 0
            && snapshot_attract_scene.sequence() == 1
            && snapshot_attract_scene.state() >= 4;
        let world_map_mode7_brightness_is_early_published =
            rom_attract_world_map_mode7_brightness_is_early_published(
                snapshot_frame.main_module,
                snapshot_frame.submodule,
                snapshot_attract_scene.sequence(),
                snapshot_attract_scene.state(),
            );
        // At the same split boundary, OAM above, Link OBJ VRAM below, and the
        // doorway scroll publish from live state while the remaining staged
        // display controls retain their independently measured generation.
        let publish_live_overworld_bad_weather_scroll = rom_overworld_bad_weather_scroll_is_live(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
            self.game_state.frame.main_module,
            self.game_state.frame.submodule,
            display.ppu.bg_layer[0].h_scroll,
            display.ppu.bg_layer[0].v_scroll,
            display.ppu.bg_layer[1].h_scroll,
            display.ppu.bg_layer[1].v_scroll,
            self.ppu.bg_layer[0].h_scroll,
            self.ppu.bg_layer[0].v_scroll,
            self.ppu.bg_layer[1].h_scroll,
            self.ppu.bg_layer[1].v_scroll,
        );
        let publish_live_overworld_transition_half_color =
            rom_overworld_transition_half_color_is_live(
                snapshot_frame.main_module,
                snapshot_frame.submodule,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                display.ppu.half_color,
                self.ppu.half_color,
            );
        let publication_signals = DisplayPublicationSignals {
            retain_previous_nmi_display_memory,
            module_oam_publication_is_deferred,
            dungeon_exit_crosses_nmi_boundary,
            publish_live_overworld_bad_weather_scroll,
            publish_live_overworld_transition_half_color,
            attract_map_retains_display_memory: retain_previous_nmi_display_memory
                && snapshot_frame.main_module == 20
                && snapshot_frame.submodule == 0,
            world_map_fade_display,
            world_map_mode7_brightness_is_early_published,
            // The entry scanout keeps the captured camera while composing the
            // mixed Link-OAM handoff. Starting with the following hold-item
            // scanout, the camera written by that handoff is live. Keeping the
            // two phases separate avoids publishing the scroll one frame early.
            dungeon_item_hold_publishes_live_scroll: dungeon_item_hold_publishes_live_scroll(
                snapshot_frame,
                FollowerLinkState::load_from_ram(&display.ram).handler_state(),
                display.dungeon_item_hold_entry_scanout,
            ),
        };
        let publication_plan = DisplayPublicationPlan::resolve(&display, publication_signals);
        let display_phase_differs_from_live = snapshot_frame.main_module
            != self.game_state.frame.main_module
            || snapshot_frame.submodule != self.game_state.frame.submodule;
        if diagnostics.display_oam && display_phase_differs_from_live {
            eprintln!(
                "display_oam snapshot={:02x}/{:02x} live={:02x}/{:02x} retain={} reasons=module:{}/captured_ppu:{}/dungeon_exit:{} snapshot_math={:02x}/{:02x}/{}/{} live_math={:02x}/{:02x}/{}/{} snapshot_oam={:02x?} live_oam={:02x?}",
                snapshot_frame.main_module,
                snapshot_frame.submodule,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                publication_plan.retain_captured_oam,
                module_oam_publication_is_deferred,
                matches!(
                    publication_plan.oam_scanout_source,
                    OamScanoutSource::RetainCapturedBeforeNmi
                ),
                dungeon_exit_crosses_nmi_boundary,
                display.ppu.math_enabled,
                display.ppu.prevent_math_mode,
                display.ppu.subtract_color,
                display.ppu.half_color,
                self.ppu.math_enabled,
                self.ppu.prevent_math_mode,
                self.ppu.subtract_color,
                self.ppu.half_color,
                &display.ppu.oam[..4],
                &self.ppu.oam[..4],
            );
        }
        let live_forced_blank = self.ppu.forced_blank;
        let live_forced_blank_from_scanline = self
            .active_display_force_blank_event
            .or(self.ppu.forced_blank_from_scanline);
        let live_retain_active_display_history = self.ppu.retain_active_display_history;
        // This capture owns the game-authored INIDISP value for the active
        // frame. The live PPU has already run the following NMI boundary and
        // can therefore be one fade step ahead.
        let captured_screen_brightness =
            display.ram[crate::game_state::constants::INIDISP_COPY] & 0x0f;
        if diagnostics.nmi_latch && (1648..=1655).contains(&self.frame_ctr_dbg) {
            eprintln!(
                "display_blanking host={} snapshot_forced={} snapshot_prefix={} snapshot_from={:?} live_forced={} live_from={:?}",
                self.frame_ctr_dbg,
                display.ppu.forced_blank,
                display.ppu.forced_blank_scanlines,
                display.ppu.forced_blank_from_scanline,
                live_forced_blank,
                live_forced_blank_from_scanline,
            );
        }
        std::mem::swap(&mut self.ram, &mut display.ram);
        std::mem::swap(&mut self.ppu, &mut display.ppu);
        std::mem::swap(&mut self.dma, &mut display.dma);
        std::mem::swap(&mut self.vram_chr_source, &mut display.vram_chr_source);
        std::mem::swap(
            &mut self.vram_chr_preview_source,
            &mut display.vram_chr_preview_source,
        );
        self.compose_display_registers(&display, &publication_plan);
        let previous_dialogue_scanout = self.displayed_dialogue_scanout();
        if diagnostics.scroll_retain && self.dialogue_scroll_phase() != DialogueScrollPhase::Idle {
            let dialogue_scroll_phase = self.dialogue_scroll_phase();
            let scanout_sum = |scanout: &DialogueTextScanout| {
                scanout
                    .vram
                    .iter()
                    .map(|&word| u64::from(word & 0xff) + u64::from(word >> 8))
                    .sum::<u64>()
            };
            eprintln!(
                "scroll_retain host={} phase={dialogue_scroll_phase:?} frozen={:?} completion={:?} nmi_retained={}",
                self.frame_ctr_dbg,
                self.dialogue_scroll_frozen_scanout.as_ref().map(&scanout_sum),
                self.dialogue_scroll_completion_scanout
                    .as_ref()
                    .map(&scanout_sum),
                retain_previous_nmi_display_memory,
            );
        }
        self.compose_display_vram(
            &display,
            &publication_plan,
            retained_full_tilemap_vram.as_ref(),
        );
        self.compose_display_chr_sources(&display, &publication_plan);
        self.staged_presented_vram_chr_source = Some(self.vram_chr_source.clone());
        self.staged_presented_vram_chr_preview_source = Some(self.vram_chr_preview_source.clone());
        self.compose_display_cgram(&display, &publication_plan);
        if let Some(previous_dialogue_scanout) = previous_dialogue_scanout.as_ref() {
            self.ppu.vram[0x7c00..0x7ff0].copy_from_slice(&previous_dialogue_scanout.vram);
        }
        if diagnostics.scroll_retain && self.dialogue_scroll_phase() != DialogueScrollPhase::Idle {
            let dialogue_scroll_phase = self.dialogue_scroll_phase();
            let presented_sum: u64 = self.ppu.vram[0x7c00..0x7ff0]
                .iter()
                .map(|w| u64::from(w & 0xff) + u64::from(w >> 8))
                .sum();
            eprintln!(
                "scroll_present host={} presented_sum={presented_sum} phase={dialogue_scroll_phase:?}",
                self.frame_ctr_dbg,
            );
        }
        self.compose_display_oam(&display, &publication_plan);
        self.staged_presented_oam = Some(self.ppu.oam.clone());
        let presented_obj_vram = self.ppu.obj_vram_latch.as_deref().unwrap_or(&self.ppu.vram);
        self.staged_presented_obj_vram = Some(presented_obj_vram[0x4000..0x4400].to_vec());
        self.compose_display_raster(
            live_forced_blank,
            live_forced_blank_from_scanline,
            live_retain_active_display_history,
            captured_screen_brightness,
            &publication_plan,
        );
        if diagnostics.nmi_latch && (1648..=1655).contains(&self.frame_ctr_dbg) {
            eprintln!(
                "display_blanking_composed host={} forced={} prefix={} from={:?}",
                self.frame_ctr_dbg,
                self.ppu.forced_blank,
                self.ppu.forced_blank_scanlines,
                self.ppu.forced_blank_from_scanline,
            );
        }
        self.sync_native_game_state_from_ram();
        // The RAM-derived rebuild reconstitutes the palette mirror from the
        // snapshot's WRAM shadow, which already holds THIS frame's palette
        // writes; hardware scanout shows the palette uploaded in the PREVIOUS
        // vblank. Re-publish the composed pre-NMI CGRAM image so effect
        // materials and live-CGRAM readers see what the PPU actually displays
        // (the attract palette filter diverges from Snes9x otherwise).
        self.game_state
            .display
            .palette_provenance
            .0
            .reconstitute_cgram(&self.ppu.cgram);
        if diagnostics.capture.attract_timeline && (5640..=5700).contains(&self.frame_ctr_dbg) {
            let trace = format!(
                "attract_display_present host={} snapshot={:02x}/{:02x} live={:02x}/{:02x} world_map_fade={} bright={} c0={:04x} c1={:04x} fixed={:02x},{:02x},{:02x} math={:02x}/{:02x}",
                self.frame_ctr_dbg,
                snapshot_frame.main_module,
                snapshot_frame.submodule,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                world_map_fade_display,
                self.ppu.brightness,
                self.ppu.cgram[0],
                self.ppu.cgram[1],
                self.ppu.fixed_color_r,
                self.ppu.fixed_color_g,
                self.ppu.fixed_color_b,
                self.ppu.math_enabled,
                self.ppu.prevent_math_mode,
            );
            append_parity_trace("attract-display-timeline.trace", &trace);
        }
        // Semantic dialogue data is another representation of the same BG3
        // generation as the presented VRAM. Resolve that generation once
        // above and use it for both representations: independently repeating
        // only part of the VRAM-retention condition lets semantic text lead
        // the exact hardware pixels during an interrupted VWF render.
        // A dialogue scroll override still owns both representations.
        let presented_dialogue = self.resolve_displayed_dialogue_metadata(
            &pristine_snapshot,
            previous_dialogue_scanout.as_ref(),
            publication_plan.vram_generation,
        );
        if diagnostics.capture.frame_boundary {
            let published_shadow_oam_diff =
                display.published_shadow_oam_dma.as_deref().map(|shadow| {
                    let mismatches = self
                        .ppu
                        .oam
                        .iter()
                        .flat_map(|word| word.to_le_bytes())
                        .zip(shadow.iter().flat_map(|word| word.to_le_bytes()))
                        .enumerate()
                        .filter_map(|(index, (presented, published))| {
                            (presented != published).then_some((index, presented, published))
                        })
                        .collect::<Vec<_>>();
                    (mismatches.len(), mismatches.first().copied())
                });
            eprintln!(
                "frame_boundary_present host={} vram={:?} animated_bg={:?} link_obj={:?} oam={:?} bg_scroll={:?} bg1=({:04x},{:04x}) retained_obj={} published_shadow_oam_diff={:?} dialogue_runs=live:{}/captured:{}/presented:{} scroll_override={}",
                self.frame_ctr_dbg,
                publication_plan.vram_generation,
                publication_plan.animated_bg_scanout_generation,
                publication_plan.link_obj_scanout_generation,
                publication_plan.oam_scanout_source,
                publication_plan.bg_scroll_source,
                self.ppu.bg_layer[0].h_scroll,
                self.ppu.bg_layer[0].v_scroll,
                matches!(
                    pristine_snapshot.obj_generation,
                    DisplayObjGeneration::RetainCapturedMemory { .. }
                ),
                published_shadow_oam_diff,
                self.published_bg3_vwf_glyph_runs.len(),
                pristine_snapshot.published_bg3_vwf_glyph_runs.len(),
                presented_dialogue.glyph_runs.len(),
                previous_dialogue_scanout.is_some(),
            );
        }
        let saved_published_dialogue = presented_dialogue.replace_in(self);
        let captured = capture(self);
        saved_published_dialogue.replace_in(self);

        std::mem::swap(&mut self.ram, &mut display.ram);
        std::mem::swap(&mut self.ppu, &mut display.ppu);
        std::mem::swap(&mut self.dma, &mut display.dma);
        std::mem::swap(&mut self.vram_chr_source, &mut display.vram_chr_source);
        std::mem::swap(
            &mut self.vram_chr_preview_source,
            &mut display.vram_chr_preview_source,
        );
        self.game_state = saved_game_state;
        drop(display);
        if from_display_slot {
            self.display_snapshot = Some(pristine_snapshot);
        } else {
            self.visible_display_snapshot = Some(pristine_snapshot);
        }
        captured
    }

    pub fn vram(&self) -> &[u16] {
        &self.ppu.vram
    }

    /// Read-only access to the per-VRAM-slot logical CHR source table
    /// (animation-modeled asset renderer M1 bookkeeping).
    pub fn vram_chr_source(&self) -> &crate::chr_source::VramChrSourceTable {
        &self.vram_chr_source
    }

    /// Read-only access to the raw per-VRAM-slot CHR source table used by
    /// authoring/preview tooling. This table preserves sprite pack/tile identity
    /// even when the render source table is content-hashed for correctness.
    pub fn vram_chr_preview_source(&self) -> &crate::chr_source::VramChrSourceTable {
        &self.vram_chr_preview_source
    }

    pub fn bg3_vwf_glyph_runs(&self) -> &[Bg3VwfGlyphRun] {
        &self.bg3_vwf_glyph_runs
    }

    pub fn published_bg3_vwf_glyph_runs(&self) -> &[Bg3VwfGlyphRun] {
        &self.published_bg3_vwf_glyph_runs
    }

    pub fn bg3_vwf_glyph_run_dialogue_offsets(&self) -> &[u16] {
        &self.bg3_vwf_glyph_run_dialogue_offsets
    }

    pub fn published_bg3_vwf_glyph_run_dialogue_offsets(&self) -> &[u16] {
        &self.published_bg3_vwf_glyph_run_dialogue_offsets
    }

    pub fn published_dialogue_message_id(&self) -> u16 {
        self.published_dialogue_message_id
    }

    pub fn dialogue_ir_for_decoded_bytes(
        &self,
        decoded: &[u8],
    ) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        zelda3_compat::legacy_dialogue_ir(self.dialogue_flags, decoded)
    }

    pub fn current_dialogue_ir(&self) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        zelda3_compat::legacy_dialogue_ir(
            self.dialogue_flags,
            self.game_state.messaging.decoded_text.as_slice(),
        )
    }

    pub fn current_dialogue_message_id(&self) -> u16 {
        self.game_state.messaging.dialogue_message_index.value()
    }

    /// Whether BG3 is currently owned by the dialogue renderer.
    ///
    /// The selected message id and decoded glyph state intentionally survive
    /// after a box closes, so neither is a valid presentation signal by itself.
    pub fn is_dialogue_display_active(&self) -> bool {
        self.game_state.messaging.runtime.module() != 0
    }

    pub fn set_current_dialogue_message_id(&mut self, message_id: u16) {
        self.dialogue_message_index_mut().set_value(message_id);
    }

    pub fn current_source_dialogue_ir(&self) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        self.source_dialogue_ir_for_message(self.current_dialogue_message_id())
            .unwrap_or_default()
    }

    pub fn current_dialogue_runtime_substitutions(
        &self,
    ) -> crate::dialogue_ir::DialogueRuntimeSubstitutions {
        let mut player_name = Vec::new();
        self.text_write_player_name_vec(&mut player_name);
        crate::dialogue_ir::DialogueRuntimeSubstitutions {
            player_name,
            number_pairs: [
                self.game_state.messaging.dialogue_number.packed_digits(0),
                self.game_state.messaging.dialogue_number.packed_digits(1),
            ],
        }
    }

    pub fn current_source_render_dialogue_ir(&self) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        let source_ir = self.current_source_dialogue_ir();
        if source_ir.is_empty() {
            return Vec::new();
        }
        crate::dialogue_ir::expand_runtime_dialogue_ir(
            &source_ir,
            &self.current_dialogue_runtime_substitutions(),
        )
    }

    pub fn current_visible_source_render_dialogue_ir(
        &self,
    ) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        let source_ir = self.current_source_dialogue_ir();
        if source_ir.is_empty() {
            return Vec::new();
        }
        let render_ir = crate::dialogue_ir::expand_runtime_render_dialogue_ir(
            &source_ir,
            &self.current_dialogue_runtime_substitutions(),
        );
        crate::dialogue_ir::visible_dialogue_ir_prefix(
            &render_ir,
            usize::from(self.game_state.messaging.runtime.dialogue_msg_read_pos()),
        )
    }

    /// Dialogue render IR that is actually on screen right now.
    ///
    /// Returns empty unless the text engine is currently handling a message
    /// (`MESSAGING_MODULE != 0`). The message id, read position, cached layout, and even the
    /// game's live BG3 glyph-run list all persist after a message closes, so a renderer keying off
    /// any of those would paint phantom glyphs over a closed box. The messaging module is the
    /// game's own "message open/rendering" state — what classic's BG3 content reflects — so gating
    /// on it keeps the hi-res VWF overlay in step with what the box actually shows.
    pub fn current_displayed_source_render_dialogue_ir(
        &self,
    ) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        if !self.is_dialogue_display_active() {
            return Vec::new();
        }
        self.current_visible_source_render_dialogue_ir()
    }

    /// Source dialogue semantics corresponding to the BG3 generation most
    /// recently committed by NMI_UploadBG3Text.
    pub fn published_displayed_source_render_dialogue_ir(
        &self,
    ) -> Vec<crate::dialogue_ir::DialogueIrOp> {
        if !self.is_dialogue_display_active() || self.published_bg3_vwf_glyph_runs.is_empty() {
            return Vec::new();
        }
        let Some(source_ir) =
            self.source_dialogue_ir_for_message(self.published_dialogue_message_id)
        else {
            return Vec::new();
        };
        let render_ir = crate::dialogue_ir::expand_runtime_render_dialogue_ir(
            &source_ir,
            &self.current_dialogue_runtime_substitutions(),
        );
        crate::dialogue_ir::visible_dialogue_ir_prefix(
            &render_ir,
            usize::from(self.published_dialogue_msg_read_pos),
        )
    }

    pub fn source_dialogue_ir_for_message(
        &self,
        message_id: u16,
    ) -> Option<Vec<crate::dialogue_ir::DialogueIrOp>> {
        self.assets
            .as_ref()
            .and_then(|assets| assets.source_dialogue_ir_for_message(message_id))
    }

    pub fn dialogue_vwf_widths(&self) -> Option<Vec<u8>> {
        let dialogue_font = self.asset_memblk(95, self.dialogue_font_blk_index)?;
        Some(find_index_in_memblk(dialogue_font, 1).ptr.to_vec())
    }

    pub fn dialogue_vwf_origin_tile_number(&self) -> u16 {
        self.game_state
            .messaging
            .vwf_render
            .tile_word_at_byte_offset(0)
            & 0x03ff
    }

    pub fn bg3_vwf_glyph_run_dialogue_ir(
        &self,
        run_index: usize,
    ) -> Option<crate::dialogue_ir::DialogueIrOp> {
        zelda3_compat::legacy_glyph_run_dialogue_ir(
            self.dialogue_flags,
            self.game_state.messaging.decoded_text.as_slice(),
            &self.bg3_vwf_glyph_run_dialogue_offsets,
            run_index,
        )
    }

    pub fn published_bg3_vwf_glyph_run_dialogue_ir(
        &self,
        run_index: usize,
    ) -> Option<crate::dialogue_ir::DialogueIrOp> {
        let offset = *self
            .published_bg3_vwf_glyph_run_dialogue_offsets
            .get(run_index)?;
        if offset == zelda3_compat::UNKNOWN_DIALOGUE_OFFSET {
            return None;
        }
        self.published_displayed_source_render_dialogue_ir()
            .into_iter()
            .find(|op| op.offset == usize::from(offset))
    }

    pub fn restore_bg3_vwf_glyph_runs(&mut self, runs: Vec<Bg3VwfGlyphRun>) {
        self.bg3_vwf_glyph_runs = runs;
        self.bg3_vwf_glyph_run_dialogue_offsets =
            vec![zelda3_compat::UNKNOWN_DIALOGUE_OFFSET; self.bg3_vwf_glyph_runs.len()];
    }

    pub(crate) fn clear_bg3_vwf_glyph_runs(&mut self) {
        self.bg3_vwf_glyph_runs.clear();
        self.bg3_vwf_glyph_run_dialogue_offsets.clear();
    }

    pub(crate) fn publish_bg3_vwf_glyph_runs(&mut self) {
        self.published_bg3_vwf_glyph_runs
            .clone_from(&self.bg3_vwf_glyph_runs);
        self.published_bg3_vwf_glyph_run_dialogue_offsets
            .clone_from(&self.bg3_vwf_glyph_run_dialogue_offsets);
        self.published_dialogue_msg_read_pos =
            self.game_state.messaging.runtime.dialogue_msg_read_pos();
        self.published_dialogue_message_id = self.bg3_vwf_glyph_run_dialogue_message_id;
    }

    pub(crate) fn record_bg3_vwf_glyph_run(
        &mut self,
        glyph_code: u8,
        glyph_x: u8,
        line_ptr: usize,
        width: u8,
        dialogue_offset: u16,
    ) {
        const TEXT_TILE_ROW_BYTES: usize = 0x150;
        const TILE_PIXEL_WIDTH: usize = 8;

        if width == 0 {
            return;
        }

        self.bg3_vwf_glyph_run_dialogue_message_id = self.current_dialogue_message_id();

        let tile_row = line_ptr / TEXT_TILE_ROW_BYTES;
        let origin_tile_number = self
            .game_state
            .messaging
            .vwf_render
            .tile_word_at_byte_offset(0)
            & 0x03ff;
        self.bg3_vwf_glyph_runs.push(Bg3VwfGlyphRun {
            glyph_code: u16::from(glyph_code),
            origin_tile_number,
            x: i16::from(glyph_x),
            y: (tile_row * TILE_PIXEL_WIDTH) as i16,
            width,
        });
        self.bg3_vwf_glyph_run_dialogue_offsets
            .push(dialogue_offset);
    }

    pub(crate) fn scroll_bg3_vwf_glyph_runs_up_one_pixel(&mut self) {
        for run in &mut self.bg3_vwf_glyph_runs {
            run.y -= 1;
        }
        let mut next_runs = Vec::with_capacity(self.bg3_vwf_glyph_runs.len());
        let mut next_offsets = Vec::with_capacity(self.bg3_vwf_glyph_run_dialogue_offsets.len());
        for (index, run) in self.bg3_vwf_glyph_runs.iter().copied().enumerate() {
            if run.y > -16 {
                next_runs.push(run);
                next_offsets.push(
                    self.bg3_vwf_glyph_run_dialogue_offsets
                        .get(index)
                        .copied()
                        .unwrap_or(zelda3_compat::UNKNOWN_DIALOGUE_OFFSET),
                );
            }
        }
        self.bg3_vwf_glyph_runs = next_runs;
        self.bg3_vwf_glyph_run_dialogue_offsets = next_offsets;
    }

    pub fn vram_mut(&mut self) -> &mut [u16] {
        &mut self.ppu.vram
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
    pub fn apply_link_graphics(&mut self, file: &[u8]) -> bool {
        if file.len() < 27 || &file[0..4] != b"ZSPR" {
            return false;
        }

        let Ok(pixel_offs) = read_le_u32(file, 9).map(|v| v as usize) else {
            return false;
        };
        let pixel_length = read_le_u16(file, 13) as usize;
        let Ok(palette_offs) = read_le_u32(file, 15).map(|v| v as usize) else {
            return false;
        };
        let palette_length = read_le_u16(file, 19) as usize;
        let pixel_end = match pixel_offs.checked_add(pixel_length) {
            Some(end) => end,
            None => return false,
        };
        let palette_end = match palette_offs.checked_add(palette_length) {
            Some(end) => end,
            None => return false,
        };
        if pixel_end > file.len() || palette_end > file.len() || pixel_length != 0x7000 {
            return false;
        }

        let Some(assets) = self.assets.as_mut() else {
            return false;
        };
        if assets.asset(57).map(|asset| asset.len()) != Some(0x7000)
            || assets.asset(81).map(|asset| asset.len()) != Some(150)
        {
            return false;
        }

        let Some(link_graphics) = assets.asset_mut(57) else {
            return false;
        };
        link_graphics.copy_from_slice(&file[pixel_offs..pixel_offs + 0x7000]);

        if palette_length >= 120 {
            let Some(armor_and_gloves) = assets.asset_mut(81) else {
                return false;
            };
            armor_and_gloves[..120].copy_from_slice(&file[palette_offs..palette_offs + 120]);
        }
        if palette_length >= 124 {
            self.gloves_color = [
                read_word_from_slice(file, palette_offs + 120),
                read_word_from_slice(file, palette_offs + 122),
            ];
        }

        true
    }

    /// Resume a translated caller suffix that owns this pre-main CPU slice.
    ///
    /// Returning `true` means the caller reached its trailing NMI and consumed
    /// the host frame; no fresh module iteration may run afterward.
    fn resume_pre_main_caller_continuation(
        &mut self,
        input: u16,
        oam_dma_source: Option<&[u8]>,
    ) -> bool {
        let Some(continuation) = self.game_execution_scheduler.pre_main_caller_continuation()
        else {
            return false;
        };

        match continuation {
            PreMainCallerContinuation::DialogueVwfReturn => {
                self.finish_pre_main_caller_continuation(continuation);
                self.dialogue_fast_forward_hold_active = false;
                self.complete_module0e_interface_after_run();
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
                self.capture_display_snapshot();
                // The interrupted caller has now reached the ordinary
                // game-loop boundary. Run its trailing NMI exactly once:
                // it consumes the preprocessed-audio marker and publishes
                // the completed BG3 text for the following scanout.
                self.interrupt_nmi(input, oam_dma_source, false);
                self.dialogue_vwf_handler_entry_phase =
                    messaging::VwfHandlerEntryPhase::AfterDeferredCallerSuffix;
            }
            PreMainCallerContinuation::FileSelectCheckerboardUpload => {
                self.complete_file_select_checkerboard_upload();
                // This continuation resumes after the prior CPU slice crossed
                // an NMI boundary. The completed stripe packet is consumable
                // now instead of carrying the stale latch into the next upload.
                self.clear_nmi_update_latch();
                self.capture_display_snapshot();
                self.interrupt_nmi(input, oam_dma_source, false);
            }
            PreMainCallerContinuation::NamePlayerTilemapUpload => {
                self.complete_module_name_player_1();
                // SelectFile_Func1 crosses exactly one vblank. Its suffix has
                // returned through Module_MainRouting, so Main_PrepSpritesForNmi
                // may publish the tilemap packet.
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
                self.capture_display_snapshot();
                self.interrupt_nmi(input, oam_dma_source, false);
            }
            PreMainCallerContinuation::SpiralStairsSecondPaletteFilter => {
                self.finish_pre_main_caller_continuation(continuation);
                self.complete_spiral_stairs_second_palette_filter();
                // Resume the ordinary game-loop suffix without repeating its
                // frame-counter/OAM-clear prefix or the Link movement that ran
                // before the interrupted first palette pass.
                self.complete_module07_dungeon_after_submodule();
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
                self.next_display_obj_scanout_generation = Some(ObjScanoutGenerations {
                    oam: OamScanoutSource::RetainResidentPpuOam,
                    link_obj: GraphicsDmaGeneration::LiveAfterMain,
                    link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
                });
                self.capture_display_snapshot();
                self.interrupt_nmi(input, oam_dma_source, false);
            }
            PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter => {
                self.finish_pre_main_caller_continuation(continuation);
                let animated_bg_operands = self.stage_spiral_stairs_second_grayscale_nmi();
                self.complete_spiral_stairs_second_grayscale_palette_filter();
                self.complete_module07_dungeon_after_submodule();
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
                self.next_display_obj_scanout_generation = Some(ObjScanoutGenerations {
                    oam: OamScanoutSource::RetainResidentPpuOam,
                    link_obj: GraphicsDmaGeneration::LiveAfterMain,
                    link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
                });
                self.capture_display_snapshot();
                self.interrupt_nmi_with_animated_bg_operands(
                    input,
                    oam_dma_source,
                    false,
                    Some(animated_bg_operands),
                );
            }
        }
        true
    }

    /// Resume game-thread work whose caller returns through a leading NMI.
    ///
    /// The continuation owns the display generations at that boundary. A
    /// `true` result consumes the host frame even when the continuation asks
    /// to remain scheduled for another module iteration.
    fn resume_after_pre_main_nmi(&mut self, input: u16, oam_dma_source: Option<&[u8]>) -> bool {
        let Some(resume) = self.game_execution_scheduler.take_pre_main_nmi_resume() else {
            return false;
        };

        let quadrant_build_or_upload_uses_published_oam = matches!(
            resume,
            PreMainNmiResume::DungeonSupertileQuadrantBuildReturn
                | PreMainNmiResume::DungeonSupertileQuadrantBuildPublishedReturn
                | PreMainNmiResume::DungeonSupertileQuadrantUploads
        ) && self.game_state.frame.main_module
            == 7
            && self.game_state.frame.submodule == 2
            && self.game_state.frame.subsubmodule == 5;
        let mut scanout = resume.scanout_generations();
        if quadrant_build_or_upload_uses_published_oam {
            // The leading NMI publishes the completed shadow DMA while the
            // room-quadrant build/upload caller remains suspended. Live PPU
            // OAM has already moved on, while the prior presented image is one
            // step older; select the newly published generation between those
            // two boundaries.
            if let Some(obj) = scanout.obj.as_mut() {
                obj.oam = OamScanoutSource::ComposePublishedShadowDma;
            }
        }
        self.next_display_vram_generation = scanout.vram;
        self.next_display_animated_bg_scanout_generation = scanout.animated_bg;
        self.next_display_bg_scroll_generation = scanout.bg_scroll;
        self.next_display_obj_scanout_generation = scanout.obj;
        if matches!(
            resume,
            PreMainNmiResume::DungeonSupertileQuadrantBuildReturn
                | PreMainNmiResume::DungeonSupertileQuadrantBuildPublishedReturn
                | PreMainNmiResume::DungeonSupertileQuadrantUploads
        ) {
            // The suspended quadrant caller returns with the software latch
            // still set, but the ROM reaches this leading hardware NMI before
            // the next module iteration can replace its upload request.
            self.clear_nmi_update_latch();
        }
        self.capture_display_snapshot_with_override(Some(scanout.publication));
        self.interrupt_nmi(input, oam_dma_source, false);
        self.replay_trace_col("before-game-loop");
        self.replay_trace_ram_watch("before-game-loop");
        self.zelda_run_game_loop();
        self.replay_trace_col("after-game-loop");
        self.replay_trace_ram_watch("after-game-loop");

        if resume == PreMainNmiResume::DungeonSupertileQuadrantBuildPublishedReturn {
            // The held state-6 OAM generation belongs only to the two return
            // scanouts above. Release it before the filtering continuation
            // captures state 8, whose room sprites use the live table again.
            self.next_display_obj_memory_generation =
                Some(DisplayObjGeneration::FollowModuleCadence);
        }

        if resume.continues_after_main(self.game_state.frame)
            && self.game_execution_scheduler.is_idle()
        {
            let next_resume = if resume == PreMainNmiResume::DungeonSupertileQuadrantBuildReturn {
                PreMainNmiResume::DungeonSupertileQuadrantBuildPublishedReturn
            } else {
                PreMainNmiResume::DungeonSupertileQuadrantUploads
            };
            self.game_execution_scheduler
                .schedule_pre_main_nmi_resume(next_resume);
        }
        if resume == PreMainNmiResume::OverworldAuxGraphicsReturn {
            // The main slice authors the following upload. Keep it private
            // until the next NMI, while retaining the hardware OAM that was
            // active when the transition began.
            self.next_display_vram_generation = DisplayVramGeneration::RetainCapturedBeforeNmi;
            self.next_display_bg_scroll_generation = DisplayBgScrollGeneration::ComposeLiveAfterNmi;
            let oam = self
                .display_snapshot
                .as_ref()
                .map(|snapshot| snapshot.ppu.oam.clone())
                .unwrap_or_else(|| self.ppu.oam.clone());
            let vram = self
                .display_snapshot
                .as_ref()
                .map(|snapshot| snapshot.ppu.vram[0x4000..0x4400].to_vec())
                .unwrap_or_else(|| self.ppu.vram[0x4000..0x4400].to_vec());
            self.active_display_obj_generation =
                DisplayObjGeneration::RetainCapturedMemory { oam, vram };
        }
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        true
    }

    /// `zelda_run_frame_internal`.
    ///
    /// The actual module routing, poly loop, and NMI handler are intentionally
    /// skeletal. Future ports should land behind this entry point so the
    /// lockstep oracle starts validating them immediately.
    pub fn run_frame_internal(&mut self, input: u16, run_what: u8) {
        self.sync_native_game_state_from_ram();
        self.link_obj_dma_completed_this_frame = false;
        self.active_display_force_blank_event = None;
        self.last_sprite_main_timing_workload = None;
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        self.replay_trace_col("run-frame-entry");
        self.replay_trace_ram_watch("run-frame-entry");
        if CaptureDisplayDiagnostics::from_env().frame_boundary {
            eprintln!(
                "frame_boundary_entry host={} main={:02x} sub={:02x} frame_counter={:02x} work={:?} caller={:?} dialogue_init={} dialogue_scroll={:?} bg1=({:04x},{:04x}) scroll_copy=({:04x},{:04x}) animated_bg=(source={:04x},countdown={:04x})",
                self.frame_ctr_dbg,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                self.game_state.frame.frame_counter,
                self.game_execution_scheduler.current_work(),
                self.game_execution_scheduler.pre_main_caller_continuation(),
                self.normal_dialogue_initialization_phase,
                self.dialogue_scroll_phase(),
                self.ppu.bg_layer[0].h_scroll,
                self.ppu.bg_layer[0].v_scroll,
                self.game_state.display.ppu_scroll_copy.bg1_h_copy(),
                self.game_state.display.ppu_scroll_copy.bg1_v_copy(),
                self.game_state.display.animated_tile_data_source_usize(),
                self.game_state.display.bg_tile_animation_countdown,
            );
        }
        if !self.initialized {
            self.zelda_initialize();
        }
        self.pre_nmi_animated_bg_scanout = (self.rom_startup_timing()
            && self.game_state.display.has_animated_tile_data_source())
        .then(|| {
            let destination_address = self
                .game_state
                .display
                .animated_tile_vram_destination_usize();
            (destination_address + 0x200 <= self.ppu.vram.len()).then(|| PreNmiAnimatedBgScanout {
                destination_address,
                vram: self.ppu.vram[destination_address..destination_address + 0x200].to_vec(),
            })
        })
        .flatten();
        self.pre_main_graphics_dma = if self.rom_startup_timing() {
            // Capture both the DMA operands and their hardware phase at the
            // host boundary. A main-thread module switch cannot retroactively
            // move a leading NMI behind the CPU work it already preceded.
            let animated_tile = self
                .game_state
                .display
                .has_animated_tile_data_source()
                .then(|| {
                    let source_address = self.game_state.display.animated_tile_data_source_usize();
                    let destination_address = self
                        .game_state
                        .display
                        .animated_tile_vram_destination_usize();
                    (source_address + 0x400 <= self.ram.len()
                        && destination_address + 0x200 <= self.ppu.vram.len())
                    .then(|| PreMainAnimatedTileDma {
                        source_address,
                        destination_address,
                        data: self.ram[source_address..source_address + 0x400].to_vec(),
                    })
                })
                .flatten();
            Some(PreMainGraphicsDma {
                entry_frame: self.game_state.frame,
                entry_plan: rom_graphics_dma_plan_at_host_boundary(self.game_state.frame),
                entry_dialogue_text_render_state: self
                    .game_state
                    .messaging
                    .runtime
                    .text_render_state(),
                entry_link_handler_state: self.game_state.player.follower_link.handler_state(),
                animated_tile,
                link_operands: PreMainLinkDmaOperands::capture(&self.ram),
                link_obj_vram: self.ppu.vram[0x4000..0x4400].to_vec(),
                oam_shadow: self.sprite_oam_shadow_buffer().to_vec(),
            })
        } else {
            None
        };
        // Reuse the OAM shadow captured with the other pre-main DMA operands so
        // NMI consumption and display publication cannot select adjacent copies.
        let oam_dma_source = self
            .pre_main_graphics_dma
            .as_ref()
            .map(|graphics| graphics.oam_shadow.clone());
        let frame = self.game_state.frame;
        if self.rom_startup_timing
            && rom_intro_poly_thread_is_active(frame.main_module, frame.submodule)
            && self.game_state.display.has_pending_polyhedral_update()
        {
            // The NMI at the frame boundary consumes the buffer completed in a
            // prior CPU slice. A buffer completed below remains pending until
            // the next boundary instead of being uploaded in the same frame.
            self.nmi_poly_upload_from_deferred = true;
            self.nmi_update_irqgfx();
        }
        // Oracle-guided scheduler experiment: allow a hot-reloaded rule to
        // place the vblank boundary before this CPU slice, including the
        // reset-delay slices that otherwise return before the usual NMI site.
        // This changes only scheduling, never PPU contents.
        if self.rom_startup_timing() && self.parity_runtime_nmi_rule_matches("pre_nmi") {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
        }
        if self.rom_startup_timing() && self.rom_reset_frame_delay != 0 {
            // Isolate the reset-to-main handoff as a first-class timing
            // boundary. A parity policy can run the boot initialization
            // without executing the first game-loop slice, which is distinct
            // from both a reset-delay frame and a normal frame.
            if self.parity_runtime_nmi_rule_matches("boot_init_only") {
                self.rom_reset_frame_delay = 0;
                self.zelda_initialization_code();
                self.capture_display_snapshot();
                return;
            }
            self.rom_reset_frame_delay = self.rom_reset_frame_delay.saturating_sub(1);
            self.capture_display_snapshot();
            return;
        }
        let initialized_audio_bank_this_frame =
            !self.game_state.display.has_animated_tile_data_source();
        if self.rom_startup_timing() && !self.audio_nmi_processed_before_main {
            // Live NMI samples/publishes the previous audio commands before the
            // main CPU performs this frame's game work. This is also true on the
            // first initialized frame: the intro chime written by `intro_init`
            // must wait for the following NMI rather than leaking out early.
            self.interrupt_nmi_audio_parts();
            self.audio_nmi_processed_before_main = true;
        }
        if initialized_audio_bank_this_frame {
            self.zelda_initialization_code();
        }
        if self.resume_pre_main_caller_continuation(input, oam_dma_source.as_deref()) {
            return;
        }
        if self.rom_startup_timing() && self.dialogue_scroll_is_return_only() {
            // The scroll copy and RenderText handler returned after the prior
            // frame's NMI. On this boundary the next NMI sees $12 still
            // latched, so it leaves $17/$0710 pending; only afterward does the
            // caller suffix reach Main_PrepSpritesForNmi and clear $12.
            // This measured return-only slice is distinct from both the 2/3
            // pixel copy slices and from a fresh module iteration.
            self.finish_dialogue_scroll_return();
            // The interrupted NMI cannot consume the pending dialogue upload,
            // but it still advances the ordinary vblank-owned presentation
            // state (animated BG tiles, Link DMA, and OAM). Capture after that
            // NMI so the scanout combines those updates with the still-pending
            // dialogue buffer, exactly as the hardware does.
            // BG scroll is a separate register generation: writes performed by
            // this NMI configure the active frame that follows vblank, even
            // though the pending dialogue-memory upload remains deferred.
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            self.capture_display_snapshot();
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
            // The completed text becomes visible at the next display
            // publication. Put it directly in the staged slot so the next
            // capture promotes it once after this text-buffer hold. Keep the
            // semantic glyph positions with the exact buffer they describe.
            let completed_scanout = self.dialogue_text_scanout_from_render_buffer();
            self.stage_dialogue_scroll_completion_after_return(completed_scanout);
            return;
        }
        if self.rom_startup_timing() {
            let startup_step = self.game_execution_scheduler.advance_startup_sequence();
            let consumes_frame = match startup_step {
                Some(StartupSequenceStep::FileSelectWaiting) => true,
                Some(StartupSequenceStep::CompleteFileSelectGraphics) => {
                    self.complete_module_select_file_0();
                    true
                }
                Some(StartupSequenceStep::ResumeFileSelectModule) | None => false,
                Some(StartupSequenceStep::SelectedGameLoadWaiting) => true,
                Some(StartupSequenceStep::BeginPreDungeonAudio) => {
                    self.begin_selected_game_load_pre_dungeon_audio();
                    true
                }
                Some(StartupSequenceStep::CompleteSelectedGameLoad) => {
                    self.complete_module05_load_file_after_resumption();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    true
                }
            };
            if consumes_frame {
                self.capture_display_snapshot();
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                return;
            }
        }
        if self.resume_dungeon_landing_wipe_return(input, oam_dma_source.as_deref()) {
            return;
        }
        if self.rom_startup_timing() && self.normal_dialogue_initialization_phase != 0 {
            let mut retained_dialogue_completion_oam = None;
            match self.normal_dialogue_initialization_phase {
                3..=5 => {
                    if self.normal_dialogue_initialization_phase == 5
                        && self.game_state.frame.main_module == 14
                    {
                        // Dungeon main authored the first dialogue OAM shadow
                        // after the entry scanout's DMA. The following vblank
                        // publishes that OAM-only domain even though the long
                        // text initializer keeps the rest of NMI_DoUpdates
                        // gated for its remaining CPU slices.
                        let oam_byte_len = self.ppu.oam.len() * 2;
                        let entry_shadow = self
                            .display_snapshot
                            .as_ref()
                            .and_then(|snapshot| {
                                snapshot
                                    .published_shadow_oam_dma
                                    .as_ref()
                                    .map(|oam| {
                                        oam.iter()
                                            .flat_map(|word| word.to_le_bytes())
                                            .collect::<Vec<_>>()
                                    })
                                    .or_else(|| {
                                        snapshot
                                            .ram
                                            .get(OAM_BUF..OAM_BUF + oam_byte_len)
                                            .map(Vec::from)
                                    })
                            })
                            .unwrap_or_else(|| self.sprite_oam_shadow_buffer().to_vec());
                        self.publish_dialogue_initialization_oam_dma(&entry_shadow);
                    }
                    self.normal_dialogue_initialization_phase -= 1;
                }
                2 => {
                    self.next_display_obj_memory_generation =
                        Some(DisplayObjGeneration::RetainCapturedOam {
                            oam: self.ppu.oam.clone(),
                        });
                    self.complete_text_initialization_prefix();
                    self.prepare_text_character_buffer_for_carry();
                    self.normal_dialogue_initialization_phase = 1;
                }
                1 => {
                    let retained_oam = self.ppu.oam.clone();
                    self.next_display_obj_memory_generation =
                        Some(DisplayObjGeneration::RetainCapturedOam {
                            oam: retained_oam.clone(),
                        });
                    retained_dialogue_completion_oam = Some(retained_oam);
                    self.complete_text_initialization_carry_suffix();
                    self.normal_dialogue_initialization_phase = 0;
                    self.complete_module0e_interface_after_run();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                _ => unreachable!(),
            }
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            if let Some(retained_oam) = retained_dialogue_completion_oam {
                // The initializer's return NMI completes its non-OAM work, but
                // the OAM shadow it just authored is not resident until the
                // following ordinary NMI. Keep the already-published dialogue
                // entry generation for this final held scanout and live PPU.
                self.ppu.oam.clone_from_slice(&retained_oam);
            }
            return;
        }
        if run_what & crate::RUN_POLY != 0 {
            self.zelda_run_poly_loop();
        }
        if self.intro_zelda_fade_transition_pending {
            self.complete_intro_zelda_fade_transition();
        }
        if self.intro_title_fade_suffix_pending {
            self.complete_intro_zelda_fade_suffix(true);
        }
        if self.intro_bg_fade_suffix_pending {
            self.complete_intro_bg_fade_suffix();
        }
        if self.rom_startup_timing() && self.intro_initialization_work_frames_pending != 0 {
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            self.intro_initialization_work_frames_pending -= 1;
            self.intro_initialization_reset_obj_control_pending = false;
            return;
        }
        if self.rom_startup_timing() && self.intro_memory_darken_frame_delay != 0 {
            let frame = &self.game_state.frame;
            if frame.main_module == 0 && frame.submodule == 3 {
                self.intro_animate_triforce();
            }
            self.intro_memory_darken_frame_delay =
                self.intro_memory_darken_frame_delay.saturating_sub(1);
            if self.intro_memory_darken_frame_delay == 0 {
                self.intro_initialize_memory_darken_finish();
            }
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        if self.rom_startup_timing()
            && rom_intro_poly_initialization_is_active(
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            )
            && self.intro_poly_thread_initialization_phase != 0
            && run_what & crate::RUN_MAIN != 0
        {
            let (begin_main_loop, complete_module, next_phase) =
                rom_intro_poly_init_decision(self.intro_poly_thread_initialization_phase);
            self.intro_poly_thread_initialization_phase = next_phase;
            if begin_main_loop {
                self.increment_frame_counter();
                self.clear_oam_buffer();
            }
            if complete_module {
                self.intro_initialize_triforce_poly_thread();
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
            }
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        if self.rom_startup_timing()
            && self.game_state.frame.main_module == 20
            && self.attract_init_graphics_phase != 0
        {
            let (complete_graphics, next_phase) =
                rom_attract_init_graphics_decision(self.attract_init_graphics_phase);
            self.attract_init_graphics_phase = next_phase;
            if complete_graphics {
                self.complete_attract_init_graphics();
            }
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        if self.rom_startup_timing()
            && self.game_state.frame.main_module == 20
            && self.attract_first_story_render_delay != 0
        {
            let opening_polka_dots = self.game_state.ending.attract_scene.sequence() == 0;
            if (opening_polka_dots && self.attract_first_story_render_delay == 7)
                || (!opening_polka_dots && self.attract_first_story_render_delay == 6)
            {
                self.increment_frame_counter();
                self.clear_oam_buffer();
                if opening_polka_dots {
                    self.ResetHUDPalettes4and5();
                }
            } else if opening_polka_dots && self.attract_first_story_render_delay == 3 {
                self.complete_text_initialization_state_prefix();
            } else if opening_polka_dots && self.attract_first_story_render_delay == 2 {
                self.Attract_DecompressStoryGFX();
                self.complete_text_initialization_suffix();
                self.attract_build_legend_image_tile_map(0);
                // The ROM resumes the first polka-dot story tick on this
                // GFX-completion boundary. Its visible text work is deferred
                // to the next slice, but the scene lifetime counter is not.
                self.attract_scene_mut().decrement_legend_ctr();
                self.clear_nmi_update_latch();
            } else if self.attract_first_story_render_delay == 1 {
                self.attract_enact_story();
                if !opening_polka_dots {
                    self.nmi_prepare_sprites();
                }
                self.clear_nmi_update_latch();
            }
            self.attract_first_story_render_delay =
                self.attract_first_story_render_delay.saturating_sub(1);
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        if self.rom_startup_timing() && self.dungeon_exit_spotlight_table_delay != 0 {
            self.dungeon_exit_spotlight_table_delay -= 1;
            // This is the prefix of a real main-loop iteration. The ROM has
            // ticked the frame counter and cleared OAM before vblank interrupts
            // the large-radius circle calculation; module routing resumes on a
            // later host frame without repeating that prefix.
            if !self.dungeon_exit_spotlight_resume_module {
                self.increment_frame_counter();
                self.clear_oam_buffer();
                self.latch_nmi_update();
                self.dungeon_exit_spotlight_resume_module = true;
            }
            self.capture_display_snapshot_with_publication(
                DisplaySnapshotPublication::AdvanceStaged,
            );
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        let mut resume_main_after_spotlight_return = false;
        let mut spotlight_scanout_started_before_resumed_main = None;
        let scheduled_work_entry_scroll = BgScrollRegisterScanout::capture(&self.ppu);
        let scheduled_work_step = if self.rom_startup_timing() {
            self.game_execution_scheduler.advance_work_one_nmi_slice()
        } else {
            None
        };
        if let Some(GameWorkStep::Complete(
            GameWorkContinuation::FinishSpiralStaircasePaletteFilter { tail },
        )) = scheduled_work_step
        {
            // The palette walk and caller tail return before this vblank, so
            // palette/tilemap publication is live. Core graphics DMA remains
            // gated across the return, however: its newly advanced animated
            // source does not reach VRAM until the next completed boundary.
            self.complete_dungeon_spiral_staircase_palette_filter(tail);
            self.complete_module07_dungeon_after_submodule();
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            self.set_core_update_disable_flag(1);
            if matches!(self.game_state.frame.subsubmodule, 12..=15) {
                self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                    PreMainNmiResume::DungeonSupertileQuadrantUploads,
                );
            }
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return;
        }
        if let Some(work_slice) = scheduled_work_step {
            if let GameWorkStep::Complete(continuation) = work_slice {
                let publication = continuation.completion_publication(scheduled_work_entry_scroll);
                if let Some(generation) = publication.bg_scroll {
                    self.next_display_bg_scroll_generation = generation;
                }
                if let Some(generation) = publication.obj {
                    self.next_display_obj_scanout_generation = Some(generation);
                }
            }
            let publication_override = match work_slice {
                GameWorkStep::Complete(GameWorkContinuation::FinishSpotlightIteration {
                    iteration,
                }) => Some(iteration.completion_publication()),
                // The interrupted entry build finishes mid-frame; its first
                // table generation belongs to the scanout already staged.
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonExitSpotlightEntry) => {
                    Some(DisplaySnapshotPublication::AdvanceStaged)
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishItemReceiptGraphics {
                    ..
                }) => Some(DisplaySnapshotPublication::RetainPublished),
                _ => self
                    .game_execution_scheduler
                    .in_flight_display_publication(),
            };
            let big_key_drop_graphics_slice = matches!(
                work_slice,
                GameWorkStep::Complete(GameWorkContinuation::FinishBigKeyDropGraphics { .. })
            ) || matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishBigKeyDropGraphics { .. })
            );
            if big_key_drop_graphics_slice {
                // Core DMA remains gated while the decompressor owns the main
                // stack, so OAM keeps the shadow published at entry. PPU
                // registers are still rewritten by every NMI and must be
                // captured afresh rather than retaining the entire snapshot.
                if let Some(oam) = self.last_presented_oam.clone() {
                    self.next_display_obj_memory_generation =
                        Some(DisplayObjGeneration::RetainCapturedOam { oam });
                }
                self.next_display_obj_scanout_generation = Some(ObjScanoutGenerations {
                    oam: OamScanoutSource::ComposePublishedShadowDma,
                    link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                });
            }
            match work_slice {
                GameWorkStep::Waiting => {}
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractWorldMap) => {
                    self.complete_attract_scene_world_map();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractWorldMapExit) => {
                    self.complete_attract_world_map_exit();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapLightLoad) => {
                    self.world_map_load_light_world_map();
                    // TransferMode7Characters returns to WorldMap_LoadLightWorldMap,
                    // then through Module0E_Interface and ZeldaRunGameLoop. The
                    // measured return frame therefore publishes sprite DMA state
                    // and releases the software NMI latch just like an ordinary
                    // completed module iteration.
                    self.complete_module0e_interface_after_run();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractThroneRoom) => {
                    self.complete_attract_scene_throne_room();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractZeldaPrison) => {
                    self.complete_attract_prep_zelda_prison();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractMaidenWarp) => {
                    self.complete_attract_prep_maiden_warp();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishAttractEndOfStory) => {
                    self.complete_attract_scene_end_of_story();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonFallingEntrance {
                    work,
                }) => {
                    // Both long Module 11 calls return just after the final
                    // interrupted NMI. Preserve that ordering: the scanout and
                    // NMI consume the in-flight generation first; only then
                    // does the caller suffix publish work for the next
                    // boundary. In particular, the room loader's HUD/CGRAM
                    // requests must remain pending for the following frame.
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    match work {
                        DungeonFallingEntranceWork::RoomAndTilesets => {
                            self.complete_module11_02_load_entrance();
                        }
                        DungeonFallingEntranceWork::SpriteGraphics => {
                            self.DungeonTransition_LoadSpriteGFX();
                        }
                    }
                    // Both calls return through Module11 and the ordinary game
                    // loop suffix after their final interrupted NMI slice.
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    return;
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonSupertileTransition { work },
                ) => {
                    match work {
                        DungeonSupertileTransitionWork::RoomLoad => {
                            // Dungeon_LoadRoom has returned before this NMI.
                            // Begin the next real call on the same suspended
                            // stack so its NMI request and graphics generation
                            // are visible at the correct boundary.
                            self.continue_module07_02_01_after_room_load();
                        }
                        DungeonSupertileTransitionWork::AuxiliarySpriteGraphics => {
                            self.complete_module07_02_01_after_auxiliary_sprite_graphics();
                            let scheduled = self.begin_dungeon_supertile_transition_work(
                                DungeonSupertileTransitionWork::RoomLoadCallerResume,
                            );
                            debug_assert!(scheduled);
                        }
                        DungeonSupertileTransitionWork::SpriteConversion => {
                            self.complete_dungeon_inter_room_transition_state3_after_sprite_conversion();
                            let scheduled = self.begin_dungeon_supertile_transition_work(
                                DungeonSupertileTransitionWork::SpriteConversionCallerResume,
                            );
                            debug_assert!(scheduled);
                        }
                        DungeonSupertileTransitionWork::QuadrantTilemapBuild => {
                            // The state-5 room-quadrant builder returns after
                            // the vblank that interrupted its tilemap loop.
                            // Only its caller suffix advances to state 6.
                            let resumes_state_6_after_leading_nmi =
                                self.game_state.frame.subsubmodule == 5
                                    && self.game_state.world.location.dungeon_room_index() == 0x72;
                            let resumes_state_11_before_next_vblank =
                                self.game_state.frame.subsubmodule == 10
                                    && self.game_state.world.location.dungeon_room_index() == 0x72;
                            self.complete_dungeon_inter_room_transition_not_dark_room();
                            if resumes_state_6_after_leading_nmi {
                                // Preserve the state-5 BG1 request until the
                                // next host's leading NMI. State 6 must not run
                                // first and replace it with the BG2 request.
                                self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                                    PreMainNmiResume::DungeonSupertileQuadrantBuildReturn,
                                );
                            }
                            if resumes_state_11_before_next_vblank {
                                // The state-10 BG1 quadrant DMA crosses the NMI
                                // boundary before the ROM begins state 11. The
                                // other NMI domains still publish at the
                                // trailing host boundary, so consume only this
                                // tilemap generation here. State 11 authors the
                                // following BG2 request into the same buffer.
                                self.nmi_upload_tilemap();

                                // The room-$72 return then reaches the next
                                // main-loop prefix and completes a full state-11
                                // iteration, including Link animation and the
                                // ordinary dungeon caller suffix. Its BG2
                                // request remains pending for the next NMI.
                                self.room_72_interrupted_main_prefix_oam_offset_active = true;
                                self.increment_frame_counter();
                                self.clear_oam_buffer();
                                self.Module07_02_SupertileTransition();
                                self.follower_link_state_mut()
                                    .decrement_link_dma_countdown();
                                self.complete_module07_dungeon_after_submodule();
                                self.nmi_prepare_sprites();
                                self.clear_nmi_update_latch();
                            }
                        }
                        DungeonSupertileTransitionWork::SpiralRoomInitialization => {
                            self.increment_subsubmodule();
                            let scheduled = self.begin_dungeon_supertile_transition_work(
                                DungeonSupertileTransitionWork::SpiralRoomCallerResume,
                            );
                            debug_assert!(scheduled);
                        }
                        DungeonSupertileTransitionWork::SpiralRoomCallerResume => {
                            self.complete_module07_dungeon_after_submodule();
                            self.nmi_prepare_sprites();
                            self.clear_nmi_update_latch();
                        }
                        DungeonSupertileTransitionWork::SpiralBgCharacters34 => {
                            self.increment_subsubmodule();
                            self.complete_module07_dungeon_after_submodule();
                            self.nmi_prepare_sprites();
                            self.clear_nmi_update_latch();
                        }
                        DungeonSupertileTransitionWork::SpiralSpriteGraphics => {
                            self.complete_dungeon_transition_load_sprite_gfx();
                            self.complete_module07_dungeon_after_submodule();
                            self.nmi_prepare_sprites();
                            self.clear_nmi_update_latch();
                            self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                                PreMainNmiResume::DungeonSupertileQuadrantUploads,
                            );
                        }
                        work @ (DungeonSupertileTransitionWork::RoomLoadCallerResume
                        | DungeonSupertileTransitionWork::SpriteConversionCallerResume) => {
                            // Snes9x returns the host call at the NMI that
                            // interrupted the long Module 7 stack. Resume the
                            // common sprite/HUD suffix on the following host
                            // call, just as the ROM resumes its interrupted
                            // caller. In particular, sprite initialization RNG
                            // belongs to this post-NMI generation.
                            self.complete_module07_dungeon_after_submodule();
                            self.capture_display_snapshot();
                            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                            if work == DungeonSupertileTransitionWork::RoomLoadCallerResume {
                                self.stage_live_animated_bg_scanout();
                            }
                            self.nmi_prepare_sprites();
                            self.clear_nmi_update_latch();
                            if work.next_module_resumes_after_pre_main_nmi() {
                                self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                                    PreMainNmiResume::DungeonSupertileQuadrantUploads,
                                );
                            }
                            return;
                        }
                    }
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonSupertileFilteringReturn,
                ) => {
                    // The palette/translucency call has advanced to state 8,
                    // but its caller returns through one vblank before the
                    // first scroll iteration. Publish that entry scanout and
                    // resume Module 7 on the following host frame.
                    let filtering_was_interrupted_before_return =
                        self.game_state.world.location.dungeon_room_index() == 0x72;
                    let pre_scroll_oam = if filtering_was_interrupted_before_return {
                        self.display_snapshot.as_ref().map(|display| {
                            let previously_presented_oam = self
                                .staged_presented_oam
                                .as_deref()
                                .or(self.last_presented_oam.as_deref());
                            dungeon_supertile_interrupted_filter_oam(
                                &display.ppu.oam,
                                previously_presented_oam,
                            )
                        })
                    } else {
                        self.display_snapshot.as_ref().map(|display| {
                            dungeon_supertile_pre_scroll_oam(
                                &display.ppu.oam,
                                display.published_shadow_oam_dma.as_deref(),
                            )
                        })
                    };
                    if filtering_was_interrupted_before_return {
                        self.set_subsubmodule(8);
                    }
                    self.capture_display_snapshot();
                    if let Some(oam) = pre_scroll_oam {
                        // The first state-8 shadow is the OAM buffer cleared by
                        // the resumed module prefix. The return scanout and the
                        // first scroll scanout both display this same pre-scroll
                        // image; later iterations use their ordinary prior
                        // published shadow.
                        let generation = DisplayObjGeneration::RetainCapturedOam { oam };
                        if let Some(display) = self.display_snapshot.as_mut() {
                            display.obj_generation = generation.clone();
                        }
                        if !filtering_was_interrupted_before_return {
                            self.next_display_obj_memory_generation = Some(generation);
                        }
                    }
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    // This is the vblank through which the filtering call
                    // returns. Its animated-BG DMA completes before the first
                    // state-8 scanout (the post-NMI pot/torch CHR is visible),
                    // even though the caller's first scroll iteration remains
                    // deferred to the next host frame.
                    self.stage_live_animated_bg_scanout();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    if filtering_was_interrupted_before_return {
                        // Room $72's filtering call was interrupted before its
                        // return, so one further host frame reaches only that
                        // caller suffix. State 8 is visible, but its first
                        // scroll iteration does not run until the next frame.
                        self.game_execution_scheduler.schedule_work(
                            GameWorkContinuation::HoldDungeonSupertileFilteringReturn,
                            1,
                        );
                    }
                    return;
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::HoldDungeonSupertileFilteringReturn,
                ) => {
                    let first_state8_oam = dungeon_supertile_first_state8_oam(&self.ppu.oam);
                    self.next_display_obj_memory_generation =
                        Some(DisplayObjGeneration::RetainCapturedOam {
                            oam: first_state8_oam.clone(),
                        });
                    self.next_display_obj_scanout_generation =
                        Some(dungeon_subtile_palette_filter_return_obj_scanout());
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    self.next_display_obj_memory_generation =
                        Some(DisplayObjGeneration::RetainCapturedOam {
                            oam: dungeon_supertile_second_state8_oam(&first_state8_oam),
                        });
                    self.next_display_obj_scanout_generation = Some(ObjScanoutGenerations {
                        oam: OamScanoutSource::RetainResidentPpuOam,
                        link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                        link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    });
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishPreDungeonEntranceLoad) => {
                    // The final sprite-reset slice is interrupted before
                    // Module_PreDungeon publishes module 7 and releases the
                    // main-loop NMI latch. Resume that caller suffix only
                    // after this scanout boundary.
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    self.module_pre_dungeon_after_audio_prefix();
                    if self.game_execution_scheduler.work_is_pending() {
                        return;
                    }
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishPreDungeonSongBankTransfer) => {
                    // The upload receiver was selected when the transfer
                    // began. The original caller remains suspended through
                    // this boundary, so finish its semantic suffix only after
                    // the copy returns.
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    self.complete_module_pre_dungeon_after_song_bank_transfer();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishItemReceiptGraphics {
                    continuation,
                }) => {
                    // The $21 item sheet returns through the ordinary module
                    // epilogue in v1.0.0: sprite preparation releases the NMI
                    // latch, then the common boundary below publishes the
                    // complete OAM image. Keep the measured retained path while
                    // Link is still in handler 21, but once that receipt has
                    // ended it would retain an older packed OAM size bit after
                    // the lower sprite entry has already advanced.
                    let ordinary_gfx_21_return = self
                        .item_receipt_graphics_return_uses_ordinary_module_epilogue(continuation);
                    if !ordinary_gfx_21_return {
                        // The final measured slice is the vblank that interrupts
                        // the decompressor itself. The software NMI latch is still
                        // set here, so hardware retains its resident OAM and Link
                        // tiles. Dungeon animated tiles are a separate DMA domain
                        // which still completes before this scanout is captured.
                        let graphics_dma_plan = rom_graphics_dma_plan(
                            self.game_state.frame.main_module,
                            self.game_state.frame.submodule,
                        );
                        self.nmi_core_animated_bg_update(graphics_dma_plan);
                        self.capture_display_snapshot_with_publication(
                            DisplaySnapshotPublication::RetainPublished,
                        );
                        self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    }
                    if let ItemReceiptGraphicsContinuation::ResumeUnclePassage {
                        receipt,
                        sprite_slot,
                        dungeon,
                    } = continuation
                    {
                        self.complete_ancilla_add_item_receipt(receipt);
                        self.complete_link_receive_item(receipt.item);
                        self.complete_uncle_passage_item_receipt(sprite_slot as usize);
                        self.complete_sprite_main_after_interrupted_slot(sprite_slot as usize);
                        self.complete_module07_after_sprite_main(dungeon);
                    }
                    // The decompressor has finally returned through
                    // Module_MainRouting. Only now can the ROM publish sprite
                    // DMA sources and release the software NMI latch.
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    if !ordinary_gfx_21_return {
                        self.stage_atomic_item_graphics_return_obj_scanout(continuation);
                        return;
                    }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishBigKeyDropGraphics {
                    sprite_slot,
                    dungeon,
                }) => {
                    let sprite_slot = usize::from(sprite_slot);
                    // Resume exactly where PrepareEnemyDrop was interrupted:
                    // finish the fixed big-key graphics call, its drop setup,
                    // the remaining lower sprite slots, and the Module 7
                    // sprite/camera suffix. The ordinary game-loop epilogue
                    // below then publishes the new OAM/CHR generation.
                    self.sprite_prep_big_key_load_graphics(sprite_slot);
                    self.complete_prepare_enemy_drop(sprite_slot);
                    self.complete_sprite_main_after_interrupted_slot(sprite_slot);
                    self.complete_module07_after_sprite_main(dungeon);
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishDungeonMapGraphicsPreparation,
                ) => {
                    // InitializeTilesets returns just after this interrupt. The
                    // active scanout is still the fully blank in-flight frame;
                    // only afterward does the dungeon-map caller publish its
                    // palettes/tilemaps and release the main-loop NMI latch.
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    self.complete_dungeon_map_graphics_preparation();
                    self.complete_module0e_interface_after_run();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonMapRoomDrawing) => {
                    // The second dungeon-map room plane is still being built
                    // when vblank interrupts the ROM. Preserve that scanout,
                    // then resume the routine and its Module0E caller suffix;
                    // room markers run on the next ordinary main iteration.
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    self.complete_dungeon_map_room_drawing();
                    self.complete_module0e_interface_after_run();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonMapRecovery) => {
                    // Tileset conversion and the room-quadrant rebuild return
                    // after this interrupt. The active scanout remains forced
                    // blank; publish the restored dungeon and audio state only
                    // through the resumed Module0E/game-loop suffix.
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    self.complete_dungeon_map_recovery();
                    self.complete_module0e_interface_after_run();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonSubtilePaletteFilter) => {
                    // ApplyPaletteFilter_bounce returns after the NMI that
                    // interrupted its color loop. Resume the caller's optional
                    // second filter pass only after that boundary instead of
                    // collapsing both passes into one host frame.
                    if self.game_state.display.palette_filter.countdown() != 0 {
                        self.ApplyPaletteFilter_bounce();
                    }
                    // The filter interrupted Module07's translated call stack.
                    // Resume its ordinary sprite/Link/HUD suffix before
                    // NMI_PrepareSprites so LinkOam_Main's new DMA pose is the
                    // source consumed by the ensuing NMI. OAM and Link CHR keep
                    // independent scanout generations across this boundary.
                    self.complete_module07_dungeon_after_submodule();
                    self.next_display_obj_scanout_generation = Some(ObjScanoutGenerations {
                        oam: OamScanoutSource::RetainResidentPpuOam,
                        link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                        link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    });
                    // This resumed caller suffix reaches the next NMI only
                    // after the scanout selected here. Keep the animated BG
                    // batch on the same completed host-boundary generation;
                    // publishing its newly prepared $3b00/$3c00 words here
                    // advances dungeon torches by one physical frame.
                    self.next_display_animated_bg_scanout_generation =
                        Some(AnimatedBgScanoutGeneration::HostBoundaryBeforeNmi);
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishSpiralStaircasePaletteFilter { .. },
                ) => {
                    unreachable!("spiral palette completion handled before generic publication")
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishDungeonExitSpotlightEntry) => {
                    // The vblank-interrupted first IrisSpotlight_ConfigureTable
                    // build finishes here: table copy, first radius write,
                    // submodule advance, and the Link/OAM suffix run, then the
                    // scheduled iteration return owns the boundary that reaches
                    // Main_PrepSpritesForNmi (oracle $0c00d trace: no prep on
                    // the resumed-build frame, one at the following return).
                    // The frame counter was ticked by the entry frame's main
                    // prefix; this resumed slice must not tick it again.
                    self.complete_dungeon_exit_spotlight_entry();
                    self.next_display_obj_scanout_generation =
                        Some(ObjScanoutGenerations::coherent(
                            GraphicsDmaGeneration::HostBoundaryBeforeMain,
                        ));
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishSpotlightIteration {
                    ..
                }) => {
                    // The opening or closing iris has returned through
                    // LinkOam_Main and the normal game-loop suffix only after
                    // this scanout has started. Its HDMA table can already be
                    // visible, but OAM and Link OBJ CHR still belong to the
                    // host-boundary generation; the NMI below publishes their
                    // newly prepared sources for the following scanout.
                    self.next_display_obj_scanout_generation =
                        Some(ObjScanoutGenerations::coherent(
                            GraphicsDmaGeneration::HostBoundaryBeforeMain,
                        ));
                    if self.iris_spotlight_goal_transition_pending {
                        self.iris_spotlight_goal_transition_pending = false;
                        self.complete_iris_spotlight_goal_transition();
                        // The ROM returns from IrisSpotlight_ConfigureTable to
                        // Spotlight_ConfigureTableAndControl, observes the
                        // freshly cleared submodule, and immediately runs
                        // OpenSpotlight_Next2. Keep that caller suffix on the
                        // same deferred return boundary as the goal transition.
                        self.OpenSpotlight_Next2();
                    }
                    let close_vertical_center = spotlight_vertical_center(
                        self.game_state.player.follower_link.y(),
                        self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
                    );
                    resume_main_after_spotlight_return = self.game_state.frame.main_module == 15
                        && rom_dungeon_exit_spotlight_radius_update_crosses_before_nmi(
                            self.game_state.display.spotlight_hdma.window_radius(),
                            close_vertical_center,
                        );
                    // The $3f->$38 crossing return resumes the next main slice
                    // before this frame's trailing NMI; that resumed slice
                    // reaches Main_PrepSpritesForNmi through the ordinary
                    // game-loop suffix. Prepping here as well double-advanced
                    // the animation countdowns (oracle $0c00d trace shows one
                    // write on the crossing frame, not two). Mid-close
                    // long-table iterations already prepped in their own main
                    // slice; their return boundary must not prep again.
                    let iteration_prepped_with_main = self.game_state.frame.main_module == 15
                        && rom_long_close_iteration_prep_returns_with_main(
                            self.game_state.display.spotlight_hdma.window_radius(),
                            close_vertical_center,
                        );
                    if !resume_main_after_spotlight_return && !iteration_prepped_with_main {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    if self.game_state.frame.main_module == 15
                        && rom_dungeon_exit_spotlight_table_needs_entry_slice(
                            self.game_state.display.spotlight_hdma.window_radius(),
                            close_vertical_center,
                        )
                    {
                        self.dungeon_exit_spotlight_table_delay =
                            DUNGEON_EXIT_SPOTLIGHT_INTER_ITERATION_HOLD_FRAMES;
                    }
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishPreOverworldProperties {
                    overworld_screen,
                    animated_tiles,
                }) => {
                    self.complete_pre_overworld_load_properties(overworld_screen, animated_tiles);
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishPreOverworldOverlays) => {
                    self.complete_pre_overworld_load_overlays();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishPreOverworldScreenBuild) => {
                    self.complete_pre_overworld_screen_build();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapExitTilesets) => {
                    // InitializeTilesets has returned through WorldMap_ExitMap
                    // and Module0E_Interface. Publish the same caller suffix
                    // that an uninterrupted game-loop iteration would reach.
                    self.complete_world_map_exit_after_tileset_load();
                    self.complete_module0e_interface_after_run();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapOverlayReload) => {
                    self.finish_overworld_load_overlays();
                    self.complete_module09_overworld_after_submodule();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishWorldMapAmbientMap8) => {
                    self.Overworld_LoadAmbientOverlay(false);
                    self.complete_module09_overworld_after_submodule();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishOverworldAuxGraphics) => {
                    // PC/V-counter traces remain in LoadTransAuxGFX and
                    // PrepTransAuxGfx through this frame's vblank, returning
                    // to Module09_LoadAuxGFX immediately afterward. Preserve
                    // that ordering: this scanout uses the pre-load display,
                    // while the completed graphics and caller suffix become
                    // CPU-visible before the next frame.
                    //
                    // The interrupt's scroll-register writes occur at this
                    // vblank and are visible in the scanout even though the
                    // decompressed VRAM generation remains the captured one.
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    self.complete_module09_load_aux_gfx();
                    self.complete_module09_overworld_after_submodule();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    self.game_execution_scheduler
                        .schedule_pre_main_nmi_resume(PreMainNmiResume::OverworldAuxGraphicsReturn);
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishOverworldMapQuadrants {
                    screen_map_and_sprite_gfx_tail_nmi_slices,
                }) => {
                    // SomeTileMapChange increments the submodule before the
                    // remaining screen-map build and sprite conversion return.
                    // Publish that CPU-visible generation after this vblank,
                    // then keep the caller stack suspended for its measured
                    // four-boundary tail.
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    self.complete_module09_load_new_map_quadrants();
                    self.game_execution_scheduler.schedule_work(
                        GameWorkContinuation::FinishOverworldScreenMapAndSpriteGraphicsTail,
                        screen_map_and_sprite_gfx_tail_nmi_slices,
                    );
                    return;
                }
                GameWorkStep::Complete(
                    GameWorkContinuation::FinishOverworldScreenMapAndSpriteGraphicsTail,
                ) => {
                    // The initial screen-map build and 3bpp-to-4bpp sprite
                    // conversion return after this frame's NMI. The caller
                    // suffix then publishes sprite DMA sources and releases
                    // the software NMI latch for the following boundary.
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    self.complete_module09_load_new_map_and_gfx_tail();
                    self.complete_module09_overworld_after_submodule();
                    // Although this caller suffix returns after NMI, Snes9x
                    // exposes its direct BG register writes in the scanout
                    // returned for this boundary. Publish only that register
                    // domain into the captured image: VRAM, OAM, CGRAM, and
                    // the remaining controls still belong to the pre-return
                    // generation.
                    let returned_scroll = self.bg_scroll_scanout_from_nmi_register_mirrors();
                    // These direct PPU writes occur after this scanout's NMI.
                    // They configure the following active frame; the immutable
                    // snapshot captured above retains the registers that were
                    // already being scanned out.
                    self.publish_bg_scroll_for_following_scanout(returned_scroll);
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    // The map/sprite graphics tail has now returned and
                    // rebuilt the same transition-entry sprite image. The
                    // snapshot above still owns the held hardware OAM for
                    // this scanout; subsequent frames resume normal cadence.
                    self.active_display_obj_generation = DisplayObjGeneration::FollowModuleCadence;
                    // The next ordinary Module09 iteration begins at the
                    // vblank edge immediately following this returned
                    // graphics tail. Carry that CPU phase explicitly into
                    // the sprite-loader timing decision instead of encoding
                    // the route or overworld screen number.
                    self.next_overworld_sprite_reload_entry_phase =
                        Some(OverworldSpriteReloadEntryPhase::VblankEdgeAfterGraphicsTail);
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::FinishOverworldSpriteReloadTail {
                    post_return_hold_nmi_slices,
                    return_phase: _,
                    epilogue_phase,
                    mut resume_scanout,
                }) => {
                    // The long sprite reset/load loop is interrupted in the
                    // ROM. Workload-derived return phase owns publication and
                    // epilogue order independently from the host hold count.
                    self.complete_module09_load_new_sprites_after_reload();
                    self.complete_module09_overworld_after_prepublished_rain();
                    // The provisional sprite generation may have advanced
                    // rain before the suspended ROM call stack reached it.
                    // Complete the typed scanout with the real transition
                    // return while preserving whichever BG1 generation the
                    // provisional phase recorded.
                    let returned_scroll = self.bg_scroll_scanout_from_nmi_register_mirrors();
                    resume_scanout = resume_scanout.complete_transition_return(returned_scroll);
                    if epilogue_phase == NmiPhase::BeforeNmi {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    // A V=213 return reaches this NMI before the full epilogue,
                    // so its camera mirrors are live in the emitted scanout.
                    // A return just after NMI retains the captured scroll.
                    // VRAM and OAM keep their independent generations.
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    if epilogue_phase == NmiPhase::AfterNmi {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    if post_return_hold_nmi_slices != 0 {
                        self.game_execution_scheduler.schedule_work(
                            GameWorkContinuation::HoldOverworldSpriteReloadReturn,
                            post_return_hold_nmi_slices,
                        );
                    } else {
                        self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                            PreMainNmiResume::OverworldSpriteReloadReturn {
                                scanout: resume_scanout,
                            },
                        );
                    }
                    return;
                }
                GameWorkStep::Complete(GameWorkContinuation::HoldOverworldSpriteReloadReturn) => {
                    // The light sprite loader returns at V=213, so its camera
                    // and caller suffix are already visible. Snes9x remains in
                    // submodule 5 for the following scanout, however; the next
                    // Overworld_StartScrollTransition call lands at V=255,
                    // after that image has been emitted. Hold only that next
                    // main-loop iteration while still running the frame NMI.
                    self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                        PreMainNmiResume::OverworldSpriteReloadReturn {
                            scanout: OverworldSpriteReloadResumeScanout::ByReturnPhase(
                                NmiPhase::BeforeNmi,
                            ),
                        },
                    );
                }
            }
            // The original ROM returns to the NMI boundary after the final
            // main-thread work slice. Attract loaders and item graphics both
            // publish only after their measured continuation completes.
            let item_receipt_graphics_slice = matches!(
                work_slice,
                GameWorkStep::Complete(GameWorkContinuation::FinishItemReceiptGraphics { .. })
            ) || matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishItemReceiptGraphics { .. })
            );
            if item_receipt_graphics_slice {
                // The main-loop latch holds OAM, Link OBJ, and ordinary tilemap
                // publication while the decompressor is interrupted. Dungeon
                // animated tiles are an independent NMI DMA domain and still
                // upload from the just-authored buffer before this scanout is
                // captured.
                let graphics_dma_plan = rom_graphics_dma_plan(
                    self.game_state.frame.main_module,
                    self.game_state.frame.submodule,
                );
                self.nmi_core_animated_bg_update(graphics_dma_plan);
            }
            self.capture_display_snapshot_with_override(publication_override);
            if let GameWorkStep::Complete(GameWorkContinuation::FinishSpotlightIteration {
                iteration,
            }) = work_slice
            {
                if iteration.projects_following_table_tail_on_completion() {
                    self.project_following_spotlight_tail_to_active_scanout(iteration.phase);
                }
            }
            if resume_main_after_spotlight_return {
                // The $3f->$38 circle update occurs before the trailing NMI,
                // but that main slice cannot replace the scanout which already
                // started at the return boundary above. Preserve the active
                // display generation while the ordinary capture below stages
                // the resumed CPU generation for the following scanout.
                spotlight_scanout_started_before_resumed_main = self.display_snapshot.clone();
            } else {
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                return;
            }
        }
        if self.rom_startup_timing()
            && run_what & crate::RUN_MAIN != 0
            && !self.dungeon_exit_spotlight_resume_module
        {
            if self.uncle_passage_item_receipt_starts_this_main_slice() {
                // ROM trace: Uncle_InPassage enters Link_ReceiveItem only
                // after the pending NMI has consumed the dialogue-clear
                // publication. The item decompressor then spans four further
                // vblanks with the main-loop latch set. Run this boundary
                // before the atomic port mutates sprite/OAM/palette state so
                // every display domain belongs to the same hardware
                // generation.
                self.clear_nmi_update_latch();
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                self.capture_display_snapshot();
                self.replay_trace_col("before-game-loop");
                self.replay_trace_ram_watch("before-game-loop");
                self.zelda_run_game_loop();
                self.replay_trace_col("after-game-loop");
                self.replay_trace_ram_watch("after-game-loop");
                debug_assert!(matches!(
                    self.game_execution_scheduler.current_work(),
                    Some(GameWorkContinuation::FinishItemReceiptGraphics { .. })
                ));
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return;
            }
            if self.dialogue_long_scroll_starts_this_frame() {
                // Snes9x enters this host slice with NMI pending, consumes the
                // prior RenderText publication, then starts the slow scroll
                // copy. The copy crosses the following vblank before
                // Main_PrepSpritesForNmi can run. Preserve that real ordering
                // instead of coalescing both boundaries after Module0E.
                self.finish_dialogue_character_render_call();
                // Ordinary host frames consumed the previous handler's NMI
                // publication at their trailing boundary. Re-open the exact
                // pre-main boundary here before consuming that carry so this
                // scroll starts from the ROM's $12 == 0 state.
                self.clear_nmi_update_latch();
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                // This host scanout includes the upload consumed by the
                // boundary above, while the CPU state returned below includes
                // the next upload authored by submodule 2. Capture between
                // those two generations.
                self.capture_display_snapshot();
                self.replay_trace_col("before-game-loop");
                self.replay_trace_ram_watch("before-game-loop");
                self.zelda_run_game_loop();
                self.replay_trace_col("after-game-loop");
                self.replay_trace_ram_watch("after-game-loop");
                debug_assert!(self.dialogue_scroll_is_copying_remaining_pixels());
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return;
            }
            if self.resume_after_pre_main_nmi(input, oam_dma_source.as_deref()) {
                return;
            }
            if self.game_execution_scheduler.is_idle()
                && rom_dungeon_supertile_scroll_runs_after_leading_nmi(
                    frame,
                    self.game_state.world.location.dungeon_room_index(),
                )
            {
                // The active image is emitted from the leading NMI. Capture
                // that completed PPU generation before the CPU authors the
                // following scroll step; there is no second NMI in this host
                // call.
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                self.capture_display_snapshot();
                self.zelda_run_game_loop();
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return;
            }
            self.nmi_read_joypads(input);
            self.joypad_sampled_before_main = true;
        }
        if run_what & crate::RUN_MAIN != 0 {
            self.replay_trace_col("before-game-loop");
            self.replay_trace_ram_watch("before-game-loop");
            self.zelda_run_game_loop();
            self.replay_trace_col("after-game-loop");
            self.replay_trace_ram_watch("after-game-loop");
        }
        if self.rom_startup_timing()
            && run_what & crate::RUN_MAIN != 0
            && self.game_execution_scheduler.is_idle()
            && rom_dungeon_supertile_filter_return_resumes_first_scroll_after_nmi(
                frame,
                self.game_state.frame,
                self.game_state.world.location.dungeon_room_index(),
            )
        {
            // State 7 returned before this vblank. Preserve the scanout it
            // selected (including the just-completed animated-BG DMA), then
            // resume the first state-8 main iteration without running a second
            // NMI in this host call. This keeps displayed scroll at the filter
            // boundary while live CPU state advances one scroll step, matching
            // the ROM's post-frame chronology.
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            self.zelda_run_game_loop();
            self.assert_native_frame_state_matches_ram();
            self.assert_native_world_location_state_matches_ram();
            self.assert_native_display_state_matches_ram();
            return;
        }
        if matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishItemReceiptGraphics { .. })
        ) {
            // The decompressor's entry slice has already set the software NMI
            // latch, but the vblank that interrupts it still completes the
            // independently wired graphics DMAs. Link consumes the operands
            // captured at the host boundary before the atomic caller entered
            // the decompressor, while dungeon animated BG follows the main
            // slice that advanced its source address and consumes live operands.
            let captured_link_operands = self
                .pre_main_graphics_dma
                .as_ref()
                .map(|graphics| graphics.link_operands);
            self.nmi_core_link_graphics_update(captured_link_operands);
            self.link_obj_dma_completed_this_frame = true;
            let mut graphics_dma_plan = rom_graphics_dma_plan(
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            );
            graphics_dma_plan.animated_bg_operands = GraphicsDmaGeneration::LiveAfterMain;
            self.nmi_core_animated_bg_update(graphics_dma_plan);
        }
        let dialogue_scroll_finished_copy =
            self.rom_startup_timing() && self.dialogue_scroll_is_return_only();
        let publication_override = self
            .game_execution_scheduler
            .in_flight_display_publication();
        // The circle builder is suspended across this vblank. Its WRAM table
        // is CPU-visible, but HDMA has already consumed the preceding staged
        // generation for the scanout that ends here.
        self.capture_display_snapshot_with_override(publication_override);
        if self.dialogue_scroll_is_completion_pending_publication() {
            // The early scroll return is complete before this NMI, but the
            // scanout captured immediately above still owns the preceding
            // published text. Stage the completed buffer only after that
            // boundary so the next capture promotes it exactly once.
            let completed_scanout = self.dialogue_text_scanout_from_render_buffer();
            self.stage_early_dialogue_scroll_completion(completed_scanout);
        }
        self.replay_trace_col("before-nmi");
        self.replay_trace_ram_watch("before-nmi");
        let defer_interface_exit_bg_upload = interface_exit_bg_upload_misses_current_scanout(
            frame.main_module,
            self.game_state.frame.main_module,
            self.game_state.display.has_bg_vram_load(),
        );
        self.interrupt_nmi(
            input,
            oam_dma_source.as_deref(),
            defer_interface_exit_bg_upload,
        );
        if dialogue_scroll_finished_copy {
            // The final copy slice reaches vblank before the RenderText caller
            // suffix. The ROM NMI always publishes $2123..$2132 even while
            // $12 keeps DMA work gated, and Snes9x exposes that color-composition
            // generation in this scanout. Keep BG scroll and display memory on
            // their independently measured generations.
            let color_math_scanout = self.color_math_scanout_from_nmi_register_mirrors();
            if let Some(snapshot) = self.display_snapshot.as_mut() {
                color_math_scanout.publish_to(&mut snapshot.ppu);
            }
        }
        if let Some(scanout) = spotlight_scanout_started_before_resumed_main {
            self.display_snapshot = Some(scanout);
        }
        self.replay_trace_col("after-nmi");
        self.replay_trace_ram_watch("after-nmi");
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        self.sync_overworld_map16_state_from_ram();
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

    fn emu_synchronize_whole_state(&mut self) {
        if let Some(sync_all) = self.emu_syncall {
            sync_all(self);
        }
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

    fn simple_hdma_get_ptr(&self, p: u32) -> Option<Vec<u8>> {
        Self::simple_hdma_get_ptr_from_ram(&self.ram, p)
    }

    fn simple_hdma_get_ptr_from_ram(ram: &[u8], p: u32) -> Option<Vec<u8>> {
        match p {
            0x0cfa87 => Some(ATTRACT_BG_DMA_SETUP.to_vec()),
            0x0cfa94 => Some(ATTRACT_TILEMAP_DMA_SETUP.to_vec()),
            0x0ebd53 => Some(ENDING_HDMA_SETUP.to_vec()),
            0x00f2fb => Some(SPOTLIGHT_INDIRECT_HDMA_SETUP.to_vec()),
            0x0abdcf => Some(MAP_MODE_HDMA_SETUP_NEAR.to_vec()),
            0x0abdd6 => Some(MAP_MODE_HDMA_SETUP_FAR.to_vec()),
            0x0abddd => Some(ATTRACT_INDIRECT_HDMA_SETUP.to_vec()),
            0x02c80c => Some(PRAYING_SCENE_HDMA_SETUP.to_vec()),
            0x001b00 => Some(Self::ram_bytes_from(ram, HDMA_TABLE_DYNAMIC, 0x1e0)),
            0x001be0 => Some(Self::ram_bytes_from(ram, HDMA_TABLE_DYNAMIC + 0xe0, 0x100)),
            0x001bf0 => Some(Self::ram_bytes_from(ram, HDMA_TABLE_DYNAMIC + 0xf0, 0xf0)),
            0x0add27 => Some(Self::u16_table_bytes(&MAP_MODE_PERSPECTIVE_ZOOMS_NEAR, 0)),
            0x0ade07 => Some(Self::u16_table_bytes(
                &MAP_MODE_PERSPECTIVE_ZOOMS_NEAR,
                0xe0,
            )),
            0x0adee7 => Some(Self::u16_table_bytes(&MAP_MODE_PERSPECTIVE_ZOOMS_FAR, 0)),
            0x0adfc7 => Some(Self::u16_table_bytes(&MAP_MODE_PERSPECTIVE_ZOOMS_FAR, 0xe0)),
            0x000600 => Some(Self::ram_bytes_from(ram, DEBUG_ROOM_BOUNDS_TOP, 2)),
            0x000602 => Some(Self::ram_bytes_from(ram, OVERWORLD_SCROLL_Y_END, 2)),
            0x000604 => Some(Self::ram_bytes_from(ram, OVERWORLD_SCROLL_X_START, 2)),
            0x000606 => Some(Self::ram_bytes_from(ram, OVERWORLD_SCROLL_X_END, 2)),
            0x0000e2 => Some(Self::ram_bytes_from(
                ram,
                PpuScrollCopyState::bg2_h_copy2_offset(),
                2,
            )),
            _ => None,
        }
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

    fn simple_hdma_init(&self, c: &mut SimpleHdma, dc: &DmaChannel) {
        Self::simple_hdma_init_from_ram(&self.ram, c, dc);
    }

    fn simple_hdma_init_from_ram(ram: &[u8], c: &mut SimpleHdma, dc: &DmaChannel) {
        if !dc.hdma_active {
            c.table = None;
            return;
        }
        c.table =
            Self::simple_hdma_get_ptr_from_ram(ram, dc.a_adr as u32 | ((dc.a_bank as u32) << 16));
        c.table_pos = 0;
        c.indir.clear();
        c.indir_pos = 0;
        c.rep_count = 0;
        c.mode = dc.mode | ((dc.indirect as u8) << 6);
        c.ppu_addr = dc.b_adr;
        c.indir_bank = dc.ind_bank;
    }

    fn simple_hdma_table_byte(c: &mut SimpleHdma) -> Option<u8> {
        let table = c.table.as_ref()?;
        let value = table.get(c.table_pos).copied()?;
        c.table_pos += 1;
        Some(value)
    }

    fn simple_hdma_line_writes(ram: &[u8], c: &mut SimpleHdma) -> ([(u32, u8); 4], usize) {
        let mut writes = [(0, 0); 4];
        if c.table.is_none() {
            return (writes, 0);
        }

        let mut do_transfer = false;
        if c.rep_count & 0x7f == 0 {
            let Some(rep_count) = Self::simple_hdma_table_byte(c) else {
                c.table = None;
                return (writes, 0);
            };
            c.rep_count = rep_count;
            if c.rep_count == 0 {
                c.table = None;
                return (writes, 0);
            }
            if c.mode & 0x40 != 0 {
                let Some(lo) = Self::simple_hdma_table_byte(c) else {
                    c.table = None;
                    return (writes, 0);
                };
                let Some(hi) = Self::simple_hdma_table_byte(c) else {
                    c.table = None;
                    return (writes, 0);
                };
                c.indir = Self::simple_hdma_get_ptr_from_ram(
                    ram,
                    ((c.indir_bank as u32) << 16) | lo as u32 | ((hi as u32) << 8),
                )
                .unwrap_or_default();
                c.indir_pos = 0;
            }
            do_transfer = true;
        }

        let mut write_count = 0;
        if do_transfer || c.rep_count & 0x80 != 0 {
            for j in 0..SIMPLE_HDMA_TRANSFER_LENGTH[(c.mode & 7) as usize] {
                let value = if c.mode & 0x40 != 0 {
                    let value = c.indir.get(c.indir_pos).copied().unwrap_or(0);
                    c.indir_pos += 1;
                    value
                } else {
                    Self::simple_hdma_table_byte(c).unwrap_or(0)
                };
                let offset = SIMPLE_HDMA_B_ADR_OFFSETS[(c.mode & 7) as usize][j];
                let adr = 0x2100 + c.ppu_addr.wrapping_add(offset) as u32;
                writes[write_count] = (adr, value);
                write_count += 1;
            }
        }
        c.rep_count = c.rep_count.wrapping_sub(1);
        (writes, write_count)
    }

    fn simple_hdma_do_line(&mut self, c: &mut SimpleHdma) {
        let (writes, write_count) = Self::simple_hdma_line_writes(&self.ram, c);
        for (address, value) in writes.into_iter().take(write_count) {
            self.zelda_ppu_write(address, value);
        }
    }

    fn hdma_channel_targets_window_latches(channel: &DmaChannel) -> bool {
        let mode = usize::from(channel.mode & 7);
        SIMPLE_HDMA_B_ADR_OFFSETS[mode][..SIMPLE_HDMA_TRANSFER_LENGTH[mode]]
            .iter()
            .any(|offset| matches!(channel.b_adr.wrapping_add(*offset), 0x26..=0x29))
    }

    fn hdma_state_targets_window_latches(enable_mask: u8, dma: &DmaState) -> bool {
        dma.channel.iter().enumerate().any(|(index, channel)| {
            enable_mask & (1 << index) != 0 && Self::hdma_channel_targets_window_latches(channel)
        })
    }

    fn final_window_latches_after_scanout(snapshot: &DisplaySnapshot) -> Option<[u8; 4]> {
        let captured_targets_window = Self::hdma_state_targets_window_latches(
            snapshot.ram[crate::game_state::constants::HDMAEN_COPY],
            &snapshot.dma,
        );
        let spotlight_targets_window = match &snapshot.spotlight_scanout_generation {
            SpotlightScanoutGeneration::ComposeLiveAfterNmi(live) => {
                let mut dma = snapshot.dma.clone();
                dma.channel[6..8].copy_from_slice(&live.dma_channels);
                Self::hdma_state_targets_window_latches(live.hdma_enable_mask, &dma)
            }
            SpotlightScanoutGeneration::CapturedBeforeNmi => false,
        };
        if !captured_targets_window && !spotlight_targets_window {
            return None;
        }

        let mut ram = snapshot.ram.clone();
        let mut dma = snapshot.dma.clone();
        snapshot
            .spotlight_scanout_generation
            .compose_hdma_into(&mut ram, &mut dma);
        snapshot.hdma_table_generation.compose_into(&mut ram);

        let enable_mask = ram[crate::game_state::constants::HDMAEN_COPY];
        let mut channels = dma.channel;
        let mut hdma: [SimpleHdma; 8] = std::array::from_fn(|_| SimpleHdma::default());
        let mut active = [false; 8];
        for index in 0..8 {
            channels[index].hdma_active = enable_mask & (1 << index) != 0;
            active[index] = channels[index].hdma_active
                && Self::hdma_channel_targets_window_latches(&channels[index]);
            if active[index] {
                Self::simple_hdma_init_from_ram(&ram, &mut hdma[index], &channels[index]);
            }
        }
        if !active.iter().any(|enabled| *enabled) {
            return None;
        }

        let mut latches = [
            snapshot.ppu.window1_left,
            snapshot.ppu.window1_right,
            snapshot.ppu.window2_left,
            snapshot.ppu.window2_right,
        ];
        // The scanline renderer performs the line-zero transfer before the
        // first visible row and leaves the transfer after row 223 in the PPU.
        for _ in 0..=224 {
            for index in 0..8 {
                if !active[index] {
                    continue;
                }
                let (writes, write_count) = Self::simple_hdma_line_writes(&ram, &mut hdma[index]);
                for (address, value) in writes.into_iter().take(write_count) {
                    if let 0x2126..=0x2129 = address {
                        latches[(address - 0x2126) as usize] = value;
                    }
                }
            }
        }
        Some(latches)
    }

    /// Retiring HDMA leaves its final register values in the physical PPU.
    ///
    /// The immutable display snapshot owns the table generation hardware just
    /// consumed. Replaying the live WRAM table here would be wrong because the
    /// CPU may already be authoring the following scanout.
    fn commit_retiring_display_window_latches(&mut self) {
        let latches = self
            .display_snapshot
            .as_ref()
            .and_then(|snapshot| Self::final_window_latches_after_scanout(snapshot));
        let Some(latches) = latches else {
            return;
        };
        for (offset, value) in latches.into_iter().enumerate() {
            self.zelda_ppu_write(0x2126 + offset as u32, value);
        }
    }

    /// Capture CGRAM after running all active HDMA channels for the first scanline.
    ///
    /// ALttP loads dungeon floor palette entries via HDMA per-scanline. The pre-render
    /// CGRAM is black for these entries because HDMA hasn't run yet. Running one HDMA
    /// line gives a CGRAM representative of the visible screen area.
    ///
    /// Runs all 8 HDMA channels (not just 6+7) because CGRAM writes can come from
    /// any channel depending on the room. Saves and restores all PPU state modified
    /// by HDMA so the actual render call (`zelda_draw_ppu_frame`) is unaffected.
    pub fn cgram_after_first_hdma_line(&mut self) -> Vec<u16> {
        let mut channels = self.dma.channel;
        for i in 0..8 {
            channels[i].hdma_active = self.game_state.display.is_hdma_channel_enabled(i);
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
        let saved_scroll_prev = self.ppu.scroll_prev;
        let saved_scroll_prev2 = self.ppu.scroll_prev2;
        let saved_bg_scrolls: [(u16, u16); 4] =
            std::array::from_fn(|i| (self.ppu.bg_layer[i].h_scroll, self.ppu.bg_layer[i].v_scroll));
        let saved_m7_matrix = self.ppu.m7_matrix;
        let saved_m7_prev = self.ppu.m7_prev;

        let mut hdma: [SimpleHdma; 8] = Default::default();
        for i in 0..8 {
            self.simple_hdma_init(&mut hdma[i], &channels[i]);
        }
        for i in 0..8 {
            self.simple_hdma_do_line(&mut hdma[i]);
        }

        let result = self.ppu.cgram.clone();

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
        self.ppu.scroll_prev = saved_scroll_prev;
        self.ppu.scroll_prev2 = saved_scroll_prev2;
        for (i, &(h_scroll, v_scroll)) in saved_bg_scrolls.iter().enumerate() {
            self.ppu.bg_layer[i].h_scroll = h_scroll;
            self.ppu.bg_layer[i].v_scroll = v_scroll;
        }
        self.ppu.m7_matrix = saved_m7_matrix;
        self.ppu.m7_prev = saved_m7_prev;

        result
    }

    /// Simulate all 8 HDMA channels for 224 scanlines and capture window 1
    /// left/right boundaries per scanline.
    ///
    /// Used by the GPU renderer to reconstruct the HDMA-driven spotlight oval.
    /// Saves and restores all PPU state so the actual render call is unaffected.
    /// Simulate 224 HDMA scanlines and capture per-scanline window boundaries and
    /// main-screen layer-enable register (TM / screen_enabled[0]).
    ///
    /// Returns `(window1_left, window1_right, window2_left, window2_right,
    /// screen_enabled_main, bg_h_scroll, bg_v_scroll, mode7_matrix)` per scanline.
    /// ALttP writes TM via HDMA to enable/disable layers (OBJ, BG3, etc.) on a
    /// per-scanline basis, and can update BG scroll during rendering; the GPU
    /// uses this to match the CPU's per-row rendering.
    ///
    /// PPU and DMA latches are restored after capture, but a one-shot V-counter
    /// IRQ is consumed just as it is by `zelda_draw_ppu_frame` and real hardware.
    pub fn ppu_scanline_windows(
        &mut self,
    ) -> Box<[(u8, u8, u8, u8, u8, [u16; 4], [u16; 4], [i16; 8], bool); 224]> {
        let saved_channels: [_; 8] = std::array::from_fn(|i| self.dma.channel[i]);
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
        let saved_scroll_prev = self.ppu.scroll_prev;
        let saved_scroll_prev2 = self.ppu.scroll_prev2;
        let saved_bg_scrolls: [(u16, u16); 4] =
            std::array::from_fn(|i| (self.ppu.bg_layer[i].h_scroll, self.ppu.bg_layer[i].v_scroll));
        let saved_m7_matrix = self.ppu.m7_matrix;
        let saved_m7_prev = self.ppu.m7_prev;

        let mut hdma_chans = [SimpleHdma::default(), SimpleHdma::default()];
        self.simple_hdma_init(&mut hdma_chans[0], &self.dma.channel[6]);
        self.simple_hdma_init(&mut hdma_chans[1], &self.dma.channel[7]);

        let mut result = Box::new(
            [(
                0u8, 0u8, 0u8, 0u8, 0u8, [0u16; 4], [0u16; 4], [0i16; 8], false,
            ); 224],
        );
        for line in 0..=224usize {
            if line == 128 && self.game_state.display.has_irq_control_flag() {
                let name_scroll_x = self.game_state.messaging.select_file_menu.name_scroll_x();
                self.zelda_ppu_write(0x2111, name_scroll_x as u8);
                self.zelda_ppu_write(0x2111, (name_scroll_x >> 8) as u8);
                self.zelda_ppu_write(0x2112, 0);
                self.zelda_ppu_write(0x2112, 0);
            }

            if (1..=224).contains(&line) {
                result[line - 1] = (
                    self.ppu.window1_left,
                    self.ppu.window1_right,
                    self.ppu.window2_left,
                    self.ppu.window2_right,
                    self.ppu.screen_enabled[0],
                    std::array::from_fn(|i| self.ppu.bg_layer[i].h_scroll),
                    std::array::from_fn(|i| self.ppu.bg_layer[i].v_scroll),
                    self.ppu.m7_matrix,
                    line - 1 < usize::from(self.ppu.forced_blank_scanlines)
                        || self
                            .ppu
                            .forced_blank_from_scanline
                            .is_some_and(|start| line - 1 >= usize::from(start))
                        || (self.ppu.forced_blank
                            && self.ppu.forced_blank_from_scanline.is_none()
                            && self.ppu.forced_blank_scanlines == 0),
                );
            }

            self.simple_hdma_do_line(&mut hdma_chans[0]);
            self.simple_hdma_do_line(&mut hdma_chans[1]);
        }

        self.dma.channel = saved_channels;
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
        self.ppu.scroll_prev = saved_scroll_prev;
        self.ppu.scroll_prev2 = saved_scroll_prev2;
        for (i, &(h_scroll, v_scroll)) in saved_bg_scrolls.iter().enumerate() {
            self.ppu.bg_layer[i].h_scroll = h_scroll;
            self.ppu.bg_layer[i].v_scroll = v_scroll;
        }
        self.ppu.m7_matrix = saved_m7_matrix;
        self.ppu.m7_prev = saved_m7_prev;

        result
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

    /// Simulate 224 HDMA scanlines and capture a full CGRAM snapshot per scanline.
    pub fn ppu_scanline_cgram(&mut self) -> Vec<Vec<u16>> {
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

        let mut result = Vec::with_capacity(224);
        for _ in 0..224 {
            for i in 0..8 {
                self.simple_hdma_do_line(&mut hdma[i]);
            }
            result.push(self.ppu.cgram.clone());
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

    fn selected_intro_poly_display_buffer(&self) -> Vec<u16> {
        if env::var_os("ZELDA3_INTRO_POLY_PRESENT_OBJ_LATCH").is_some() {
            if let Some(latched_vram) = self.ppu.obj_vram_latch.as_deref() {
                if latched_vram.len() >= 0x5c00 {
                    return latched_vram[0x5800..0x5c00].to_vec();
                }
            }
        }
        let diagnostic_lag = env::var("ZELDA3_INTRO_POLY_PRESENT_HISTORY_LAG")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        if diagnostic_lag != 0 {
            if let Some((_, vram, _)) = self
                .intro_poly_vram_history
                .len()
                .checked_sub(1 + diagnostic_lag)
                .and_then(|index| self.intro_poly_vram_history.get(index))
            {
                return vram.clone();
            }
        }
        self.ppu.vram[0x5800..0x5c00].to_vec()
    }

    fn set_mode7_perspective_correction(&mut self, low: u16, high: u16) {
        self.ppu.mode7_perspective_low = if low != 0 { 1.0 / low as f32 } else { 0.0 };
        self.ppu.mode7_perspective_high = if high != 0 { 1.0 / high as f32 } else { 0.0 };
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

    pub fn save_func(ctx_in: &mut ByteArray, data: &mut [u8]) {
        ByteArray_AppendData(ctx_in, data);
    }

    pub fn load_func(ctx: &mut LoadFuncState<'_>, data: &mut [u8]) {
        debug_assert!(ctx.remaining() >= data.len());
        let end = ctx.pos + data.len();
        data.copy_from_slice(&ctx.p[ctx.pos..end]);
        ctx.pos = end;
    }

    fn save_load_call(func: &mut SaveLoadFunc<'_, '_>, data: &mut [u8]) {
        match func {
            SaveLoadFunc::Save(ctx) => Self::save_func(ctx, data),
            SaveLoadFunc::Load(ctx) => Self::load_func(ctx, data),
        }
    }

    fn internal_save_load(&mut self, func: &mut SaveLoadFunc<'_, '_>) {
        let mut junk = [0u8; 58];
        Self::save_load_call(func, &mut junk[..27]);

        let mut apu_ram = if matches!(func, SaveLoadFunc::Save(_)) {
            self.save_audio_apu_ram_c_saveload().to_vec()
        } else {
            vec![0; APU_RAM_SAVELOAD_SIZE]
        };
        Self::save_load_call(func, &mut apu_ram);
        if matches!(func, SaveLoadFunc::Load(_)) {
            self.load_audio_apu_ram_c_saveload(&apu_ram);
        }

        let mut junk40 = [0u8; 40];
        Self::save_load_call(func, &mut junk40);

        let mut dsp = if matches!(func, SaveLoadFunc::Save(_)) {
            self.save_audio_dsp_c_saveload()
        } else {
            vec![0; DSP_SAVELOAD_SIZE]
        };
        Self::save_load_call(func, &mut dsp);
        if matches!(func, SaveLoadFunc::Load(_)) {
            self.load_audio_dsp_c_saveload(&dsp)
                .expect("invalid DSP saveload block");
        }

        let mut junk15 = [0u8; 15];
        Self::save_load_call(func, &mut junk15);

        let mut dma_slot = if matches!(func, SaveLoadFunc::Save(_)) {
            self.dma.save_c_saveload()
        } else {
            vec![0; DMA_SAVELOAD_SLOT_SIZE]
        };
        Self::save_load_call(func, &mut dma_slot);
        if matches!(func, SaveLoadFunc::Load(_)) {
            self.dma
                .load_c_saveload(&dma_slot)
                .expect("invalid DMA saveload block");
        }

        let mut ppu_slot = if matches!(func, SaveLoadFunc::Save(_)) {
            self.ppu.save_c_saveload()
        } else {
            vec![0; PPU_SAVELOAD_SLOT_SIZE]
        };
        Self::save_load_call(func, &mut ppu_slot);
        if matches!(func, SaveLoadFunc::Load(_)) {
            self.ppu
                .load_c_saveload(&ppu_slot)
                .expect("invalid PPU saveload block");
        }

        Self::save_load_call(func, &mut self.sram);

        Self::save_load_call(func, &mut junk);
        Self::save_load_call(func, &mut self.ram);

        let mut junk4 = [0u8; 4];
        Self::save_load_call(func, &mut junk4);
    }

    fn load_snes_state(&mut self, func: &mut SaveLoadFunc<'_, '_>) {
        self.internal_save_load(func);
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

    /// Rebuild the palette-provenance mirror from the restored palette shadow
    /// after a full-state snapshot load. Each word is tagged `Copied` with its
    /// restored value, so the mirror equals the shadow (coherence check clean,
    /// no `Unknown`) and the renderer's mirror substitution stays valid. Only
    /// used at snapshot-restore boundaries, where the shadow is the authority.
    fn reconstitute_palette_mirror_from_shadow(&mut self) {
        use zelda3_palette::{Bank, MirrorWord, SourceTag, PALETTE_WORDS};
        // The committed CGRAM image the renderer consumes tracks what the PPU holds, which the
        // restore bulk-loads directly; capture it before borrowing the mirror.
        let cgram: Vec<u16> = self.ppu.cgram.to_vec();
        let pb = &self.game_state.display.palette_buffer;
        let main: Vec<u16> = (0..PALETTE_WORDS).map(|i| pb.main_color(i)).collect();
        let aux: Vec<u16> = (0..PALETTE_WORDS).map(|i| pb.aux_color(i)).collect();
        let backup = pb.overworld_palette_backup().to_vec();
        let mirror = &mut self.game_state.display.palette_provenance.0;
        for i in 0..PALETTE_WORDS {
            mirror.bank_mut(Bank::Main)[i] = MirrorWord::Known(main[i], SourceTag::Copied);
            mirror.bank_mut(Bank::Aux)[i] = MirrorWord::Known(aux[i], SourceTag::Copied);
            let off = i * 2;
            let bv = u16::from(backup.get(off).copied().unwrap_or(0))
                | (u16::from(backup.get(off + 1).copied().unwrap_or(0)) << 8);
            mirror.bank_mut(Bank::Backup)[i] = MirrorWord::Known(bv, SourceTag::Copied);
        }
        // A restore also bulk-loads ppu.cgram; the committed CGRAM image is otherwise only
        // refreshed at upload commits (which may not run for many frames during a fade), so
        // reconstitute it here too or the renderer substitutes a stale palette.
        mirror.reconstitute_cgram(&cgram);
    }

    fn save_snes_state(&mut self, func: &mut SaveLoadFunc<'_, '_>) {
        self.backup_spotlight_hdma_to_saveload_buffer();
        self.zelda_save_music_state_to_ram_locked();
        self.internal_save_load(func);
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

    pub fn state_recorder_read_next_replay_state(&mut self, sr: &mut StateRecorder) -> u16 {
        assert!(sr.replay_mode);
        while sr.frames_since_last >= sr.replay_next_cmd_at {
            let mut replay_pos = sr.replay_pos as usize;
            if replay_pos != sr.replay_pos_last_complete as usize {
                sr.frames_since_last = 0;
                if sr.replay_cmd < 0xc0 {
                    sr.last_inputs ^= 1 << (sr.replay_cmd >> 4);
                } else if sr.replay_cmd < 0xd0 {
                    let mut nb = 1usize + ((sr.replay_cmd >> 2) & 3) as usize;
                    if nb == 4 {
                        loop {
                            let t = sr.log.data[replay_pos];
                            replay_pos += 1;
                            nb += t as usize;
                            if t != 255 {
                                break;
                            }
                        }
                    }
                    let mut addr = (((sr.replay_cmd >> 1) & 1) as u32) << 16;
                    addr |= (sr.log.data[replay_pos] as u32) << 8;
                    replay_pos += 1;
                    addr |= sr.log.data[replay_pos] as u32;
                    replay_pos += 1;
                    while nb != 0 {
                        let offset = (addr & 0x1ffff) as usize;
                        self.set_compatibility_ram_byte(offset, sr.log.data[replay_pos]);
                        replay_pos += 1;
                        self.emu_sync_memory_region(offset, 1);
                        addr = addr.wrapping_add(1);
                        nb -= 1;
                    }
                } else if sr.replay_cmd < 0xe0 {
                    let snapshot_size =
                        Self::state_recorder_read_vl(&sr.log.data, &mut replay_pos) as usize;
                    assert!(snapshot_size <= sr.log.size().saturating_sub(replay_pos));
                    let snapshot_end = replay_pos + snapshot_size;
                    let mut state = LoadFuncState::new(&sr.log.data[replay_pos..snapshot_end]);
                    let mut load = SaveLoadFunc::Load(&mut state);
                    self.load_snes_state(&mut load);
                    assert_eq!(state.remaining(), 0);
                    replay_pos = snapshot_end;
                    sr.last_inputs = 0;
                } else {
                    panic!("unknown replay command {:02x}", sr.replay_cmd);
                }
            }
            sr.replay_pos_last_complete = replay_pos as u32;
            if replay_pos >= sr.log.size() {
                sr.replay_pos = replay_pos as u32;
                sr.replay_next_cmd_at = u32::MAX;
                break;
            }

            let cmd = sr.log.data[replay_pos];
            replay_pos += 1;
            let mask = if cmd < 0xc0 { 0xf } else { 0x1 };
            let mut frames = (cmd & mask) as u32;
            if frames == mask as u32 {
                loop {
                    let t = sr.log.data[replay_pos];
                    replay_pos += 1;
                    frames += t as u32;
                    if t != 255 {
                        break;
                    }
                }
            }
            sr.replay_next_cmd_at = frames;
            sr.replay_cmd = cmd;
            sr.replay_pos = replay_pos as u32;
        }
        sr.frames_since_last = sr.frames_since_last.wrapping_add(1);
        sr.replay_frame_counter = sr.replay_frame_counter.wrapping_add(1);
        if sr.replay_frame_counter >= sr.total_frames {
            sr.replay_mode = false;
        }
        sr.last_inputs
    }

    pub fn state_recorder_stop_replay(sr: &mut StateRecorder) {
        if !sr.replay_mode {
            return;
        }
        sr.replay_mode = false;
        sr.total_frames = sr.replay_frame_counter;
        sr.log.data.truncate(sr.replay_pos_last_complete as usize);
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

    pub fn state_recorder_save<W: Write>(&mut self, sr: &mut StateRecorder, f: &mut W) {
        let mut arr = ByteArray::default();
        let mut save = SaveLoadFunc::Save(&mut arr);
        self.save_snes_state(&mut save);
        assert!(sr.base_snapshot.data.is_empty() || sr.base_snapshot.size() == arr.size());

        let mut hdr = [0u32; 8];
        hdr[0] = 1;
        hdr[1] = sr.total_frames;
        hdr[2] = sr.log.size() as u32;
        hdr[3] = sr.last_inputs as u32;
        hdr[4] = sr.frames_since_last;
        hdr[5] = if sr.base_snapshot.size() != 0 { 1 } else { 0 };
        hdr[6] = arr.size() as u32;
        if sr.replay_mode {
            hdr[5] |= sr.replay_pos_last_complete << 1;
            hdr[7] = sr.replay_frame_counter;
        }
        for value in hdr {
            f.write_all(&value.to_le_bytes()).expect("fwrite failed");
        }
        f.write_all(&sr.log.data).expect("fwrite failed");
        f.write_all(&sr.base_snapshot.data).expect("fwrite failed");
        f.write_all(&arr.data).expect("fwrite failed");
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
        let inputs = inputs as u16;
        let mut directions = inputs & 0x00f0;
        if directions & 0x0030 == 0x0030 {
            directions &= !0x0010;
        }
        if directions & 0x00c0 == 0x00c0 {
            directions &= !0x0040;
        }
        (inputs & !0x00f0) | directions
    }

    pub fn state_recorder_read_next_replay_state_with_input_override(
        &mut self,
        sr: &mut StateRecorder,
        input_override: Option<u16>,
    ) -> u16 {
        let replay_input = self.state_recorder_read_next_replay_state(sr);
        input_override.unwrap_or(replay_input)
    }

    pub fn zelda_run_frame(&mut self, inputs: i32) -> bool {
        self.zelda_run_frame_with_replay_input_override(inputs, None)
    }

    pub fn zelda_run_frame_with_replay_input_override(
        &mut self,
        inputs: i32,
        replay_input_override: Option<u16>,
    ) -> bool {
        self.rom_random_replay.begin_frame();
        let raw_inputs = inputs as u16;
        let raw_replay_input_override = replay_input_override;
        let inputs = Self::sanitize_frame_inputs(inputs);
        let replay_input_override =
            replay_input_override.map(|input| Self::sanitize_frame_inputs(input as i32));
        self.frame_ctr_dbg = self.frame_ctr_dbg.wrapping_add(1);
        self.replay_trace_ram_watch("frame-entry");
        let mut state_recorder = std::mem::take(&mut self.state_recorder);
        let is_replay = state_recorder.replay_mode;
        let input_state = if is_replay {
            let input_state = self.state_recorder_read_next_replay_state_with_input_override(
                &mut state_recorder,
                replay_input_override,
            );
            self.replay_trace_col("after-replay-command");
            self.replay_trace_ram_watch("after-replay-command");
            input_state
        } else {
            Self::state_recorder_record(&mut state_recorder, inputs);
            let apui00 = self.zelda_is_music_playing() as u8;
            if apui00 != self.game_state.system_signals.apui00() {
                self.set_apui00(apui00);
                let apui00_offset = SystemSignalsState::apui00_offset();
                self.emu_sync_memory_region(apui00_offset, 1);
                Self::state_recorder_record_patch_byte(
                    &mut state_recorder,
                    apui00_offset as u32,
                    &[apui00],
                    1,
                );
            }
            if self.game_state.display.has_animated_tile_data_source() {
                if self.game_state.system_signals.bugs_fixed() < BUGFIX_LATEST {
                    if !self.rom_startup_timing {
                        self.set_bugs_fixed(BUGFIX_LATEST);
                        self.emu_sync_memory_region(RAM_BUGS_FIXED, 1);
                        Self::state_recorder_record_patch_byte(
                            &mut state_recorder,
                            RAM_BUGS_FIXED as u32,
                            &[BUGFIX_LATEST],
                            1,
                        );
                    }
                }
                let enhanced_features0 = self.game_state.enhanced_features.bits();
                let wanted_zelda_features = self.wanted_zelda_features;
                if enhanced_features0 != wanted_zelda_features {
                    self.enhanced_features_mut().set_bits(wanted_zelda_features);
                    self.emu_sync_memory_region(ENHANCED_FEATURES0, 4);
                    Self::state_recorder_record_patch_byte(
                        &mut state_recorder,
                        ENHANCED_FEATURES0 as u32,
                        &wanted_zelda_features.to_le_bytes(),
                        4,
                    );
                }
            }
            inputs
        };
        self.previous_host_controller_input = if is_replay {
            raw_replay_input_override.unwrap_or(input_state)
        } else {
            raw_inputs
        };
        self.state_recorder = state_recorder;

        self.sync_native_game_state_from_ram();
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        let frame = &self.game_state.frame;
        let use_timed_poly_worker = self.rom_startup_timing
            && rom_intro_poly_thread_is_active(frame.main_module, frame.submodule);
        let title_fade_poly_thread =
            self.rom_startup_timing && frame.main_module == 0 && frame.submodule == 5;
        let bg_fade_poly_thread = self.rom_startup_timing
            && frame.main_module == 0
            && frame.submodule == 7
            && self.game_state.intro_sword.anim_step_raw() == 2;
        let wait_player_poly_teardown = self.rom_startup_timing
            && rom_intro_wait_player_tears_down_poly_thread(
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                self.game_state.display.nmi_thread_active,
            );
        self.snes9x_hold_intro_step_this_frame = false;
        let poly_thread_teardown_frame = self.intro_poly_thread_teardown_pending;
        self.intro_poly_thread_teardown_pending = false;
        let run_what = if poly_thread_teardown_frame {
            0
        } else if wait_player_poly_teardown {
            3
        } else if title_fade_poly_thread {
            let poly_phase = self.intro_title_fade_poly_phase;
            let run_main = rom_intro_title_fade_runs_main(poly_phase);
            self.intro_title_fade_defer_suffix_this_frame =
                rom_intro_title_fade_should_yield_suffix(poly_phase);
            self.intro_title_fade_poly_phase = (self.intro_title_fade_poly_phase + 1) % 3;
            if run_main {
                3
            } else {
                2
            }
        } else if bg_fade_poly_thread {
            let (run_main, yield_before_suffix, carry_frames, poly_phase) =
                rom_intro_bg_fade_main_decision(
                    self.intro_bg_fade_carry_frames,
                    self.intro_bg_fade_poly_phase,
                );
            self.intro_bg_fade_defer_suffix_this_frame = yield_before_suffix;
            self.intro_bg_fade_carry_frames = carry_frames;
            self.intro_bg_fade_poly_phase = poly_phase;
            if run_main {
                3
            } else {
                2
            }
        } else if rom_file_select_teardown_runs_with_outgoing_poly_worker(
            frame.main_module,
            frame.submodule,
            self.game_state.display.nmi_thread_active,
            self.game_state.display.nmi_thread_uses_poly_stack(),
        ) {
            3
        } else if legacy_poly_scheduler_is_active(
            self.game_state.system_signals.bugs_fixed(),
            use_timed_poly_worker,
            self.game_state.display.nmi_thread_active,
        ) {
            if self.game_state.display.nmi_thread_uses_poly_stack() {
                2
            } else {
                1
            }
        } else {
            let virq = self.game_state.display.vertical_irq_trigger;
            let carry = if self.game_state.display.nmi_thread_active {
                if use_timed_poly_worker {
                    self.game_state.ending.attract_scene.intro_did_run_step() != 0
                } else {
                    let carry = self.advance_crystal_rotation_counter(virq);
                    self.emu_sync_memory_region(CRYSTAL_ROTATION_COUNTER, 1);
                    carry
                }
            } else {
                false
            };
            if carry {
                3
            } else {
                1
            }
        };
        if !title_fade_poly_thread {
            self.intro_title_fade_poly_phase = 0;
            self.intro_title_fade_defer_suffix_this_frame = false;
        }
        if !bg_fade_poly_thread {
            self.intro_bg_fade_carry_frames = 0;
            self.intro_bg_fade_poly_phase = 0;
            self.intro_bg_fade_defer_suffix_this_frame = false;
        }
        if self.emu_runframe.is_none()
            || self.game_state.enhanced_features.bits() != 0
            || self.dialogue_flags != 0
        {
            crate::types::ww_set_cur_frame(self.frame_ctr_dbg);
            self.replay_trace_ram_watch("before-run-frame-internal");
            self.zelda_run_frame_internal(input_state, run_what as u8);
            self.replay_trace_ram_watch("after-run-frame-internal");
        } else if let Some(func) = self.emu_runframe {
            func(self, input_state, run_what);
        }
        self.zelda_push_apu_state();
        self.replay_trace_ram_watch("after-apu");
        is_replay
    }

    pub fn install_rom_random_replay(
        &mut self,
        samples: Vec<crate::RomRandomSample>,
        start_execution_frame: u32,
    ) {
        self.rom_random_replay
            .install(samples, start_execution_frame);
    }

    pub fn finish_rom_random_replay(&self) -> Result<(), String> {
        self.rom_random_replay.finish()
    }

    pub fn finish_rom_random_replay_through(&self, end_execution_frame: u32) -> Result<(), String> {
        self.rom_random_replay.finish_through(end_execution_frame)
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

    pub fn save_load_slot(&mut self, cmd: SaveLoadCommand, which: i32) {
        if let Some(path) = Self::save_slot_path(cmd, which) {
            if cmd == SaveLoadCommand::Save {
                if let Ok(mut file) = fs::File::create(path) {
                    println!("*** Saving slot {which}");
                    let mut state_recorder = std::mem::take(&mut self.state_recorder);
                    self.state_recorder_save(&mut state_recorder, &mut file);
                    self.state_recorder = state_recorder;
                }
            } else if let Ok(mut file) = fs::File::open(path) {
                let action = if cmd == SaveLoadCommand::Load {
                    "Loading"
                } else {
                    "Replaying"
                };
                println!("*** {action} slot {which}");
                let mut state_recorder = std::mem::take(&mut self.state_recorder);
                self.state_recorder_load(
                    &mut state_recorder,
                    &mut file,
                    cmd == SaveLoadCommand::Replay,
                );
                self.state_recorder = state_recorder;
            }
        }
    }

    pub fn replay_save_file(&mut self, path: &Path) -> std::io::Result<()> {
        let mut file = fs::File::open(path)?;
        let mut state_recorder = std::mem::take(&mut self.state_recorder);
        self.state_recorder_load(&mut state_recorder, &mut file, true);
        self.state_recorder = state_recorder;
        Ok(())
    }

    fn save_slot_path(cmd: SaveLoadCommand, which: i32) -> Option<PathBuf> {
        if which & 256 != 0 {
            if cmd == SaveLoadCommand::Save {
                return None;
            }
            let index = (which - 256) as usize;
            Some(Path::new("saves/ref").join(REFERENCE_SAVE_NAMES[index]))
        } else {
            Some(PathBuf::from(format!("saves/save{which}.sav")))
        }
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

    pub fn zelda_read_sram(&mut self) {
        let path = Self::sram_path();
        if let Ok(mut file) = fs::File::open(&path) {
            let mut total = 0usize;
            while total < SRAM_SIZE {
                match file.read(&mut self.sram[total..SRAM_SIZE]) {
                    Ok(0) => break,
                    Ok(n) => total += n,
                    Err(_) => break,
                }
            }
            if total != SRAM_SIZE {
                eprintln!("Error reading {}", path.display());
            }
            self.emu_synchronize_whole_state();
        }
    }

    pub fn zelda_write_sram(&self) {
        let path = Self::sram_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let backup_path = path.with_extension("bak");
        let _ = fs::rename(&path, &backup_path);
        match fs::File::create(&path) {
            Ok(mut file) => {
                let _ = file.write_all(&self.sram);
            }
            Err(_) => eprintln!("Unable to write {}", path.display()),
        }
    }

    fn sram_path() -> PathBuf {
        Self::sram_path_from_env(
            env::var_os("ZELDA3_SAVE_DIR"),
            env::var_os("XDG_DATA_HOME"),
            env::var_os("HOME"),
        )
    }

    fn sram_path_from_env(
        save_dir: Option<OsString>,
        xdg_data_home: Option<OsString>,
        home: Option<OsString>,
    ) -> PathBuf {
        if let Some(save_dir) = non_empty_path(save_dir) {
            return save_dir.join("sram.dat");
        }
        if let Some(xdg_data_home) = non_empty_path(xdg_data_home) {
            return xdg_data_home
                .join("zelda3-rs")
                .join("saves")
                .join("sram.dat");
        }
        if let Some(home) = non_empty_path(home) {
            return home
                .join(".local")
                .join("share")
                .join("zelda3-rs")
                .join("saves")
                .join("sram.dat");
        }
        PathBuf::from("saves/sram.dat")
    }

    fn hdma_setup(
        &mut self,
        addr6: u32,
        addr7: u32,
        transfer_unit: u8,
        reg6: u8,
        reg7: u8,
        indirect_bank: u8,
    ) {
        if addr6 != 0 {
            let ch = &mut self.dma.channel[6];
            ch.mode = transfer_unit & 7;
            ch.fixed = transfer_unit & 8 != 0;
            ch.decrement = transfer_unit & 0x10 != 0;
            ch.unused_bit = transfer_unit & 0x20 != 0;
            ch.indirect = transfer_unit & 0x40 != 0;
            ch.from_b = transfer_unit & 0x80 != 0;
            ch.b_adr = reg6;
            ch.a_adr = addr6 as u16;
            ch.a_bank = (addr6 >> 16) as u8;
            ch.ind_bank = indirect_bank;
        }

        let ch = &mut self.dma.channel[7];
        ch.mode = transfer_unit & 7;
        ch.fixed = transfer_unit & 8 != 0;
        ch.decrement = transfer_unit & 0x10 != 0;
        ch.unused_bit = transfer_unit & 0x20 != 0;
        ch.indirect = transfer_unit & 0x40 != 0;
        ch.from_b = transfer_unit & 0x80 != 0;
        ch.b_adr = reg7;
        ch.a_adr = addr7 as u16;
        ch.a_bank = (addr7 >> 16) as u8;
        ch.ind_bank = indirect_bank;
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

    fn zelda_run_poly_loop(&mut self) {
        let can_run_poly = self.game_state.ending.attract_scene.intro_did_run_step() != 0
            && !self.game_state.display.has_pending_polyhedral_update();
        if can_run_poly {
            let frame = self.game_state.frame;
            let use_timed_worker = self.rom_startup_timing
                && rom_intro_poly_thread_is_active(frame.main_module, frame.submodule);
            if use_timed_worker {
                if !self.poly_job_in_flight {
                    self.poly_run_frame();
                    self.poly_job_hold_frames = self.last_poly_work.worker_frames() - 1;
                    self.poly_job_in_flight = true;
                }
                if self.poly_job_hold_frames != 0 {
                    self.poly_job_hold_frames -= 1;
                    return;
                }
                self.poly_job_in_flight = false;
            } else {
                self.poly_run_frame();
            }
            self.attract_scene_mut().clear_intro_did_run_step();
            self.request_polyhedral_nmi_update();
        }
    }

    fn zelda_run_game_loop(&mut self) {
        if !self.dialogue_scroll_cpu_is_idle() {
            // Lag frame of an in-flight message-line scroll: the ROM's main
            // loop is still inside the scroll copy, so nothing else runs —
            // no frame-counter tick, no OAM clear, no module routing. The
            // The measured continuation copies the remaining three pixels,
            // then returns through the RenderText handler after this frame's
            // NMI. Phase 1 is consumed separately by run_frame_internal as a
            // return-only slice so its NMI stays before the game-loop suffix.
            debug_assert!(self.dialogue_scroll_is_copying_remaining_pixels());
            let passes = 3;
            let completion_timing = self.finish_dialogue_scroll_remaining_pixels();
            let command_done = self.render_text_scroll_pixels(passes);
            // The slow $0e:cfe2 text-buffer copy has now returned through
            // RenderText_Draw_MessageCharacters and RunInterface. The ROM
            // advances past the scroll command only when the low nibble
            // wrapped, then publishes $17/$0710 at $0e:c9f9/$0e:c9fc.
            if command_done {
                let read_pos = self.game_state.messaging.runtime.dialogue_msg_read_pos();
                self.messaging_state_mut()
                    .set_dialogue_msg_read_pos(read_pos.wrapping_add(1));
            }
            self.finish_dialogue_character_render_call();
            // Only at this measured continuation boundary does the ROM
            // execute Module0E_Interface's $00:f873 scroll-register suffix.
            self.complete_module0e_interface_after_run();
            if completion_timing == DialogueScrollCompletionTiming::BeforeNextVblank {
                // The v=2 oracle entry returns through Main_PrepSpritesForNmi
                // before the next vblank. Complete the caller suffix now; the
                // outer display boundary stages its text generation only after
                // preserving the scanout that ended before this publication.
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
            }
            return;
        }
        // A held frame is one the ROM spends inside a fast-forward message
        // render slice: the NMI does the VWF text upload but skips the core
        // game update, so the frame counter and sprites (incl. Link's
        // animation) do not advance. `dialogue_fast_forward_hold_active` stays
        // set through `module_main_routing` so `Module0E_Interface` skips its
        // sprite/Link update, then rotates below.
        let hold_core = self.dialogue_fast_forward_hold_active;
        let resume_dungeon_exit_spotlight =
            std::mem::take(&mut self.dungeon_exit_spotlight_resume_module);
        let run_game_loop_prefix = !hold_core && !resume_dungeon_exit_spotlight;
        if run_game_loop_prefix {
            self.increment_frame_counter();
            self.replay_trace_ram_watch("game-loop-after-frame-counter");
            self.clear_oam_buffer();
            self.replay_trace_ram_watch("game-loop-after-clear-oam");
        }
        self.module_main_routing();
        self.replay_trace_ram_watch("game-loop-after-module");
        // A vblank can interrupt the 65816 inside the VWF loop. Keep the
        // main-thread continuation separate from the ROM's $0710 NMI gate:
        // mid-glyph $0710 is still zero, while a completed handler sets it to 2.
        self.dialogue_fast_forward_hold_active =
            std::mem::take(&mut self.dialogue_fast_forward_hold_pending);
        if !self.dialogue_scroll_cpu_is_idle() {
            // The long scroll copy has crossed vblank before the ROM reaches
            // Main_PrepSpritesForNmi or clears $12. Its continuation is resumed
            // by the dedicated scheduler in run_frame_internal.
            return;
        }
        if self.rom_startup_timing()
            && (self.game_execution_scheduler.work_is_pending()
                || self.pre_main_caller_continuation_is(
                    PreMainCallerContinuation::SpiralStairsSecondPaletteFilter,
                )
                || self.pre_main_caller_continuation_is(
                    PreMainCallerContinuation::SpiralStairsSecondGrayscalePaletteFilter,
                )
                || self.dungeon_landing_wipe_return_slices_remaining != 0
                || self.normal_dialogue_initialization_phase != 0)
        {
            // A mid-close long-table iteration finishes its circle build and
            // reaches Main_PrepSpritesForNmi in this same main slice; only
            // the display-boundary bookkeeping waits for the scheduled
            // return (oracle traces pair the radius write and the $0c00d
            // decrement on one run for radii 105..63 on the maximal table).
            if self.game_state.frame.main_module == 15
                && self
                    .game_execution_scheduler
                    .spotlight_iteration()
                    .is_some_and(SpotlightIteration::is_closing)
                && rom_long_close_iteration_prep_returns_with_main(
                    self.game_state.display.spotlight_hdma.window_radius(),
                    spotlight_vertical_center(
                        self.game_state.player.follower_link.y(),
                        self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
                    ),
                )
            {
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
            }
            return;
        }
        // In the ROM this call is after Module_MainRouting. When vblank interrupts
        // the VWF loop, the main thread has not reached NMI_PrepareSprites yet;
        // the actual interrupt still runs separately below and observes $0710.
        let partial_nmi = std::mem::take(&mut self.rom_load_partial_nmi_this_frame);
        if !self.dialogue_fast_forward_hold_active && !partial_nmi {
            self.nmi_prepare_sprites();
            self.replay_trace_ram_watch("game-loop-after-prepare-sprites");
            // The ROM clears nmi_boolean only after Module_MainRouting and
            // NMI_PrepareSprites return. A vblank-interrupted VWF slice has
            // reached neither point, so its NMI must observe the still-set
            // latch and skip NMI_DoUpdates (including OAM DMA and joypads).
            self.clear_nmi_update_latch();
        }
        self.replay_trace_ram_watch("game-loop-exit");
    }

    fn clear_oam_buffer(&mut self) {
        for i in 0..128 {
            self.oam_state_mut().hide_sprite_row(i);
        }
    }

    fn run_dungeon_submodule(&mut self) {
        match self.game_state.frame.submodule {
            0 => self.module07_00_player_control(),
            1 => self.Module07_01_SubtileTransition(),
            2 => self.Module07_02_SupertileTransition(),
            3 => self.Module07_03_OverlayChange(),
            4 => self.Module07_04_UnlockDoor(),
            5 => self.Module07_05_ControlShutters(),
            6 => self.Module07_06_FatInterRoomStairs(),
            7 => self.Module07_07_FallingTransition(),
            8 => self.Module07_08_NorthIntraRoomStairs(),
            9 => self.Module07_09_OpenCrackedDoor(),
            10 => self.Module07_0A_ChangeBrightness(),
            11 => self.Module07_0B_DrainSwampPool(),
            12 => self.Module07_0C_FloodSwampWater(),
            13 => self.Module07_0D_FloodDam(),
            14 => self.Module07_0E_SpiralStairs(),
            15 => self.Module07_0F_LandingWipe(),
            16 => self.Module07_10_SouthIntraRoomStairs(),
            17..=19 => self.Module07_11_StraightInterroomStairs(),
            20 => self.Module07_14_RecoverFromFall(),
            21 => self.Module07_15_WarpPad(),
            22 => self.Module07_16_UpdatePegs(),
            23 => self.Module07_17_PressurePlate(),
            24 => self.Module07_18_RescuedMaiden(),
            25 => self.Module07_19_MirrorFade(),
            26 => self.Module07_1A_RoomDraw_OpenTriforceDoor_bounce(),
            _ => panic!("invalid dungeon submodule index"),
        }
    }

    fn handle_link_from_1d(&mut self) {
        self.follower_link_state_mut().clear_item_in_hand();
        self.follower_link_state_mut().clear_position_mode();
        self.follower_link_state_mut().clear_action_scratch_state();
        self.follower_link_state_mut().set_y_button_action_step(0);
        self.follower_link_state_mut().set_y_button_action_flags(0);
        self.follower_link_state_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.follower_link_state_mut()
            .clear_state_item_and_grab_flags();
        self.follower_link_state_mut().clear_defense_flags();
        self.link_reset_swimming_state();
        self.follower_link_state_mut().clear_direction_lock_bits(1);
        self.follower_link_state_mut().clear_z_high();
        if self.game_state.player.follower_link.electrocute_on_touch() != 0 {
            if self.game_state.player.follower_link.is_cape_active() {
                self.link_force_unequip_cape_quietly();
            }
            self.link_reset_sword_and_item_usage();
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.follower_link_state_mut().clear_action_handler_timer();
            self.follower_link_state_mut()
                .set_spin_attack_delay_timer(2);
            self.follower_link_state_mut().clear_animation_step();
            self.follower_link_state_mut().clear_cardinal_direction();
            self.set_sound_effect_1_with_link_pan(43);
            self.follower_link_state_mut().set_handler_state(7);
            self.link_state_zapped();
        } else {
            self.follower_link_state_mut()
                .clear_moving_against_diag_tile();
            self.follower_link_state_mut().set_handler_state(6);
            self.link_state_recoil();
        }
    }

    fn ancilla_add_tablet_spell(&mut self, ty: u8) {
        self.ancilla_add_simple(ty, 0);
    }

    fn link_state_pits_after_aux_state(&mut self) {
        self.replay_trace_submodule("pits-entry");
        self.replay_trace_player_state("pits-entry");
        self.tile_detect_main_handler(4);
        self.replay_trace_submodule("pits-after-tile-detect");
        self.replay_trace_player_state("pits-after-tile-detect");
        if self.game_state.player.tile_detection.pit_tile() & 1 == 0 {
            if self
                .game_state
                .enhanced_features
                .has(FEATURES0_MISC_BUG_FIXES)
            {
                self.follower_link_state_mut().clear_near_pit_state();
            }
            if self.game_state.player.follower_link.is_running() {
                self.link_state_dashing();
                return;
            }
            self.follower_link_state_mut().set_speed_setting(0);
            self.link_cancel_dash();
            if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
                self.follower_link_state_mut().clear_direction_lock_bits(1);
            }
            self.follower_link_state_mut().clear_near_pit_state();
            let handler_state = if !self.game_state.player.follower_link.is_bunny_mirror() {
                0
            } else if self.game_state.player.follower_link.has_moon_pearl() {
                3
            } else {
                23
            };
            self.follower_link_state_mut()
                .set_handler_state(handler_state);
            match self.game_state.player.follower_link.handler_state() {
                23 => self.player_handler_17_bunny(),
                3 => self.link_state_temporary_bunny(),
                _ => self.link_state_default(),
            }
            self.replay_trace_submodule("pits-no-pit-exit");
            return;
        }

        self.player_tile_detect_nearby();
        self.replay_trace_submodule("pits-after-nearby");
        self.replay_trace_player_state("pits-after-nearby");
        self.follower_link_state_mut().set_speed_setting(4);
        if self.game_state.player.tile_detection.pit_tile() & 0x0f == 0 {
            self.follower_link_state_mut().clear_near_pit_state();
            self.follower_link_state_mut().set_speed_setting(0);
            let handler_state = if !self.game_state.player.follower_link.is_bunny_mirror() {
                0
            } else if self.game_state.player.follower_link.has_moon_pearl() {
                3
            } else {
                23
            };
            self.follower_link_state_mut()
                .set_handler_state(handler_state);
            self.link_cancel_dash();
            if self.game_state.player.follower_link.button_mask_b_y() & 0x80 == 0 {
                self.follower_link_state_mut().clear_direction_lock_bits(1);
            }
            self.replay_trace_submodule("pits-clear-low-nibble-exit");
            return;
        }

        if self.game_state.player.tile_detection.pit_tile() & 0x0f != 0x0f {
            self.replay_trace_player_state("pits-edge-slide-entry");
            let mut i = 3i8;
            loop {
                if self.game_state.player.tile_detection.pit_tile() & 0x0f
                    == FALL_HOLE_PIT_DIRS[i as usize]
                {
                    i += 4;
                    break;
                }
                i -= 1;
                if i < 0 {
                    i = 3;
                    let mut pit_tile = self.game_state.player.tile_detection.pit_tile();
                    while pit_tile & 1 == 0 {
                        i -= 1;
                        pit_tile >>= 1;
                    }
                    break;
                }
            }
            self.tile_detect_position_mut()
                .set_fall_hole_scan_index(i as u8);
            let idx = i as usize;
            if self.game_state.player.follower_link.direction() & FALL_HOLE_DIRS[idx] != 0 {
                self.follower_link_state_mut()
                    .set_last_direction_from_current_direction();
                self.follower_link_state_mut().set_speed_setting(6);
                self.link_handle_moving_animation_full_long_entry();
            } else {
                let old_dir = self.game_state.player.follower_link.direction();
                self.follower_link_state_mut()
                    .add_direction_flags(FALL_HOLE_DIRS2[idx]);
                if old_dir != 0 {
                    self.link_handle_moving_animation_full_long_entry();
                }
            }
            self.link_handle_diagonal_collision();
            self.link_handle_velocity();
            self.link_handle_cardinal_collision();
            self.apply_links_movement_to_camera();
            self.replay_trace_submodule("pits-edge-slide-exit");
            self.replay_trace_player_state("pits-edge-slide-exit");
            return;
        }

        if !self.game_state.player.follower_link.near_pit_state_is(2) {
            if self.game_state.player.follower_link.has_moon_pearl() {
                self.follower_link_state_mut()
                    .clear_bunny_transform_after_moon_pearl();
            }
            self.follower_link_state_mut().set_direction(0);
            self.follower_link_state_mut().set_near_pit_state(2);
            self.follower_link_state_mut()
                .set_sprite_damage_disable_timer(1);
            self.follower_link_state_mut().set_button_mask_b_y(0);
            self.follower_link_state_mut().set_button_b_frames(0);
            self.follower_link_state_mut().clear_item_in_hand();
            self.follower_link_state_mut().clear_position_mode();
            self.follower_link_state_mut().set_incapacitated_timer(0);
            self.follower_link_state_mut().clear_auxiliary_state();
            self.ancilla_sfx3_near(31);
        }

        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().set_incapacitated_timer(0);
        self.follower_link_state_mut().set_z(0);
        self.follower_link_state_mut().set_actual_z_velocity(0);
        self.follower_link_state_mut().clear_auxiliary_state();
        self.follower_link_state_mut().clear_given_damage();
        self.follower_link_state_mut().clear_transforming();
        self.link_force_unequip_cape_quietly();
        self.follower_link_state_mut()
            .increment_sprite_damage_disable_timer();
        if (self
            .follower_link_state_mut()
            .decrement_sprite_oam_state_timer() as i8)
            >= 0
        {
            return;
        }

        self.follower_link_state_mut().advance_pit_data_index();
        let x = self.game_state.player.follower_link.pit_data_index();
        self.follower_link_state_mut().set_sprite_oam_state_timer(9);
        if self.game_state.sprites.follower_runtime.indicator() != 13 && x == 1 {
            self.follower_state_mut().set_appearance_none_flag(x);
        }

        if x == 6 {
            self.link_cancel_dash();
            self.set_submodule(7);
            self.follower_link_state_mut().set_pit_data_index(6);
            self.follower_link_state_mut().set_near_pit_state(3);
            self.follower_link_state_mut().set_visibility_status(12);
            self.follower_link_state_mut().set_speed_modifier(16);
            let y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2())
                as u8;
            self.follower_link_state_mut()
                .clear_state_item_and_grab_flags();
            self.follower_link_state_mut().set_y_button_action_timer(0);
            if self.game_state.world.location.is_indoors() {
                let room = self.game_state.world.location.dungeon_room_index();
                self.dungeon_room_tracking_mut().set_room_index_prev(room);
                self.Dungeon_FlagRoomData_Quadrants();
                if self.Dungeon_IsPitThatHurtsPlayer() {
                    self.dungeon_pit_do_damage();
                    return;
                }
            }
            let previous_room = self.game_state.world.location.dungeon_room_index();
            self.dungeon_room_tracking_mut()
                .set_room_index_prev(previous_room);
            let room = self.game_state.dungeon.header.travel_destination(0);
            self.set_dungeon_room_index(room);
            let player_y = self.game_state.player.follower_link.y();
            self.tile_detect_position_mut().set_y(player_y);
            let new_y = self
                .game_state
                .player
                .follower_link
                .y()
                .wrapping_sub(y as u16)
                .wrapping_sub(0x10);
            self.follower_link_state_mut().set_y(new_y);
            if self.game_state.world.location.is_indoors() {
                self.handle_layer_of_destination();
            } else if self.game_state.world.location.overworld_screen_index() != 5 {
                self.Overworld_GetPitDestination();
                self.set_main_module(17);
                self.set_submodule(0);
                self.set_subsubmodule(0);
            } else {
                self.replay_trace_submodule("pits-before-take-damage");
                self.TakeDamageFromPit();
            }
        }
        self.replay_trace_submodule("pits-exit");
    }

    fn link_state_tree_pull_reset_to_normal(&mut self) {
        self.follower_link_state_mut().set_facing(0);
        self.follower_link_state_mut().clear_state_bits();
        self.follower_link_state_mut().clear_direction_lock();
        self.follower_link_state_mut().clear_handler_state();
    }

    fn link_state_tree_pull_tail(&mut self) {
        self.link_move_position();
        self.link_handle_cardinal_collision();
        self.handle_indoor_camera_and_doors();
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

    fn finish_ground_movement_camera_tail(&mut self) {
        self.follower_link_state_mut().clear_pit_correction();
        if self.apply_links_movement_to_camera_called && self.game_state.enhanced_features.has(4096)
        {
            return;
        }
        self.handle_indoor_camera_and_doors();
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

    fn cache_camera_properties_for_player(&mut self) {
        let bg2_x = self.game_state.display.ppu_scroll_copy.bg2_h_copy2();
        let bg2_y = self.game_state.display.ppu_scroll_copy.bg2_v_copy2();
        self.cache_bg2_live_scroll_from(bg2_x, bg2_y);
        self.follower_link_state_mut().cache_current_position();
        let y_start = self.game_state.world.room_bounds.y_bound(0);
        let y_end = self.game_state.world.room_bounds.y_bound(2);
        let x_start = self.game_state.world.room_bounds.x_bound(0);
        let x_end = self.game_state.world.room_bounds.x_bound(2);
        self.set_cached_room_bounds(y_start, y_end, x_start, x_end);
        self.cache_scroll_targets();
        self.cache_camera_scroll();
        self.cache_quadrant_fullsize_state();
        self.follower_link_state_mut().cache_current_quadrants();
        self.follower_link_state_mut().cache_facing();
        self.follower_link_state_mut().cache_lower_level_states();
        let doorway_state = self.game_state.player.follower_link.doorway_state();
        self.cache_standing_in_doorway(doorway_state);
        self.dungeon_stair_movement_mut().cache_current_floor();
    }

    fn store_link_safe_return_position(&mut self, x: u16, y: u16) {
        self.follower_link_state_mut()
            .store_safe_return_position(x, y);
    }

    fn restore_link_safe_return_position(&mut self) {
        self.follower_link_state_mut()
            .restore_position_from_safe_return();
    }

    fn set_link_z_coord_mirror_low_ff(&mut self) {
        self.follower_link_state_mut().force_z_mirror_low_ff();
    }

    fn set_backdrop_color_black(&mut self) {
        self.set_fixed_color_red(0x20);
        self.set_fixed_color_green(0x40);
        self.set_fixed_color_blue(0x80);
    }

    fn ancilla_x(&self, k: usize) -> u16 {
        self.ancilla_slot_view(k).x()
    }

    fn ancilla_y(&self, k: usize) -> u16 {
        self.ancilla_slot_view(k).y()
    }

    fn sprite_y(&self, k: usize) -> u16 {
        self.sprite_slot_view(k).y()
    }

    fn set_oam_helper0_at(&mut self, oam: usize, x: u16, y: u16, charnum: u8, flags: u8, big: u8) {
        self.oam_state_mut()
            .write_clipped_entry_with_extended(oam, x, y, charnum, flags, big);
    }

    fn set_oam_helper1_at(&mut self, oam: usize, x: u16, y: u8, charnum: u8, flags: u8, big: u8) {
        self.oam_state_mut()
            .write_entry_with_extended(oam, x, y, charnum, flags, big);
    }

    fn write_intro_x(&mut self, k: usize, value: i16) {
        self.intro_actor_mut(k).set_x(value);
    }

    fn write_intro_y(&mut self, k: usize, value: i16) {
        self.intro_actor_mut(k).set_y(value);
    }

    fn set_oam_plain(&mut self, index: usize, x: u8, y: u8, charnum: u8, flags: u8, big: u8) {
        self.oam_state_mut()
            .write_indexed_entry_with_extended(index, x, y, charnum, flags, big);
    }

    fn set_oam_helper0_index(
        &mut self,
        index: usize,
        x: u16,
        y: u16,
        charnum: u8,
        flags: u8,
        big: u8,
    ) {
        self.oam_state_mut()
            .write_indexed_clipped_entry_with_extended(index, x, y, charnum, flags, big);
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

    fn palette_asset_word_snes(&self, addr: u32) -> Option<u16> {
        for &(base, asset) in PALETTE_ASSET_SNES_RANGES {
            let Some(byte_offset) = addr.checked_sub(base).map(|offset| offset as usize) else {
                continue;
            };
            let data = self.asset_raw(asset)?;
            if byte_offset + 1 < data.len() {
                return Some(read_word_from_slice(data, byte_offset));
            }
        }
        None
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

    pub fn replay_asset_memblk_bytes(&self, asset: usize, index: usize) -> Option<Vec<u8>> {
        self.asset_memblk(asset, index).map(|blk| blk.ptr.to_vec())
    }

    pub fn replay_asset_word(&self, asset: usize, word_index: usize) -> Option<u16> {
        let bytes = self.asset_raw(asset)?;
        let offset = word_index.checked_mul(2)?;
        (offset + 1 < bytes.len()).then(|| read_word_from_slice(bytes, offset))
    }

    pub fn mode7_character_source(&self) -> Option<&[u8]> {
        self.asset_raw(66)
    }

    pub fn replay_gloves_color(&self, index: usize) -> u16 {
        self.gloves_color[index & 1]
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

    fn clear_attract_low_work_area(&mut self) {
        SystemWorkArea::clear_attract_low_work_area(&mut self.ram);
        // The cleared range is shared by the attract controller, Link state,
        // and several scratch models. Refresh every native owner before the
        // initializer writes through a bridge, otherwise stale values can
        // immediately re-project the bytes the ROM just cleared.
        self.sync_native_game_state_from_ram();
    }

    fn clear_poly_thread_work_area(&mut self) {
        SystemWorkArea::clear_poly_thread_work_area(&mut self.ram);
        // The clear zeros the poly thread work area (0x1f00-0x1fff) directly in RAM, but the
        // PolyState native model (num_vertices at 0x1f3f, projected vertices, face coords, raster
        // edges) still holds the previous polyhedron's values. Without resyncing, a stale poly
        // field re-stamps RAM (e.g. num_vertices reverts the just-cleared 0x1f3f), leaving the
        // poly scratch a frame out of phase with the old clone (f465536).
        self.game_state.poly = crate::game_state::PolyState::load_from_ram(&self.ram);
    }

    fn write_poly_thread_init_bytes(&mut self) {
        SystemWorkArea::write_poly_thread_bootstrap_bytes(&mut self.ram);
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

    fn has_player_layer_collision(&self, mask: u8) -> bool {
        self.game_state
            .player
            .tile_detection
            .has_layer_collision(mask)
    }

    fn set_player_layer_collision(&mut self, mask: u8, enabled: bool) {
        self.tile_detect_position_mut()
            .set_layer_collision(mask, enabled);
    }

    fn set_player_layer_collision_flags(&mut self, value: u8) {
        self.tile_detect_position_mut()
            .set_layer_collision_flags(value);
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
