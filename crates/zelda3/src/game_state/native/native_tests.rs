use super::display::{
    HudInventoryOrderState, HudRuntimeState, OverworldPaletteBackupState, PaletteBufferState,
    PaletteFilterState, TrinexxPaletteState, WaterHdmaWindowState,
};
use super::dungeon::{
    DungeonDoorState, DungeonEntranceBackupState, DungeonHeaderState, DungeonRoomDoorSetupState,
    DungeonScratchWordState, DungeonTorchState,
};
use super::effects;
use super::effects::{
    BlastWallState, BombosSpellState, DiggingGamePrizeState, DoorDebrisState,
    EffectAngleScratchState, EntranceEffectState, QuakeBoltState, QuakeSpellState,
    SkullWoodsFireState, TowerSealState,
};
use super::ending::{AttractSceneState, EndingCreditState, IntroSceneState};
use super::inventory::{
    DungeonKeySlotsState, InventoryItemsState, MirrorWarpState, PlayerResourcesState,
};
use super::messaging::{
    DecodedMessageTextState, DialogueMessageIndexState, DialogueNumberState,
    DialoguePointerTableState, DialogueSourceOffsetState, MessagingRuntimeState,
    MultiselectChoiceState, SharedMessageTimerState,
};
use super::player::{
    PushedBlockState, SpecialExitPositionState, SwimAccelerationState, TileDetectionState,
};
use super::poly::{
    PolyFaceCoordsState, PolyProjectedVerticesState, PolyRasterEdgeState, PolyRuntimeState,
};
use super::sprites::{
    ChainChompHistoryState, DualLayerTileCacheState, EnemyDamageSubclassTableState,
    EtherOrbitState, FailedSpinSparkleSpawnState, FollowerRuntimeState, GarnishRuntimeState,
    MazeGameTimerState, OverworldSpriteLoadedState, OverworldSpritePresenceState,
    PrizeDropCycleState, SpriteDrawHitboxWorkState, SpriteSlotsState, SpriteSystemState,
    TagalongTrailState,
};
use super::system::NativeSystemWorkAreaBridgeMut;
use super::world::{
    BirdTravelDestinationsState, OverworldConfigTableState, OverworldEntranceState,
    OverworldEventInfoState, OverworldExitState, OverworldMap16State, OverworldMapUiState,
    OverworldMapZoomState, OverworldScreenSizeState, OverworldScrollDeltaState,
    OverworldTransitionState, RoomBoundsState, WeatherVaneState, WorldCameraBoundariesState,
    WorldPaletteThemeState, WorldRegionState, WorldScrollState, WorldTransientState,
    DOOR_ANIMATION_REPLACEMENT_TILE_INDEX,
};
use super::*;

use crate::game_state::constants::messaging as messaging_constants;
use crate::game_state::constants::*;
use crate::types::{read_le_u16, write_le_u16};
use snes::WRAM_SIZE;

#[path = "native_tests/display_tests.rs"]
mod display_tests;
#[path = "native_tests/dungeon_tests.rs"]
mod dungeon_tests;
#[path = "native_tests/effects_tests.rs"]
mod effects_tests;
#[path = "native_tests/ending_tests.rs"]
mod ending_tests;
#[path = "native_tests/frame_tests.rs"]
mod frame_tests;
#[path = "native_tests/inventory_tests.rs"]
mod inventory_tests;
#[path = "native_tests/messaging_tests.rs"]
mod messaging_tests;
#[path = "native_tests/misc_tests.rs"]
mod misc_tests;
#[path = "native_tests/oam_tests.rs"]
mod oam_tests;
#[path = "native_tests/player_tests.rs"]
mod player_tests;
#[path = "native_tests/poly_tests.rs"]
mod poly_tests;
#[path = "native_tests/sprites_tests.rs"]
mod sprites_tests;
#[path = "native_tests/system_tests.rs"]
mod system_tests;
#[path = "native_tests/world_tests.rs"]
mod world_tests;
