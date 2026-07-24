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
    ANIMATED_TILE_VRAM_ADDR, CRYSTAL_ROTATION_COUNTER, HDMA_TABLE_DYNAMIC, MESSAGING_BUF_LOAD_GFX,
    MOVING_WALL_REPLACEMENT_BUFFER, OVERWORLD_SCROLL_X_END, OVERWORLD_SCROLL_X_START,
    OVERWORLD_SCROLL_Y_END, RESERVED_HDMA_TABLE, VWF_ARR,
};
use crate::game_state::{
    lanmola_flat_trail_entry_from_ram, loaded_room_data_word, Bg1MovementAccumulatorState,
    BirdTravelDestinationState, BlastWallExplosionSlotState, BlastWallFireballSlotState,
    BlastWallFragmentSlotState, BombosBlastState, BombosFireColumnState, BossHomePositionRead,
    CachedSpriteRead, CompatibilityBytesView, CompatibilityBytesViewMut, DungeonStairList,
    FollowerLinkState, GameState, GraphicsDecompressionScratch, HappinessPondRupeeSlotState,
    HappinessPondRupeeSnapshot, HistoryPositionState, HudStateRead, HudTilemapState,
    IntroActorRead, LanmolaFlatTrailEntry, LanmolaSegmentMotionState, LinkDmaSourceSlot,
    MsuResumeInfoState, MsuResumeSlot, MultiselectChoiceRead, NativeAncillaSlotBridgeMut,
    NativeAncillaSlotView, NativeArcheryGameBridgeMut, NativeArmosKnightHomePositionBridgeMut,
    NativeArrghusPuffHomePositionBridgeMut, NativeAttractSceneBridgeMut,
    NativeAttractVramDestinationBridgeMut, NativeBeamosLaserHistoryBridgeMut,
    NativeBg1MovementAccumulatorBridgeMut, NativeBirdTravelDestinationBridgeMut,
    NativeBlastWallBridgeMut, NativeBlastWallExplosionBridgeMut, NativeBlastWallFireballBridgeMut,
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
const ROM_SELECTED_GAME_LOAD_FRAMES: u8 = 77;
// The original CPU reaches Module_PreDungeon's audio prefix after 19 NMI
// slices, then continues the room build while NMI publishes that command.
const ROM_SELECTED_GAME_LOAD_PRE_DUNGEON_AUDIO_REMAINING: u8 = 58;
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
// Item $12 uses receive-item graphics $14. The ROM decompresses packs $5b and
// $5a, then expands the selected high-plane tiles while the main-loop NMI
// latch remains set. Snes9x reaches the main-loop epilogue after four
// intervening vblanks (PCs $00e7d6, $00e7af, $00d642, then $00805f).
const ITEM_RECEIPT_GFX_14_NMI_SLICES: u8 = 4;
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

const fn rom_full_tilemap_scanout_uses_pre_nmi_vram(
    pending_full_tilemap_upload: bool,
    forced_blank_prefix_scanlines: u8,
) -> bool {
    // NMI subroutine 1 uploads the complete $800-byte staging buffer. A request
    // present in the pre-NMI snapshot was authored for the following vblank, so
    // this scanout retains the tilemap generation already in VRAM. The initial
    // menu upload is the measured overrun exception: its DMA ends at V=249 and
    // INIDISP returns at V=1, making the new tilemap visible from scanline one
    // while only the first line retains forced blank.
    pending_full_tilemap_upload && forced_blank_prefix_scanlines == 0
}

const fn rom_world_map_force_blank_scanline(
    main_module: u8,
    submodule: u8,
    map_state: u8,
    inidisp_copy: u8,
    snapshot_forced_blank: bool,
    live_forced_blank: bool,
) -> Option<u8> {
    // WorldMap_FadeOut reaches zero after the hardware NMI and writes $2100=$80
    // during active display. Instrumented Snes9x records V=43 and CurrentLine=43
    // on the continuous clean route: scanlines 0..42 retain brightness 1, while
    // scanline 43 onward is blank. Later route instances can reach this write at
    // scanline 30 depending on the 65816's host-entry NMI phase; do not guess at
    // that distinction from unrelated ROM latches.
    if main_module == 0x0e
        && submodule == 7
        && map_state == 1
        && inidisp_copy == 0x80
        && !snapshot_forced_blank
        && live_forced_blank
    {
        Some(43)
    } else {
        None
    }
}

const fn rom_display_memory_publication_is_deferred(
    main_module: u8,
    submodule: u8,
    pending_main_thread_stripe: bool,
) -> bool {
    // A mode-1 stripe packet pending at the capture boundary was authored by
    // the main thread after the active frame's hardware NMI. It is consumed by
    // the following NMI, so publishing live post-NMI memory here would expose
    // every menu stripe (file select, naming, copy, erase) one frame early.
    // The dungeon landing wipe has the same split CPU/NMI cadence: each iris
    // and sprite step is authored after its active-frame upload boundary.
    // Dialogue character tiles use their own BG3 NMI packet but share that
    // next-publication cadence. WorldMap_HandleSprites likewise authors the
    // map marker after the active frame's OAM DMA; it appears at the following
    // NMI rather than immediately in Module 14/submodule 7.
    pending_main_thread_stripe
        || rom_dungeon_landing_wipe_is_active(main_module, submodule)
        || (main_module == 14 && submodule == 2)
}

const fn rom_display_oam_publication_is_deferred(
    main_module: u8,
    submodule: u8,
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
            pending_main_thread_stripe,
        )
        || (main_module == 4 && submodule == 3)
        || (main_module == 14 && submodule == 7)
        || rom_player_sprite_scanout_uses_pre_nmi_generation(main_module, submodule)
}

const fn rom_player_sprite_scanout_uses_pre_nmi_generation(main_module: u8, submodule: u8) -> bool {
    // Snes9x returns at vblank before the new OAM and Link OBJ CHR uploads.
    // This applies both to ordinary player control and the overworld doorway
    // auxiliary-GFX load, and scroll transitions, whose Module 9/submodules 1,
    // 6 through 8, and $0a slices have already authored the following Link pose
    // and OBJ CHR when the preceding scanout is presented.
    (submodule == 0 && (main_module == 7 || matches!(main_module, 9 | 11)))
        || (main_module == 9 && matches!(submodule, 1 | 6..=8 | 0x0a))
}

const fn rom_animated_tile_dma_uses_pre_main_operands(main_module: u8, submodule: u8) -> bool {
    // This transition's long main-thread slice advances the animation source
    // before the native scheduler reaches its coarse NMI call. On hardware,
    // Snes9x resumes the already-pending NMI first, so its DMA consumes the
    // source and destination operands from the host-frame boundary.
    main_module == 9 && submodule == 5
}

const fn rom_dungeon_exit_entry_oam_publication_is_deferred(
    snapshot_main_module: u8,
    live_main_module: u8,
    live_submodule: u8,
) -> bool {
    // The module switch occurs after the active frame's OAM DMA. The following
    // main-loop slice runs Dungeon_PrepExitWithSpotlight and advances to
    // submodule 1, whose next NMI legitimately publishes the new sprite table.
    snapshot_main_module == 0x0f && live_main_module == 0x0f && live_submodule == 0
}

const fn rom_dungeon_exit_entry_scroll_publication_is_live(
    snapshot_main_module: u8,
    snapshot_submodule: u8,
    live_main_module: u8,
    live_submodule: u8,
) -> bool {
    // The first module-0x0f main-loop slice arms the deferred iris HDMA table,
    // but the preceding NMI has already published the doorway camera step.
    // Snes9x route frame 4782 scans BG1/BG2 at V=0x113 (the PPU's raw 0x112
    // plus its render-line increment), while the deferred control snapshot is
    // still at raw V=0x110. Publish only those live scroll registers here; the
    // iris controls, table, and OAM retain their independently measured lag.
    snapshot_main_module == 0x0f
        && snapshot_submodule == 0
        && live_main_module == 0x0f
        && live_submodule == 1
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
        && snapshot_submodule == 3
        && live_main_module == 9
        && live_submodule == 3
        && snapshot_half_color != live_half_color
}

const fn rom_display_snapshot_is_one_frame_deferred(main_module: u8, submodule: u8) -> bool {
    // The dungeon-exit entry setup authors its first circle before NMI enables
    // the window controls, so retain the preceding display once for submodule
    // zero. During the active close, Snes9x PC/V-counter traces show the ROM
    // rebuilding the table while HDMA consumes it in that same scanout; those
    // submodule-one frames must publish the live table instead.
    //
    // The landing wipe and overworld-entry open retain their independently
    // measured following-frame publication boundaries.
    rom_dungeon_landing_wipe_is_active(main_module, submodule)
        || (main_module == 0x0f && submodule == 0)
        || (main_module == 0x10 && submodule == 1)
}

const fn rom_attract_world_map_display_is_one_frame_deferred(
    main_module: u8,
    submodule: u8,
    sequence: u8,
    attract_state: u8,
) -> bool {
    main_module == 20 && submodule == 0 && sequence == 1 && attract_state >= 4
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

const fn rom_file_select_initial_graphics_decision(phase: u8) -> (bool, bool, u8) {
    if phase > 1 {
        (true, phase == 2, phase - 1)
    } else {
        (false, false, 0)
    }
}

const fn rom_selected_game_load_decision(remaining_frames: u8) -> (bool, bool, u8) {
    match remaining_frames {
        0 => (false, false, 0),
        1 => (false, true, 0),
        ROM_SELECTED_GAME_LOAD_PRE_DUNGEON_AUDIO_REMAINING => (true, false, remaining_frames - 1),
        _ => (false, false, remaining_frames - 1),
    }
}

const fn rom_item_receipt_graphics_nmi_slices(gfx: u8) -> u8 {
    match gfx {
        // This is the measured $5b/$5a decompression path. Keep unmeasured
        // graphics on the existing immediate path until their ROM timing has
        // been traced; shield/sword receipts do additional decompression.
        0x14 => ITEM_RECEIPT_GFX_14_NMI_SLICES,
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
const PRE_OVERWORLD_SCREEN_BUILD_NMI_SLICES: u8 = 17;
const WORLD_MAP_LIGHT_LOAD_NMI_SLICES: u8 = 5;
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
struct OverworldSpriteReloadTiming {
    load_nmi_slices: u8,
    post_return_hold_nmi_slices: u8,
}

const fn overworld_sprite_reload_timing(
    workload: OverworldSpriteReloadWorkload,
    entry_phase: OverworldSpriteReloadEntryPhase,
) -> OverworldSpriteReloadTiming {
    if matches!(
        entry_phase,
        OverworldSpriteReloadEntryPhase::VblankEdgeAfterGraphicsTail
    ) {
        // Clean Snes9x PC/V-counter traces for screen $1b enter
        // Module09_LoadNewSprites ($02:abed) at V=254, take NMI inside
        // Sprite_ResetAll ($09:c47b), resume at $09:c4ac at V=5, reach
        // Sprite_ActivateAllProxima ($09:c55e) at V=12, and return to
        // Overworld_StartScrollTransition ($02:ac27) at V=48. That causal
        // entry phase, rather than the record count alone, makes this reload
        // span exactly two host NMI boundaries.
        return OverworldSpriteReloadTiming {
            load_nmi_slices: 2,
            post_return_hold_nmi_slices: 0,
        };
    }

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
    if timing_units <= OVERWORLD_SPRITE_RELOAD_SAME_FRAME_BUDGET_UNITS {
        OverworldSpriteReloadTiming {
            load_nmi_slices: 3,
            // The light loader returns before NMI, but its next Module09
            // iteration does not reach Overworld_StartScrollTransition until
            // V=255 of the following scanout.
            post_return_hold_nmi_slices: 1,
        }
    } else {
        OverworldSpriteReloadTiming {
            load_nmi_slices: 4,
            post_return_hold_nmi_slices: 0,
        }
    }
}
// WorldMap_ExitMap enters InitializeTilesets while forced blank. From the
// From the first interrupted tileset-load frame through the boundary where the
// ROM writes music control $f3 and returns as module $09/$20, clean Snes9x
// state probes observe 33 later NMI slices (clean-route frames 17751..17783).
const WORLD_MAP_EXIT_TILESET_LOAD_NMI_SLICES: u8 = 33;
// Module $09/$20 begins on the following frame and leaves
// overworld_screen_index set to the temporary rain overlay ($9f) while
// LoadOverworldOverlay crosses six NMI boundaries (17785..17790).
const WORLD_MAP_OVERLAY_RELOAD_NMI_SLICES: u8 = 6;
// Module $09/$21 then spends four NMI boundaries converting the restored main
// Map16 page before it publishes INIDISP=0 and advances to fade submodule $22.
const WORLD_MAP_AMBIENT_MAP8_NMI_SLICES: u8 = 4;
const DUNGEON_EXIT_SPOTLIGHT_ACTIVE_SCANOUT_LIVE_TAIL_START: usize = 221;

const fn rom_dungeon_exit_spotlight_table_needs_entry_slice(radius: u16) -> bool {
    // Snes9x PC traces show the $7e and $77 circle builds crossing vblank
    // inside IrisSpotlight_ConfigureTable. From $70 downward the next table is
    // far enough along at the first boundary to publish in that same slice.
    radius >= 0x77
}

const fn rom_dungeon_exit_spotlight_resumes_during_return(radius: u16) -> bool {
    // The radius-$46 build returns at V=228 with radius $3f, then the next
    // main-loop iteration begins at V=261. That radius-$3f build commits $38
    // at V=221 of the following active scanout, before Snes9x returns the host
    // frame. Collapse that single phase-changing return boundary so subsequent
    // iterations remain aligned to the even host frames measured in WRAM.
    radius == 0x3f
}

const fn rom_dungeon_exit_spotlight_scanout_is_mixed(
    main_module: u8,
    submodule: u8,
    radius: u16,
) -> bool {
    main_module == 0x0f && submodule == 1 && radius != 0 && radius <= 0x38
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RomWorkContinuation {
    FinishAttractWorldMap,
    FinishWorldMapLightLoad,
    FinishAttractThroneRoom,
    FinishAttractZeldaPrison,
    FinishAttractMaidenWarp,
    FinishAttractEndOfStory,
    FinishItemReceiptGraphics,
    FinishSpotlightIteration,
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
    },
    HoldOverworldSpriteReloadReturn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RomWorkSlice {
    Waiting,
    Complete(RomWorkContinuation),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PendingRomWork {
    continuation: Option<RomWorkContinuation>,
    nmi_slices_remaining: u8,
}

impl PendingRomWork {
    fn schedule(continuation: RomWorkContinuation, nmi_slices: u8) -> Self {
        debug_assert!(nmi_slices != 0);
        Self {
            continuation: Some(continuation),
            nmi_slices_remaining: nmi_slices,
        }
    }

    fn is_pending(self) -> bool {
        self.continuation.is_some()
    }

    fn advance_one_nmi_slice(&mut self) -> RomWorkSlice {
        match self.continuation {
            None => RomWorkSlice::Waiting,
            Some(continuation) => {
                self.nmi_slices_remaining = self.nmi_slices_remaining.saturating_sub(1);
                if self.nmi_slices_remaining == 0 {
                    self.continuation.take();
                    RomWorkSlice::Complete(continuation)
                } else {
                    RomWorkSlice::Waiting
                }
            }
        }
    }

    fn finish(&mut self) {
        *self = Self::default();
    }
}

const fn rom_dungeon_landing_wipe_is_active(main_module: u8, submodule: u8) -> bool {
    main_module == 7 && submodule == 15
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
    submodule: u8,
    messaging_module: u8,
    attract_sequence: u8,
) -> u8 {
    if messaging_module != 0 {
        0
    } else if (main_module == 14 && submodule == 2)
        || (main_module == 20 && matches!(attract_sequence, 3 | 4))
    {
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
    dialogue_source_ir_table: Option<Vec<Vec<crate::dialogue_ir::DialogueIrOp>>>,
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
            dialogue_source_ir_table,
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
            dialogue_source_ir_table,
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
        let parsed_table;
        let table = if let Some(table) = self.dialogue_source_ir_table.as_ref() {
            table
        } else {
            parsed_table =
                Self::parse_dialogue_source_ir_table(&self.data, &self.ranges, &self.names)?;
            &parsed_table
        };
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
                scanout.offsets[layer][0] = (((u16::from(value)) << 8)
                    | (u16::from(previous) & 0xf8)
                    | (u16::from(previous2) & 0x07))
                    & 0x03ff;
                previous = value;
                previous2 = value;
            }
            for value in [v_low, v_high] {
                scanout.offsets[layer][1] =
                    (((u16::from(value)) << 8) | u16::from(previous)) & 0x03ff;
                previous = value;
            }
        }
        scanout
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub(crate) struct DialogueScrollContinuation(u8);

impl DialogueScrollContinuation {
    const IDLE: Self = Self(0);
    const RETURN_ONLY: Self = Self(1);
    const COPY_REMAINING_PIXELS: Self = Self(2);

    pub(crate) fn begin() -> Self {
        Self::COPY_REMAINING_PIXELS
    }

    pub(crate) fn is_idle(self) -> bool {
        self == Self::IDLE
    }

    fn is_copying_remaining_pixels(self) -> bool {
        self == Self::COPY_REMAINING_PIXELS
    }

    fn is_return_only(self) -> bool {
        self == Self::RETURN_ONLY
    }

    fn finish_remaining_pixels(&mut self) {
        debug_assert!(self.is_copying_remaining_pixels());
        *self = Self::RETURN_ONLY;
    }

    fn finish_return(&mut self) {
        debug_assert!(self.is_return_only());
        *self = Self::IDLE;
    }

    fn diagnostic_code(self) -> u8 {
        self.0
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
    /// Remaining 65816 master-cycle work for a VWF glyph interrupted by the
    /// preceding display boundary. The glyph becomes architecturally complete
    /// only when this reaches zero on a resumed main-thread slice.
    #[serde(skip)]
    pub(crate) dialogue_vwf_glyph_cycle_debt: u32,
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
    /// Coherent BG3 text VRAM and semantic glyph metadata as of the scroll call
    /// frame's scanout. The ROM's NMI only re-uploads the VWF buffer on
    /// main-loop iteration frames, so both representations stay frozen at the
    /// iteration-start generation while pixel-copy slices are in flight.
    #[serde(default, alias = "dialogue_scroll_frozen_text")]
    pub(crate) dialogue_scroll_frozen_scanout: Option<DialogueTextScanout>,
    /// Set while the current frame performed a long-scroll pixel-copy slice
    /// (the two-pixel start or three-pixel continuation, not the return-only
    /// suffix or cheap completing call). Latched into
    /// `dialogue_scroll_stale_scanout` at the display boundary.
    #[serde(skip)]
    pub(crate) dialogue_scroll_ran_this_frame: bool,
    /// The scanout for the presented frame falls while the ROM is mid-scroll:
    /// Snes9x displays the text generation from TWO boundaries back there
    /// (one further than the ordinary dialogue snapshot retention).
    #[serde(default)]
    pub(crate) dialogue_scroll_stale_scanout: bool,
    /// Dedicated one-frame override presenting the freshly completed coherent
    /// scanout on the group-completion frame (see the lag handler). Separate
    /// from the frozen state to avoid cascading into adjacent scroll groups.
    #[serde(skip)]
    pub(crate) dialogue_scroll_completion_scanout: Option<DialogueTextScanout>,
    #[serde(skip)]
    pub(crate) dialogue_scroll_completion_staged: Option<DialogueTextScanout>,
    /// One scanout of pre-transition BG scroll provenance. The ROM is still
    /// inside the long sprite reload while Rust's atomic simulation has
    /// already authored the next NMI scroll copies.
    #[serde(skip)]
    overworld_transition_scroll_hold: Option<[u16; 8]>,
    #[serde(skip)]
    overworld_transition_scroll_hold_pending: Option<[u16; 8]>,
    #[serde(skip)]
    overworld_transition_scroll_hold_staged: Option<[u16; 8]>,
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
    pending_rom_work: PendingRomWork,
    #[serde(skip)]
    next_overworld_sprite_reload_entry_phase: Option<OverworldSpriteReloadEntryPhase>,
    #[serde(skip)]
    joypad_sampled_before_main: bool,
    #[serde(skip)]
    audio_nmi_processed_before_main: bool,
    #[serde(skip)]
    file_select_initial_graphics_phase: u8,
    #[serde(skip)]
    file_select_checkerboard_suffix_pending: bool,
    #[serde(skip)]
    name_player_tilemap_suffix_pending: bool,
    #[serde(skip)]
    selected_game_load_remaining_frames: u8,
    #[serde(skip)]
    dungeon_landing_wipe_carry_pending: bool,
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
    #[serde(skip)]
    deferred_display_snapshot: Option<Box<DisplaySnapshot>>,
    /// Animated-BG DMA operands as they existed at the host vblank boundary.
    /// Snes9x resumes a pending NMI before the following main slice can advance
    /// the animation source.
    #[serde(skip)]
    pre_main_animated_tile_dma: Option<PreMainAnimatedTileDma>,
    #[serde(default)]
    nmi_forced_blank_scanlines_pending: u8,
    nmi_forced_blank_from_scanline_pending: Option<u8>,
    #[serde(default)]
    nmi_active_display_blanking_candidate: NmiActiveDisplayBlanking,
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

#[derive(Clone)]
struct PreMainAnimatedTileDma {
    source_address: usize,
    destination_address: usize,
    data: Vec<u8>,
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
        // C parity quirk: for a large `block` (e.g. a killed dungeon sprite whose
        // load block is near 0xffff) the wrapped address can spill OUT of the
        // overworld-sprite-loaded table and land deep in low WRAM modeled by the
        // live sprite slots (SPRITE_FLAGS4 at 0xf60 == OVERWORLD_SPRITE_WAS_LOADED +
        // 0x1fe0 wrapped). The raw `&=` above already wrote that byte, but the
        // sprite-slot native model didn't see it, so its bulk projection would
        // re-stamp the stale value on a later slot's sync. Resync the live slots
        // from RAM so the direct write sticks (matches the old clone's raw RAM).
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
            dialogue_vwf_glyph_cycle_debt: 0,
            published_bg3_vwf_glyph_runs: Vec::new(),
            published_bg3_vwf_glyph_run_dialogue_offsets: Vec::new(),
            published_dialogue_msg_read_pos: 0,
            published_dialogue_message_id: 0,
            dialogue_scroll_frozen_scanout: None,
            dialogue_scroll_completion_scanout: None,
            dialogue_scroll_completion_staged: None,
            overworld_transition_scroll_hold: None,
            overworld_transition_scroll_hold_pending: None,
            overworld_transition_scroll_hold_staged: None,
            dialogue_scroll_ran_this_frame: false,
            dialogue_scroll_stale_scanout: false,
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
            pending_rom_work: PendingRomWork::default(),
            next_overworld_sprite_reload_entry_phase: None,
            joypad_sampled_before_main: false,
            audio_nmi_processed_before_main: false,
            file_select_initial_graphics_phase: 0,
            file_select_checkerboard_suffix_pending: false,
            name_player_tilemap_suffix_pending: false,
            selected_game_load_remaining_frames: 0,
            dungeon_landing_wipe_carry_pending: false,
            dungeon_exit_spotlight_table_delay: 0,
            dungeon_exit_spotlight_resume_module: false,
            iris_spotlight_goal_transition_pending: false,
            normal_dialogue_initialization_phase: 0,
            hud_tilemap_nmi_publication_phase: 0,
            intro_poly_upload_delay: 0,
            intro_sprite_animation_start_delay: 0,
            display_snapshot: None,
            visible_display_snapshot: None,
            deferred_display_snapshot: None,
            pre_main_animated_tile_dma: None,
            nmi_forced_blank_scanlines_pending: 0,
            nmi_forced_blank_from_scanline_pending: None,
            nmi_active_display_blanking_candidate: NmiActiveDisplayBlanking::default(),
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
        self.pending_rom_work = PendingRomWork::default();
        self.joypad_sampled_before_main = false;
        self.audio_nmi_processed_before_main = false;
        self.file_select_initial_graphics_phase = 0;
        self.file_select_checkerboard_suffix_pending = false;
        self.name_player_tilemap_suffix_pending = false;
        self.selected_game_load_remaining_frames = 0;
        self.dungeon_landing_wipe_carry_pending = false;
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
            self.pending_rom_work = PendingRomWork::default();
            self.joypad_sampled_before_main = false;
            self.audio_nmi_processed_before_main = false;
            self.file_select_initial_graphics_phase = 0;
            self.file_select_checkerboard_suffix_pending = false;
            self.name_player_tilemap_suffix_pending = false;
            self.selected_game_load_remaining_frames = 0;
            self.dungeon_landing_wipe_carry_pending = false;
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
        self.rom_startup_timing = true;
    }

    pub(super) fn rom_startup_timing(&self) -> bool {
        self.rom_startup_timing
    }

    pub(super) fn schedule_spotlight_iteration_return(&mut self) {
        if !self.rom_startup_timing() {
            return;
        }
        debug_assert!(!self.pending_rom_work.is_pending());
        self.pending_rom_work = PendingRomWork::schedule(
            RomWorkContinuation::FinishSpotlightIteration,
            SPOTLIGHT_ITERATION_SUFFIX_NMI_SLICES,
        );
    }

    pub(super) fn begin_pre_overworld_properties_work(
        &mut self,
        overworld_screen: u8,
        animated_tiles: u8,
    ) -> bool {
        if !self.rom_startup_timing() || self.game_state.frame.main_module != 8 {
            return false;
        }
        debug_assert!(!self.pending_rom_work.is_pending());
        self.pending_rom_work = PendingRomWork::schedule(
            RomWorkContinuation::FinishPreOverworldProperties {
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
        debug_assert!(!self.pending_rom_work.is_pending());
        self.pending_rom_work = PendingRomWork::schedule(
            RomWorkContinuation::FinishPreOverworldOverlays,
            PRE_OVERWORLD_OVERLAYS_NMI_SLICES,
        );
        true
    }

    pub(super) fn begin_pre_overworld_screen_build_work(&mut self) -> bool {
        if !self.rom_startup_timing() || self.game_state.frame.main_module != 8 {
            return false;
        }
        debug_assert!(!self.pending_rom_work.is_pending());
        self.pending_rom_work = PendingRomWork::schedule(
            RomWorkContinuation::FinishPreOverworldScreenBuild,
            PRE_OVERWORLD_SCREEN_BUILD_NMI_SLICES,
        );
        true
    }

    pub(super) fn begin_selected_game_load(&mut self) {
        self.enable_force_blank();
        self.selected_game_load_remaining_frames = ROM_SELECTED_GAME_LOAD_FRAMES;
        // The ROM starts the heavy save-file load on this frame; its NMI is
        // PARTIAL (no Main_PrepSpritesForNmi — Snes9x holds 0xc00d here while
        // rust's game loop otherwise decrements once more on this entry frame,
        // the single event that left the BG-tile animation phase one step ahead
        // for the rest of the route, surfacing at frame 14661).
        self.rom_load_partial_nmi_this_frame = true;
    }

    #[doc(hidden)]
    pub fn zelda_debug_selected_game_load_remaining_frames(&self) -> u8 {
        self.selected_game_load_remaining_frames
    }

    pub(super) fn begin_item_receipt_graphics_work(&mut self, gfx: u8) {
        if !self.rom_startup_timing() {
            return;
        }
        let nmi_slices = rom_item_receipt_graphics_nmi_slices(gfx);
        if nmi_slices == 0 {
            return;
        }
        debug_assert!(!self.pending_rom_work.is_pending());
        self.pending_rom_work =
            PendingRomWork::schedule(RomWorkContinuation::FinishItemReceiptGraphics, nmi_slices);
    }

    pub(super) fn begin_attract_throne_room_work(&mut self) {
        debug_assert!(!self.pending_rom_work.is_pending());
        let retained_sprite_subset_2 = self.game_state.sprites.workspace.graphics_subset(2);
        self.pending_rom_work = PendingRomWork::schedule(
            RomWorkContinuation::FinishAttractThroneRoom,
            attract_throne_room_nmi_slices(retained_sprite_subset_2),
        );
    }

    pub(super) fn begin_attract_world_map_work(&mut self) {
        debug_assert!(!self.pending_rom_work.is_pending());
        // Snes9x executing the original ROM reaches attract state 4 on host
        // frame 5651. The work starts at frame 5646, so exactly five NMI
        // slices elapse before the world-map continuation runs. Seven slices
        // delayed the live scene by two frames and made the source-native
        // renderer faithfully draw the wrong state.
        self.pending_rom_work =
            PendingRomWork::schedule(RomWorkContinuation::FinishAttractWorldMap, 5);
    }

    pub(super) fn begin_world_map_light_load_work(&mut self) -> bool {
        if !self.rom_startup_timing() {
            return false;
        }
        debug_assert!(!self.pending_rom_work.is_pending());
        // The original CPU enters WorldMap_LoadLightWorldMap after host frame
        // 5934 and does not return until frame 5940. The entry frame performs
        // the first portion of the ROM work; five later NMI slices elapse
        // before the state increment and NMI-7 request become observable.
        self.pending_rom_work = PendingRomWork::schedule(
            RomWorkContinuation::FinishWorldMapLightLoad,
            WORLD_MAP_LIGHT_LOAD_NMI_SLICES,
        );
        true
    }

    pub(super) fn begin_attract_zelda_prison_work(&mut self) {
        debug_assert!(!self.pending_rom_work.is_pending());
        self.pending_rom_work = PendingRomWork::schedule(
            RomWorkContinuation::FinishAttractZeldaPrison,
            ATTRACT_ZELDA_PRISON_NMI_SLICES,
        );
    }

    pub(super) fn begin_attract_maiden_warp_work(&mut self) {
        debug_assert!(!self.pending_rom_work.is_pending());
        self.pending_rom_work = PendingRomWork::schedule(
            RomWorkContinuation::FinishAttractMaidenWarp,
            ATTRACT_MAIDEN_WARP_NMI_SLICES,
        );
    }

    pub(super) fn begin_attract_end_of_story_work(&mut self) {
        debug_assert!(!self.pending_rom_work.is_pending());
        self.pending_rom_work = PendingRomWork::schedule(
            RomWorkContinuation::FinishAttractEndOfStory,
            ATTRACT_END_OF_STORY_NMI_SLICES,
        );
    }

    pub(super) fn stage_overworld_transition_scroll_scanout_hold(&mut self) {
        self.overworld_transition_scroll_hold_pending = Some(std::array::from_fn(|index| {
            let layer = &self.ppu.bg_layer[index / 2];
            if index & 1 == 0 {
                layer.h_scroll
            } else {
                layer.v_scroll
            }
        }));
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
        self.ppu.refresh_brightness_cache();
        // The upcoming NMI may latch a fresh pre-upload CGRAM image; the one
        // from the previous frame has been consumed by that frame's renders.
        self.cgram_upload_latch = None;
        self.dialogue_scroll_stale_scanout =
            std::mem::take(&mut self.dialogue_scroll_ran_this_frame);
        if !self.dialogue_scroll_stale_scanout {
            self.dialogue_scroll_frozen_scanout = None;
        }
        // The completion override displays one boundary after the final copy
        // slice: internally the scroll finishes on frame N, but Snes9x scans
        // the finished text out on N+1.
        self.dialogue_scroll_completion_scanout = self.dialogue_scroll_completion_staged.take();
        self.overworld_transition_scroll_hold = self.overworld_transition_scroll_hold_staged.take();
        self.overworld_transition_scroll_hold_staged =
            self.overworld_transition_scroll_hold_pending.take();
        let frame = self.game_state.frame;
        if std::env::var_os("ZELDA3_DEBUG_ATTRACT_TIMELINE").is_some()
            && (5640..=5700).contains(&self.frame_ctr_dbg)
        {
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
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open("/tmp/zelda3-attract-display-timeline.trace")
            {
                let _ = writeln!(file, "{trace}");
            }
        }
        if std::env::var_os("ZELDA3_DEBUG_FRAME_BOUNDARY").is_some() {
            eprintln!(
                "frame_boundary_before host={} main={:02x} sub={:02x} latch={} pending={} target={:04x} disable={:02x}",
                self.frame_ctr_dbg,
                frame.main_module,
                frame.submodule,
                self.game_state.display.nmi_update_is_latched(),
                self.game_state.display.pending_nmi_subroutine,
                self.game_state.display.nmi_load_target_address,
                self.game_state.display.core_update_disable_flag,
            );
        }
        let mut snapshot = Box::new(DisplaySnapshot {
            ram: self.ram.clone(),
            ppu: self.ppu.clone(),
            dma: self.dma.clone(),
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
            snapshot.ppu.obj_tile_adr2 = 0x1000;
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
        self.visible_display_snapshot = None;
        if rom_display_snapshot_is_one_frame_deferred(frame.main_module, frame.submodule)
            || rom_attract_world_map_display_is_one_frame_deferred(
                frame.main_module,
                frame.submodule,
                self.game_state.ending.attract_scene.sequence(),
                self.game_state.ending.attract_scene.state(),
            )
        {
            let previous = self.deferred_display_snapshot.replace(snapshot);
            self.display_snapshot = previous.or_else(|| self.deferred_display_snapshot.clone());
        } else {
            self.deferred_display_snapshot = None;
            self.display_snapshot = Some(snapshot);
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

        let snapshot_frame = crate::game_state::FrameState::load_from_ram(&display.ram);
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
        let retain_pre_nmi_full_tilemap = rom_full_tilemap_scanout_uses_pre_nmi_vram(
            pending_full_tilemap_upload,
            display.ppu.forced_blank_scanlines,
        );
        let retain_previous_nmi_display_memory = retain_pre_nmi_full_tilemap
            || (rom_display_memory_publication_is_deferred(
                snapshot_frame.main_module,
                snapshot_frame.submodule,
                pending_main_thread_stripe,
            ) && !consumed_dialogue_box_clear)
            || (snapshot_frame.main_module == 20
                && snapshot_frame.submodule == 0
                // Snes9x retains the pre-NMI attract image through the sequence-1
                // load/fade-out. Mode 7 begins publishing immediately once that
                // sequence has entered its fade-in state.
                && !(self.game_state.ending.attract_scene.sequence() == 1
                    && self.game_state.ending.attract_scene.state() >= 4));
        let retain_previous_nmi_oam = rom_display_oam_publication_is_deferred(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
            display.ppu.forced_blank_scanlines != 0,
            pending_main_thread_stripe,
        ) || rom_dungeon_exit_entry_oam_publication_is_deferred(
            snapshot_frame.main_module,
            self.game_state.frame.main_module,
            self.game_state.frame.submodule,
        );
        let world_map_fade_display = snapshot_frame.main_module == 20
            && snapshot_frame.submodule == 0
            && self.game_state.ending.attract_scene.sequence() == 1
            && self.game_state.ending.attract_scene.state() >= 4;
        let publish_live_dungeon_exit_scroll = rom_dungeon_exit_entry_scroll_publication_is_live(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
            self.game_state.frame.main_module,
            self.game_state.frame.submodule,
        );
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
        // Module 10 defers the iris control snapshot by one frame, but animated
        // BG tiles still come from the current frame's pre-NMI VRAM. The live
        // VRAM below is post-NMI and would expose a newly selected animation
        // phase one scanout early at the exact Main_PrepSpritesForNmi boundary.
        let current_pre_nmi_animated_bg_vram = (self.game_state.frame.main_module == 0x10
            && self.game_state.frame.submodule == 1)
            .then(|| {
                self.deferred_display_snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.ppu.vram[0x3c00..0x3e00].to_vec())
            })
            .flatten();
        if std::env::var_os("ZELDA3_DEBUG_DISPLAY_OAM").is_some()
            && (snapshot_frame.main_module == 20 || self.game_state.frame.main_module == 20)
        {
            eprintln!(
                "display_oam snapshot={:02x}/{:02x} live={:02x}/{:02x} retain={} snapshot_math={:02x}/{:02x}/{}/{} live_math={:02x}/{:02x}/{}/{} snapshot_oam={:02x?} live_oam={:02x?}",
                snapshot_frame.main_module,
                snapshot_frame.submodule,
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
                retain_previous_nmi_oam,
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
        let live_forced_blank_from_scanline = self.ppu.forced_blank_from_scanline;
        let live_retain_active_display_history = self.ppu.retain_active_display_history;
        let live_brightness = self.ppu.brightness;
        let world_map_force_blank_from_scanline = rom_world_map_force_blank_scanline(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
            display.ram[crate::game_state::constants::OVERWORLD_MAP_STATE],
            display.ram[crate::game_state::constants::INIDISP_COPY],
            display.ppu.forced_blank,
            live_forced_blank,
        );
        if std::env::var_os("ZELDA3_DEBUG_NMI_LATCH").is_some()
            && (1648..=1655).contains(&self.frame_ctr_dbg)
        {
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
        if let Some(scroll) = self.overworld_transition_scroll_hold {
            for (index, layer) in self.ppu.bg_layer.iter_mut().enumerate() {
                layer.h_scroll = scroll[index * 2];
                layer.v_scroll = scroll[index * 2 + 1];
            }
        }
        if publish_live_dungeon_exit_scroll {
            for (shown, live) in self.ppu.bg_layer.iter_mut().zip(&display.ppu.bg_layer) {
                shown.h_scroll = live.h_scroll;
                shown.v_scroll = live.v_scroll;
            }
        }
        if publish_live_overworld_bad_weather_scroll {
            self.ppu.bg_layer[0].h_scroll = display.ppu.bg_layer[0].h_scroll;
            self.ppu.bg_layer[0].v_scroll = display.ppu.bg_layer[0].v_scroll;
        }
        if publish_live_overworld_transition_half_color {
            self.ppu.half_color = display.ppu.half_color;
        }
        // NMI publishes display memory for the upcoming active frame. Keep the
        // captured control registers, but compose them with the newly uploaded
        // VRAM/OAM/CGRAM rather than showing the previous frame's memory.
        // The polygon worker publishes through its NMI handshake at the start
        // of the frame. Preserve that completed pre-NMI buffer rather than a
        // job that may have finished later in the current CPU slice.
        let presented_poly = self.selected_intro_poly_display_buffer();
        // Snes9x ends `retro_run` at vblank entry: active gameplay has already
        // authored the next Link pose, but the returned scanout still uses the
        // OBJ CHR generation uploaded at the preceding NMI. Keep that pre-NMI
        // generation for player control and the measured overworld doorway
        // transition instead of composing the post-main-loop upload one frame early.
        let retain_previous_link_obj_vram = rom_player_sprite_scanout_uses_pre_nmi_generation(
            snapshot_frame.main_module,
            snapshot_frame.submodule,
        );
        let previous_link_obj_vram =
            retain_previous_link_obj_vram.then(|| self.ppu.vram[0x4000..0x4400].to_vec());
        // The normal overworld animation upload targets VRAM $3c00. Snes9x
        // normally returns the active frame that ended at this vblank, so
        // retain the captured pre-NMI generation across overworld overlays.
        // The resumed bad-weather tail is the measured exception: its rain
        // scroll and newly uploaded animated CHR are both visible on the same
        // scanout. Module 10's interrupted main thread selects its newer
        // pre-NMI image from `deferred_display_snapshot` above instead.
        let previous_overworld_animated_bg_vram = (current_pre_nmi_animated_bg_vram.is_none()
            && !publish_live_overworld_bad_weather_scroll
            && read_le_u16(&self.ram, ANIMATED_TILE_VRAM_ADDR) == 0x3c00)
            .then(|| self.ppu.vram[0x3c00..0x3e00].to_vec());
        // During a message-line scroll the ROM's NMI re-uploads the VWF text
        // buffer every frame while the copy is still in flight; Snes9x's
        // scanout shows the generation uploaded at the PREVIOUS vblank.
        // Present that pre-NMI generation of the text tile area while the
        // scroll is active (typing frames are unaffected: the region is
        // static between letter uploads).
        // During a message-line scroll's spanned frames, Snes9x's scanout
        // shows the text generation from TWO display boundaries back (the
        // ordinary dialogue presentation keeps the whole pre-NMI snapshot,
        // which is only one back). Applied after the branch below so it
        // overrides both the retained and the recomposed paths.
        // The completion override (freshly-scrolled buffer, group-completion
        // frame) takes precedence over the frozen (group-start) generation.
        let previous_dialogue_scanout =
            self.dialogue_scroll_completion_scanout.clone().or_else(|| {
                self.dialogue_scroll_stale_scanout
                    .then(|| self.dialogue_scroll_frozen_scanout.clone())
                    .flatten()
            });
        if std::env::var_os("ZELDA3_DEBUG_SCROLL_RETAIN").is_some() {
            eprintln!(
                "scroll_retain host={} lag={} stale_scanout={} two_back={} nmi_retained={}",
                self.frame_ctr_dbg,
                self.dialogue_scroll_continuation.diagnostic_code(),
                self.dialogue_scroll_stale_scanout,
                self.dialogue_scroll_frozen_scanout.is_some(),
                retain_previous_nmi_display_memory,
            );
        }
        if !retain_previous_nmi_display_memory {
            self.ppu.vram.clone_from(&display.ppu.vram);
            if let Some(animated_bg_vram) = current_pre_nmi_animated_bg_vram {
                self.ppu.vram[0x3c00..0x3e00].copy_from_slice(&animated_bg_vram);
            }
            if let Some(animated_bg_vram) = previous_overworld_animated_bg_vram {
                self.ppu.vram[0x3c00..0x3e00].copy_from_slice(&animated_bg_vram);
            }
            if let Some(previous_link_obj_vram) = previous_link_obj_vram {
                self.ppu.vram[0x4000..0x4400].copy_from_slice(&previous_link_obj_vram);
            }
            self.ppu.vram[0x5800..0x5c00].copy_from_slice(&presented_poly);
            // CGRAM: the NMI's main-palette-buffer upload is only visible on
            // the NEXT scanout (hardware uploads it in the vblank after this
            // frame was scanned), so when that upload ran this frame, display
            // its latched pre-upload image. Direct CGRAM writes outside that
            // upload (e.g. the intro poly flash, which the real game performs
            // mid-frame from the IRQ thread) stay same-frame visible via the
            // live image. The attract palette filter is byte-exact against
            // Snes9x only with this split.
            if world_map_fade_display {
                // Keep the palette from the scanout preceding the world-map
                // fade. The new Mode 7 memory and INIDISP step are visible,
                // but CGRAM is consumed on the following NMI boundary.
            } else if let Some(latch) = self.cgram_upload_latch.as_ref() {
                self.ppu.cgram.copy_from_slice(latch);
            } else {
                self.ppu.cgram.clone_from(&display.ppu.cgram);
            }
        } else if snapshot_frame.main_module == 20 && snapshot_frame.submodule == 0 {
            // The opening-attract transition retains its prior VRAM/CGRAM image
            // for this scanout, but its main-thread scene handoff has already
            // published the new BG scroll registers. Snes9x displays that
            // combination (old memory at the new origin); retaining the whole
            // PPU snapshot leaves the GPU one pixel up/left on the legend's
            // first active frame.
            for (shown, live) in self.ppu.bg_layer.iter_mut().zip(&display.ppu.bg_layer) {
                shown.h_scroll = live.h_scroll;
                shown.v_scroll = live.v_scroll;
            }
        }
        if let Some(previous_dialogue_scanout) = previous_dialogue_scanout.as_ref() {
            self.ppu.vram[0x7c00..0x7ff0].copy_from_slice(&previous_dialogue_scanout.vram);
        }
        if std::env::var_os("ZELDA3_DEBUG_SCROLL_RETAIN").is_some()
            && (self.dialogue_scroll_stale_scanout || !self.dialogue_scroll_continuation.is_idle())
        {
            let presented_sum: u64 = self.ppu.vram[0x7c00..0x7ff0]
                .iter()
                .map(|w| u64::from(w & 0xff) + u64::from(w >> 8))
                .sum();
            eprintln!(
                "scroll_present host={} presented_sum={presented_sum} stale={} lag={}",
                self.frame_ctr_dbg,
                self.dialogue_scroll_stale_scanout,
                self.dialogue_scroll_continuation.diagnostic_code(),
            );
        }
        // OAM publication has an independent cadence from the large VRAM and
        // CGRAM uploads above.  The opening attract scene deliberately keeps
        // its story image in the previous display-memory generation, while its
        // sprite DMA has already completed and is visible on this scanout.
        if !retain_previous_nmi_oam {
            self.ppu.oam.clone_from(&display.ppu.oam);
        }
        self.ppu.obj_vram_latch = None;
        self.ppu.obj_previous_frame_vram = display.ppu.obj_previous_frame_vram.clone();
        // A force-blank write published by NMI takes effect before the next
        // active scanline even though the rest of the frame remains sourced
        // from the coherent pre-NMI display snapshot.
        self.ppu.forced_blank |= live_forced_blank;
        if live_forced_blank {
            self.ppu.forced_blank_from_scanline =
                world_map_force_blank_from_scanline.or(live_forced_blank_from_scanline);
            self.ppu.retain_active_display_history = world_map_force_blank_from_scanline.is_none()
                && (self.ppu.retain_active_display_history || live_retain_active_display_history);
        }
        if std::env::var_os("ZELDA3_DEBUG_NMI_LATCH").is_some()
            && (1648..=1655).contains(&self.frame_ctr_dbg)
        {
            eprintln!(
                "display_blanking_composed host={} forced={} prefix={} from={:?}",
                self.frame_ctr_dbg,
                self.ppu.forced_blank,
                self.ppu.forced_blank_scanlines,
                self.ppu.forced_blank_from_scanline,
            );
        }
        // During the opening world-map fade Snes9x consumes the newly written
        // INIDISP level for Mode 7 BG1 while CGRAM remains on the deferred
        // display generation. Preserve that source-specific scanout timing for
        // the modern GPU finalizer without changing the composed frame's
        // global brightness or reintroducing a PPU compositor.
        self.ppu.mode7_scanout_brightness_override =
            world_map_fade_display.then_some(live_brightness);
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
        if std::env::var_os("ZELDA3_DEBUG_ATTRACT_TIMELINE").is_some()
            && (5640..=5700).contains(&self.frame_ctr_dbg)
        {
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
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open("/tmp/zelda3-attract-display-timeline.trace")
            {
                let _ = writeln!(file, "{trace}");
            }
        }
        // Semantic dialogue data is another representation of the same BG3
        // generation as the presented VRAM. A dialogue scroll override owns
        // both representations. Otherwise the retained-memory path uses the
        // pristine pre-NMI snapshot, while recomposed memory uses the live
        // post-NMI semantic publication still held on `self`.
        let presented_dialogue = if let Some(scanout) = previous_dialogue_scanout.as_ref() {
            (
                scanout.glyph_runs.clone(),
                scanout.glyph_run_dialogue_offsets.clone(),
                scanout.dialogue_msg_read_pos,
                scanout.dialogue_message_id,
            )
        } else if retain_previous_nmi_display_memory {
            (
                pristine_snapshot.published_bg3_vwf_glyph_runs.clone(),
                pristine_snapshot
                    .published_bg3_vwf_glyph_run_dialogue_offsets
                    .clone(),
                pristine_snapshot.published_dialogue_msg_read_pos,
                pristine_snapshot.published_dialogue_message_id,
            )
        } else {
            (
                self.published_bg3_vwf_glyph_runs.clone(),
                self.published_bg3_vwf_glyph_run_dialogue_offsets.clone(),
                self.published_dialogue_msg_read_pos,
                self.published_dialogue_message_id,
            )
        };
        let saved_published_dialogue = (
            std::mem::replace(&mut self.published_bg3_vwf_glyph_runs, presented_dialogue.0),
            std::mem::replace(
                &mut self.published_bg3_vwf_glyph_run_dialogue_offsets,
                presented_dialogue.1,
            ),
            std::mem::replace(
                &mut self.published_dialogue_msg_read_pos,
                presented_dialogue.2,
            ),
            std::mem::replace(
                &mut self.published_dialogue_message_id,
                presented_dialogue.3,
            ),
        );
        let captured = capture(self);
        (
            self.published_bg3_vwf_glyph_runs,
            self.published_bg3_vwf_glyph_run_dialogue_offsets,
            self.published_dialogue_msg_read_pos,
            self.published_dialogue_message_id,
        ) = saved_published_dialogue;

        std::mem::swap(&mut self.ram, &mut display.ram);
        std::mem::swap(&mut self.ppu, &mut display.ppu);
        std::mem::swap(&mut self.dma, &mut display.dma);
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

    /// `zelda_run_frame_internal`.
    ///
    /// The actual module routing, poly loop, and NMI handler are intentionally
    /// skeletal. Future ports should land behind this entry point so the
    /// lockstep oracle starts validating them immediately.
    pub fn run_frame_internal(&mut self, input: u16, run_what: u8) {
        self.sync_native_game_state_from_ram();
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        self.assert_native_display_state_matches_ram();
        self.replay_trace_col("run-frame-entry");
        self.replay_trace_ram_watch("run-frame-entry");
        if !self.initialized {
            self.zelda_initialize();
        }
        self.pre_main_animated_tile_dma = if self.rom_startup_timing()
            && rom_animated_tile_dma_uses_pre_main_operands(
                self.game_state.frame.main_module,
                self.game_state.frame.submodule,
            )
            && self.game_state.display.has_animated_tile_data_source()
        {
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
        } else {
            None
        };
        // Retain the OAM shadow at the host-frame boundary. The interrupted
        // title main thread has one late-authored OBJ region whose NMI source is
        // selected from this coherent snapshot; ordinary OAM remains current.
        let oam_dma_source = self
            .rom_startup_timing()
            .then(|| self.sprite_oam_shadow_buffer().to_vec());
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
            self.interrupt_nmi_audio_parts_locked();
            self.audio_nmi_processed_before_main = true;
        }
        if initialized_audio_bank_this_frame {
            self.zelda_initialization_code();
        }
        if self.rom_startup_timing() && self.dialogue_scroll_continuation.is_return_only() {
            let current_scanout_scroll = BgScrollRegisterScanout::capture(&self.ppu);
            // The scroll copy and RenderText handler returned after the prior
            // frame's NMI. On this boundary the next NMI sees $12 still
            // latched, so it leaves $17/$0710 pending; only afterward does the
            // caller suffix reach Main_PrepSpritesForNmi and clear $12.
            // This measured return-only slice is distinct from both the 2/3
            // pixel copy slices and from a fresh module iteration.
            self.dialogue_scroll_continuation.finish_return();
            // The interrupted NMI cannot consume the pending dialogue upload,
            // but it still advances the ordinary vblank-owned presentation
            // state (animated BG tiles, Link DMA, and OAM). Capture after that
            // NMI so the scanout combines those updates with the still-pending
            // dialogue buffer, exactly as the hardware does.
            // BG scroll is a separate register generation: writes performed by
            // this NMI configure the next active frame, while retro_run returns
            // the scanout that just ended. Preserve that pre-NMI register
            // generation without deferring the newly published memory domains.
            self.dialogue_scroll_stale_scanout = false;
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            self.capture_display_snapshot();
            if let Some(snapshot) = self.display_snapshot.as_mut() {
                current_scanout_scroll.publish_to(&mut snapshot.ppu);
            }
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
            // The completed text becomes visible at the next display
            // publication. Put it directly in the staged slot so the next
            // capture promotes it once after this text-buffer hold. Keep the
            // semantic glyph positions with the exact buffer they describe.
            self.dialogue_scroll_completion_staged =
                Some(self.dialogue_text_scanout_from_render_buffer());
            return;
        }
        if self.file_select_checkerboard_suffix_pending {
            self.complete_file_select_checkerboard_upload();
            // This continuation resumes after the prior CPU slice crossed an
            // NMI boundary. Allow the checkerboard stripe packet completed
            // above to be consumed now instead of carrying the stale latch
            // into the next fixed file-select upload.
            self.clear_nmi_update_latch();
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        if self.name_player_tilemap_suffix_pending {
            self.complete_module_name_player_1();
            // SelectFile_Func1 crosses exactly one vblank in the ROM. By this
            // boundary its suffix has returned through Module_MainRouting, so
            // Main_PrepSpritesForNmi has run and the next NMI may consume the
            // completed tilemap packet.
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        if self.rom_startup_timing() && self.file_select_initial_graphics_phase > 1 {
            let (_, complete_graphics, next_phase) =
                rom_file_select_initial_graphics_decision(self.file_select_initial_graphics_phase);
            self.file_select_initial_graphics_phase = next_phase;
            if complete_graphics {
                self.complete_module_select_file_0();
            }
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        if self.file_select_initial_graphics_phase == 1 {
            self.file_select_initial_graphics_phase = 0;
        }
        if self.rom_startup_timing() && self.selected_game_load_remaining_frames != 0 {
            let (begin_pre_dungeon_audio, complete_load, next_remaining_frames) =
                rom_selected_game_load_decision(self.selected_game_load_remaining_frames);
            self.selected_game_load_remaining_frames = next_remaining_frames;
            if begin_pre_dungeon_audio {
                self.begin_selected_game_load_pre_dungeon_audio();
            }
            if complete_load {
                self.complete_module05_load_file_after_resumption();
                self.nmi_prepare_sprites();
                self.clear_nmi_update_latch();
            }
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        if self.rom_startup_timing() && self.dungeon_landing_wipe_carry_pending {
            self.dungeon_landing_wipe_carry_pending = false;
            if self.iris_spotlight_goal_transition_pending {
                self.iris_spotlight_goal_transition_pending = false;
                self.spotlight_hdma_reset_prefix = Some(std::array::from_fn(|index| {
                    self.spotlight_hdma_table_dynamic_entry(index)
                }));
                self.complete_iris_spotlight_goal_transition();
                self.complete_module07_0f_operate_spotlight_suffix();
            }
            self.complete_module07_dungeon_after_submodule();
            self.nmi_prepare_sprites();
            self.clear_nmi_update_latch();
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        if self.rom_startup_timing() && self.normal_dialogue_initialization_phase != 0 {
            match self.normal_dialogue_initialization_phase {
                3..=5 => {
                    self.normal_dialogue_initialization_phase -= 1;
                }
                2 => {
                    self.complete_text_initialization_prefix();
                    self.prepare_text_character_buffer_for_carry();
                    self.normal_dialogue_initialization_phase = 1;
                }
                1 => {
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
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        if self.rom_startup_timing() && self.pending_rom_work.is_pending() {
            match self.pending_rom_work.advance_one_nmi_slice() {
                RomWorkSlice::Waiting => {}
                RomWorkSlice::Complete(RomWorkContinuation::FinishAttractWorldMap) => {
                    self.complete_attract_scene_world_map();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishWorldMapLightLoad) => {
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
                RomWorkSlice::Complete(RomWorkContinuation::FinishAttractThroneRoom) => {
                    self.complete_attract_scene_throne_room();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishAttractZeldaPrison) => {
                    self.complete_attract_prep_zelda_prison();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishAttractMaidenWarp) => {
                    self.complete_attract_prep_maiden_warp();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishAttractEndOfStory) => {
                    self.complete_attract_scene_end_of_story();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishItemReceiptGraphics) => {
                    // The decompressor has finally returned through
                    // Module_MainRouting. Only now can the ROM publish sprite
                    // DMA sources and release the software NMI latch.
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishSpotlightIteration) => {
                    // The opening or closing iris has returned through
                    // LinkOam_Main and the normal game-loop suffix. Publish its
                    // DMA sources and release the software NMI latch at the
                    // measured boundary.
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
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    if self.game_state.frame.main_module == 15
                        && rom_dungeon_exit_spotlight_resumes_during_return(
                            self.game_state.display.spotlight_hdma.window_radius(),
                        )
                    {
                        // The ROM has enough vblank time at this exact phase to
                        // return through the suffix and enter the next main-loop
                        // iteration before the host-visible scanout completes.
                        // Preserve the table generation already consumed by
                        // scanlines 0..220 before advancing the CPU state.
                        self.capture_display_snapshot();
                        self.nmi_read_joypads(input);
                        self.joypad_sampled_before_main = true;
                        self.increment_frame_counter();
                        self.clear_oam_buffer();
                        self.latch_nmi_update();
                        self.module_main_routing();
                        // PC/V-counter probes show the new reserved-table tail
                        // reaching HDMA before scanlines 221..223. Compose only
                        // that measured suffix into the pre-calculation image.
                        let byte_start = DUNGEON_EXIT_SPOTLIGHT_ACTIVE_SCANOUT_LIVE_TAIL_START * 2;
                        let byte_end = 224 * 2;
                        let live_tails =
                            [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE].map(|table_base| {
                                self.ram[table_base + byte_start..table_base + byte_end].to_vec()
                            });
                        if let Some(display) = self.display_snapshot.as_mut() {
                            for (table_base, live_tail) in [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE]
                                .into_iter()
                                .zip(live_tails)
                            {
                                display.ram[table_base + byte_start..table_base + byte_end]
                                    .copy_from_slice(&live_tail);
                            }
                        }
                        self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                        return;
                    } else if self.game_state.frame.main_module == 15
                        && rom_dungeon_exit_spotlight_table_needs_entry_slice(
                            self.game_state.display.spotlight_hdma.window_radius(),
                        )
                    {
                        self.dungeon_exit_spotlight_table_delay =
                            DUNGEON_EXIT_SPOTLIGHT_INTER_ITERATION_HOLD_FRAMES;
                    }
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishPreOverworldProperties {
                    overworld_screen,
                    animated_tiles,
                }) => {
                    self.complete_pre_overworld_load_properties(overworld_screen, animated_tiles);
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishPreOverworldOverlays) => {
                    self.complete_pre_overworld_load_overlays();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishPreOverworldScreenBuild) => {
                    self.complete_pre_overworld_screen_build();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishWorldMapExitTilesets) => {
                    // InitializeTilesets has returned through WorldMap_ExitMap
                    // and Module0E_Interface. Publish the same caller suffix
                    // that an uninterrupted game-loop iteration would reach.
                    self.complete_world_map_exit_after_tileset_load();
                    self.complete_module0e_interface_after_run();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishWorldMapOverlayReload) => {
                    self.finish_overworld_load_overlays();
                    self.complete_module09_overworld_after_submodule();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishWorldMapAmbientMap8) => {
                    self.Overworld_LoadAmbientOverlay(false);
                    self.complete_module09_overworld_after_submodule();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishOverworldAuxGraphics) => {
                    // PC/V-counter traces remain in LoadTransAuxGFX and
                    // PrepTransAuxGfx through this frame's vblank, returning
                    // to Module09_LoadAuxGFX immediately afterward. Preserve
                    // that ordering: this scanout uses the pre-load display,
                    // while the completed graphics and caller suffix become
                    // CPU-visible before the next frame.
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    self.complete_module09_load_aux_gfx();
                    self.complete_module09_overworld_after_submodule();
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    return;
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishOverworldMapQuadrants {
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
                    self.pending_rom_work = PendingRomWork::schedule(
                        RomWorkContinuation::FinishOverworldScreenMapAndSpriteGraphicsTail,
                        screen_map_and_sprite_gfx_tail_nmi_slices,
                    );
                    return;
                }
                RomWorkSlice::Complete(
                    RomWorkContinuation::FinishOverworldScreenMapAndSpriteGraphicsTail,
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
                    returned_scroll.publish_to(&mut self.ppu);
                    if let Some(snapshot) = self.display_snapshot.as_mut() {
                        returned_scroll.publish_to(&mut snapshot.ppu);
                    }
                    self.nmi_prepare_sprites();
                    self.clear_nmi_update_latch();
                    // The next ordinary Module09 iteration begins at the
                    // vblank edge immediately following this returned
                    // graphics tail. Carry that CPU phase explicitly into
                    // the sprite-loader timing decision instead of encoding
                    // the route or overworld screen number.
                    self.next_overworld_sprite_reload_entry_phase =
                        Some(OverworldSpriteReloadEntryPhase::VblankEdgeAfterGraphicsTail);
                    return;
                }
                RomWorkSlice::Complete(RomWorkContinuation::FinishOverworldSpriteReloadTail {
                    post_return_hold_nmi_slices,
                }) => {
                    // The long sprite reset/load loop is interrupted in the
                    // ROM. On the final slice, the CPU returns through
                    // Overworld_SetFixedColAndScroll before the vblank that
                    // ends this scanout. Snes9x therefore exposes the returned
                    // camera registers on this frame, not the following one.
                    //
                    // The measured light screen returns from $09:c55e to
                    // $02:ac27 at V=213. That leaves enough time for the
                    // submodule tail and Module09's caller suffix, but not for
                    // Main_PrepSpritesForNmi and the subsequent $0710 latch
                    // clear. The suffix's direct scroll writes are therefore
                    // visible while this NMI still suppresses OAM/animated-tile
                    // DMA. Finish the game-loop epilogue after the interrupt.
                    self.complete_module09_load_new_sprites_after_reload();
                    self.complete_module09_overworld_after_submodule();
                    if post_return_hold_nmi_slices == 0 {
                        // The measured heavy screen crosses the preceding NMI
                        // inside the loader and reaches this completion
                        // boundary early in the following scanout. Its full
                        // game-loop epilogue therefore precedes this NMI.
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    self.capture_display_snapshot();
                    self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                    if post_return_hold_nmi_slices != 0 {
                        self.nmi_prepare_sprites();
                        self.clear_nmi_update_latch();
                    }
                    if post_return_hold_nmi_slices != 0 {
                        self.pending_rom_work = PendingRomWork::schedule(
                            RomWorkContinuation::HoldOverworldSpriteReloadReturn,
                            post_return_hold_nmi_slices,
                        );
                    }
                    return;
                }
                RomWorkSlice::Complete(RomWorkContinuation::HoldOverworldSpriteReloadReturn) => {
                    // The light sprite loader returns at V=213, so its camera
                    // and caller suffix are already visible. Snes9x remains in
                    // submodule 5 for the following scanout, however; the next
                    // Overworld_StartScrollTransition call lands at V=255,
                    // after that image has been emitted. Hold only that next
                    // main-loop iteration while still running the frame NMI.
                }
            }
            // The original ROM returns to the NMI boundary after the final
            // main-thread work slice. Attract loaders and item graphics both
            // publish only after their measured continuation completes.
            self.capture_display_snapshot();
            self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
            return;
        }
        if self.rom_startup_timing()
            && run_what & crate::RUN_MAIN != 0
            && !self.dungeon_exit_spotlight_resume_module
        {
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
                self.capture_display_snapshot();
                self.interrupt_nmi(input, oam_dma_source.as_deref(), false);
                self.replay_trace_col("before-game-loop");
                self.replay_trace_ram_watch("before-game-loop");
                self.zelda_run_game_loop();
                self.replay_trace_col("after-game-loop");
                self.replay_trace_ram_watch("after-game-loop");
                debug_assert!(self
                    .dialogue_scroll_continuation
                    .is_copying_remaining_pixels());
                self.dialogue_scroll_stale_scanout =
                    std::mem::take(&mut self.dialogue_scroll_ran_this_frame);
                self.assert_native_frame_state_matches_ram();
                self.assert_native_world_location_state_matches_ram();
                self.assert_native_display_state_matches_ram();
                return;
            }
            self.nmi_read_joypads(input);
            self.joypad_sampled_before_main = true;
        }
        let dungeon_exit_spotlight_scanout_prefix = rom_dungeon_exit_spotlight_scanout_is_mixed(
            self.game_state.frame.main_module,
            self.game_state.frame.submodule,
            self.game_state.display.spotlight_hdma.window_radius(),
        )
        .then(|| {
            let byte_end = DUNGEON_EXIT_SPOTLIGHT_ACTIVE_SCANOUT_LIVE_TAIL_START * 2;
            [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE]
                .map(|table_base| self.ram[table_base..table_base + byte_end].to_vec())
        });
        if run_what & crate::RUN_MAIN != 0 {
            self.replay_trace_col("before-game-loop");
            self.replay_trace_ram_watch("before-game-loop");
            self.zelda_run_game_loop();
            self.replay_trace_col("after-game-loop");
            self.replay_trace_ram_watch("after-game-loop");
        }
        let dialogue_scroll_finished_copy =
            self.rom_startup_timing() && self.dialogue_scroll_continuation.is_return_only();
        self.capture_display_snapshot();
        if let (Some(prefixes), Some(display)) = (
            dungeon_exit_spotlight_scanout_prefix,
            self.display_snapshot.as_mut(),
        ) {
            let byte_end = DUNGEON_EXIT_SPOTLIGHT_ACTIVE_SCANOUT_LIVE_TAIL_START * 2;
            for (table_base, prefix) in [HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE]
                .into_iter()
                .zip(prefixes)
            {
                display.ram[table_base..table_base + byte_end].copy_from_slice(&prefix);
            }
        }
        self.replay_trace_col("before-nmi");
        self.replay_trace_ram_watch("before-nmi");
        let defer_dialogue_exit_bg_upload = frame.main_module == 14
            && frame.submodule == 2
            && self.game_state.frame.main_module != 14
            && self.game_state.display.has_bg_vram_load();
        self.interrupt_nmi(
            input,
            oam_dma_source.as_deref(),
            defer_dialogue_exit_bg_upload,
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
        match p {
            0x0cfa87 => Some(ATTRACT_BG_DMA_SETUP.to_vec()),
            0x0cfa94 => Some(ATTRACT_TILEMAP_DMA_SETUP.to_vec()),
            0x0ebd53 => Some(ENDING_HDMA_SETUP.to_vec()),
            0x00f2fb => Some(SPOTLIGHT_INDIRECT_HDMA_SETUP.to_vec()),
            0x0abdcf => Some(MAP_MODE_HDMA_SETUP_NEAR.to_vec()),
            0x0abdd6 => Some(MAP_MODE_HDMA_SETUP_FAR.to_vec()),
            0x0abddd => Some(ATTRACT_INDIRECT_HDMA_SETUP.to_vec()),
            0x02c80c => Some(PRAYING_SCENE_HDMA_SETUP.to_vec()),
            0x001b00 => Some(self.ram_bytes(HDMA_TABLE_DYNAMIC, 0x1e0)),
            0x001be0 => Some(self.ram_bytes(HDMA_TABLE_DYNAMIC + 0xe0, 0x100)),
            0x001bf0 => Some(self.ram_bytes(HDMA_TABLE_DYNAMIC + 0xf0, 0xf0)),
            0x0add27 => Some(Self::u16_table_bytes(&MAP_MODE_PERSPECTIVE_ZOOMS_NEAR, 0)),
            0x0ade07 => Some(Self::u16_table_bytes(
                &MAP_MODE_PERSPECTIVE_ZOOMS_NEAR,
                0xe0,
            )),
            0x0adee7 => Some(Self::u16_table_bytes(&MAP_MODE_PERSPECTIVE_ZOOMS_FAR, 0)),
            0x0adfc7 => Some(Self::u16_table_bytes(&MAP_MODE_PERSPECTIVE_ZOOMS_FAR, 0xe0)),
            0x000600 => Some(self.ram_bytes(DEBUG_ROOM_BOUNDS_TOP, 2)),
            0x000602 => Some(self.ram_bytes(OVERWORLD_SCROLL_Y_END, 2)),
            0x000604 => Some(self.ram_bytes(OVERWORLD_SCROLL_X_START, 2)),
            0x000606 => Some(self.ram_bytes(OVERWORLD_SCROLL_X_END, 2)),
            0x0000e2 => Some(self.ram_bytes(PpuScrollCopyState::bg2_h_copy2_offset(), 2)),
            _ => None,
        }
    }

    fn ram_bytes(&self, offset: usize, len: usize) -> Vec<u8> {
        self.ram
            .get(offset..offset + len)
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
        if !dc.hdma_active {
            c.table = None;
            return;
        }
        c.table = self.simple_hdma_get_ptr(dc.a_adr as u32 | ((dc.a_bank as u32) << 16));
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

    fn simple_hdma_do_line(&mut self, c: &mut SimpleHdma) {
        if c.table.is_none() {
            return;
        }

        let mut do_transfer = false;
        if c.rep_count & 0x7f == 0 {
            let Some(rep_count) = Self::simple_hdma_table_byte(c) else {
                c.table = None;
                return;
            };
            c.rep_count = rep_count;
            if c.rep_count == 0 {
                c.table = None;
                return;
            }
            if c.mode & 0x40 != 0 {
                let Some(lo) = Self::simple_hdma_table_byte(c) else {
                    c.table = None;
                    return;
                };
                let Some(hi) = Self::simple_hdma_table_byte(c) else {
                    c.table = None;
                    return;
                };
                c.indir = self
                    .simple_hdma_get_ptr(
                        ((c.indir_bank as u32) << 16) | lo as u32 | ((hi as u32) << 8),
                    )
                    .unwrap_or_default();
                c.indir_pos = 0;
            }
            do_transfer = true;
        }

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
                self.zelda_ppu_write(adr, value);
            }
        }
        c.rep_count = c.rep_count.wrapping_sub(1);
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
        SystemWorkArea::clear_startup_low_memory(&mut self.ram);
        // The reset code at ROM $008900 initially writes `00 80 19`, but the
        // Snes9x DMA trace proves that the first visible NMI reads `00 80 00`.
        // This port reaches this setup after reset execution, so retain only
        // the bytes that remain live at that NMI boundary.
        self.ram[0x0000] = 0x00;
        self.ram[0x0001] = 0x80;
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
        if !self.dialogue_scroll_continuation.is_idle() {
            // Lag frame of an in-flight message-line scroll: the ROM's main
            // loop is still inside the scroll copy, so nothing else runs —
            // no frame-counter tick, no OAM clear, no module routing. The
            // The measured continuation copies the remaining three pixels,
            // then returns through the RenderText handler after this frame's
            // NMI. Phase 1 is consumed separately by run_frame_internal as a
            // return-only slice so its NMI stays before the game-loop suffix.
            debug_assert!(self
                .dialogue_scroll_continuation
                .is_copying_remaining_pixels());
            let passes = 3;
            self.dialogue_scroll_continuation.finish_remaining_pixels();
            self.dialogue_scroll_ran_this_frame = true;
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
        if !hold_core && !resume_dungeon_exit_spotlight {
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
        if !self.dialogue_scroll_continuation.is_idle() {
            // The long scroll copy has crossed vblank before the ROM reaches
            // Main_PrepSpritesForNmi or clears $12. Its continuation is resumed
            // by the dedicated scheduler in run_frame_internal.
            return;
        }
        if self.rom_startup_timing()
            && (self.pending_rom_work.is_pending()
                || self.dungeon_landing_wipe_carry_pending
                || self.normal_dialogue_initialization_phase != 0)
        {
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
#[path = "zelda_rtl_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "chr_source_tests.rs"]
mod chr_source_tests;
