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

use snes::consts::PPU_X_PIXELS;
use snes::ppu::PpuRenderFlags;
use snes::{DmaChannel, DmaState, PpuState, WRAM_SIZE};

use crate::config::config_value_bytes;
#[cfg(test)]
use crate::game_state::constants::messaging::MODULE as MESSAGING_MODULE;
use crate::game_state::constants::nmi::{
    BG_CHAR_BUFFER_1 as NMI_BG_CHAR_BUFFER_1, BG_CHAR_HALF_BUFFER as NMI_BG_CHAR_HALF_BUFFER,
};
use crate::game_state::constants::player::LAYER_COLLISION_FLAGS as PLAYER_LAYER_COLLISION_FLAGS;
use crate::game_state::constants::{
    OVERWORLD_SCROLL_X_END, OVERWORLD_SCROLL_X_START, OVERWORLD_SCROLL_Y_END,
};
use crate::game_state::{
    AltSpriteSlotViewMut, AncillaSlotView, AncillaSlotViewMut, AncillaSpawnScratchViewMut,
    ArcheryGameView, ArcheryGameViewMut, ArmosKnightHomeView, ArmosKnightHomeViewMut,
    ArrghusPuffHomeView, AttractStateView, AttractStateViewMut, AttractVramTargetView,
    AttractVramTargetViewMut, BeamosLaserHistoryView, BeamosLaserHistoryViewMut, Bg1MoveCalcView,
    Bg1MoveCalcViewMut, BirdTravelDestinationView, BirdTravelDestinationViewMut,
    BirdTravelStatusViewMut, BlastWallExplosionView, BlastWallExplosionViewMut,
    BlastWallFireballView, BlastWallFireballViewMut, BlastWallFragmentView,
    BlastWallFragmentViewMut, BlastWallScratchView, BlastWallScratchViewMut, BombosBlastView,
    BombosBlastViewMut, BombosFireColumnView, BombosFireColumnViewMut, BombosSpellScratchView,
    BombosSpellScratchViewMut, CachedSpriteSlotView, CachedSpriteSlotViewMut,
    ChainChompHistoryView, ChainChompHistoryViewMut, DialogueMessageIndexView,
    DialogueMessageIndexViewMut, DialogueNumberView, DialogueNumberViewMut,
    DialogueSourceOffsetViewMut, DiggingGamePrizeView, DiggingGamePrizeViewMut, DisplayNmiView,
    DisplayNmiViewMut, DisplayState, DoorDebrisView, DoorDebrisViewMut, DrawScratchPositionView,
    DrawScratchPositionViewMut, DualLayerTileCacheView, DualLayerTileCacheViewMut,
    DungeonEntranceBackupViewMut, DungeonHeaderView, DungeonHeaderViewMut, DungeonKeySlotsView,
    DungeonKeySlotsViewMut, DungeonMapScratchView, DungeonMapScratchViewMut, DungeonMapViewMut,
    DungeonSecretScratchView, DungeonSecretScratchViewMut, DungeonStairList, DungeonStateView,
    DungeonStateViewMut, DungeonTorchView, DungeonTorchViewMut, EffectAngleScratchView,
    EffectAngleScratchViewMut, EndingCreditStateView, EndingCreditStateViewMut, EndingScratchView,
    EndingScratchViewMut, EnemyDamageDataView, EnemyDamageDataViewMut, EnhancedFeaturesView,
    EnhancedFeaturesViewMut, EtherOrbitView, EtherOrbitViewMut, FollowerStateView,
    FollowerStateViewMut, FrameState, GameState, GarnishSlotView, GarnishSlotViewMut,
    GarnishStateView, GarnishStateViewMut, GraphicsScratchViewMut, HappinessPondRupeeView,
    HappinessPondRupeeViewMut, HitboxScratchOffsetView, HitboxScratchOffsetViewMut,
    HudInventoryOrderView, HudInventoryOrderViewMut, HudStateView, HudStateViewMut, IntroActorView,
    IntroActorViewMut, IntroStateView, IntroStateViewMut, IntroSwordView, IntroSwordViewMut,
    InventoryStateView, InventoryStateViewMut, LanmolaSegmentMotionView,
    LanmolaSegmentMotionViewMut, MazeGameTimerView, MazeGameTimerViewMut, MemorizedTileView,
    MemorizedTileViewMut, MessagingRenderBufferView, MessagingRenderBufferViewMut,
    MessagingStateView, MessagingStateViewMut, MessagingTextView, MessagingTextViewMut,
    MinigameStateView, MinigameStateViewMut, MirrorWarpScratchView, MirrorWarpScratchViewMut,
    MoldormHistoryView, MoldormHistoryViewMut, MultiselectChoiceView, MultiselectChoiceViewMut,
    NativeDisplayStateViewMut, NativeFrameStateView, NativeFrameStateViewMut, NativeRamBridgeView,
    NativeRamBridgeViewMut, NativeVramUploadDataViewMut, NativeWorldLocationViewMut, OamStateView,
    OamStateViewMut, OverlordSlotView, OverlordSlotViewMut, OverworldConfigTableView,
    OverworldConfigTableViewMut, OverworldEventInfoView, OverworldEventInfoViewMut,
    OverworldMap16DecodeView, OverworldMap16DecodeViewMut, OverworldPaletteBackupViewMut,
    OverworldScreenSizeView, OverworldScreenSizeViewMut, OverworldScrollDeltaView,
    OverworldScrollDeltaViewMut, OverworldSpriteLoadedView, OverworldSpriteLoadedViewMut,
    OverworldSpritePresenceView, OverworldSpritePresenceViewMut, OverworldTileUpdateView,
    OverworldTileUpdateViewMut, PaletteBufferView, PaletteBufferViewMut, PaletteFilterView,
    PaletteFilterViewMut, PlayerResourcesView, PlayerResourcesViewMut, PlayerStateView,
    PlayerStateViewMut, PlayerTileAttributeView, PolyFaceCoordsView, PolyFaceCoordsViewMut,
    PolyProjectedVertexView, PolyProjectedVertexViewMut, PolyRasterEdgeView, PolyRasterEdgeViewMut,
    PolyStateView, PolyStateViewMut, PpuScrollCopyView, PpuScrollCopyViewMut,
    PrizeDropCycleViewMut, PushedBlockView, PushedBlockViewMut, QuakeBoltView, QuakeBoltViewMut,
    QuakeSpellScratchView, QuakeSpellScratchViewMut, RamFrameControlView, RoomBoundsView,
    RoomBoundsViewMut, SaveLoadScratchView, SaveLoadScratchViewMut, SaveProgressView,
    SaveProgressViewMut, ScratchWordView, ScratchWordViewMut, SelectFileScratchView,
    SelectFileScratchViewMut, SharedMessageTimerView, SharedMessageTimerViewMut,
    SkullWoodsFireScratchView, SkullWoodsFireScratchViewMut, SkullWoodsFireView,
    SkullWoodsFireViewMut, SpecialExitPositionView, SpecialExitPositionViewMut, SpotlightHdmaView,
    SpotlightHdmaViewMut, SpriteBattleView, SpriteBattleViewMut, SpriteSlotView, SpriteSlotViewMut,
    SpriteSystemView, SpriteSystemViewMut, SpriteWorkspaceView, SpriteWorkspaceViewMut,
    StarTileView, StarTileViewMut, SwamolaHistoryView, SwamolaHistoryViewMut, SwamolaTargetView,
    SwamolaTargetViewMut, SwimAccelerationView, SwimAccelerationViewMut, SystemSignalsView,
    SystemSignalsViewMut, TagalongSlotView, TagalongSlotViewMut, TempCounterView,
    TempCounterViewMut, TileDetectPositionView, TileDetectPositionViewMut, TowerSealOrbitView,
    TowerSealOrbitViewMut, TowerSealScratchView, TowerSealScratchViewMut, TowerSealSparkleView,
    TowerSealSparkleViewMut, TrinexxPaletteView, TrinexxPaletteViewMut, VramLoadStateView,
    VramLoadStateViewMut, VramUploadDataView, VwfGlyphSpacingView, VwfGlyphSpacingViewMut,
    WaterHdmaWindowView, WaterHdmaWindowViewMut, WeatherVaneDebrisView, WeatherVaneDebrisViewMut,
    WeatherVaneStateView, WeatherVaneStateViewMut, WorldLocationState, WorldStateView,
};
use crate::types::{read_le_u16, write_le_u16, xy, MemBlk};
use crate::util::{find_index_in_memblk, ByteArray, ByteArray_AppendByte, ByteArray_AppendData};

const BSNES_ROM_RESET_FRAME_DELAY: u8 = 174;
const BSNES_INTRO_MEMORY_DARKEN_FRAME_DELAY: u8 = 17;
const BSNES_POLY_SCHEDULER_FRAME_THRESHOLD: u8 = 0x1d;
const BSNES_INTRO_POLY_BOOTSTRAP_STEPS: u8 = 0;
const BSNES_INTRO_THREAD_START_DELAY: u8 = 7;
const BSNES_INTRO_SPRITE_ANIMATION_START_DELAY: u8 = 5;
const BSNES_POLY_UPLOAD_DEFER_UNTIL_FRAME_COUNTER: u8 = 0x42;
const BSNES_NMI_POLY_UPLOAD_DEFER_FRAMES: u8 = 3;

fn configured_rom_reset_frame_delay() -> u8 {
    env::var("ZELDA3_BSNES_ROM_RESET_FRAME_DELAY")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(BSNES_ROM_RESET_FRAME_DELAY)
}

pub(super) fn configured_intro_memory_darken_frame_delay() -> u8 {
    env::var("ZELDA3_BSNES_INTRO_MEMORY_DARKEN_FRAME_DELAY")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(BSNES_INTRO_MEMORY_DARKEN_FRAME_DELAY)
}

fn configured_poly_scheduler_frame_threshold() -> u8 {
    env::var("ZELDA3_BSNES_POLY_SCHEDULER_FRAME_THRESHOLD")
        .ok()
        .and_then(|value| {
            u8::from_str_radix(value.trim_start_matches("0x"), 16)
                .or_else(|_| value.parse())
                .ok()
        })
        .unwrap_or(BSNES_POLY_SCHEDULER_FRAME_THRESHOLD)
}

pub(super) fn configured_intro_poly_bootstrap_steps() -> u8 {
    env::var("ZELDA3_BSNES_INTRO_POLY_BOOTSTRAP_STEPS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(BSNES_INTRO_POLY_BOOTSTRAP_STEPS)
}

pub(super) fn configured_intro_thread_start_delay() -> u8 {
    env::var("ZELDA3_BSNES_INTRO_THREAD_START_DELAY")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(BSNES_INTRO_THREAD_START_DELAY)
}

pub(super) fn configured_intro_sprite_animation_start_delay() -> u8 {
    env::var("ZELDA3_BSNES_INTRO_SPRITE_ANIMATION_START_DELAY")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(BSNES_INTRO_SPRITE_ANIMATION_START_DELAY)
}

fn configured_nmi_poly_upload_defer_frames() -> u8 {
    env::var("ZELDA3_BSNES_NMI_POLY_UPLOAD_DEFER_FRAMES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(BSNES_NMI_POLY_UPLOAD_DEFER_FRAMES)
}

fn configured_poly_upload_defer_until_frame_counter() -> u8 {
    env::var("ZELDA3_BSNES_POLY_UPLOAD_DEFER_UNTIL_FRAME_COUNTER")
        .ok()
        .and_then(|value| {
            u8::from_str_radix(value.trim_start_matches("0x"), 16)
                .or_else(|_| value.parse())
                .ok()
        })
        .unwrap_or(BSNES_POLY_UPLOAD_DEFER_UNTIL_FRAME_COUNTER)
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

const MAIN_MODULE_INDEX: usize = 0x10;
const LINK_Y_COORD: usize = 0x20;
const LINK_X_COORD: usize = 0x22;
const LINK_Z_COORD: usize = 0x24;
const LINK_DIRECTION_LAST: usize = 0x26;
const ATTRACT_NEXT_LEGEND_GFX: usize = 0x26;
const INIDISP_COPY: usize = 0x13;
const NMI_LOAD_BG_FROM_VRAM: usize = 0x14;
const NMI_COPY_PACKETS_FLAG: usize = 0x18;
const FLAG_UPDATE_CGRAM_IN_NMI: usize = 0x15;
const FLAG_UPDATE_HUD_IN_NMI: usize = 0x16;
const FRAME_COUNTER: usize = 0x1a;
// Shared zero-page scratch; NES_Ver2 aliases include BMWORK/CRTNL/CRTNR, but these slots
// are reused by unrelated player, overworld, and tile-detection code paths.
const SCRATCH_0: usize = 0x72;
const SCRATCH_A: usize = 0x73;
const SCRATCH_1: usize = 0x74;
const PLAYER_IS_INDOORS: usize = 0x1b;
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
const CGWSEL_COPY: usize = 0x99;
const CGADSUB_COPY: usize = 0x9a;
const COLDATA_COPY0: usize = 0x9c;
const COLDATA_COPY1: usize = 0x9d;
const COLDATA_COPY2: usize = 0x9e;
const BG1_X_OFFSET: usize = 0x11a;
const BG1_Y_OFFSET: usize = 0x11c;
const FLAG_CUSTOM_SPELL_ANIM_ACTIVE: usize = 0x112;
const OAM_CUR_PTR: usize = 0x90;
const OAM_EXT_CUR_PTR: usize = 0x92;
const OVERWORLD_SCREEN_INDEX: usize = 0x8a;
const OVERLAY_INDEX: usize = 0x8c;
const LAST_LIGHT_VS_DARK_WORLD: usize = 0x7b;
const SUBMODULE_INDEX: usize = 0x11;
const SUBSUBMODULE_INDEX: usize = 0xb0;
const DUNG_DRAW_WIDTH_INDICATOR: usize = 0xb2;
const DUNG_DRAW_HEIGHT_INDICATOR: usize = 0xb4;
const DUNGEON_ROOM_INDEX_PREV: usize = 0xa2;
const DUNG_LINE_PTRS_ROW0: usize = 0xbf;
const DUNG_LOAD_PTR_OFFS: usize = 0xba;
const DUNGEON_ROOM_INDEX: usize = 0xa0;
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
const ANIMATED_TILE_VRAM_ADDR: usize = 0x134;
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
const SAVED_MODULE_FOR_MENU: usize = 0x10c;
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
const RESERVED_HDMA_TABLE: usize = 0x17000;
const HDMA_TABLE_DYNAMIC: usize = 0x1dba0;
const SAVELOAD_HDMA_TABLE: usize = 0x1b00;
const SPRITE_LIMIT_INSTANCE: usize = 0x0b6a;
const SPRITE_STUNNED: usize = 0x0b58;
const LINK_PREVENT_FROM_MOVING: usize = 0x0b7b;
const DRAG_PLAYER_X: usize = 0x0b7c;
const DRAG_PLAYER_Y: usize = 0x0b7e;
const DUNGEON_ROOM_HISTORY: usize = 0x0b80;
const ARCHERY_GAME_HIT_COUNTER: usize = 0x0b88;
const ITEM_DROP_COUNTER: usize = 0x0b9b;
const ENHANCED_FEATURES0: usize = 0x064c;
const RAM_CRYSTAL_ROTATE_COUNTER: usize = 0x0649;
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
const DUNG_REPLACEMENT_TILE_STATE: usize = 0x500;
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
const PALETTE_SP0L: usize = 0x0aac;
const PALETTE_SP5L: usize = 0x0aad;
const PALETTE_SP6L: usize = 0x0aae;
const PALETTE_SP6R_INDOORS: usize = 0x0ab1;
const HUD_PALETTE: usize = 0x0ab2;
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
const OVERWORLD_PALETTE_MODE: usize = 0x0ab3;
const OVERWORLD_PALETTE_AUX1_BP2TO4_HI: usize = 0x0ab4;
const OVERWORLD_PALETTE_AUX2_BP5TO7_HI: usize = 0x0ab5;
const PALETTE_MAIN_INDOORS: usize = 0x0ab6;
const PALETTE_MAIN_INDOORS_COPY: usize = 0x0ab7;
const OVERWORLD_PALETTE_AUX3_BP7_LO: usize = 0x0ab8;
const PALETTE_SWAP_FLAG: usize = 0x0abd;
const EXTENDED_OAM: usize = 0x0a00;
const ANIMATED_TILE_DATA_SRC: usize = 0x0adc;
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
const DMA_SOURCE_ADDR_6: usize = 0x0ac0;
const DMA_SOURCE_ADDR_11: usize = 0x0ac2;
const DMA_SOURCE_ADDR_7: usize = 0x0ac4;
const DMA_SOURCE_ADDR_12: usize = 0x0ac6;
const DMA_SOURCE_ADDR_8: usize = 0x0ac8;
const DMA_SOURCE_ADDR_13: usize = 0x0aca;
const DMA_SOURCE_ADDR_3: usize = 0x0acc;
const DMA_SOURCE_ADDR_0: usize = 0x0ace;
const DMA_SOURCE_ADDR_4: usize = 0x0ad0;
const DMA_SOURCE_ADDR_1: usize = 0x0ad2;
const DMA_SOURCE_ADDR_5: usize = 0x0ad4;
const DMA_SOURCE_ADDR_2: usize = 0x0ad6;
const DMA_SOURCE_ADDR_10: usize = 0x0ad8;
const DMA_SOURCE_ADDR_15: usize = 0x0ada;
const DMA_SOURCE_ADDR_9: usize = 0x0ae0;
const DMA_SOURCE_ADDR_14: usize = 0x0ae2;
// NES_Ver2: OPTHPT/OPTBPT, option head/body DMA pointers.
const DMA_SOURCE_ADDR_16: usize = 0x0aec;
const DMA_SOURCE_ADDR_18: usize = 0x0aee;
const DMA_SOURCE_ADDR_17: usize = 0x0af0;
const DMA_SOURCE_ADDR_19: usize = 0x0af2;
const FLAG_TRAVEL_BIRD: usize = 0x0af4;
const DMA_SOURCE_ADDR_20: usize = 0x0af6;
const DMA_SOURCE_ADDR_21: usize = 0x0af8;
const PALETTE_FILTER_COUNTDOWN: usize = 0x0c007;
const DARKENING_OR_LIGHTENING_SCREEN: usize = 0x0c009;
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
const BG_TILE_ANIMATION_COUNTDOWN: usize = 0x0c00d;
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
}

impl AssetPack {
    fn parse(data: &[u8]) -> Result<Self, String> {
        const ASSET_SIGNATURE_PREFIX: &[u8; 16] = b"Zelda3_v0     \n\0";
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

        Ok(Self {
            data: data.to_vec(),
            ranges,
        })
    }

    fn asset(&self, index: usize) -> Option<&[u8]> {
        let (start, end) = *self.ranges.get(index)?;
        Some(&self.data[start..end])
    }

    fn asset_mut(&mut self, index: usize) -> Option<&mut [u8]> {
        let (start, end) = *self.ranges.get(index)?;
        Some(&mut self.data[start..end])
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ZeldaState {
    pub ram: Vec<u8>,
    #[serde(default)]
    pub(crate) game_state: GameState,
    pub sram: Vec<u8>,
    pub ppu: PpuState,
    pub dma: DmaState,
    pub frame_ctr_dbg: u32,
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
    #[serde(skip)]
    rom_startup_timing: bool,
    #[serde(skip)]
    intro_startup_delay: u8,
    #[serde(skip)]
    rom_reset_frame_delay: u8,
    #[serde(skip)]
    intro_memory_darken_frame_delay: u8,
    #[serde(skip)]
    intro_poly_upload_delay: u8,
    #[serde(skip)]
    intro_sprite_animation_start_delay: u8,
    #[serde(skip)]
    display_snapshot: Option<Box<DisplaySnapshot>>,
    #[serde(skip)]
    visible_display_snapshot: Option<Box<DisplaySnapshot>>,
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
    #[serde(skip)]
    bsnes_poly_scheduler_counter: u8,
    #[serde(skip)]
    bsnes_hold_intro_step_this_frame: bool,
    #[serde(skip)]
    bsnes_intro_step_carry_phase_active: bool,
    #[serde(skip)]
    bsnes_intro_step_hold_alternate: bool,
    #[serde(skip)]
    replay_reload_file_select_stall: u8,
    #[serde(skip)]
    replay_loadfile_stall: u8,
    #[serde(skip)]
    overworld_map16_load: OverworldMap16LoadState,
    #[serde(skip)]
    overworld_prev_map16_load: OverworldMap16LoadState,
    #[serde(skip)]
    overworld_spexit_map16_src_off: u16,
    #[serde(skip)]
    overworld_exit_map16_src_off: u16,
    #[serde(skip)]
    small_overworld_map16_scroll_backup: SmallOverworldMap16ScrollBackupState,
    ending_coords: sprite::PrepOamCoordsRet,
    #[serde(skip)]
    intro_poly_vram_history: Vec<(u8, Vec<u16>, Vec<u16>)>,
    #[serde(skip)]
    intro_poly_presented_vram: Option<(u8, Vec<u16>)>,
    #[serde(skip)]
    audio: audio::AudioState,
    #[serde(skip)]
    emu_memory_ptr: Option<Vec<u8>>,
    #[serde(skip)]
    emu_runframe: Option<ZeldaRunFrameFunc>,
    #[serde(skip)]
    emu_syncall: Option<ZeldaSyncAllFunc>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OverworldMap16LoadState {
    pub src_off: u16,
    pub dst_off: u16,
    pub y_unit: u16,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SmallOverworldMap16ScrollBackupState {
    pub src_off: u16,
    pub dst_off: u16,
    pub y_unit: u16,
}

#[derive(Clone)]
struct DisplaySnapshot {
    ram: Vec<u8>,
    ppu: PpuState,
    dma: DmaState,
    intro_poly_upload_delay: u8,
    intro_sprite_animation_start_delay: u8,
    rom_reset_frame_delay: u8,
    intro_memory_darken_frame_delay: u8,
    nmi_poly_upload_deferred: u8,
    obj_vram_latch_generation: u64,
    bsnes_poly_scheduler_counter: u8,
}

pub type ZeldaRunFrameFunc = fn(&mut ZeldaState, u16, i32);
pub type ZeldaSyncAllFunc = fn(&mut ZeldaState);

fn default_gloves_color() -> [u16; 2] {
    [0x52f6, 0x0376]
}

const SEMANTIC_MAP16_LOAD_SRC_OFF: usize = 0x0084;
const SEMANTIC_MAP16_LOAD_DST_OFF: usize = 0x0086;
const SEMANTIC_MAP16_LOAD_Y_UNIT: usize = 0x0088;
const SEMANTIC_MAP16_LOAD_SRC_OFF_SPEXIT: usize = 0x0c10e;
const SEMANTIC_MAP16_LOAD_SRC_OFF_EXIT: usize = 0x0c14e;
const SEMANTIC_SMALL_OW_MAP16_SRC_OFF_BACKUP: usize = 0x0c172;
const SEMANTIC_SMALL_OW_MAP16_DST_OFF_BACKUP: usize = 0x0c174;
const SEMANTIC_SMALL_OW_MAP16_Y_UNIT_BACKUP: usize = 0x0c176;
const SEMANTIC_MAP16_LOAD_SRC_OFF_PREV: usize = 0x0c215;
const SEMANTIC_MAP16_LOAD_Y_UNIT_PREV: usize = 0x0c217;
const SEMANTIC_MAP16_LOAD_DST_OFF_PREV: usize = 0x0c219;

fn wram_patch_addr(addr: usize) -> u32 {
    debug_assert!(addr < WRAM_SIZE);
    addr as u32
}

impl ZeldaState {
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
        let frame = self.frame_state();
        eprintln!(
            "replay-col frame={} {label} main={} sub={} subsub={} col=0x{:02x},0x{:02x} door=0x{:02x} last=0x{:02x} dlast=0x{:02x} speed=0x{:02x}/0x{:02x} dir=0x{:02x} state=0x{:02x} x=0x{:04x} y=0x{:04x}",
            self.frame_ctr_dbg,
            frame.main_module,
            frame.submodule,
            frame.subsubmodule,
            self.tile_detect_position_view().tile_collision_bits_primary(),
            self.tile_detect_position_view().tile_collision_bits_secondary(),
            self.dungeon_state_view().door_open_counter_low(),
            self.player_state_view().last_direction(),
            self.player_state_view().swim_direction_flags(),
            self.player_state_view().speed_setting(),
            self.player_state_view().speed_modifier(),
            self.player_state_view().direction(),
            self.player_state_view().handler_state(),
            self.player_state_view().x(),
            self.player_state_view().y(),
        );
    }

    fn replay_trace_ram_watch(&self, label: &str) {
        let Some(target) = Self::parse_trace_env_u32("ZELDA3_REPLAY_RAM_WATCH_FRAME") else {
            return;
        };
        if self.frame_ctr_dbg != target {
            return;
        }
        let watched_addr = Self::parse_trace_env_u32("ZELDA3_REPLAY_RAM_WATCH_ADDR")
            .and_then(|addr| self.ram.get(addr as usize).map(|value| (addr, *value)));
        let frame = self.frame_state();
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
            self.player_state_view().swim_direction_flags(),
            self.tile_detect_position_view().deepwater() as u8,
            self.tile_detect_position_view().deepwater_high(),
            self.tile_detect_position_view().normal_tiles() as u8,
            self.tile_detect_position_view().normal_tiles_high(),
            self.player_state_view().deep_water_state(),
            self.player_state_view().palette_bits_of_oam(),
            self.tile_detect_position_view().palette_bits_high(),
            self.tile_detect_position_view().deepwater(),
            self.tile_detect_position_view().normal_tiles(),
            self.player_state_view().deep_water_state(),
            self.player_state_view().x(),
            self.player_state_view().y(),
            self.player_state_view().handler_state(),
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
        let frame = self.frame_state();
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
            self.system_signals_view().ambient_sound_effect(),
            self.system_signals_view().sound_effect_1(),
            self.system_signals_view().sound_effect_2(),
            self.system_signals_view().raw_sfx_pan_value(),
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
        let frame = self.frame_state();
        if frame.frame_counter as u32 != target {
            return false;
        }
        if let Some(target) = Self::parse_trace_env_u32("ZELDA3_REPLAY_TRACE_SUB_MAIN") {
            if frame.main_module as u32 != target {
                return false;
            }
        }
        if let Some(target) = Self::parse_trace_env_u32("ZELDA3_REPLAY_TRACE_SUB_OW") {
            if u32::from(self.world_location_state().overworld_screen) != target {
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
        let frame = self.frame_state();
        let world_location = self.world_location_state();
        eprintln!(
            "replay-sub frame={} {label} main={} sub={} subsub={} state=0x{:02x} nearpit=0x{:02x} pit=0x{:02x} water=0x{:04x} deep=0x{:04x} flippers=0x{:02x} bunny=0x{:02x} pearl=0x{:02x} indoors={} ow=0x{:04x} vis=0x{:02x} x=0x{:04x} y=0x{:04x} subpix=0x{:02x}/0x{:02x} vel=0x{:02x}/0x{:02x} yvel=0x{:02x} dir=0x{:02x} last=0x{:02x} dlast=0x{:02x} r14=0x{:04x} r12=0x{:04x} normal=0x{:04x} vledge=0x{:02x} stair=0x{:02x} drag=0x{:02x} hp=0x{:02x}",
            frame.frame_counter,
            frame.main_module,
            frame.submodule,
            frame.subsubmodule,
            self.player_state_view().handler_state(),
            self.player_state_view().near_pit_state(),
            self.tile_detect_position_view().pit_tile(),
            self.tile_detect_position_view().water_staircase(),
            self.tile_detect_position_view().deepwater(),
            self.player_state_view().flippers(),
            self.player_state_view().is_bunny_mirror() as u8,
            self.player_state_view().moon_pearl(),
            world_location.indoor_flag,
            world_location.overworld_screen,
            self.player_state_view().visibility_status(),
            self.player_state_view().x(),
            self.player_state_view().y(),
            self.player_state_view().x_subpixel(),
            self.player_state_view().y_subpixel(),
            self.player_state_view().actual_x_velocity(),
            self.player_state_view().actual_y_velocity(),
            self.player_state_view().y_velocity(),
            self.player_state_view().direction(),
            self.player_state_view().last_direction(),
            self.player_state_view().last_direction_moved_towards(),
            self.tile_detect_position_view().collision_bits(),
            self.tile_detect_position_view().slope_collision_bits(),
            self.tile_detect_position_view().normal_tiles(),
            self.tile_detect_position_view().vertical_ledge(),
            self.tile_detect_position_view().stair_tile(),
            self.player_state_view().defense_flags(),
            self.player_resources_view().current_health(),
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

    pub fn debug_bsnes_poly_scheduler_counter(&self) -> u8 {
        self.bsnes_poly_scheduler_counter
    }

    pub fn debug_bsnes_hold_intro_step_this_frame(&self) -> bool {
        self.bsnes_hold_intro_step_this_frame
    }

    pub fn debug_bsnes_intro_step_carry_phase_active(&self) -> bool {
        self.bsnes_intro_step_carry_phase_active
    }

    pub fn debug_bsnes_intro_step_hold_alternate(&self) -> bool {
        self.bsnes_intro_step_hold_alternate
    }

    pub(crate) fn player_state_view(&self) -> PlayerStateView<'_> {
        PlayerStateView::new(&self.ram)
    }

    pub(crate) fn player_state_view_mut(&mut self) -> PlayerStateViewMut<'_> {
        PlayerStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn enhanced_features_view(&self) -> EnhancedFeaturesView<'_> {
        EnhancedFeaturesView::new(&self.ram)
    }

    pub(crate) fn enhanced_features_view_mut(&mut self) -> EnhancedFeaturesViewMut<'_> {
        EnhancedFeaturesViewMut::new(&mut self.ram)
    }

    pub(crate) fn system_signals_view(&self) -> SystemSignalsView<'_> {
        SystemSignalsView::new(&self.ram)
    }

    pub(crate) fn system_signals_view_mut(&mut self) -> SystemSignalsViewMut<'_> {
        SystemSignalsViewMut::new(&mut self.ram)
    }

    pub(crate) fn display_nmi_view(&self) -> DisplayNmiView<'_> {
        DisplayNmiView::new(&self.ram)
    }

    pub(crate) fn display_nmi_view_mut(&mut self) -> DisplayNmiViewMut<'_> {
        DisplayNmiViewMut::new(&mut self.ram)
    }

    pub(crate) fn native_ram_bridge_view(&self) -> NativeRamBridgeView<'_> {
        NativeRamBridgeView::new(&self.ram)
    }

    pub(crate) fn native_ram_bridge_view_mut(&mut self) -> NativeRamBridgeViewMut<'_> {
        NativeRamBridgeViewMut::new(&mut self.ram)
    }

    pub(crate) fn set_sound_effect_1_with_link_pan(&mut self, effect: u8) {
        let sound_effect = self.link_calculate_sfx_pan() | effect;
        self.system_signals_view_mut()
            .set_sound_effect_1(sound_effect);
    }

    pub(crate) fn set_sound_effect_2_with_link_pan(&mut self, effect: u8) {
        let sound_effect = self.link_calculate_sfx_pan() | effect;
        self.system_signals_view_mut()
            .set_sound_effect_2(sound_effect);
    }

    pub(crate) fn set_sound_effect_1_with_ancilla_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.ancilla_calculate_sfx_pan(slot) | effect;
        self.system_signals_view_mut()
            .set_sound_effect_1(sound_effect);
    }

    pub(crate) fn set_sound_effect_2_with_ancilla_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.ancilla_calculate_sfx_pan(slot) | effect;
        self.system_signals_view_mut()
            .set_sound_effect_2(sound_effect);
    }

    pub(crate) fn set_sound_effect_1_with_sprite_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.sprite_calculate_sfx_pan(slot) | effect;
        self.system_signals_view_mut()
            .set_sound_effect_1(sound_effect);
    }

    pub(crate) fn set_sound_effect_2_with_sprite_pan(&mut self, slot: usize, effect: u8) {
        let sound_effect = self.sprite_calculate_sfx_pan(slot) | effect;
        self.system_signals_view_mut()
            .set_sound_effect_2(sound_effect);
    }

    pub(crate) fn special_exit_position_view(&self) -> SpecialExitPositionView<'_> {
        SpecialExitPositionView::new(&self.ram)
    }

    pub(crate) fn special_exit_position_view_mut(&mut self) -> SpecialExitPositionViewMut<'_> {
        SpecialExitPositionViewMut::new(&mut self.ram)
    }

    pub(crate) fn swim_acceleration_view(&self) -> SwimAccelerationView<'_> {
        SwimAccelerationView::new(&self.ram)
    }

    pub(crate) fn swim_acceleration_view_mut(&mut self) -> SwimAccelerationViewMut<'_> {
        SwimAccelerationViewMut::new(&mut self.ram)
    }

    pub(crate) fn bg1_move_calc_view(&self) -> Bg1MoveCalcView<'_> {
        Bg1MoveCalcView::new(&self.ram)
    }

    pub(crate) fn bg1_move_calc_view_mut(&mut self) -> Bg1MoveCalcViewMut<'_> {
        Bg1MoveCalcViewMut::new(&mut self.ram)
    }

    pub(crate) fn tile_detect_position_view(&self) -> TileDetectPositionView<'_> {
        TileDetectPositionView::new(&self.ram)
    }

    pub(crate) fn tile_detect_position_view_mut(&mut self) -> TileDetectPositionViewMut<'_> {
        TileDetectPositionViewMut::new(&mut self.ram)
    }

    pub(crate) fn ppu_scroll_copy_view(&self) -> PpuScrollCopyView<'_> {
        PpuScrollCopyView::new(&self.ram)
    }

    pub(crate) fn ppu_scroll_copy_view_mut(&mut self) -> PpuScrollCopyViewMut<'_> {
        PpuScrollCopyViewMut::new(&mut self.ram)
    }

    pub(crate) fn attract_vram_target_view(&self) -> AttractVramTargetView<'_> {
        AttractVramTargetView::new(&self.ram)
    }

    pub(crate) fn attract_vram_target_view_mut(&mut self) -> AttractVramTargetViewMut<'_> {
        AttractVramTargetViewMut::new(&mut self.ram)
    }

    pub(crate) fn attract_state_view(&self) -> AttractStateView<'_> {
        AttractStateView::new(&self.ram)
    }

    pub(crate) fn attract_state_view_mut(&mut self) -> AttractStateViewMut<'_> {
        AttractStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn dialogue_message_index_view(&self) -> DialogueMessageIndexView<'_> {
        DialogueMessageIndexView::new(&self.ram)
    }

    pub(crate) fn dialogue_message_index_view_mut(&mut self) -> DialogueMessageIndexViewMut<'_> {
        DialogueMessageIndexViewMut::new(&mut self.ram)
    }

    pub(crate) fn multiselect_choice_view(&self) -> MultiselectChoiceView<'_> {
        MultiselectChoiceView::new(&self.ram)
    }

    pub(crate) fn multiselect_choice_view_mut(&mut self) -> MultiselectChoiceViewMut<'_> {
        MultiselectChoiceViewMut::new(&mut self.ram)
    }

    pub(crate) fn pushed_block_view(&self) -> PushedBlockView<'_> {
        PushedBlockView::new(&self.ram)
    }

    pub(crate) fn pushed_block_view_mut(&mut self) -> PushedBlockViewMut<'_> {
        PushedBlockViewMut::new(&mut self.ram)
    }

    pub(crate) fn player_tile_attribute_view(&self) -> PlayerTileAttributeView<'_> {
        PlayerTileAttributeView::new(&self.ram)
    }

    pub(crate) fn inventory_state_view(&self) -> InventoryStateView<'_> {
        InventoryStateView::new(&self.ram)
    }

    pub(crate) fn inventory_state_view_mut(&mut self) -> InventoryStateViewMut<'_> {
        InventoryStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn player_resources_view(&self) -> PlayerResourcesView<'_> {
        PlayerResourcesView::new(&self.ram)
    }

    pub(crate) fn player_resources_view_mut(&mut self) -> PlayerResourcesViewMut<'_> {
        PlayerResourcesViewMut::new(&mut self.ram)
    }

    pub(crate) fn frame_control_view(&self) -> RamFrameControlView<'_> {
        RamFrameControlView::new(&self.ram)
    }

    pub(crate) fn frame_control_view_mut(&mut self) -> NativeFrameStateViewMut<'_> {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
    }

    pub(crate) fn native_frame_control_view(&self) -> NativeFrameStateView<'_> {
        NativeFrameStateView::new(&self.game_state.frame, &self.ram)
    }

    pub(crate) fn native_frame_control_view_mut(&mut self) -> NativeFrameStateViewMut<'_> {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
    }

    pub(crate) fn ram_frame_control_view(&self) -> RamFrameControlView<'_> {
        RamFrameControlView::new(&self.ram)
    }

    pub(crate) fn frame_state(&self) -> &FrameState {
        &self.game_state.frame
    }

    pub(crate) fn set_main_module(&mut self, value: u8) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .set_main_module(value);
    }

    pub(crate) fn set_main_module_word(&mut self, value: u16) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .set_main_module_word(value);
    }

    pub(crate) fn set_submodule(&mut self, value: u8) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .set_submodule(value);
    }

    pub(crate) fn increment_submodule(&mut self) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .increment_submodule();
    }

    pub(crate) fn decrement_submodule(&mut self) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .decrement_submodule();
    }

    pub(crate) fn set_subsubmodule(&mut self, value: u8) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .set_subsubmodule(value);
    }

    pub(crate) fn increment_subsubmodule(&mut self) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .increment_subsubmodule();
    }

    pub(crate) fn decrement_subsubmodule(&mut self) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .decrement_subsubmodule();
    }

    pub(crate) fn set_frame_counter(&mut self, value: u8) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .set_frame_counter(value);
    }

    pub(crate) fn increment_frame_counter(&mut self) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .increment_frame_counter();
    }

    pub(crate) fn set_saved_module_for_menu(&mut self, value: u8) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .set_saved_module_for_menu(value);
    }

    pub(crate) fn clear_saved_module_for_menu(&mut self) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .clear_saved_module_for_menu();
    }

    pub(crate) fn save_main_module_for_menu(&mut self) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .save_main_module_for_menu();
    }

    pub(crate) fn save_submodule_for_menu(&mut self) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .save_submodule_for_menu();
    }

    pub(crate) fn clear_modal_pause_flag(&mut self) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .clear_modal_pause_flag();
    }

    pub(crate) fn set_modal_pause_flag(&mut self, value: u8) {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .set_modal_pause_flag(value);
    }

    pub(crate) fn increment_modal_pause_flag(&mut self) -> u8 {
        NativeFrameStateViewMut::new(&mut self.game_state.frame, &mut self.ram)
            .increment_modal_pause_flag()
    }

    pub(crate) fn world_location_state(&self) -> &WorldLocationState {
        &self.game_state.world_location
    }

    pub(crate) fn set_dungeon_room(&mut self, value: u16) {
        NativeWorldLocationViewMut::new(&mut self.game_state.world_location, &mut self.ram)
            .set_dungeon_room(value);
    }

    pub(crate) fn set_dungeon_room_index(&mut self, value: u8) {
        NativeWorldLocationViewMut::new(&mut self.game_state.world_location, &mut self.ram)
            .set_dungeon_room_index(value);
    }

    pub(crate) fn increment_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        NativeWorldLocationViewMut::new(&mut self.game_state.world_location, &mut self.ram)
            .increment_dungeon_room_index_by(value)
    }

    pub(crate) fn decrement_dungeon_room_index_by(&mut self, value: u8) -> u8 {
        NativeWorldLocationViewMut::new(&mut self.game_state.world_location, &mut self.ram)
            .decrement_dungeon_room_index_by(value)
    }

    pub(crate) fn set_overworld_screen(&mut self, value: u8) {
        NativeWorldLocationViewMut::new(&mut self.game_state.world_location, &mut self.ram)
            .set_overworld_screen(value);
    }

    pub(crate) fn set_overworld_screen_word(&mut self, value: u16) {
        NativeWorldLocationViewMut::new(&mut self.game_state.world_location, &mut self.ram)
            .set_overworld_screen_word(value);
    }

    pub(crate) fn set_indoor_flag(&mut self, value: u8) {
        NativeWorldLocationViewMut::new(&mut self.game_state.world_location, &mut self.ram)
            .set_indoor_flag(value);
    }

    pub(crate) fn display_state(&self) -> &DisplayState {
        &self.game_state.display
    }

    pub(crate) fn set_screen_brightness(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_screen_brightness(value);
    }

    pub(crate) fn increment_screen_brightness(&mut self) -> u8 {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .increment_screen_brightness()
    }

    pub(crate) fn decrement_screen_brightness(&mut self) -> u8 {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .decrement_screen_brightness()
    }

    pub(crate) fn latch_nmi_update(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .latch_nmi_update();
    }

    pub(crate) fn clear_nmi_update_latch(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_nmi_update_latch();
    }

    pub(crate) fn set_core_update_disable_flag(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_core_update_disable_flag(value);
    }

    pub(crate) fn set_core_update_disable_flag_word(&mut self, value: u16) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_core_update_disable_flag_word(value);
    }

    pub(crate) fn clear_core_update_disable_flag(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_core_update_disable_flag();
    }

    pub(crate) fn increment_core_update_disable_flag(&mut self) -> u8 {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .increment_core_update_disable_flag()
    }

    pub(crate) fn set_pending_nmi_subroutine(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_pending_nmi_subroutine(value);
    }

    pub(crate) fn clear_pending_nmi_subroutine(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_pending_nmi_subroutine();
    }

    pub(crate) fn take_pending_nmi_subroutine(&mut self) -> u8 {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .take_pending_nmi_subroutine()
    }

    pub(crate) fn set_bg_vram_load_mode(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_bg_vram_load_mode(value);
    }

    pub(crate) fn queue_tilemap_update(&mut self, destination_page: u8, source_offset: u16) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .queue_tilemap_update(destination_page, source_offset);
    }

    pub(crate) fn clear_pending_tilemap_update_destination(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_pending_tilemap_update_destination();
    }

    pub(crate) fn set_bg_mode(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_bg_mode(value);
    }

    pub(crate) fn set_main_screen_layers(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_main_screen_layers(value);
    }

    pub(crate) fn and_main_screen_layers(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .and_main_screen_layers(value);
    }

    pub(crate) fn or_main_screen_layers(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .or_main_screen_layers(value);
    }

    pub(crate) fn set_sub_screen_layers(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_sub_screen_layers(value);
    }

    pub(crate) fn clear_sub_screen_layers_word(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_sub_screen_layers_word();
    }

    pub(crate) fn and_sub_screen_layers(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .and_sub_screen_layers(value);
    }

    pub(crate) fn or_sub_screen_layers(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .or_sub_screen_layers(value);
    }

    pub(crate) fn set_layer_masks_word(&mut self, value: u16) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_layer_masks_word(value);
    }

    pub(crate) fn set_bg12_window_selection(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_bg12_window_selection(value);
    }

    pub(crate) fn set_bg34_window_selection(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_bg34_window_selection(value);
    }

    pub(crate) fn set_object_color_window_selection(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_object_color_window_selection(value);
    }

    pub(crate) fn set_main_screen_window_layers(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_main_screen_window_layers(value);
    }

    pub(crate) fn set_sub_screen_window_layers(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_sub_screen_window_layers(value);
    }

    pub(crate) fn set_window_layer_masks(
        &mut self,
        bg12_window_selection: u8,
        bg34_window_selection: u8,
        object_color_window_selection: u8,
        main_screen_window_layers: u8,
        sub_screen_window_layers: u8,
    ) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_window_layer_masks(
                bg12_window_selection,
                bg34_window_selection,
                object_color_window_selection,
                main_screen_window_layers,
                sub_screen_window_layers,
            );
    }

    pub(crate) fn clear_window_layer_masks(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_window_layer_masks();
    }

    pub(crate) fn clear_window_main_sub_masks(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_window_main_sub_masks();
    }

    pub(crate) fn clear_bg_vram_load_mode(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_bg_vram_load_mode();
    }

    pub(crate) fn set_nmi_copy_packets_request(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_nmi_copy_packets_request(value);
    }

    pub(crate) fn request_nmi_copy_packets(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .request_nmi_copy_packets();
    }

    pub(crate) fn clear_nmi_copy_packets_request(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_nmi_copy_packets_request();
    }

    pub(crate) fn request_polyhedral_nmi_update(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .request_polyhedral_nmi_update();
    }

    pub(crate) fn clear_pending_polyhedral_update(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_pending_polyhedral_update();
    }

    pub(crate) fn set_chr_halfslot_request(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_chr_halfslot_request(value);
    }

    pub(crate) fn clear_chr_halfslot_request(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_chr_halfslot_request();
    }

    pub(crate) fn increment_chr_halfslot_request(&mut self) -> u8 {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .increment_chr_halfslot_request()
    }

    pub(crate) fn activate_nmi_thread(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .activate_nmi_thread();
    }

    pub(crate) fn deactivate_nmi_thread(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .deactivate_nmi_thread();
    }

    pub(crate) fn set_nmi_thread_stack_pointer(&mut self, value: u16) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_nmi_thread_stack_pointer(value);
    }

    pub(crate) fn set_irq_control_flag(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_irq_control_flag(value);
    }

    pub(crate) fn clear_irq_control_flag(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_irq_control_flag();
    }

    pub(crate) fn set_vertical_irq_trigger(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_vertical_irq_trigger(value);
    }

    pub(crate) fn set_sprite_dma_head_pointer(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_sprite_dma_head_pointer(value);
    }

    pub(crate) fn set_sprite_dma_body_pointer(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_sprite_dma_body_pointer(value);
    }

    pub(crate) fn set_hdma_enable_mask(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_hdma_enable_mask(value);
    }

    pub(crate) fn clear_hdma_enable_mask(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_hdma_enable_mask();
    }

    pub(crate) fn set_mosaic_copy(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_mosaic_copy(value);
    }

    pub(crate) fn set_mosaic_copy_from_level_or(&mut self, mask: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_mosaic_copy_from_level_or(mask);
    }

    pub(crate) fn set_mosaic_level(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_mosaic_level(value);
    }

    pub(crate) fn clear_mosaic_level(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_mosaic_level();
    }

    pub(crate) fn clear_mosaic_level_word(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_mosaic_level_word();
    }

    pub(crate) fn increment_mosaic_level_by(&mut self, value: u8) -> u8 {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .increment_mosaic_level_by(value)
    }

    pub(crate) fn decrement_mosaic_level_by(&mut self, value: u8) -> u8 {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .decrement_mosaic_level_by(value)
    }

    pub(crate) fn set_mosaic_target_level(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_mosaic_target_level(value);
    }

    pub(crate) fn set_mosaic_target_level_word(&mut self, value: u16) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_mosaic_target_level_word(value);
    }

    pub(crate) fn clear_mosaic_target_level(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_mosaic_target_level();
    }

    pub(crate) fn clear_mosaic_target_level_word(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_mosaic_target_level_word();
    }

    pub(crate) fn set_mosaic_direction(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_mosaic_direction(value);
    }

    pub(crate) fn clear_mosaic_direction(&mut self) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_mosaic_direction();
    }

    pub(crate) fn set_nmi_load_target_page(&mut self, value: u8) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_nmi_load_target_page(value);
    }

    pub(crate) fn set_nmi_load_target_address(&mut self, value: u16) {
        NativeDisplayStateViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_nmi_load_target_address(value);
    }

    pub(crate) fn world_state_view(&self) -> WorldStateView<'_> {
        WorldStateView::new(&self.ram)
    }

    pub(crate) fn world_state_view_mut(&mut self) -> NativeWorldLocationViewMut<'_> {
        NativeWorldLocationViewMut::new(&mut self.game_state.world_location, &mut self.ram)
    }

    pub(crate) fn dungeon_state_view(&self) -> DungeonStateView<'_> {
        DungeonStateView::new(&self.ram)
    }

    pub(crate) fn dungeon_state_view_mut(&mut self) -> DungeonStateViewMut<'_> {
        DungeonStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn save_progress_view(&self) -> SaveProgressView<'_> {
        SaveProgressView::new(&self.ram)
    }

    pub(crate) fn save_progress_view_mut(&mut self) -> SaveProgressViewMut<'_> {
        SaveProgressViewMut::new(&mut self.ram)
    }

    pub(crate) fn mirror_warp_scratch_view(&self) -> MirrorWarpScratchView<'_> {
        MirrorWarpScratchView::new(&self.ram)
    }

    pub(crate) fn mirror_warp_scratch_view_mut(&mut self) -> MirrorWarpScratchViewMut<'_> {
        MirrorWarpScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn dungeon_entrance_backup_view_mut(&mut self) -> DungeonEntranceBackupViewMut<'_> {
        DungeonEntranceBackupViewMut::new(&mut self.ram)
    }

    pub(crate) fn dungeon_header_view(&self) -> DungeonHeaderView<'_> {
        DungeonHeaderView::new(&self.ram)
    }

    pub(crate) fn dungeon_header_view_mut(&mut self) -> DungeonHeaderViewMut<'_> {
        DungeonHeaderViewMut::new(&mut self.ram)
    }

    pub(crate) fn dungeon_key_slots_view(&self) -> DungeonKeySlotsView<'_> {
        DungeonKeySlotsView::new(&self.ram)
    }

    pub(crate) fn dungeon_key_slots_view_mut(&mut self) -> DungeonKeySlotsViewMut<'_> {
        DungeonKeySlotsViewMut::new(&mut self.ram)
    }

    pub(crate) fn dungeon_torch_view(&self) -> DungeonTorchView<'_> {
        DungeonTorchView::new(&self.ram)
    }

    pub(crate) fn dungeon_torch_view_mut(&mut self) -> DungeonTorchViewMut<'_> {
        DungeonTorchViewMut::new(&mut self.ram)
    }

    pub(crate) fn dungeon_map_view_mut(&mut self) -> DungeonMapViewMut<'_> {
        DungeonMapViewMut::new(&mut self.ram)
    }

    pub(crate) fn scratch_word_view(&self) -> ScratchWordView<'_> {
        ScratchWordView::new(&self.ram)
    }

    pub(crate) fn scratch_word_view_mut(&mut self) -> ScratchWordViewMut<'_> {
        ScratchWordViewMut::new(&mut self.ram)
    }

    pub(crate) fn ending_scratch_view(&self) -> EndingScratchView<'_> {
        EndingScratchView::new(&self.ram)
    }

    pub(crate) fn ending_scratch_view_mut(&mut self) -> EndingScratchViewMut<'_> {
        EndingScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn save_load_scratch_view(&self) -> SaveLoadScratchView<'_> {
        SaveLoadScratchView::new(&self.ram)
    }

    pub(crate) fn save_load_scratch_view_mut(&mut self) -> SaveLoadScratchViewMut<'_> {
        SaveLoadScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn dungeon_map_scratch_view(&self) -> DungeonMapScratchView<'_> {
        DungeonMapScratchView::new(&self.ram)
    }

    pub(crate) fn dungeon_map_scratch_view_mut(&mut self) -> DungeonMapScratchViewMut<'_> {
        DungeonMapScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn dungeon_secret_scratch_view(&self) -> DungeonSecretScratchView<'_> {
        DungeonSecretScratchView::new(&self.ram)
    }

    pub(crate) fn dungeon_secret_scratch_view_mut(&mut self) -> DungeonSecretScratchViewMut<'_> {
        DungeonSecretScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn temp_counter_view(&self) -> TempCounterView<'_> {
        TempCounterView::new(&self.ram)
    }

    pub(crate) fn temp_counter_view_mut(&mut self) -> TempCounterViewMut<'_> {
        TempCounterViewMut::new(&mut self.ram)
    }

    pub(crate) fn overworld_event_info_view(&self) -> OverworldEventInfoView<'_> {
        OverworldEventInfoView::new(&self.ram)
    }

    pub(crate) fn overworld_event_info_view_mut(&mut self) -> OverworldEventInfoViewMut<'_> {
        OverworldEventInfoViewMut::new(&mut self.ram)
    }

    pub(crate) fn overworld_config_table_view(&self) -> OverworldConfigTableView<'_> {
        OverworldConfigTableView::new(&self.ram)
    }

    pub(crate) fn overworld_config_table_view_mut(&mut self) -> OverworldConfigTableViewMut<'_> {
        OverworldConfigTableViewMut::new(&mut self.ram)
    }

    pub(crate) fn palette_buffer_view(&self) -> PaletteBufferView<'_> {
        PaletteBufferView::new(&self.ram)
    }

    pub(crate) fn palette_buffer_view_mut(&mut self) -> PaletteBufferViewMut<'_> {
        PaletteBufferViewMut::new(&mut self.ram)
    }

    pub(crate) fn palette_filter_view(&self) -> PaletteFilterView<'_> {
        PaletteFilterView::new(&self.ram)
    }

    pub(crate) fn palette_filter_view_mut(&mut self) -> PaletteFilterViewMut<'_> {
        PaletteFilterViewMut::new(&mut self.ram)
    }

    pub(crate) fn hud_state_view(&self) -> HudStateView<'_> {
        HudStateView::new(&self.ram)
    }

    pub(crate) fn hud_state_view_mut(&mut self) -> HudStateViewMut<'_> {
        HudStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn hud_inventory_order_view(&self) -> HudInventoryOrderView<'_> {
        HudInventoryOrderView::new(&self.ram)
    }

    pub(crate) fn hud_inventory_order_view_mut(&mut self) -> HudInventoryOrderViewMut<'_> {
        HudInventoryOrderViewMut::new(&mut self.ram)
    }

    pub(crate) fn archery_game_view(&self) -> ArcheryGameView<'_> {
        ArcheryGameView::new(&self.ram)
    }

    pub(crate) fn archery_game_view_mut(&mut self) -> ArcheryGameViewMut<'_> {
        ArcheryGameViewMut::new(&mut self.ram)
    }

    pub(crate) fn minigame_state_view(&self) -> MinigameStateView<'_> {
        MinigameStateView::new(&self.ram)
    }

    pub(crate) fn minigame_state_view_mut(&mut self) -> MinigameStateViewMut<'_> {
        MinigameStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn sprite_battle_view(&self) -> SpriteBattleView<'_> {
        SpriteBattleView::new(&self.ram)
    }

    pub(crate) fn sprite_battle_view_mut(&mut self) -> SpriteBattleViewMut<'_> {
        SpriteBattleViewMut::new(&mut self.ram)
    }

    pub(crate) fn shared_message_timer_view(&self) -> SharedMessageTimerView<'_> {
        SharedMessageTimerView::new(&self.ram)
    }

    pub(crate) fn shared_message_timer_view_mut(&mut self) -> SharedMessageTimerViewMut<'_> {
        SharedMessageTimerViewMut::new(&mut self.ram)
    }

    pub(crate) fn intro_state_view(&self) -> IntroStateView<'_> {
        IntroStateView::new(&self.ram)
    }

    pub(crate) fn intro_state_view_mut(&mut self) -> IntroStateViewMut<'_> {
        IntroStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn ending_credit_state_view(&self) -> EndingCreditStateView<'_> {
        EndingCreditStateView::new(&self.ram)
    }

    pub(crate) fn ending_credit_state_view_mut(&mut self) -> EndingCreditStateViewMut<'_> {
        EndingCreditStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn intro_sword_view(&self) -> IntroSwordView<'_> {
        IntroSwordView::new(&self.ram)
    }

    pub(crate) fn intro_sword_view_mut(&mut self) -> IntroSwordViewMut<'_> {
        IntroSwordViewMut::new(&mut self.ram)
    }

    pub(crate) fn graphics_scratch_view_mut(&mut self) -> GraphicsScratchViewMut<'_> {
        GraphicsScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn overworld_screen_size_view(&self) -> OverworldScreenSizeView<'_> {
        OverworldScreenSizeView::new(&self.ram)
    }

    pub(crate) fn overworld_screen_size_view_mut(&mut self) -> OverworldScreenSizeViewMut<'_> {
        OverworldScreenSizeViewMut::new(&mut self.ram)
    }

    pub(crate) fn overworld_map16_decode_view(&self) -> OverworldMap16DecodeView<'_> {
        OverworldMap16DecodeView::new(&self.ram)
    }

    pub(crate) fn overworld_map16_decode_view_mut(&mut self) -> OverworldMap16DecodeViewMut<'_> {
        OverworldMap16DecodeViewMut::new(&mut self.ram)
    }

    pub(crate) fn room_bounds_view(&self) -> RoomBoundsView<'_> {
        RoomBoundsView::new(&self.ram)
    }

    pub(crate) fn room_bounds_view_mut(&mut self) -> RoomBoundsViewMut<'_> {
        RoomBoundsViewMut::new(&mut self.ram)
    }

    pub(crate) fn vram_upload_data_view(&self) -> VramUploadDataView<'_> {
        VramUploadDataView::new(&self.ram)
    }

    pub(crate) fn vram_upload_data_view_mut(&mut self) -> NativeVramUploadDataViewMut<'_> {
        NativeVramUploadDataViewMut::new(&mut self.game_state.display, &mut self.ram)
    }

    pub(crate) fn set_vram_upload_cursor(&mut self, value: u16) {
        NativeVramUploadDataViewMut::new(&mut self.game_state.display, &mut self.ram)
            .set_offset(value);
    }

    pub(crate) fn clear_vram_upload_cursor(&mut self) {
        NativeVramUploadDataViewMut::new(&mut self.game_state.display, &mut self.ram)
            .clear_offset();
    }

    pub(crate) fn poly_state_view(&self) -> PolyStateView<'_> {
        PolyStateView::new(&self.ram)
    }

    pub(crate) fn poly_state_view_mut(&mut self) -> PolyStateViewMut<'_> {
        PolyStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn poly_projected_vertex_view(&self) -> PolyProjectedVertexView<'_> {
        PolyProjectedVertexView::new(&self.ram)
    }

    pub(crate) fn poly_projected_vertex_view_mut(&mut self) -> PolyProjectedVertexViewMut<'_> {
        PolyProjectedVertexViewMut::new(&mut self.ram)
    }

    pub(crate) fn poly_face_coords_view(&self) -> PolyFaceCoordsView<'_> {
        PolyFaceCoordsView::new(&self.ram)
    }

    pub(crate) fn poly_face_coords_view_mut(&mut self) -> PolyFaceCoordsViewMut<'_> {
        PolyFaceCoordsViewMut::new(&mut self.ram)
    }

    pub(crate) fn poly_raster_edge_view(&self) -> PolyRasterEdgeView<'_> {
        PolyRasterEdgeView::new(&self.ram)
    }

    pub(crate) fn poly_raster_edge_view_mut(&mut self) -> PolyRasterEdgeViewMut<'_> {
        PolyRasterEdgeViewMut::new(&mut self.ram)
    }

    pub(crate) fn intro_actor_view(&self, slot: usize) -> IntroActorView<'_> {
        IntroActorView::new(&self.ram, slot)
    }

    pub(crate) fn intro_actor_view_mut(&mut self, slot: usize) -> IntroActorViewMut<'_> {
        IntroActorViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn effect_angle_scratch_view(&self) -> EffectAngleScratchView<'_> {
        EffectAngleScratchView::new(&self.ram)
    }

    pub(crate) fn effect_angle_scratch_view_mut(&mut self) -> EffectAngleScratchViewMut<'_> {
        EffectAngleScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn quake_bolt_view(&self, slot: usize) -> QuakeBoltView<'_> {
        QuakeBoltView::new(&self.ram, slot)
    }

    pub(crate) fn quake_bolt_view_mut(&mut self, slot: usize) -> QuakeBoltViewMut<'_> {
        QuakeBoltViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn quake_spell_scratch_view(&self) -> QuakeSpellScratchView<'_> {
        QuakeSpellScratchView::new(&self.ram)
    }

    pub(crate) fn quake_spell_scratch_view_mut(&mut self) -> QuakeSpellScratchViewMut<'_> {
        QuakeSpellScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn bombos_fire_column_view(&self, slot: usize) -> BombosFireColumnView<'_> {
        BombosFireColumnView::new(&self.ram, slot)
    }

    pub(crate) fn bombos_fire_column_view_mut(
        &mut self,
        slot: usize,
    ) -> BombosFireColumnViewMut<'_> {
        BombosFireColumnViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn bombos_blast_view(&self, slot: usize) -> BombosBlastView<'_> {
        BombosBlastView::new(&self.ram, slot)
    }

    pub(crate) fn bombos_blast_view_mut(&mut self, slot: usize) -> BombosBlastViewMut<'_> {
        BombosBlastViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn bombos_spell_scratch_view(&self) -> BombosSpellScratchView<'_> {
        BombosSpellScratchView::new(&self.ram)
    }

    pub(crate) fn bombos_spell_scratch_view_mut(&mut self) -> BombosSpellScratchViewMut<'_> {
        BombosSpellScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn tower_seal_orbit_view(&self, slot: usize) -> TowerSealOrbitView<'_> {
        TowerSealOrbitView::new(&self.ram, slot)
    }

    pub(crate) fn tower_seal_orbit_view_mut(&mut self, slot: usize) -> TowerSealOrbitViewMut<'_> {
        TowerSealOrbitViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn tower_seal_sparkle_view(&self, slot: usize) -> TowerSealSparkleView<'_> {
        TowerSealSparkleView::new(&self.ram, slot)
    }

    pub(crate) fn tower_seal_sparkle_view_mut(
        &mut self,
        slot: usize,
    ) -> TowerSealSparkleViewMut<'_> {
        TowerSealSparkleViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn tower_seal_scratch_view(&self) -> TowerSealScratchView<'_> {
        TowerSealScratchView::new(&self.ram)
    }

    pub(crate) fn tower_seal_scratch_view_mut(&mut self) -> TowerSealScratchViewMut<'_> {
        TowerSealScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn blast_wall_explosion_view(&self, slot: usize) -> BlastWallExplosionView<'_> {
        BlastWallExplosionView::new(&self.ram, slot)
    }

    pub(crate) fn blast_wall_explosion_view_mut(
        &mut self,
        slot: usize,
    ) -> BlastWallExplosionViewMut<'_> {
        BlastWallExplosionViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn blast_wall_fragment_view(&self, slot: usize) -> BlastWallFragmentView<'_> {
        BlastWallFragmentView::new(&self.ram, slot)
    }

    pub(crate) fn blast_wall_fragment_view_mut(
        &mut self,
        slot: usize,
    ) -> BlastWallFragmentViewMut<'_> {
        BlastWallFragmentViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn blast_wall_fireball_view(&self, slot: usize) -> BlastWallFireballView<'_> {
        BlastWallFireballView::new(&self.ram, slot)
    }

    pub(crate) fn blast_wall_fireball_view_mut(
        &mut self,
        slot: usize,
    ) -> BlastWallFireballViewMut<'_> {
        BlastWallFireballViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn blast_wall_scratch_view(&self) -> BlastWallScratchView<'_> {
        BlastWallScratchView::new(&self.ram)
    }

    pub(crate) fn blast_wall_scratch_view_mut(&mut self) -> BlastWallScratchViewMut<'_> {
        BlastWallScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn skull_woods_fire_view(&self, slot: usize) -> SkullWoodsFireView<'_> {
        SkullWoodsFireView::new(&self.ram, slot)
    }

    pub(crate) fn skull_woods_fire_view_mut(&mut self, slot: usize) -> SkullWoodsFireViewMut<'_> {
        SkullWoodsFireViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn skull_woods_fire_scratch_view(&self) -> SkullWoodsFireScratchView<'_> {
        SkullWoodsFireScratchView::new(&self.ram)
    }

    pub(crate) fn skull_woods_fire_scratch_view_mut(&mut self) -> SkullWoodsFireScratchViewMut<'_> {
        SkullWoodsFireScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn happiness_pond_rupee_view(&self, slot: usize) -> HappinessPondRupeeView<'_> {
        HappinessPondRupeeView::new(&self.ram, slot)
    }

    pub(crate) fn happiness_pond_rupee_view_mut(
        &mut self,
        slot: usize,
    ) -> HappinessPondRupeeViewMut<'_> {
        HappinessPondRupeeViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn weather_vane_state_view(&self) -> WeatherVaneStateView<'_> {
        WeatherVaneStateView::new(&self.ram)
    }

    pub(crate) fn weather_vane_state_view_mut(&mut self) -> WeatherVaneStateViewMut<'_> {
        WeatherVaneStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn weather_vane_debris_view(&self, slot: usize) -> WeatherVaneDebrisView<'_> {
        WeatherVaneDebrisView::new(&self.ram, slot)
    }

    pub(crate) fn weather_vane_debris_view_mut(
        &mut self,
        slot: usize,
    ) -> WeatherVaneDebrisViewMut<'_> {
        WeatherVaneDebrisViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn bird_travel_destination_view(
        &self,
        slot: usize,
    ) -> BirdTravelDestinationView<'_> {
        BirdTravelDestinationView::new(&self.ram, slot)
    }

    pub(crate) fn bird_travel_destination_view_mut(
        &mut self,
        slot: usize,
    ) -> BirdTravelDestinationViewMut<'_> {
        BirdTravelDestinationViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn bird_travel_status_view_mut(&mut self) -> BirdTravelStatusViewMut<'_> {
        BirdTravelStatusViewMut::new(&mut self.ram)
    }

    pub(crate) fn moldorm_history_view(&self, slot: usize) -> MoldormHistoryView<'_> {
        MoldormHistoryView::new(&self.ram, slot)
    }

    pub(crate) fn moldorm_history_view_mut(&mut self, slot: usize) -> MoldormHistoryViewMut<'_> {
        MoldormHistoryViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn swamola_target_view(&self, slot: usize) -> SwamolaTargetView<'_> {
        SwamolaTargetView::new(&self.ram, slot)
    }

    pub(crate) fn swamola_target_view_mut(&mut self, slot: usize) -> SwamolaTargetViewMut<'_> {
        SwamolaTargetViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn swamola_history_view(&self, slot: usize) -> SwamolaHistoryView<'_> {
        SwamolaHistoryView::new(&self.ram, slot)
    }

    pub(crate) fn swamola_history_view_mut(&mut self, slot: usize) -> SwamolaHistoryViewMut<'_> {
        SwamolaHistoryViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn beamos_laser_history_view(&self, slot: usize) -> BeamosLaserHistoryView<'_> {
        BeamosLaserHistoryView::new(&self.ram, slot)
    }

    pub(crate) fn beamos_laser_history_view_mut(
        &mut self,
        slot: usize,
    ) -> BeamosLaserHistoryViewMut<'_> {
        BeamosLaserHistoryViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn lanmola_segment_motion_view(&self, slot: usize) -> LanmolaSegmentMotionView<'_> {
        LanmolaSegmentMotionView::new(&self.ram, slot)
    }

    pub(crate) fn lanmola_segment_motion_view_mut(
        &mut self,
        slot: usize,
    ) -> LanmolaSegmentMotionViewMut<'_> {
        LanmolaSegmentMotionViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn door_debris_view(&self) -> DoorDebrisView<'_> {
        DoorDebrisView::new(&self.ram)
    }

    pub(crate) fn door_debris_view_mut(&mut self) -> DoorDebrisViewMut<'_> {
        DoorDebrisViewMut::new(&mut self.ram)
    }

    pub(crate) fn digging_game_prize_view(&self) -> DiggingGamePrizeView<'_> {
        DiggingGamePrizeView::new(&self.ram)
    }

    pub(crate) fn digging_game_prize_view_mut(&mut self) -> DiggingGamePrizeViewMut<'_> {
        DiggingGamePrizeViewMut::new(&mut self.ram)
    }

    pub(crate) fn draw_scratch_position_view(&self) -> DrawScratchPositionView<'_> {
        DrawScratchPositionView::new(&self.ram)
    }

    pub(crate) fn draw_scratch_position_view_mut(&mut self) -> DrawScratchPositionViewMut<'_> {
        DrawScratchPositionViewMut::new(&mut self.ram)
    }

    pub(crate) fn hitbox_scratch_offset_view(&self) -> HitboxScratchOffsetView<'_> {
        HitboxScratchOffsetView::new(&self.ram)
    }

    pub(crate) fn hitbox_scratch_offset_view_mut(&mut self) -> HitboxScratchOffsetViewMut<'_> {
        HitboxScratchOffsetViewMut::new(&mut self.ram)
    }

    pub(crate) fn overworld_scroll_delta_view(&self) -> OverworldScrollDeltaView<'_> {
        OverworldScrollDeltaView::new(&self.ram)
    }

    pub(crate) fn overworld_scroll_delta_view_mut(&mut self) -> OverworldScrollDeltaViewMut<'_> {
        OverworldScrollDeltaViewMut::new(&mut self.ram)
    }

    pub(crate) fn dialogue_number_view_mut(&mut self) -> DialogueNumberViewMut<'_> {
        DialogueNumberViewMut::new(&mut self.ram)
    }

    pub(crate) fn dialogue_number_view(&self) -> DialogueNumberView<'_> {
        DialogueNumberView::new(&self.ram)
    }

    pub(crate) fn messaging_state_view(&self) -> MessagingStateView<'_> {
        MessagingStateView::new(&self.ram)
    }

    pub(crate) fn messaging_state_view_mut(&mut self) -> MessagingStateViewMut<'_> {
        MessagingStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn messaging_text_view(&self) -> MessagingTextView<'_> {
        MessagingTextView::new(&self.ram)
    }

    pub(crate) fn messaging_text_view_mut(&mut self) -> MessagingTextViewMut<'_> {
        MessagingTextViewMut::new(&mut self.ram)
    }

    pub(crate) fn messaging_render_buffer_view(&self) -> MessagingRenderBufferView<'_> {
        MessagingRenderBufferView::new(&self.ram)
    }

    pub(crate) fn messaging_render_buffer_view_mut(&mut self) -> MessagingRenderBufferViewMut<'_> {
        MessagingRenderBufferViewMut::new(&mut self.ram)
    }

    pub(crate) fn vwf_glyph_spacing_view(&self) -> VwfGlyphSpacingView<'_> {
        VwfGlyphSpacingView::new(&self.ram)
    }

    pub(crate) fn vwf_glyph_spacing_view_mut(&mut self) -> VwfGlyphSpacingViewMut<'_> {
        VwfGlyphSpacingViewMut::new(&mut self.ram)
    }

    pub(crate) fn dialogue_source_offset_view_mut(&mut self) -> DialogueSourceOffsetViewMut<'_> {
        DialogueSourceOffsetViewMut::new(&mut self.ram)
    }

    pub(crate) fn select_file_scratch_view(&self) -> SelectFileScratchView<'_> {
        SelectFileScratchView::new(&self.ram)
    }

    pub(crate) fn select_file_scratch_view_mut(&mut self) -> SelectFileScratchViewMut<'_> {
        SelectFileScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn arrghus_puff_home_view(&self, puff_slot: usize) -> ArrghusPuffHomeView<'_> {
        ArrghusPuffHomeView::new(&self.ram, puff_slot)
    }

    pub(crate) fn armos_knight_home_view(&self, slot: usize) -> ArmosKnightHomeView<'_> {
        ArmosKnightHomeView::new(&self.ram, slot)
    }

    pub(crate) fn armos_knight_home_view_mut(&mut self, slot: usize) -> ArmosKnightHomeViewMut<'_> {
        ArmosKnightHomeViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn alt_sprite_slot_view_mut(&mut self, slot: usize) -> AltSpriteSlotViewMut<'_> {
        AltSpriteSlotViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn cached_sprite_slot_view(&self, slot: usize) -> CachedSpriteSlotView<'_> {
        CachedSpriteSlotView::new(&self.ram, slot)
    }

    pub(crate) fn cached_sprite_slot_view_mut(
        &mut self,
        slot: usize,
    ) -> CachedSpriteSlotViewMut<'_> {
        CachedSpriteSlotViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn tagalong_slot_view(&self, slot: usize) -> TagalongSlotView<'_> {
        TagalongSlotView::new(&self.ram, slot)
    }

    pub(crate) fn tagalong_slot_view_mut(&mut self, slot: usize) -> TagalongSlotViewMut<'_> {
        TagalongSlotViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn follower_state_view(&self) -> FollowerStateView<'_> {
        FollowerStateView::new(&self.ram)
    }

    pub(crate) fn follower_state_view_mut(&mut self) -> FollowerStateViewMut<'_> {
        FollowerStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn chain_chomp_history_view(&self) -> ChainChompHistoryView<'_> {
        ChainChompHistoryView::new(&self.ram)
    }

    pub(crate) fn chain_chomp_history_view_mut(&mut self) -> ChainChompHistoryViewMut<'_> {
        ChainChompHistoryViewMut::new(&mut self.ram)
    }

    pub(crate) fn ancilla_spawn_scratch_view_mut(&mut self) -> AncillaSpawnScratchViewMut<'_> {
        AncillaSpawnScratchViewMut::new(&mut self.ram)
    }

    pub(crate) fn maze_game_timer_view(&self) -> MazeGameTimerView<'_> {
        MazeGameTimerView::new(&self.ram)
    }

    pub(crate) fn maze_game_timer_view_mut(&mut self) -> MazeGameTimerViewMut<'_> {
        MazeGameTimerViewMut::new(&mut self.ram)
    }

    pub(crate) fn enemy_damage_data_view(&self) -> EnemyDamageDataView<'_> {
        EnemyDamageDataView::new(&self.ram)
    }

    pub(crate) fn enemy_damage_data_view_mut(&mut self) -> EnemyDamageDataViewMut<'_> {
        EnemyDamageDataViewMut::new(&mut self.ram)
    }

    pub(crate) fn ether_orbit_view(&self) -> EtherOrbitView<'_> {
        EtherOrbitView::new(&self.ram)
    }

    pub(crate) fn ether_orbit_view_mut(&mut self) -> EtherOrbitViewMut<'_> {
        EtherOrbitViewMut::new(&mut self.ram)
    }

    pub(crate) fn prize_drop_cycle_view_mut(&mut self) -> PrizeDropCycleViewMut<'_> {
        PrizeDropCycleViewMut::new(&mut self.ram)
    }

    pub(crate) fn dual_layer_tile_cache_view(&self) -> DualLayerTileCacheView<'_> {
        DualLayerTileCacheView::new(&self.ram)
    }

    pub(crate) fn dual_layer_tile_cache_view_mut(&mut self) -> DualLayerTileCacheViewMut<'_> {
        DualLayerTileCacheViewMut::new(&mut self.ram)
    }

    pub(crate) fn sprite_slot_view(&self, slot: usize) -> SpriteSlotView<'_> {
        SpriteSlotView::new(&self.ram, slot)
    }

    pub(crate) fn sprite_slot_view_mut(&mut self, slot: usize) -> SpriteSlotViewMut<'_> {
        SpriteSlotViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn sprite_system_view(&self) -> SpriteSystemView<'_> {
        SpriteSystemView::new(&self.ram)
    }

    pub(crate) fn sprite_system_view_mut(&mut self) -> SpriteSystemViewMut<'_> {
        SpriteSystemViewMut::new(&mut self.ram)
    }

    pub(crate) fn sprite_workspace_view(&self) -> SpriteWorkspaceView<'_> {
        SpriteWorkspaceView::new(&self.ram)
    }

    pub(crate) fn sprite_workspace_view_mut(&mut self) -> SpriteWorkspaceViewMut<'_> {
        SpriteWorkspaceViewMut::new(&mut self.ram)
    }

    pub(crate) fn ancilla_slot_view(&self, slot: usize) -> AncillaSlotView<'_> {
        AncillaSlotView::new(&self.ram, slot)
    }

    pub(crate) fn ancilla_slot_view_mut(&mut self, slot: usize) -> AncillaSlotViewMut<'_> {
        AncillaSlotViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn overlord_slot_view(&self, slot: usize) -> OverlordSlotView<'_> {
        OverlordSlotView::new(&self.ram, slot)
    }

    pub(crate) fn overlord_slot_view_mut(&mut self, slot: usize) -> OverlordSlotViewMut<'_> {
        OverlordSlotViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn garnish_slot_view(&self, slot: usize) -> GarnishSlotView<'_> {
        GarnishSlotView::new(&self.ram, slot)
    }

    pub(crate) fn garnish_slot_view_mut(&mut self, slot: usize) -> GarnishSlotViewMut<'_> {
        GarnishSlotViewMut::new(&mut self.ram, slot)
    }

    pub(crate) fn garnish_state_view(&self) -> GarnishStateView<'_> {
        GarnishStateView::new(&self.ram)
    }

    pub(crate) fn garnish_state_view_mut(&mut self) -> GarnishStateViewMut<'_> {
        GarnishStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn oam_state_view(&self) -> OamStateView<'_> {
        OamStateView::new(&self.ram)
    }

    pub(crate) fn oam_state_view_mut(&mut self) -> OamStateViewMut<'_> {
        OamStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn overworld_sprite_presence_view(&self) -> OverworldSpritePresenceView<'_> {
        OverworldSpritePresenceView::new(&self.ram)
    }

    pub(crate) fn overworld_sprite_presence_view_mut(
        &mut self,
    ) -> OverworldSpritePresenceViewMut<'_> {
        OverworldSpritePresenceViewMut::new(&mut self.ram)
    }

    pub(crate) fn memorized_tile_view(&self) -> MemorizedTileView<'_> {
        MemorizedTileView::new(&self.ram)
    }

    pub(crate) fn memorized_tile_view_mut(&mut self) -> MemorizedTileViewMut<'_> {
        MemorizedTileViewMut::new(&mut self.ram)
    }

    pub(crate) fn overworld_sprite_loaded_view(&self) -> OverworldSpriteLoadedView<'_> {
        OverworldSpriteLoadedView::new(&self.ram)
    }

    pub(crate) fn overworld_sprite_loaded_view_mut(&mut self) -> OverworldSpriteLoadedViewMut<'_> {
        OverworldSpriteLoadedViewMut::new(&mut self.ram)
    }

    pub(crate) fn vram_load_state_view(&self) -> VramLoadStateView<'_> {
        VramLoadStateView::new(&self.ram)
    }

    pub(crate) fn vram_load_state_view_mut(&mut self) -> VramLoadStateViewMut<'_> {
        VramLoadStateViewMut::new(&mut self.ram)
    }

    pub(crate) fn overworld_tile_update_view(&self) -> OverworldTileUpdateView<'_> {
        OverworldTileUpdateView::new(&self.ram)
    }

    pub(crate) fn overworld_tile_update_view_mut(&mut self) -> OverworldTileUpdateViewMut<'_> {
        OverworldTileUpdateViewMut::new(&mut self.ram)
    }

    pub(crate) fn star_tile_view(&self) -> StarTileView<'_> {
        StarTileView::new(&self.ram)
    }

    pub(crate) fn star_tile_view_mut(&mut self) -> StarTileViewMut<'_> {
        StarTileViewMut::new(&mut self.ram)
    }

    pub(crate) fn trinexx_palette_view(&self) -> TrinexxPaletteView<'_> {
        TrinexxPaletteView::new(&self.ram)
    }

    pub(crate) fn trinexx_palette_view_mut(&mut self) -> TrinexxPaletteViewMut<'_> {
        TrinexxPaletteViewMut::new(&mut self.ram)
    }

    pub(crate) fn spotlight_hdma_view(&self) -> SpotlightHdmaView<'_> {
        SpotlightHdmaView::new(&self.ram)
    }

    pub(crate) fn spotlight_hdma_view_mut(&mut self) -> SpotlightHdmaViewMut<'_> {
        SpotlightHdmaViewMut::new(&mut self.ram)
    }

    pub(crate) fn water_hdma_window_view(&self) -> WaterHdmaWindowView<'_> {
        WaterHdmaWindowView::new(&self.ram)
    }

    pub(crate) fn water_hdma_window_view_mut(&mut self) -> WaterHdmaWindowViewMut<'_> {
        WaterHdmaWindowViewMut::new(&mut self.ram)
    }

    pub(crate) fn overworld_palette_backup_view_mut(
        &mut self,
    ) -> OverworldPaletteBackupViewMut<'_> {
        OverworldPaletteBackupViewMut::new(&mut self.ram)
    }

    pub fn overworld_map16_load_state(&self) -> OverworldMap16LoadState {
        self.overworld_map16_load
    }

    pub fn set_overworld_map16_load_state(&mut self, state: OverworldMap16LoadState) {
        self.overworld_map16_load = state;
    }

    pub fn overworld_prev_map16_load_state(&self) -> OverworldMap16LoadState {
        self.overworld_prev_map16_load
    }

    pub fn set_overworld_prev_map16_load_state(&mut self, state: OverworldMap16LoadState) {
        self.overworld_prev_map16_load = state;
    }

    pub fn overworld_spexit_map16_src_off(&self) -> u16 {
        self.overworld_spexit_map16_src_off
    }

    pub fn set_overworld_spexit_map16_src_off(&mut self, src_off: u16) {
        self.overworld_spexit_map16_src_off = src_off;
    }

    pub fn overworld_exit_map16_src_off(&self) -> u16 {
        self.overworld_exit_map16_src_off
    }

    pub fn set_overworld_exit_map16_src_off(&mut self, src_off: u16) {
        self.overworld_exit_map16_src_off = src_off;
    }

    pub fn small_overworld_map16_scroll_backup_state(
        &self,
    ) -> SmallOverworldMap16ScrollBackupState {
        self.small_overworld_map16_scroll_backup
    }

    pub fn set_small_overworld_map16_scroll_backup_state(
        &mut self,
        state: SmallOverworldMap16ScrollBackupState,
    ) {
        self.small_overworld_map16_scroll_backup = state;
    }

    pub(crate) fn sync_native_game_state_from_ram(&mut self) {
        self.game_state = GameState::load_from_ram(&self.ram);
    }

    pub(crate) fn write_native_game_state_to_ram(&mut self) {
        self.game_state.write_to_ram(&mut self.ram);
    }

    #[track_caller]
    pub(crate) fn assert_native_frame_state_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.frame,
            crate::game_state::FrameState::load_from_ram(&self.ram),
            "native frame state diverged from RAM projection",
        );
    }

    #[track_caller]
    pub(crate) fn assert_native_world_location_state_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.world_location,
            crate::game_state::WorldLocationState::load_from_ram(&self.ram),
            "native world location state diverged from RAM projection",
        );
    }

    #[track_caller]
    pub(crate) fn assert_native_display_state_matches_ram(&self) {
        debug_assert_eq!(
            self.game_state.display,
            crate::game_state::DisplayState::load_from_ram(&self.ram),
            "native display state diverged from RAM projection",
        );
    }

    pub fn sync_overworld_map16_load_from_ram(&mut self) {
        self.overworld_map16_load = OverworldMap16LoadState {
            src_off: self
                .native_ram_bridge_view()
                .word_at(SEMANTIC_MAP16_LOAD_SRC_OFF),
            dst_off: self
                .native_ram_bridge_view()
                .word_at(SEMANTIC_MAP16_LOAD_DST_OFF),
            y_unit: self
                .native_ram_bridge_view()
                .word_at(SEMANTIC_MAP16_LOAD_Y_UNIT),
        };
        self.overworld_prev_map16_load = OverworldMap16LoadState {
            src_off: self
                .native_ram_bridge_view()
                .word_at(SEMANTIC_MAP16_LOAD_SRC_OFF_PREV),
            dst_off: self
                .native_ram_bridge_view()
                .word_at(SEMANTIC_MAP16_LOAD_DST_OFF_PREV),
            y_unit: self
                .native_ram_bridge_view()
                .word_at(SEMANTIC_MAP16_LOAD_Y_UNIT_PREV),
        };
        self.overworld_spexit_map16_src_off = self
            .native_ram_bridge_view()
            .word_at(SEMANTIC_MAP16_LOAD_SRC_OFF_SPEXIT);
        self.overworld_exit_map16_src_off = self
            .native_ram_bridge_view()
            .word_at(SEMANTIC_MAP16_LOAD_SRC_OFF_EXIT);
        self.small_overworld_map16_scroll_backup = SmallOverworldMap16ScrollBackupState {
            src_off: self
                .native_ram_bridge_view()
                .word_at(SEMANTIC_SMALL_OW_MAP16_SRC_OFF_BACKUP),
            dst_off: self
                .native_ram_bridge_view()
                .word_at(SEMANTIC_SMALL_OW_MAP16_DST_OFF_BACKUP),
            y_unit: self
                .native_ram_bridge_view()
                .word_at(SEMANTIC_SMALL_OW_MAP16_Y_UNIT_BACKUP),
        };
    }

    pub fn new() -> Self {
        let mut state = Self {
            ram: vec![0; WRAM_SIZE],
            game_state: GameState::default(),
            sram: vec![0; SRAM_SIZE],
            ppu: PpuState::new(),
            dma: DmaState::new(),
            frame_ctr_dbg: 0,
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
            intro_startup_delay: 0,
            rom_reset_frame_delay: 0,
            intro_memory_darken_frame_delay: 0,
            intro_poly_upload_delay: 0,
            intro_sprite_animation_start_delay: 0,
            display_snapshot: None,
            visible_display_snapshot: None,
            nmi_poly_upload_deferred: 0,
            nmi_poly_upload_started: false,
            nmi_poly_deferred_upload_bypasses_latch: false,
            nmi_poly_upload_from_deferred: false,
            obj_vram_latch_generation: 0,
            bsnes_poly_scheduler_counter: 0,
            bsnes_hold_intro_step_this_frame: false,
            bsnes_intro_step_carry_phase_active: false,
            bsnes_intro_step_hold_alternate: false,
            replay_reload_file_select_stall: 0,
            replay_loadfile_stall: 0,
            overworld_map16_load: OverworldMap16LoadState::default(),
            overworld_prev_map16_load: OverworldMap16LoadState::default(),
            overworld_spexit_map16_src_off: 0,
            overworld_exit_map16_src_off: 0,
            small_overworld_map16_scroll_backup: SmallOverworldMap16ScrollBackupState::default(),
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
        self.dma.reset();
        self.ppu.reset();
        self.ram.fill(0);
        if !preserve_sram {
            self.sram.fill(0);
        }
        self.zelda_restore_music_after_load_locked(true);
        self.initialized = true;
        self.apply_links_movement_to_camera_called = false;
        self.intro_startup_delay = 0;
        self.rom_reset_frame_delay = if self.rom_startup_timing {
            configured_rom_reset_frame_delay()
        } else {
            0
        };
        self.intro_memory_darken_frame_delay = 0;
        self.intro_sprite_animation_start_delay = 0;
        self.nmi_poly_upload_deferred = 0;
        self.nmi_poly_upload_started = false;
        self.nmi_poly_deferred_upload_bypasses_latch = false;
        self.nmi_poly_upload_from_deferred = false;
        self.obj_vram_latch_generation = 0;
        self.bsnes_poly_scheduler_counter = 0;
        self.bsnes_hold_intro_step_this_frame = false;
        self.bsnes_intro_step_carry_phase_active = false;
        self.bsnes_intro_step_hold_alternate = false;
        self.intro_poly_vram_history.clear();
        self.intro_poly_presented_vram = None;
        self.sync_overworld_map16_load_from_ram();
        self.display_snapshot = None;
        self.visible_display_snapshot = None;
        self.emu_synchronize_whole_state();
    }

    pub fn set_rom_startup_timing(&mut self, enabled: bool) {
        self.rom_startup_timing = enabled;
        self.zelda_set_rom_startup_audio_phase(enabled);
        if !enabled {
            self.intro_startup_delay = 0;
            self.rom_reset_frame_delay = 0;
            self.intro_memory_darken_frame_delay = 0;
            self.intro_sprite_animation_start_delay = 0;
            self.nmi_poly_upload_deferred = 0;
            self.nmi_poly_upload_started = false;
            self.nmi_poly_deferred_upload_bypasses_latch = false;
            self.nmi_poly_upload_from_deferred = false;
            self.obj_vram_latch_generation = 0;
            self.bsnes_poly_scheduler_counter = 0;
            self.bsnes_hold_intro_step_this_frame = false;
            self.bsnes_intro_step_carry_phase_active = false;
            self.bsnes_intro_step_hold_alternate = false;
            self.intro_poly_vram_history.clear();
            self.intro_poly_presented_vram = None;
            self.display_snapshot = None;
            self.visible_display_snapshot = None;
        } else if self
            .native_ram_bridge_view()
            .word_at(ANIMATED_TILE_DATA_SRC)
            == 0
        {
            self.rom_reset_frame_delay = configured_rom_reset_frame_delay();
        }
    }

    pub(super) fn rom_startup_timing(&self) -> bool {
        self.rom_startup_timing
    }

    pub(super) fn capture_display_snapshot(&mut self) {
        let snapshot = Box::new(DisplaySnapshot {
            ram: self.ram.clone(),
            ppu: self.ppu.clone(),
            dma: self.dma.clone(),
            intro_poly_upload_delay: self.intro_poly_upload_delay,
            intro_sprite_animation_start_delay: self.intro_sprite_animation_start_delay,
            rom_reset_frame_delay: self.rom_reset_frame_delay,
            intro_memory_darken_frame_delay: self.intro_memory_darken_frame_delay,
            nmi_poly_upload_deferred: self.nmi_poly_upload_deferred,
            obj_vram_latch_generation: self.obj_vram_latch_generation,
            bsnes_poly_scheduler_counter: self.bsnes_poly_scheduler_counter,
        });
        let frame = self.frame_state();
        if frame.main_module == 0 && matches!(frame.submodule, 3 | 4) {
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
        self.display_snapshot = Some(snapshot);
    }

    pub fn vram(&self) -> &[u16] {
        &self.ppu.vram
    }

    pub fn vram_mut(&mut self) -> &mut [u16] {
        &mut self.ppu.vram
    }

    pub fn set_rom(&mut self, rom: &[u8]) {
        self.rom = strip_copier_header(rom).to_vec();
    }

    pub fn set_assets(&mut self, assets: &[u8]) -> Result<(), String> {
        self.assets = Some(AssetPack::parse(assets)?);
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
        self.replay_trace_col("run-frame-entry");
        self.replay_trace_ram_watch("run-frame-entry");
        if !self.initialized {
            self.zelda_initialize();
        }
        if self.rom_startup_timing() && self.rom_reset_frame_delay != 0 {
            self.rom_reset_frame_delay = self.rom_reset_frame_delay.saturating_sub(1);
            self.capture_display_snapshot();
            return;
        }
        if self
            .native_ram_bridge_view()
            .word_at(ANIMATED_TILE_DATA_SRC)
            == 0
        {
            self.zelda_initialization_code();
        }
        if run_what & crate::RUN_POLY != 0 {
            self.zelda_run_poly_loop();
        }
        if self.rom_startup_timing() && self.intro_memory_darken_frame_delay != 0 {
            let frame = self.frame_state();
            if frame.main_module == 0 && frame.submodule == 3 {
                self.intro_animate_triforce();
            }
            self.intro_memory_darken_frame_delay =
                self.intro_memory_darken_frame_delay.saturating_sub(1);
            if self.intro_memory_darken_frame_delay == 0 {
                self.intro_initialize_memory_darken_finish();
            }
            self.capture_display_snapshot();
            self.interrupt_nmi(input);
            return;
        }
        let frame = *self.frame_state();
        if self.rom_startup_timing
            && frame.main_module == 0
            && matches!(frame.submodule, 3 | 4)
            && frame.frame_counter >= 0x85
            && self.display_state().has_pending_polyhedral_update()
        {
            self.nmi_poly_upload_from_deferred = true;
            self.nmi_update_irqgfx();
        }
        if run_what & crate::RUN_MAIN != 0 {
            self.replay_trace_col("before-game-loop");
            self.replay_trace_ram_watch("before-game-loop");
            self.zelda_run_game_loop();
            self.replay_trace_col("after-game-loop");
            self.replay_trace_ram_watch("after-game-loop");
        }
        if self.rom_startup_timing
            && frame.main_module == 0
            && matches!(frame.submodule, 3 | 4)
            && frame.frame_counter >= 0x86
        {
            self.zelda_run_poly_loop();
        }
        self.capture_display_snapshot();
        self.replay_trace_col("before-nmi");
        self.replay_trace_ram_watch("before-nmi");
        self.interrupt_nmi(input);
        self.replay_trace_col("after-nmi");
        self.replay_trace_ram_watch("after-nmi");
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        self.sync_overworld_map16_load_from_ram();
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
        let bytes = self.native_ram_bridge_view().range(offset, n).to_vec();
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
            0x0000e2 => Some(self.ram_bytes(PpuScrollCopyView::bg2_h_copy2_offset(), 2)),
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
        for i in 0..8 {
            self.dma.channel[i].hdma_active = self.display_state().is_hdma_channel_enabled(i);
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

        let channels: [_; 8] = std::array::from_fn(|i| self.dma.channel[i]);
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
    pub fn ppu_scanline_windows(
        &mut self,
    ) -> Box<[(u8, u8, u8, u8, u8, [u16; 4], [u16; 4], [i16; 8]); 224]> {
        let saved_channels: [_; 8] = std::array::from_fn(|i| self.dma.channel[i]);
        for i in 0..8 {
            self.dma.channel[i].hdma_active = self.display_state().is_hdma_channel_enabled(i);
        }

        let saved_irq_control_flag = self.display_state().irq_control_flag;
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
        let saved_bg_scrolls: [(u16, u16); 4] =
            std::array::from_fn(|i| (self.ppu.bg_layer[i].h_scroll, self.ppu.bg_layer[i].v_scroll));
        let saved_m7_matrix = self.ppu.m7_matrix;
        let saved_m7_prev = self.ppu.m7_prev;

        let mut hdma_chans = [SimpleHdma::default(), SimpleHdma::default()];
        self.simple_hdma_init(&mut hdma_chans[0], &self.dma.channel[6]);
        self.simple_hdma_init(&mut hdma_chans[1], &self.dma.channel[7]);

        let mut result =
            Box::new([(0u8, 0u8, 0u8, 0u8, 0u8, [0u16; 4], [0u16; 4], [0i16; 8]); 224]);
        for line in 0..=224usize {
            if line == 128 && self.display_state().has_irq_control_flag() {
                let name_scroll_x = self.select_file_scratch_view().name_scroll_x();
                self.zelda_ppu_write(0x2111, name_scroll_x as u8);
                self.zelda_ppu_write(0x2111, (name_scroll_x >> 8) as u8);
                self.zelda_ppu_write(0x2112, 0);
                self.zelda_ppu_write(0x2112, 0);
                if self.display_state().irq_control_has_vcounter_marker() {
                    self.clear_irq_control_flag();
                }
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
                );
            }

            self.simple_hdma_do_line(&mut hdma_chans[0]);
            self.simple_hdma_do_line(&mut hdma_chans[1]);
        }

        self.set_irq_control_flag(saved_irq_control_flag);
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
            self.dma.channel[i].hdma_active = self.display_state().is_hdma_channel_enabled(i);
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
            self.dma.channel[i].hdma_active = self.display_state().is_hdma_channel_enabled(i);
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
        let mut extra_left = 0u16;
        let mut extra_right = 0u16;
        let mut extra_bottom = 0u16;
        let frame = *self.frame_state();
        let mut module = frame.main_module;
        if module == 14 {
            module = self.frame_state().saved_module_for_menu;
        }

        if module == 9 {
            if frame.main_module == 14
                && frame.submodule == 7
                && self.world_state_view().overworld_map_state() >= 4
            {
                extra_left = PPU_SIDE_SPACE_LIMIT;
                extra_right = PPU_SIDE_SPACE_LIMIT;
                extra_bottom = 16;
            } else {
                let bg2x = self.world_state_view().bg2_x();
                let bg2y = self.world_state_view().bg2_y();
                extra_left = bg2x.wrapping_sub(self.world_state_view().scroll_x_start());
                extra_right = self.world_state_view().scroll_x_end().wrapping_sub(bg2x);
                extra_bottom = self.world_state_view().scroll_y_end().wrapping_sub(bg2y);
            }
        } else if module == 7 {
            if !(self.dungeon_state_view().dungeon_dark_with_lantern()
                && self.display_state().sub_screen_layers != 0)
            {
                let qm = (self.world_state_view().quadrant_fullsize_x() >> 1) as usize;
                let bg2x = self.world_state_view().bg2_x();
                extra_left = bg2x.saturating_sub(self.room_bounds_view().x_bound(qm));
                extra_right = self.room_bounds_view().x_bound(qm + 2).saturating_sub(bg2x);
            }
            let qy = (self.world_state_view().quadrant_fullsize_y() >> 1) as usize;
            let bg2y = self.world_state_view().bg2_y();
            extra_bottom = self.room_bounds_view().y_bound(qy + 2).saturating_sub(bg2y);
        } else if module == 20 || module == 0 || module == 1 {
            extra_left = PPU_SIDE_SPACE_LIMIT;
            extra_right = PPU_SIDE_SPACE_LIMIT;
            extra_bottom = 16;
        }

        self.ppu.extra_left_cur = extra_left.min(PPU_SIDE_SPACE_LIMIT) as u8;
        self.ppu.extra_right_cur = extra_right.min(PPU_SIDE_SPACE_LIMIT) as u8;
        self.ppu.extra_bottom_cur = extra_bottom.min(16) as u8;
    }

    pub fn zelda_draw_ppu_frame(
        &mut self,
        pixel_buffer: &mut [u8],
        pitch: usize,
        render_flags: PpuRenderFlags,
    ) {
        self.ppu.obj_vram_latch = None;
        let render_width =
            if self.ppu.extra_left_right != 0 && self.ppu.current_render_scale(render_flags) == 1 {
                PPU_X_PIXELS
            } else {
                pitch / 4
            };
        let render_pitch = render_width * 4;
        let output_height = if render_flags.contains(PpuRenderFlags::HEIGHT_240) {
            240
        } else {
            224
        };
        let use_wide_render = render_width > pitch / 4;
        let active_pitch = if use_wide_render { render_pitch } else { pitch };
        let mut wide_buffer = if use_wide_render {
            vec![0; render_pitch * (output_height + 1)]
        } else {
            Vec::new()
        };

        if use_wide_render {
            self.ppu
                .begin_drawing(&mut wide_buffer, active_pitch, render_flags);
        } else {
            self.ppu
                .begin_drawing(pixel_buffer, active_pitch, render_flags);
        }

        for i in 0..8 {
            self.dma.channel[i].hdma_active = self.display_state().is_hdma_channel_enabled(i);
        }
        let mut hdma_chans = [SimpleHdma::default(), SimpleHdma::default()];
        self.simple_hdma_init(&mut hdma_chans[0], &self.dma.channel[6]);
        self.simple_hdma_init(&mut hdma_chans[1], &self.dma.channel[7]);

        if render_flags.contains(PpuRenderFlags::MODE7_4X4) && self.ppu.mode == 7 {
            if hdma_chans[0].table.as_deref() == Some(&MAP_MODE_HDMA_SETUP_NEAR) {
                self.set_mode7_perspective_correction(
                    MAP_MODE_PERSPECTIVE_ZOOMS_NEAR[0],
                    MAP_MODE_PERSPECTIVE_ZOOMS_NEAR[223],
                );
            } else if hdma_chans[0].table.as_deref() == Some(&MAP_MODE_HDMA_SETUP_FAR) {
                self.set_mode7_perspective_correction(
                    MAP_MODE_PERSPECTIVE_ZOOMS_FAR[0],
                    MAP_MODE_PERSPECTIVE_ZOOMS_FAR[223],
                );
            } else if hdma_chans[0].table.as_deref() == Some(&ATTRACT_INDIRECT_HDMA_SETUP) {
                let low = self.display_nmi_view().hdma_table_dynamic_entry(0);
                let high = self.display_nmi_view().hdma_table_dynamic_entry(223);
                self.set_mode7_perspective_correction(low, high);
            } else {
                self.set_mode7_perspective_correction(0, 0);
            }
        }

        if self.ppu.extra_left_right != 0 || render_flags.contains(PpuRenderFlags::HEIGHT_240) {
            self.configure_ppu_side_space();
        }

        for line in 0..=output_height {
            if line == 128 && self.display_state().has_irq_control_flag() {
                let name_scroll_x = self.select_file_scratch_view().name_scroll_x();
                self.zelda_ppu_write(0x2111, name_scroll_x as u8);
                self.zelda_ppu_write(0x2111, (name_scroll_x >> 8) as u8);
                self.zelda_ppu_write(0x2112, 0);
                self.zelda_ppu_write(0x2112, 0);
                if self.display_state().irq_control_has_vcounter_marker() {
                    self.clear_irq_control_flag();
                }
            }
            self.ppu.run_line(line as i32);
            self.simple_hdma_do_line(&mut hdma_chans[0]);
            self.simple_hdma_do_line(&mut hdma_chans[1]);
        }

        if let Some(rendered) = self.ppu.render_buffer.as_ref() {
            if use_wide_render {
                let crop_x = self.ppu.extra_left_right as usize;
                let row_bytes = (pitch / 4).min(256) * 4;
                for y in 0..output_height {
                    let src = y * active_pitch + crop_x * 4;
                    let dst = y * pitch;
                    if src + row_bytes <= rendered.len() && dst + row_bytes <= pixel_buffer.len() {
                        pixel_buffer[dst..dst + row_bytes]
                            .copy_from_slice(&rendered[src..src + row_bytes]);
                    }
                }
            } else {
                let n = pixel_buffer.len().min(rendered.len());
                pixel_buffer[..n].copy_from_slice(&rendered[..n]);
            }
        }
        self.ppu.finish_drawing();
    }

    pub fn zelda_draw_display_frame(
        &mut self,
        pixel_buffer: &mut [u8],
        pitch: usize,
        render_flags: PpuRenderFlags,
    ) {
        self.zelda_draw_ppu_frame(pixel_buffer, pitch, render_flags);
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
        let src = self.ram_bytes(SAVELOAD_HDMA_TABLE, 224 * 2);
        self.native_ram_bridge_view_mut()
            .copy_to(HDMA_TABLE_DYNAMIC, &src);
        self.zelda_restore_music_after_load_locked(false);
        self.sync_native_game_state_from_ram();
        self.assert_native_world_location_state_matches_ram();
        self.sync_overworld_map16_load_from_ram();
        self.emu_synchronize_whole_state();
    }

    fn save_snes_state(&mut self, func: &mut SaveLoadFunc<'_, '_>) {
        let src = self.ram_bytes(HDMA_TABLE_DYNAMIC, 224 * 2);
        self.native_ram_bridge_view_mut()
            .copy_to(SAVELOAD_HDMA_TABLE, &src);
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
                        self.native_ram_bridge_view_mut()
                            .set_byte_at(offset, sr.log.data[replay_pos]);
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

    pub fn zelda_run_frame(&mut self, mut inputs: i32) -> bool {
        if inputs & 0x30 == 0x30 {
            inputs ^= 0x30;
        }
        if inputs & 0xc0 == 0xc0 {
            inputs ^= 0xc0;
        }

        self.frame_ctr_dbg = self.frame_ctr_dbg.wrapping_add(1);
        self.replay_trace_ram_watch("frame-entry");
        let mut state_recorder = std::mem::take(&mut self.state_recorder);
        let is_replay = state_recorder.replay_mode;
        let input_state = if is_replay {
            let input_state = self.state_recorder_read_next_replay_state(&mut state_recorder);
            self.replay_trace_col("after-replay-command");
            self.replay_trace_ram_watch("after-replay-command");
            input_state
        } else {
            Self::state_recorder_record(&mut state_recorder, inputs as u16);
            let apui00 = self.zelda_is_music_playing() as u8;
            if apui00 != self.system_signals_view().apui00() {
                self.system_signals_view_mut().set_apui00(apui00);
                let apui00_offset = SystemSignalsView::apui00_offset();
                self.emu_sync_memory_region(apui00_offset, 1);
                Self::state_recorder_record_patch_byte(
                    &mut state_recorder,
                    apui00_offset as u32,
                    &[apui00],
                    1,
                );
            }
            if self
                .native_ram_bridge_view()
                .word_at(ANIMATED_TILE_DATA_SRC)
                != 0
            {
                if self.system_signals_view().bugs_fixed() < BUGFIX_LATEST {
                    if !self.rom_startup_timing {
                        self.system_signals_view_mut().set_bugs_fixed(BUGFIX_LATEST);
                        self.emu_sync_memory_region(RAM_BUGS_FIXED, 1);
                        Self::state_recorder_record_patch_byte(
                            &mut state_recorder,
                            RAM_BUGS_FIXED as u32,
                            &[BUGFIX_LATEST],
                            1,
                        );
                    }
                }
                let enhanced_features0 = self.enhanced_features_view().bits();
                let wanted_zelda_features = self.wanted_zelda_features;
                if enhanced_features0 != wanted_zelda_features {
                    self.enhanced_features_view_mut()
                        .set_bits(wanted_zelda_features);
                    self.emu_sync_memory_region(ENHANCED_FEATURES0, 4);
                    Self::state_recorder_record_patch_byte(
                        &mut state_recorder,
                        ENHANCED_FEATURES0 as u32,
                        &wanted_zelda_features.to_le_bytes(),
                        4,
                    );
                }
            }
            inputs as u16
        };
        self.state_recorder = state_recorder;

        self.sync_native_game_state_from_ram();
        self.assert_native_frame_state_matches_ram();
        self.assert_native_world_location_state_matches_ram();
        let frame = self.frame_state();
        let use_bsnes_poly_scheduler = self.rom_startup_timing
            && frame.main_module == 0
            && matches!(frame.submodule, 3 | 4)
            && frame.frame_counter >= configured_poly_scheduler_frame_threshold();
        self.bsnes_hold_intro_step_this_frame = false;
        let run_what = if self.system_signals_view().bugs_fixed() < BUGFIX_POLY_RENDERER
            && !use_bsnes_poly_scheduler
        {
            if self.display_state().nmi_thread_uses_poly_stack() {
                2
            } else {
                1
            }
        } else {
            let virq = self.display_state().vertical_irq_trigger;
            let carry = if self.display_state().nmi_thread_active {
                if use_bsnes_poly_scheduler {
                    let previous_counter = self.bsnes_poly_scheduler_counter;
                    let step_waiting = self.attract_state_view().intro_step_index() == 1
                        && self.attract_state_view().intro_did_run_step() != 0;
                    let first_frame_boundary_hold = step_waiting
                        && !self.bsnes_intro_step_carry_phase_active
                        && previous_counter == 0;
                    self.bsnes_hold_intro_step_this_frame = first_frame_boundary_hold
                        || (step_waiting
                            && self.bsnes_intro_step_carry_phase_active
                            && self.bsnes_intro_step_hold_alternate);
                    if first_frame_boundary_hold {
                        self.bsnes_intro_step_carry_phase_active = true;
                        self.bsnes_intro_step_hold_alternate = false;
                    } else if step_waiting && self.bsnes_intro_step_carry_phase_active {
                        self.bsnes_intro_step_hold_alternate =
                            !self.bsnes_intro_step_hold_alternate;
                    }
                    self.bsnes_poly_scheduler_counter =
                        self.bsnes_poly_scheduler_counter.wrapping_add(virq);
                    self.attract_state_view().intro_did_run_step() != 0
                } else {
                    let carry = Self::increment_crystal_countdown(
                        self.native_ram_bridge_view_mut()
                            .range_mut(RAM_CRYSTAL_ROTATE_COUNTER, 1)
                            .first_mut()
                            .expect("crystal rotate counter byte exists"),
                        virq,
                    ) != 0;
                    self.emu_sync_memory_region(RAM_CRYSTAL_ROTATE_COUNTER, 1);
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
        if self.emu_runframe.is_none()
            || self.enhanced_features_view().bits() != 0
            || self.dialogue_flags != 0
        {
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
        const REFERENCE_SAVES: [&str; 13] = [
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
        if which & 256 != 0 {
            if cmd == SaveLoadCommand::Save {
                return None;
            }
            let index = (which - 256) as usize;
            Some(Path::new("saves/ref").join(REFERENCE_SAVES[index]))
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
        self.native_ram_bridge_view_mut()
            .set_byte_at(addr as usize, value);
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
                let rupees = self.player_resources_view().rupees_goal().wrapping_add(100);
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
                self.player_state_view().cheat_walk_through_walls() ^ 1,
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
        self.native_ram_bridge_view_mut()
            .set_word_at(ANIMATED_TILE_DATA_SRC, 0xa680);
        self.native_ram_bridge_view_mut()
            .set_word_at(DMA_SOURCE_ADDR_9, 0xb280);
        self.native_ram_bridge_view_mut()
            .set_word_at(DMA_SOURCE_ADDR_14, 0xb280 + 0x60);
        self.sync_native_game_state_from_ram();
        self.assert_native_world_location_state_matches_ram();
    }

    fn startup_initialize_memory(&mut self) {
        self.native_ram_bridge_view_mut().fill(0, 0x2000, 0);
        self.native_ram_bridge_view_mut()
            .set_word_at(MAIN_PALETTE_BUFFER, 0);
        self.clear_selected_save_slot();

        for offset in [0x03e5, 0x08e5, 0x0de5] {
            if read_le_u16(&self.sram, offset) != 0x55aa {
                write_le_u16(&mut self.sram, offset, 0);
            }
        }

        self.set_screen_brightness(0x80);
        self.system_signals_view_mut().increment_cgram_update_flag();
        self.sync_native_game_state_from_ram();
        self.assert_native_world_location_state_matches_ram();
    }

    fn zelda_run_poly_loop(&mut self) {
        let can_run_poly = self.attract_state_view().intro_did_run_step() != 0
            && !self.display_state().has_pending_polyhedral_update();
        if can_run_poly {
            self.poly_run_frame();
            self.attract_state_view_mut().clear_intro_did_run_step();
            self.request_polyhedral_nmi_update();
        }
    }

    fn increment_crystal_countdown(a: &mut u8, v: u8) -> i32 {
        let t = (*a as u16).wrapping_add(v as u16);
        *a = t as u8;
        (t >> 8) as i32
    }

    fn zelda_run_game_loop(&mut self) {
        self.increment_frame_counter();
        self.replay_trace_ram_watch("game-loop-after-frame-counter");
        self.clear_oam_buffer();
        self.replay_trace_ram_watch("game-loop-after-clear-oam");
        self.module_main_routing();
        self.replay_trace_ram_watch("game-loop-after-module");
        self.nmi_prepare_sprites();
        self.replay_trace_ram_watch("game-loop-after-prepare-sprites");
        self.clear_nmi_update_latch();
        self.replay_trace_ram_watch("game-loop-exit");
    }

    fn clear_oam_buffer(&mut self) {
        for i in 0..128 {
            self.oam_state_view_mut().hide_sprite_row(i);
        }
    }

    fn run_dungeon_submodule(&mut self) {
        match self.frame_state().submodule {
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
        self.player_state_view_mut().clear_item_in_hand();
        self.player_state_view_mut().clear_position_mode();
        self.player_state_view_mut().clear_action_scratch_state();
        self.player_state_view_mut().set_y_button_action_step(0);
        self.player_state_view_mut().set_y_button_action_flags(0);
        self.player_state_view_mut()
            .clear_button_mask_b_y_bits(0x40);
        self.player_state_view_mut()
            .clear_state_item_and_grab_flags();
        self.player_state_view_mut().clear_defense_flags();
        self.link_reset_swimming_state();
        self.player_state_view_mut().clear_direction_lock_bits(1);
        self.player_state_view_mut().clear_z_high();
        if self.player_state_view().electrocute_on_touch() != 0 {
            if self.player_state_view().is_cape_active() {
                self.link_force_unequip_cape_quietly();
            }
            self.link_reset_sword_and_item_usage();
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.player_state_view_mut().clear_action_handler_timer();
            self.player_state_view_mut().set_spin_attack_delay_timer(2);
            self.player_state_view_mut().clear_animation_step();
            self.player_state_view_mut().clear_cardinal_direction();
            self.set_sound_effect_1_with_link_pan(43);
            self.player_state_view_mut().set_handler_state(7);
            self.link_state_zapped();
        } else {
            self.player_state_view_mut()
                .clear_moving_against_diag_tile();
            self.player_state_view_mut().set_handler_state(6);
            self.link_state_recoil();
        }
    }

    fn ancilla_add_tablet_spell(&mut self, ty: u8) {
        self.ancilla_add_simple(ty, 0);
    }

    fn link_state_pits_after_aux_state(&mut self) {
        const FEATURES0_MISC_BUG_FIXES: u32 = 4096;
        self.replay_trace_submodule("pits-entry");
        self.replay_trace_player_state("pits-entry");
        self.tile_detect_main_handler(4);
        self.replay_trace_submodule("pits-after-tile-detect");
        self.replay_trace_player_state("pits-after-tile-detect");
        if self.tile_detect_position_view().pit_tile() & 1 == 0 {
            if self.enhanced_features_view().has(FEATURES0_MISC_BUG_FIXES) {
                self.player_state_view_mut().clear_near_pit_state();
            }
            if self.player_state_view().is_running() {
                self.link_state_dashing();
                return;
            }
            self.player_state_view_mut().set_speed_setting(0);
            self.link_cancel_dash();
            if self.player_state_view().button_mask_b_y() & 0x80 == 0 {
                self.player_state_view_mut().clear_direction_lock_bits(1);
            }
            self.player_state_view_mut().clear_near_pit_state();
            let player_state = self.player_state_view();
            let handler_state = if !player_state.is_bunny_mirror() {
                0
            } else if player_state.has_moon_pearl() {
                3
            } else {
                23
            };
            self.player_state_view_mut()
                .set_handler_state(handler_state);
            match self.player_state_view().handler_state() {
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
        self.player_state_view_mut().set_speed_setting(4);
        if self.tile_detect_position_view().pit_tile() & 0x0f == 0 {
            self.player_state_view_mut().clear_near_pit_state();
            self.player_state_view_mut().set_speed_setting(0);
            let player_state = self.player_state_view();
            let handler_state = if !player_state.is_bunny_mirror() {
                0
            } else if player_state.has_moon_pearl() {
                3
            } else {
                23
            };
            self.player_state_view_mut()
                .set_handler_state(handler_state);
            self.link_cancel_dash();
            if self.player_state_view().button_mask_b_y() & 0x80 == 0 {
                self.player_state_view_mut().clear_direction_lock_bits(1);
            }
            self.replay_trace_submodule("pits-clear-low-nibble-exit");
            return;
        }

        if self.tile_detect_position_view().pit_tile() & 0x0f != 0x0f {
            self.replay_trace_player_state("pits-edge-slide-entry");
            let mut i = 3i8;
            loop {
                if self.tile_detect_position_view().pit_tile() & 0x0f
                    == FALL_HOLE_PIT_DIRS[i as usize]
                {
                    i += 4;
                    break;
                }
                i -= 1;
                if i < 0 {
                    i = 3;
                    let mut pit_tile = self.tile_detect_position_view().pit_tile();
                    while pit_tile & 1 == 0 {
                        i -= 1;
                        pit_tile >>= 1;
                    }
                    break;
                }
            }
            self.tile_detect_position_view_mut()
                .set_fall_hole_scan_index(i as u8);
            let idx = i as usize;
            if self.player_state_view().direction() & FALL_HOLE_DIRS[idx] != 0 {
                self.player_state_view_mut()
                    .set_last_direction_from_current_direction();
                self.player_state_view_mut().set_speed_setting(6);
                self.link_handle_moving_animation_full_long_entry();
            } else {
                let old_dir = self.player_state_view().direction();
                self.player_state_view_mut()
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

        if !self.player_state_view().near_pit_state_is(2) {
            if self.player_state_view().has_moon_pearl() {
                self.player_state_view_mut()
                    .clear_bunny_transform_after_moon_pearl();
            }
            self.player_state_view_mut().set_direction(0);
            self.player_state_view_mut().set_near_pit_state(2);
            self.player_state_view_mut()
                .set_sprite_damage_disable_timer(1);
            self.player_state_view_mut().set_button_mask_b_y(0);
            self.player_state_view_mut().set_button_b_frames(0);
            self.player_state_view_mut().clear_item_in_hand();
            self.player_state_view_mut().clear_position_mode();
            self.player_state_view_mut().set_incapacitated_timer(0);
            self.player_state_view_mut().clear_auxiliary_state();
            self.ancilla_sfx3_near(31);
        }

        self.player_state_view_mut().clear_direction_lock();
        self.player_state_view_mut().set_incapacitated_timer(0);
        self.player_state_view_mut().set_z(0);
        self.player_state_view_mut().set_actual_z_velocity(0);
        self.player_state_view_mut().clear_auxiliary_state();
        self.player_state_view_mut().clear_given_damage();
        self.player_state_view_mut().clear_transforming();
        self.link_force_unequip_cape_quietly();
        self.player_state_view_mut()
            .increment_sprite_damage_disable_timer();
        if (self
            .player_state_view_mut()
            .decrement_sprite_oam_state_timer() as i8)
            >= 0
        {
            return;
        }

        self.player_state_view_mut().advance_pit_data_index();
        let x = self.player_state_view().pit_data_index();
        self.player_state_view_mut().set_sprite_oam_state_timer(9);
        if self.follower_state_view().indicator() != 13 && x == 1 {
            self.follower_state_view_mut().set_appearance_none_flag(x);
        }

        if x == 6 {
            self.link_cancel_dash();
            self.set_submodule(7);
            self.player_state_view_mut().set_pit_data_index(6);
            self.player_state_view_mut().set_near_pit_state(3);
            self.player_state_view_mut().set_visibility_status(12);
            self.player_state_view_mut().set_speed_modifier(16);
            let y = self
                .player_state_view()
                .y()
                .wrapping_sub(self.world_state_view().bg2_y()) as u8;
            self.player_state_view_mut()
                .clear_state_item_and_grab_flags();
            self.player_state_view_mut().set_y_button_action_timer(0);
            if self.world_location_state().is_indoors() {
                let room = self.world_location_state().dungeon_room_index();
                self.dungeon_state_view_mut().set_room_index_prev(room);
                self.Dungeon_FlagRoomData_Quadrants();
                if self.Dungeon_IsPitThatHurtsPlayer() {
                    self.dungeon_pit_do_damage();
                    return;
                }
            }
            let previous_room = self.world_location_state().dungeon_room_index();
            self.dungeon_state_view_mut()
                .set_room_index_prev(previous_room);
            let room = self.dungeon_header_view().travel_destination(0);
            self.set_dungeon_room_index(room);
            let player_y = self.player_state_view().y();
            self.tile_detect_position_view_mut().set_y(player_y);
            let new_y = self
                .player_state_view()
                .y()
                .wrapping_sub(y as u16)
                .wrapping_sub(0x10);
            self.player_state_view_mut().set_y(new_y);
            if self.world_location_state().is_indoors() {
                self.handle_layer_of_destination();
            } else if self.world_location_state().overworld_screen_index() != 5 {
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
        self.player_state_view_mut().set_facing(0);
        self.player_state_view_mut().clear_state_bits();
        self.player_state_view_mut().clear_direction_lock();
        self.player_state_view_mut().clear_handler_state();
    }

    fn link_state_tree_pull_tail(&mut self) {
        self.link_move_position();
        self.link_handle_cardinal_collision();
        self.handle_indoor_camera_and_doors();
    }

    fn finish_recoil_landing(&mut self) {
        if self.player_state_view().lower_level_state() == 2 {
            self.player_state_view_mut().set_lower_level_state(0);
        }
        if self.player_state_view().about_to_jump_off_ledge() != 0 {
            self.dungeon_handle_layer_change();
        }
        self.player_state_view_mut().set_z(0);
        self.player_state_view_mut().clear_auxiliary_state();
        self.player_state_view_mut().set_speed_setting(0);
        self.player_state_view_mut().clear_direction_lock();
        self.player_state_view_mut().clear_item_in_hand();
        self.player_state_view_mut().clear_position_mode();
        self.player_state_view_mut().clear_action_handler_timer();
        self.player_state_view_mut()
            .clear_sprite_damage_disable_timer();
        self.player_state_view_mut().clear_electrocute_on_touch();
        self.player_state_view_mut().clear_actual_velocity_xy();
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
        self.player_state_view_mut().clear_movement_velocity();
        self.finish_ground_movement_camera_tail();
    }

    fn finish_ground_movement_camera_tail(&mut self) {
        self.player_state_view_mut().clear_pit_correction();
        if self.apply_links_movement_to_camera_called && self.enhanced_features_view().has(4096) {
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
            self.dungeon_state_view_mut()
                .set_bg2_tile(tile_pos as usize, src[i]);
            self.dungeon_state_view_mut()
                .set_bg2_attr(tile_pos as usize, attr);
        }

        let upload = self.display_state().vram_upload_cursor_usize();
        let dst = self.vram_upload_data_view().data_address(upload);
        for (i, &tile_pos) in positions.iter().enumerate() {
            let base = dst + i * 6;
            let addr = self.Dungeon_MapVramAddr(tile_pos);
            self.vram_upload_data_view_mut().write_le_u16_at(base, addr);
            self.vram_upload_data_view_mut()
                .write_le_u16_at(base + 2, 0x0100);
            self.vram_upload_data_view_mut()
                .write_le_u16_at(base + 4, src[i]);
        }
        self.vram_upload_data_view_mut()
            .write_le_u16_at(dst + 24, 0xffff);
        self.vram_upload_data_view_mut()
            .set_offset(upload.wrapping_add(24) as u16);
        self.set_bg_vram_load_mode(1);
        self.Dungeon_FlagRoomData_Quadrants();
        if self.system_signals_view().sound_effect_2() == 0 {
            self.system_signals_view_mut().set_sound_effect_2(14);
        }
        loc & 0x7fff
    }

    fn cache_camera_properties_for_player(&mut self) {
        self.ppu_scroll_copy_view_mut().cache_bg2_live_scroll();
        self.player_state_view_mut().cache_current_position();
        let y_start = self.room_bounds_view().y_bound(0);
        let y_end = self.room_bounds_view().y_bound(2);
        let x_start = self.room_bounds_view().x_bound(0);
        let x_end = self.room_bounds_view().x_bound(2);
        self.dungeon_state_view_mut()
            .set_cached_room_bounds(y_start, y_end, x_start, x_end);
        self.world_state_view_mut().cache_scroll_targets();
        self.ppu_scroll_copy_view_mut().cache_camera_scroll();
        self.world_state_view_mut().cache_quadrant_fullsize_state();
        self.player_state_view_mut().cache_current_quadrants();
        self.player_state_view_mut().cache_facing();
        self.player_state_view_mut().cache_lower_level_states();
        self.dungeon_state_view_mut().cache_standing_in_doorway();
        self.dungeon_state_view_mut().cache_current_floor();
    }

    fn move_link_coord(&mut self, subpixel: usize, coord: usize, vel: u8) -> u16 {
        self.native_ram_bridge_view_mut()
            .move_link_axis_by_velocity(subpixel, coord, vel)
    }

    fn move_link_coord_subpixel_delta(&mut self, subpixel: usize, coord: usize, delta: u16) -> u16 {
        self.native_ram_bridge_view_mut()
            .move_link_axis_by_subpixel_delta(subpixel, coord, delta)
    }

    fn store_link_safe_return_position(&mut self, x: u16, y: u16) {
        self.player_state_view_mut()
            .store_safe_return_position(x, y);
    }

    fn restore_link_safe_return_position(&mut self) {
        self.player_state_view_mut()
            .restore_position_from_safe_return();
    }

    fn set_link_z_coord_mirror_low_ff(&mut self) {
        self.player_state_view_mut().force_z_mirror_low_ff();
    }

    fn set_backdrop_color_black(&mut self) {
        self.palette_filter_view_mut().set_fixed_color_red(0x20);
        self.palette_filter_view_mut().set_fixed_color_green(0x40);
        self.palette_filter_view_mut().set_fixed_color_blue(0x80);
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
        self.oam_state_view_mut()
            .write_clipped_entry_with_extended(oam, x, y, charnum, flags, big);
    }

    fn set_oam_helper1_at(&mut self, oam: usize, x: u16, y: u8, charnum: u8, flags: u8, big: u8) {
        self.oam_state_view_mut()
            .write_entry_with_extended(oam, x, y, charnum, flags, big);
    }

    fn write_intro_x(&mut self, k: usize, value: i16) {
        self.intro_actor_view_mut(k).set_x(value);
    }

    fn write_intro_y(&mut self, k: usize, value: i16) {
        self.intro_actor_view_mut(k).set_y(value);
    }

    fn set_oam_plain(&mut self, index: usize, x: u8, y: u8, charnum: u8, flags: u8, big: u8) {
        self.oam_state_view_mut()
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
        self.oam_state_view_mut()
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
        const PALETTE_ASSETS: &[(u32, usize)] = &[
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

        for &(base, asset) in PALETTE_ASSETS {
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

    fn decrement_word(&mut self, offset: usize) -> u16 {
        let value = self
            .native_ram_bridge_view()
            .word_at(offset)
            .wrapping_sub(1);
        self.native_ram_bridge_view_mut().set_word_at(offset, value);
        value
    }

    fn ram_byte(&self, offset: usize) -> u8 {
        self.native_ram_bridge_view().byte_at(offset)
    }

    fn set_ram_byte(&mut self, offset: usize, value: u8) {
        self.native_ram_bridge_view_mut().set_byte_at(offset, value);
    }

    fn copy_to_ram(&mut self, offset: usize, data: &[u8]) {
        self.native_ram_bridge_view_mut().copy_to(offset, data);
    }

    fn fill_ram(&mut self, offset: usize, len: usize, value: u8) {
        self.native_ram_bridge_view_mut().fill(offset, len, value);
    }

    fn has_player_layer_collision(&self, mask: u8) -> bool {
        self.ram_byte(PLAYER_LAYER_COLLISION_FLAGS) & mask == mask
    }

    fn set_player_layer_collision(&mut self, mask: u8, enabled: bool) {
        let flags = self.ram_byte(PLAYER_LAYER_COLLISION_FLAGS);
        let flags = if enabled { flags | mask } else { flags & !mask };
        self.set_ram_byte(PLAYER_LAYER_COLLISION_FLAGS, flags);
    }

    fn read_u8_ram(&self, offset: usize) -> u8 {
        self.native_ram_bridge_view().byte_at(offset)
    }

    fn write_u8_ram(&mut self, offset: usize, value: u8) {
        self.native_ram_bridge_view_mut().set_byte_at(offset, value);
    }

    fn read_u16_ram(&self, offset: usize) -> u16 {
        self.native_ram_bridge_view().word_at(offset)
    }

    fn write_u16_ram(&mut self, offset: usize, value: u16) {
        self.native_ram_bridge_view_mut().set_word_at(offset, value);
    }

    fn read_u32_ram(&self, offset: usize) -> u32 {
        self.native_ram_bridge_view().long_at(offset)
    }

    fn write_u32_ram(&mut self, offset: usize, value: u32) {
        self.native_ram_bridge_view_mut().set_long_at(offset, value);
    }

    fn read_i16_ram(&self, offset: usize) -> i16 {
        self.read_u16_ram(offset) as i16
    }

    fn write_i16_ram(&mut self, offset: usize, value: i16) {
        self.write_u16_ram(offset, value as u16);
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

fn copy_le_u16(bytes: &mut [u8], dst: usize, src: usize) {
    let value = read_le_u16(bytes, src);
    write_le_u16(bytes, dst, value);
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
    const UPPER_BITMASKS: [u16; 16] = [
        0x8000, 0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100, 0x0080, 0x0040, 0x0020,
        0x0010, 0x0008, 0x0004, 0x0002, 0x0001,
    ];
    UPPER_BITMASKS[index & 0x0f]
}

fn receive_item_tab1(item: u8) -> u8 {
    const RECEIVE_ITEM_OAM_EXT_SIZES: [u8; 76] = [
        0, 0, 0, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 0, 2, 0, 2, 2, 0, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 2, 2, 2, 2, 2,
        2, 2, 2, 2, 2, 2, 0, 0, 2, 0, 2, 2, 2, 0, 2, 2,
    ];
    RECEIVE_ITEM_OAM_EXT_SIZES
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn receive_item_tab2(item: u8) -> i8 {
    const RECEIVE_ITEM_DRAW_Y_OFFSETS: [i8; 76] = [
        -5, -5, -5, -5, -5, -4, -4, -5, -5, -4, -4, -4, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4,
        -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -5, -4, -4, -4, -4, -4, -4, -2, -4, -4, -4,
        -4, -4, -4, -4, -4, -4, -2, -2, -2, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -2, -2, -4,
        -2, -4, -4, -4, -5, -4, -4,
    ];
    RECEIVE_ITEM_DRAW_Y_OFFSETS
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn receive_item_tab3(item: u8) -> u8 {
    const RECEIVE_ITEM_PALETTE_BITS: [u8; 76] = [
        4, 4, 4, 4, 4, 0, 0, 4, 4, 4, 4, 4, 5, 0, 0, 0, 0, 0, 0, 4, 0, 4, 0, 0, 4, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 4, 4, 0, 4, 0, 0, 0, 4, 0, 0,
    ];
    RECEIVE_ITEM_PALETTE_BITS
        .get(item as usize)
        .copied()
        .unwrap_or(0)
}

fn memory_location_to_give_item_to(item: u8) -> usize {
    const MEMORY_LOCATIONS: [usize; 76] = [
        0xf359, 0xf359, 0xf359, 0xf359, 0xf35a, 0xf35a, 0xf35a, 0xf345, 0xf346, 0xf34b, 0xf342,
        0xf340, 0xf341, 0xf344, 0xf35c, 0xf347, 0xf348, 0xf349, 0xf34a, 0xf34c, 0xf34c, 0xf350,
        0xf35c, 0xf36b, 0xf351, 0xf352, 0xf353, 0xf354, 0xf354, 0xf34e, 0xf356, 0xf357, 0xf37a,
        0xf34d, 0xf35b, 0xf35b, 0xf36f, 0xf364, 0xf36c, 0xf375, 0xf375, 0xf344, 0xf341, 0xf35c,
        0xf35c, 0xf35c, 0xf36d, 0xf36e, 0xf36e, 0xf375, 0xf366, 0xf368, 0xf360, 0xf360, 0xf360,
        0xf374, 0xf374, 0xf374, 0xf340, 0xf340, 0xf35c, 0xf35c, 0xf36c, 0xf36c, 0xf360, 0xf360,
        0xf372, 0xf376, 0xf376, 0xf373, 0xf360, 0xf360, 0xf35c, 0xf359, 0xf34c, 0xf355,
    ];
    MEMORY_LOCATIONS.get(item as usize).copied().unwrap_or(0)
}

fn value_to_give_item_to(item: u8) -> u8 {
    const VALUES: [u8; 76] = [
        1, 2, 3, 4, 1, 2, 3, 1, 1, 1, 1, 1, 1, 2, 0xff, 1, 1, 1, 1, 1, 2, 1, 0xff, 0xff, 1, 1, 2,
        1, 2, 1, 1, 1, 0xff, 1, 0xff, 2, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 2, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfb, 0xec, 0xff, 0xff, 0xff, 1, 3, 0xff, 0xff,
        0xff, 0xff, 0x9c, 0xce, 0xff, 1, 10, 0xff, 0xff, 0xff, 0xff, 1, 3, 1,
    ];
    VALUES.get(item as usize).copied().unwrap_or(0xff)
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
mod tests {
    use super::*;

    fn test_sync_all(state: &mut ZeldaState) {
        state.ram[0x42] = state.ram[0x42].wrapping_add(1);
    }

    fn link_test_byte(state: &ZeldaState, addr: usize) -> u8 {
        state.ram[addr]
    }

    fn set_link_test_byte(state: &mut ZeldaState, addr: usize, value: u8) {
        state.ram[addr] = value;
    }

    fn link_test_word(state: &ZeldaState, addr: usize) -> u16 {
        read_le_u16(&state.ram, addr)
    }

    fn set_link_test_word(state: &mut ZeldaState, addr: usize, value: u16) {
        write_le_u16(&mut state.ram, addr, value);
    }

    #[test]
    fn owns_oracle_compared_memory_regions() {
        let state = ZeldaState::new();
        assert_eq!(state.ram.len(), WRAM_SIZE);
        assert_eq!(state.sram.len(), SRAM_SIZE);
        assert_eq!(state.vram().len(), VRAM_WORDS);
    }

    #[test]
    fn scratch_word_high_does_not_alias_nmi_subroutine_index() {
        let mut state = ZeldaState::new();
        state.scratch_word_view_mut().set_word(0x0200);
        state.set_pending_nmi_subroutine(11);

        assert_eq!(state.scratch_word_view_mut().decrement_high(), 1);

        assert_eq!(state.scratch_word_view().word(), 0x0100);
        assert_eq!(state.display_state().pending_nmi_subroutine, 11);
    }

    #[test]
    fn reset_can_preserve_sram() {
        let mut state = ZeldaState::new();
        state.ram[1] = 1;
        state.sram[1] = 2;
        state.vram_mut()[1] = 3;

        state.reset(true);

        assert_eq!(state.ram[1], 0);
        assert_eq!(state.sram[1], 2);
        assert_eq!(state.vram()[1], 0);
    }

    #[test]
    fn sram_path_prefers_explicit_save_dir() {
        assert_eq!(
            ZeldaState::sram_path_from_env(
                Some("/tmp/z3-save".into()),
                Some("/tmp/xdg".into()),
                Some("/tmp/home".into()),
            ),
            PathBuf::from("/tmp/z3-save/sram.dat")
        );
    }

    #[test]
    fn sram_path_uses_xdg_data_home_for_deck_safe_default() {
        assert_eq!(
            ZeldaState::sram_path_from_env(None, Some("/tmp/xdg".into()), Some("/tmp/home".into())),
            PathBuf::from("/tmp/xdg/zelda3-rs/saves/sram.dat")
        );
    }

    #[test]
    fn sram_path_falls_back_to_home_data_dir() {
        assert_eq!(
            ZeldaState::sram_path_from_env(None, None, Some("/tmp/home".into())),
            PathBuf::from("/tmp/home/.local/share/zelda3-rs/saves/sram.dat")
        );
    }

    #[test]
    fn rom_palette_words_fall_back_to_generated_assets_without_rom() {
        let mut state = ZeldaState::new();
        let ranges: [(u32, usize, u16); 14] = [
            (PALETTE_DUNGEON_BG_MAIN_SNES_ADDR, 79, 0x1111),
            (PALETTE_MAIN_SPRITE_SNES_ADDR, 80, 0x2222),
            (PALETTE_ARMOR_AND_GLOVES_SNES_ADDR, 81, 0x3333),
            (PALETTE_SWORD_SNES_ADDR, 82, 0x4444),
            (PALETTE_SHIELD_SNES_ADDR, 83, 0x5555),
            (PALETTE_SPRITE_AUX3_SNES_ADDR, 84, 0x6666),
            (PALETTE_MISC_SPRITE_INDOORS_SNES_ADDR, 85, 0x7777),
            (PALETTE_SPRITE_AUX1_SNES_ADDR, 86, 0x8888),
            (PALETTE_OVERWORLD_BG_MAIN_SNES_ADDR, 87, 0x9999),
            (PALETTE_OVERWORLD_BG_AUX12_SNES_ADDR, 88, 0xaaaa),
            (PALETTE_OVERWORLD_BG_AUX3_SNES_ADDR, 89, 0xbbbb),
            (PALETTE_PALACE_MAP_BG_SNES_ADDR, 90, 0xcccc),
            (PALETTE_PALACE_MAP_SPRITE_SNES_ADDR, 91, 0xdddd),
            (HUD_PALETTE_SNES_ADDR, 92, 0xeeee),
        ];
        let mut data = Vec::new();
        let mut asset_ranges = vec![(0, 0); 93];
        for &(_, asset, value) in &ranges {
            let start = data.len();
            data.extend_from_slice(&value.to_le_bytes());
            data.extend_from_slice(&(value ^ 0xffff).to_le_bytes());
            asset_ranges[asset] = (start, data.len());
        }
        state.assets = Some(AssetPack {
            data,
            ranges: asset_ranges,
        });

        for &(base, _, value) in &ranges {
            assert_eq!(state.rom_or_asset_word_snes(base), Some(value));
            assert_eq!(state.rom_or_asset_word_snes(base + 2), Some(value ^ 0xffff));
        }
    }

    #[test]
    fn player_layer_collision_helpers_preserve_unrelated_flags() {
        let mut state = ZeldaState::new();
        state.ram[PLAYER_LAYER_COLLISION_FLAGS] = 0xf0;

        state.set_player_layer_collision(
            crate::game_state::constants::player::LAYER_COLLISION_BG1,
            true,
        );
        assert_eq!(state.ram[PLAYER_LAYER_COLLISION_FLAGS], 0xf1);
        assert!(!state.has_player_layer_collision(
            crate::game_state::constants::player::LAYER_COLLISION_BOTH
        ));

        state.set_player_layer_collision(
            crate::game_state::constants::player::LAYER_COLLISION_BG2,
            true,
        );
        assert_eq!(state.ram[PLAYER_LAYER_COLLISION_FLAGS], 0xf3);
        assert!(state.has_player_layer_collision(
            crate::game_state::constants::player::LAYER_COLLISION_BOTH
        ));

        state.set_player_layer_collision(
            crate::game_state::constants::player::LAYER_COLLISION_BG1,
            false,
        );
        assert_eq!(state.ram[PLAYER_LAYER_COLLISION_FLAGS], 0xf2);
        assert!(!state.has_player_layer_collision(
            crate::game_state::constants::player::LAYER_COLLISION_BOTH
        ));
    }

    #[test]
    fn intro_background_settings_write_ppu_tilemap_regs() {
        let mut state = ZeldaState::new();
        state.ppu.bg_layer[0].tilemap_adr = 0;
        state.ppu.bg_layer[1].tilemap_adr = 0;
        state.ppu.bg_layer[2].tilemap_adr = 0;

        state.intro_initialize_background_settings();

        assert_eq!(state.display_state().bg_mode, 9);
        assert_eq!(state.display_state().mosaic_copy, 0);
        assert_eq!(state.ppu.bg_layer[0].tilemap_adr, 0x1000);
        assert!(state.ppu.bg_layer[0].tilemap_wider);
        assert!(state.ppu.bg_layer[0].tilemap_higher);
        assert_eq!(state.ppu.bg_layer[1].tilemap_adr, 0);
        assert!(state.ppu.bg_layer[1].tilemap_wider);
        assert!(state.ppu.bg_layer[1].tilemap_higher);
        assert_eq!(state.ppu.bg_layer[2].tilemap_adr, 0x6000);
        assert!(state.ppu.bg_layer[2].tilemap_wider);
        assert!(state.ppu.bg_layer[2].tilemap_higher);
    }

    #[test]
    fn triforce_poly_step0_falls_through_once_like_c() {
        let mut state = ZeldaState::new();
        state.attract_state_view_mut().set_intro_step_index(0);
        state.poly_state_view_mut().set_config1(10);
        state.ram[SUBSUBMODULE_INDEX] = 8;
        state.poly_state_view_mut().set_angle_a(7);
        state.poly_state_view_mut().set_angle_b(11);

        state.triforce_room_handle_poly();

        assert_eq!(state.poly_state_view().config1(), 8);
        assert_eq!(state.attract_state_view().intro_step_index(), 0);
        assert_eq!(state.ram[SUBSUBMODULE_INDEX], 8);
        assert_eq!(state.poly_state_view().angle_a(), 8);
        assert_eq!(state.poly_state_view().angle_b(), 13);
        assert_eq!(state.attract_state_view().intro_did_run_step(), 1);
        assert_eq!(state.ram[0x1e02], 0);
        assert_eq!(state.attract_state_view().intro_frame_counter(), 1);
    }

    #[test]
    fn credits_module_sets_oam_region_words_like_c() {
        let mut state = ZeldaState::new();
        state.set_submodule(38);
        for offset in 0..6 {
            state.ram[0x0fe0 + offset] = 0xff;
        }

        state.module1_a_credits();

        assert_eq!(read_le_u16(&state.ram, 0x0fe0), 0x0030);
        assert_eq!(read_le_u16(&state.ram, 0x0fe2), 0x01d0);
        assert_eq!(read_le_u16(&state.ram, 0x0fe4), 0x0000);
    }

    #[test]
    fn credits_prep_resets_sprite_properties_before_scene_setup() {
        let mut state = ZeldaState::new();
        state.ram[SUBMODULE_INDEX] = 0;
        let k = 15;
        for base in [
            SPRITE_PAUSE,
            SPRITE_E,
            SPRITE_X_VEL,
            SPRITE_Y_VEL,
            SPRITE_AI_STATE,
            SPRITE_A,
            SPRITE_DELAY_MAIN,
            SPRITE_OAM_FLAGS,
            SPRITE_STATE,
            SPRITE_FLAGS5,
            SPRITE_DEFL_BITS,
        ] {
            state.ram[base + k] = 0xa5;
        }

        state.credits_prep_and_load_sprites();

        for base in [
            SPRITE_PAUSE,
            SPRITE_E,
            SPRITE_X_VEL,
            SPRITE_Y_VEL,
            SPRITE_AI_STATE,
            SPRITE_A,
            SPRITE_DELAY_MAIN,
            SPRITE_OAM_FLAGS,
            SPRITE_STATE,
            SPRITE_FLAGS5,
            SPRITE_DEFL_BITS,
        ] {
            assert_eq!(state.ram[base + k], 0, "base ${base:04x}");
        }
    }

    #[test]
    fn credits_scene_fade_advances_scratch_when_fade_not_complete() {
        let mut state = ZeldaState::new();
        state.ram[SUBMODULE_INDEX] = 0;
        state.set_screen_brightness(2);
        state.ending_scratch_view_mut().set_primary_word(0x0300);

        state.credits_handle_scene_fade();

        assert_eq!(state.display_state().screen_brightness, 1);
        assert_eq!(state.ending_scratch_view().primary_word(), 0x0301);
        assert_eq!(state.ram[SUBMODULE_INDEX], 0);
    }

    #[test]
    fn credits_scene_fade_holds_scratch_when_fade_completes() {
        let mut state = ZeldaState::new();
        state.ram[SUBMODULE_INDEX] = 0;
        state.set_screen_brightness(1);
        state.ending_scratch_view_mut().set_primary_word(0x0300);

        state.credits_handle_scene_fade();

        assert_eq!(state.display_state().screen_brightness, 0);
        assert_eq!(state.ending_scratch_view().primary_word(), 0x0300);
        assert_eq!(state.ram[SUBMODULE_INDEX], 1);
    }

    #[test]
    fn overworld_map16_wram_slots_are_bridge_only() {
        let source = include_str!("overworld.rs");
        for symbol in [
            "MAP16_LOAD_SRC_OFF_OVERWORLD",
            "MAP16_LOAD_DST_OFF_OVERWORLD",
            "MAP16_LOAD_Y_UNIT_OVERWORLD",
            "MAP16_LOAD_SRC_OFF_PREV_OVERWORLD",
            "MAP16_LOAD_Y_UNIT_PREV_OVERWORLD",
            "MAP16_LOAD_DST_OFF_PREV_OVERWORLD",
            "MAP16_LOAD_SRC_OFF_SPEXIT_OVERWORLD",
            "MAP16_LOAD_SRC_OFF_EXIT_OVERWORLD",
            "ORANGE_BLUE_BARRIER_STATE_OVERWORLD",
            "SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF",
            "SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT",
        ] {
            let count = source.matches(symbol).count();
            assert!(
                (1..=2).contains(&count),
                "{symbol} should appear only at its const declaration and optional bridge write"
            );
        }
    }

    #[test]
    fn migrated_link_world_reads_use_semantic_views() {
        for (path, source) in [
            ("ancilla.rs", include_str!("ancilla.rs")),
            ("dungeon.rs", include_str!("dungeon.rs")),
            ("ending.rs", include_str!("ending.rs")),
            ("hud.rs", include_str!("hud.rs")),
            ("load_gfx.rs", include_str!("load_gfx.rs")),
            ("messaging.rs", include_str!("messaging.rs")),
            ("misc.rs", include_str!("misc.rs")),
            ("overlord.rs", include_str!("overlord.rs")),
            ("overworld.rs", include_str!("overworld.rs")),
            ("player.rs", include_str!("player.rs")),
            ("player_oam.rs", include_str!("player_oam.rs")),
            ("sprite.rs", include_str!("sprite.rs")),
            ("sprite_main.rs", include_str!("sprite_main.rs")),
            ("sprite_main_blind.rs", include_str!("sprite_main_blind.rs")),
            ("sprite_main_draw.rs", include_str!("sprite_main_draw.rs")),
            (
                "sprite_main_dungeon_npcs.rs",
                include_str!("sprite_main_dungeon_npcs.rs"),
            ),
            ("sprite_main_ganon.rs", include_str!("sprite_main_ganon.rs")),
            ("sprite_main_guard.rs", include_str!("sprite_main_guard.rs")),
            (
                "sprite_main_helmasaur_king.rs",
                include_str!("sprite_main_helmasaur_king.rs"),
            ),
            (
                "sprite_main_hinox_shop.rs",
                include_str!("sprite_main_hinox_shop.rs"),
            ),
            (
                "sprite_main_mothula.rs",
                include_str!("sprite_main_mothula.rs"),
            ),
            ("sprite_main_npcs.rs", include_str!("sprite_main_npcs.rs")),
            ("sprite_main_prep.rs", include_str!("sprite_main_prep.rs")),
            (
                "sprite_main_small_bosses.rs",
                include_str!("sprite_main_small_bosses.rs"),
            ),
            ("sprite_main_world.rs", include_str!("sprite_main_world.rs")),
            ("tile_detect.rs", include_str!("tile_detect.rs")),
        ] {
            for needle in [
                "self.native_ram_bridge_view().word_at(LINK_X_COORD)",
                "self.native_ram_bridge_view().word_at(LINK_Y_COORD)",
                "self.native_ram_bridge_view().word_at(LINK_Z_COORD)",
                "self.native_ram_bridge_view().word_at(OVERWORLD_SCREEN_INDEX)",
                "self.native_ram_bridge_view().word_at(DUNGEON_ROOM_INDEX)",
            ] {
                assert!(
                    !source.contains(needle),
                    "{path} should use typed semantic views for {needle}"
                );
            }
        }
    }

    #[test]
    fn migrated_player_coordinate_writes_use_semantic_views() {
        for (path, source) in [
            ("ancilla.rs", include_str!("ancilla.rs")),
            ("dungeon.rs", include_str!("dungeon.rs")),
            ("overworld.rs", include_str!("overworld.rs")),
            ("player.rs", include_str!("player.rs")),
            ("player_oam.rs", include_str!("player_oam.rs")),
            ("sprite.rs", include_str!("sprite.rs")),
            ("sprite_main_draw.rs", include_str!("sprite_main_draw.rs")),
            (
                "sprite_main_dungeon_npcs.rs",
                include_str!("sprite_main_dungeon_npcs.rs"),
            ),
            (
                "sprite_main_mothula.rs",
                include_str!("sprite_main_mothula.rs"),
            ),
            ("sprite_main_world.rs", include_str!("sprite_main_world.rs")),
            ("zelda_rtl.rs", include_str!("zelda_rtl.rs")),
        ] {
            for needle in [
                concat!(
                    "self.native_ram_bridge_view_mut().set_word_at(",
                    "LINK_X_COORD,"
                ),
                concat!(
                    "self.native_ram_bridge_view_mut().set_word_at(",
                    "LINK_Y_COORD,"
                ),
                concat!(
                    "self.native_ram_bridge_view_mut().set_word_at(",
                    "LINK_Z_COORD,"
                ),
            ] {
                assert!(
                    !source.contains(needle),
                    "{path} should use typed semantic views for {needle}"
                );
            }
        }
    }

    #[test]
    fn migrated_select_file_frame_control_uses_semantic_views() {
        for (path, source) in [
            ("ancilla.rs", include_str!("ancilla.rs")),
            ("attract.rs", include_str!("attract.rs")),
            ("audio.rs", include_str!("audio.rs")),
            ("dungeon.rs", include_str!("dungeon.rs")),
            ("ending.rs", include_str!("ending.rs")),
            ("hud.rs", include_str!("hud.rs")),
            ("load_gfx.rs", include_str!("load_gfx.rs")),
            ("messaging.rs", include_str!("messaging.rs")),
            ("misc.rs", include_str!("misc.rs")),
            ("overlord.rs", include_str!("overlord.rs")),
            ("overworld.rs", include_str!("overworld.rs")),
            ("player.rs", include_str!("player.rs")),
            ("player_oam.rs", include_str!("player_oam.rs")),
            ("select_file.rs", include_str!("select_file.rs")),
            ("sprite.rs", include_str!("sprite.rs")),
            ("sprite_main_draw.rs", include_str!("sprite_main_draw.rs")),
            (
                "sprite_main_dungeon_npcs.rs",
                include_str!("sprite_main_dungeon_npcs.rs"),
            ),
            ("sprite_main_ganon.rs", include_str!("sprite_main_ganon.rs")),
            ("sprite_main_guard.rs", include_str!("sprite_main_guard.rs")),
            (
                "sprite_main_mothula.rs",
                include_str!("sprite_main_mothula.rs"),
            ),
            ("sprite_main_npcs.rs", include_str!("sprite_main_npcs.rs")),
            ("sprite_main_prep.rs", include_str!("sprite_main_prep.rs")),
            (
                "sprite_main_small_bosses.rs",
                include_str!("sprite_main_small_bosses.rs"),
            ),
            ("sprite_main_world.rs", include_str!("sprite_main_world.rs")),
        ] {
            for needle in [
                "self.ram[MAIN_MODULE_INDEX]",
                "self.ram[SUBMODULE_INDEX]",
                "self.ram[SUBSUBMODULE_INDEX]",
            ] {
                assert!(
                    !source.contains(needle),
                    "{path} should use typed frame-control views for {needle}"
                );
            }
        }
    }

    #[test]
    fn zelda_rtl_frame_control_raw_slots_are_guard_needles_only() {
        let source = include_str!("zelda_rtl.rs");
        for needle in [
            concat!("self.", "ram[MAIN_MODULE_INDEX]"),
            concat!("self.", "ram[SUBMODULE_INDEX]"),
            concat!("self.", "ram[SUBSUBMODULE_INDEX]"),
        ] {
            assert_eq!(
                source.matches(needle).count(),
                1,
                "zelda_rtl.rs should only contain the guard needle for {needle}"
            );
        }
    }

    #[test]
    fn emu_callback_setup_syncs_whole_state_and_regions() {
        let mut state = ZeldaState::new();
        state.zelda_setup_emu_callbacks(Some(vec![0; 16]), None, Some(test_sync_all));

        state.emu_synchronize_whole_state();
        assert_eq!(state.ram[0x42], 1);

        state.ram[0x1234..0x1238].copy_from_slice(&[1, 2, 3, 4]);
        state.emu_sync_memory_region(0x1234, 4);
        let emu = state.emu_memory_ptr.as_ref().unwrap();
        assert_eq!(&emu[0x1234..0x1238], &[1, 2, 3, 4]);
    }

    #[test]
    fn byte_array_append_vl_matches_c_encoding() {
        let mut arr = ByteArray::default();

        ZeldaState::byte_array_append_vl(&mut arr, 0);
        ZeldaState::byte_array_append_vl(&mut arr, 254);
        ZeldaState::byte_array_append_vl(&mut arr, 255);
        ZeldaState::byte_array_append_vl(&mut arr, 511);

        assert_eq!(arr.data, vec![0, 254, 255, 0, 255, 255, 1]);

        let mut pos = 0usize;
        assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 0);
        assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 254);
        assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 255);
        assert_eq!(ZeldaState::state_recorder_read_vl(&arr.data, &mut pos), 511);
        assert_eq!(pos, arr.data.len());
    }

    #[test]
    fn save_and_load_func_append_and_copy_bytes() {
        let mut arr = ByteArray::default();
        let mut src = [1, 2, 3, 4];
        ZeldaState::save_func(&mut arr, &mut src);
        assert_eq!(arr.data, src);

        let mut st = LoadFuncState::new(&arr.data);
        let mut dst = [0; 4];
        ZeldaState::load_func(&mut st, &mut dst);
        assert_eq!(dst, src);
        assert_eq!(st.remaining(), 0);
    }

    #[test]
    fn snes_state_save_load_roundtrips_runtime_regions() {
        let mut state = ZeldaState::new();
        state.ram[0x100] = 0x12;
        write_le_u16(&mut state.ram, SEMANTIC_MAP16_LOAD_SRC_OFF, 0x1390);
        write_le_u16(&mut state.ram, SEMANTIC_MAP16_LOAD_DST_OFF, 0x001f);
        write_le_u16(&mut state.ram, SEMANTIC_MAP16_LOAD_Y_UNIT, 0x000e);
        state.sync_overworld_map16_load_from_ram();
        state.ram[HDMA_TABLE_DYNAMIC] = 0xab;
        state.sram[0x22] = 0x34;
        state.ppu.cgram[7] = 0x2468;
        state.ppu.bg_layer[1].tilemap_higher = true;
        state.ppu.bg_layer[1].tilemap_adr = 0x1357;
        state.dma.channel[6].a_adr = 0x4567;

        let mut arr = ByteArray::default();
        let mut save = SaveLoadFunc::Save(&mut arr);
        state.save_snes_state(&mut save);
        assert_eq!(state.ram[0x1b00], 0xab);

        state.ram[0x100] = 0;
        write_le_u16(&mut state.ram, SEMANTIC_MAP16_LOAD_SRC_OFF, 0);
        write_le_u16(&mut state.ram, SEMANTIC_MAP16_LOAD_DST_OFF, 0);
        write_le_u16(&mut state.ram, SEMANTIC_MAP16_LOAD_Y_UNIT, 0);
        state.sync_overworld_map16_load_from_ram();
        state.ram[HDMA_TABLE_DYNAMIC] = 0;
        state.sram[0x22] = 0;
        state.ppu.cgram[7] = 0;
        state.ppu.bg_layer[1].tilemap_higher = false;
        state.ppu.bg_layer[1].tilemap_adr = 0;
        state.dma.channel[6].a_adr = 0;

        let mut st = LoadFuncState::new(&arr.data);
        let mut load = SaveLoadFunc::Load(&mut st);
        state.load_snes_state(&mut load);

        assert_eq!(state.ram[0x100], 0x12);
        assert_eq!(
            state.overworld_map16_load_state(),
            OverworldMap16LoadState {
                src_off: 0x1390,
                dst_off: 0x001f,
                y_unit: 0x000e
            }
        );
        assert_eq!(state.ram[HDMA_TABLE_DYNAMIC], 0xab);
        assert_eq!(state.sram[0x22], 0x34);
        assert_eq!(state.ppu.cgram[7], 0x2468);
        assert!(state.ppu.bg_layer[1].tilemap_higher);
        assert_eq!(state.ppu.bg_layer[1].tilemap_adr, 0x1357);
        assert_eq!(state.dma.channel[6].a_adr, 0x4567);
    }

    #[test]
    fn state_recorder_records_input_edges_like_c() {
        let mut sr = StateRecorder {
            last_inputs: 0xffff,
            frames_since_last: 99,
            total_frames: 88,
            replay_mode: true,
            log: ByteArray { data: vec![1, 2] },
            base_snapshot: ByteArray { data: vec![3] },
            ..StateRecorder::default()
        };
        ZeldaState::state_recorder_init(&mut sr);
        assert_eq!(sr, StateRecorder::default());

        ZeldaState::state_recorder_record(&mut sr, 0x0001);
        ZeldaState::state_recorder_record(&mut sr, 0x0001);
        ZeldaState::state_recorder_record(&mut sr, 0x0003);

        assert_eq!(sr.last_inputs, 0x0003);
        assert_eq!(sr.frames_since_last, 1);
        assert_eq!(sr.total_frames, 3);
        assert_eq!(sr.log.data, vec![0x00, 0x12]);
    }

    #[test]
    fn state_recorder_records_long_waits_and_patch_bytes() {
        let mut sr = StateRecorder {
            frames_since_last: 20,
            ..StateRecorder::default()
        };
        ZeldaState::state_recorder_record_cmd(&mut sr, 0x00);
        assert_eq!(sr.frames_since_last, 0);
        assert_eq!(sr.log.data, vec![0x0f, 5]);

        ZeldaState::state_recorder_record_patch_byte(
            &mut sr,
            0x10020,
            &[0xaa, 0xbb, 0xcc, 0xdd, 0xee],
            5,
        );
        assert_eq!(
            sr.log.data,
            vec![0x0f, 5, 0xce, 1, 0x00, 0x20, 0xaa, 0xbb, 0xcc, 0xdd, 0xee]
        );
    }

    #[test]
    fn state_recorder_replays_input_edges_like_c() {
        let mut state = ZeldaState::new();
        let mut sr = StateRecorder {
            replay_mode: true,
            total_frames: 3,
            log: ByteArray {
                data: vec![0x00, 0x12],
            },
            ..StateRecorder::default()
        };

        assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0x0001);
        assert!(sr.replay_mode);
        assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0x0001);
        assert!(sr.replay_mode);
        assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0x0003);
        assert!(!sr.replay_mode);
    }

    #[test]
    fn state_recorder_replays_patch_bytes_and_can_stop() {
        let mut state = ZeldaState::new();
        let mut sr = StateRecorder {
            replay_mode: true,
            total_frames: 1,
            log: ByteArray {
                data: vec![0xce, 1, 0x00, 0x20, 0xaa, 0xbb, 0xcc, 0xdd, 0xee],
            },
            ..StateRecorder::default()
        };

        assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0);
        assert_eq!(
            &state.ram[0x10020..0x10025],
            &[0xaa, 0xbb, 0xcc, 0xdd, 0xee]
        );
        assert!(!sr.replay_mode);

        sr.replay_mode = true;
        sr.replay_frame_counter = 7;
        sr.replay_pos_last_complete = 3;
        ZeldaState::state_recorder_stop_replay(&mut sr);
        assert!(!sr.replay_mode);
        assert_eq!(sr.total_frames, 7);
        assert_eq!(sr.log.data, vec![0xce, 1, 0x00]);
    }

    #[test]
    fn state_recorder_replays_snapshot_boundary_commands() {
        let mut state = ZeldaState::new();
        state.ram[0x1234] = 0x5a;
        state.sram[0x234] = 0x6b;
        state.ppu.cgram[3] = 0x1357;

        let mut snapshot = ByteArray::default();
        let mut save = SaveLoadFunc::Save(&mut snapshot);
        state.save_snes_state(&mut save);

        state.ram[0x1234] = 0;
        state.sram[0x234] = 0;
        state.ppu.cgram[3] = 0;

        let mut log = ByteArray::default();
        ByteArray_AppendByte(&mut log, 0xd0);
        ZeldaState::byte_array_append_vl(&mut log, snapshot.size() as u32);
        ByteArray_AppendData(&mut log, &snapshot.data);

        let mut sr = StateRecorder {
            replay_mode: true,
            total_frames: 1,
            last_inputs: 0xffff,
            log,
            ..StateRecorder::default()
        };

        assert_eq!(state.state_recorder_read_next_replay_state(&mut sr), 0);
        assert_eq!(state.ram[0x1234], 0x5a);
        assert_eq!(state.sram[0x234], 0x6b);
        assert_eq!(state.ppu.cgram[3], 0x1357);
        assert_eq!(sr.last_inputs, 0);
        assert!(!sr.replay_mode);
    }

    #[test]
    fn state_recorder_clear_key_log_rebases_snapshot_and_active_inputs() {
        let mut state = ZeldaState::new();
        state.ram[0x100] = 0x56;
        let mut sr = StateRecorder {
            last_inputs: 0x0003,
            frames_since_last: 5,
            total_frames: 12,
            log: ByteArray {
                data: vec![0xaa, 0xbb],
            },
            ..StateRecorder::default()
        };

        state.state_recorder_clear_key_log(&mut sr);

        assert!(!sr.base_snapshot.data.is_empty());
        assert_eq!(sr.log.data, vec![0x00, 0x10]);
        assert_eq!(sr.frames_since_last, 0);
        assert_eq!(sr.total_frames, 0);
    }

    #[test]
    fn read_from_file_and_state_recorder_save_load_match_c_layout() {
        let mut cursor = std::io::Cursor::new(vec![1, 2, 3, 4]);
        let mut bytes = [0; 4];
        ZeldaState::read_from_file(&mut cursor, &mut bytes);
        assert_eq!(bytes, [1, 2, 3, 4]);

        let mut state = ZeldaState::new();
        state.ram[0x123] = 0x45;
        state.sram[0x234] = 0x67;
        let mut sr = StateRecorder {
            last_inputs: 0x00ff,
            frames_since_last: 7,
            total_frames: 9,
            log: ByteArray {
                data: vec![0x01, 0x23],
            },
            ..StateRecorder::default()
        };
        let mut out = Vec::new();
        state.state_recorder_save(&mut sr, &mut out);

        state.ram[0x123] = 0;
        state.sram[0x234] = 0;
        let mut loaded = StateRecorder::default();
        state.state_recorder_load(&mut loaded, &mut std::io::Cursor::new(out), false);

        assert_eq!(loaded.last_inputs, 0x00ff);
        assert_eq!(loaded.frames_since_last, 7);
        assert_eq!(loaded.total_frames, 9);
        assert_eq!(loaded.log.data, vec![0x01, 0x23]);
        assert_eq!(state.ram[0x123], 0x45);
        assert_eq!(state.sram[0x234], 0x67);
    }

    #[test]
    fn zelda_run_frame_sanitizes_inputs_and_records_features() {
        let mut state = ZeldaState::new();
        state.wanted_zelda_features = 0x1000;
        write_le_u16(&mut state.ram, ANIMATED_TILE_DATA_SRC, 1);

        let was_replay = state.zelda_run_frame(0x30 | 0xc0 | 1);

        assert!(!was_replay);
        assert_eq!(state.frame_ctr_dbg, 1);
        assert_eq!(state.state_recorder.last_inputs, 1);
        assert_eq!(state.ram[RAM_BUGS_FIXED], BUGFIX_LATEST);
        assert_eq!(state.read_u32_ram(ENHANCED_FEATURES0), 0x1000);
    }

    #[test]
    fn language_and_save_slot_shells_match_defaults() {
        let mut state = ZeldaState::new();
        state.dialogue_blk_index = 7;
        state.dialogue_font_blk_index = 8;
        state.dialogue_flags = 9;

        state.zelda_set_language(None);

        assert_eq!(state.dialogue_blk_index, 0);
        assert_eq!(state.dialogue_font_blk_index, 0);
        assert_eq!(state.dialogue_flags, 0);
        assert_eq!(
            ZeldaState::save_slot_path(SaveLoadCommand::Load, 3).unwrap(),
            PathBuf::from("saves/save3.sav")
        );
        assert!(ZeldaState::save_slot_path(SaveLoadCommand::Save, 256).is_none());
        assert_eq!(
            ZeldaState::save_slot_path(SaveLoadCommand::Replay, 256).unwrap(),
            Path::new("saves/ref").join("Chapter 1 - Zelda's Rescue.sav")
        );
    }

    #[test]
    fn multi_patch_and_patch_command_update_ram_and_log() {
        let mut state = ZeldaState::new();
        let mut sr = StateRecorder::default();
        let mut mp = StateRecoderMultiPatch::default();
        ZeldaState::state_recoder_multi_patch_init(&mut mp);

        state.state_recoder_multi_patch_patch(&mut sr, &mut mp, 0x20, 0xaa);
        state.state_recoder_multi_patch_patch(&mut sr, &mut mp, 0x21, 0xbb);
        ZeldaState::state_recoder_multi_patch_commit(&mut sr, &mut mp);

        assert_eq!(&state.ram[0x20..0x22], &[0xaa, 0xbb]);
        assert_eq!(sr.log.data, vec![0xc4, 0x00, 0x20, 0xaa, 0xbb]);

        state.patch_command('w');
        assert_eq!(state.ram[0xf372], 80);
        assert_eq!(state.ram[0xf373], 80);
        assert!(!state.state_recorder.log.data.is_empty());
    }

    #[test]
    fn item_receipt_applies_inventory_reward_side_effects() {
        let mut state = ZeldaState::new();
        state.ram[ITEM_RECEIPT_METHOD] = 0;
        let reward_addr = memory_location_to_give_item_to(0x24);
        state.ram[reward_addr] = 98;
        set_link_test_word(&mut state, LINK_X_COORD, 0x100);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x80);
        state.world_state_view_mut().set_bg2_x(0x40);

        set_link_test_byte(&mut state, LINK_RECEIVEITEM_INDEX, 0x24);
        state.ancilla_add_item_receipt(0x22, 4, 0);

        assert_eq!(state.ram[reward_addr], 99);
        assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0x22);
        assert_eq!(state.ram[ANCILLA_ITEM_TO_LINK + 4], 0x24);
        assert_eq!(state.ancilla_slot_view(4).work_byte_1(), 0);
        assert_eq!(state.ancilla_slot_view(4).work_byte_3(), 9);
        assert_eq!(state.ancilla_slot_view(4).work_byte_4(), 5);
        assert_eq!(state.system_signals_view().sound_effect_2(), 0x4f);
    }

    #[test]
    fn item_receipt_places_chest_item_with_c_offsets() {
        let mut state = ZeldaState::new();
        state.ram[ITEM_RECEIPT_METHOD] = 1;
        write_le_u16(&mut state.ram, DUNG_LOADE_BGOFFS_H_COPY, 0x1200);
        write_le_u16(&mut state.ram, DUNG_LOADE_BGOFFS_V_COPY, 0x3400);

        set_link_test_byte(&mut state, LINK_RECEIVEITEM_INDEX, 0);
        state.ancilla_add_item_receipt(0x22, 4, 0x0182);

        assert_eq!(state.ram[ANCILLA_X_LO + 4], 0x0c);
        assert_eq!(state.ram[ANCILLA_X_HI + 4], 0x12);
        assert_eq!(state.ram[ANCILLA_Y_LO + 4], 0x13);
        assert_eq!(state.ram[ANCILLA_Y_HI + 4], 0x34);
    }

    #[test]
    fn receive_item_enters_hold_item_state_for_normal_receipts() {
        let mut state = ZeldaState::new();
        state.ram[ITEM_RECEIPT_METHOD] = 0;
        state.player_state_view_mut().set_auxiliary_state(1);
        state.player_state_view_mut().set_incapacitated_timer(7);
        state.ram[COUNTDOWN_FOR_BLINK] = 8;
        state.player_state_view_mut().set_state_bits(0xff);
        state.player_state_view_mut().set_button_mask_b_y(0xff);
        state
            .player_state_view_mut()
            .set_y_button_action_flags(0xff);
        state.player_state_view_mut().set_button_b_frames(0xff);
        state.player_state_view_mut().set_speed_setting(0xff);
        set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 0xff);
        state.player_state_view_mut().set_item_in_hand(0xff);
        state.player_state_view_mut().set_position_mode(0xff);
        state.ram[PLAYER_HANDLER_TIMER] = 0xff;
        set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 0);

        state.link_receive_item(0x20, 0);

        assert_eq!(state.player_state_view().auxiliary_state(), 0);
        assert_eq!(state.player_state_view().incapacitated_timer(), 0);
        assert_eq!(state.ram[COUNTDOWN_FOR_BLINK], 0);
        assert_eq!(link_test_byte(&state, LINK_RECEIVEITEM_INDEX), 0x20);
        assert_eq!(link_test_byte(&state, LINK_ITEM_HOLDING_TIMER), 0x60);
        assert_eq!(state.player_state_view().state_bits(), 0);
        assert_eq!(state.player_state_view().button_mask_b_y(), 0);
        assert_eq!(state.player_state_view().y_button_action_flags(), 0);
        assert_eq!(state.player_state_view().button_b_frames(), 0);
        assert_eq!(state.player_state_view().speed_setting(), 0);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION), 0);
        assert_eq!(state.player_state_view().item_in_hand(), 0);
        assert_eq!(state.player_state_view().position_mode(), 0);
        assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
        assert_eq!(state.player_state_view().handler_state(), 21);
        assert_eq!(link_test_byte(&state, LINK_POSE_FOR_ITEM), 2);
        assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 1);
    }

    #[test]
    fn overworld_tile_attribute_uses_map16_and_map8_assets() {
        let mut state = ZeldaState::new();
        write_le_u16(&mut state.ram, OVERWORLD_OFFSET_BASE_Y, 0x20);
        write_le_u16(&mut state.ram, OVERWORLD_OFFSET_MASK_Y, 0x1f);
        write_le_u16(&mut state.ram, OVERWORLD_OFFSET_BASE_X, 3);
        write_le_u16(&mut state.ram, OVERWORLD_OFFSET_MASK_X, 0x3f);
        state.dungeon_state_view_mut().set_bg2_tile(32, 5);

        let mut data = vec![0; 0x100];
        write_le_u16(&mut data, (5 * 4 + 2) * 2, 0x4007);
        data[0x80 + 7] = 0x10;
        let mut ranges = vec![(0, 0); 164];
        ranges[70] = (0, 0x80);
        ranges[163] = (0x80, 0x100);
        state.assets = Some(AssetPack { data, ranges });

        assert_eq!(
            state.overworld_get_tile_attribute_at_location(4, 0x28),
            0x11
        );
    }

    #[test]
    fn outdoor_y_collision_starts_falling_into_pit() {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(0);
        state.tile_detect_position_view_mut().or_pit_tile(5);

        state.start_movement_collision_checks_y_handle_outdoors();

        assert_eq!(link_test_byte(&state, LINK_SPRITE_OAM_STATE_TIMER), 9);
        assert_eq!(state.player_state_view().near_pit_state(), 1);
        assert_eq!(state.player_state_view().handler_state(), 1);
    }

    #[test]
    fn outdoor_x_deepwater_without_flippers_hops_from_safe_return() {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(0);
        set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 3);
        set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 3);
        write_le_u16(&mut state.ram, TILEDETECT_DEEPWATER, 4);
        set_link_test_byte(&mut state, LINK_Y_COORD_SAFE_RETURN_LO, 0x34);
        set_link_test_byte(&mut state, LINK_Y_COORD_SAFE_RETURN_HI, 0x12);
        set_link_test_byte(&mut state, LINK_X_COORD_SAFE_RETURN_LO, 0x78);
        set_link_test_byte(&mut state, LINK_X_COORD_SAFE_RETURN_HI, 0x56);

        state.start_movement_collision_checks_x_handle_outdoors();

        assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x1234);
        assert_eq!(link_test_word(&state, LINK_X_COORD), 0x5678);
        assert_eq!(link_test_byte(&state, LINK_IS_IN_DEEP_WATER), 1);
        assert_eq!(state.player_state_view().swim_direction_flags(), 3);
        assert_eq!(state.player_state_view().actual_x_velocity(), 16);
        assert_eq!(state.player_state_view().actual_y_velocity(), 0);
        assert_eq!(state.player_state_view().actual_z_velocity(), 24);
        assert_eq!(state.player_state_view().incapacitated_timer(), 16);
        assert_eq!(state.player_state_view().auxiliary_state(), 1);
        assert_eq!(state.player_state_view().handler_state(), 6);
    }

    #[test]
    fn outdoor_y_spike_damage_rebounds_and_unequips_cape() {
        let mut state = ZeldaState::new();
        state
            .tile_detect_position_view_mut()
            .set_spike_cactus_tiles(1);
        set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 0);
        set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 1);
        set_link_test_byte(&mut state, LINK_ELECTROCUTE_ON_TOUCH, 1);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x40);

        state.start_movement_collision_checks_y_handle_outdoors();

        assert_eq!(link_test_byte(&state, LINK_GIVE_DAMAGE), 8);
        assert_eq!(link_test_byte(&state, LINK_BUNNY_TRANSFORM_TIMER), 32);
        assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
        assert_eq!(link_test_byte(&state, LINK_ELECTROCUTE_ON_TOUCH), 0);
        assert_eq!(state.player_state_view().actual_y_velocity(), 24);
        assert_eq!(state.player_state_view().actual_x_velocity(), 0);
        assert_eq!(state.player_state_view().actual_z_velocity(), 36);
        assert_eq!(state.player_state_view().incapacitated_timer(), 24);
        assert_eq!(state.player_state_view().auxiliary_state(), 1);
    }

    #[test]
    fn outdoor_x_spike_damage_applies_tile_rebound() {
        let mut state = ZeldaState::new();
        state
            .tile_detect_position_view_mut()
            .set_spike_cactus_tiles(1);
        set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 2);
        set_link_test_word(&mut state, LINK_X_COORD, 0x40);

        state.start_movement_collision_checks_x_handle_outdoors();

        assert_eq!(link_test_byte(&state, LINK_GIVE_DAMAGE), 8);
        assert_eq!(state.player_state_view().actual_x_velocity(), 24);
        assert_eq!(state.player_state_view().actual_y_velocity(), 0);
        assert_eq!(state.player_state_view().actual_z_velocity(), 36);
        assert_eq!(state.player_state_view().incapacitated_timer(), 24);
        assert_eq!(state.player_state_view().auxiliary_state(), 1);
    }

    #[test]
    fn outdoor_x_misc_bugfix_runs_slope_check_while_dashing_vertically() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_running_state(1);
        state.player_state_view_mut().set_facing(0);
        set_link_test_byte(&mut state, LINK_X_VEL, 1);
        state
            .tile_detect_position_view_mut()
            .set_slope_collision_bits(5);
        set_link_test_word(&mut state, LINK_X_COORD, 0x44);
        write_le_u16(&mut state.ram, ENHANCED_FEATURES0, 0x1000);

        state.start_movement_collision_checks_x_handle_outdoors();

        assert_eq!(link_test_word(&state, LINK_X_COORD), 0x40);
        assert_eq!(link_test_byte(&state, LINK_MOVING_AGAINST_DIAG_TILE), 0x25);
    }

    #[test]
    fn x_snap_helpers_match_doorway_adjustment_rules() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_X_VEL, 4);
        set_link_test_word(&mut state, LINK_X_COORD, 0x43);

        state.snap_on_x();

        assert_eq!(link_test_word(&state, LINK_X_COORD), 0x40);

        state.tile_detect_position_view_mut().set_collision_bits(4);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x80);
        state.calculate_snap_scratch_x();

        assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x7f);

        state.tile_detect_position_view_mut().set_collision_bits(1);
        set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 0);
        set_link_test_byte(&mut state, LINK_X_VEL, 4);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x90);
        let speed = state.change_axis_of_perpendicular_door_movement_x();

        assert_eq!(speed, -4);
        assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x8c);
        assert_eq!(state.player_state_view().facing(), 0);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 2, 2);
    }

    #[test]
    fn perform_dash_sets_start_dash_state_and_tagalong_timeout() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_somaria_platform_state(1);
        state.link_perform_dash();
        assert_eq!(state.player_state_view().handler_state(), 0);

        state.player_state_view_mut().set_somaria_platform_state(0);
        state
            .player_state_view_mut()
            .set_y_button_action_flags(0xff);
        state.player_state_view_mut().set_button_mask_b_y(0x7f);
        state.player_state_view_mut().set_state_bits(0x7f);
        state.player_state_view_mut().set_item_in_hand(3);
        state.ram[PLAYER_DEFENSE_FLAGS] = 0xff;
        set_link_test_byte(&mut state, LINK_MOVING_AGAINST_DIAG_TILE, 0xff);
        state.player_state_view_mut().set_speed_setting(5);
        state.ram[FOLLOWER_INDICATOR] = 2;

        state.link_perform_dash();

        assert_eq!(state.player_state_view().y_button_action_flags(), 0);
        assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 29);
        assert_eq!(link_test_byte(&state, LINK_DASH_CTR), 64);
        assert_eq!(state.player_state_view().handler_state(), 17);
        assert_eq!(state.player_state_view().running_state(), 1);
        assert_eq!(state.player_state_view().button_mask_b_y(), 0);
        assert_eq!(state.player_state_view().state_bits(), 0);
        assert_eq!(state.player_state_view().item_in_hand(), 0);
        assert_eq!(state.ram[PLAYER_DEFENSE_FLAGS], 0);
        assert_eq!(link_test_byte(&state, LINK_MOVING_AGAINST_DIAG_TILE), 0);
        assert_eq!(state.player_state_view().speed_setting(), 0);
        assert_eq!(read_le_u16(&state.ram, TIMER_TAGALONG_REACQUIRE), 64);
    }

    #[test]
    fn perform_rupee_pull_resets_properties_and_enters_pull_state() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_facing(2);
        state.link_perform_rupee_pull();
        assert_eq!(state.player_state_view().handler_state(), 0);

        state.player_state_view_mut().set_facing(0);
        state.player_state_view_mut().set_actual_y_velocity(9);
        state.player_state_view_mut().set_actual_x_velocity(8);
        state.player_state_view_mut().set_button_mask_b_y(0xff);
        set_link_test_byte(&mut state, LINK_CAPE_MODE, 1);

        state.link_perform_rupee_pull();

        assert_eq!(state.player_state_view().grabbing_wall(), 2);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 2, 2);
        assert_eq!(state.player_state_view().handler_state(), 29);
        assert_eq!(state.player_state_view().actual_y_velocity(), 0);
        assert_eq!(state.player_state_view().actual_x_velocity(), 0);
        assert_eq!(state.player_state_view().button_mask_b_y(), 0);
        assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 0);
    }

    #[test]
    fn handle_nudging_reverts_perpendicular_step_when_probe_blocks() {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(1);
        write_le_u16(&mut state.ram, TILEMAP_LOCATION_CALC_MASK, 0x01ff);
        set_link_test_word(&mut state, LINK_X_COORD, 0x20);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x20);
        set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 0);
        let offset = 0x145;
        state.dungeon_state_view_mut().set_bg2_attr(offset, 1);

        state.handle_nudging(1);

        assert_eq!(link_test_word(&state, LINK_X_COORD), 0x1f);
        assert_eq!(state.tile_detect_position_view().collision_bits(), 1);
    }

    #[test]
    fn ledge_hop_timer_restores_previous_position_until_triggered() {
        let mut state = ZeldaState::new();
        set_link_test_word(&mut state, LINK_Y_COORD, 0x120);
        set_link_test_word(&mut state, LINK_X_COORD, 0x240);
        set_link_test_word(&mut state, LINK_Y_COORD_PREV, 0x100);
        set_link_test_word(&mut state, LINK_X_COORD_PREV, 0x200);
        set_link_test_byte(&mut state, LINK_SUBPIXEL_Y, 3);
        set_link_test_byte(&mut state, LINK_SUBPIXEL_X, 4);
        set_link_test_byte(&mut state, LINK_TIMER_JUMP_LEDGE, 2);

        assert!(!state.run_ledge_hop_timer());
        assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x100);
        assert_eq!(link_test_word(&state, LINK_X_COORD), 0x200);
        assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_Y), 0);
        assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_X), 0);
    }

    #[test]
    fn slope_y_flag_adjusts_position_and_diag_bits() {
        let mut state = ZeldaState::new();
        set_link_test_word(&mut state, LINK_Y_COORD, 0x40);
        set_link_test_word(&mut state, LINK_X_COORD, 0x05);
        state
            .tile_detect_position_view_mut()
            .set_slope_collision_bits(1);
        state.tile_detect_position_view_mut().clear_diag_state();
        state.ram[TILEDETECT_WHICH_Y_POS] = 1;
        set_link_test_byte(&mut state, LINK_Y_VEL, 0xff);

        state.flag_moving_into_slopes_y();

        assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x44);
        assert_eq!(link_test_byte(&state, LINK_MOVING_AGAINST_DIAG_TILE), 0x19);
    }

    #[test]
    fn slope_x_flag_uses_second_y_probe_word_for_r12_bit_4() {
        let mut state = ZeldaState::new();
        set_link_test_word(&mut state, LINK_X_COORD, 0x41);
        state
            .tile_detect_position_view_mut()
            .set_slope_collision_bits(4);
        state.tile_detect_position_view_mut().clear_diag_state();
        write_le_u16(&mut state.ram, TILEDETECT_WHICH_Y_POS, 0);
        write_le_u16(&mut state.ram, TILEDETECT_WHICH_Y_POS + 2, 4);
        set_link_test_byte(&mut state, LINK_X_VEL, 0xff);

        state.flag_moving_into_slopes_x();

        assert_eq!(link_test_word(&state, LINK_X_COORD), 0x44);
        assert_eq!(link_test_byte(&state, LINK_MOVING_AGAINST_DIAG_TILE), 0x26);
    }

    #[test]
    fn indoor_y_doorway_collision_changes_to_perpendicular_axis() {
        let mut state = ZeldaState::new();
        state.ram[IS_STANDING_IN_DOORWAY] = 2;
        set_link_test_byte(&mut state, LINK_Y_VEL, 4);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x40);
        set_link_test_word(&mut state, LINK_X_COORD, 0x90);
        state.tile_detect_position_view_mut().set_collision_bits(1);

        state.start_movement_collision_checks_y_handle_indoors();

        assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x3c);
        assert_eq!(link_test_word(&state, LINK_X_COORD), 0x8f);
        assert_eq!(state.player_state_view().facing(), 4);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 2, 2);
    }

    #[test]
    fn dash_repel_applies_tile_rebound_state() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_running_state(1);
        set_link_test_byte(&mut state, LINK_DASH_CTR, 32);
        set_link_test_byte(&mut state, LINK_FLAG_MOVING, 2);
        set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 3);

        state.repel_dash();

        assert_eq!(
            state.player_state_view().actual_x_velocity(),
            0u8.wrapping_sub(24)
        );
        assert_eq!(state.player_state_view().actual_y_velocity(), 0);
        assert_eq!(state.player_state_view().actual_z_velocity(), 36);
        assert_eq!(state.player_state_view().incapacitated_timer(), 24);
        assert_eq!(state.player_state_view().auxiliary_state(), 1);
        assert_eq!(link_test_byte(&state, LINK_WANT_MAKE_NOISE_WHEN_DASHED), 1);
        assert_eq!(link_test_byte(&state, LINK_DIRECTION), 1);
        assert_eq!(state.swim_acceleration_view().acceleration(2), 256);
    }

    #[test]
    fn sprite_repel_dash_uses_facing_as_rebound_direction() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_running_state(1);
        set_link_test_byte(&mut state, LINK_DASH_CTR, 32);
        state.player_state_view_mut().set_facing(4);

        state.sprite_repel_dash();

        assert_eq!(link_test_byte(&state, LINK_LAST_DIRECTION_MOVED_TOWARDS), 2);
        assert_eq!(state.player_state_view().actual_x_velocity(), 24);
        assert_eq!(state.player_state_view().actual_y_velocity(), 0);
        assert_eq!(state.player_state_view().incapacitated_timer(), 24);
    }

    #[test]
    fn flag67_with_directions_derives_direction_from_actual_velocity() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_DIRECTION, 0xff);
        state.player_state_view_mut().set_actual_y_velocity(0xf0);
        state.player_state_view_mut().set_actual_x_velocity(2);

        state.flag67_with_directions();

        assert_eq!(link_test_byte(&state, LINK_DIRECTION), 9);
    }

    #[test]
    fn move_position_applies_sand_drag_to_velocity_delta() {
        let mut state = ZeldaState::new();
        set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
        state.player_state_view_mut().set_actual_x_velocity(16);
        state
            .player_state_view_mut()
            .set_actual_y_velocity(0u8.wrapping_sub(16));
        write_le_u16(&mut state.ram, DRAG_PLAYER_X, 1);
        write_le_u16(&mut state.ram, DRAG_PLAYER_Y, 0xffff);

        state.link_move_position();

        assert_eq!(link_test_word(&state, LINK_X_COORD), 0x0102);
        assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x01fe);
        assert_eq!(link_test_byte(&state, LINK_X_VEL), 2);
        assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0xfe);
        assert_eq!(link_test_byte(&state, LINK_X_COORD_SAFE_RETURN_LO), 0x00);
        assert_eq!(link_test_byte(&state, LINK_X_COORD_SAFE_RETURN_HI), 0x01);
        assert_eq!(link_test_byte(&state, LINK_Y_COORD_SAFE_RETURN_LO), 0x00);
        assert_eq!(link_test_byte(&state, LINK_Y_COORD_SAFE_RETURN_HI), 0x02);
    }

    #[test]
    fn move_position_applies_moving_floor_before_velocity_delta() {
        let mut state = ZeldaState::new();
        set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
        state.ram[DUNG_HDR_COLLISION] = 1;
        state.ram[PLAYER_LAYER_COLLISION_FLAGS] =
            crate::game_state::constants::player::LAYER_COLLISION_BOTH;
        write_le_u16(&mut state.ram, DUNG_FLOOR_X_VEL, 2);
        write_le_u16(&mut state.ram, DUNG_FLOOR_Y_VEL, 0xffff);

        state.link_move_position();

        assert_eq!(link_test_word(&state, LINK_X_COORD), 0x0102);
        assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x01ff);
        assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0x09);
        assert_eq!(link_test_byte(&state, LINK_X_VEL), 2);
        assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0xff);
    }

    #[test]
    fn swim_stroke_updates_subpixels_and_actual_velocity() {
        let mut state = ZeldaState::new();
        set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
        write_le_u16(&mut state.ram, SWIM_STROKE_FRAME_COUNTER, 1);
        write_le_u16(&mut state.ram, SWIM_STROKE_FRAME_COUNTER + 2, 1);
        state.swim_acceleration_view_mut().set_mode(0, 0);
        state.swim_acceleration_view_mut().set_mode(2, 0);
        state.swim_acceleration_view_mut().set_acceleration(0, 4);
        state.swim_acceleration_view_mut().set_acceleration(2, 4);
        state.swim_acceleration_view_mut().set_max_speed(0, 32);
        state.swim_acceleration_view_mut().set_max_speed(2, 32);
        state
            .swim_acceleration_view_mut()
            .set_acceleration_direction(0, 1);
        state
            .swim_acceleration_view_mut()
            .set_acceleration_direction(2, 1);

        state.handle_swim_stroke_and_subpixels();

        assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0x05);
        assert_eq!(state.swim_acceleration_view().acceleration(0), 12);
        assert_eq!(state.swim_acceleration_view().acceleration(2), 12);
        assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_X), 12);
        assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_Y), 12);
        assert_eq!(state.player_state_view().actual_x_velocity(), 0);
        assert_eq!(state.player_state_view().actual_y_velocity(), 0);
    }

    #[test]
    fn moving_animation_uses_some_direction_bits_when_flag_moving() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 1);
        set_link_test_byte(&mut state, LINK_FLAG_MOVING, 1);
        state.player_state_view_mut().set_swim_direction_flags(8);
        state.player_state_view_mut().set_joypad1h_last(8);

        state.link_handle_moving_animation_full_long_entry();

        assert_eq!(state.player_state_view().facing(), 0);
        assert_eq!(state.player_state_view().animation_step(), 1);
    }

    #[test]
    fn moving_animation_doorway_diagonal_picks_perpendicular_facing() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 5);
        set_link_test_byte(&mut state, LINK_NUM_ORTHOGONAL_DIRECTIONS, 1);
        state.ram[IS_STANDING_IN_DOORWAY] = 2;

        state.link_handle_moving_animation_full_long_entry();

        assert_eq!(state.player_state_view().facing(), 6);
    }

    #[test]
    fn moving_animation_dash_advances_dash_cycle() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 1);
        state.player_state_view_mut().set_running_state(1);
        set_link_test_byte(&mut state, LINK_COUNTDOWN_FOR_DASH, 32);
        set_link_test_byte(&mut state, LINK_FRAME_CHANGE_COUNTER, 1);

        state.link_handle_moving_animation_full_long_entry();

        assert_eq!(link_test_byte(&state, LINK_FRAME_CHANGE_COUNTER), 0);
        assert_eq!(state.player_state_view().animation_step(), 1);
    }

    #[test]
    fn apply_links_movement_to_camera_adjusts_crossed_x_quadrant() {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(1);
        state.ram[DUNG_LAYOUT_AND_STARTING_QUADRANT] = 0x20;
        state.set_dungeon_room(2);
        state.room_bounds_view_mut().set_x_bound(0, 0x0100);
        state.room_bounds_view_mut().set_x_bound(2, 0x0120);
        set_link_test_word(&mut state, LINK_X_COORD, 0x0102);
        set_link_test_byte(&mut state, LINK_X_COORD_SAFE_RETURN_HI, 0);
        set_link_test_byte(&mut state, LINK_Y_COORD_SAFE_RETURN_HI, 0);

        state.handle_indoor_camera_and_doors();

        assert_eq!(link_test_byte(&state, LINK_X_PAGE_MOVEMENT_DELTA), 1);
        assert_eq!(link_test_byte(&state, LINK_QUADRANT_X), 1);
        assert_eq!(state.ram[COMPOSITE_OF_LAYOUT_AND_QUADRANT], 0x21);
        assert_eq!(state.room_bounds_view().x_bound(0), 0x0200);
        assert_eq!(state.room_bounds_view().x_bound(2), 0x0220);
        assert_ne!(read_le_u16(&state.ram, DUNG_QUADRANTS_VISITED), 0);
    }

    #[test]
    fn doorway_east_transition_offsets_link_and_starts_subtile_transition() {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(1);
        state.ram[IS_STANDING_IN_DOORWAY] = 2;
        set_link_test_byte(&mut state, LINK_DIRECTION, 1);
        set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 1);
        state.set_main_module(7);
        state.ram[SUBMODULE_INDEX] = 0;
        set_link_test_word(&mut state, LINK_X_COORD, 0x00eb);
        set_link_test_byte(&mut state, LINK_X_COORD_SAFE_RETURN_HI, 0);
        state.set_dungeon_room(0x0104);
        state.room_bounds_view_mut().set_x_bound(0, 0x0100);
        state.room_bounds_view_mut().set_x_bound(1, 0x0120);

        state.handle_indoor_camera_and_doors();

        assert_eq!(link_test_byte(&state, LINK_X_PAGE_MOVEMENT_DELTA), 1);
        assert_eq!(link_test_word(&state, LINK_X_COORD), 0x00f3);
        assert_eq!(state.ram[SUBMODULE_INDEX], 1);
        assert_eq!(state.ram[OVERWORLD_SCREEN_TRANSITION], 2);
        assert_eq!(read_le_u16(&state.ram, LEFT_RIGHT_SCROLL_TARGET), 256);
        assert_eq!(read_le_u16(&state.ram, LEFT_RIGHT_SCROLL_TARGET_END), 0);
        assert_eq!(read_le_u16(&state.ram, CAMERA_X_COORD_SCROLL_LOW), 383);
    }

    #[test]
    fn doorway_south_transition_uses_vertical_camera_target_pair() {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(1);
        state.ram[IS_STANDING_IN_DOORWAY] = 1;
        set_link_test_byte(&mut state, LINK_DIRECTION, 4);
        set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 4);
        state.set_main_module(7);
        state.ram[SUBMODULE_INDEX] = 0;
        set_link_test_word(&mut state, LINK_Y_COORD, 0x00e4);
        set_link_test_byte(&mut state, LINK_Y_COORD_SAFE_RETURN_HI, 0);
        state.set_dungeon_room(0x0104);
        state.room_bounds_view_mut().set_y_bound(0, 0x0100);
        state.room_bounds_view_mut().set_y_bound(1, 0x0120);

        state.handle_indoor_camera_and_doors();

        assert_eq!(link_test_byte(&state, LINK_Y_PAGE_MOVEMENT_DELTA), 1);
        assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x00f4);
        assert_eq!(state.ram[SUBMODULE_INDEX], 1);
        assert_eq!(state.ram[OVERWORLD_SCREEN_TRANSITION], 0);
        assert_eq!(read_le_u16(&state.ram, UP_DOWN_SCROLL_TARGET), 256);
        assert_eq!(read_le_u16(&state.ram, UP_DOWN_SCROLL_TARGET_END), 16);
        assert_eq!(read_le_u16(&state.ram, CAMERA_Y_COORD_SCROLL_LOW), 376);
    }

    #[test]
    fn dungeon_edge_transition_right_starts_super_tile_scroll() {
        let mut state = ZeldaState::new();
        state.set_main_module(7);
        set_link_test_byte(&mut state, LINK_X_VEL, 1);
        set_link_test_byte(&mut state, LINK_DIRECTION, 1);
        set_link_test_word(&mut state, LINK_X_COORD, 0x01e9);
        state.set_dungeon_room(0x0104);
        state.room_bounds_view_mut().set_x_bound(0, 0x0100);
        state.room_bounds_view_mut().set_x_bound(1, 0x0120);

        state.Dungeon_TryScreenEdgeTransition();

        assert_eq!(link_test_byte(&state, LINK_QUADRANT_X), 1);
        assert_eq!(state.ram[SUBMODULE_INDEX], 2);
        assert_eq!(state.ram[OVERWORLD_SCREEN_TRANSITION], 2);
        assert_eq!(read_le_u16(&state.ram, LEFT_RIGHT_SCROLL_TARGET), 256);
        assert_eq!(read_le_u16(&state.ram, LEFT_RIGHT_SCROLL_TARGET_END), 0);
    }

    #[test]
    fn edge_transition_recoil_guard_restores_previous_position() {
        let mut state = ZeldaState::new();
        state.set_main_module(7);
        set_link_test_byte(&mut state, LINK_X_VEL, 1);
        state.player_state_view_mut().set_incapacitated_timer(5);
        state.player_state_view_mut().set_actual_x_velocity(12);
        state.player_state_view_mut().set_actual_y_velocity(34);
        set_link_test_word(&mut state, LINK_X_COORD, 0x01e9);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x0123);
        set_link_test_word(&mut state, LINK_X_COORD_PREV, 0x0088);
        set_link_test_word(&mut state, LINK_Y_COORD_PREV, 0x0099);

        state.Dungeon_TryScreenEdgeTransition();

        assert_eq!(state.player_state_view().actual_x_velocity(), 0);
        assert_eq!(state.player_state_view().actual_y_velocity(), 0);
        assert_eq!(link_test_byte(&state, LINK_RECOILMODE_TIMER), 3);
        assert_eq!(link_test_word(&state, LINK_X_COORD), 0x0088);
        assert_eq!(link_test_word(&state, LINK_Y_COORD), 0x0099);
        assert_eq!(state.ram[SUBMODULE_INDEX], 0);
    }

    #[test]
    fn recoil_z_velocity_shift_matches_c_do_while_condition() {
        fn run_recoil_step(initial_recoil_timer: u8) -> (u8, u8) {
            let mut state = ZeldaState::new();
            state.player_state_view_mut().set_handler_state(2);
            state.player_state_view_mut().set_auxiliary_state(1);
            state.player_state_view_mut().set_incapacitated_timer(8);
            set_link_test_byte(&mut state, LINK_RECOILMODE_TIMER, initial_recoil_timer);
            state.player_state_view_mut().set_actual_z_velocity(0xf8);
            set_link_test_byte(&mut state, LINK_ACTUAL_VEL_Z_COPY, 0x24);
            set_link_test_word(&mut state, LINK_Z_COORD, 0xffff);

            state.link_state_recoil();

            (
                link_test_byte(&state, LINK_RECOILMODE_TIMER),
                state.player_state_view().actual_z_velocity(),
            )
        }

        assert_eq!(run_recoil_step(0), (1, 0x09));
        assert_eq!(run_recoil_step(1), (2, 0x12));
        assert_eq!(run_recoil_step(2), (3, 0x12));
    }

    #[test]
    fn cache_camera_properties_if_outdoors_snapshots_scroll_state() {
        let mut state = ZeldaState::new();
        state.world_state_view_mut().set_bg2_x(0x1111);
        state.world_state_view_mut().set_bg2_y(0x2222);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x3333);
        set_link_test_word(&mut state, LINK_X_COORD, 0x4444);
        state.room_bounds_view_mut().set_y_bound(0, 0x5555);
        state.room_bounds_view_mut().set_x_bound(2, 0x6666);
        write_le_u16(&mut state.ram, UP_DOWN_SCROLL_TARGET, 0x7777);
        write_le_u16(&mut state.ram, LEFT_RIGHT_SCROLL_TARGET_END, 0x8888);
        write_le_u16(&mut state.ram, CAMERA_Y_COORD_SCROLL_LOW, 0x9999);
        state.ram[QUADRANT_FULLSIZE_Y] = 2;
        set_link_test_byte(&mut state, LINK_QUADRANT_Y, 2);
        state.player_state_view_mut().set_facing(8);
        state.player_state_view_mut().set_lower_level_state(1);
        state.ram[IS_STANDING_IN_DOORWAY] = 2;
        state.ram[DUNG_CUR_FLOOR] = 0xff;

        state.cache_camera_properties_if_outdoors();

        assert_eq!(state.ppu_scroll_copy_view().bg2_h_copy2_cached(), 0x1111);
        assert_eq!(state.ppu_scroll_copy_view().bg2_v_copy2_cached(), 0x2222);
        assert_eq!(link_test_word(&state, LINK_Y_COORD_CACHED), 0x3333);
        assert_eq!(link_test_word(&state, LINK_X_COORD_CACHED), 0x4444);
        assert_eq!(read_le_u16(&state.ram, CACHED_ROOM_BOUNDS_Y_START), 0x5555);
        assert_eq!(read_le_u16(&state.ram, CACHED_ROOM_BOUNDS_X_END), 0x6666);
        assert_eq!(
            read_le_u16(&state.ram, UP_DOWN_SCROLL_TARGET_CACHED),
            0x7777
        );
        assert_eq!(
            read_le_u16(&state.ram, LEFT_RIGHT_SCROLL_TARGET_END_CACHED),
            0x8888
        );
        assert_eq!(
            read_le_u16(&state.ram, CAMERA_Y_COORD_SCROLL_LOW_CACHED),
            0x9999
        );
        assert_eq!(state.ram[QUADRANT_FULLSIZE_Y_CACHED], 2);
        assert_eq!(link_test_byte(&state, LINK_QUADRANT_Y_CACHED), 2);
        assert_eq!(link_test_byte(&state, LINK_DIRECTION_FACING_CACHED), 8);
        assert_eq!(link_test_byte(&state, LINK_IS_ON_LOWER_LEVEL_CACHED), 1);
        assert_eq!(state.ram[IS_STANDING_IN_DOORWAY_CACHED], 2);
        assert_eq!(state.ram[DUNG_CUR_FLOOR_CACHED], 0xff);
    }

    #[test]
    fn bit_sum_and_swim_permission_helpers_match_c_state_writes() {
        assert_eq!(ZeldaState::bit_sum4(0b1011_1111), 4);
        assert_eq!(ZeldaState::bit_sum4(0), 0);

        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_ITEM_FLIPPERS, 1);
        state.check_ability_to_swim();
        assert_eq!(link_test_byte(&state, LINK_VISIBILITY_STATUS), 0);

        set_link_test_byte(&mut state, LINK_IS_BUNNY_MIRROR, 1);
        set_link_test_byte(&mut state, LINK_ITEM_MOON_PEARL, 1);
        state.set_indoor_flag(1);
        state.check_ability_to_swim();

        assert_eq!(link_test_byte(&state, LINK_IS_BUNNY_MIRROR), 0);
        assert_eq!(link_test_byte(&state, LINK_VISIBILITY_STATUS), 0x0c);
        assert_eq!(state.ram[SUBMODULE_INDEX], 20);
    }

    #[test]
    fn dungeon_layer_change_updates_floor_room_and_visited_flags() {
        let mut state = ZeldaState::new();
        state.set_dungeon_room(0x0104);
        state.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 1;
        state.ram[QUADRANT_FULLSIZE_Y] = 1;
        state.ram[QUADRANT_FULLSIZE_X] = 1;
        set_link_test_byte(&mut state, LINK_QUADRANT_Y, 1);
        set_link_test_byte(&mut state, LINK_QUADRANT_X, 1);

        state.dungeon_handle_layer_change();

        assert_eq!(state.world_location_state().dungeon_room, 0x0114);
        assert_eq!(link_test_byte(&state, LINK_IS_ON_LOWER_LEVEL_MIRROR), 1);
        assert_eq!(state.player_state_view().lower_level_state(), 1);
        assert_eq!(state.ram[ABOUT_TO_JUMP_OFF_LEDGE], 0);
        assert_ne!(read_le_u16(&state.ram, DUNG_QUADRANTS_VISITED), 0);

        write_le_u16(&mut state.ram, KIND_OF_IN_ROOM_STAIRCASE, 2);
        state.player_state_view_mut().set_lower_level_state(0);
        state.dungeon_handle_layer_change();
        assert_eq!(state.player_state_view().lower_level_state(), 0);
    }

    #[test]
    fn link_initialize_applies_misc_bugfix_cleanup() {
        let mut state = ZeldaState::new();
        write_le_u16(&mut state.ram, ENHANCED_FEATURES0, 0x1000);
        state.player_state_view_mut().set_button_mask_b_y(0xff);
        state.ram[ABOUT_TO_JUMP_OFF_LEDGE] = 1;
        set_link_test_byte(&mut state, LINK_IS_NEAR_MOVEABLE_STATUE, 1);
        set_link_test_byte(&mut state, LINK_ON_CONVEYOR_BELT, 1);
        set_link_test_byte(&mut state, LINK_FLAG_MOVING, 1);
        write_le_u16(&mut state.ram, BG1_Y_OFFSET, 0x1234);
        write_le_u16(&mut state.ram, BG1_X_OFFSET, 0x5678);
        state.ram[SAVEGAME_IS_DARKWORLD] = 1;

        state.link_initialize();

        assert_eq!(state.player_state_view().facing(), 2);
        assert_eq!(state.player_state_view().button_mask_b_y() & 0x40, 0);
        assert_eq!(state.ram[ABOUT_TO_JUMP_OFF_LEDGE], 0);
        assert_eq!(link_test_byte(&state, LINK_IS_NEAR_MOVEABLE_STATUE), 0);
        assert_eq!(link_test_byte(&state, LINK_ON_CONVEYOR_BELT), 0);
        assert_eq!(link_test_byte(&state, LINK_FLAG_MOVING), 0);
        assert_eq!(read_le_u16(&state.ram, BG1_Y_OFFSET), 0);
        assert_eq!(read_le_u16(&state.ram, BG1_X_OFFSET), 0);
        assert_eq!(state.player_state_view().handler_state(), 23);
        assert_eq!(link_test_byte(&state, LINK_IS_BUNNY), 1);
        assert_eq!(link_test_byte(&state, LINK_IS_BUNNY_MIRROR), 1);
    }

    #[test]
    fn link_reset_properties_a_clears_reset_chain_state() {
        let mut state = ZeldaState::new();
        state.ram.fill(0xff);
        write_le_u16(&mut state.ram, ENHANCED_FEATURES0, 0x1000);

        state.link_reset_properties_a();

        assert_eq!(link_test_byte(&state, LINK_DIRECTION_LAST), 0);
        assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0);
        assert_eq!(link_test_byte(&state, LINK_FLAG_MOVING), 0);
        assert_eq!(state.ram[SWIMMING_COUNTDOWN], 0);
        assert_eq!(link_test_byte(&state, LINK_SWIM_HARD_STROKE), 0);
        assert_eq!(link_test_byte(&state, LINK_MAYBE_SWIM_FASTER), 0);
        assert_eq!(state.swim_acceleration_view().speed_active_flag(0), 0);
        assert_eq!(link_test_byte(&state, LINK_IS_BUNNY), 0);
        assert_eq!(link_test_byte(&state, LINK_IS_BUNNY_MIRROR), 0);
        assert_eq!(link_test_byte(&state, LINK_TIMER_TEMPBUNNY), 0);
        assert_eq!(state.player_state_view().on_somaria_platform(), 0);
        assert_eq!(state.ram[PLAYER_DEFENSE_FLAGS], 0);
        assert_eq!(state.player_state_view().near_pit_state(), 0);
        assert_eq!(state.ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE], 0);
        assert_eq!(read_le_u16(&state.ram, TILEDETECT_MISC_TILES), 0);
        assert_eq!(state.player_state_view().button_mask_b_y(), 0);
        assert_eq!(state.player_state_view().state_bits(), 0);
        assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 0);
        assert_eq!(state.player_state_view().hookshot_interlock(), 0);
        assert_eq!(link_test_byte(&state, LINK_IS_NEAR_MOVEABLE_STATUE), 0);
    }

    #[test]
    fn damaging_pit_reset_restores_ground_or_permabunny_state() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_IS_BUNNY, 1);
        set_link_test_byte(&mut state, LINK_ITEM_MOON_PEARL, 0);
        state.player_state_view_mut().set_swim_direction_flags(8);
        set_link_test_byte(&mut state, LINK_IS_IN_DEEP_WATER, 1);
        set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 1);
        state.player_state_view_mut().set_pit_data_index(1);
        state.ram[SWIMMING_COUNTDOWN] = 7;
        state
            .swim_acceleration_view_mut()
            .set_speed_active_flag(0, 0x1234);

        state.link_reset_state_after_damaging_pit();

        assert_eq!(state.player_state_view().handler_state(), 23);
        assert_eq!(link_test_byte(&state, LINK_DIRECTION_LAST), 8);
        assert_eq!(link_test_byte(&state, LINK_IS_IN_DEEP_WATER), 0);
        assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
        assert_eq!(state.player_state_view().pit_data_index(), 0);
        assert_eq!(state.ram[SWIMMING_COUNTDOWN], 0);
        assert_eq!(state.swim_acceleration_view().speed_active_flag(0), 0);

        set_link_test_byte(&mut state, LINK_ITEM_MOON_PEARL, 1);
        state.player_state_view_mut().set_handler_state(6);
        state.link_reset_state_after_damaging_pit();
        assert_eq!(state.player_state_view().handler_state(), 0);
    }

    #[test]
    fn z_velocity_change_matches_recoil_and_turtle_rock_rules() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_actual_z_velocity(10);
        state.player_change_z(2);
        assert_eq!(state.player_state_view().actual_z_velocity(), 8);

        state.player_state_view_mut().set_handler_state(19);
        state.link_handle_change_in_z_velocity();
        assert_eq!(state.player_state_view().actual_z_velocity(), 7);

        state.player_state_view_mut().set_actual_z_velocity(0xfe);
        set_link_test_byte(&mut state, LINK_Z_COORD, 0);
        state.link_handle_change_in_z_velocity();
        assert_eq!(state.player_state_view().actual_z_velocity(), 0xfe);

        set_link_test_byte(&mut state, LINK_Z_COORD, 0x80);
        state.link_handle_change_in_z_velocity();
        assert_eq!(link_test_word(&state, LINK_Z_COORD), 0xffff);
        assert_eq!(state.player_state_view().actual_z_velocity(), 0xff);
    }

    #[test]
    fn set_to_deep_water_resets_swim_state_and_latches_direction() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 8);
        state.player_state_view_mut().set_grabbing_wall(1);
        state.player_state_view_mut().set_speed_setting(2);
        state.ram[SWIMMING_COUNTDOWN] = 7;
        state
            .swim_acceleration_view_mut()
            .set_acceleration(0, 0x1234);

        state.link_set_to_deep_water();

        assert_eq!(link_test_byte(&state, LINK_IS_IN_DEEP_WATER), 1);
        assert_eq!(state.player_state_view().swim_direction_flags(), 8);
        assert_eq!(state.player_state_view().grabbing_wall(), 0);
        assert_eq!(state.player_state_view().speed_setting(), 0);
        assert_eq!(state.ram[SWIMMING_COUNTDOWN], 0);
        assert_eq!(state.swim_acceleration_view().acceleration(0), 0);
    }

    #[test]
    fn splash_upon_landing_routes_deep_water_and_bunny_states() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_IS_IN_DEEP_WATER, 1);
        set_link_test_byte(&mut state, LINK_CAPE_MODE, 1);
        set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 1);
        set_link_test_byte(&mut state, LINK_ELECTROCUTE_ON_TOUCH, 1);

        state.link_splash_upon_landing();

        assert_eq!(state.player_state_view().handler_state(), 4);
        assert_eq!(state.system_signals_view().sound_effect_1() & 0x3f, 0x24);
        assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 0);
        assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
        assert_eq!(link_test_byte(&state, LINK_ELECTROCUTE_ON_TOUCH), 0);

        let mut bunny = ZeldaState::new();
        set_link_test_byte(&mut bunny, LINK_IS_BUNNY_MIRROR, 1);
        set_link_test_byte(&mut bunny, LINK_IS_IN_DEEP_WATER, 1);
        set_link_test_byte(&mut bunny, LINK_ITEM_MOON_PEARL, 1);
        set_link_test_byte(&mut bunny, LINK_IS_BUNNY, 1);
        bunny.player_state_view_mut().set_animation_step(3);
        bunny
            .swim_acceleration_view_mut()
            .set_speed_active_flag(0, 0x1111);

        bunny.link_splash_upon_landing();

        assert_eq!(bunny.player_state_view().handler_state(), 0);
        assert_eq!(link_test_byte(&bunny, LINK_IS_BUNNY), 0);
        assert_eq!(bunny.player_state_view().auxiliary_state(), 0);
        assert_eq!(bunny.player_state_view().animation_step(), 0);
        assert_eq!(bunny.swim_acceleration_view().speed_active_flag(0), 0);
    }

    #[test]
    fn swim_accels_start_ramp_and_snap_to_table() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_joypad1h_last(0x0d);
        state.swim_acceleration_view_mut().set_acceleration(0, 0);
        state.swim_acceleration_view_mut().set_max_speed(0, 0);
        state.swim_acceleration_view_mut().set_acceleration(2, 260);
        state.swim_acceleration_view_mut().set_max_speed(2, 384);

        state.link_handle_swim_accels();

        assert_eq!(state.swim_acceleration_view().acceleration(0), 1);
        assert_eq!(state.swim_acceleration_view().max_speed(0), 240);
        assert_eq!(state.swim_acceleration_view().max_speed(2), 288);

        state.link_handle_swim_accels();
        assert_eq!(state.swim_acceleration_view().max_speed(0), 384);
    }

    #[test]
    fn swim_flag_max_accels_promotes_active_axes() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_FLAG_MOVING, 1);
        state
            .swim_acceleration_view_mut()
            .set_acceleration(0, 0x0110);
        state.swim_acceleration_view_mut().set_acceleration(2, 0);
        state.swim_acceleration_view_mut().set_max_speed(2, 0x2222);

        state.link_flag_max_accels();

        assert_eq!(state.swim_acceleration_view().max_speed(0), 0x0110);
        assert_eq!(state.swim_acceleration_view().mode(0), 1);
        assert_eq!(state.swim_acceleration_view().max_speed(2), 0x2222);
        assert_eq!(state.swim_acceleration_view().mode(2), 0);
    }

    #[test]
    fn swim_ice_max_accel_sets_both_axes_when_flag_moving() {
        let mut state = ZeldaState::new();
        state.swim_acceleration_view_mut().set_max_speed(0, 0x1111);

        state.link_set_ice_max_accel();
        assert_eq!(state.swim_acceleration_view().max_speed(0), 0x1111);

        set_link_test_byte(&mut state, LINK_FLAG_MOVING, 1);
        state.link_set_ice_max_accel();

        assert_eq!(state.swim_acceleration_view().max_speed(0), 0x0180);
        assert_eq!(state.swim_acceleration_view().max_speed(2), 0x0180);
    }

    #[test]
    fn swim_momentum_sets_direction_and_starting_accel() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_joypad1h_last(0x09);
        set_link_test_byte(&mut state, LINK_FLAG_MOVING, 2);
        state.player_state_view_mut().set_swim_direction_flags(0x04);
        set_link_test_byte(&mut state, LINK_DIRECTION, 0x08);
        state.swim_acceleration_view_mut().set_max_speed(2, 0x1234);

        state.link_set_momentum();

        assert_eq!(read_le_u16(&state.ram, SWIM_STROKE_FRAME_COUNTER), 8);
        assert_eq!(state.swim_acceleration_view().mode(0), 2);
        assert_eq!(state.swim_acceleration_view().max_speed(0), 240);
        assert_eq!(read_le_u16(&state.ram, SWIM_STROKE_FRAME_COUNTER + 2), 8);
        assert_eq!(state.swim_acceleration_view().mode(2), 0);
        assert_eq!(state.swim_acceleration_view().acceleration_direction(2), 1);
        assert_eq!(state.swim_acceleration_view().max_speed(2), 0x1234);
    }

    #[test]
    fn reset_all_acceleration_clears_swim_accel_pairs() {
        let mut state = ZeldaState::new();
        for offset in [0, 2] {
            state
                .swim_acceleration_view_mut()
                .set_speed_active_flag(offset, 0xffff);
            state.swim_acceleration_view_mut().set_mode(offset, 0xffff);
            state
                .swim_acceleration_view_mut()
                .set_acceleration(offset, 0xffff);
            state
                .swim_acceleration_view_mut()
                .set_max_speed(offset, 0xffff);
        }
        for offset in [SWIM_STROKE_FRAME_COUNTER, SWIM_STROKE_FRAME_COUNTER + 2] {
            write_le_u16(&mut state.ram, offset, 0xffff);
        }
        state
            .swim_acceleration_view_mut()
            .set_acceleration_direction(0, 0xffff);

        state.reset_all_acceleration();

        for offset in [0, 2] {
            assert_eq!(state.swim_acceleration_view().speed_active_flag(offset), 0);
            assert_eq!(state.swim_acceleration_view().mode(offset), 0);
            assert_eq!(state.swim_acceleration_view().acceleration(offset), 0);
            assert_eq!(state.swim_acceleration_view().max_speed(offset), 0);
        }
        for offset in [SWIM_STROKE_FRAME_COUNTER, SWIM_STROKE_FRAME_COUNTER + 2] {
            assert_eq!(read_le_u16(&state.ram, offset), 0);
        }
        assert_eq!(
            state.swim_acceleration_view().acceleration_direction(0),
            0xffff
        );
    }

    #[test]
    fn swimming_handler_without_flippers_only_clears_action_state() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_button_mask_b_y(0xff);
        state.player_state_view_mut().set_button_b_frames(9);
        set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 7);
        set_link_test_byte(&mut state, LINK_SPIN_ATTACK_STEP_COUNTER, 6);
        state.player_state_view_mut().set_state_bits(5);
        state.player_state_view_mut().set_picking_throw_state(4);
        set_link_test_byte(&mut state, LINK_ITEM_FLIPPERS, 0);

        state.player_handler_04_swimming();

        assert_eq!(state.player_state_view().button_mask_b_y(), 0);
        assert_eq!(state.player_state_view().button_b_frames(), 0);
        assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 0);
        assert_eq!(link_test_byte(&state, LINK_SPIN_ATTACK_STEP_COUNTER), 0);
        assert_eq!(state.player_state_view().state_bits(), 0);
        assert_eq!(state.player_state_view().picking_throw_state(), 0);
    }

    #[test]
    fn swimming_handler_starts_hard_stroke_and_advances_swim_animation() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_handler_state(4);
        set_link_test_byte(&mut state, LINK_ITEM_FLIPPERS, 1);
        set_link_test_byte(&mut state, LINK_FRAME_CHANGE_COUNTER, 7);
        state.player_state_view_mut().set_filtered_joypad_l(0x80);
        state.player_state_view_mut().set_joypad1h_last(8);
        state.swim_acceleration_view_mut().set_acceleration(0, 1);

        state.player_handler_04_swimming();

        assert_eq!(link_test_byte(&state, LINK_FRAME_CHANGE_COUNTER), 0);
        assert_eq!(state.player_state_view().animation_step(), 1);
        assert_eq!(state.ram[SWIM_STROKE_ANIM_STEP], 0);
        assert_eq!(link_test_byte(&state, LINK_SWIM_HARD_STROKE), 0x80);
        assert_eq!(link_test_byte(&state, LINK_MAYBE_SWIM_FASTER), 1);
        assert_eq!(state.ram[SWIMMING_COUNTDOWN], 6);
        assert_eq!(state.system_signals_view().sound_effect_1() & 0x3f, 37);
        assert_eq!(state.player_state_view().swim_direction_flags(), 8);
    }

    #[test]
    fn swim_movement_without_input_resets_idle_flag_moving_state() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_handler_state(4);
        set_link_test_byte(&mut state, LINK_FLAG_MOVING, 1);
        state.ram[PLAYER_DEFENSE_FLAGS] = 0xff;
        state.ram[PIT_CORRECTION_ACTIVE_FLAG] = 1;
        state
            .swim_acceleration_view_mut()
            .set_speed_active_flag(0, 0x1111);

        state.link_handle_swim_movements();

        assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0);
        assert_eq!(link_test_byte(&state, LINK_X_VEL), 0);
        assert_eq!(state.ram[PLAYER_DEFENSE_FLAGS] & 0x0f, 0);
        assert_eq!(state.swim_acceleration_view().speed_active_flag(0), 0);
        assert_eq!(state.ram[PIT_CORRECTION_ACTIVE_FLAG], 0);
    }

    #[test]
    fn set_the_max_accel_marks_ready_axis_and_resets_inactive_axis() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_joypad1h_last(0x0c);
        state.swim_acceleration_view_mut().set_acceleration(0, 260);
        state.swim_acceleration_view_mut().set_max_speed(0, 240);
        state.swim_acceleration_view_mut().set_mode(0, 0);
        state
            .swim_acceleration_view_mut()
            .set_speed_active_flag(2, 1);
        state.swim_acceleration_view_mut().set_max_speed(2, 384);

        state.link_set_the_max_accel();

        assert_eq!(state.swim_acceleration_view().speed_active_flag(0), 1);
        assert_eq!(state.swim_acceleration_view().mode(0), 1);
        assert_eq!(state.swim_acceleration_view().speed_active_flag(2), 0);
        assert_eq!(state.swim_acceleration_view().max_speed(2), 240);

        set_link_test_byte(&mut state, LINK_SWIM_HARD_STROKE, 1);
        state
            .swim_acceleration_view_mut()
            .set_speed_active_flag(0, 0);
        state.link_set_the_max_accel();
        assert_eq!(state.swim_acceleration_view().speed_active_flag(0), 0);
        assert_eq!(state.swim_acceleration_view().mode(0), 1);
    }

    #[test]
    fn handle_toss_clears_a_press_state_when_throwing() {
        let mut state = ZeldaState::new();
        state
            .player_state_view_mut()
            .set_y_button_action_flags(0x80);
        state.player_state_view_mut().set_filtered_joypad_l(0x80);
        state.player_state_view_mut().set_item_action_step_var(7);
        state.player_state_view_mut().set_throw_oam_state_index(8);
        state.player_state_view_mut().set_y_button_action_step(9);
        set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 0xff);

        assert!(state.link_handle_toss());

        assert_eq!(state.player_state_view().item_action_step_var(), 0);
        assert_eq!(state.player_state_view().throw_oam_state_index(), 0);
        assert_eq!(state.player_state_view().y_button_action_step(), 0);
        assert_eq!(state.player_state_view().y_button_action_flags(), 0);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);

        state
            .player_state_view_mut()
            .set_y_button_action_flags(0x80);
        state.player_state_view_mut().set_filtered_joypad_l(0x80);
        state.player_state_view_mut().set_picking_throw_state(1);
        assert!(!state.link_handle_toss());
        assert_eq!(state.player_state_view().y_button_action_flags(), 0x80);
    }

    #[test]
    fn halt_link_when_using_items_stops_floor_and_platform_motion() {
        let mut state = ZeldaState::new();
        state.ram[DUNG_HDR_COLLISION_2] = 2;
        state.ram[PLAYER_LAYER_COLLISION_FLAGS] =
            crate::game_state::constants::player::LAYER_COLLISION_BOTH;
        set_link_test_byte(&mut state, LINK_Y_VEL, 0x80);
        set_link_test_byte(&mut state, LINK_X_VEL, 0x40);
        set_link_test_byte(&mut state, LINK_DIRECTION, 0x0f);
        set_link_test_byte(&mut state, LINK_SUBPIXEL_Y, 0x55);
        set_link_test_byte(&mut state, LINK_SUBPIXEL_X, 0xaa);
        set_link_test_byte(&mut state, LINK_MOVING_AGAINST_DIAG_TILE, 1);

        state.halt_link_when_using_items();

        assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0);
        assert_eq!(link_test_byte(&state, LINK_X_VEL), 0);
        assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0);
        assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_Y), 0);
        assert_eq!(link_test_byte(&state, LINK_SUBPIXEL_X), 0);
        assert_eq!(link_test_byte(&state, LINK_MOVING_AGAINST_DIAG_TILE), 0);

        state.ram[DUNG_HDR_COLLISION_2] = 0;
        state.ram[PLAYER_LAYER_COLLISION_FLAGS] = 0;
        state.player_state_view_mut().set_somaria_platform_state(1);
        set_link_test_byte(&mut state, LINK_Y_VEL, 7);
        set_link_test_byte(&mut state, LINK_DIRECTION, 0x0f);
        state.halt_link_when_using_items();
        assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0);
        assert_eq!(link_test_byte(&state, LINK_Y_VEL), 7);
    }

    #[test]
    fn cape_lift_and_active_item_paths_drain_and_unequip() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_CAPE_MODE, 1);
        state.ram[CURRENT_ITEM_ACTIVE] = 19;
        state.ram[CURRENT_ITEM_Y] = 19;
        state.ram[CAPE_DECREMENT_COUNTER] = 1;
        set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 0);
        set_link_test_byte(&mut state, LINK_MAGIC_POWER, 1);

        state.player_check_handle_cape_stuff();

        assert_eq!(link_test_byte(&state, LINK_MAGIC_POWER), 0);
        assert_eq!(state.ram[CAPE_DECREMENT_COUNTER], 4);
        assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 0);
        assert_eq!(link_test_byte(&state, LINK_BUNNY_TRANSFORM_TIMER), 32);
        assert_eq!(state.system_signals_view().sound_effect_1() & 0x3f, 21);

        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_CAPE_MODE, 1);
        state.ram[CURRENT_ITEM_ACTIVE] = 19;
        state.ram[CURRENT_ITEM_Y] = 19;
        state.ram[CAPE_DECREMENT_COUNTER] = 1;
        set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 2);
        set_link_test_byte(&mut state, LINK_MAGIC_POWER, 2);
        state.player_state_view_mut().set_grabbing_wall(1);
        write_le_u16(&mut state.ram, ENHANCED_FEATURES0, 0x1000);

        state.link_handle_cape_passive_lift_check();

        assert_eq!(link_test_byte(&state, LINK_MAGIC_POWER), 1);
        assert_eq!(state.ram[CAPE_DECREMENT_COUNTER], 8);
        assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 1);
    }

    #[test]
    fn y_button_magic_and_item_reset_helpers_match_c_gates() {
        let mut state = ZeldaState::new();
        assert!(!state.check_y_button_press());
        state.player_state_view_mut().set_filtered_joypad_h(0x40);
        assert!(state.check_y_button_press());
        assert_eq!(state.player_state_view().button_mask_b_y() & 0x40, 0x40);
        assert!(!state.check_y_button_press());

        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_MAGIC_POWER, 20);
        set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 1);
        assert!(state.link_check_magic_cost(0));
        assert_eq!(link_test_byte(&state, LINK_MAGIC_POWER), 12);

        set_link_test_byte(&mut state, LINK_MAGIC_POWER, 1);
        set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 0);
        assert!(!state.link_check_magic_cost(1));
        assert_eq!(state.system_signals_view().sound_effect_1() & 0x3f, 60);
        assert_eq!(state.dialogue_message_index_view().value(), 123);
        assert_eq!(state.ram[MAIN_MODULE_INDEX], 14);

        set_link_test_byte(&mut state, LINK_MAGIC_POWER, 125);
        set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 0);
        write_le_u16(&mut state.ram, ENHANCED_FEATURES0, 0x1000);
        state.refund_magic(0);
        assert_eq!(link_test_byte(&state, LINK_MAGIC_POWER), 128);

        state.player_state_view_mut().set_y_button_action_step(1);
        state.player_state_view_mut().set_y_button_action_flags(2);
        state.player_state_view_mut().set_state_bits(3);
        state.player_state_view_mut().set_picking_throw_state(4);
        state.player_state_view_mut().set_grabbing_wall(5);
        set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 0xff);
        state.link_item_reset_from_overworld_things();
        assert_eq!(state.player_state_view().y_button_action_step(), 0);
        assert_eq!(state.player_state_view().y_button_action_flags(), 0);
        assert_eq!(state.player_state_view().state_bits(), 0);
        assert_eq!(state.player_state_view().picking_throw_state(), 0);
        assert_eq!(state.player_state_view().grabbing_wall(), 0);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
    }

    #[test]
    fn cape_item_activation_and_no_magic_prompt_match_c_state() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut state, LINK_MAGIC_POWER, 10);
        set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 1);

        state.link_item_cape();

        assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 1);
        assert_eq!(state.ram[CAPE_DECREMENT_COUNTER], 8);
        assert_eq!(link_test_byte(&state, LINK_BUNNY_TRANSFORM_TIMER), 20);
        assert_eq!(state.system_signals_view().sound_effect_1() & 0x3f, 20);
        assert_eq!(state.player_state_view().button_mask_b_y() & 0x40, 0);

        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_filtered_joypad_h(0x40);
        state.link_item_cape();

        assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 0);
        assert_eq!(state.system_signals_view().sound_effect_1() & 0x3f, 60);
        assert_eq!(state.dialogue_message_index_view().value(), 123);
        assert_eq!(state.ram[MAIN_MODULE_INDEX], 14);
    }

    #[test]
    fn cape_item_timer_gate_drain_and_manual_unequip_match_c_state() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_BUNNY_TRANSFORM_TIMER, 2);
        set_link_test_byte(&mut state, LINK_DIRECTION, 0x0f);
        state.ram[DUNG_HDR_COLLISION_2] = 2;
        state.ram[PLAYER_LAYER_COLLISION_FLAGS] =
            crate::game_state::constants::player::LAYER_COLLISION_BOTH;
        set_link_test_byte(&mut state, LINK_Y_VEL, 7);

        state.link_item_cape();

        assert_eq!(link_test_byte(&state, LINK_BUNNY_TRANSFORM_TIMER), 1);
        assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0);
        assert_eq!(link_test_byte(&state, LINK_Y_VEL), 0);

        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_CAPE_MODE, 1);
        state.ram[CAPE_DECREMENT_COUNTER] = 1;
        set_link_test_byte(&mut state, LINK_MAGIC_POWER, 2);
        set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 0);
        set_link_test_byte(&mut state, LINK_BUNNY_TRANSFORM_TIMER, 5);
        set_link_test_byte(&mut state, LINK_DIRECTION, 0x0f);

        state.link_item_cape();

        assert_eq!(link_test_byte(&state, LINK_MAGIC_POWER), 1);
        assert_eq!(state.ram[CAPE_DECREMENT_COUNTER], 4);
        assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 1);
        assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 1);
        assert_eq!(link_test_byte(&state, LINK_DIRECTION), 0);

        state.ram[CAPE_DECREMENT_COUNTER] = 2;
        set_link_test_byte(&mut state, LINK_BUNNY_TRANSFORM_TIMER, 0);
        state.player_state_view_mut().set_filtered_joypad_h(0x40);
        state.link_item_cape();

        assert_eq!(link_test_byte(&state, LINK_CAPE_MODE), 0);
        assert_eq!(link_test_byte(&state, LINK_BUNNY_TRANSFORM_TIMER), 32);
        assert_eq!(state.system_signals_view().sound_effect_1() & 0x3f, 21);
    }

    #[test]
    fn rod_hammer_and_bow_item_handlers_advance_c_timers() {
        let mut rod = ZeldaState::new();
        rod.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut rod, LINK_MAGIC_POWER, 20);
        rod.ram[EQ_SELECTED_ROD] = 1;
        rod.link_item_rod();
        assert_eq!(link_test_byte(&rod, LINK_MAGIC_POWER), 4);
        assert!(rod.player_state_view().item_in_hand_has(1));
        assert_eq!(link_test_byte(&rod, LINK_DEBUG_VALUE_2), 1);
        assert_eq!(link_test_byte(&rod, LINK_DELAY_TIMER_SPIN_ATTACK), 2);
        assert_eq!(rod.ancilla_slot_view(4).ancilla_type(), 2);

        let mut hammer = ZeldaState::new();
        hammer.player_state_view_mut().set_filtered_joypad_h(0x40);
        hammer.link_item_hammer();
        assert_eq!(hammer.player_state_view().item_in_hand(), 2);
        assert_eq!(link_test_byte(&hammer, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
        assert_eq!(link_test_byte(&hammer, LINK_DELAY_TIMER_SPIN_ATTACK), 2);

        let mut bow = ZeldaState::new();
        bow.player_state_view_mut().set_button_mask_b_y(0x40);
        bow.player_state_view_mut().set_item_in_hand(0x10);
        set_link_test_byte(&mut bow, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
        bow.ram[PLAYER_HANDLER_TIMER] = 2;
        set_link_test_byte(&mut bow, LINK_CANT_CHANGE_DIRECTION, 1);
        set_link_test_byte(&mut bow, LINK_NUM_ARROWS, 2);
        bow.player_state_view_mut().set_button_b_frames(12);
        bow.link_item_bow();
        assert_eq!(link_test_byte(&bow, LINK_NUM_ARROWS), 1);
        assert!(!bow.player_state_view().item_in_hand_has(0x10));
        assert_eq!(bow.player_state_view().button_mask_b_y() & 0x40, 0);
        assert_eq!(bow.player_state_view().button_b_frames(), 9);
        assert_eq!(bow.ancilla_slot_view(4).ancilla_type(), 9);
    }

    #[test]
    fn boomerang_bombs_book_and_desert_prayer_match_c_state() {
        let mut boom = ZeldaState::new();
        boom.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut boom, LINK_ITEM_BOOMERANG, 1);
        boom.link_item_boomerang();
        assert_eq!(boom.player_state_view().item_in_hand(), 0x80);
        assert_eq!(boom.ram[FLAG_FOR_BOOMERANG_IN_PLACE], 1);
        assert_eq!(link_test_byte(&boom, LINK_DELAY_TIMER_SPIN_ATTACK), 6);
        assert_eq!(link_test_byte(&boom, LINK_CANT_CHANGE_DIRECTION) & 1, 1);

        set_link_test_byte(&mut boom, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
        boom.ram[PLAYER_HANDLER_TIMER] = 1;
        boom.link_item_boomerang();
        assert_eq!(boom.player_state_view().item_in_hand(), 0);
        assert_eq!(boom.ram[PLAYER_HANDLER_TIMER], 0);
        assert_eq!(boom.player_state_view().button_mask_b_y() & 0x40, 0);
        assert_eq!(link_test_byte(&boom, LINK_CANT_CHANGE_DIRECTION) & 1, 0);

        let mut bombs = ZeldaState::new();
        bombs.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut bombs, LINK_ITEM_BOMBS, 1);
        bombs.link_item_bombs();
        // C `AncillaAdd_Bomb(7, 1)` allocates via `Ancilla_AllocInit(7, 1)`, which
        // for ancilla types 7/8 walks slots [limit..0], so slot 1 receives the
        // bomb ancilla. See zelda3/src/ancilla.c:5763 and ancilla.c:6990.
        assert_eq!(bombs.ancilla_slot_view(1).ancilla_type(), 7);
        assert_eq!(bombs.player_state_view().button_mask_b_y() & 0x40, 0);
        assert_eq!(link_test_byte(&bombs, LINK_ITEM_BOMBS), 0);
        assert_eq!(bombs.player_state_view().item_in_hand(), 0);

        let mut book = ZeldaState::new();
        book.player_state_view_mut().set_filtered_joypad_h(0x40);
        book.link_item_book();
        assert_eq!(book.system_signals_view().sound_effect_1() & 0x3f, 60);

        let mut prayer = ZeldaState::new();
        prayer.player_state_view_mut().set_filtered_joypad_h(0x40);
        prayer.ram[ITEM_PICKUP_IN_PROGRESS_FLAG] = 1;
        prayer.set_main_module(9);
        set_link_test_byte(&mut prayer, LINK_DIRECTION, 0x0f);
        prayer.link_item_book();
        assert_eq!(prayer.ram[SUBMODULE_INDEX], 5);
        assert_eq!(prayer.ram[SAVED_MODULE_FOR_MENU], 9);
        assert_eq!(prayer.ram[MAIN_MODULE_INDEX], 14);
        assert_eq!(prayer.frame_state().modal_pause_flag, 1);
        assert_eq!(prayer.player_state_view().y_button_action_timer(), 22);
        assert_eq!(prayer.player_state_view().state_bits(), 2);
        assert_eq!(link_test_byte(&prayer, LINK_DIRECTION), 0);
        assert_eq!(prayer.system_signals_view().ambient_sound_effect(), 17);
        assert_eq!(prayer.system_signals_view().music_control(), 242);
    }

    #[test]
    fn lamp_powder_and_shovel_item_handlers_match_core_state() {
        let mut lamp = ZeldaState::new();
        lamp.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut lamp, LINK_ITEM_TORCH, 1);
        set_link_test_byte(&mut lamp, LINK_MAGIC_POWER, 32);
        set_link_test_byte(&mut lamp, LINK_CANT_CHANGE_DIRECTION, 1);
        lamp.player_state_view_mut().set_button_b_frames(9);
        lamp.link_item_lamp();
        assert_eq!(link_test_byte(&lamp, LINK_MAGIC_POWER), 28);
        assert_eq!(lamp.player_state_view().button_mask_b_y(), 0);
        assert_eq!(lamp.player_state_view().button_b_frames(), 0);
        assert_eq!(link_test_byte(&lamp, LINK_CANT_CHANGE_DIRECTION), 0);
        assert_eq!(lamp.ancilla_slot_view(4).ancilla_type(), 0x1a);
        assert_eq!(lamp.ancilla_slot_view(3).ancilla_type(), 0x2f);

        let mut powder = ZeldaState::new();
        powder.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut powder, LINK_ITEM_MUSHROOM, 2);
        set_link_test_byte(&mut powder, LINK_MAGIC_POWER, 16);
        powder.link_item_powder();
        assert_eq!(link_test_byte(&powder, LINK_MAGIC_POWER), 8);
        assert_eq!(powder.player_state_view().item_in_hand(), 0x40);
        assert_eq!(link_test_byte(&powder, LINK_DELAY_TIMER_SPIN_ATTACK), 1);
        assert_eq!(link_test_byte(&powder, LINK_DIRECTION), 0);

        let mut shovel = ZeldaState::new();
        shovel.player_state_view_mut().set_filtered_joypad_h(0x40);
        shovel.link_item_shovel();
        assert_eq!(shovel.player_state_view().position_mode(), 1);
        assert_eq!(link_test_byte(&shovel, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
        assert_eq!(link_test_byte(&shovel, LINK_DELAY_TIMER_SPIN_ATTACK), 6);

        set_link_test_byte(&mut shovel, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
        shovel.player_state_view_mut().set_item_action_step_var(2);
        shovel.link_item_shovel();
        assert_eq!(shovel.player_state_view().item_action_step_var(), 0);
        assert_eq!(shovel.ram[PLAYER_HANDLER_TIMER], 0);
        assert_eq!(shovel.player_state_view().button_mask_b_y() & 0x40, 0);
        assert_eq!(shovel.player_state_view().position_mode(), 0);
        assert_eq!(link_test_byte(&shovel, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
    }

    #[test]
    fn flute_item_countdown_and_weather_vane_branch_match_c_state() {
        let mut countdown = ZeldaState::new();
        countdown.player_state_view_mut().set_button_mask_b_y(0x40);
        countdown.ram[FLUTE_COUNTDOWN] = 2;
        countdown.link_item_flute();
        assert_eq!(countdown.ram[FLUTE_COUNTDOWN], 1);
        assert_eq!(countdown.player_state_view().button_mask_b_y() & 0x40, 0x40);

        let mut flute = ZeldaState::new();
        flute.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut flute, LINK_ITEM_FLUTE, 2);
        flute.set_overworld_screen_word(0x18);
        set_link_test_word(&mut flute, LINK_Y_COORD, 0x780);
        set_link_test_word(&mut flute, LINK_X_COORD, 0x200);
        flute.link_item_flute();
        assert_eq!(flute.ram[FLUTE_COUNTDOWN], 128);
        assert_eq!(flute.system_signals_view().sound_effect_1(), 0);
        assert_eq!(flute.ram[SUBMODULE_INDEX], 45);
        assert_eq!(flute.ancilla_slot_view(4).ancilla_type(), 55);

        let mut shovel_dispatch = ZeldaState::new();
        shovel_dispatch
            .player_state_view_mut()
            .set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut shovel_dispatch, LINK_ITEM_FLUTE, 1);
        shovel_dispatch.link_item_shovel_and_flute();
        assert_eq!(shovel_dispatch.player_state_view().position_mode(), 1);
    }

    #[test]
    fn medallion_item_start_and_state_progression_match_core_state() {
        let mut ether = ZeldaState::new();
        ether.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut ether, LINK_SWORD_TYPE, 1);
        set_link_test_byte(&mut ether, LINK_MAGIC_POWER, 64);
        ether.link_item_ether();
        assert_eq!(link_test_byte(&ether, LINK_MAGIC_POWER), 32);
        assert_eq!(ether.player_state_view().handler_state(), 8);
        assert_eq!(link_test_byte(&ether, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
        assert_eq!(link_test_byte(&ether, LINK_DELAY_TIMER_SPIN_ATTACK), 5);
        assert_eq!(ether.ram[STEP_COUNTER_FOR_SPIN_ATTACK], 0);
        assert_eq!(ether.system_signals_view().sound_effect_2() & 0x3f, 35);

        set_link_test_byte(&mut ether, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
        ether.ram[STEP_COUNTER_FOR_SPIN_ATTACK] = 9;
        ether.link_state_using_ether();
        assert_eq!(ether.ram[STEP_COUNTER_FOR_SPIN_ATTACK], 10);
        assert_eq!(ether.ram[SPIN_ATTACK_SOUND_LATCH], 1);
        assert_eq!(ether.ancilla_slot_view(4).ancilla_type(), 24);

        let mut quake = ZeldaState::new();
        quake.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut quake, LINK_SWORD_TYPE, 1);
        set_link_test_byte(&mut quake, LINK_MAGIC_POWER, 64);
        quake.link_item_quake();
        assert_eq!(quake.player_state_view().handler_state(), 10);
        assert_eq!(link_test_byte(&quake, LINK_ACTUAL_VEL_Z_MIRROR), 40);
        assert_eq!(link_test_byte(&quake, LINK_ACTUAL_VEL_Z_COPY_MIRROR), 40);
        assert_eq!(link_test_byte(&quake, LINK_Z_COORD_MIRROR), 0);

        let mut blocked = ZeldaState::new();
        blocked.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut blocked, LINK_MAGIC_POWER, 64);
        blocked.link_item_bombos();
        assert_eq!(blocked.player_state_view().handler_state(), 0);
        assert_eq!(blocked.system_signals_view().sound_effect_1() & 0x3f, 60);
    }

    #[test]
    fn mirror_item_crossing_and_follower_cleanup_match_core_state() {
        let mut mirror = ZeldaState::new();
        mirror.player_state_view_mut().set_filtered_joypad_h(0x40);
        write_le_u16(&mut mirror.ram, ENHANCED_FEATURES0, 8);
        mirror
            .world_state_view_mut()
            .set_overworld_screen_word(0x40);
        set_link_test_word(&mut mirror, LINK_Y_COORD, 0x1234);
        set_link_test_word(&mut mirror, LINK_X_COORD, 0x5678);
        mirror.player_state_view_mut().set_actual_x_velocity(7);
        mirror.player_state_view_mut().set_actual_y_velocity(9);
        mirror.link_item_mirror();
        assert_eq!(mirror.ram[LAST_LIGHT_VS_DARK_WORLD], 0x40);
        assert_eq!(mirror.bird_travel_destination_view(15).y(), 0x1234);
        assert_eq!(mirror.bird_travel_destination_view(15).x(), 0x5678);
        assert_eq!(mirror.ram[SUBMODULE_INDEX], 35);
        assert_eq!(
            link_test_byte(&mirror, LINK_TRIGGERED_BY_WHIRLPOOL_SPRITE),
            1
        );
        assert_eq!(mirror.player_state_view().handler_state(), 20);
        assert_eq!(mirror.player_state_view().actual_x_velocity(), 0);
        assert_eq!(mirror.player_state_view().actual_y_velocity(), 0);

        let mut crossing = ZeldaState::new();
        crossing.ram[LAST_LIGHT_VS_DARK_WORLD] = 0;
        crossing
            .world_state_view_mut()
            .set_overworld_screen_word(0x40);
        let mut data = vec![0; 0x100];
        data[0x80] = 1;
        let mut ranges = vec![(0, 0); 164];
        ranges[70] = (0, 0x80);
        ranges[163] = (0x80, 0x100);
        crossing.assets = Some(AssetPack { data, ranges });
        crossing.link_state_crossing_worlds();
        assert_eq!(crossing.ram[SUBMODULE_INDEX], 44);
        assert_eq!(crossing.player_state_view().handler_state(), 20);

        let mut follower = ZeldaState::new();
        follower.ram[FOLLOWER_INDICATOR] = 13;
        follower.ram[FOLLOWER_DROPPED] = 1;
        set_link_test_byte(&mut follower, LINK_CAPE_MODE, 1);
        follower.handle_followers_after_mirroring();
        assert_eq!(follower.ram[SUPER_BOMB_INDICATOR_TIMER], 0xfe);
        assert_eq!(follower.ram[SUPER_BOMB_INDICATOR_COUNTER], 0);
        assert_eq!(follower.ram[FOLLOWER_INDICATOR], 0);
        assert_eq!(link_test_byte(&follower, LINK_CAPE_MODE), 0);
        assert_eq!(link_test_byte(&follower, LINK_BUNNY_TRANSFORM_TIMER), 0);
    }

    #[test]
    fn hookshot_item_and_timeout_state_match_core_state() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_filtered_joypad_h(0x40);
        state.player_state_view_mut().set_facing(4);
        set_link_test_word(&mut state, LINK_X_COORD, 0x0100);
        set_link_test_word(&mut state, LINK_Y_COORD, 0x0200);
        state
            .swim_acceleration_view_mut()
            .set_speed_active_flag(0, 1);

        state.link_item_hookshot();

        assert_eq!(state.player_state_view().handler_state(), 19);
        assert_eq!(state.player_state_view().position_mode(), 4);
        assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 1);
        assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 7);
        assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0x1f);
        assert_eq!(state.messaging_state_view().game_over_letter_cursor(), 4);
        assert_eq!(state.ram[ANCILLA_X_VEL + 4], 0xc0);
        assert_eq!(read_le_u16(&state.ram, ANCILLA_X_LO + 4), 0x00fc);

        state.ancilla_slot_view_mut(4).set_ancilla_type(0);
        set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
        state.player_state_view_mut().set_button_b_frames(12);
        state.link_state_hookshotting();
        assert_eq!(state.player_state_view().handler_state(), 0);
        assert!(!state.player_state_view().position_mode_has(4));
        assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
        assert_eq!(state.player_state_view().button_mask_b_y() & 0x40, 0);
        assert_eq!(state.player_state_view().button_b_frames(), 9);
    }

    #[test]
    fn cane_of_somaria_start_consumes_magic_and_enters_item_pose() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut state, LINK_MAGIC_POWER, 32);
        set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 0);

        state.link_item_cane_of_somaria();

        assert_eq!(link_test_byte(&state, LINK_MAGIC_POWER), 24);
        assert_eq!(state.player_state_view().button_mask_b_y() & 0x40, 0x40);
        assert!(state.player_state_view().position_mode_has(8));
        assert_eq!(link_test_byte(&state, LINK_DEBUG_VALUE_2), 1);
        assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 2);
        assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0x2c);
    }

    #[test]
    fn cane_of_byrna_start_and_finish_match_timer_state() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_filtered_joypad_h(0x40);
        set_link_test_byte(&mut state, LINK_MAGIC_POWER, 40);
        set_link_test_byte(&mut state, LINK_MAGIC_CONSUMPTION, 0);

        state.link_item_cane_of_byrna();

        assert_eq!(link_test_byte(&state, LINK_MAGIC_POWER), 24);
        assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0x30);
        assert_eq!(state.player_state_view().position_mode(), 8);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
        assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 18);

        state.ancilla_slot_view_mut(4).set_ancilla_type(0);
        state.player_state_view_mut().set_button_mask_b_y(0x40);
        set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
        state.ram[PLAYER_HANDLER_TIMER] = 2;
        state.link_item_cane_of_byrna();

        assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
        assert_eq!(state.player_state_view().button_mask_b_y() & 0x40, 0);
        assert_eq!(state.player_state_view().position_mode(), 0);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
    }

    #[test]
    fn bug_net_start_and_finish_match_c_timer_table() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_filtered_joypad_h(0x40);
        state.player_state_view_mut().set_facing(4);

        state.link_item_net();

        assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 9);
        assert_eq!(link_test_byte(&state, LINK_DELAY_TIMER_SPIN_ATTACK), 2);
        assert_eq!(state.player_state_view().position_mode(), 16);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 1);
        assert_eq!(state.system_signals_view().sound_effect_1() & 0x3f, 50);

        state.player_state_view_mut().set_button_mask_b_y(0x40);
        set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
        state.player_state_view_mut().set_item_action_step_var(9);
        state.link_item_net();

        assert_eq!(state.player_state_view().item_action_step_var(), 0);
        assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
        assert_eq!(state.player_state_view().button_mask_b_y() & 0x40, 0);
        assert_eq!(state.player_state_view().position_mode(), 0);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
        assert_eq!(state.player_state_view().oam_x_offset(), 0x80);
        assert_eq!(state.player_state_view().oam_y_offset(), 0x80);
    }

    #[test]
    fn bug_net_right_facing_finish_does_not_read_past_timer_table() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_button_mask_b_y(0x40);
        state.player_state_view_mut().set_facing(6);
        set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
        state.player_state_view_mut().set_item_action_step_var(9);
        state.ram[PLAYER_HANDLER_TIMER] = 8;
        state.player_state_view_mut().set_position_mode(16);
        set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 1);

        state.link_item_net();

        assert_eq!(state.player_state_view().item_action_step_var(), 0);
        assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
        assert_eq!(state.player_state_view().button_mask_b_y() & 0x40, 0);
        assert_eq!(state.player_state_view().position_mode(), 0);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION) & 1, 0);
        assert_eq!(state.player_state_view().oam_x_offset(), 0x80);
        assert_eq!(state.player_state_view().oam_y_offset(), 0x80);
    }

    #[test]
    fn link_zap_mosaic_bounces_between_zero_and_c0() {
        let mut state = ZeldaState::new();
        state.set_mosaic_level(0xb0);

        state.LinkZap_HandleMosaic();

        assert_eq!(state.display_state().mosaic_level, 0xc0);
        assert_eq!(state.display_state().mosaic_direction, 1);
        assert_eq!(state.display_state().mosaic_copy, 0x63);
        assert_eq!(state.display_state().bg_mode, 9);

        state.set_mosaic_level(0x10);
        state.LinkZap_HandleMosaic();
        assert_eq!(state.display_state().mosaic_level, 0);
        assert_eq!(state.display_state().mosaic_direction, 0);
        assert_eq!(state.display_state().mosaic_copy, 3);
    }

    #[test]
    fn zapped_state_advances_timer_and_finishes_on_eighth_pulse() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_handler_state(7);
        set_link_test_byte(&mut state, LINK_DELAY_TIMER_SPIN_ATTACK, 0);
        state.ram[PLAYER_HANDLER_TIMER] = 7;
        set_link_test_byte(&mut state, LINK_DISABLE_SPRITE_DAMAGE, 1);
        set_link_test_byte(&mut state, LINK_ELECTROCUTE_ON_TOUCH, 1);
        state.player_state_view_mut().set_auxiliary_state(1);
        state.set_mosaic_level(0x20);
        state.set_mosaic_direction(1);

        state.link_state_zapped();

        assert_eq!(state.ram[PLAYER_HANDLER_TIMER], 0);
        assert_eq!(state.player_state_view().handler_state(), 0);
        assert_eq!(link_test_byte(&state, LINK_DISABLE_SPRITE_DAMAGE), 0);
        assert_eq!(link_test_byte(&state, LINK_ELECTROCUTE_ON_TOUCH), 0);
        assert_eq!(state.player_state_view().auxiliary_state(), 0);
        assert_eq!(state.display_state().mosaic_level, 0);
        assert_eq!(state.display_state().mosaic_copy, 3);
        assert_eq!(state.display_state().bg_mode, 9);
    }

    #[test]
    fn load_actual_gear_palettes_applies_enhanced_glove_color() {
        let mut state = ZeldaState::new();
        write_le_u16(&mut state.ram, ENHANCED_FEATURES0, 0x1000);
        set_link_test_byte(&mut state, LINK_ITEM_GLOVES, 2);

        state.load_actual_gear_palettes();

        assert_eq!(
            read_le_u16(&state.ram, AUX_PALETTE_BUFFER + 0xfd * 2),
            0x0376
        );
        assert_eq!(
            read_le_u16(&state.ram, MAIN_PALETTE_BUFFER + 0xfd * 2),
            0x0376
        );
        assert_eq!(state.ram[FLAG_UPDATE_CGRAM_IN_NMI], 2);
    }

    #[test]
    fn cancel_dash_clears_running_state_and_dash_ancilla() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_running_state(1);
        set_link_test_byte(&mut state, LINK_COUNTDOWN_FOR_DASH, 12);
        state.player_state_view_mut().set_speed_setting(16);
        set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 1);
        state.swim_acceleration_view_mut().set_mode(0, 0x1234);
        state.ancilla_slot_view_mut(0).set_ancilla_type(0x1e);
        state.ancilla_slot_view_mut(4).set_ancilla_type(0x1e);

        state.link_cancel_dash();

        assert_eq!(state.ancilla_slot_view(0).ancilla_type(), 0);
        assert_eq!(state.ancilla_slot_view(4).ancilla_type(), 0);
        assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 0);
        assert_eq!(state.player_state_view().speed_setting(), 0);
        assert_eq!(state.player_state_view().running_state(), 0);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION), 0);
        assert_eq!(state.swim_acceleration_view().mode(0), 0);
    }

    #[test]
    fn exiting_dash_resets_or_counts_down_like_c_state() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_handler_state(18);
        set_link_test_byte(&mut state, LINK_COUNTDOWN_FOR_DASH, 3);

        state.link_state_exiting_dash();

        assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 4);
        assert_eq!(state.player_state_view().handler_state(), 18);

        state.player_state_view_mut().set_joypad1h_last(1);
        state.player_state_view_mut().set_running_state(1);
        state.player_state_view_mut().set_speed_setting(16);
        set_link_test_byte(&mut state, LINK_CANT_CHANGE_DIRECTION, 1);
        state.player_state_view_mut().set_button_b_frames(8);
        state.swim_acceleration_view_mut().set_mode(0, 0x1234);

        state.link_state_exiting_dash();

        assert_eq!(link_test_byte(&state, LINK_COUNTDOWN_FOR_DASH), 0);
        assert_eq!(state.player_state_view().speed_setting(), 0);
        assert_eq!(state.player_state_view().handler_state(), 0);
        assert_eq!(state.player_state_view().running_state(), 0);
        assert_eq!(state.swim_acceleration_view().mode(0), 0);
        assert_eq!(link_test_byte(&state, LINK_CANT_CHANGE_DIRECTION), 0);
    }

    #[test]
    fn item_tile_behavior_routes_overworld_attr_to_tile_execute() {
        let mut state = ZeldaState::new();
        write_le_u16(&mut state.ram, TILEMAP_LOCATION_CALC_MASK, 0x01ff);
        write_le_u16(&mut state.ram, OVERWORLD_OFFSET_MASK_Y, 0x1f);
        write_le_u16(&mut state.ram, OVERWORLD_OFFSET_MASK_X, 0x3f);
        state.dungeon_state_view_mut().set_bg2_tile(16, 7);

        let mut data = vec![0; 0x100];
        write_le_u16(&mut data, 7 * 4 * 2, 3);
        data[0x80 + 3] = 1;
        let mut ranges = vec![(0, 0); 164];
        ranges[70] = (0, 0x80);
        ranges[163] = (0x80, 0x100);
        state.assets = Some(AssetPack { data, ranges });

        state.tile_detect_main_handler(1);

        assert_eq!(state.tile_detect_position_view().collision_bits(), 0);
        assert_eq!(read_le_u16(&state.ram, TILEDETECT_NORMAL_TILES), 1);
    }

    #[test]
    fn tile_main_handler_shallow_water_sets_ripple_and_slosh_sound() {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(1);
        set_link_test_byte(&mut state, LINK_DIRECTION, 1);
        write_le_u16(&mut state.ram, TILEMAP_LOCATION_CALC_MASK, 0x01ff);
        state
            .dungeon_state_view_mut()
            .set_bg2_attr(16 * 8 + 1, 0x09);

        state.tile_detect_main_handler(0);

        assert_eq!(state.player_state_view().water_ripple_or_grass_state(), 1);
        assert_eq!(state.system_signals_view().sound_effect_1() & 0x3f, 28);
        assert_eq!(state.ram[RAW_SFX_PAN_VALUE], 28);
    }

    #[test]
    fn tile_main_handler_spike_trigger_applies_damage_and_bunny_reset() {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(1);
        set_link_test_byte(&mut state, LINK_IS_BUNNY, 1);
        set_link_test_byte(&mut state, LINK_IS_BUNNY_MIRROR, 1);
        set_link_test_byte(&mut state, LINK_ITEM_MOON_PEARL, 1);
        set_link_test_word(&mut state, LINK_TIMER_TEMPBUNNY, 0x1234);
        set_link_test_byte(&mut state, LINK_NEED_FOR_POOF_FOR_TRANSFORM, 1);
        write_le_u16(&mut state.ram, TILEMAP_LOCATION_CALC_MASK, 0x01ff);
        state
            .dungeon_state_view_mut()
            .set_bg2_attr(16 * 8 + 1, 0x0d);

        state.tile_detect_main_handler(0);

        assert_eq!(link_test_byte(&state, LINK_GIVE_DAMAGE), 8);
        assert_eq!(link_test_byte(&state, LINK_IS_BUNNY), 0);
        assert_eq!(link_test_byte(&state, LINK_IS_BUNNY_MIRROR), 0);
        assert_eq!(link_test_byte(&state, LINK_NEED_FOR_POOF_FOR_TRANSFORM), 0);
        assert_eq!(link_test_word(&state, LINK_TIMER_TEMPBUNNY), 0);
    }

    #[test]
    fn tile_main_handler_icy_floor_starts_sliding_state() {
        let mut state = ZeldaState::new();
        state.set_indoor_flag(1);
        set_link_test_byte(&mut state, LINK_DIRECTION, 4);
        set_link_test_byte(&mut state, LINK_DIRECTION_LAST, 8);
        write_le_u16(&mut state.ram, TILEMAP_LOCATION_CALC_MASK, 0x01ff);
        state
            .dungeon_state_view_mut()
            .set_bg2_attr(16 * 8 + 1, 0x0e);

        state.tile_detect_main_handler(0);

        assert_eq!(link_test_byte(&state, LINK_FLAG_MOVING), 1);
        assert_eq!(state.player_state_view().swim_direction_flags(), 8);
        assert_eq!(state.player_state_view().water_ripple_or_grass_state(), 0);
    }

    #[test]
    fn push_block_target_flag_reads_dungeon_attr_table() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_lower_level_state(1);
        state
            .dungeon_state_view_mut()
            .set_bg2_attr(0x1000 + 0x145, 0x72);

        assert_eq!(state.push_block_get_target_tile_flag(5, 0x28), 0x72);
    }

    #[test]
    fn push_block_attempt_checks_both_target_tiles() {
        let mut state = ZeldaState::new();
        write_le_u16(&mut state.ram, TILEMAP_LOCATION_CALC_MASK, 0x01ff);
        set_link_test_byte(&mut state, LINK_LAST_DIRECTION_MOVED_TOWARDS, 0);
        state.dungeon_state_view_mut().set_bg2_attr(0x18 * 8 + 4, 0);
        state
            .dungeon_state_view_mut()
            .set_bg2_attr(0x18 * 8 + 5, 11);

        assert!(state.push_block_attempt_to_push_the_block(0, 0x20, 0x20));

        state.dungeon_state_view_mut().set_bg2_attr(0x18 * 8 + 5, 9);
        assert!(!state.push_block_attempt_to_push_the_block(0, 0x20, 0x20));
    }

    #[test]
    fn bottled_item_receipt_fills_first_open_bottle() {
        let mut state = ZeldaState::new();
        set_link_test_byte(&mut state, LINK_ITEM_BOTTLE_INFO, 2);
        state.ram[LINK_ITEM_BOTTLE_INFO + 1] = 4;

        state.item_receipt_give_bottled_item(0x2f);

        assert_eq!(link_test_byte(&state, LINK_ITEM_BOTTLE_INFO), 4);
        assert_eq!(state.ram[LINK_ITEM_BOTTLE_INFO + 1], 4);
    }

    #[test]
    fn first_frame_runs_startup_writes() {
        let mut state = ZeldaState::new();
        state.sram[0x03e5] = 0xaa;
        state.sram[0x03e6] = 0x55;
        state.sram[0x08e5] = 0x12;
        state.ram[MAIN_PALETTE_BUFFER] = 0xff;

        state.run_frame_internal(0, crate::RUN_MAIN);

        assert_eq!(read_le_u16(&state.ram, ANIMATED_TILE_DATA_SRC), 0xa680);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_9), 0xb280);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_14), 0xb2e0);
        assert_eq!(state.display_state().screen_brightness, 15);
        assert_eq!(state.ram[FLAG_UPDATE_CGRAM_IN_NMI], 0);
        assert_eq!(read_le_u16(&state.sram, 0x03e5), 0x55aa);
        assert_eq!(read_le_u16(&state.sram, 0x08e5), 0);
        assert_eq!(state.selected_save_slot_x2(), 0);
        assert_eq!(read_le_u16(&state.ram, MAIN_PALETTE_BUFFER), 0);
    }

    #[test]
    fn game_loop_clears_oam_y_slots_and_keeps_nmi_update_latched() {
        let mut state = ZeldaState::new();
        state.latch_nmi_update();

        state.run_frame_internal(0, crate::RUN_MAIN);

        assert_eq!(state.frame_state().frame_counter, 1);
        assert!(state.display_state().nmi_update_is_latched());
        for i in 4..128 {
            assert_eq!(state.ram[OAM_BUF + i * 4 + 1], 0xf0);
        }
    }

    #[test]
    fn ppu_write_helpers_route_to_ppu_registers() {
        let mut state = ZeldaState::new();

        state.zelda_ppu_write(0x2100, 0x8f);
        assert!(state.ppu.forced_blank);
        assert_eq!(state.ppu.brightness, 0x0f);

        state.zelda_ppu_write_word(0x2116, 0x1234);
        assert_eq!(state.ppu.vram_pointer, 0x1234);
    }

    #[test]
    fn hdma_setup_and_simple_hdma_line_write_ppu() {
        let mut state = ZeldaState::new();
        state.hdma_setup(0x0cfa87, 0x0cfa94, 0, 0, 0, 0);

        assert_eq!(state.dma.channel[6].a_adr, 0xfa87);
        assert_eq!(state.dma.channel[6].a_bank, 0x0c);
        assert_eq!(state.dma.channel[6].b_adr, 0);
        assert_eq!(state.dma.channel[7].a_adr, 0xfa94);

        state.dma.channel[6].hdma_active = true;
        let mut hdma = SimpleHdma::default();
        state.simple_hdma_init(&mut hdma, &state.dma.channel[6]);
        state.simple_hdma_do_line(&mut hdma);

        assert!(state.ppu.forced_blank);
        assert_eq!(state.ppu.brightness, 0x0f);
        assert_eq!(hdma.rep_count, 0x1f);
    }

    #[test]
    fn simple_hdma_get_ptr_maps_mode7_zoom_tables() {
        let state = ZeldaState::new();

        assert_eq!(
            state.simple_hdma_get_ptr(0x0add27).unwrap()[0..4],
            [0x77, 0x01, 0x76, 0x01]
        );
        assert_eq!(
            state.simple_hdma_get_ptr(0x0ade07).unwrap()[0..4],
            [0x35, 0x01, 0x35, 0x01]
        );
        assert_eq!(
            state.simple_hdma_get_ptr(0x0adee7).unwrap()[0..4],
            [0x88, 0x00, 0x88, 0x00]
        );
        assert_eq!(
            state.simple_hdma_get_ptr(0x0adfc7).unwrap()[0..4],
            [0x70, 0x00, 0x70, 0x00]
        );
    }

    #[test]
    fn draw_ppu_frame_applies_mode7_perspective_correction() {
        let mut state = ZeldaState::new();
        let mut pixels = vec![0u8; 256 * 224 * 4];
        state.ppu.mode = 7;
        state.set_hdma_enable_mask(1 << 6);
        state.hdma_setup(0x0abdcf, 0, 0, 0, 0, 0x0a);

        state.zelda_draw_ppu_frame(&mut pixels, 256 * 4, PpuRenderFlags::MODE7_4X4);

        assert_eq!(state.ppu.mode7_perspective_low, 1.0 / 375.0);
        assert_eq!(state.ppu.mode7_perspective_high, 1.0 / 264.0);
    }

    #[test]
    fn configure_ppu_side_space_matches_module_cases() {
        let mut state = ZeldaState::new();
        state.set_main_module(20);
        state.configure_ppu_side_space();
        assert_eq!(state.ppu.extra_left_cur, PPU_SIDE_SPACE_LIMIT as u8);
        assert_eq!(state.ppu.extra_right_cur, PPU_SIDE_SPACE_LIMIT as u8);
        assert_eq!(state.ppu.extra_bottom_cur, 16);

        state.set_main_module(7);
        state.world_state_view_mut().set_bg2_x(0x0110);
        state.world_state_view_mut().set_bg2_y(0x0108);
        state.room_bounds_view_mut().set_x_bound(0, 0x0100);
        state.room_bounds_view_mut().set_x_bound(2, 0x0140);
        state.room_bounds_view_mut().set_y_bound(2, 0x0120);
        state.ram[QUADRANT_FULLSIZE_X] = 0;
        state.ram[QUADRANT_FULLSIZE_Y] = 0;
        state.configure_ppu_side_space();
        assert_eq!(state.ppu.extra_left_cur, 0x10);
        assert_eq!(state.ppu.extra_right_cur, 0x30);
        assert_eq!(state.ppu.extra_bottom_cur, 16);
    }

    #[test]
    fn draw_ppu_frame_runs_irq_and_hdma_side_effects() {
        let mut state = ZeldaState::new();
        let mut pixels = vec![0u8; 256 * 224 * 4];
        state.set_irq_control_flag(0x80);
        state
            .select_file_scratch_view_mut()
            .set_name_scroll_x(0x01f0);
        state.set_hdma_enable_mask(1 << 6);
        state.hdma_setup(0x0cfa87, 0, 0, 0, 0, 0);

        state.zelda_draw_ppu_frame(&mut pixels, 256 * 4, PpuRenderFlags::empty());

        assert_eq!(state.display_state().irq_control_flag, 0);
        assert!(state.ppu.forced_blank);
        assert_eq!(state.ppu.brightness, 0x0f);
        assert_eq!(state.ppu.render_pitch, (PPU_X_PIXELS * 4) as u32);
        assert_eq!(
            state.ppu.render_buffer.as_ref().unwrap().len(),
            PPU_X_PIXELS * (224 + 1) * 4
        );
        assert_eq!(pixels.len(), 256 * 224 * 4);
    }

    #[test]
    fn display_snapshot_draw_uses_c_style_current_vram_not_obj_latch() {
        let mut state = ZeldaState::new();
        let mut pixels = vec![0u8; 256 * 224 * 4];

        state.ppu.obj_vram_latch = Some(vec![0x1111; VRAM_WORDS]);
        state.capture_display_snapshot();
        state.zelda_draw_display_frame(&mut pixels, 256 * 4, PpuRenderFlags::empty());
        assert!(state.ppu.obj_vram_latch.is_none());

        state.ppu.obj_vram_latch = Some(vec![0x1111; VRAM_WORDS]);
        state.obj_vram_latch_generation = 1;
        state.capture_display_snapshot();
        state.obj_vram_latch_generation = 2;
        state.ppu.obj_vram_latch = Some(vec![0x2222; VRAM_WORDS]);
        state.zelda_draw_display_frame(&mut pixels, 256 * 4, PpuRenderFlags::empty());
        assert!(state.ppu.obj_vram_latch.is_none());
    }

    #[test]
    fn first_intro_step_matches_top_level_state_writes() {
        let mut state = ZeldaState::new();

        state.run_frame_internal(0, crate::RUN_MAIN);

        assert_eq!(state.ram[SUBMODULE_INDEX], 1);
        assert_eq!(state.ram[SUBSUBMODULE_INDEX], 1);
        assert_eq!(state.display_state().screen_brightness, 15);
        assert_eq!(state.display_state().main_screen_layers, 16);
        assert_eq!(state.display_state().bg_mode, 9);
        assert_eq!(state.palette_filter_view().color_window_selection(), 0x20);
        assert_eq!(state.palette_filter_view().color_math_control(), 0x20);
        assert_eq!(state.palette_filter_view().fixed_color_red(), 0x20);
        assert_eq!(state.palette_filter_view().fixed_color_green(), 0x40);
        assert_eq!(state.palette_filter_view().fixed_color_blue(), 0x80);
        assert_eq!(state.display_state().core_update_disable_flag, 0x80);
        assert_eq!(state.display_state().nmi_load_target_page(), 0x46);
        assert_eq!(state.display_state().pending_nmi_subroutine, 0);
        assert_eq!(state.system_signals_view().sound_effect_2(), 0);
        assert_eq!(state.ending_scratch_view().primary_word(), 0x1bfe);
        assert_eq!(state.ending_scratch_view().secondary_word(), 0x17fe);
        assert_eq!(
            &state.ram[OAM_BUF..OAM_BUF + 16],
            &[
                0x60, 0x68, 0x69, 0x32, 0x70, 0x68, 0x6b, 0x32, 0x80, 0x68, 0x6d, 0x32, 0x88, 0x68,
                0x6e, 0x32
            ]
        );
        assert_eq!(
            &state.ram[BYTEWISE_EXTENDED_OAM..BYTEWISE_EXTENDED_OAM + 4],
            &[2; 4]
        );
        assert_eq!(state.ram[EXTENDED_OAM], 0xaa);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_3), 0x8080);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_0), 0x8280);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_4), 0x8840);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_1), 0x8a40);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_5), 0x9a40);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_2), 0x9a40);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_6), 0x9000);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_11), 0x9180);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_7), 0x9300);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_12), 0x93c0);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_8), 0x9480);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_13), 0x9560);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_10), 0xa480);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_15), 0xa580);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_16), 0xb940);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_18), 0xbb40);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_17), 0xb940);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_19), 0xbb40);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_20), 0xb540);
        assert_eq!(read_le_u16(&state.ram, DMA_SOURCE_ADDR_21), 0xb740);
        assert_eq!(read_le_u16(&state.ram, BG_TILE_ANIMATION_COUNTDOWN), 0xffff);
        assert_eq!(link_test_word(&state, LINK_DMA_COUNTDOWN), 0xffff);
        assert_eq!(state.ppu.cgram[144], 0x7fff);
    }

    #[test]
    fn graphics_half_slot_transforms_uncompressed_sprite_pack() {
        let mut state = ZeldaState::new();
        let mut pack = vec![0; 0x300 + 24 * 32];
        for i in 0..24 * 32 {
            pack[0x300 + i] = i as u8;
        }
        let mut data = vec![0; 8 * 2];
        data.extend_from_slice(&pack);
        data.extend_from_slice(&8u16.to_le_bytes());
        let mut ranges = vec![(0, 0); 65];
        ranges[64] = (0, data.len());
        state.assets = Some(AssetPack { data, ranges });

        state.set_chr_halfslot_request(20);
        state.graphics_load_chr_half_slot();

        assert_eq!(state.display_state().nmi_load_target_page(), 0x46);
        assert_eq!(state.display_state().pending_nmi_subroutine, 11);
        assert_eq!(&state.ram[0x11000..0x11004], &[0, 1, 2, 3]);
        assert_eq!(&state.ram[0x11010..0x11014], &[16, 17, 17, 19]);
        assert_eq!(&state.ram[0x11020..0x11024], &[24, 25, 26, 27]);
    }

    #[test]
    fn nmi_subroutine_11_uploads_bg_char_half_to_vram() {
        let mut state = ZeldaState::new();
        write_le_u16(&mut state.ram, 0x11000, 0x1234);
        write_le_u16(&mut state.ram, 0x11002, 0xabcd);
        state.set_nmi_load_target_page(0x46);
        state.set_pending_nmi_subroutine(11);

        state.nmi_do_updates();

        assert_eq!(state.ppu.vram[0x4600], 0x1234);
        assert_eq!(state.ppu.vram[0x4601], 0xabcd);
        assert_eq!(state.display_state().pending_nmi_subroutine, 0);
    }

    #[test]
    fn intro_submodule_one_continues_memory_clear_and_logo_oam() {
        let mut state = ZeldaState::new();
        state.run_frame_internal(0, crate::RUN_MAIN);
        state.run_frame_internal(0, crate::RUN_MAIN);

        assert_eq!(state.ram[SUBMODULE_INDEX], 1);
        assert_eq!(state.ram[SUBSUBMODULE_INDEX], 2);
        assert_eq!(
            &state.ram[OAM_BUF..OAM_BUF + 16],
            &[
                0x60, 0x68, 0x69, 0x32, 0x70, 0x68, 0x6b, 0x32, 0x80, 0x68, 0x6d, 0x32, 0x88, 0x68,
                0x6e, 0x32
            ]
        );
        assert_eq!(state.ending_scratch_view().primary_word(), 0x17fe);
        assert_eq!(state.ending_scratch_view().secondary_word(), 0x13fe);
    }

    #[test]
    fn intro_fade_in_bg_start_skips_to_file_select_loader() {
        let mut state = ZeldaState::new();
        state.set_main_module(0);
        state.set_submodule(7);
        state.set_subsubmodule(0xf3);
        state.palette_filter_view_mut().set_countdown(0);
        state.player_state_view_mut().set_filtered_joypad_h(0x10);
        state.set_indoor_flag(1);
        set_link_test_byte(&mut state, LINK_Y_COORD, 0x12);
        state.ram[LINK_Y_COORD + 0x6f] = 0x34;
        state.ram[SAVE_DUNG_INFO] = 0x56;

        state.module00_intro();

        assert_eq!(state.display_state().irq_control_flag, 0xff);
        assert_eq!(state.display_state().main_screen_layers, 0x15);
        assert_eq!(state.display_state().sub_screen_layers, 0);
        assert_eq!(state.world_location_state().indoor_flag, 0);
        assert_eq!(state.system_signals_view().music_control(), 0xf1);
        assert_eq!(state.ram[MAIN_MODULE_INDEX], 1);
        assert_eq!(state.ram[SUBMODULE_INDEX], 0);
        assert_eq!(state.ram[RESTART_CHECK_FLAG], 1);
        assert_eq!(link_test_byte(&state, LINK_Y_COORD), 0);
        assert_eq!(state.ram[LINK_Y_COORD + 0x6f], 0);
        assert_eq!(state.ram[SAVE_DUNG_INFO], 0);
    }

    #[test]
    fn name_file_x_scroll_both_horizontal_bits_match_c_rom_table() {
        let mut state = ZeldaState::new();
        state.player_state_view_mut().set_joypad1h_last(0x03);
        state.select_file_scratch_view_mut().set_name_column(21);

        state.name_file_check_for_scroll_input_x();

        let select_file = state.select_file_scratch_view();
        assert_eq!(select_file.name_column(), 53);
        assert_eq!(select_file.name_scroll_x_step(), 1);
        assert_eq!(select_file.name_scroll_x_direction(), 2);
    }
}
