//! Native game-state model.
//!
//! Byte-backed views remain the compatibility surface while native state is
//! proven subsystem by subsystem. Native structs own domain fields and can be
//! projected to or loaded from WRAM during the transition.

mod display;
mod dungeon;
mod effects;
mod ending;
mod frame;
mod inventory;
mod messaging;
mod misc;
mod oam;
mod player;
mod poly;
mod sprites;
mod world;

pub(crate) use display::{
    DisplayState, GraphicsDecompressionScratch, HudInventoryOrderState, HudStateRead,
    LinkDmaSourceSlot, NativeAttractVramDestinationBridgeMut, NativeDisplayStateBridgeMut,
    NativeGraphicsScratchBridgeMut, NativeHudInventoryOrderBridgeMut, NativeHudStateBridgeMut,
    NativeOverworldPaletteBackupBridgeMut, NativePaletteBufferBridgeMut,
    NativePaletteFilterBridgeMut, NativePpuScrollCopyBridgeMut, NativeSpotlightHdmaBridgeMut,
    NativeTrinexxPaletteBridgeMut, NativeVramUploadBufferBridgeMut, NativeWaterHdmaWindowBridgeMut,
    PaletteBufferState, PaletteFilterState, PpuScrollCopyState, SpotlightHdmaState,
    TrinexxPaletteState, WaterHdmaWindowState,
};
pub(crate) use dungeon::{
    loaded_room_data_word, DungeonBg2AttributeState, DungeonDoorState, DungeonEnvironmentState,
    DungeonHeaderState, DungeonMovableBlockState, DungeonMovingFloorState,
    DungeonObjectTrackingState, DungeonRoomDoorSetupState, DungeonRoomEffectsState,
    DungeonRoomItemState, DungeonRoomLoadState, DungeonRoomParserState, DungeonRoomRuntimeState,
    DungeonRoomTilemapState, DungeonRoomTrackingState, DungeonSavegameState,
    DungeonScratchWordState, DungeonStairList, DungeonStairListsState, DungeonStairMovementState,
    DungeonState, DungeonTorchState, NativeDungeonBg2AttributeBridgeMut,
    NativeDungeonDoorBridgeMut, NativeDungeonEntranceBackupBridgeMut,
    NativeDungeonEnvironmentBridgeMut, NativeDungeonHeaderBridgeMut,
    NativeDungeonMovableBlockBridgeMut, NativeDungeonMovingFloorBridgeMut,
    NativeDungeonObjectTrackingBridgeMut, NativeDungeonRoomDoorSetupBridgeMut,
    NativeDungeonRoomEffectsBridgeMut, NativeDungeonRoomItemBridgeMut,
    NativeDungeonRoomLoadBridgeMut, NativeDungeonRoomParserBridgeMut,
    NativeDungeonRoomRuntimeBridgeMut, NativeDungeonRoomTilemapBridgeMut,
    NativeDungeonRoomTrackingBridgeMut, NativeDungeonSavegameBridgeMut,
    NativeDungeonScratchWordBridgeMut, NativeDungeonStairListsBridgeMut,
    NativeDungeonStairMovementBridgeMut, NativeDungeonTorchBridgeMut,
};
pub(crate) use effects::{
    BlastWallExplosionSlotState, BlastWallFireballSlotState, BlastWallFragmentSlotState,
    BlastWallState, BombosBlastState, BombosFireColumnState, BombosSpellState,
    DiggingGamePrizeState, DoorDebrisState, EffectAngleScratchState, EffectState,
    HappinessPondRupeeSlotState, HappinessPondRupeeSnapshot, HistoryPositionState,
    LanmolaSegmentMotionState, NativeBeamosLaserHistoryBridgeMut, NativeBlastWallBridgeMut,
    NativeBlastWallExplosionBridgeMut, NativeBlastWallFireballBridgeMut,
    NativeBlastWallFragmentBridgeMut, NativeBombosBlastBridgeMut, NativeBombosFireColumnBridgeMut,
    NativeBombosSpellBridgeMut, NativeDiggingGamePrizeBridgeMut, NativeDoorDebrisBridgeMut,
    NativeEffectAngleScratchBridgeMut, NativeHappinessPondRupeeBridgeMut,
    NativeLanmolaSegmentMotionBridgeMut, NativeMoldormHistoryBridgeMut, NativeQuakeBoltBridgeMut,
    NativeQuakeSpellBridgeMut, NativeSkullWoodsFireBridgeMut, NativeSkullWoodsFireSlotBridgeMut,
    NativeSwamolaHistoryBridgeMut, NativeSwamolaTargetBridgeMut, NativeTowerSealBridgeMut,
    NativeTowerSealOrbitBridgeMut, NativeTowerSealSparkleBridgeMut,
    NativeWeatherVaneDebrisBridgeMut, QuakeBoltSlotState, QuakeSpellState, SkullWoodsFireSlotState,
    SkullWoodsFireState, TowerSealOrbitState, TowerSealSparkleState, TowerSealState,
    WeatherVaneDebrisSlotState,
};
pub(crate) use ending::{
    AttractSceneState, EndingCreditState, EndingState, IntroActorRead, IntroSceneState,
    NativeAttractSceneBridgeMut, NativeEndingCreditBridgeMut, NativeIntroActorBridgeMut,
    NativeIntroSceneBridgeMut,
};
pub(crate) use frame::{
    FrameState, NativeFrameStateBridgeMut, NativeSystemSignalsBridgeMut, SystemSignalsState,
};
pub(crate) use inventory::{
    DungeonKeySlotsState, InventoryItemsState, InventoryState, MirrorWarpState,
    NativeDungeonKeySlotsBridgeMut, NativeInventoryItemsBridgeMut, NativeMirrorWarpBridgeMut,
    NativePlayerResourcesBridgeMut, NativeSaveProgressBridgeMut, PlayerResourcesState,
    SaveProgressState,
};
pub(crate) use messaging::{
    DecodedMessageTextState, DialogueMessageIndexState, DialogueNumberState,
    MessagingRenderBufferState, MessagingRuntimeState, MessagingState, MultiselectChoiceRead,
    NativeDecodedMessageTextBridgeMut, NativeDialogueMessageIndexBridgeMut,
    NativeDialogueNumberBridgeMut, NativeDialogueSourceOffsetBridgeMut,
    NativeMessagingRenderBufferBridgeMut, NativeMessagingRuntimeBridgeMut,
    NativeMultiselectChoiceBridgeMut, NativeSelectFileMenuBridgeMut,
    NativeSharedMessageTimerBridgeMut, NativeVwfRenderBridgeMut, SelectFileMenuState,
    SharedMessageTimerState, VwfRenderState,
};
pub(crate) use misc::{
    ArcheryGameState, DungeonMapDisplayState, DungeonSecretState, EnhancedFeaturesState,
    IntroSwordState, MemorizedTileState, MinigameState, NativeArcheryGameBridgeMut,
    NativeDungeonMapDisplayBridgeMut, NativeDungeonSecretBridgeMut,
    NativeEnhancedFeaturesBridgeMut, NativeIntroSwordBridgeMut, NativeMemorizedTileBridgeMut,
    NativeMinigameBridgeMut, NativeSaveLoadTransferBridgeMut, NativeScratchCounterBridgeMut,
    NativeSpriteBattleBridgeMut, SaveLoadTransferState, ScratchCounterState, SpriteBattleState,
};
pub(crate) use oam::{NativeOamStateBridgeMut, OamState};
pub(crate) use player::{
    Bg1MovementAccumulatorState, NativeBg1MovementAccumulatorBridgeMut, NativePushedBlockBridgeMut,
    NativeSpecialExitPositionBridgeMut, NativeSwimAccelerationBridgeMut,
    NativeTileDetectionBridgeMut, PlayerState, PlayerTileAttributeTableState, PushedBlockState,
    SpecialExitPositionState, SwimAccelerationState, TileDetectionState,
};
pub(crate) use poly::{
    NativePolyFaceCoordsBridgeMut, NativePolyProjectedVerticesBridgeMut,
    NativePolyRasterEdgeBridgeMut, NativePolyRuntimeBridgeMut, PolyFaceCoordsState,
    PolyProjectedVerticesState, PolyRasterEdgeState, PolyRuntimeState, PolyState,
};
pub(crate) use sprites::{
    armos_knight_home_position_from_ram, arrghus_puff_home_position_from_ram, BossHomePositionRead,
    CachedSpriteRead, ChainChompHistoryState, DualLayerTileCacheState,
    EnemyDamageSubclassTableState, EtherOrbitState, FollowerRuntimeState, GarnishRuntimeState,
    MazeGameTimerState, NativeArmosKnightHomePositionBridgeMut, NativeCachedSpriteBridgeMut,
    NativeChainChompHistoryBridgeMut, NativeDualLayerTileCacheBridgeMut,
    NativeEnemyDamageSubclassTableBridgeMut, NativeEtherOrbitBridgeMut,
    NativeFailedSpinSparkleSpawnBridgeMut, NativeFollowerRuntimeBridgeMut,
    NativeGarnishRuntimeBridgeMut, NativeMazeGameTimerBridgeMut,
    NativeOverworldSpriteLoadedBridgeMut, NativeOverworldSpritePresenceBridgeMut,
    NativePrizeDropCycleBridgeMut, NativeSpriteDrawWorkPositionBridgeMut,
    NativeSpriteHitboxWorkOffsetBridgeMut, NativeSpriteSystemBridgeMut,
    NativeSpriteWorkspaceBridgeMut, NativeTagalongSlotBridgeMut, OverworldSpriteLoadedState,
    OverworldSpritePresenceState, SpriteDrawHitboxWorkState, SpriteState, SpriteSystemState,
    SpriteWorkspaceState, TagalongSlotRead,
};
pub(crate) use world::{
    BirdTravelDestinationState, NativeBirdTravelDestinationBridgeMut,
    NativeOverworldConfigTableBridgeMut, NativeOverworldEntranceBridgeMut,
    NativeOverworldEventInfoBridgeMut, NativeOverworldExitBridgeMut, NativeOverworldMap16BridgeMut,
    NativeOverworldMap16DecodeBridgeMut, NativeOverworldMapUiBridgeMut,
    NativeOverworldMapZoomBridgeMut, NativeOverworldScreenSizeBridgeMut,
    NativeOverworldScrollDeltaBridgeMut, NativeOverworldTransitionBridgeMut,
    NativeRoomBoundsBridgeMut, NativeWeatherVaneBridgeMut, NativeWorldCameraBoundariesBridgeMut,
    NativeWorldLocationBridgeMut, NativeWorldPaletteThemeBridgeMut, NativeWorldRegionBridgeMut,
    NativeWorldScrollBridgeMut, NativeWorldTransientBridgeMut, OverworldConfigTableRead,
    OverworldEventInfoState, OverworldMap16Decode, OverworldMap16SourcePage, OverworldMap16State,
    RoomBoundsState, WeatherVaneState, WorldCameraBoundariesState, WorldLocationState,
    WorldPaletteThemeState, WorldRegionState, WorldScrollState, WorldState, WorldTransientState,
};
pub use world::{OverworldMap16LoadState, SmallOverworldMap16ScrollBackupState};

#[cfg(test)]
use crate::game_state::constants::*;
#[cfg(test)]
use crate::types::{read_le_u16, write_le_u16};
#[cfg(test)]
use display::{HudRuntimeState, HudTilemapState, OverworldPaletteBackupState};
#[cfg(test)]
use dungeon::DungeonEntranceBackupState;
#[cfg(test)]
use effects::EntranceEffectState;
#[cfg(test)]
use effects::QuakeBoltState;
#[cfg(test)]
use messaging::{DialoguePointerTableState, DialogueSourceOffsetState, MultiselectChoiceState};
#[cfg(test)]
use sprites::{FailedSpinSparkleSpawnState, PrizeDropCycleState, TagalongTrailState};
#[cfg(test)]
use world::{
    BirdTravelDestinationsState, OverworldConfigTableState, OverworldEntranceState,
    OverworldExitState, OverworldMapUiState, OverworldMapZoomState, OverworldScreenSizeState,
    OverworldScrollDeltaState, OverworldTransitionState,
};

fn ram_byte(ram: &[u8], offset: usize) -> u8 {
    ram.get(offset).copied().unwrap_or(0)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GameState {
    pub(crate) frame: FrameState,
    pub(crate) system_signals: SystemSignalsState,
    pub(crate) enhanced_features: EnhancedFeaturesState,
    pub(crate) scratch_counter: ScratchCounterState,
    pub(crate) minigame: MinigameState,
    pub(crate) intro_sword: IntroSwordState,
    pub(crate) archery_game: ArcheryGameState,
    pub(crate) sprite_battle: SpriteBattleState,
    pub(crate) memorized_tiles: MemorizedTileState,
    pub(crate) dungeon_secret: DungeonSecretState,
    pub(crate) save_load_transfer: SaveLoadTransferState,
    pub(crate) dungeon_map_display: DungeonMapDisplayState,
    pub(crate) dungeon: DungeonState,
    pub(crate) sprites: SpriteState,
    pub(crate) player: PlayerState,
    pub(crate) inventory: InventoryState,
    pub(crate) world: WorldState,
    pub(crate) poly: PolyState,
    pub(crate) display: DisplayState,
    pub(crate) effects: EffectState,
    pub(crate) ending: EndingState,
    pub(crate) messaging: MessagingState,
    pub(crate) oam: OamState,
}

impl GameState {
    pub(crate) fn load_from_ram(ram: &[u8]) -> Self {
        Self {
            frame: FrameState::load_from_ram(ram),
            system_signals: SystemSignalsState::load_from_ram(ram),
            enhanced_features: EnhancedFeaturesState::load_from_ram(ram),
            scratch_counter: ScratchCounterState::load_from_ram(ram),
            minigame: MinigameState::load_from_ram(ram),
            intro_sword: IntroSwordState::load_from_ram(ram),
            archery_game: ArcheryGameState::load_from_ram(ram),
            sprite_battle: SpriteBattleState::load_from_ram(ram),
            memorized_tiles: MemorizedTileState::load_from_ram(ram),
            dungeon_secret: DungeonSecretState::load_from_ram(ram),
            save_load_transfer: SaveLoadTransferState::load_from_ram(ram),
            dungeon_map_display: DungeonMapDisplayState::load_from_ram(ram),
            dungeon: DungeonState::load_from_ram(ram),
            sprites: SpriteState::load_from_ram(ram),
            player: PlayerState::load_from_ram(ram),
            inventory: InventoryState::load_from_ram(ram),
            world: WorldState::load_from_ram(ram),
            poly: PolyState::load_from_ram(ram),
            display: DisplayState::load_from_ram(ram),
            effects: EffectState::load_from_ram(ram),
            ending: EndingState::load_from_ram(ram),
            messaging: MessagingState::load_from_ram(ram),
            oam: OamState::load_from_ram(ram),
        }
    }

    pub(crate) fn write_to_ram(&self, ram: &mut [u8]) {
        self.frame.write_to_ram(ram);
        self.system_signals.write_to_ram(ram);
        self.enhanced_features.write_to_ram(ram);
        self.scratch_counter.write_to_ram(ram);
        self.minigame.write_to_ram(ram);
        self.intro_sword.write_to_ram(ram);
        self.archery_game.write_to_ram(ram);
        self.sprite_battle.write_to_ram(ram);
        self.memorized_tiles.write_to_ram(ram);
        self.dungeon_secret.write_to_ram(ram);
        self.save_load_transfer.write_to_ram(ram);
        self.dungeon_map_display.write_to_ram(ram);
        self.dungeon.write_to_ram(ram);
        self.sprites.write_to_ram(ram);
        self.player.write_to_ram(ram);
        self.inventory.write_to_ram(ram);
        self.ending.write_to_ram(ram);
        self.messaging.write_to_ram(ram);
        self.world.write_to_ram(ram);
        self.poly.write_to_ram(ram);
        self.display.write_to_ram(ram);
        self.effects.write_to_ram(ram);
        self.oam.write_to_ram(ram);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::constants::messaging as messaging_constants;
    use snes::WRAM_SIZE;

    #[test]
    fn frame_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MAIN_MODULE] = 7;
        ram[SUBMODULE] = 2;
        ram[SUBSUBMODULE] = 9;
        ram[FRAME_COUNTER] = 0x42;
        ram[SAVED_MODULE_FOR_MENU] = 5;
        ram[MODAL_PAUSE_FLAG] = 1;

        let mut frame = FrameState::load_from_ram(&ram);
        assert_eq!(frame.main_module, 7);
        assert_eq!(frame.main_module_word(), 0x0207);
        assert_eq!(frame.submodule, 2);
        assert_eq!(frame.subsubmodule, 9);
        assert_eq!(frame.frame_counter, 0x42);
        assert_eq!(frame.saved_module_for_menu, 5);
        assert_eq!(frame.modal_pause_flag, 1);

        frame.main_module = 14;
        frame.submodule = 3;
        frame.subsubmodule = 1;
        frame.frame_counter = 0x80;
        frame.saved_module_for_menu = 7;
        frame.modal_pause_flag = 2;
        frame.write_to_ram(&mut ram);

        assert_eq!(ram[MAIN_MODULE], 14);
        assert_eq!(ram[SUBMODULE], 3);
        assert_eq!(ram[SUBSUBMODULE], 1);
        assert_eq!(ram[FRAME_COUNTER], 0x80);
        assert_eq!(ram[SAVED_MODULE_FOR_MENU], 7);
        assert_eq!(ram[MODAL_PAUSE_FLAG], 2);
    }

    #[test]
    fn native_frame_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MAIN_MODULE] = 1;
        ram[SUBMODULE] = 2;
        ram[SUBSUBMODULE] = 3;
        ram[FRAME_COUNTER] = 4;
        ram[SAVED_MODULE_FOR_MENU] = 8;
        ram[MODAL_PAUSE_FLAG] = 1;

        let mut frame = FrameState::load_from_ram(&ram);
        {
            let mut bridge = NativeFrameStateBridgeMut::new(&mut frame, &mut ram);
            bridge.increment_submodule();
            bridge.set_subsubmodule(9);
            bridge.increment_frame_counter();
            bridge.save_main_module_for_menu();
            bridge.clear_saved_module_for_menu();
            bridge.save_submodule_for_menu();
            bridge.clear_modal_pause_flag();
            bridge.increment_modal_pause_flag();
            bridge.set_modal_pause_flag(6);
        }

        assert_eq!(frame.main_module, 1);
        assert_eq!(frame.submodule, 3);
        assert_eq!(frame.subsubmodule, 9);
        assert_eq!(frame.frame_counter, 5);
        assert_eq!(frame.saved_module_for_menu, 3);
        assert_eq!(frame.modal_pause_flag, 6);
        assert_eq!(ram[SUBMODULE], 3);
        assert_eq!(ram[SUBSUBMODULE], 9);
        assert_eq!(ram[FRAME_COUNTER], 5);
        assert_eq!(ram[SAVED_MODULE_FOR_MENU], 3);
        assert_eq!(ram[MODAL_PAUSE_FLAG], 6);
    }

    #[test]
    fn native_frame_bridge_projects_native_state_over_stale_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut frame = FrameState {
            main_module: 7,
            submodule: 2,
            subsubmodule: 9,
            frame_counter: 0x42,
            saved_module_for_menu: 5,
            modal_pause_flag: 1,
        };
        frame.write_to_ram(&mut ram);

        ram[MAIN_MODULE] = 0xaa;
        ram[FRAME_COUNTER] = 0xbb;

        {
            let mut bridge = NativeFrameStateBridgeMut::new(&mut frame, &mut ram);
            bridge.set_submodule(3);
        }

        assert_eq!(frame.main_module, 7);
        assert_eq!(frame.submodule, 3);
        assert_eq!(frame.frame_counter, 0x42);
        assert_eq!(FrameState::load_from_ram(&ram), frame);
        assert_eq!(ram[MAIN_MODULE], 7);
        assert_eq!(ram[SUBMODULE], 3);
        assert_eq!(ram[FRAME_COUNTER], 0x42);
    }

    #[test]
    fn system_signals_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MUSIC_CONTROL] = 0xf2;
        ram[CURRENT_MUSIC_CONTROL] = 0x13;
        ram[LAST_MUSIC_CONTROL] = 0x12;
        ram[QUEUED_MUSIC_CONTROL] = 0x09;
        ram[SOUND_EFFECT_1] = 0x2d;
        ram[SOUND_EFFECT_2] = 0x1b;
        ram[SOUND_EFFECT_AMBIENT] = 0x05;
        ram[SOUND_EFFECT_AMBIENT_LAST] = 0x07;
        ram[MSU_VOLUME] = 0x80;
        ram[RAM_APUI00] = 1;
        ram[RAW_SFX_PAN_VALUE] = 0xc0;
        ram[FLAG_UPDATE_CGRAM_IN_NMI] = 2;
        ram[FLAG_UPDATE_HUD_IN_NMI] = 3;
        ram[GAME_OVER_CHECK_FLAG] = 4;
        ram[RESTART_CHECK_FLAG] = 5;
        ram[RAM_BUGS_FIXED] = 0x42;
        ram[DEATH_BACKUP_CURRENT_MUSIC] = 0x22;
        ram[DEATH_BACKUP_AMBIENT_SOUND] = 0x33;

        let system_signals = SystemSignalsState::load_from_ram(&ram);
        assert_eq!(system_signals.music_control(), 0xf2);
        assert_eq!(system_signals.current_music_control(), 0x13);
        assert_eq!(system_signals.last_music_control(), 0x12);
        assert_eq!(system_signals.queued_music_control(), 0x09);
        assert_eq!(system_signals.sound_effect_1(), 0x2d);
        assert_eq!(system_signals.sound_effect_2(), 0x1b);
        assert_eq!(system_signals.ambient_sound_effect(), 0x05);
        assert_eq!(system_signals.last_ambient_sound_effect(), 0x07);
        assert_eq!(system_signals.msu_volume(), 0x80);
        assert_eq!(system_signals.apui00(), 1);
        assert_eq!(system_signals.raw_sfx_pan_value(), 0xc0);
        assert!(system_signals.should_update_cgram());
        assert!(system_signals.should_update_hud());
        assert_eq!(system_signals.game_over_check_flag(), 4);
        assert_eq!(system_signals.restart_check_flag(), 5);
        assert_eq!(system_signals.bugs_fixed(), 0x42);
        assert_eq!(system_signals.death_backup_current_music(), 0x22);
        assert_eq!(system_signals.death_backup_ambient_sound(), 0x33);

        let mut projected = vec![0; WRAM_SIZE];
        system_signals.write_to_ram(&mut projected);
        for offset in [
            MUSIC_CONTROL,
            CURRENT_MUSIC_CONTROL,
            LAST_MUSIC_CONTROL,
            QUEUED_MUSIC_CONTROL,
            SOUND_EFFECT_1,
            SOUND_EFFECT_2,
            SOUND_EFFECT_AMBIENT,
            SOUND_EFFECT_AMBIENT_LAST,
            MSU_VOLUME,
            RAM_APUI00,
            RAW_SFX_PAN_VALUE,
            FLAG_UPDATE_CGRAM_IN_NMI,
            FLAG_UPDATE_HUD_IN_NMI,
            GAME_OVER_CHECK_FLAG,
            RESTART_CHECK_FLAG,
            RAM_BUGS_FIXED,
            DEATH_BACKUP_CURRENT_MUSIC,
            DEATH_BACKUP_AMBIENT_SOUND,
        ] {
            assert_eq!(projected[offset], ram[offset]);
        }
    }

    #[test]
    fn native_system_signals_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[CURRENT_MUSIC_CONTROL] = 0x12;
        ram[SOUND_EFFECT_AMBIENT] = 0x05;
        ram[SOUND_EFFECT_1] = 0;
        ram[SOUND_EFFECT_2] = 0;
        ram[FLAG_UPDATE_CGRAM_IN_NMI] = 0xff;
        ram[FLAG_UPDATE_HUD_IN_NMI] = 1;
        ram[GAME_OVER_CHECK_FLAG] = 7;

        let mut system_signals = SystemSignalsState::load_from_ram(&ram);
        {
            let mut bridge = NativeSystemSignalsBridgeMut::new(&mut system_signals, &mut ram);
            assert!(bridge.queue_sound_effect_1_if_empty(0x2d));
            assert!(bridge.queue_sound_effect_2_if_empty(0x1b));
            assert!(!bridge.queue_sound_effect_1_if_empty(0x33));
            bridge.set_sound_effect_1_word(0x3412);
            bridge.set_ambient_sound_effect_word(0x5607);
            bridge.save_current_music_as_last();
            bridge.save_ambient_sound_effect_as_last();
            bridge.increment_cgram_update_flag();
            bridge.increment_hud_update_flag();
            bridge.clear_game_over_check_flag();
            bridge.set_restart_check_flag(9);
            bridge.set_raw_sfx_pan_value(0x80);
            bridge.set_death_backup_current_music(0x21);
            bridge.set_death_backup_ambient_sound(0x22);
        }

        assert_eq!(system_signals.sound_effect_1(), 0x56);
        assert_eq!(system_signals.sound_effect_2(), 0x34);
        assert_eq!(system_signals.ambient_sound_effect(), 0x07);
        assert_eq!(system_signals.last_music_control(), 0x12);
        assert_eq!(system_signals.last_ambient_sound_effect(), 0x07);
        assert_eq!(system_signals.raw_sfx_pan_value(), 0x80);
        assert_eq!(system_signals.restart_check_flag(), 9);
        assert_eq!(system_signals.death_backup_current_music(), 0x21);
        assert_eq!(system_signals.death_backup_ambient_sound(), 0x22);
        assert_eq!(ram[SOUND_EFFECT_1], 0x56);
        assert_eq!(ram[SOUND_EFFECT_2], 0x34);
        assert_eq!(ram[SOUND_EFFECT_AMBIENT], 0x07);
        assert_eq!(ram[LAST_MUSIC_CONTROL], 0x12);
        assert_eq!(ram[SOUND_EFFECT_AMBIENT_LAST], 0x07);
        assert_eq!(ram[FLAG_UPDATE_CGRAM_IN_NMI], 0);
        assert_eq!(ram[FLAG_UPDATE_HUD_IN_NMI], 2);
        assert_eq!(ram[GAME_OVER_CHECK_FLAG], 0);
        assert_eq!(ram[RESTART_CHECK_FLAG], 9);
        assert_eq!(ram[RAW_SFX_PAN_VALUE], 0x80);
        assert_eq!(ram[DEATH_BACKUP_CURRENT_MUSIC], 0x21);
        assert_eq!(ram[DEATH_BACKUP_AMBIENT_SOUND], 0x22);
    }

    #[test]
    fn enhanced_features_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[ENHANCED_FEATURE_FLAGS] = 0x78;
        ram[ENHANCED_FEATURE_FLAGS + 1] = 0x56;
        ram[ENHANCED_FEATURE_FLAGS + 2] = 0x34;
        ram[ENHANCED_FEATURE_FLAGS + 3] = 0x12;

        let features = EnhancedFeaturesState::load_from_ram(&ram);
        assert_eq!(features.bits(), 0x1234_5678);
        assert!(features.has(0x1000_0000));
        assert!(!features.is_empty());

        let mut projected = vec![0; WRAM_SIZE];
        features.write_to_ram(&mut projected);
        assert_eq!(
            &projected[ENHANCED_FEATURE_FLAGS..ENHANCED_FEATURE_FLAGS + 4],
            &[0x78, 0x56, 0x34, 0x12]
        );
    }

    #[test]
    fn native_enhanced_features_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, ENHANCED_FEATURE_FLAGS, 0x1000);

        let mut features = EnhancedFeaturesState::default();
        {
            let mut bridge = NativeEnhancedFeaturesBridgeMut::new(&mut features, &mut ram);
            bridge.set_bits(0x1234_5678);
        }

        assert_eq!(features.bits(), 0x1234_5678);
        assert_eq!(read_le_u16(&ram, ENHANCED_FEATURE_FLAGS), 0x5678);
        assert_eq!(read_le_u16(&ram, ENHANCED_FEATURE_FLAGS + 2), 0x1234);
    }

    #[test]
    fn scratch_counter_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TEMP_COUNTER] = 0x80;

        let mut counter = ScratchCounterState::load_from_ram(&ram);
        assert_eq!(counter.value(), 0x80);
        assert_eq!(counter.as_usize(), 0x80);
        assert!(counter.is_negative());
        counter.set(2);
        assert_eq!(counter.decrement(), 1);
        counter.write_to_ram(&mut ram);

        assert_eq!(ram[TEMP_COUNTER], 1);
    }

    #[test]
    fn native_scratch_counter_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TEMP_COUNTER] = 0;

        let mut counter = ScratchCounterState::default();
        {
            let mut bridge = NativeScratchCounterBridgeMut::new(&mut counter, &mut ram);
            assert_eq!(bridge.decrement(), 0xff);
            bridge.set(7);
        }

        assert_eq!(counter.value(), 7);
        assert_eq!(ram[TEMP_COUNTER], 7);
    }

    #[test]
    fn memorized_tile_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, NUM_MEMORIZED_TILES, 4);
        write_le_u16(&mut ram, MEMORIZED_TILE_ADDR, 0x1111);
        write_le_u16(&mut ram, MEMORIZED_TILE_VALUE, 0x2222);
        write_le_u16(&mut ram, MEMORIZED_TILE_ADDR + 2, 0x3333);
        write_le_u16(&mut ram, MEMORIZED_TILE_VALUE + 2, 0x4444);

        let mut memorized_tiles = MemorizedTileState::load_from_ram(&ram);
        assert_eq!(memorized_tiles.count(), 4);
        assert_eq!(memorized_tiles.entry_addr(0), 0x1111);
        assert_eq!(memorized_tiles.entry_value(0), 0x2222);
        assert_eq!(memorized_tiles.entry_addr(1), 0x3333);
        assert_eq!(memorized_tiles.entry_value(1), 0x4444);
        assert_eq!(memorized_tiles.entry_addr(0x80), 0);

        memorized_tiles.append_entry(0x5555, 0x6666);
        memorized_tiles.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, NUM_MEMORIZED_TILES), 6);
        assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR + 4), 0x5555);
        assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 4), 0x6666);
    }

    #[test]
    fn native_memorized_tile_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, NUM_MEMORIZED_TILES, 2);
        write_le_u16(&mut ram, MEMORIZED_TILE_ADDR, 0x1111);
        write_le_u16(&mut ram, MEMORIZED_TILE_VALUE, 0x2222);
        write_le_u16(&mut ram, MEMORIZED_TILE_ADDR + 0xfe, 0xffff);

        let mut memorized_tiles = MemorizedTileState::default();
        {
            let mut bridge = NativeMemorizedTileBridgeMut::new(&mut memorized_tiles, &mut ram);
            bridge.append_entry(0x3333, 0x4444);
            bridge.set_entry_addr(4, 0x5555);
            bridge.set_entry_value(4, 0x6666);
            bridge.set_count(6);
            bridge.clear_entry_addresses();
        }

        assert_eq!(memorized_tiles.count(), 6);
        assert_eq!(memorized_tiles.entry_addr(0), 0);
        assert_eq!(memorized_tiles.entry_addr(1), 0);
        assert_eq!(memorized_tiles.entry_addr(2), 0);
        assert_eq!(memorized_tiles.entry_value(1), 0x4444);
        assert_eq!(memorized_tiles.entry_value(2), 0x6666);
        assert_eq!(read_le_u16(&ram, NUM_MEMORIZED_TILES), 6);
        assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR), 0);
        assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_ADDR + 0xfe), 0);
        assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 2), 0x4444);
        assert_eq!(read_le_u16(&ram, MEMORIZED_TILE_VALUE + 4), 0x6666);
    }

    #[test]
    fn dungeon_secret_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DUNGEON_SECRET_PENDING_KIND] = 0x82;
        ram[DUNGEON_SECRET_PENDING_KIND + 1] = 0x7f;
        ram[OVERWORLD_SECRET_SUBST_CTR] = 3;

        let mut secret = DungeonSecretState::load_from_ram(&ram);
        assert_eq!(secret.pending_kind(), 0x82);
        assert_eq!(secret.graphics_kind(), Some(2));
        assert!(secret.has_pending_kind());
        assert!(secret.is_available());
        assert_eq!(secret.overworld_subst_counter(), 3);

        secret.set_powder_pending_kind();
        secret.increment_overworld_subst_counter();
        secret.mark_graphics_kind();
        secret.write_to_ram(&mut ram);

        assert_eq!(ram[DUNGEON_SECRET_PENDING_KIND], 0x84);
        assert_eq!(ram[DUNGEON_SECRET_PENDING_KIND + 1], 0);
        assert_eq!(ram[OVERWORLD_SECRET_SUBST_CTR], 4);
    }

    #[test]
    fn native_dungeon_secret_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DUNGEON_SECRET_PENDING_KIND] = 0xff;
        ram[DUNGEON_SECRET_PENDING_KIND + 1] = 0x44;
        ram[OVERWORLD_SECRET_SUBST_CTR] = 0xff;

        let mut secret = DungeonSecretState::default();
        {
            let mut bridge = NativeDungeonSecretBridgeMut::new(&mut secret, &mut ram);
            bridge.clear_pending_kind();
            bridge.set_pending_kind(2);
            bridge.or_pending_kind(4);
            bridge.mark_graphics_kind();
            bridge.increment_overworld_subst_counter();
            bridge.set_powder_pending_kind();
        }

        assert_eq!(secret.pending_kind(), 4);
        assert_eq!(secret.graphics_kind(), None);
        assert_eq!(secret.overworld_subst_counter(), 0);
        assert_eq!(ram[DUNGEON_SECRET_PENDING_KIND], 4);
        assert_eq!(ram[DUNGEON_SECRET_PENDING_KIND + 1], 0);
        assert_eq!(ram[OVERWORLD_SECRET_SUBST_CTR], 0);
    }

    #[test]
    fn save_load_transfer_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, SAVE_LOAD_SOURCE_OFFSET, 0x1234);

        let mut transfer = SaveLoadTransferState::load_from_ram(&ram);
        assert_eq!(transfer.source_offset(), 0x1234);
        assert_eq!(transfer.source_offset_usize(), 0x1234);

        transfer.set_source_offset(0x4567);
        transfer.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, SAVE_LOAD_SOURCE_OFFSET), 0x4567);
    }

    #[test]
    fn native_save_load_transfer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, SAVE_LOAD_SOURCE_OFFSET, 0x1234);

        let mut transfer = SaveLoadTransferState::default();
        {
            let mut bridge = NativeSaveLoadTransferBridgeMut::new(&mut transfer, &mut ram);
            bridge.set_source_offset(0x4567);
        }

        assert_eq!(transfer.source_offset(), 0x4567);
        assert_eq!(read_le_u16(&ram, SAVE_LOAD_SOURCE_OFFSET), 0x4567);
    }

    #[test]
    fn dungeon_map_display_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_MAP_SCROLL_DRAW_OFFSET, 0x0010);
        write_le_u16(&mut ram, DUNGEON_MAP_SCROLL_INPUT, 0x0008);
        write_le_u16(&mut ram, DUNGEON_MAP_MARKER_X_OFFSET, 0x0044);
        write_le_u16(&mut ram, DUNGEON_MAP_MARKER_Y_OFFSET, 0x0055);
        write_le_u16(&mut ram, DUNGEON_MAP_LOCATION_MARKER_BASE_Y, 0xabcd);
        ram[DUNGMAP_INIT_STATE] = 2;
        write_le_u16(&mut ram, DUNGMAP_CUR_FLOOR, 0x1234);
        ram[DUNGMAP_FLOOR_SCROLL_STEP] = 3;
        write_le_u16(&mut ram, DUNGMAP_IDX, 0x0006);
        write_le_u16(&mut ram, DUNGMAP_SCROLL_TARGET_Y, 0x0080);
        write_le_u16(&mut ram, DUNGMAP_PLAYER_MARKER_X, 0x0090);
        write_le_u16(&mut ram, DUNGMAP_PLAYER_MARKER_Y, 0x00a0);

        let mut display = DungeonMapDisplayState::load_from_ram(&ram);
        assert_eq!(display.scroll_draw_offset(), 0x0010);
        assert_eq!(display.scroll_input_direction_index(), 1);
        assert_eq!(display.marker_x_offset(), 0x0044);
        assert_eq!(display.marker_y_offset(), 0x0055);
        assert_eq!(display.location_marker_base_y(), 0xcd);
        assert_eq!(display.dungmap_init_state(), 2);
        assert_eq!(display.current_floor(), 0x1234);
        assert_eq!(display.dungmap_cur_floor(), 0x1234);
        assert_eq!(display.dungmap_floor_scroll_step(), 3);
        assert_eq!(display.dungmap_idx(), 0x0006);
        assert_eq!(display.dungmap_scroll_target_y(), 0x0080);
        assert_eq!(display.dungmap_player_marker_x(), 0x0090);
        assert_eq!(display.dungmap_player_marker_y(), 0x00a0);

        display.clear_current_floor_high();
        display.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, DUNGMAP_CUR_FLOOR), 0x0034);
        assert_eq!(read_le_u16(&ram, DUNGEON_MAP_MARKER_X_OFFSET), 0x0044);
    }

    #[test]
    fn native_dungeon_map_display_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_MAP_MARKER_Y_OFFSET, 0x01f0);
        write_le_u16(&mut ram, DUNGMAP_CUR_FLOOR, 0x1234);

        let mut display = DungeonMapDisplayState::default();
        {
            let mut bridge = NativeDungeonMapDisplayBridgeMut::new(&mut display, &mut ram);
            bridge.reset_marker_offsets();
            bridge.shift_marker_x_left();
            bridge.set_location_marker_base_y(0x77);
            bridge.set_scroll_input(0x0008);
            bridge.increment_dungmap_init_state();
            bridge.set_dungmap_floor_scroll_step(4);
            bridge.set_dungmap_idx(0x0012);
            bridge.set_dungmap_scroll_target_y(0x0070);
            bridge.set_dungmap_player_marker_x(0x0088);
            bridge.set_dungmap_player_marker_y(0x0099);
            bridge.clear_current_floor_high();
        }

        assert_eq!(display.marker_x_offset(), 0x0030);
        assert_eq!(display.marker_y_offset(), 0x0040);
        assert_eq!(display.location_marker_base_y(), 0x77);
        assert_eq!(display.scroll_input_direction_index(), 1);
        assert_eq!(display.dungmap_init_state(), 1);
        assert_eq!(display.dungmap_floor_scroll_step(), 4);
        assert_eq!(display.dungmap_idx(), 0x0012);
        assert_eq!(display.dungmap_scroll_target_y(), 0x0070);
        assert_eq!(display.dungmap_player_marker_x_byte(), 0x88);
        assert_eq!(display.dungmap_player_marker_y(), 0x0099);
        assert_eq!(display.current_floor(), 0x0034);
        assert_eq!(read_le_u16(&ram, DUNGMAP_CUR_FLOOR), 0x0034);
        assert_eq!(read_le_u16(&ram, DUNGEON_MAP_MARKER_X_OFFSET), 0x0030);
        assert_eq!(read_le_u16(&ram, DUNGEON_MAP_MARKER_Y_OFFSET), 0x0040);
        assert_eq!(
            read_le_u16(&ram, DUNGEON_MAP_LOCATION_MARKER_BASE_Y),
            0x0077
        );
    }

    #[test]
    fn dungeon_header_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS] = 0x12;
        ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS + 4] = 0x34;
        ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE] = 1;
        ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 1] = 2;
        ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 2] = 3;
        ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 3] = 0;
        ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 4] = 1;

        let mut header = DungeonHeaderState::load_from_ram(&ram);
        assert_eq!(header.travel_destination(0), 0x12);
        assert_eq!(header.travel_destination(4), 0x34);
        assert_eq!(header.hole_teleporter_plane(0), 1);
        assert_eq!(header.hole_teleporter_plane(4), 1);
        assert_eq!(header.staircase_plane(0), 2);
        assert_eq!(header.staircase_plane(3), 1);

        header.set_hole_teleporter_planes(0b11_10_01_00, 0b101);
        header.write_to_ram(&mut ram);

        assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE], 0);
        assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 1], 1);
        assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 2], 2);
        assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 3], 3);
        assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 4], 1);
        assert_eq!(ram[DUNGEON_HEADER_STAIRCASE_PLANE], 1);
        assert_eq!(ram[DUNGEON_HEADER_STAIRCASE_PLANE + 3], 1);
        assert_eq!(ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS], 0x12);
        assert_eq!(ram[DUNGEON_HEADER_TRAVEL_DESTINATIONS + 4], 0x34);
    }

    #[test]
    fn native_dungeon_header_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE] = 3;

        let mut header = DungeonHeaderState::default();
        {
            let mut bridge = NativeDungeonHeaderBridgeMut::new(&mut header, &mut ram);
            bridge.set_hole_teleporter_planes(0b00_11_10_01, 2);
        }

        assert_eq!(header.hole_teleporter_plane(0), 1);
        assert_eq!(header.hole_teleporter_plane(1), 2);
        assert_eq!(header.hole_teleporter_plane(2), 3);
        assert_eq!(header.hole_teleporter_plane(3), 0);
        assert_eq!(header.hole_teleporter_plane(4), 2);
        assert_eq!(header.staircase_plane(0), 2);
        assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE], 1);
        assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 1], 2);
        assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 2], 3);
        assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 3], 0);
        assert_eq!(ram[DUNGEON_HEADER_HOLE_TELEPORTER_PLANE + 4], 2);
    }

    #[test]
    fn dungeon_scratch_word_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_WORK_R16, 0x1201);
        write_le_u16(&mut ram, DUNGEON_WORK_R18, 0x3456);

        let mut scratch = DungeonScratchWordState::load_from_ram(&ram);
        assert_eq!(scratch.high(), 0x12);
        assert_eq!(scratch.word(), 0x1201);
        assert_eq!(scratch.minigame_previous_chest_choice(), 1);
        assert_eq!(scratch.primary_word(), 0x1201);
        assert_eq!(scratch.secondary_word(), 0x3456);
        assert_eq!(scratch.primary_low(), 1);
        assert_eq!(scratch.secondary_low(), 0x56);

        assert_eq!(scratch.decrement_high(), 0x11);
        assert_eq!(scratch.decrement_ganon_door_bounce_low(), 0);
        scratch.set_liftable_tile_probe_position(0x789a, 0xbcde);
        scratch.set_minigame_previous_chest_choice(0xef);
        scratch.set_primary_low(0x34);
        assert_eq!(scratch.decrement_primary_low(), 0x33);
        assert_eq!(scratch.increment_secondary_low(), 0xdf);
        scratch.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, ENDING_WORK_PRIMARY), 0x7833);
        assert_eq!(read_le_u16(&ram, ENDING_WORK_SECONDARY), 0xbcdf);
    }

    #[test]
    fn native_dungeon_scratch_word_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_WORK_R16, 0x0201);

        let mut scratch = DungeonScratchWordState::default();
        {
            let mut bridge = NativeDungeonScratchWordBridgeMut::new(&mut scratch, &mut ram);
            assert_eq!(bridge.decrement_high(), 1);
            bridge.set_ganon_door_bounce_countdown(0x0002);
            assert_eq!(bridge.decrement_ganon_door_bounce_low(), 1);
            bridge.clear_module_transition_counter();
            bridge.set_minigame_previous_chest_choice(7);
            bridge.set_liftable_tile_probe_position(0x1234, 0x5678);
            bridge.clear_word();
            bridge.set_word(0xabcd);
            bridge.set_primary_word(0x1234);
            bridge.set_secondary_word(0x5678);
            bridge.clear_primary_word();
            bridge.set_primary_low(2);
            assert_eq!(bridge.decrement_primary_low(), 1);
            assert_eq!(bridge.increment_secondary_low(), 0x79);
        }

        assert_eq!(scratch.word(), 1);
        assert_eq!(scratch.primary_word(), 1);
        assert_eq!(scratch.secondary_word(), 0x5679);
        assert_eq!(read_le_u16(&ram, DUNGEON_WORK_R16), 1);
        assert_eq!(read_le_u16(&ram, ENDING_WORK_PRIMARY), 1);
        assert_eq!(read_le_u16(&ram, DUNGEON_WORK_R18), 0x5679);
        assert_eq!(read_le_u16(&ram, ENDING_WORK_SECONDARY), 0x5679);
    }

    #[test]
    fn minigame_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[IS_ARCHER_OR_SHOVEL_GAME] = 2;
        ram[MINIGAME_CREDITS] = 3;
        ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 1;
        write_le_u16(&mut ram, BOOMERANG_TEMP_X, 0x1234);
        write_le_u16(&mut ram, BOOMERANG_TEMP_Y, 0xabcd);

        let mut minigame = MinigameState::load_from_ram(&ram);
        assert_eq!(minigame.is_archer_or_shovel_game(), 2);
        assert_eq!(minigame.credits(), 3);
        assert_eq!(minigame.flag_boomerang_in_place(), 1);
        assert_eq!(minigame.boomerang_temp_x(), 0x1234);
        assert_eq!(minigame.boomerang_temp_y(), 0xabcd);

        minigame.clear_is_archer_or_shovel_game();
        minigame.decrement_credits();
        minigame.clear_flag_boomerang_in_place();
        minigame.set_boomerang_temp_x(0x4567);
        minigame.set_boomerang_temp_y(0xcdef);
        minigame.write_to_ram(&mut ram);

        assert_eq!(ram[IS_ARCHER_OR_SHOVEL_GAME], 0);
        assert_eq!(ram[MINIGAME_CREDITS], 2);
        assert_eq!(ram[FLAG_FOR_BOOMERANG_IN_PLACE], 0);
        assert_eq!(read_le_u16(&ram, BOOMERANG_TEMP_X), 0x4567);
        assert_eq!(read_le_u16(&ram, BOOMERANG_TEMP_Y), 0xcdef);
    }

    #[test]
    fn native_minigame_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[IS_ARCHER_OR_SHOVEL_GAME] = 2;
        ram[MINIGAME_CREDITS] = 3;
        ram[FLAG_FOR_BOOMERANG_IN_PLACE] = 1;
        write_le_u16(&mut ram, BOOMERANG_TEMP_X, 0x1234);
        write_le_u16(&mut ram, BOOMERANG_TEMP_Y, 0xabcd);

        let mut minigame = MinigameState::default();
        {
            let mut bridge = NativeMinigameBridgeMut::new(&mut minigame, &mut ram);
            bridge.clear_is_archer_or_shovel_game();
            bridge.set_credits(5);
            bridge.decrement_credits();
            bridge.clear_flag_boomerang_in_place();
            bridge.set_boomerang_temp_x(0x4567);
            bridge.set_boomerang_temp_y(0xcdef);
        }

        assert_eq!(minigame.is_archer_or_shovel_game(), 0);
        assert_eq!(minigame.credits(), 4);
        assert_eq!(minigame.flag_boomerang_in_place(), 0);
        assert_eq!(minigame.boomerang_temp_x(), 0x4567);
        assert_eq!(minigame.boomerang_temp_y(), 0xcdef);
        assert_eq!(ram[IS_ARCHER_OR_SHOVEL_GAME], 0);
        assert_eq!(ram[MINIGAME_CREDITS], 4);
        assert_eq!(ram[FLAG_FOR_BOOMERANG_IN_PLACE], 0);
        assert_eq!(read_le_u16(&ram, BOOMERANG_TEMP_X), 0x4567);
        assert_eq!(read_le_u16(&ram, BOOMERANG_TEMP_Y), 0xcdef);
    }

    #[test]
    fn intro_sword_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, INTRO_SWORD_YPOS, 0x1234);
        ram[INTRO_SWORD_SPARKLE_TIMER] = 5;
        ram[INTRO_SWORD_SPARKLE_STEP] = 1;
        ram[INTRO_SWORD_ANIM_STEP] = 4;
        ram[INTRO_SWORD_SPARKLE_Y_OFFSET] = 7;
        write_le_u16(&mut ram, INTRO_SWORD_FLASH_RGB_CHANNEL, 0xab02);

        let mut intro_sword = IntroSwordState::load_from_ram(&ram);
        assert_eq!(intro_sword.ypos(), 0x1234);
        assert_eq!(intro_sword.sparkle_timer(), 5);
        assert_eq!(intro_sword.sparkle_step(), 1);
        assert_eq!(intro_sword.anim_phase(), 2);
        assert_eq!(intro_sword.anim_step_raw(), 4);
        assert_eq!(intro_sword.sparkle_y_offset(), 7);
        assert_eq!(intro_sword.flash_rgb_channel(), 2);

        intro_sword.advance_ypos();
        intro_sword.decrement_sparkle_timer();
        intro_sword.advance_anim_step();
        intro_sword.advance_sparkle_y_offset();
        intro_sword.cycle_flash_rgb_channel();
        intro_sword.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, INTRO_SWORD_YPOS), 0x1244);
        assert_eq!(ram[INTRO_SWORD_SPARKLE_TIMER], 4);
        assert_eq!(ram[INTRO_SWORD_ANIM_STEP], 6);
        assert_eq!(ram[INTRO_SWORD_SPARKLE_Y_OFFSET], 11);
        assert_eq!(read_le_u16(&ram, INTRO_SWORD_FLASH_RGB_CHANNEL), 0xab00);
    }

    #[test]
    fn native_intro_sword_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, INTRO_SWORD_YPOS, 0x1234);
        ram[INTRO_SWORD_SPARKLE_TIMER] = 5;
        ram[INTRO_SWORD_SPARKLE_STEP] = 0;
        ram[INTRO_SWORD_ANIM_STEP] = 4;
        ram[INTRO_SWORD_SPARKLE_Y_OFFSET] = 7;
        write_le_u16(&mut ram, INTRO_SWORD_FLASH_RGB_CHANNEL, 0xab01);

        let mut intro_sword = IntroSwordState::default();
        {
            let mut bridge = NativeIntroSwordBridgeMut::new(&mut intro_sword, &mut ram);
            bridge.advance_ypos();
            bridge.decrement_sparkle_timer();
            assert!(bridge.decrement_sparkle_step_check_negative());
            bridge.advance_anim_step();
            bridge.advance_sparkle_y_offset();
            bridge.cycle_flash_rgb_channel();
            bridge.set_flash_rgb_channel_word(0x0201);
        }

        assert_eq!(intro_sword.ypos(), 0x1244);
        assert_eq!(intro_sword.sparkle_timer(), 4);
        assert_eq!(intro_sword.sparkle_step(), 0xff);
        assert_eq!(intro_sword.anim_step_raw(), 6);
        assert_eq!(intro_sword.sparkle_y_offset(), 11);
        assert_eq!(intro_sword.flash_rgb_channel(), 1);
        assert_eq!(read_le_u16(&ram, INTRO_SWORD_YPOS), 0x1244);
        assert_eq!(ram[INTRO_SWORD_SPARKLE_TIMER], 4);
        assert_eq!(ram[INTRO_SWORD_SPARKLE_STEP], 0xff);
        assert_eq!(ram[INTRO_SWORD_ANIM_STEP], 6);
        assert_eq!(ram[INTRO_SWORD_SPARKLE_Y_OFFSET], 11);
        assert_eq!(read_le_u16(&ram, INTRO_SWORD_FLASH_RGB_CHANNEL), 0x0201);
    }

    #[test]
    fn archery_game_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[ARCHERY_GAME_HIT_COUNTER] = 8;
        ram[ARCHERY_GAME_ARROWS_LEFT] = 5;
        ram[ARCHERY_GAME_OUT_OF_ARROWS] = 1;

        let mut archery = ArcheryGameState::load_from_ram(&ram);
        assert_eq!(archery.hit_counter(), 8);
        assert_eq!(archery.arrows_left(), 5);
        assert_eq!(archery.out_of_arrows(), 1);

        archery.increment_hit_counter();
        archery.decrement_arrows_left();
        archery.clear_out_of_arrows();
        archery.write_to_ram(&mut ram);

        assert_eq!(ram[ARCHERY_GAME_HIT_COUNTER], 9);
        assert_eq!(ram[ARCHERY_GAME_ARROWS_LEFT], 4);
        assert_eq!(ram[ARCHERY_GAME_OUT_OF_ARROWS], 0);
    }

    #[test]
    fn native_archery_game_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[ARCHERY_GAME_HIT_COUNTER] = 0xff;
        ram[ARCHERY_GAME_ARROWS_LEFT] = 0;
        ram[ARCHERY_GAME_OUT_OF_ARROWS] = 0xff;

        let mut archery = ArcheryGameState::default();
        {
            let mut bridge = NativeArcheryGameBridgeMut::new(&mut archery, &mut ram);
            bridge.increment_hit_counter();
            bridge.clear_hit_counter();
            bridge.set_arrows_left(5);
            bridge.decrement_arrows_left();
            bridge.increment_out_of_arrows();
            bridge.clear_out_of_arrows();
        }

        assert_eq!(archery.hit_counter(), 0);
        assert_eq!(archery.arrows_left(), 4);
        assert_eq!(archery.out_of_arrows(), 0);
        assert_eq!(ram[ARCHERY_GAME_HIT_COUNTER], 0);
        assert_eq!(ram[ARCHERY_GAME_ARROWS_LEFT], 4);
        assert_eq!(ram[ARCHERY_GAME_OUT_OF_ARROWS], 0);
    }

    #[test]
    fn sprite_battle_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[NUM_SPRITES_KILLED] = 3;
        ram[TIMES_HURT_BY_SPRITES] = 4;
        ram[ITEM_DROP_LUCK] = 5;
        ram[LUCK_KILL_COUNTER] = 6;
        ram[ITEM_DROP_COUNTER] = 7;
        ram[DAMAGE_TYPE_DETERMINER] = 8;
        ram[SET_WHEN_DAMAGING_ENEMIES] = 9;

        let battle = SpriteBattleState::load_from_ram(&ram);
        assert_eq!(battle.sprites_killed(), 3);
        assert_eq!(battle.times_hurt_by_sprites(), 4);
        assert_eq!(battle.item_drop_luck(), 5);
        assert_eq!(battle.luck_kill_counter(), 6);
        assert_eq!(battle.item_drop_counter(), 7);
        assert_eq!(battle.damage_type_determiner(), 8);
        assert_eq!(battle.damaging_enemies_timer(), 9);

        let mut projected = vec![0; WRAM_SIZE];
        battle.write_to_ram(&mut projected);
        assert_eq!(SpriteBattleState::load_from_ram(&projected), battle);
    }

    #[test]
    fn native_sprite_battle_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[NUM_SPRITES_KILLED] = 0xff;
        ram[TIMES_HURT_BY_SPRITES] = 0xff;
        ram[LUCK_KILL_COUNTER] = 0xff;
        ram[ITEM_DROP_COUNTER] = 0xff;
        ram[SET_WHEN_DAMAGING_ENEMIES] = 0x81;

        let mut battle = SpriteBattleState::default();
        {
            let mut bridge = NativeSpriteBattleBridgeMut::new(&mut battle, &mut ram);
            bridge.clear_sprites_killed();
            bridge.increment_sprites_killed();
            bridge.clear_times_hurt_by_sprites();
            bridge.increment_times_hurt_by_sprites();
            bridge.set_item_drop_luck(2);
            bridge.clear_luck_kill_counter();
            bridge.increment_luck_kill_counter();
            bridge.clear_item_drop_counter();
            bridge.increment_item_drop_counter();
            bridge.set_damage_type_determiner(10);
            bridge.set_damaging_enemies_timer(2);
            bridge.tick_damaging_enemies_timer();
            bridge.clear_damaging_enemies_timer();
        }

        assert_eq!(battle.sprites_killed(), 1);
        assert_eq!(battle.times_hurt_by_sprites(), 1);
        assert_eq!(battle.item_drop_luck(), 2);
        assert_eq!(battle.luck_kill_counter(), 1);
        assert_eq!(battle.item_drop_counter(), 1);
        assert_eq!(battle.damage_type_determiner(), 10);
        assert_eq!(battle.damaging_enemies_timer(), 0);
        assert_eq!(ram[NUM_SPRITES_KILLED], 1);
        assert_eq!(ram[TIMES_HURT_BY_SPRITES], 1);
        assert_eq!(ram[ITEM_DROP_LUCK], 2);
        assert_eq!(ram[LUCK_KILL_COUNTER], 1);
        assert_eq!(ram[ITEM_DROP_COUNTER], 1);
        assert_eq!(ram[DAMAGE_TYPE_DETERMINER], 10);
        assert_eq!(ram[SET_WHEN_DAMAGING_ENEMIES], 0);
    }

    #[test]
    fn special_exit_position_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, LINK_X_COORD_SPEXIT, 0x0900);
        write_le_u16(&mut ram, LINK_Y_COORD_SPEXIT, 0x0500);

        let mut position = SpecialExitPositionState::load_from_ram(&ram);
        assert_eq!(position.x(), 0x0900);
        assert_eq!(position.y(), 0x0500);
        assert_eq!(position.map_zoom_x_offset(), 0x0010);
        assert_eq!(position.map_zoom_y(), 0x0008);

        position = SpecialExitPositionState::load_from_ram(&[0]);
        let mut projected = vec![0; WRAM_SIZE];
        position.write_to_ram(&mut projected);
        assert_eq!(read_le_u16(&projected, LINK_X_COORD_SPEXIT), 0);
        assert_eq!(read_le_u16(&projected, LINK_Y_COORD_SPEXIT), 0);
    }

    #[test]
    fn native_special_exit_position_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, LINK_X_COORD_SPEXIT, 0x0100);
        write_le_u16(&mut ram, LINK_Y_COORD_SPEXIT, 0x0200);
        write_le_u16(&mut ram, LINK_X_COORD, 0x0300);
        write_le_u16(&mut ram, LINK_Y_COORD, 0x0400);

        let mut position = SpecialExitPositionState::load_from_ram(&ram);
        {
            let mut bridge = NativeSpecialExitPositionBridgeMut::new(&mut position, &mut ram);
            bridge.set_x(0x0500);
            bridge.set_y(0x0600);
            bridge.offset_position(0x0010, 0x0020);
            bridge.store_from_player();
            bridge.set_position(0x0700, 0x0800);
            bridge.restore_player_position();
        }

        assert_eq!(position.x(), 0x0700);
        assert_eq!(position.y(), 0x0800);
        assert_eq!(read_le_u16(&ram, LINK_X_COORD_SPEXIT), 0x0700);
        assert_eq!(read_le_u16(&ram, LINK_Y_COORD_SPEXIT), 0x0800);
        assert_eq!(read_le_u16(&ram, LINK_X_COORD), 0x0700);
        assert_eq!(read_le_u16(&ram, LINK_Y_COORD), 0x0800);
    }

    #[test]
    fn swim_acceleration_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, SWIM_ACCELERATION_MODE, 1);
        write_le_u16(&mut ram, SWIM_ACCELERATION_MODE + 2, 2);
        write_le_u16(&mut ram, SWIM_SPEED_ACTIVE_FLAG, 3);
        write_le_u16(&mut ram, SWIM_SPEED_ACTIVE_FLAG + 2, 4);
        write_le_u16(&mut ram, SWIM_MAX_SPEED, 0x0180);
        write_le_u16(&mut ram, SWIM_MAX_SPEED + 2, 0x0240);
        write_le_u16(&mut ram, SWIM_ACCELERATION_DIRECTION, 5);
        write_le_u16(&mut ram, SWIM_ACCELERATION_DIRECTION + 2, 6);
        write_le_u16(&mut ram, SWIM_ACCELERATION, 7);
        write_le_u16(&mut ram, SWIM_ACCELERATION + 2, 8);

        let swim = SwimAccelerationState::load_from_ram(&ram);
        assert_eq!(swim.mode(0), 1);
        assert_eq!(swim.mode(2), 2);
        assert_eq!(swim.mode(1), 0);
        assert_eq!(swim.mode_low(1), 2);
        assert_eq!(swim.speed_active_flag(0), 3);
        assert_eq!(swim.speed_active_flag(2), 4);
        assert_eq!(swim.max_speed(0), 0x0180);
        assert_eq!(swim.max_speed(2), 0x0240);
        assert_eq!(swim.acceleration_direction(0), 5);
        assert_eq!(swim.acceleration_direction(2), 6);
        assert_eq!(swim.acceleration(0), 7);
        assert_eq!(swim.acceleration(2), 8);
        assert!(swim.has_any_acceleration());

        let mut projected = vec![0; WRAM_SIZE];
        swim.write_to_ram(&mut projected);
        assert_eq!(SwimAccelerationState::load_from_ram(&projected), swim);
    }

    #[test]
    fn native_swim_acceleration_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, SWIM_ACCELERATION_MODE, 0xffff);
        write_le_u16(&mut ram, SWIM_ACCELERATION_MODE + 2, 0xffff);
        write_le_u16(&mut ram, SWIM_SPEED_ACTIVE_FLAG, 0xffff);
        write_le_u16(&mut ram, SWIM_MAX_SPEED, 0xffff);
        write_le_u16(&mut ram, SWIM_ACCELERATION_DIRECTION + 2, 0xffff);
        write_le_u16(&mut ram, SWIM_ACCELERATION, 0xffff);

        let mut swim = SwimAccelerationState::load_from_ram(&ram);
        {
            let mut bridge = NativeSwimAccelerationBridgeMut::new(&mut swim, &mut ram);
            bridge.set_mode(0, 1);
            bridge.set_mode(2, 2);
            bridge.clear_mode_low_axis();
            bridge.set_speed_active_flag(0, 3);
            bridge.set_max_speed_both_axes(0x0180);
            bridge.set_max_speed(2, 0x0240);
            bridge.set_acceleration_direction(2, 4);
            bridge.set_acceleration(0, 5);
            bridge.set_acceleration(2, 6);
            bridge.clear_axis_motion(0);
            bridge.set_mode(1, 9);
        }

        assert_eq!(swim.mode(0), 0);
        assert_eq!(swim.mode(2), 2);
        assert_eq!(swim.speed_active_flag(0), 0);
        assert_eq!(swim.max_speed(0), 0);
        assert_eq!(swim.max_speed(2), 0x0240);
        assert_eq!(swim.acceleration_direction(2), 4);
        assert_eq!(swim.acceleration(0), 0);
        assert_eq!(swim.acceleration(2), 6);
        assert_eq!(read_le_u16(&ram, SWIM_ACCELERATION_MODE), 0);
        assert_eq!(read_le_u16(&ram, SWIM_ACCELERATION_MODE + 2), 2);
        assert_eq!(read_le_u16(&ram, SWIM_SPEED_ACTIVE_FLAG), 0);
        assert_eq!(read_le_u16(&ram, SWIM_MAX_SPEED), 0);
        assert_eq!(read_le_u16(&ram, SWIM_MAX_SPEED + 2), 0x0240);
        assert_eq!(read_le_u16(&ram, SWIM_ACCELERATION_DIRECTION + 2), 4);
        assert_eq!(read_le_u16(&ram, SWIM_ACCELERATION), 0);
        assert_eq!(read_le_u16(&ram, SWIM_ACCELERATION + 2), 6);
    }

    #[test]
    fn pushed_block_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[PUSHEDBLOCKS_X_LO] = 0x34;
        ram[PUSHEDBLOCKS_X_HI] = 0x12;
        ram[PUSHEDBLOCKS_Y_LO + 2] = 0x78;
        ram[PUSHEDBLOCKS_Y_HI + 2] = 0x56;
        ram[PUSHEDBLOCKS_SUBPIXEL + 2] = 0x9a;
        ram[PUSHEDBLOCKS_TARGET + 2] = 0x0b;
        ram[PUSHEDBLOCK_FACING_PLAYER + 2] = 4;
        ram[PUSHED_BLOCK_MODE] = 3;
        ram[PUSHED_BLOCK_ANIMATION_TIMER] = 7;
        ram[PUSH_BLOCK_DIRECTION] = 6;

        let pushed = PushedBlockState::load_from_ram(&ram);
        assert_eq!(pushed.x(0), 0x1234);
        assert_eq!(pushed.y(1), 0x5678);
        assert_eq!(pushed.y_fixed24(1), 0x56789a);
        assert_eq!(pushed.target_low(1), 0x0b);
        assert_eq!(pushed.facing_player(1), 4);
        assert_eq!(pushed.animation_mode(), 3);
        assert_eq!(pushed.animation_timer(), 7);
        assert_eq!(pushed.push_direction(), 6);
        assert_eq!(pushed.push_direction_index(), 3);
        assert_eq!(pushed.x(2), 0);

        let mut projected = vec![0; WRAM_SIZE];
        pushed.write_to_ram(&mut projected);
        assert_eq!(PushedBlockState::load_from_ram(&projected), pushed);
    }

    #[test]
    fn native_pushed_block_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[PUSHEDBLOCKS_X_LO + 1] = 0xff;
        ram[PUSHEDBLOCKS_X_HI + 1] = 0xff;
        ram[PUSHEDBLOCKS_Y_LO + 1] = 0xff;
        ram[PUSHEDBLOCKS_Y_HI + 1] = 0xff;
        ram[PUSHEDBLOCKS_TARGET + 1] = 0xff;
        ram[PUSHEDBLOCKS_SUBPIXEL + 1] = 0xff;

        let mut pushed = PushedBlockState::load_from_ram(&ram);
        {
            let mut bridge = NativePushedBlockBridgeMut::new(&mut pushed, &mut ram);
            bridge.init_slot(0, 0x1234, 0x5678);
            bridge.set_facing_player(1, 4);
            bridge.set_target_low(1, 0x0b);
            bridge.set_push_direction(6);
            bridge.set_animation_mode(2);
            bridge.reset_animation_timer();
            assert_eq!(bridge.decrement_animation_timer(), 8);
            assert_eq!(bridge.advance_animation_mode(), 3);
            bridge.set_x_fixed24(1, 0x00abcdu32);
            bridge.set_y_fixed24(1, 0x001234u32);
            bridge.set_target_low(4, 0xff);
        }

        assert_eq!(pushed.x(0), 0x1234);
        assert_eq!(pushed.y(0), 0x5678);
        assert_eq!(pushed.x_fixed24(1), 0x00ab34);
        assert_eq!(pushed.y_fixed24(1), 0x001234);
        assert_eq!(pushed.target_low(1), 0x0b);
        assert_eq!(pushed.facing_player(1), 4);
        assert_eq!(pushed.push_direction_index(), 3);
        assert_eq!(pushed.animation_mode(), 3);
        assert_eq!(pushed.animation_timer(), 9);
        assert_eq!(read_le_u16(&ram, PUSHEDBLOCKS_X_LO), 0x0034);
        assert_eq!(read_le_u16(&ram, PUSHEDBLOCKS_X_HI), 0x0012);
        assert_eq!(read_le_u16(&ram, PUSHEDBLOCKS_Y_LO), 0x0078);
        assert_eq!(read_le_u16(&ram, PUSHEDBLOCKS_Y_HI), 0x0056);
        assert_eq!(ram[PUSHEDBLOCKS_X_LO + 2], 0xab);
        assert_eq!(ram[PUSHEDBLOCKS_X_HI + 2], 0);
        assert_eq!(ram[PUSHEDBLOCKS_SUBPIXEL + 2], 0x34);
        assert_eq!(ram[PUSH_BLOCK_DIRECTION], 6);
        assert_eq!(ram[PUSHED_BLOCK_MODE], 3);
        assert_eq!(ram[PUSHED_BLOCK_ANIMATION_TIMER], 9);
    }

    #[test]
    fn maze_game_timer_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MAZE_GAME_TIMER_LO, 0x0012);
        write_le_u16(&mut ram, MAZE_GAME_TIMER_HI, 0x0034);
        write_le_u16(&mut ram, MAZE_GAME_TIMER_SNAPSHOT_LO, 0x0056);
        write_le_u16(&mut ram, MAZE_GAME_TIMER_SNAPSHOT_HI, 0x0078);

        let timer = MazeGameTimerState::load_from_ram(&ram);
        assert_eq!(timer.elapsed_low(), 0x0012);
        assert_eq!(timer.elapsed_high(), 0x0034);
        assert_eq!(timer.snapshot_low(), 0x0056);

        let mut projected = vec![0; WRAM_SIZE];
        timer.write_to_ram(&mut projected);
        assert_eq!(MazeGameTimerState::load_from_ram(&projected), timer);
    }

    #[test]
    fn native_maze_game_timer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MAZE_GAME_TIMER_LO, 0xffff);
        write_le_u16(&mut ram, MAZE_GAME_TIMER_HI, 0xffff);
        write_le_u16(&mut ram, MAZE_GAME_TIMER_SNAPSHOT_LO, 0xffff);
        write_le_u16(&mut ram, MAZE_GAME_TIMER_SNAPSHOT_HI, 0xffff);

        let mut timer = MazeGameTimerState::default();
        {
            let mut bridge = NativeMazeGameTimerBridgeMut::new(&mut timer, &mut ram);
            bridge.clear_elapsed();
            assert_eq!(bridge.increment_elapsed_low(), 1);
            assert_eq!(bridge.increment_elapsed_high(), 1);
            bridge.capture_snapshot();
        }

        assert_eq!(timer.elapsed_low(), 1);
        assert_eq!(timer.elapsed_high(), 1);
        assert_eq!(timer.snapshot_low(), 1);
        assert_eq!(read_le_u16(&ram, MAZE_GAME_TIMER_LO), 1);
        assert_eq!(read_le_u16(&ram, MAZE_GAME_TIMER_HI), 1);
        assert_eq!(read_le_u16(&ram, MAZE_GAME_TIMER_SNAPSHOT_LO), 1);
        assert_eq!(read_le_u16(&ram, MAZE_GAME_TIMER_SNAPSHOT_HI), 1);
    }

    #[test]
    fn prize_drop_cycle_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[PRIZE_DROP_CYCLE] = 2;
        ram[PRIZE_DROP_CYCLE + 15] = 7;

        let cycle = PrizeDropCycleState::load_from_ram(&ram);
        assert_eq!(cycle.next_index_for_slot(0), 2);
        assert_eq!(cycle.next_index_for_slot(15), 7);
        assert_eq!(cycle.next_index_for_slot(16), 0);

        let mut projected = vec![0; WRAM_SIZE];
        cycle.write_to_ram(&mut projected);
        assert_eq!(PrizeDropCycleState::load_from_ram(&projected), cycle);
        assert_eq!(projected[PRIZE_DROP_CYCLE], 2);
        assert_eq!(projected[PRIZE_DROP_CYCLE + 15], 7);
    }

    #[test]
    fn native_prize_drop_cycle_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[PRIZE_DROP_CYCLE + 3] = 7;

        let mut cycle = PrizeDropCycleState::default();
        {
            let mut bridge = NativePrizeDropCycleBridgeMut::new(&mut cycle, &mut ram);
            assert_eq!(bridge.take_next_index(3), 7);
            assert_eq!(bridge.take_next_index(3), 0);
            assert_eq!(bridge.take_next_index(18), 0);
        }

        assert_eq!(cycle.next_index_for_slot(3), 1);
        assert_eq!(ram[PRIZE_DROP_CYCLE + 3], 1);
    }

    #[test]
    fn dual_layer_tile_cache_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DUAL_LAYER_TILE_CACHE] = 0x1c;
        ram[DUAL_LAYER_TILE_CACHE + 15] = 0x2a;

        let cache = DualLayerTileCacheState::load_from_ram(&ram);
        assert_eq!(cache.tile_type(0), 0x1c);
        assert_eq!(cache.tile_type(15), 0x2a);
        assert_eq!(cache.tile_type(16), 0);

        let mut projected = vec![0; WRAM_SIZE];
        cache.write_to_ram(&mut projected);
        assert_eq!(DualLayerTileCacheState::load_from_ram(&projected), cache);
        assert_eq!(projected[DUAL_LAYER_TILE_CACHE], 0x1c);
        assert_eq!(projected[DUAL_LAYER_TILE_CACHE + 15], 0x2a);
    }

    #[test]
    fn native_dual_layer_tile_cache_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DUAL_LAYER_TILE_CACHE + 4] = 0x1c;

        let mut cache = DualLayerTileCacheState::default();
        {
            let mut bridge = NativeDualLayerTileCacheBridgeMut::new(&mut cache, &mut ram);
            bridge.set_tile_type(4, 0x2a);
            bridge.set_tile_type(18, 0x7f);
        }

        assert_eq!(cache.tile_type(4), 0x2a);
        assert_eq!(cache.tile_type(18), 0);
        assert_eq!(ram[DUAL_LAYER_TILE_CACHE + 4], 0x2a);
    }

    #[test]
    fn tagalong_trail_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TAGALONG_X_LO + 3] = 0x34;
        ram[TAGALONG_X_HI + 3] = 0x12;
        ram[TAGALONG_Y_LO + 3] = 0x78;
        ram[TAGALONG_Y_HI + 3] = 0x56;
        ram[TAGALONG_Z + 3] = 0xf0;
        ram[TAGALONG_LAYERBITS + 3] = 0x23;

        let trail = TagalongTrailState::load_from_ram(&ram);
        assert_eq!(trail.x(3), 0x1234);
        assert_eq!(trail.y(3), 0x5678);
        assert_eq!(trail.z(3), 0xf0);
        assert_eq!(trail.layer_bits(3), 0x23);
        assert_eq!(trail.x(20), 0);

        let mut projected = vec![0; WRAM_SIZE];
        trail.write_to_ram(&mut projected);
        assert_eq!(TagalongTrailState::load_from_ram(&projected), trail);
    }

    #[test]
    fn native_tagalong_slot_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TAGALONG_X_LO + 2] = 1;
        ram[TAGALONG_X_HI + 2] = 2;

        let mut trail = TagalongTrailState::default();
        {
            let mut slot = NativeTagalongSlotBridgeMut::new(&mut trail, &mut ram, 2);
            slot.set_position(0x1234, 0x5678);
            slot.set_y_high(0x9a);
            slot.set_z(0xf8);
            slot.set_layer_bits(0x23);
        }

        assert_eq!(trail.x(2), 0x1234);
        assert_eq!(trail.y(2), 0x9a78);
        assert_eq!(trail.z(2), 0xf8);
        assert_eq!(trail.layer_bits(2), 0x23);
        assert_eq!(ram[TAGALONG_X_LO + 2], 0x34);
        assert_eq!(ram[TAGALONG_X_HI + 2], 0x12);
        assert_eq!(ram[TAGALONG_Y_LO + 2], 0x78);
        assert_eq!(ram[TAGALONG_Y_HI + 2], 0x9a);
        assert_eq!(ram[TAGALONG_Z + 2], 0xf8);
        assert_eq!(ram[TAGALONG_LAYERBITS + 2], 0x23);

        {
            let mut out_of_range = NativeTagalongSlotBridgeMut::new(&mut trail, &mut ram, 20);
            out_of_range.set_position(0xffff, 0xffff);
            out_of_range.set_z(0xff);
        }

        assert_eq!(trail.x(20), 0);
        assert_eq!(trail.z(20), 0);
    }

    #[test]
    fn chain_chomp_history_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, CHAIN_CHOMP_HISTORY_X + 4, 0x1234);
        write_le_u16(&mut ram, CHAIN_CHOMP_HISTORY_Y + 4, 0x5678);
        write_le_u16(&mut ram, CHAIN_CHOMP_HISTORY_X + 0xfe, 0x9abc);
        write_le_u16(&mut ram, CHAIN_CHOMP_HISTORY_Y + 0xfe, 0xdef0);

        let mut history = ChainChompHistoryState::load_from_ram(&ram);
        assert_eq!(history.x(2), 0x1234);
        assert_eq!(history.y(2), 0x5678);
        assert_eq!(history.x(0x7f), 0x9abc);
        assert_eq!(history.y(0x7f), 0xdef0);
        assert_eq!(history.x(0x80), 0);
        history.set_x(2, 0x1111);
        history.set_y(2, 0x2222);
        history.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_X + 4), 0x1111);
        assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_Y + 4), 0x2222);
        assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_X + 0xfe), 0x9abc);
        assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_Y + 0xfe), 0xdef0);
    }

    #[test]
    fn native_chain_chomp_history_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, CHAIN_CHOMP_HISTORY_X, 0x1234);
        write_le_u16(&mut ram, CHAIN_CHOMP_HISTORY_Y, 0x5678);

        let mut history = ChainChompHistoryState::default();
        {
            let mut bridge = NativeChainChompHistoryBridgeMut::new(&mut history, &mut ram);
            bridge.set_x(0, 0x1111);
            bridge.set_y(0, 0x2222);
            bridge.set_x(0x80, 0xffff);
            bridge.set_y(0x80, 0xffff);
        }

        assert_eq!(history.x(0), 0x1111);
        assert_eq!(history.y(0), 0x2222);
        assert_eq!(history.x(0x80), 0);
        assert_eq!(history.y(0x80), 0);
        assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_X), 0x1111);
        assert_eq!(read_le_u16(&ram, CHAIN_CHOMP_HISTORY_Y), 0x2222);
    }

    #[test]
    fn ether_orbit_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[ETHER_ANGLE + 2] = 0x3f;
        ram[ETHER_RADIUS] = 0x20;
        write_le_u16(&mut ram, ETHER_BEAM_Y, 0x1234);
        write_le_u16(&mut ram, ETHER_BEAM_TOP_BUCKET, 0xabcd);
        write_le_u16(&mut ram, ETHER_ORBIT_X, 0x4567);
        write_le_u16(&mut ram, ETHER_ORBIT_Y, 0x89ab);
        ram[ETHER_SPIN_COUNTDOWN] = 1;
        write_le_u16(&mut ram, ETHER_ORB_X, 0xdef0);
        write_le_u16(&mut ram, ETHER_ORB_Y, 0x1357);

        let mut orbit = EtherOrbitState::load_from_ram(&ram);
        assert_eq!(orbit.angle(2), 0x3f);
        assert_eq!(orbit.radius(), 0x20);
        assert_eq!(orbit.beam_y(), 0x1234);
        assert_eq!(orbit.beam_top_bucket(), 0xcd);
        assert_eq!(orbit.orbit_x(), 0x4567);
        assert_eq!(orbit.swordbeam_temp_y(), 0x89ab);
        assert_eq!(orbit.orb_x(), 0xdef0);
        assert_eq!(orbit.orb_y(), 0x1357);
        orbit.advance_angle(2);
        orbit.set_beam_top_bucket(0x55);
        orbit.set_swordbeam_temp(0x1111, 0x2222);
        orbit.write_to_ram(&mut ram);

        assert_eq!(ram[ETHER_ANGLE + 2], 0);
        assert_eq!(read_le_u16(&ram, ETHER_BEAM_TOP_BUCKET), 0xab55);
        assert_eq!(read_le_u16(&ram, ETHER_ORBIT_X), 0x1111);
        assert_eq!(read_le_u16(&ram, ETHER_ORBIT_Y), 0x2222);
    }

    #[test]
    fn native_ether_orbit_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, ETHER_BEAM_TOP_BUCKET, 0x1200);
        ram[ETHER_SPIN_COUNTDOWN] = 0;

        let mut orbit = EtherOrbitState::default();
        {
            let mut bridge = NativeEtherOrbitBridgeMut::new(&mut orbit, &mut ram);
            bridge.set_angle(0, 0x3f);
            assert_eq!(bridge.advance_angle(0), 0);
            bridge.set_radius(0x40);
            assert_eq!(bridge.tick_spin_countdown(), 0xff);
            bridge.set_spin_countdown(3);
            bridge.set_beam_top_bucket(0x34);
            bridge.initialize_beam_adjusted_y(0x5678);
            bridge.set_beam_y(0x9abc);
            bridge.set_orbit_position(0x1111, 0x2222);
            bridge.set_orb_position(0x3333, 0x4444);
        }

        assert_eq!(orbit.angle(0), 0);
        assert_eq!(orbit.radius(), 0x40);
        assert_eq!(orbit.beam_top_bucket(), 0x78);
        assert_eq!(orbit.beam_y(), 0x9abc);
        assert_eq!(orbit.orbit_x(), 0x1111);
        assert_eq!(orbit.orb_y(), 0x4444);
        assert_eq!(ram[ETHER_ANGLE], 0);
        assert_eq!(ram[ETHER_RADIUS], 0x40);
        assert_eq!(ram[ETHER_SPIN_COUNTDOWN], 3);
        assert_eq!(read_le_u16(&ram, ETHER_BEAM_TOP_BUCKET), 0x5678);
        assert_eq!(read_le_u16(&ram, ETHER_BEAM_Y), 0x9abc);
        assert_eq!(read_le_u16(&ram, ETHER_ORBIT_X), 0x1111);
        assert_eq!(read_le_u16(&ram, ETHER_ORBIT_Y), 0x2222);
        assert_eq!(read_le_u16(&ram, ETHER_ORB_X), 0x3333);
        assert_eq!(read_le_u16(&ram, ETHER_ORB_Y), 0x4444);
    }

    #[test]
    fn enemy_damage_subclass_table_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[ENEMY_DAMAGE_DATA] = 3;
        ram[ENEMY_DAMAGE_DATA + 0x918] = 2;
        ram[ENEMY_DAMAGE_DATA + 0x0fff] = 7;

        let table = EnemyDamageSubclassTableState::load_from_ram(&ram);
        assert_eq!(table.entry(0), 3);
        assert_eq!(table.entry(0x918), 2);
        assert_eq!(table.entry(0x0fff), 7);
        assert_eq!(table.entry(0x1000), 0);

        let mut projected = vec![0; WRAM_SIZE];
        table.write_to_ram(&mut projected);
        assert_eq!(
            EnemyDamageSubclassTableState::load_from_ram(&projected),
            table
        );
    }

    #[test]
    fn native_enemy_damage_subclass_table_loads_packed_nibbles_and_dual_writes_overrides() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[ENEMY_DAMAGE_DATA + 0x918] = 9;

        let packed = vec![0xab, 0xcd, 0xef];
        let mut table = EnemyDamageSubclassTableState::default();
        {
            let mut bridge = NativeEnemyDamageSubclassTableBridgeMut::new(&mut table, &mut ram);
            bridge.load_from_packed_nibbles(&packed);
            bridge.set_entry(0x918, 2);
            bridge.set_entry(0x1000, 7);
        }

        assert_eq!(table.entry(0), 0x0a);
        assert_eq!(table.entry(1), 0x0b);
        assert_eq!(table.entry(2), 0x0c);
        assert_eq!(table.entry(3), 0x0d);
        assert_eq!(table.entry(4), 0x0e);
        assert_eq!(table.entry(5), 0x0f);
        assert_eq!(table.entry(6), 0);
        assert_eq!(table.entry(0x918), 2);
        assert_eq!(table.entry(0x1000), 0);
        assert_eq!(ram[ENEMY_DAMAGE_DATA], 0x0a);
        assert_eq!(ram[ENEMY_DAMAGE_DATA + 1], 0x0b);
        assert_eq!(ram[ENEMY_DAMAGE_DATA + 0x918], 2);
    }

    #[test]
    fn sprite_draw_hitbox_work_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DRAW_WORK_POSITION_X] = 0x34;
        ram[DRAW_WORK_POSITION_Y] = 0x12;
        ram[HITBOX_WORK_Y_OFFSET] = 0xfc;
        ram[DRAW_WORK_FLAGS_HI] = 0x80;

        let work = SpriteDrawHitboxWorkState::load_from_ram(&ram);
        assert_eq!(work.x_low(), 0x34);
        assert_eq!(work.y_low(), 0x12);
        assert_eq!(work.low_position_word(), 0x1234);
        assert_eq!(work.hitbox_y_low_offset(), 0xfc);
        assert_eq!(work.hitbox_x_high_offset(), 0x80);

        let mut projected = vec![0; WRAM_SIZE];
        work.write_to_ram(&mut projected);
        assert_eq!(SpriteDrawHitboxWorkState::load_from_ram(&projected), work);
        assert_eq!(projected[DRAW_WORK_FLAGS_HI], 0x80);
        assert_eq!(projected[HITBOX_WORK_X_OFFSET], 0x80);
    }

    #[test]
    fn native_sprite_draw_hitbox_work_bridges_share_flags_offset_byte() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DRAW_WORK_POSITION_X] = 0x10;
        ram[DRAW_WORK_POSITION_Y] = 0x20;
        ram[HITBOX_WORK_Y_OFFSET] = 0x30;
        ram[HITBOX_WORK_X_OFFSET] = 0x40;

        let mut work = SpriteDrawHitboxWorkState::default();
        {
            let mut draw = NativeSpriteDrawWorkPositionBridgeMut::new(&mut work, &mut ram);
            draw.set_low_position_word(0x9abc);
            draw.offset_low_position(1, 2);
            draw.set_flags_high(0x7f);
        }

        assert_eq!(work.low_position_word(), 0x9cbd);
        assert_eq!(work.hitbox_x_high_offset(), 0x7f);
        assert_eq!(ram[DRAW_WORK_POSITION_X], 0xbd);
        assert_eq!(ram[DRAW_WORK_POSITION_Y], 0x9c);
        assert_eq!(ram[DRAW_WORK_FLAGS_HI], 0x7f);

        {
            let mut hitbox = NativeSpriteHitboxWorkOffsetBridgeMut::new(&mut work, &mut ram);
            hitbox.set_offsets(0xfc, 0x08);
        }

        assert_eq!(work.hitbox_y_low_offset(), 0xfc);
        assert_eq!(work.hitbox_x_high_offset(), 0x08);
        assert_eq!(ram[HITBOX_WORK_Y_OFFSET], 0xfc);
        assert_eq!(ram[HITBOX_WORK_X_OFFSET], 0x08);
        assert_eq!(ram[DRAW_WORK_FLAGS_HI], 0x08);
    }

    #[test]
    fn effect_angle_scratch_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        for slot in 0..9 {
            ram[EFFECT_ANGLE_WORK + slot] = slot as u8;
        }

        let mut angles = EffectAngleScratchState::load_from_ram(&ram);
        assert_eq!(angles.angle(2), 2);
        assert_eq!(angles.trailing_angle(), 4);
        assert_eq!(angles.radial_radius(), 8);

        angles.set_angles4(&[10, 20, 30, 40], 0);
        assert_eq!(angles.add_angle_mod64(1, 50), 6);
        assert_eq!(angles.add_trailing_angle_mod64(63), 3);
        angles.set_radial_radius(14);
        angles.write_to_ram(&mut ram);

        assert_eq!(ram[EFFECT_ANGLE_WORK], 10);
        assert_eq!(ram[EFFECT_ANGLE_WORK + 1], 6);
        assert_eq!(ram[EFFECT_ANGLE_WORK + 4], 3);
        assert_eq!(ram[EFFECT_ANGLE_WORK + 8], 14);
    }

    #[test]
    fn native_effect_angle_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[EFFECT_ANGLE_WORK + 1] = 60;
        ram[EFFECT_ANGLE_WORK + 4] = 2;
        ram[EFFECT_ANGLE_WORK + 8] = 9;

        let mut angles = EffectAngleScratchState::default();
        {
            let mut bridge = NativeEffectAngleScratchBridgeMut::new(&mut angles, &mut ram);
            bridge.set_angle(0, 12);
            bridge.set_angles4(&[1, 2, 3, 4, 5], 1);
            assert_eq!(bridge.add_angle_mod64(1, 63), 2);
            assert_eq!(bridge.add_trailing_angle_mod64(10), 12);
            bridge.set_radial_radius(20);
        }

        assert_eq!(angles.angle(0), 2);
        assert_eq!(angles.angle(1), 2);
        assert_eq!(angles.trailing_angle(), 12);
        assert_eq!(angles.radial_radius(), 20);
        assert_eq!(ram[EFFECT_ANGLE_WORK], 2);
        assert_eq!(ram[EFFECT_ANGLE_WORK + 1], 2);
        assert_eq!(ram[EFFECT_ANGLE_WORK + 4], 12);
        assert_eq!(ram[EFFECT_ANGLE_WORK + 8], 20);
    }

    #[test]
    fn quake_spell_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[QUAKE_ACTIVE_BOLT_LIMIT] = 4;
        ram[QUAKE_PENDING_STEP] = 1;
        write_le_u16(&mut ram, QUAKE_ORIGIN_X, 0x1234);
        write_le_u16(&mut ram, QUAKE_ORIGIN_Y, 0x5678);
        write_le_u16(&mut ram, QUAKE_SCREEN_SHAKE_Y, 3);

        let mut quake = QuakeSpellState::load_from_ram(&ram);
        assert_eq!(quake.active_bolt_limit(), 4);
        assert_eq!(quake.pending_step(), 1);
        assert_eq!(quake.origin_x(), 0x1234);
        assert_eq!(quake.origin_y(), 0x5678);
        assert_eq!(quake.screen_shake_y(), 3);
        assert_eq!(quake.invert_screen_shake_y(), 3);
        quake.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, QUAKE_SCREEN_SHAKE_Y), 0xfffd);
    }

    #[test]
    fn native_quake_spell_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[QUAKE_ACTIVE_BOLT_LIMIT] = 4;
        ram[QUAKE_PENDING_STEP] = 1;
        write_le_u16(&mut ram, QUAKE_SCREEN_SHAKE_Y, 5);

        let mut quake = QuakeSpellState::default();
        {
            let mut bridge = NativeQuakeSpellBridgeMut::new(&mut quake, &mut ram);
            bridge.set_active_bolt_limit(2);
            bridge.set_pending_step(3);
            bridge.set_origin(0x4567, 0x89ab);
            assert_eq!(bridge.invert_screen_shake_y(), 5);
            bridge.set_screen_shake_y(9);
        }

        assert_eq!(quake.active_bolt_limit(), 2);
        assert_eq!(quake.pending_step(), 3);
        assert_eq!(quake.origin_x(), 0x4567);
        assert_eq!(quake.origin_y(), 0x89ab);
        assert_eq!(quake.screen_shake_y(), 9);
        assert_eq!(ram[QUAKE_ACTIVE_BOLT_LIMIT], 2);
        assert_eq!(ram[QUAKE_PENDING_STEP], 3);
        assert_eq!(read_le_u16(&ram, QUAKE_ORIGIN_X), 0x4567);
        assert_eq!(read_le_u16(&ram, QUAKE_ORIGIN_Y), 0x89ab);
        assert_eq!(read_le_u16(&ram, QUAKE_SCREEN_SHAKE_Y), 9);
    }

    #[test]
    fn native_quake_bolt_bridge_syncs_seeded_ram_and_dual_writes_slot_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[QUAKE_BOLT_TIMER + 2] = 7;
        ram[QUAKE_BOLT_PHASE + 2] = 0xfe;

        let mut bolts = QuakeBoltState::default();
        {
            let mut bridge = NativeQuakeBoltBridgeMut::new(&mut bolts, &mut ram, 2);
            assert_eq!(bridge.tick_timer(), 6);
            assert_eq!(bridge.advance_phase(), 0xff);
            bridge.set_timer(1);
            bridge.set_phase(0x10);
        }

        assert_eq!(bolts.slot(2).timer(), 1);
        assert_eq!(bolts.slot(2).phase(), 0x10);
        assert_eq!(ram[QUAKE_BOLT_TIMER + 2], 1);
        assert_eq!(ram[QUAKE_BOLT_PHASE + 2], 0x10);
    }

    #[test]
    fn bombos_spell_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[BOMBOS_MODE] = 2;
        ram[BOMBOS_FIRE_COLUMN_RADIUS] = 16;
        ram[BOMBOS_BLAST_RELEASE_LOCKED] = 1;
        ram[BOMBOS_BLAST_RELEASE_COUNTDOWN] = 0x80;
        write_le_u16(&mut ram, BOMBOS_FIRE_COLUMN_SEED_X, 0x1234);
        write_le_u16(&mut ram, BOMBOS_FIRE_COLUMN_SEED_Y, 0x5678);
        write_le_u16(&mut ram, BOMBOS_BLAST_X + 4, 0x9abc);
        write_le_u16(&mut ram, BOMBOS_BLAST_Y + 4, 0xdef0);

        let mut bombos = BombosSpellState::load_from_ram(&ram);
        assert_eq!(bombos.mode(), 2);
        assert_eq!(bombos.fire_column_radius(), 16);
        assert!(bombos.blast_release_locked());
        assert_eq!(bombos.fire_column_seed_x(0), 0x1234);
        assert_eq!(bombos.fire_column_seed_y(0), 0x5678);
        assert_eq!(bombos.blast_x(2), 0x9abc);
        assert_eq!(bombos.blast_y(2), 0xdef0);

        bombos.set_mode(1);
        assert_eq!(bombos.grow_fire_column_radius(200, 207), 207);
        bombos.set_blast_release_locked(false);
        assert_eq!(bombos.tick_blast_release_countdown(), 0x7f);
        bombos.set_fire_column_seed_position(1, 0x1111, 0x2222);
        bombos.set_blast_position(3, 0x3333, 0x4444);
        bombos.write_to_ram(&mut ram);

        assert_eq!(ram[BOMBOS_MODE], 1);
        assert_eq!(ram[BOMBOS_FIRE_COLUMN_RADIUS], 207);
        assert_eq!(ram[BOMBOS_BLAST_RELEASE_LOCKED], 0);
        assert_eq!(ram[BOMBOS_BLAST_RELEASE_COUNTDOWN], 0x7f);
        assert_eq!(read_le_u16(&ram, BOMBOS_FIRE_COLUMN_SEED_X + 2), 0x1111);
        assert_eq!(read_le_u16(&ram, BOMBOS_FIRE_COLUMN_SEED_Y + 2), 0x2222);
        assert_eq!(read_le_u16(&ram, BOMBOS_BLAST_X + 6), 0x3333);
        assert_eq!(read_le_u16(&ram, BOMBOS_BLAST_Y + 6), 0x4444);
    }

    #[test]
    fn native_bombos_spell_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[BOMBOS_FIRE_COLUMN_RADIUS] = 10;
        ram[BOMBOS_BLAST_RELEASE_COUNTDOWN] = 1;

        let mut bombos = BombosSpellState::default();
        {
            let mut bridge = NativeBombosSpellBridgeMut::new(&mut bombos, &mut ram);
            bridge.set_mode(2);
            assert_eq!(bridge.grow_fire_column_radius(5, 207), 15);
            bridge.set_blast_release_locked(true);
            assert_eq!(bridge.tick_blast_release_countdown(), 0);
            bridge.set_blast_release_countdown(4);
            bridge.set_fire_column_seed_position(0, 0x1234, 0x5678);
            bridge.set_blast_position(15, 0x9abc, 0xdef0);
        }

        assert_eq!(bombos.mode(), 2);
        assert_eq!(bombos.fire_column_radius(), 15);
        assert!(bombos.blast_release_locked());
        assert_eq!(bombos.fire_column_seed_x(0), 0x1234);
        assert_eq!(bombos.fire_column_seed_y(0), 0x5678);
        assert_eq!(bombos.blast_x(15), 0x9abc);
        assert_eq!(bombos.blast_y(15), 0xdef0);
        assert_eq!(ram[BOMBOS_MODE], 2);
        assert_eq!(ram[BOMBOS_FIRE_COLUMN_RADIUS], 15);
        assert_eq!(ram[BOMBOS_BLAST_RELEASE_LOCKED], 1);
        assert_eq!(ram[BOMBOS_BLAST_RELEASE_COUNTDOWN], 4);
        assert_eq!(read_le_u16(&ram, BOMBOS_FIRE_COLUMN_SEED_X), 0x1234);
        assert_eq!(read_le_u16(&ram, BOMBOS_FIRE_COLUMN_SEED_Y), 0x5678);
        assert_eq!(read_le_u16(&ram, BOMBOS_BLAST_X + 30), 0x9abc);
        assert_eq!(read_le_u16(&ram, BOMBOS_BLAST_Y + 30), 0xdef0);
    }

    #[test]
    fn native_bombos_slot_bridges_preserve_overlapping_fire_column_layout() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[BOMBOS_FIRE_COLUMN_TIMER + 3] = 1;
        ram[BOMBOS_FIRE_COLUMN_PHASE + 3] = 0xfe;
        ram[BOMBOS_BLAST_TIMER + 7] = 1;
        ram[BOMBOS_BLAST_PHASE + 7] = 0xfe;

        let mut bombos = BombosSpellState::default();
        {
            let mut column = NativeBombosFireColumnBridgeMut::new(&mut bombos, &mut ram, 3);
            assert_eq!(column.tick_timer(), 0);
            assert_eq!(column.advance_phase(), 0xff);
            column.set_position(0x1234, 0x56cc);
        }
        assert_eq!(bombos.fire_column(3).timer(), 0);
        assert_eq!(bombos.fire_column(3).phase(), 0xff);
        assert_eq!(bombos.fire_column(3).x(), 0x1234);
        assert_eq!(bombos.fire_column(3).y(), 0x56cc);

        {
            let mut column = NativeBombosFireColumnBridgeMut::new(&mut bombos, &mut ram, 7);
            column.set_radial_angle(0x77);
        }
        assert_eq!(bombos.fire_column(3).y(), 0x5677);
        assert_eq!(bombos.fire_column(7).radial_angle(), 0x77);

        {
            let mut blast = NativeBombosBlastBridgeMut::new(&mut bombos, &mut ram, 7);
            assert_eq!(blast.tick_timer(), 0);
            assert_eq!(blast.advance_phase(), 0xff);
        }
        assert_eq!(bombos.blast(7).phase(), 0xff);
        assert_eq!(ram[BOMBOS_FIRE_COLUMN_TIMER + 3], 0);
        assert_eq!(ram[BOMBOS_FIRE_COLUMN_PHASE + 3], 0xff);
        assert_eq!(ram[BOMBOS_FIRE_COLUMN_X_LO + 3], 0x34);
        assert_eq!(ram[BOMBOS_FIRE_COLUMN_X_HI + 3], 0x12);
        assert_eq!(ram[BOMBOS_FIRE_COLUMN_Y_LO + 3], 0x77);
        assert_eq!(ram[BOMBOS_FIRE_COLUMN_Y_HI + 3], 0x56);
        assert_eq!(ram[BOMBOS_FIRE_COLUMN_RADIAL_ANGLE + 7], 0x77);
        assert_eq!(ram[BOMBOS_BLAST_TIMER + 7], 0);
        assert_eq!(ram[BOMBOS_BLAST_PHASE + 7], 0xff);
    }

    #[test]
    fn native_happiness_pond_rupee_bridge_loads_and_stores_snapshots() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut effects = EffectState::load_from_ram(&ram);

        {
            let mut bridge = NativeHappinessPondRupeeBridgeMut::new(
                &mut effects.happiness_pond_rupees,
                &mut ram,
                4,
            );
            bridge.initialize(0x1234, 0x5678, 0x9a, 0xbc, 0xde);
        }

        let rupee = effects.happiness_pond_rupees.rupee(4);
        assert!(rupee.is_active());
        assert_eq!(rupee.step(), 0);
        let snapshot = rupee.snapshot();
        assert_eq!(snapshot.x_low, 0x34);
        assert_eq!(snapshot.x_high, 0x12);
        assert_eq!(snapshot.y_low, 0x78);
        assert_eq!(snapshot.y_high, 0x56);
        assert_eq!(snapshot.x_velocity, 0x9a);
        assert_eq!(snapshot.y_velocity, 0xbc);
        assert_eq!(snapshot.z_velocity, 0xde);
        assert_eq!(snapshot.item_to_link, 53);
        assert_eq!(snapshot.timer, 15);

        let stored = HappinessPondRupeeSnapshot {
            y_low: 1,
            y_high: 2,
            x_low: 3,
            x_high: 4,
            z: 5,
            y_velocity: 6,
            x_velocity: 7,
            z_velocity: 8,
            y_subpixel: 9,
            x_subpixel: 10,
            z_subpixel: 11,
            item_to_link: 12,
            timer: 13,
            step: 14,
        };
        {
            let mut bridge = NativeHappinessPondRupeeBridgeMut::new(
                &mut effects.happiness_pond_rupees,
                &mut ram,
                4,
            );
            bridge.store_snapshot(stored);
        }
        let expected_snapshot = HappinessPondRupeeSnapshot {
            timer: 12,
            ..stored
        };
        assert_eq!(
            effects.happiness_pond_rupees.rupee(4).snapshot(),
            expected_snapshot
        );
        assert_eq!(ram[HAPPINESS_POND_Y_LO + 4], 1);
        assert_eq!(ram[HAPPINESS_POND_Y_HI + 4], 2);
        assert_eq!(ram[HAPPINESS_POND_X_LO + 4], 3);
        assert_eq!(ram[HAPPINESS_POND_X_HI + 4], 4);
        assert_eq!(ram[HAPPINESS_POND_Z + 4], 5);
        assert_eq!(ram[HAPPINESS_POND_Y_VEL + 4], 6);
        assert_eq!(ram[HAPPINESS_POND_X_VEL + 4], 7);
        assert_eq!(ram[HAPPINESS_POND_Z_VEL + 4], 8);
        assert_eq!(ram[HAPPINESS_POND_Y_SUBPIXEL + 4], 9);
        assert_eq!(ram[HAPPINESS_POND_X_SUBPIXEL + 4], 10);
        assert_eq!(ram[HAPPINESS_POND_Z_SUBPIXEL + 4], 11);
        assert_eq!(ram[HAPPINESS_POND_ITEM_TO_LINK + 4], 12);
        assert_eq!(ram[HAPPINESS_POND_TIMER + 4], 13);
        assert_eq!(ram[HAPPINESS_POND_STEP + 4], 14);

        {
            let mut bridge = NativeHappinessPondRupeeBridgeMut::new(
                &mut effects.happiness_pond_rupees,
                &mut ram,
                4,
            );
            bridge.clear();
        }
        assert!(!effects.happiness_pond_rupees.rupee(4).is_active());
        assert_eq!(ram[HAPPINESS_POND_ACTIVE + 4], 0);
    }

    #[test]
    fn native_weather_vane_debris_bridge_updates_transient_slots() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut effects = EffectState::load_from_ram(&ram);

        {
            let mut bridge = NativeWeatherVaneDebrisBridgeMut::new(
                &mut effects.weather_vane_debris,
                &mut ram,
                3,
            );
            bridge.initialize(0x1234, 0x5678, 0x9a, 0xbc, 0xde, 0x21, 1);
        }
        let debris = effects.weather_vane_debris.debris(3).snapshot();
        assert_eq!(
            debris,
            effects::WeatherVaneDebrisSnapshot {
                y: 0x5678,
                x: 0x1234,
                z: 0x21,
                y_velocity: 0xbc,
                x_velocity: 0x9a,
                z_velocity: 0xde,
                draw_state: 1,
            }
        );
        assert_eq!(ram[WEATHERVANE_ANIM_TIMER + 3], 1);

        {
            let mut bridge = NativeWeatherVaneDebrisBridgeMut::new(
                &mut effects.weather_vane_debris,
                &mut ram,
                3,
            );
            assert_eq!(bridge.tick_animation(), 1);
            assert_eq!(bridge.tick_z_velocity(), 0xdd);
            bridge.mark_finished_if_landed(0xef);
        }
        assert!(!effects.weather_vane_debris.debris(3).is_finished());
        {
            let mut bridge = NativeWeatherVaneDebrisBridgeMut::new(
                &mut effects.weather_vane_debris,
                &mut ram,
                3,
            );
            bridge.mark_finished_if_landed(0xf0);
            bridge.save_position(0xabcd, 0xef01, 0x45);
        }
        let debris = effects.weather_vane_debris.debris(3);
        assert!(debris.is_finished());
        assert_eq!(debris.snapshot().x, 0xabcd);
        assert_eq!(debris.snapshot().y, 0xef01);
        assert_eq!(debris.snapshot().z, 0x45);
        assert_eq!(ram[WEATHERVANE_X_LO + 3], 0xcd);
        assert_eq!(ram[WEATHERVANE_X_HI + 3], 0xab);
        assert_eq!(ram[WEATHERVANE_Y_LO + 3], 0x01);
        assert_eq!(ram[WEATHERVANE_Y_HI + 3], 0xef);
        assert_eq!(ram[WEATHERVANE_Z + 3], 0x45);
        assert_eq!(ram[WEATHERVANE_Z_VELOCITY + 3], 0xdd);
        assert_eq!(ram[WEATHERVANE_DRAW_STATE + 3], 0xff);
    }

    #[test]
    fn native_sprite_history_bridges_update_position_and_motion_banks() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut effects = EffectState::load_from_ram(&ram);

        {
            let mut bridge =
                NativeMoldormHistoryBridgeMut::new(&mut effects.sprite_histories, &mut ram, 7);
            bridge.set_position(0x1234, 0x5678);
        }
        assert_eq!(effects.sprite_histories.moldorm_history(7).x(), 0x1234);
        assert_eq!(effects.sprite_histories.moldorm_history(7).y(), 0x5678);
        {
            let mut bridge =
                NativeMoldormHistoryBridgeMut::new(&mut effects.sprite_histories, &mut ram, 7);
            bridge.set_low_position(0xab, 0xcd);
        }
        assert_eq!(effects.sprite_histories.moldorm_history(7).x(), 0x12ab);
        assert_eq!(effects.sprite_histories.moldorm_history(7).y(), 0x56cd);
        assert_eq!(ram[MOLDORM_HISTORY_X_LO + 7], 0xab);
        assert_eq!(ram[MOLDORM_HISTORY_Y_LO + 7], 0xcd);

        {
            let mut bridge =
                NativeSwamolaTargetBridgeMut::new(&mut effects.sprite_histories, &mut ram, 2);
            bridge.set_position(0x2345, 0x6789);
            bridge.set_x_low(0xef);
            bridge.set_y_low(0x01);
        }
        assert_eq!(effects.sprite_histories.swamola_target(2).x(), 0x23ef);
        assert_eq!(effects.sprite_histories.swamola_target(2).y(), 0x6701);

        {
            let mut bridge =
                NativeSwamolaHistoryBridgeMut::new(&mut effects.sprite_histories, &mut ram, 0x40);
            bridge.set_position(0x3456, 0x789a);
        }
        assert_eq!(effects.sprite_histories.swamola_history(0x40).x(), 0x3456);
        assert_eq!(effects.sprite_histories.swamola_history(0x40).y(), 0x789a);

        {
            let mut bridge =
                NativeBeamosLaserHistoryBridgeMut::new(&mut effects.sprite_histories, &mut ram, 9);
            bridge.set_position(0x4567, 0x89ab);
        }
        assert_eq!(effects.sprite_histories.beamos_laser_history(9).x(), 0x4567);
        assert_eq!(effects.sprite_histories.beamos_laser_history(9).y(), 0x89ab);

        {
            let mut bridge = NativeLanmolaSegmentMotionBridgeMut::new(
                &mut effects.sprite_histories,
                &mut ram,
                9,
            );
            bridge.set_z_offset(0x55);
            bridge.set_direction(0xaa);
        }
        let segment = effects.sprite_histories.lanmola_segment_motion(9);
        assert_eq!(segment.z_offset(), 0x55);
        assert_eq!(segment.direction(), 0xaa);
        assert_eq!(ram[BEAMOS_LASER_HISTORY_X_HI + 9], 0x55);
        assert_eq!(ram[BEAMOS_LASER_HISTORY_Y_HI + 9], 0xaa);
        assert_eq!(effects.sprite_histories.beamos_laser_history(9).x(), 0x5567);
        assert_eq!(effects.sprite_histories.beamos_laser_history(9).y(), 0xaaab);
    }

    #[test]
    fn native_cached_sprite_bridge_updates_alt_and_live_banks() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut state = SpriteState::load_from_ram(&ram);

        {
            let mut bridge =
                NativeCachedSpriteBridgeMut::new(&mut state.cached_sprites, &mut ram, 3);
            bridge.cache_sprite_header(0xaa, 0x11, 0x22, 0x33, 0x44, 0x55);
        }
        let slot = state.cached_sprites.slot(3);
        assert!(!slot.is_active());
        assert_eq!(slot.type_byte(), 0xaa);
        assert_eq!(slot.y_high(), 0x44);
        assert_eq!(ram[ALT_SPRITE_TYPE + 3], 0xaa);
        assert_eq!(ram[ALT_SPRITE_X_LO + 3], 0x11);
        assert_eq!(ram[ALT_SPRITE_X_HI + 3], 0x22);
        assert_eq!(ram[ALT_SPRITE_Y_LO + 3], 0x33);
        assert_eq!(ram[ALT_SPRITE_Y_HI + 3], 0x44);
        assert_eq!(ram[ALT_SPRITE_GRAPHICS + 3], 0x55);

        {
            let mut bridge =
                NativeCachedSpriteBridgeMut::new(&mut state.cached_sprites, &mut ram, 3);
            bridge.initialize_trinexx_component();
            bridge.set_type_byte(0x66);
            bridge.set_y_high(0x77);
        }
        assert_eq!(state.cached_sprites.slot(3).type_byte(), 0x66);
        assert_eq!(state.cached_sprites.slot(3).y_high(), 0x77);
        assert_eq!(ram[ALT_SPRITE_X_HI + 3], 0);

        for (index, live) in CACHED_SPRITE_LIVE_FIELDS.iter().copied().enumerate() {
            ram[live + 3] = index as u8;
        }
        {
            let mut bridge =
                NativeCachedSpriteBridgeMut::new(&mut state.cached_sprites, &mut ram, 3);
            bridge.cache_live_fields();
        }
        for (index, alt) in CACHED_SPRITE_ALT_FIELDS.iter().copied().enumerate() {
            assert_eq!(ram[alt + 3], index as u8);
        }

        for (index, live) in CACHED_SPRITE_LIVE_FIELDS.iter().copied().enumerate() {
            ram[live + 3] = 0x80 | index as u8;
        }
        let mut backup = [0; 24];
        {
            let mut bridge =
                NativeCachedSpriteBridgeMut::new(&mut state.cached_sprites, &mut ram, 3);
            bridge.load_cached_into_live(&mut backup);
            bridge.clear_state();
        }
        for (index, live) in CACHED_SPRITE_LIVE_FIELDS.iter().copied().enumerate() {
            assert_eq!(backup[index], 0x80 | index as u8);
            assert_eq!(ram[live + 3], index as u8);
        }
        assert!(!state.cached_sprites.slot(3).is_active());
        {
            let mut bridge =
                NativeCachedSpriteBridgeMut::new(&mut state.cached_sprites, &mut ram, 3);
            bridge.restore_live_from_backup(&backup);
        }
        for (index, live) in CACHED_SPRITE_LIVE_FIELDS.iter().copied().enumerate() {
            assert_eq!(ram[live + 3], 0x80 | index as u8);
        }

        {
            let mut bridge =
                NativeCachedSpriteBridgeMut::new(&mut state.cached_sprites, &mut ram, 0x1a);
            bridge.initialize_trinexx_component();
        }
        assert_eq!(state.cached_sprites.slot(0x1a).type_byte(), 0x40);
        assert_eq!(ram[ALT_SPRITE_TYPE + 0x1a], 0x40);
    }

    #[test]
    fn native_boss_home_positions_load_and_update_overlord_scratch() {
        let mut ram = vec![0; WRAM_SIZE];
        let puff_slot = 5;
        let puff_overlord_slot = puff_slot + 7;
        ram[OVERLORD_X_LO + puff_overlord_slot] = 0x34;
        ram[OVERLORD_Y_LO + puff_overlord_slot] = 0x12;
        ram[OVERLORD_GEN1 + puff_overlord_slot] = 0x78;
        ram[OVERLORD_GEN3 + puff_overlord_slot] = 0x56;

        let mut state = SpriteState::load_from_ram(&ram);
        let puff_home = state
            .boss_home_positions
            .arrghus_puff_home_position(puff_slot);
        assert_eq!(puff_home.x(), 0x1234);
        assert_eq!(puff_home.y(), 0x5678);

        {
            let mut home = NativeArmosKnightHomePositionBridgeMut::new(
                &mut state.boss_home_positions,
                &mut ram,
                3,
            );
            home.set_position(0x9abc, 0xdef0);
        }
        let armos_home = state.boss_home_positions.armos_knight_home_position(3);
        assert_eq!(armos_home.x(), 0x9abc);
        assert_eq!(armos_home.y(), 0xdef0);
        assert_eq!(ram[OVERLORD_X_HI + 3], 0xbc);
        assert_eq!(ram[OVERLORD_Y_HI + 3], 0x9a);
        assert_eq!(ram[OVERLORD_GEN2 + 3], 0xf0);
        assert_eq!(ram[OVERLORD_FLOOR + 3], 0xde);
    }

    #[test]
    fn tower_seal_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TOWER_SEAL_RING_RADIUS] = 32;
        write_le_u16(&mut ram, TOWER_SEAL_CENTER_X, 0x1234);
        write_le_u16(&mut ram, TOWER_SEAL_CENTER_Y, 0x5678);
        ram[TOWER_SEAL_WAIT_COUNTDOWN] = 2;

        let mut tower = TowerSealState::load_from_ram(&ram);
        assert_eq!(tower.ring_radius(), 32);
        assert_eq!(tower.center_x(), 0x1234);
        assert_eq!(tower.center_y(), 0x5678);
        tower.set_ring_radius(48);
        tower.set_center(0x9abc, 0xdef0);
        assert_eq!(tower.tick_wait_countdown(), 1);
        tower.write_to_ram(&mut ram);

        assert_eq!(ram[TOWER_SEAL_RING_RADIUS], 48);
        assert_eq!(read_le_u16(&ram, TOWER_SEAL_CENTER_X), 0x9abc);
        assert_eq!(read_le_u16(&ram, TOWER_SEAL_CENTER_Y), 0xdef0);
        assert_eq!(ram[TOWER_SEAL_WAIT_COUNTDOWN], 1);
    }

    #[test]
    fn native_tower_seal_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TOWER_SEAL_WAIT_COUNTDOWN] = 1;

        let mut tower = TowerSealState::default();
        {
            let mut bridge = NativeTowerSealBridgeMut::new(&mut tower, &mut ram);
            bridge.set_ring_radius(48);
            bridge.set_center(0x1234, 0x5678);
            assert_eq!(bridge.tick_wait_countdown(), 0);
            bridge.set_wait_countdown(240);
        }

        assert_eq!(tower.ring_radius(), 48);
        assert_eq!(tower.center_x(), 0x1234);
        assert_eq!(tower.center_y(), 0x5678);
        assert_eq!(ram[TOWER_SEAL_RING_RADIUS], 48);
        assert_eq!(read_le_u16(&ram, TOWER_SEAL_CENTER_X), 0x1234);
        assert_eq!(read_le_u16(&ram, TOWER_SEAL_CENTER_Y), 0x5678);
        assert_eq!(ram[TOWER_SEAL_WAIT_COUNTDOWN], 240);
    }

    #[test]
    fn native_tower_seal_slot_bridges_sync_transient_orbits_and_sparkles() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TOWER_SEAL_ORBIT_ANGLE + 2] = 0x3f;
        ram[TOWER_SEAL_SPARKLE_TIMER + 5] = 1;

        let mut tower = TowerSealState::default();
        {
            let mut orbit = NativeTowerSealOrbitBridgeMut::new(&mut tower, &mut ram, 2);
            assert_eq!(orbit.advance_angle_mod64(), 0);
            orbit.set_base_sparkle_position(0x1234, 0x5678);
        }
        {
            let mut sparkle = NativeTowerSealSparkleBridgeMut::new(&mut tower, &mut ram, 5);
            sparkle.set_phase(1);
            assert_eq!(sparkle.tick_timer(), 0);
            assert_eq!(sparkle.advance_phase(), 2);
            sparkle.set_position(0x9abc, 0xdef0);
            assert_eq!(sparkle.base_sparkle_position(2), (0x1234, 0x5678));
        }

        assert_eq!(tower.orbit(2).angle(), 0);
        assert_eq!(tower.sparkle(5).phase(), 2);
        assert_eq!(tower.sparkle(5).x(), 0x9abc);
        assert_eq!(tower.sparkle(5).y(), 0xdef0);
        assert_eq!(ram[TOWER_SEAL_ORBIT_ANGLE + 2], 0);
        assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_X_LO + 2], 0x34);
        assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_X_HI + 2], 0x12);
        assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_Y_LO + 2], 0x78);
        assert_eq!(ram[TOWER_SEAL_BASE_SPARKLE_Y_HI + 2], 0x56);
        assert_eq!(ram[TOWER_SEAL_SPARKLE_PHASE + 5], 2);
        assert_eq!(ram[TOWER_SEAL_SPARKLE_TIMER + 5], 0);
        assert_eq!(ram[TOWER_SEAL_SPARKLE_X_LO + 5], 0xbc);
        assert_eq!(ram[TOWER_SEAL_SPARKLE_X_HI + 5], 0x9a);
        assert_eq!(ram[TOWER_SEAL_SPARKLE_Y_LO + 5], 0xf0);
        assert_eq!(ram[TOWER_SEAL_SPARKLE_Y_HI + 5], 0xde);
    }

    #[test]
    fn skull_woods_fire_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[SKULL_WOODS_FIRE_STARTED] = 1;
        write_le_u16(&mut ram, SKULL_WOODS_FIRE_INNER_X, 0x1234);
        write_le_u16(&mut ram, SKULL_WOODS_FIRE_INNER_Y, 0x5678);
        write_le_u16(&mut ram, SKULL_WOODS_FIRE_OUTER_X, 0x9abc);
        write_le_u16(&mut ram, SKULL_WOODS_FIRE_OUTER_Y, 0xdef0);

        let mut fire = SkullWoodsFireState::load_from_ram(&ram);
        assert!(fire.has_started_entrance_opening());
        assert_eq!(fire.inner_x(), 0x1234);
        assert_eq!(fire.inner_y(), 0x5678);
        fire.clear_entrance_opening_started();
        assert_eq!(fire.retreat_inner_y(8), 0x5670);
        fire.set_inner_position(0x1111, 0x2222);
        fire.set_outer_position(0x3333, 0x4444);
        fire.write_to_ram(&mut ram);

        assert_eq!(ram[SKULL_WOODS_FIRE_STARTED], 0);
        assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_INNER_X), 0x1111);
        assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_INNER_Y), 0x2222);
        assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_OUTER_X), 0x3333);
        assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_OUTER_Y), 0x4444);
    }

    #[test]
    fn native_skull_woods_fire_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, SKULL_WOODS_FIRE_INNER_Y, 0x0100);

        let mut effects = EntranceEffectState::default();
        {
            let mut bridge = NativeSkullWoodsFireBridgeMut::new(&mut effects, &mut ram);
            bridge.set_entrance_opening_started();
            bridge.set_inner_position(0x0098, 0x0100);
            bridge.set_outer_position(0x0098, 0x0100);
            assert_eq!(bridge.retreat_inner_y(8), 0x00f8);
        }

        let fire = effects.skull_woods_fire();
        assert!(fire.has_started_entrance_opening());
        assert_eq!(fire.inner_x(), 0x0098);
        assert_eq!(fire.inner_y(), 0x00f8);
        assert_eq!(ram[SKULL_WOODS_FIRE_STARTED], 1);
        assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_INNER_X), 0x0098);
        assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_INNER_Y), 0x00f8);
        assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_OUTER_X), 0x0098);
        assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_OUTER_Y), 0x0100);
    }

    #[test]
    fn blast_wall_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[BLAST_WALL_ENTRY_STATE] = 3;
        ram[BLAST_WALL_SECONDARY_STATE] = 4;
        ram[BLAST_WALL_DIRECTION] = 2;
        write_le_u16(&mut ram, BLAST_WALL_CENTER_X, 0x1234);
        write_le_u16(&mut ram, BLAST_WALL_CENTER_Y, 0x5678);

        let mut wall = BlastWallState::load_from_ram(&ram);
        assert_eq!(wall.direction(), 2);
        assert_eq!(wall.center_x(), 0x1234);
        assert_eq!(wall.center_y(), 0x5678);
        wall.clear_entry_state();
        wall.clear_secondary_state();
        assert_eq!(wall.offset_center(-4, 8), (0x1230, 0x5680));
        wall.write_to_ram(&mut ram);

        assert_eq!(ram[BLAST_WALL_ENTRY_STATE], 0);
        assert_eq!(ram[BLAST_WALL_SECONDARY_STATE], 0);
        assert_eq!(ram[BLAST_WALL_DIRECTION], 2);
        assert_eq!(read_le_u16(&ram, BLAST_WALL_CENTER_X), 0x1230);
        assert_eq!(read_le_u16(&ram, BLAST_WALL_CENTER_Y), 0x5680);
    }

    #[test]
    fn native_blast_wall_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[BLAST_WALL_ENTRY_STATE] = 1;
        ram[BLAST_WALL_SECONDARY_STATE] = 1;
        write_le_u16(&mut ram, BLAST_WALL_CENTER_X, 0x0100);
        write_le_u16(&mut ram, BLAST_WALL_CENTER_Y, 0x0200);

        let mut effects = EntranceEffectState::default();
        {
            let mut bridge = NativeBlastWallBridgeMut::new(&mut effects, &mut ram);
            bridge.clear_entry_state();
            bridge.clear_secondary_state();
            assert_eq!(bridge.offset_center(2, -3), (0x0102, 0x01fd));
        }

        let wall = effects.blast_wall();
        assert_eq!(wall.center_x(), 0x0102);
        assert_eq!(wall.center_y(), 0x01fd);
        assert_eq!(ram[BLAST_WALL_ENTRY_STATE], 0);
        assert_eq!(ram[BLAST_WALL_SECONDARY_STATE], 0);
        assert_eq!(read_le_u16(&ram, BLAST_WALL_CENTER_X), 0x0102);
        assert_eq!(read_le_u16(&ram, BLAST_WALL_CENTER_Y), 0x01fd);
    }

    #[test]
    fn entrance_effect_bank_syncs_shared_blast_wall_and_skull_woods_slots() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[BLAST_WALL_EXPLOSION_PHASE] = 2;
        ram[BLAST_WALL_EXPLOSION_TIMER] = 3;
        write_le_u16(&mut ram, BLAST_WALL_FRAGMENT_X + 4, 0x0100);
        write_le_u16(&mut ram, BLAST_WALL_FRAGMENT_Y + 4, 0x0200);
        ram[BLAST_WALL_FIREBALL_TIMER + 7] = 9;

        let effects = EntranceEffectState::load_from_ram(&ram);
        assert_eq!(effects.blast_wall_explosion_slot(0).phase(), 2);
        assert_eq!(effects.blast_wall_explosion_slot(0).timer(), 3);
        assert_eq!(effects.blast_wall_fragment_slot(2).x(), 0x0100);
        assert_eq!(effects.skull_woods_fire_slot(2).y(), 0x0200);
        assert_eq!(effects.blast_wall_fireball_slot(7).timer(), 9);

        let mut effects = EntranceEffectState::default();
        NativeSkullWoodsFireSlotBridgeMut::new(&mut effects, &mut ram, 2).set_phase(0xff);
        NativeSkullWoodsFireSlotBridgeMut::new(&mut effects, &mut ram, 2).set_timer(5);
        NativeSkullWoodsFireSlotBridgeMut::new(&mut effects, &mut ram, 2)
            .set_position(0x0300, 0x0400);
        NativeBlastWallFireballBridgeMut::new(&mut effects, &mut ram, 7).set_timer(8);

        assert!(effects.skull_woods_fire_slot(2).is_finished());
        assert_eq!(ram[SKULL_WOODS_FIRE_PHASE + 2], 0xff);
        assert_eq!(ram[SKULL_WOODS_FIRE_TIMER + 2], 5);
        assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_X + 4), 0x0300);
        assert_eq!(read_le_u16(&ram, SKULL_WOODS_FIRE_Y + 4), 0x0400);
        assert_eq!(ram[BLAST_WALL_FIREBALL_TIMER + 7], 8);
    }

    #[test]
    fn digging_game_prize_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DIGGING_GAME_PRIZE_ATTEMPTS] = 24;
        ram[DIGGING_GAME_PRIZE_SPAWNED] = 0;

        let mut prize = DiggingGamePrizeState::load_from_ram(&ram);
        assert_eq!(prize.attempts(), 24);
        assert_eq!(prize.spawned_marker(), 0);
        prize.increment_attempts();
        prize.mark_spawned();
        prize.write_to_ram(&mut ram);

        assert_eq!(ram[DIGGING_GAME_PRIZE_ATTEMPTS], 25);
        assert_eq!(ram[DIGGING_GAME_PRIZE_SPAWNED], 0xeb);
    }

    #[test]
    fn native_digging_game_prize_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DIGGING_GAME_PRIZE_ATTEMPTS] = 0xff;
        ram[DIGGING_GAME_PRIZE_SPAWNED] = 0xeb;

        let mut prize = DiggingGamePrizeState::default();
        {
            let mut bridge = NativeDiggingGamePrizeBridgeMut::new(&mut prize, &mut ram);
            bridge.increment_attempts();
            bridge.clear_prize_spawned();
        }

        assert_eq!(prize.attempts(), 0);
        assert_eq!(prize.spawned_marker(), 0);
        assert_eq!(ram[DIGGING_GAME_PRIZE_ATTEMPTS], 0);
        assert_eq!(ram[DIGGING_GAME_PRIZE_SPAWNED], 0);
    }

    #[test]
    fn door_debris_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DOOR_DEBRIS_X + 4, 0x1234);
        write_le_u16(&mut ram, DOOR_DEBRIS_Y + 4, 0x5678);
        ram[DOOR_DEBRIS_X + 7] = 0x9a;
        ram[DOOR_DEBRIS_Y + 7] = 0xbc;
        ram[DOOR_DEBRIS_DIRECTION + 7] = 3;

        let debris = DoorDebrisState::load_from_ram(&ram);
        assert_eq!(debris.x_word(2), 0x1234);
        assert_eq!(debris.y_word(2), 0x5678);
        assert_eq!(debris.x(7), 0x9a);
        assert_eq!(debris.y(7), 0xbc);
        assert_eq!(debris.direction(7), 3);
        assert_eq!(debris.x_word(5), 0);

        let mut projected = vec![0; WRAM_SIZE];
        debris.write_to_ram(&mut projected);
        assert_eq!(DoorDebrisState::load_from_ram(&projected), debris);
    }

    #[test]
    fn native_door_debris_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DOOR_DEBRIS_X + 3] = 0xff;
        ram[DOOR_DEBRIS_Y + 3] = 0xff;
        ram[DOOR_DEBRIS_DIRECTION + 3] = 0xff;

        let mut debris = DoorDebrisState::default();
        {
            let mut bridge = NativeDoorDebrisBridgeMut::new(&mut debris, &mut ram);
            bridge.set_y_low_and_x_low_from_word(3, 0x1234);
            bridge.set_x_word(2, 0x4567);
            bridge.set_y_word(2, 0x89ab);
            bridge.set_direction(3, 2);
            bridge.set_direction(12, 1);
        }

        assert_eq!(debris.x(3), 0x12);
        assert_eq!(debris.y(3), 0x34);
        assert_eq!(debris.x_word(2), 0x4567);
        assert_eq!(debris.y_word(2), 0x89ab);
        assert_eq!(debris.direction(3), 2);
        assert_eq!(ram[DOOR_DEBRIS_X + 3], 0x12);
        assert_eq!(ram[DOOR_DEBRIS_Y + 3], 0x34);
        assert_eq!(read_le_u16(&ram, DOOR_DEBRIS_X + 4), 0x4567);
        assert_eq!(read_le_u16(&ram, DOOR_DEBRIS_Y + 4), 0x89ab);
        assert_eq!(ram[DOOR_DEBRIS_DIRECTION + 3], 2);
    }

    #[test]
    fn dungeon_key_slots_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[LINK_KEYS_EARNED_PER_DUNGEON] = 1;
        ram[LINK_KEYS_EARNED_PER_DUNGEON + 5] = 6;
        ram[LINK_KEYS_EARNED_PER_DUNGEON + 15] = 16;

        let slots = DungeonKeySlotsState::load_from_ram(&ram);
        assert_eq!(slots.keys_earned(0), 1);
        assert_eq!(slots.keys_earned(10), 6);
        assert_eq!(slots.keys_earned_slot(15), 16);
        assert_eq!(slots.keys_earned_slot(16), 0);

        let mut projected = vec![0; WRAM_SIZE];
        slots.write_to_ram(&mut projected);
        assert_eq!(projected[LINK_KEYS_EARNED_PER_DUNGEON], 1);
        assert_eq!(projected[LINK_KEYS_EARNED_PER_DUNGEON + 5], 6);
        assert_eq!(projected[LINK_KEYS_EARNED_PER_DUNGEON + 15], 16);
    }

    #[test]
    fn native_dungeon_key_slots_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[LINK_KEYS_EARNED_PER_DUNGEON + 2] = 3;

        let mut slots = DungeonKeySlotsState::default();
        {
            let mut bridge = NativeDungeonKeySlotsBridgeMut::new(&mut slots, &mut ram);
            bridge.set_keys_earned(4, 7);
            bridge.set_keys_earned_slot(5, 9);
            bridge.set_keys_earned_slot(16, 11);
        }

        assert_eq!(slots.keys_earned(4), 7);
        assert_eq!(slots.keys_earned_slot(5), 9);
        assert_eq!(slots.keys_earned_slot(16), 0);
        assert_eq!(ram[LINK_KEYS_EARNED_PER_DUNGEON + 2], 7);
        assert_eq!(ram[LINK_KEYS_EARNED_PER_DUNGEON + 5], 9);
        assert_eq!(ram[LINK_KEYS_EARNED_PER_DUNGEON + 15], 0);
    }

    #[test]
    fn mirror_warp_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MIRROR_WARP_TARGET_INDEX, 2);
        write_le_u16(&mut ram, MIRROR_WARP_TARGET_OFFSETS, 0xfe00);
        write_le_u16(&mut ram, MIRROR_WARP_TARGET_OFFSETS + 2, 0x0200);
        write_le_u16(&mut ram, MIRROR_WARP_VELOCITY_DELTAS, 0xffc0);
        write_le_u16(&mut ram, MIRROR_WARP_VELOCITY_DELTAS + 2, 0x0040);
        write_le_u16(&mut ram, MIRROR_WARP_WAVE_OFFSET, 0x0012);
        write_le_u16(&mut ram, MIRROR_WARP_DISPLACEMENT, 0x0034);
        write_le_u16(&mut ram, MIRROR_WARP_SUBPIXEL, 0x0056);
        ram[MIRROR_WARP_LOAD_STEP_COUNTER] = 7;
        ram[MIRROR_WARP_ANIMATION_COUNTER] = 8;

        let mut mirror = MirrorWarpState::load_from_ram(&ram);
        assert_eq!(mirror.target_index(), 1);
        assert_eq!(mirror.target_offset(), 0x0200);
        assert_eq!(mirror.velocity_delta(), 0x0040);
        assert_eq!(mirror.wave_offset(), 0x0012);
        assert_eq!(mirror.displacement(), 0x0034);
        assert_eq!(mirror.subpixel(), 0x0056);
        assert_eq!(mirror.animation_counter(), 8);

        mirror.reset_wave_and_subpixel();
        mirror.toggle_target_index();
        mirror.set_displacement(0x0078);
        mirror.set_subpixel_low_from(0x019a);
        mirror.set_wave_offset(0x00bc);
        mirror.shrink_target_offsets_for_dewaving();
        assert_eq!(mirror.increment_load_step_counter(), 8);
        assert_eq!(mirror.decrement_animation_counter(), 7);
        mirror.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_INDEX), 0);
        assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_OFFSETS), 0xff00);
        assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_OFFSETS + 2), 0x0100);
        assert_eq!(read_le_u16(&ram, MIRROR_WARP_WAVE_OFFSET), 0x00bc);
        assert_eq!(read_le_u16(&ram, MIRROR_WARP_DISPLACEMENT), 0x0078);
        assert_eq!(read_le_u16(&ram, MIRROR_WARP_SUBPIXEL), 0x009a);
        assert_eq!(ram[MIRROR_WARP_LOAD_STEP_COUNTER], 8);
        assert_eq!(ram[MIRROR_WARP_ANIMATION_COUNTER], 7);
    }

    #[test]
    fn native_mirror_warp_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MIRROR_WARP_TARGET_INDEX, 2);
        ram[MIRROR_WARP_LOAD_STEP_COUNTER] = 0xff;
        ram[MIRROR_WARP_ANIMATION_COUNTER] = 0;

        let mut mirror = MirrorWarpState::default();
        {
            let mut bridge = NativeMirrorWarpBridgeMut::new(&mut mirror, &mut ram);
            bridge.initialize_hdma_wave_state();
            bridge.toggle_target_index();
            bridge.set_displacement(0x0044);
            bridge.set_subpixel_low_from(0x0166);
            bridge.set_wave_offset(0x0088);
            bridge.shrink_target_offsets_for_dewaving();
            assert_eq!(bridge.increment_load_step_counter(), 0);
            bridge.reset_load_step_counter();
            bridge.set_animation_counter(2);
            assert_eq!(bridge.decrement_animation_counter(), 1);
        }

        assert_eq!(mirror.target_index(), 1);
        assert_eq!(mirror.target_offset(), 0x0100);
        assert_eq!(mirror.velocity_delta(), 0x0040);
        assert_eq!(mirror.wave_offset(), 0x0088);
        assert_eq!(mirror.displacement(), 0x0044);
        assert_eq!(mirror.subpixel(), 0x0066);
        assert_eq!(mirror.animation_counter(), 1);
        assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_INDEX), 2);
        assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_OFFSETS), 0xff00);
        assert_eq!(read_le_u16(&ram, MIRROR_WARP_TARGET_OFFSETS + 2), 0x0100);
        assert_eq!(read_le_u16(&ram, MIRROR_WARP_WAVE_OFFSET), 0x0088);
        assert_eq!(read_le_u16(&ram, MIRROR_WARP_DISPLACEMENT), 0x0044);
        assert_eq!(read_le_u16(&ram, MIRROR_WARP_SUBPIXEL), 0x0066);
        assert_eq!(ram[MIRROR_WARP_LOAD_STEP_COUNTER], 0);
        assert_eq!(ram[MIRROR_WARP_ANIMATION_COUNTER], 1);
    }

    #[test]
    fn save_progress_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[CUR_PALACE_INDEX_X2] = 10;
        ram[SRAM_PROGRESS_INDICATOR] = 2;
        ram[SRAM_PROGRESS_FLAGS] = 0x40;
        ram[SAVEGAME_MAP_ICONS_INDICATOR] = 7;
        ram[WHICH_STARTING_POINT] = 3;
        ram[SRAM_PROGRESS_INDICATOR_3] = 0x20;
        ram[SAVEGAME_IS_DARKWORLD] = 0x40;
        ram[HUD_CUR_ITEM] = 1;
        ram[HUD_CUR_ITEM_X] = 2;
        ram[HUD_CUR_ITEM_L] = 3;
        ram[HUD_CUR_ITEM_R] = 4;
        write_le_u16(&mut ram, SAVE_DUNG_INFO + 0x109 * 2, 0x0080);
        write_le_u16(&mut ram, DEATHS_PER_PALACE + 4 * 2, 0x0012);
        write_le_u16(&mut ram, PENDING_DEATH_SAVE_COUNTER, 0x0034);
        write_le_u16(&mut ram, TOTAL_DEATH_SAVE_COUNTER, 0xffff);
        ram[HUD_POST_MESSAGE_REFRESH_FLAG] = 0x80;

        let mut progress = SaveProgressState::load_from_ram(&ram);
        assert_eq!(progress.palace_index_x2(), 10);
        assert_eq!(progress.palace_index(), 5);
        assert_eq!(progress.progress_indicator_word(), 0x4002);
        assert!(progress.progress_flags_has(0x40));
        assert_eq!(progress.map_icons_indicator(), 7);
        assert_eq!(progress.dark_world_bit6(), 1);
        assert_eq!(progress.hud_current_item(), 1);
        assert_eq!(progress.hud_current_item_slot(3), 4);
        assert_eq!(progress.dungeon_info_word(0x109), 0x0080);
        assert_eq!(progress.death_count_for_palace(4), 0x0012);
        assert_eq!(progress.pending_death_save_counter(), 0x0034);
        assert!(progress.total_death_save_counter_is_uninitialized());
        assert_eq!(progress.which_starting_point(), 3);
        assert_eq!(progress.progress_indicator_3(), 0x20);

        progress.xor_palace_index_x2(2);
        progress.or_progress_flags(1);
        progress.clear_progress_indicator_3_bits(0x20);
        progress.xor_dark_world_state(0x40);
        progress.set_hud_current_item_slot(2, 9);
        progress.or_dungeon_info_word(0x109, 0x0100);
        progress.set_dungeon_info_checksum(0x5a5a);
        progress.increment_pending_death_save_counter();
        progress.set_total_death_save_counter(0x0045);
        progress.write_to_ram(&mut ram);

        assert_eq!(ram[CUR_PALACE_INDEX_X2], 8);
        assert_eq!(ram[SRAM_PROGRESS_FLAGS], 0x41);
        assert_eq!(ram[SRAM_PROGRESS_INDICATOR_3], 0);
        assert_eq!(ram[SAVEGAME_IS_DARKWORLD], 0);
        assert_eq!(ram[HUD_CUR_ITEM_L], 9);
        assert_eq!(read_le_u16(&ram, SAVE_DUNG_INFO + 0x109 * 2), 0x0180);
        assert_eq!(read_le_u16(&ram, SAVE_DUNG_INFO + 0x4fe), 0x5a5a);
        assert_eq!(read_le_u16(&ram, PENDING_DEATH_SAVE_COUNTER), 0x0035);
        assert_eq!(read_le_u16(&ram, TOTAL_DEATH_SAVE_COUNTER), 0x0045);
    }

    #[test]
    fn native_save_progress_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[SRAM_PROGRESS_FLAGS] = 0x10;
        ram[SRAM_PROGRESS_INDICATOR_3] = 0xff;
        ram[HUD_CUR_ITEM] = 1;
        write_le_u16(&mut ram, SAVE_DUNG_INFO + 2, 0x0001);

        let mut progress = SaveProgressState::default();
        {
            let mut bridge = NativeSaveProgressBridgeMut::new(&mut progress, &mut ram);
            bridge.set_palace_index_x2(0xff);
            bridge.xor_palace_index_x2(1);
            bridge.set_which_starting_point(5);
            bridge.set_progress_indicator(3);
            bridge.or_progress_flags(0x20);
            bridge.set_progress_flags(0x22);
            bridge.or_progress_indicator_3(0x01);
            bridge.clear_progress_indicator_3_bits(0xf0);
            bridge.set_map_icons_indicator(6);
            bridge.set_dark_world_state(0x40);
            bridge.xor_dark_world_state(0x40);
            bridge.set_hud_current_item(2);
            bridge.set_hud_current_item_slot(3, 7);
            bridge.set_death_count_for_palace(1, 0x0044);
            assert_eq!(bridge.increment_pending_death_save_counter(), 1);
            bridge.clear_pending_death_save_counter();
            bridge.set_total_death_save_counter(0x0055);
            bridge.request_post_message_refresh();
            assert_eq!(bridge.or_dungeon_info_word(1, 0x0100), 0x0101);
            bridge.set_dungeon_info_checksum(0x1234);
            bridge.clear_post_message_refresh_flag();
        }

        assert_eq!(progress.palace_index_x2(), 0xfe);
        assert_eq!(progress.which_starting_point(), 5);
        assert_eq!(progress.progress_indicator(), 3);
        assert_eq!(progress.progress_flags(), 0x22);
        assert_eq!(progress.progress_indicator_3(), 0x0f);
        assert_eq!(progress.map_icons_indicator(), 6);
        assert_eq!(progress.dark_world_state(), 0);
        assert_eq!(progress.hud_current_item(), 2);
        assert_eq!(progress.hud_current_item_slot(3), 7);
        assert_eq!(progress.death_count_for_palace(1), 0x0044);
        assert_eq!(progress.pending_death_save_counter(), 0);
        assert_eq!(progress.total_death_save_counter(), 0x0055);
        assert_eq!(progress.dungeon_info_word(1), 0x0101);
        assert_eq!(ram[CUR_PALACE_INDEX_X2], 0xfe);
        assert_eq!(ram[WHICH_STARTING_POINT], 5);
        assert_eq!(ram[SRAM_PROGRESS_INDICATOR], 3);
        assert_eq!(ram[SRAM_PROGRESS_FLAGS], 0x22);
        assert_eq!(ram[SRAM_PROGRESS_INDICATOR_3], 0x0f);
        assert_eq!(ram[SAVEGAME_MAP_ICONS_INDICATOR], 6);
        assert_eq!(ram[SAVEGAME_IS_DARKWORLD], 0);
        assert_eq!(ram[HUD_CUR_ITEM], 2);
        assert_eq!(ram[HUD_CUR_ITEM_R], 7);
        assert_eq!(read_le_u16(&ram, DEATHS_PER_PALACE + 2), 0x0044);
        assert_eq!(read_le_u16(&ram, PENDING_DEATH_SAVE_COUNTER), 0);
        assert_eq!(read_le_u16(&ram, TOTAL_DEATH_SAVE_COUNTER), 0x0055);
        assert_eq!(read_le_u16(&ram, SAVE_DUNG_INFO + 2), 0x0101);
        assert_eq!(read_le_u16(&ram, SAVE_DUNG_INFO + 0x4fe), 0x1234);
        assert_eq!(ram[HUD_POST_MESSAGE_REFRESH_FLAG], 0);
    }

    #[test]
    fn player_resources_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[LINK_MAGIC_POWER] = 0x40;
        ram[LINK_MAGIC_CONSUMPTION] = 2;
        ram[LINK_ITEM_BOMBS] = 7;
        ram[LINK_ITEM_BOTTLE_INDEX] = 3;
        write_le_u16(&mut ram, LINK_RUPEES_GOAL, 0x0123);
        write_le_u16(&mut ram, LINK_RUPEES_ACTUAL, 0x0045);
        write_le_u16(&mut ram, LINK_COMPASS, 0x0008);
        write_le_u16(&mut ram, LINK_BIGKEY, 0x0010);
        write_le_u16(&mut ram, LINK_DUNGEON_MAP, 0x0020);
        ram[LINK_RUPEES_IN_POND] = 30;
        ram[LINK_HEART_PIECES] = 2;
        ram[LINK_HEALTH_CAPACITY] = 0x38;
        ram[LINK_CURRENT_HEALTH] = 0x28;
        ram[LINK_NUM_KEYS] = 4;
        ram[LINK_BOMB_UPGRADES] = 1;
        ram[LINK_ARROW_UPGRADES] = 2;
        ram[LINK_HEARTS_FILLER] = 5;
        ram[LINK_MAGIC_FILLER] = 6;
        ram[LINK_WHICH_PENDANTS] = 7;
        ram[LINK_BOMB_FILLER] = 8;
        ram[LINK_ARROW_REFILL_COUNTER] = 9;
        ram[LINK_NUM_ARROWS] = 10;
        ram[LINK_ABILITY_FLAGS] = 0x11;
        ram[LINK_HAS_CRYSTALS] = 0x22;
        ram[LINK_LOWLIFE_COUNTDOWN_TIMER_BEEP] = 0x33;

        let resources = PlayerResourcesState::load_from_ram(&ram);
        assert_eq!(resources.magic_power(), 0x40);
        assert_eq!(resources.magic_consumption_level(), 2);
        assert_eq!(resources.bombs(), 7);
        assert_eq!(resources.equipped_bottle_index(), 3);
        assert_eq!(resources.rupees_goal(), 0x0123);
        assert_eq!(resources.rupees_actual(), 0x0045);
        assert!(resources.has_compass_mask(0x0008));
        assert!(resources.has_big_key_mask(0x0010));
        assert!(resources.has_dungeon_map_mask(0x0020));
        assert_eq!(resources.rupees_in_pond(), 30);
        assert_eq!(resources.heart_pieces(), 2);
        assert_eq!(resources.health_capacity(), 0x38);
        assert_eq!(resources.current_health(), 0x28);
        assert_eq!(resources.keys(), 4);
        assert_eq!(resources.bomb_upgrade_level(), 1);
        assert_eq!(resources.arrow_upgrade_level(), 2);
        assert_eq!(resources.heart_filler(), 5);
        assert_eq!(resources.magic_filler(), 6);
        assert_eq!(resources.pendant_flags(), 7);
        assert_eq!(resources.bomb_filler(), 8);
        assert_eq!(resources.arrow_filler(), 9);
        assert_eq!(resources.arrows(), 10);
        assert_eq!(resources.ability_flags(), 0x11);
        assert_eq!(resources.crystal_flags(), 0x22);
        assert_eq!(resources.low_health_beep_timer(), 0x33);

        let mut projected = vec![0; WRAM_SIZE];
        resources.write_to_ram(&mut projected);
        assert_eq!(PlayerResourcesState::load_from_ram(&projected), resources);
    }

    #[test]
    fn native_player_resources_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[LINK_ITEM_BOMBS] = 1;
        ram[LINK_NUM_ARROWS] = 2;
        ram[LINK_HEARTS_FILLER] = 0xff;
        ram[LINK_MAGIC_FILLER] = 0;
        write_le_u16(&mut ram, LINK_RUPEES_GOAL, 10);
        ram[LINK_NUM_KEYS] = 0xff;

        let mut resources = PlayerResourcesState::default();
        {
            let mut bridge = NativePlayerResourcesBridgeMut::new(&mut resources, &mut ram);
            bridge.set_bombs(4);
            bridge.decrement_bombs();
            bridge.increment_arrows_by(5);
            bridge.increment_heart_filler_word_by(2);
            bridge.add_rupees_goal(90);
            bridge.subtract_rupees_goal(25);
            bridge.increment_keys();
            bridge.add_ability_flags(0x04);
            bridge.add_crystal_flags(0x20);
            bridge.set_pendant_flags(0x07);
        }

        assert_eq!(resources.bombs(), 3);
        assert_eq!(resources.arrows(), 7);
        assert_eq!(resources.heart_filler(), 1);
        assert_eq!(resources.magic_filler(), 1);
        assert_eq!(resources.rupees_goal(), 75);
        assert_eq!(resources.keys(), 0);
        assert_eq!(resources.ability_flags(), 0x04);
        assert_eq!(resources.crystal_flags(), 0x20);
        assert_eq!(resources.pendant_flags(), 0x07);
        assert_eq!(ram[LINK_ITEM_BOMBS], 3);
        assert_eq!(ram[LINK_NUM_ARROWS], 7);
        assert_eq!(read_le_u16(&ram, LINK_HEARTS_FILLER), 0x0101);
        assert_eq!(read_le_u16(&ram, LINK_RUPEES_GOAL), 75);
        assert_eq!(ram[LINK_NUM_KEYS], 0);
        assert_eq!(ram[LINK_ABILITY_FLAGS], 0x04);
        assert_eq!(ram[LINK_HAS_CRYSTALS], 0x20);
        assert_eq!(ram[LINK_WHICH_PENDANTS], 0x07);
    }

    #[test]
    fn world_location_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_ROOM, 0x0124);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x0040);
        ram[PLAYER_IS_INDOORS] = 1;

        let mut world = WorldLocationState::load_from_ram(&ram);
        assert_eq!(world.dungeon_room, 0x0124);
        assert_eq!(world.dungeon_room_index(), 0x24);
        assert_eq!(world.overworld_screen, 0x0040);
        assert_eq!(world.overworld_screen_index(), 0x40);
        assert!(world.is_indoors());
        assert!(!world.is_outdoors());

        world.dungeon_room = 0x0181;
        world.overworld_screen = 0x005b;
        world.indoor_flag = 0;
        world.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0181);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
        assert_eq!(ram[PLAYER_IS_INDOORS], 0);
    }

    #[test]
    fn native_world_location_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_ROOM, 0x0124);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x0040);
        ram[PLAYER_IS_INDOORS] = 1;

        let mut world = WorldLocationState::load_from_ram(&ram);
        {
            let mut bridge = NativeWorldLocationBridgeMut::new(&mut world, &mut ram);
            bridge.increment_dungeon_room_index_by(2);
            bridge.set_overworld_screen(0x5b);
            bridge.set_indoor_flag(0);
        }

        assert_eq!(world.dungeon_room, 0x0126);
        assert_eq!(world.overworld_screen, 0x005b);
        assert_eq!(world.indoor_flag, 0);
        assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0126);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
        assert_eq!(ram[PLAYER_IS_INDOORS], 0);
    }

    #[test]
    fn native_world_location_bridge_projects_native_state_over_stale_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut world = WorldLocationState {
            dungeon_room: 0x0124,
            overworld_screen: 0x0040,
            indoor_flag: 1,
        };
        world.write_to_ram(&mut ram);

        write_le_u16(&mut ram, DUNGEON_ROOM, 0x00aa);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x00bb);
        ram[PLAYER_IS_INDOORS] = 0xcc;

        {
            let mut bridge = NativeWorldLocationBridgeMut::new(&mut world, &mut ram);
            bridge.set_overworld_screen(0x5b);
        }

        assert_eq!(world.dungeon_room, 0x0124);
        assert_eq!(world.overworld_screen, 0x005b);
        assert_eq!(world.indoor_flag, 1);
        assert_eq!(WorldLocationState::load_from_ram(&ram), world);
        assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0124);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
        assert_eq!(ram[PLAYER_IS_INDOORS], 1);
    }

    #[test]
    fn world_camera_boundaries_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_LOW, 0x0101);
        write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_HI, 0x0202);
        write_le_u16(&mut ram, CAMERA_X_COORD_SCROLL_LOW, 0x0303);
        write_le_u16(&mut ram, CAMERA_X_COORD_SCROLL_HI, 0x0404);
        write_le_u16(&mut ram, UP_DOWN_SCROLL_TARGET, 0x0505);
        write_le_u16(&mut ram, LEFT_RIGHT_SCROLL_TARGET, 0x0606);
        write_le_u16(&mut ram, OVERWORLD_SCROLL_UP_COUNTER, 0x0707);
        write_le_u16(&mut ram, OVERWORLD_SCROLL_LEFT_COUNTER, 0x0808);
        write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_LOW_SPEXIT, 0x0909);
        write_le_u16(&mut ram, CAMERA_X_COORD_SCROLL_LOW_SPEXIT, 0x0a0a);
        write_le_u16(&mut ram, SPECIAL_EXIT_ROOM_BOUNDS_Y_START, 0x0b0b);
        write_le_u16(&mut ram, SPECIAL_EXIT_ROOM_BOUNDS_X_END, 0x0c0c);

        let boundaries = WorldCameraBoundariesState::load_from_ram(&ram);
        assert_eq!(boundaries.camera_y_coord_scroll_low(), 0x0101);
        assert_eq!(boundaries.camera_y_coord_scroll_hi(), 0x0202);
        assert_eq!(boundaries.camera_x_coord_scroll_low(), 0x0303);
        assert_eq!(boundaries.camera_x_coord_scroll_hi(), 0x0404);
        assert_eq!(boundaries.up_down_scroll_target(0), 0x0505);
        assert_eq!(boundaries.up_down_scroll_target(2), 0x0606);
        assert_eq!(boundaries.overworld_scroll_counter_for_axis(0), 0x0707);
        assert_eq!(boundaries.overworld_scroll_counter_for_axis(2), 0x0808);
        assert_eq!(boundaries.spexit_camera_y_scroll_low(), 0x0909);
        assert_eq!(boundaries.spexit_camera_x_scroll_low(), 0x0a0a);
        assert_eq!(boundaries.spexit_room_bound_y_start(), 0x0b0b);
        assert_eq!(boundaries.spexit_room_bound_x_end(), 0x0c0c);

        let mut projected = vec![0; WRAM_SIZE];
        boundaries.write_to_ram(&mut projected);
        assert_eq!(
            WorldCameraBoundariesState::load_from_ram(&projected),
            boundaries
        );
    }

    #[test]
    fn native_world_camera_boundaries_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut boundaries = WorldCameraBoundariesState::load_from_ram(&ram);
        {
            let mut bridge = NativeWorldCameraBoundariesBridgeMut::new(&mut boundaries, &mut ram);
            bridge.set_camera_y_coord_scroll_low(0x0101);
            bridge.set_camera_y_coord_scroll_hi(0x0202);
            bridge.set_camera_x_coord_scroll_low(0x0303);
            bridge.set_camera_x_coord_scroll_hi(0x0404);
            bridge.set_up_down_scroll_target(0x0505);
            bridge.set_left_right_scroll_target(0x0606);
            bridge.set_overworld_scroll_up_counter(0x0707);
            bridge.set_overworld_scroll_left_counter(0x0808);
            bridge.set_special_exit_room_bounds(0x0909, 0x0a0a, 0x0b0b, 0x0c0c);
            bridge.save_spexit_camera_coords();
        }

        assert_eq!(boundaries.camera_y_coord_scroll_low(), 0x0101);
        assert_eq!(boundaries.camera_y_coord_scroll_hi(), 0x0202);
        assert_eq!(boundaries.camera_x_coord_scroll_low(), 0x0303);
        assert_eq!(boundaries.camera_x_coord_scroll_hi(), 0x0404);
        assert_eq!(boundaries.up_down_scroll_target(0), 0x0505);
        assert_eq!(boundaries.up_down_scroll_target(2), 0x0606);
        assert_eq!(boundaries.overworld_scroll_counter_for_axis(0), 0x0707);
        assert_eq!(boundaries.overworld_scroll_counter_for_axis(2), 0x0808);
        assert_eq!(boundaries.spexit_camera_y_scroll_low(), 0x0101);
        assert_eq!(boundaries.spexit_camera_x_scroll_low(), 0x0303);
        assert_eq!(boundaries.spexit_room_bound_y_start(), 0x0909);
        assert_eq!(boundaries.spexit_room_bound_x_end(), 0x0c0c);
        assert_eq!(WorldCameraBoundariesState::load_from_ram(&ram), boundaries);
    }

    #[test]
    fn native_world_camera_boundaries_bridge_projects_native_state_over_stale_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut boundaries = WorldCameraBoundariesState::default();
        {
            let mut bridge = NativeWorldCameraBoundariesBridgeMut::new(&mut boundaries, &mut ram);
            bridge.set_camera_y_coord_scroll_low(0x0101);
            bridge.set_camera_y_coord_scroll_hi(0x0202);
            bridge.set_camera_x_coord_scroll_low(0x0303);
            bridge.set_up_down_scroll_target(0x0404);
            bridge.set_overworld_scroll_up_counter(0x0505);
        }

        write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_LOW, 0xaaaa);
        write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_HI, 0xbbbb);
        write_le_u16(&mut ram, CAMERA_X_COORD_SCROLL_LOW, 0xcccc);
        write_le_u16(&mut ram, UP_DOWN_SCROLL_TARGET, 0xdddd);
        write_le_u16(&mut ram, OVERWORLD_SCROLL_UP_COUNTER, 0xeeee);

        {
            let mut bridge = NativeWorldCameraBoundariesBridgeMut::new(&mut boundaries, &mut ram);
            bridge.set_camera_x_coord_scroll_hi(0x0606);
        }

        assert_eq!(boundaries.camera_y_coord_scroll_low(), 0x0101);
        assert_eq!(boundaries.camera_y_coord_scroll_hi(), 0x0202);
        assert_eq!(boundaries.camera_x_coord_scroll_low(), 0x0303);
        assert_eq!(boundaries.camera_x_coord_scroll_hi(), 0x0606);
        assert_eq!(boundaries.up_down_scroll_target(0), 0x0404);
        assert_eq!(boundaries.overworld_scroll_counter_for_axis(0), 0x0505);
        assert_eq!(WorldCameraBoundariesState::load_from_ram(&ram), boundaries);
    }

    #[test]
    fn world_palette_theme_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[LAST_LIGHT_VS_DARK_WORLD] = 0x01;
        ram[AUX_BG_SUBSET_0] = 0x02;
        ram[AUX_BG_SUBSET_0 + 1] = 0x03;
        ram[AUX_BG_SUBSET_0 + 2] = 0x04;
        ram[AUX_BG_SUBSET_0 + 3] = 0x05;
        ram[OVERWORLD_PALETTE_AUX1_BP2TO4_HI] = 0x06;
        ram[OVERWORLD_PALETTE_MODE] = 0x07;
        ram[PALETTE_MAIN_INDOORS] = 0x08;
        ram[PALETTE_MAIN_INDOORS_COPY] = 0x09;
        ram[PALETTE_SWAP_FLAG] = 0x0a;
        ram[PALETTE_SP0L] = 0x0b;
        ram[PALETTE_SP5L] = 0x0c;
        ram[PALETTE_SP6L] = 0x0d;
        ram[PALETTE_SP6R_INDOORS] = 0x0e;
        ram[HUD_PALETTE] = 0x0f;
        ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = 0x10;
        ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = 0x11;
        ram[MISC_SPRITES_GRAPHICS_INDEX] = 0x12;
        ram[OVERWORLD_TILE_THEME_INDEX] = 0x13;
        ram[MAIN_TILE_THEME_INDEX] = 0x14;
        ram[AUX_TILE_THEME_INDEX] = 0x15;
        ram[OVERWORLD_SPECIAL_TILE_THEME_INDEX] = 0x16;
        ram[MAIN_TILE_THEME_INDEX_SPEXIT] = 0x17;
        ram[AUX_TILE_THEME_INDEX_SPEXIT] = 0x18;
        ram[OVERWORLD_TILE_THEME_INDEX_EXIT] = 0x19;
        ram[MAIN_TILE_THEME_INDEX_EXIT] = 0x1a;
        ram[AUX_TILE_THEME_INDEX_EXIT] = 0x1b;

        let theme = WorldPaletteThemeState::load_from_ram(&ram);
        assert_eq!(theme.last_light_vs_dark_world(), 0x01);
        assert_eq!(theme.aux_bg_subset(0), 0x02);
        assert_eq!(theme.aux_bg_subset(3), 0x05);
        assert_eq!(theme.overworld_palette_aux1_hi(), 0x06);
        assert_eq!(theme.overworld_palette_mode(), 0x07);
        assert_eq!(theme.palette_main_indoors(), 0x08);
        assert_eq!(theme.palette_main_indoors_copy(), 0x09);
        assert_eq!(theme.palette_swap_flag(), 0x0a);
        assert_eq!(theme.palette_sp0l(), 0x0b);
        assert_eq!(theme.palette_sp5l(), 0x0c);
        assert_eq!(theme.palette_sp6l(), 0x0d);
        assert_eq!(theme.palette_sp6r_indoors(), 0x0e);
        assert_eq!(theme.hud_palette(), 0x0f);
        assert_eq!(theme.overworld_palette_aux2_hi(), 0x10);
        assert_eq!(theme.overworld_palette_aux3_lo(), 0x11);
        assert_eq!(theme.misc_sprites_graphics_index(), 0x12);
        assert_eq!(theme.main_tile_theme_index(), 0x14);
        assert_eq!(theme.aux_tile_theme_index(), 0x15);

        let mut projected = vec![0; WRAM_SIZE];
        theme.write_to_ram(&mut projected);
        assert_eq!(WorldPaletteThemeState::load_from_ram(&projected), theme);
    }

    #[test]
    fn native_world_palette_theme_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut theme = WorldPaletteThemeState::load_from_ram(&ram);
        {
            let mut bridge = NativeWorldPaletteThemeBridgeMut::new(&mut theme, &mut ram);
            bridge.set_last_light_vs_dark_world(0x40);
            bridge.set_aux_bg_subset(2, 0x22);
            bridge.set_overworld_palette_aux1_hi(0x33);
            bridge.set_hud_palette(0x44);
            bridge.set_overworld_tile_theme_index(0x55);
            bridge.set_main_tile_theme_index(0x66);
            bridge.set_aux_tile_theme_index(0x77);
            bridge.set_misc_sprites_graphics_index(0x88);
            bridge.set_palette_sp6r_indoors(0x99);
            bridge.save_special_exit_tile_themes();
        }

        assert_eq!(theme.last_light_vs_dark_world(), 0x40);
        assert_eq!(theme.aux_bg_subset(2), 0x22);
        assert_eq!(theme.overworld_palette_aux1_hi(), 0x33);
        assert_eq!(theme.hud_palette(), 0x44);
        assert_eq!(theme.main_tile_theme_index(), 0x66);
        assert_eq!(theme.aux_tile_theme_index(), 0x77);
        assert_eq!(theme.misc_sprites_graphics_index(), 0x88);
        assert_eq!(theme.palette_sp6r_indoors(), 0x99);
        assert_eq!(WorldPaletteThemeState::load_from_ram(&ram), theme);
        assert_eq!(ram[LAST_LIGHT_VS_DARK_WORLD], 0x40);
        assert_eq!(ram[AUX_BG_SUBSET_0 + 2], 0x22);
        assert_eq!(ram[OVERWORLD_PALETTE_AUX1_BP2TO4_HI], 0x33);
        assert_eq!(ram[HUD_PALETTE], 0x44);
        assert_eq!(ram[OVERWORLD_TILE_THEME_INDEX], 0x55);
        assert_eq!(ram[MAIN_TILE_THEME_INDEX], 0x66);
        assert_eq!(ram[AUX_TILE_THEME_INDEX], 0x77);
        assert_eq!(ram[MISC_SPRITES_GRAPHICS_INDEX], 0x88);
        assert_eq!(ram[PALETTE_SP6R_INDOORS], 0x99);
        assert_eq!(ram[OVERWORLD_SPECIAL_TILE_THEME_INDEX], 0x55);
        assert_eq!(ram[MAIN_TILE_THEME_INDEX_SPEXIT], 0x66);
        assert_eq!(ram[AUX_TILE_THEME_INDEX_SPEXIT], 0x77);
    }

    #[test]
    fn native_world_palette_theme_bridge_projects_native_state_over_stale_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut theme = WorldPaletteThemeState::default();
        {
            let mut bridge = NativeWorldPaletteThemeBridgeMut::new(&mut theme, &mut ram);
            bridge.set_last_light_vs_dark_world(0x40);
            bridge.set_aux_bg_subset(1, 0x12);
            bridge.set_overworld_palette_aux1_hi(0x34);
            bridge.set_hud_palette(0x56);
            bridge.set_overworld_tile_theme_index(0x78);
        }

        ram[LAST_LIGHT_VS_DARK_WORLD] = 0xaa;
        ram[AUX_BG_SUBSET_0 + 1] = 0xbb;
        ram[OVERWORLD_PALETTE_AUX1_BP2TO4_HI] = 0xcc;
        ram[HUD_PALETTE] = 0xdd;
        ram[OVERWORLD_TILE_THEME_INDEX] = 0xee;

        {
            let mut bridge = NativeWorldPaletteThemeBridgeMut::new(&mut theme, &mut ram);
            bridge.set_main_tile_theme_index(0x9a);
        }

        assert_eq!(theme.last_light_vs_dark_world(), 0x40);
        assert_eq!(theme.aux_bg_subset(1), 0x12);
        assert_eq!(theme.overworld_palette_aux1_hi(), 0x34);
        assert_eq!(theme.hud_palette(), 0x56);
        assert_eq!(theme.main_tile_theme_index(), 0x9a);
        assert_eq!(WorldPaletteThemeState::load_from_ram(&ram), theme);
    }

    #[test]
    fn world_region_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, CURRENT_AREA_OF_PLAYER, 0x0102);
        write_le_u16(&mut ram, OVERWORLD_AREA_INDEX, 0x0304);
        write_le_u16(&mut ram, OVERWORLD_AREA_INDEX_SPEXIT, 0x0506);
        write_le_u16(&mut ram, OVERWORLD_AREA_INDEX_EXIT, 0x0708);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_PREV, 0x090a);
        write_le_u16(&mut ram, OVERLAY_INDEX, 0x0b0c);
        ram[RNG_SEED] = 0x0d;
        ram[IS_IN_DARK_WORLD_FLAG] = 0x0e;
        ram[FLAG_OVERWORLD_AREA_CHANGED] = 0x0f;
        write_le_u16(&mut ram, WHICH_ENTRANCE, 0x1011);
        write_le_u16(&mut ram, OW_ENTRANCE_VALUE, 0x1213);

        let region = WorldRegionState::load_from_ram(&ram);
        assert_eq!(region.current_area_of_player_word(), 0x0102);
        assert_eq!(region.overworld_area_index_word(), 0x0304);
        assert_eq!(region.spexit_area_index(), 0x0506);
        assert_eq!(region.prev_screen_index_word(), 0x090a);
        assert_eq!(region.overlay_index(), 0x0c);
        assert_eq!(region.rng_seed(), 0x0d);
        assert_eq!(region.dark_world_region_index(), 0x0e);
        assert!(region.is_in_dark_world());
        assert!(region.flag_overworld_area_changed());
        assert_eq!(region.which_entrance(), 0x1011);
        assert_eq!(region.ow_entrance_value(), 0x1213);

        let mut projected = vec![0; WRAM_SIZE];
        region.write_to_ram(&mut projected);
        assert_eq!(WorldRegionState::load_from_ram(&projected), region);
    }

    #[test]
    fn native_world_region_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut region = WorldRegionState::load_from_ram(&ram);
        {
            let mut bridge = NativeWorldRegionBridgeMut::new(&mut region, &mut ram);
            bridge.set_current_area_of_player_word(0x0102);
            bridge.set_overworld_area_index_word(0x0304);
            bridge.save_spexit_area_index();
            bridge.save_exit_area_index();
            bridge.set_prev_screen_index_word(0x0506);
            bridge.set_overlay_index_word(0x0708);
            bridge.set_rng_seed(0x09);
            bridge.set_dark_world_region_index(0x0a);
            bridge.set_flag_overworld_area_changed(0x0b);
            bridge.set_which_entrance(0x0c0d);
            bridge.set_ow_entrance_value(0x0e0f);
        }

        assert_eq!(region.current_area_of_player_word(), 0x0102);
        assert_eq!(region.overworld_area_index_word(), 0x0304);
        assert_eq!(region.spexit_area_index(), 0x0304);
        assert_eq!(region.prev_screen_index_word(), 0x0506);
        assert_eq!(region.overlay_index(), 0x08);
        assert_eq!(region.rng_seed(), 0x09);
        assert_eq!(region.dark_world_region_index(), 0x0a);
        assert!(region.flag_overworld_area_changed());
        assert_eq!(region.which_entrance(), 0x0c0d);
        assert_eq!(region.ow_entrance_value(), 0x0e0f);
        assert_eq!(WorldRegionState::load_from_ram(&ram), region);
    }

    #[test]
    fn native_world_region_bridge_projects_native_state_over_stale_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut region = WorldRegionState::default();
        {
            let mut bridge = NativeWorldRegionBridgeMut::new(&mut region, &mut ram);
            bridge.set_current_area_of_player_word(0x0102);
            bridge.set_overworld_area_index_word(0x0304);
            bridge.set_rng_seed(0x05);
            bridge.set_dark_world_region_index(0x06);
            bridge.set_which_entrance(0x0708);
        }

        write_le_u16(&mut ram, CURRENT_AREA_OF_PLAYER, 0xaaaa);
        write_le_u16(&mut ram, OVERWORLD_AREA_INDEX, 0xbbbb);
        ram[RNG_SEED] = 0xcc;
        ram[IS_IN_DARK_WORLD_FLAG] = 0xdd;
        write_le_u16(&mut ram, WHICH_ENTRANCE, 0xeeee);

        {
            let mut bridge = NativeWorldRegionBridgeMut::new(&mut region, &mut ram);
            bridge.set_ow_entrance_value(0x090a);
        }

        assert_eq!(region.current_area_of_player_word(), 0x0102);
        assert_eq!(region.overworld_area_index_word(), 0x0304);
        assert_eq!(region.rng_seed(), 0x05);
        assert_eq!(region.dark_world_region_index(), 0x06);
        assert_eq!(region.which_entrance(), 0x0708);
        assert_eq!(region.ow_entrance_value(), 0x090a);
        assert_eq!(WorldRegionState::load_from_ram(&ram), region);
    }

    #[test]
    fn world_transient_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 0x01;
        ram[ALLOW_SCROLL_Z] = 0x02;
        ram[MILESTONE_ITEM_GFX_SWAP_COUNTDOWN] = 0x03;
        write_le_u16(&mut ram, BIG_KEY_DOOR_MESSAGE_TRIGGERED, 0x0405);
        write_le_u16(&mut ram, SAVEGAME_HAS_MASTER_SWORD_FLAGS, 0x0607);
        ram[SUPER_BOMB_INDICATOR_TIMER] = 0x08;
        ram[IS_STANDING_IN_DOORWAY_CACHED] = 0x09;
        write_le_u16(&mut ram, CACHED_ROOM_BOUNDS_Y_START, 0x0a0b);
        write_le_u16(&mut ram, CACHED_ROOM_BOUNDS_X_END, 0x0c0d);
        write_le_u16(&mut ram, OVERWORLD_PEG_PUZZLE_PROGRESS, 0x0e0f);
        ram[OVERWORLD_HOLE_TILEMAP_POS] = 0x10;
        ram[HUD_CUR_ITEM_X] = 0x11;
        write_le_u16(&mut ram, DOOR_ANIMATION_STEP_INDICATOR, 0x1213);
        ram[ROOM_TRANSITIONING_FLAGS] = 0x14;
        ram[QUADRANT_FULLSIZE_X] = 0x15;
        ram[QUADRANT_FULLSIZE_Y] = 0x16;
        ram[MAPBAK_TM] = 0x17;
        ram[MAPBAK_TS] = 0x18;
        ram[OVERWORLD_HOLE_SCAN_STEP] = 0x19;
        write_le_u16(&mut ram, DUNG_REPLACEMENT_TILE_STATE + 4, 0x1a1b);

        let transient = WorldTransientState::load_from_ram(&ram);
        assert_eq!(transient.flag_custom_spell_anim_active(), 0x01);
        assert_eq!(transient.allow_scroll_z(), 0x02);
        assert_eq!(transient.milestone_item_gfx_swap_countdown(), 0x03);
        assert_eq!(transient.big_key_door_message_triggered(), 0x0405);
        assert_eq!(transient.savegame_has_master_sword_flags(), 0x0607);
        assert_eq!(transient.super_bomb_indicator_timer(), 0x08);
        assert_eq!(transient.is_standing_in_doorway_cached(), 0x09);
        assert_eq!(transient.overworld_peg_puzzle_progress(), 0x0e0f);
        assert_eq!(transient.overworld_hole_tilemap_pos(), 0x10);
        assert_eq!(transient.hud_cur_item_x(), 0x11);
        assert_eq!(transient.door_animation_step(), 0x1213);
        assert_eq!(transient.room_transitioning_flags(), 0x14);
        assert_eq!(transient.quadrant_fullsize_x(), 0x15);
        assert_eq!(transient.quadrant_fullsize_y(), 0x16);
        assert_eq!(transient.dung_replacement_tile_state(2), 0x1a1b);

        let mut projected = vec![0; WRAM_SIZE];
        transient.write_to_ram(&mut projected);
        assert_eq!(WorldTransientState::load_from_ram(&projected), transient);
    }

    #[test]
    fn native_world_transient_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut transient = WorldTransientState::default();
        {
            let mut bridge = NativeWorldTransientBridgeMut::new(&mut transient, &mut ram);
            bridge.set_custom_spell_animation_active();
            bridge.set_allow_scroll_z(0x02);
            bridge.set_room_transitioning_flags(0x03);
            bridge.set_cached_room_bounds(0x0405, 0x0607, 0x0809, 0x0a0b);
            bridge.set_standing_in_doorway_cached(0x0c);
            bridge.set_door_animation_step_word(0x0d0e);
            bridge.set_quadrant_fullsize_x(0x0f);
            bridge.set_quadrant_fullsize_y(0x10);
            bridge.cache_quadrant_fullsize_state();
            bridge.set_mapbak_tm(0x11);
            bridge.set_mapbak_ts(0x12);
            bridge.set_overworld_peg_puzzle_progress(0x1314);
            bridge.set_dung_replacement_tile_state(2, 0x1516);
        }

        assert_eq!(transient.flag_custom_spell_anim_active(), 1);
        assert_eq!(transient.allow_scroll_z(), 0x02);
        assert_eq!(transient.room_transitioning_flags(), 0x03);
        assert_eq!(transient.is_standing_in_doorway_cached(), 0x0c);
        assert_eq!(transient.door_animation_step(), 0x0d0e);
        assert_eq!(transient.quadrant_fullsize_x(), 0x0f);
        assert_eq!(transient.quadrant_fullsize_y(), 0x10);
        assert_eq!(transient.overworld_peg_puzzle_progress(), 0x1314);
        assert_eq!(transient.dung_replacement_tile_state(2), 0x1516);
        assert_eq!(ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE], 1);
        assert_eq!(ram[ALLOW_SCROLL_Z], 0x02);
        assert_eq!(ram[ROOM_TRANSITIONING_FLAGS], 0x03);
        assert_eq!(ram[IS_STANDING_IN_DOORWAY_CACHED], 0x0c);
        assert_eq!(read_le_u16(&ram, DOOR_ANIMATION_STEP_INDICATOR), 0x0d0e);
        assert_eq!(ram[QUADRANT_FULLSIZE_X], 0x0f);
        assert_eq!(ram[QUADRANT_FULLSIZE_Y], 0x10);
        assert_eq!(read_le_u16(&ram, OVERWORLD_PEG_PUZZLE_PROGRESS), 0x1314);
        assert_eq!(read_le_u16(&ram, DUNG_REPLACEMENT_TILE_STATE + 4), 0x1516);
    }

    #[test]
    fn native_world_transient_bridge_projects_native_state_over_stale_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut transient = WorldTransientState::default();
        {
            let mut bridge = NativeWorldTransientBridgeMut::new(&mut transient, &mut ram);
            bridge.set_custom_spell_animation_active();
            bridge.set_allow_scroll_z(0x02);
            bridge.set_room_transitioning_flags(0x03);
            bridge.set_door_animation_step_word(0x0405);
        }

        ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE] = 0xaa;
        ram[ALLOW_SCROLL_Z] = 0xbb;
        ram[ROOM_TRANSITIONING_FLAGS] = 0xcc;
        write_le_u16(&mut ram, DOOR_ANIMATION_STEP_INDICATOR, 0xdddd);

        {
            let mut bridge = NativeWorldTransientBridgeMut::new(&mut transient, &mut ram);
            bridge.clear_custom_spell_animation();
        }

        assert_eq!(transient.flag_custom_spell_anim_active(), 0);
        assert_eq!(transient.allow_scroll_z(), 0x02);
        assert_eq!(transient.room_transitioning_flags(), 0x03);
        assert_eq!(transient.door_animation_step(), 0x0405);
        assert_eq!(ram[FLAG_CUSTOM_SPELL_ANIM_ACTIVE], 0);
        assert_eq!(ram[ALLOW_SCROLL_Z], 0x02);
        assert_eq!(ram[ROOM_TRANSITIONING_FLAGS], 0x03);
        assert_eq!(read_le_u16(&ram, DOOR_ANIMATION_STEP_INDICATOR), 0x0405);
    }

    #[test]
    fn world_scroll_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, BG1_X_SCROLL, 0x0101);
        write_le_u16(&mut ram, BG1_Y_SCROLL, 0x0202);
        write_le_u16(&mut ram, BG2_X_SCROLL, 0x0303);
        write_le_u16(&mut ram, BG2_Y_SCROLL, 0x0404);
        write_le_u16(&mut ram, BG1_X_OFFSET, 0x0505);
        write_le_u16(&mut ram, BG1_Y_OFFSET, 0x0606);
        write_le_u16(&mut ram, CAMERA_X, 0x0707);
        write_le_u16(&mut ram, CAMERA_Y, 0x0808);
        write_le_u16(&mut ram, OVERWORLD_OFFSET_BASE_X, 0x0909);
        write_le_u16(&mut ram, OVERWORLD_OFFSET_BASE_Y, 0x0a0a);
        write_le_u16(&mut ram, OVERWORLD_OFFSET_MASK_X, 0x0b0b);
        write_le_u16(&mut ram, OVERWORLD_OFFSET_MASK_Y, 0x0c0c);
        write_le_u16(&mut ram, OVERWORLD_SCROLL_X_START, 0x0d0d);
        write_le_u16(&mut ram, OVERWORLD_SCROLL_X_END, 0x0e0e);
        write_le_u16(&mut ram, OVERWORLD_SCROLL_Y_END, 0x0f0f);

        let mut scroll = WorldScrollState::load_from_ram(&ram);
        assert_eq!(scroll.bg1_x(), 0x0101);
        assert_eq!(scroll.bg1_y(), 0x0202);
        assert_eq!(scroll.bg2_x(), 0x0303);
        assert_eq!(scroll.bg2_y(), 0x0404);
        assert_eq!(scroll.bg1_x_offset(), 0x0505);
        assert_eq!(scroll.bg1_y_offset(), 0x0606);
        assert_eq!(scroll.camera_x(), 0x0707);
        assert_eq!(scroll.camera_y(), 0x0808);
        assert_eq!(scroll.overworld_offset_base_x(), 0x0909);
        assert_eq!(scroll.overworld_offset_base_y(), 0x0a0a);
        assert_eq!(scroll.overworld_offset_mask_x(), 0x0b0b);
        assert_eq!(scroll.overworld_offset_mask_y(), 0x0c0c);
        assert_eq!(scroll.scroll_x_start(), 0x0d0d);
        assert_eq!(scroll.scroll_x_end(), 0x0e0e);
        assert_eq!(scroll.scroll_y_end(), 0x0f0f);

        scroll.bg1_x = 0x1111;
        scroll.bg1_y = 0x2222;
        scroll.bg2_x = 0x3333;
        scroll.bg2_y = 0x4444;
        scroll.bg1_x_offset = 0x5555;
        scroll.bg1_y_offset = 0x6666;
        scroll.camera_x = 0x7777;
        scroll.camera_y = 0x8888;
        scroll.overworld_offset_base_x = 0x9999;
        scroll.overworld_offset_base_y = 0xaaaa;
        scroll.overworld_offset_mask_x = 0xbbbb;
        scroll.overworld_offset_mask_y = 0xcccc;
        scroll.scroll_x_start = 0xdddd;
        scroll.scroll_x_end = 0xeeee;
        scroll.scroll_y_end = 0xffff;
        scroll.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, BG1_X_SCROLL), 0x1111);
        assert_eq!(read_le_u16(&ram, BG1_Y_SCROLL), 0x2222);
        assert_eq!(read_le_u16(&ram, BG2_X_SCROLL), 0x3333);
        assert_eq!(read_le_u16(&ram, BG2_Y_SCROLL), 0x4444);
        assert_eq!(read_le_u16(&ram, BG1_X_OFFSET), 0x5555);
        assert_eq!(read_le_u16(&ram, BG1_Y_OFFSET), 0x6666);
        assert_eq!(read_le_u16(&ram, CAMERA_X), 0x7777);
        assert_eq!(read_le_u16(&ram, CAMERA_Y), 0x8888);
        assert_eq!(read_le_u16(&ram, OVERWORLD_OFFSET_BASE_X), 0x9999);
        assert_eq!(read_le_u16(&ram, OVERWORLD_OFFSET_BASE_Y), 0xaaaa);
        assert_eq!(read_le_u16(&ram, OVERWORLD_OFFSET_MASK_X), 0xbbbb);
        assert_eq!(read_le_u16(&ram, OVERWORLD_OFFSET_MASK_Y), 0xcccc);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCROLL_X_START), 0xdddd);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCROLL_X_END), 0xeeee);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCROLL_Y_END), 0xffff);
    }

    #[test]
    fn native_world_scroll_bridge_projects_native_state_over_stale_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut scroll = WorldScrollState {
            bg1_x: 0x0101,
            bg1_y: 0x0202,
            bg2_x: 0x0303,
            bg2_y: 0x0404,
            bg1_x_offset: 0x0505,
            bg1_y_offset: 0x0606,
            camera_x: 0x0707,
            camera_y: 0x0808,
            overworld_offset_base_x: 0x0909,
            overworld_offset_base_y: 0x0a0a,
            overworld_offset_mask_x: 0x0b0b,
            overworld_offset_mask_y: 0x0c0c,
            scroll_x_start: 0x0d0d,
            scroll_x_end: 0x0e0e,
            scroll_y_end: 0x0f0f,
        };
        scroll.write_to_ram(&mut ram);

        write_le_u16(&mut ram, BG2_X_SCROLL, 0xaaaa);
        write_le_u16(&mut ram, BG2_Y_SCROLL, 0xbbbb);
        write_le_u16(&mut ram, OVERWORLD_OFFSET_BASE_X, 0xcccc);

        {
            let mut bridge = NativeWorldScrollBridgeMut::new(&mut scroll, &mut ram);
            bridge.set_bg2_x(0x1234);
        }

        assert_eq!(scroll.bg2_x(), 0x1234);
        assert_eq!(scroll.bg2_y(), 0x0404);
        assert_eq!(scroll.overworld_offset_base_x(), 0x0909);
        assert_eq!(WorldScrollState::load_from_ram(&ram), scroll);
        assert_eq!(read_le_u16(&ram, BG2_X_SCROLL), 0x1234);
        assert_eq!(read_le_u16(&ram, BG2_Y_SCROLL), 0x0404);
        assert_eq!(read_le_u16(&ram, OVERWORLD_OFFSET_BASE_X), 0x0909);
    }

    #[test]
    fn game_state_loads_grouped_world_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DUNGEON_ROOM, 0x0124);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX, 0x0040);
        ram[PLAYER_IS_INDOORS] = 1;
        write_le_u16(&mut ram, OVERWORLD_MAP_STATE, 0x0206);
        ram[OVERWORLD_MAP_FLAGS] = 0x03;
        write_le_u16(&mut ram, BIRDTRAVEL_STATUS, 0x0004);
        ram[MODE7_ZOOM_STEP_COUNTER] = 2;
        ram[TIMER_FOR_MODE7_ZOOM] = 12;
        write_le_u16(&mut ram, OVERWORLD_AREA_IS_BIG, 0x0120);
        ram[OVERWORLD_AREA_IS_BIG_BACKUP] = 0x20;
        write_le_u16(&mut ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND, 0x03e4);
        ram[OVERWORLD_SCROLL_DELTA] = 0x11;
        ram[OVERWORLD_SCROLL_DELTA + 1] = 0x22;
        ram[OVERWORLD_SCROLL_DELTA + 2] = 0x33;
        ram[BIRD_TRAVEL_X_LO + 3] = 0x34;
        ram[BIRD_TRAVEL_X_HI + 3] = 0x12;
        ram[BIRD_TRAVEL_Y_LO + 3] = 0x78;
        ram[BIRD_TRAVEL_Y_HI + 3] = 0x56;
        write_le_u16(&mut ram, WEATHERVANE_COUNTDOWN, 0x0280);
        ram[WEATHERVANE_MUSIC_LATCH] = 1;
        ram[WEATHERVANE_SOURCE_SLOT] = 7;
        ram[WEATHERVANE_OAM_OFFSET] = 0x10;
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF, 0x1234);
        write_le_u16(&mut ram, MAP16_LOAD_DST_OFF, 0x0056);
        write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT, 0x0007);
        ram[TRIGGER_SPECIAL_ENTRANCE] = 1;
        ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = 3;
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_EXIT, 0x0022);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_SPEXIT, 0x0033);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, 0x0004);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, 0x0008);
        ram[OVERWORLD_TRANSITION_DIR] = 2;
        ram[OVERWORLD_EVENT_INFO + 0x5b] = 0x20;
        write_le_u16(&mut ram, DIALOGUE_MESSAGE_INDEX, 0x0123);
        write_le_u16(&mut ram, MULTISELECT_CHOICE, 0x0204);
        ram[MULTISELECT_CHOICE_BACKUP] = 0x07;
        ram[DIALOGUE_NUMBER_LO] = 0x12;
        ram[DIALOGUE_NUMBER_HI] = 0x34;

        let mut state = GameState::load_from_ram(&ram);
        assert_eq!(state.world.location.dungeon_room, 0x0124);
        assert_eq!(state.world.location.overworld_screen, 0x0040);
        assert_eq!(state.world.location.indoor_flag, 1);
        assert_eq!(state.world.overworld.map_ui.map_state_word(), 0x0206);
        assert_eq!(state.world.overworld.map_zoom.timer, 12);
        assert_eq!(state.world.overworld.screen_size.is_big_area_word(), 0x0120);
        assert!(state.world.overworld.screen_size.is_big_area());
        assert_eq!(
            state.world.overworld.screen_size.right_bottom_bound_word(),
            0x03e4
        );
        assert_eq!(
            state.world.overworld.scroll_delta.vertical_delta_word(),
            0x2211
        );
        assert_eq!(
            state.world.overworld.scroll_delta.horizontal_delta_word(),
            0x3322
        );
        assert_eq!(
            state
                .world
                .overworld
                .bird_travel_destinations
                .destination(3),
            BirdTravelDestinationState {
                x: 0x1234,
                y: 0x5678,
            }
        );
        assert_eq!(
            state.world.overworld.weather_vane,
            WeatherVaneState {
                countdown: 0x0280,
                music_latch: 1,
                source_slot: 7,
                oam_offset: 0x10,
            }
        );
        assert_eq!(state.world.overworld.map16.active_load.src_off, 0x1234);
        assert_eq!(state.world.overworld.entrance.sequence_counter, 3);
        assert_eq!(state.world.overworld.exit.special_exit_screen, 0x0033);
        assert_eq!(
            state.world.overworld.transition.direction_bits_word(),
            0x0008
        );
        assert_eq!(state.world.overworld.event_info.event_info(0x5b), 0x20);
        assert_eq!(state.messaging.dialogue_message_index.value(), 0x0123);
        assert_eq!(state.messaging.multiselect_choice.value(), 0x04);
        assert_eq!(
            MultiselectChoiceRead::new(
                &state.messaging.multiselect_choice,
                &state.messaging.runtime
            )
            .value_word(),
            0x0204
        );
        assert_eq!(state.messaging.multiselect_choice.backup(), 0x07);
        assert_eq!(state.messaging.dialogue_number.packed_digits(0), 0x12);
        assert_eq!(state.messaging.dialogue_number.packed_digits(1), 0x34);

        state.world.location.dungeon_room = 0x0181;
        state.world.location.overworld_screen = 0x005b;
        state.world.location.indoor_flag = 0;
        state.world.overworld.event_info.set_event_bits(0x5b, 0x40);
        state.world.overworld.map_ui.map_flags = 0x81;
        state.world.overworld.map_zoom.timer = 4;
        state.world.overworld.screen_size.big_area = 0x0020;
        state.world.overworld.screen_size.big_area_backup = 0x20;
        state.world.overworld.screen_size.right_bottom_scroll_bound = 0x01e4;
        state
            .world
            .overworld
            .scroll_delta
            .set_vertical_delta_word(0x4433);
        state
            .world
            .overworld
            .scroll_delta
            .set_horizontal_delta_word(0x5544);
        state
            .world
            .overworld
            .bird_travel_destinations
            .set_destination(3, 0x2345, 0x6789);
        state.world.overworld.weather_vane.countdown = 0x0001;
        state.world.overworld.weather_vane.music_latch = 2;
        state.world.overworld.weather_vane.source_slot = 9;
        state.world.overworld.weather_vane.oam_offset = 0x20;
        state.world.overworld.map16.active_load.src_off = 0x4567;
        state.world.overworld.entrance.sequence_counter = 9;
        state.world.overworld.exit.exit_screen = 0x0044;
        state.world.overworld.transition.direction_enum = 3;
        state.messaging.dialogue_message_index.set_value(0x0140);
        state.messaging.multiselect_choice.set_value(0x05);
        state.messaging.multiselect_choice.save_backup();
        state
            .messaging
            .dialogue_number
            .set_packed_digits(0x56, 0x78);
        state.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, DUNGEON_ROOM), 0x0181);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX), 0x005b);
        assert_eq!(ram[PLAYER_IS_INDOORS], 0);
        assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x5b], 0x60);
        assert_eq!(ram[OVERWORLD_MAP_FLAGS], 0x81);
        assert_eq!(ram[TIMER_FOR_MODE7_ZOOM], 4);
        assert_eq!(read_le_u16(&ram, OVERWORLD_AREA_IS_BIG), 0x0020);
        assert_eq!(ram[OVERWORLD_AREA_IS_BIG_BACKUP], 0x20);
        assert_eq!(
            read_le_u16(&ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND),
            0x01e4
        );
        assert_eq!(ram[OVERWORLD_SCROLL_DELTA], 0x33);
        assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 1], 0x44);
        assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 2], 0x55);
        assert_eq!(ram[BIRD_TRAVEL_X_LO + 3], 0x45);
        assert_eq!(ram[BIRD_TRAVEL_X_HI + 3], 0x23);
        assert_eq!(ram[BIRD_TRAVEL_Y_LO + 3], 0x89);
        assert_eq!(ram[BIRD_TRAVEL_Y_HI + 3], 0x67);
        assert_eq!(read_le_u16(&ram, WEATHERVANE_COUNTDOWN), 0x0001);
        assert_eq!(ram[WEATHERVANE_MUSIC_LATCH], 2);
        assert_eq!(ram[WEATHERVANE_SOURCE_SLOT], 9);
        assert_eq!(ram[WEATHERVANE_OAM_OFFSET], 0x20);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF), 0x4567);
        assert_eq!(ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER], 9);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_EXIT), 0x0044);
        assert_eq!(ram[OVERWORLD_TRANSITION_DIR], 3);
        assert_eq!(read_le_u16(&ram, DIALOGUE_MESSAGE_INDEX), 0x0140);
        assert_eq!(read_le_u16(&ram, MULTISELECT_CHOICE), 0x0205);
        assert_eq!(ram[MULTISELECT_CHOICE_BACKUP], 0x05);
        assert_eq!(ram[DIALOGUE_NUMBER_LO], 0x56);
        assert_eq!(ram[DIALOGUE_NUMBER_HI], 0x78);
    }

    #[test]
    fn overworld_event_info_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[OVERWORLD_EVENT_INFO + 0x02] = 0x40;
        ram[OVERWORLD_EVENT_INFO + 0x5b] = 0x20;
        ram[OVERWORLD_EVENT_INFO + 0x9f] = 0x02;

        let mut event_info = OverworldEventInfoState::load_from_ram(&ram);
        assert_eq!(event_info.event_info(0x02), 0x40);
        assert_eq!(event_info.event_info(0x5b), 0x20);
        assert_eq!(event_info.event_info(0x9f), 0x02);
        assert_eq!(event_info.event_info(0xa0), 0);
        assert!(event_info.has_event_bits(0x5b, 0x20));

        event_info.set_event_info(0x02, 0x10);
        event_info.set_event_bits(0x5b, 0x40);
        event_info.clear_event_bits(0x9f, 0x02);
        event_info.write_to_ram(&mut ram);

        assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x02], 0x10);
        assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x5b], 0x60);
        assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x9f], 0);
    }

    #[test]
    fn native_overworld_event_info_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[OVERWORLD_EVENT_INFO + 0x02] = 0x40;
        ram[OVERWORLD_EVENT_INFO + 0x5b] = 0x20;
        ram[OVERWORLD_EVENT_INFO + 0x9f] = 0x02;

        let mut event_info = OverworldEventInfoState::load_from_ram(&ram);
        {
            let mut bridge = NativeOverworldEventInfoBridgeMut::new(&mut event_info, &mut ram);
            bridge.set_event_info(0x02, 0x10);
            bridge.set_event_bits(0x5b, 0x40);
            bridge.clear_event_bits(0x9f, 0x02);
        }

        assert_eq!(event_info.event_info(0x02), 0x10);
        assert_eq!(event_info.event_info(0x5b), 0x60);
        assert_eq!(event_info.event_info(0x9f), 0);
        assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x02], 0x10);
        assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x5b], 0x60);
        assert_eq!(ram[OVERWORLD_EVENT_INFO + 0x9f], 0);
    }

    #[test]
    fn overworld_config_table_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[OVERWORLD_MUSIC_TABLE + 0x02] = 0x31;
        ram[OVERWORLD_MUSIC_TABLE + 0x80] = 0x42;
        ram[OVERWORLD_SPRITE_PALETTE_TABLE + 0x02] = 0x05;
        ram[OVERWORLD_SPRITE_GFX_TABLE + 0x02] = 0x18;

        let mut config_table = OverworldConfigTableState::load_from_ram(&ram);
        assert_eq!(config_table.music(0x02), 0x31);
        assert_eq!(config_table.music(0x80), 0x42);
        assert_eq!(config_table.sprite_palette(0x02), 0x05);
        assert_eq!(config_table.sprite_graphics(0x02), 0x18);
        assert_eq!(config_table.music(0xa0), 0);

        config_table.set_music(0x02, 0x6a);
        config_table.write_to_ram(&mut ram);

        assert_eq!(ram[OVERWORLD_MUSIC_TABLE + 0x02], 0x6a);
        assert_eq!(ram[OVERWORLD_MUSIC_TABLE + 0x80], 0x42);
        assert_eq!(ram[OVERWORLD_SPRITE_PALETTE_TABLE + 0x02], 0x05);
        assert_eq!(ram[OVERWORLD_SPRITE_GFX_TABLE + 0x02], 0x18);
    }

    #[test]
    fn native_overworld_config_table_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[OVERWORLD_MUSIC_TABLE + 0x02] = 0x31;
        ram[OVERWORLD_MUSIC_TABLE + 0x80] = 0x42;
        ram[OVERWORLD_SPRITE_PALETTE_TABLE + 0x02] = 0x05;
        ram[OVERWORLD_SPRITE_GFX_TABLE + 0x02] = 0x18;

        let primary = [0x24; 64];
        let secondary = [0x46; 96];
        let mut config_table = OverworldConfigTableState::load_from_ram(&ram);
        {
            let mut bridge = NativeOverworldConfigTableBridgeMut::new(&mut config_table, &mut ram);
            bridge.copy_music_primary(&primary);
            bridge.copy_music_secondary(&secondary);
            bridge.set_music(0x02, 0x6a);
            bridge.set_music(0x80, 0x7b);
        }

        assert_eq!(config_table.music(0), 0x24);
        assert_eq!(config_table.music(0x02), 0x6a);
        assert_eq!(config_table.music(0x40), 0x46);
        assert_eq!(config_table.music(0x80), 0x7b);
        assert_eq!(config_table.sprite_palette(0x02), 0x05);
        assert_eq!(config_table.sprite_graphics(0x02), 0x18);
        assert_eq!(ram[OVERWORLD_MUSIC_TABLE], 0x24);
        assert_eq!(ram[OVERWORLD_MUSIC_TABLE + 0x02], 0x6a);
        assert_eq!(ram[OVERWORLD_MUSIC_TABLE + 0x40], 0x46);
        assert_eq!(ram[OVERWORLD_MUSIC_TABLE + 0x80], 0x7b);
        assert_eq!(ram[OVERWORLD_SPRITE_PALETTE_TABLE + 0x02], 0x05);
        assert_eq!(ram[OVERWORLD_SPRITE_GFX_TABLE + 0x02], 0x18);
    }

    #[test]
    fn overworld_map_ui_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_MAP_STATE, 0x0205);
        ram[OVERWORLD_MAP_FLAGS] = 0x81;
        write_le_u16(&mut ram, BIRDTRAVEL_STATUS, 0x0307);
        ram[BIRD_TRAVEL_STATUS + 15] = 0xaa;

        let mut map_ui = OverworldMapUiState::load_from_ram(&ram);
        assert_eq!(map_ui.map_state(), 5);
        assert_eq!(map_ui.map_state_word(), 0x0205);
        assert_eq!(map_ui.map_flags, 0x81);
        assert_eq!(map_ui.birdtravel_status(), 7);
        assert_eq!(map_ui.birdtravel_status_word(), 0x0307);
        assert_eq!(map_ui.bird_travel_statuses.status(15), 0xaa);

        map_ui.map_state = 0x0104;
        map_ui.map_flags = 0x40;
        map_ui.bird_travel_statuses.set_status_word(0x0008);
        map_ui.bird_travel_statuses.set_status(15, 0x55);
        map_ui.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, OVERWORLD_MAP_STATE), 0x0104);
        assert_eq!(ram[OVERWORLD_MAP_FLAGS], 0x40);
        assert_eq!(read_le_u16(&ram, BIRDTRAVEL_STATUS), 0x0008);
        assert_eq!(ram[BIRD_TRAVEL_STATUS + 15], 0x55);
    }

    #[test]
    fn native_overworld_map_ui_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_MAP_STATE, 0x0205);
        ram[OVERWORLD_MAP_FLAGS] = 0x81;
        write_le_u16(&mut ram, BIRDTRAVEL_STATUS, 0x0307);
        ram[BIRD_TRAVEL_STATUS + 15] = 0xfe;

        let mut map_ui = OverworldMapUiState::load_from_ram(&ram);
        {
            let mut bridge = NativeOverworldMapUiBridgeMut::new(&mut map_ui, &mut ram);
            bridge.increment_map_state();
            bridge.and_map_flags(!0x80);
            bridge.or_map_flags(0x02);
            bridge.increment_birdtravel_status();
            bridge.and_birdtravel_status(7);
            bridge.set_birdtravel_status_word(0x0004);
            bridge.increment_bird_travel_stop_status(15);
            bridge.clear_bird_travel_stop_status(1);
        }

        assert_eq!(map_ui.map_state_word(), 0x0206);
        assert_eq!(map_ui.map_flags, 0x03);
        assert_eq!(map_ui.birdtravel_status_word(), 0x0004);
        assert_eq!(map_ui.bird_travel_statuses.status(15), 0xff);
        assert_eq!(map_ui.bird_travel_statuses.status(1), 0);
        assert_eq!(read_le_u16(&ram, OVERWORLD_MAP_STATE), 0x0206);
        assert_eq!(ram[OVERWORLD_MAP_FLAGS], 0x03);
        assert_eq!(read_le_u16(&ram, BIRDTRAVEL_STATUS), 0x0004);
        assert_eq!(ram[BIRD_TRAVEL_STATUS + 15], 0xff);
        assert_eq!(ram[BIRD_TRAVEL_STATUS + 1], 0);
    }

    #[test]
    fn weather_vane_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, WEATHERVANE_COUNTDOWN, 0x0280);
        ram[WEATHERVANE_MUSIC_LATCH] = 3;
        ram[WEATHERVANE_SOURCE_SLOT] = 4;
        ram[WEATHERVANE_OAM_OFFSET] = 0x10;

        let mut weather_vane = WeatherVaneState::load_from_ram(&ram);
        assert_eq!(
            weather_vane,
            WeatherVaneState {
                countdown: 0x0280,
                music_latch: 3,
                source_slot: 4,
                oam_offset: 0x10,
            }
        );

        assert_eq!(weather_vane.tick_countdown(), 0x027f);
        weather_vane.reset_oam_offset();
        weather_vane.advance_oam_offset(0xfc);
        weather_vane.music_latch = 1;
        weather_vane.source_slot = 7;
        weather_vane.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, WEATHERVANE_COUNTDOWN), 0x027f);
        assert_eq!(ram[WEATHERVANE_MUSIC_LATCH], 1);
        assert_eq!(ram[WEATHERVANE_SOURCE_SLOT], 7);
        assert_eq!(ram[WEATHERVANE_OAM_OFFSET], 0xfc);
    }

    #[test]
    fn native_weather_vane_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut weather_vane = WeatherVaneState::default();
        {
            let mut bridge = NativeWeatherVaneBridgeMut::new(&mut weather_vane, &mut ram);
            assert_eq!(bridge.tick_countdown(), 0xffff);
            bridge.set_countdown(0x0280);
            bridge.set_music_latch(1);
            bridge.set_source_slot(5);
            bridge.reset_oam_offset();
            bridge.advance_oam_offset(4);
        }

        assert_eq!(
            weather_vane,
            WeatherVaneState {
                countdown: 0x0280,
                music_latch: 1,
                source_slot: 5,
                oam_offset: 4,
            }
        );
        assert_eq!(read_le_u16(&ram, WEATHERVANE_COUNTDOWN), 0x0280);
        assert_eq!(ram[WEATHERVANE_MUSIC_LATCH], 1);
        assert_eq!(ram[WEATHERVANE_SOURCE_SLOT], 5);
        assert_eq!(ram[WEATHERVANE_OAM_OFFSET], 4);
    }

    #[test]
    fn native_weather_vane_bridge_projects_native_state_over_stale_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut weather_vane = WeatherVaneState {
            countdown: 0x0102,
            music_latch: 3,
            source_slot: 4,
            oam_offset: 5,
        };
        weather_vane.write_to_ram(&mut ram);

        write_le_u16(&mut ram, WEATHERVANE_COUNTDOWN, 0xaaaa);
        ram[WEATHERVANE_MUSIC_LATCH] = 0xbb;
        ram[WEATHERVANE_SOURCE_SLOT] = 0xcc;
        ram[WEATHERVANE_OAM_OFFSET] = 0xdd;

        {
            let mut bridge = NativeWeatherVaneBridgeMut::new(&mut weather_vane, &mut ram);
            bridge.set_music_latch(7);
        }

        assert_eq!(
            weather_vane,
            WeatherVaneState {
                countdown: 0x0102,
                music_latch: 7,
                source_slot: 4,
                oam_offset: 5,
            }
        );
        assert_eq!(WeatherVaneState::load_from_ram(&ram), weather_vane);
    }

    #[test]
    fn bird_travel_destinations_load_from_and_project_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[BIRD_TRAVEL_X_LO + 2] = 0x34;
        ram[BIRD_TRAVEL_X_HI + 2] = 0x12;
        ram[BIRD_TRAVEL_Y_LO + 2] = 0x78;
        ram[BIRD_TRAVEL_Y_HI + 2] = 0x56;

        let mut destinations = BirdTravelDestinationsState::load_from_ram(&ram);
        assert_eq!(
            destinations.destination(2),
            BirdTravelDestinationState {
                x: 0x1234,
                y: 0x5678,
            }
        );
        assert!(!destinations.destination(2).is_empty());
        assert!(destinations.destination(3).is_empty());

        destinations.set_destination(2, 0x2345, 0x6789);
        destinations.clear_destination(3);
        destinations.write_to_ram(&mut ram);

        assert_eq!(ram[BIRD_TRAVEL_X_LO + 2], 0x45);
        assert_eq!(ram[BIRD_TRAVEL_X_HI + 2], 0x23);
        assert_eq!(ram[BIRD_TRAVEL_Y_LO + 2], 0x89);
        assert_eq!(ram[BIRD_TRAVEL_Y_HI + 2], 0x67);
        assert_eq!(ram[BIRD_TRAVEL_X_LO + 3], 0);
        assert_eq!(ram[BIRD_TRAVEL_X_HI + 3], 0);
        assert_eq!(ram[BIRD_TRAVEL_Y_LO + 3], 0);
        assert_eq!(ram[BIRD_TRAVEL_Y_HI + 3], 0);
    }

    #[test]
    fn native_bird_travel_destination_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut destinations = BirdTravelDestinationsState::default();
        {
            let mut bridge = NativeBirdTravelDestinationBridgeMut::new(&mut destinations, &mut ram);
            bridge.set_destination(15, 0x2345, 0x6789);
            bridge.clear_destination(2);
        }

        assert_eq!(
            destinations.destination(15),
            BirdTravelDestinationState {
                x: 0x2345,
                y: 0x6789,
            }
        );
        assert!(destinations.destination(2).is_empty());
        assert_eq!(ram[BIRD_TRAVEL_X_LO + 15], 0x45);
        assert_eq!(ram[BIRD_TRAVEL_X_HI + 15], 0x23);
        assert_eq!(ram[BIRD_TRAVEL_Y_LO + 15], 0x89);
        assert_eq!(ram[BIRD_TRAVEL_Y_HI + 15], 0x67);
        assert_eq!(ram[BIRD_TRAVEL_X_LO + 2], 0);
        assert_eq!(ram[BIRD_TRAVEL_X_HI + 2], 0);
        assert_eq!(ram[BIRD_TRAVEL_Y_LO + 2], 0);
        assert_eq!(ram[BIRD_TRAVEL_Y_HI + 2], 0);
    }

    #[test]
    fn native_bird_travel_destination_bridge_projects_native_state_over_stale_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut destinations = BirdTravelDestinationsState::default();
        destinations.set_destination(15, 0x1234, 0x5678);
        destinations.write_to_ram(&mut ram);

        ram[BIRD_TRAVEL_X_LO + 15] = 0xaa;
        ram[BIRD_TRAVEL_X_HI + 15] = 0xbb;
        ram[BIRD_TRAVEL_Y_LO + 15] = 0xcc;
        ram[BIRD_TRAVEL_Y_HI + 15] = 0xdd;

        {
            let mut bridge = NativeBirdTravelDestinationBridgeMut::new(&mut destinations, &mut ram);
            bridge.clear_destination(2);
        }

        assert_eq!(
            destinations.destination(15),
            BirdTravelDestinationState {
                x: 0x1234,
                y: 0x5678,
            }
        );
        assert_eq!(
            BirdTravelDestinationsState::load_from_ram(&ram),
            destinations
        );
    }

    #[test]
    fn overworld_map_zoom_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MODE7_ZOOM_STEP_COUNTER] = 4;
        ram[TIMER_FOR_MODE7_ZOOM] = 12;

        let mut zoom = OverworldMapZoomState::load_from_ram(&ram);
        assert_eq!(zoom.step_counter, 4);
        assert_eq!(zoom.timer, 12);

        zoom.step_counter = 7;
        zoom.timer = 33;
        zoom.write_to_ram(&mut ram);

        assert_eq!(ram[MODE7_ZOOM_STEP_COUNTER], 7);
        assert_eq!(ram[TIMER_FOR_MODE7_ZOOM], 33);
    }

    #[test]
    fn native_overworld_map_zoom_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut zoom = OverworldMapZoomState::default();
        {
            let mut bridge = NativeOverworldMapZoomBridgeMut::new(&mut zoom, &mut ram);
            bridge.set_step_counter(4);
            bridge.decrement_timer();
            bridge.set_timer(12);
        }

        assert_eq!(zoom.step_counter, 4);
        assert_eq!(zoom.timer, 12);
        assert_eq!(ram[MODE7_ZOOM_STEP_COUNTER], 4);
        assert_eq!(ram[TIMER_FOR_MODE7_ZOOM], 12);
    }

    #[test]
    fn native_overworld_map_zoom_bridge_projects_native_state_over_stale_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut zoom = OverworldMapZoomState {
            step_counter: 2,
            timer: 8,
        };
        zoom.write_to_ram(&mut ram);

        ram[MODE7_ZOOM_STEP_COUNTER] = 0xaa;
        ram[TIMER_FOR_MODE7_ZOOM] = 0xbb;

        {
            let mut bridge = NativeOverworldMapZoomBridgeMut::new(&mut zoom, &mut ram);
            bridge.decrement_timer();
        }

        assert_eq!(
            zoom,
            OverworldMapZoomState {
                step_counter: 2,
                timer: 7,
            }
        );
        assert_eq!(OverworldMapZoomState::load_from_ram(&ram), zoom);
    }

    #[test]
    fn overworld_screen_size_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_AREA_IS_BIG, 0x0120);
        ram[OVERWORLD_AREA_IS_BIG_BACKUP] = 0x20;
        write_le_u16(&mut ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND, 0x03e4);

        let mut screen_size = OverworldScreenSizeState::load_from_ram(&ram);
        assert_eq!(screen_size.is_big_area_word(), 0x0120);
        assert!(screen_size.is_big_area());
        assert_eq!(screen_size.big_area_backup, 0x20);
        assert_eq!(screen_size.right_bottom_bound_word(), 0x03e4);

        screen_size.big_area = 0x0020;
        screen_size.big_area_backup = 0x20;
        screen_size.right_bottom_scroll_bound = 0x01e4;
        screen_size.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, OVERWORLD_AREA_IS_BIG), 0x0020);
        assert_eq!(ram[OVERWORLD_AREA_IS_BIG_BACKUP], 0x20);
        assert_eq!(
            read_le_u16(&ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND),
            0x01e4
        );
    }

    #[test]
    fn native_overworld_screen_size_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_AREA_IS_BIG, 0x0120);
        ram[OVERWORLD_AREA_IS_BIG_BACKUP] = 0x11;
        write_le_u16(&mut ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND, 0x02c0);

        let mut screen_size = OverworldScreenSizeState::load_from_ram(&ram);
        {
            let mut bridge = NativeOverworldScreenSizeBridgeMut::new(&mut screen_size, &mut ram);
            bridge.backup_big_area_low();
            bridge.clear_big_area_high();
            bridge.set_big_area_low(0x20);
            bridge.set_right_bottom_bound_low(0xe4);
            bridge.set_right_bottom_bound_high(0x01);
        }

        assert_eq!(screen_size.big_area, 0x0020);
        assert_eq!(screen_size.big_area_backup, 0x20);
        assert_eq!(screen_size.right_bottom_scroll_bound, 0x01e4);
        assert_eq!(read_le_u16(&ram, OVERWORLD_AREA_IS_BIG), 0x0020);
        assert_eq!(ram[OVERWORLD_AREA_IS_BIG_BACKUP], 0x20);
        assert_eq!(
            read_le_u16(&ram, OVERWORLD_RIGHT_BOTTOM_SCROLL_BOUND),
            0x01e4
        );
    }

    #[test]
    fn overworld_scroll_delta_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[OVERWORLD_SCROLL_DELTA] = 0x11;
        ram[OVERWORLD_SCROLL_DELTA + 1] = 0x22;
        ram[OVERWORLD_SCROLL_DELTA + 2] = 0x33;

        let mut scroll_delta = OverworldScrollDeltaState::load_from_ram(&ram);
        assert_eq!(scroll_delta.vertical_delta_low_byte(), 0x11);
        assert_eq!(scroll_delta.horizontal_delta_low_byte(), 0x22);
        assert_eq!(scroll_delta.vertical_delta_word(), 0x2211);
        assert_eq!(scroll_delta.horizontal_delta_word(), 0x3322);

        scroll_delta.set_vertical_delta_word(0x4433);
        scroll_delta.set_horizontal_delta_word(0x5544);
        scroll_delta.write_to_ram(&mut ram);

        assert_eq!(ram[OVERWORLD_SCROLL_DELTA], 0x33);
        assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 1], 0x44);
        assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 2], 0x55);
    }

    #[test]
    fn native_overworld_scroll_delta_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[OVERWORLD_SCROLL_DELTA] = 0x11;
        ram[OVERWORLD_SCROLL_DELTA + 1] = 0x22;
        ram[OVERWORLD_SCROLL_DELTA + 2] = 0x33;

        let mut scroll_delta = OverworldScrollDeltaState::load_from_ram(&ram);
        {
            let mut bridge = NativeOverworldScrollDeltaBridgeMut::new(&mut scroll_delta, &mut ram);
            bridge.set_vertical_delta_low_byte(0x44);
            bridge.set_horizontal_delta_low_byte(0x55);
            bridge.set_vertical_delta_word(0x6677);
            bridge.set_horizontal_delta_word(0x8899);
            bridge.clear_vertical_delta_low_byte();
        }

        assert_eq!(scroll_delta.vertical_delta_low_byte(), 0);
        assert_eq!(scroll_delta.horizontal_delta_low_byte(), 0x99);
        assert_eq!(scroll_delta.vertical_delta_word(), 0x9900);
        assert_eq!(scroll_delta.horizontal_delta_word(), 0x8899);
        assert_eq!(ram[OVERWORLD_SCROLL_DELTA], 0);
        assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 1], 0x99);
        assert_eq!(ram[OVERWORLD_SCROLL_DELTA + 2], 0x88);
    }

    #[test]
    fn overworld_map16_load_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF, 0x1234);
        write_le_u16(&mut ram, MAP16_LOAD_DST_OFF, 0x0056);
        write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT, 0x0007);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_PREV, 0x2345);
        write_le_u16(&mut ram, MAP16_LOAD_DST_OFF_PREV, 0x0067);
        write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT_PREV, 0x0008);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_SPEXIT, 0x3456);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_EXIT, 0x4567);
        write_le_u16(&mut ram, ORANGE_BLUE_BARRIER_STATE, 0x5678);
        write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF, 0x0079);
        write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT, 0x000a);

        let mut map16 = OverworldMap16State::load_from_ram(&ram);
        assert_eq!(map16.active_load.src_off, 0x1234);
        assert_eq!(map16.active_load.dst_off, 0x0056);
        assert_eq!(map16.active_load.y_unit, 0x0007);
        assert_eq!(map16.previous_load.src_off, 0x2345);
        assert_eq!(map16.previous_load.dst_off, 0x0067);
        assert_eq!(map16.previous_load.y_unit, 0x0008);
        assert_eq!(map16.special_exit_src_off, 0x3456);
        assert_eq!(map16.exit_src_off, 0x4567);
        assert_eq!(map16.small_scroll_backup.src_off, 0x5678);
        assert_eq!(map16.small_scroll_backup.dst_off, 0x0079);
        assert_eq!(map16.small_scroll_backup.y_unit, 0x000a);

        map16.active_load.src_off = 0x2222;
        map16.active_load.dst_off = 0x0034;
        map16.active_load.y_unit = 0x0009;
        map16.previous_load.src_off = 0x3333;
        map16.previous_load.dst_off = 0x0045;
        map16.previous_load.y_unit = 0x000b;
        map16.special_exit_src_off = 0x4444;
        map16.exit_src_off = 0x5555;
        map16.small_scroll_backup = SmallOverworldMap16ScrollBackupState {
            src_off: 0x6666,
            dst_off: 0x0056,
            y_unit: 0x000c,
        };
        map16.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF), 0x2222);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF), 0x0034);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT), 0x0009);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_PREV), 0x3333);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF_PREV), 0x0045);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT_PREV), 0x000b);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_SPEXIT), 0x4444);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_EXIT), 0x5555);
        assert_eq!(read_le_u16(&ram, ORANGE_BLUE_BARRIER_STATE), 0x6666);
        assert_eq!(
            read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF),
            0x0056
        );
        assert_eq!(
            read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT),
            0x000c
        );
    }

    #[test]
    fn native_overworld_map16_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF, 0x1234);
        write_le_u16(&mut ram, MAP16_LOAD_DST_OFF, 0x0056);
        write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT, 0x0007);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_PREV, 0x2345);
        write_le_u16(&mut ram, MAP16_LOAD_DST_OFF_PREV, 0x0067);
        write_le_u16(&mut ram, MAP16_LOAD_Y_UNIT_PREV, 0x0008);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_SPEXIT, 0x3456);
        write_le_u16(&mut ram, MAP16_LOAD_SRC_OFF_EXIT, 0x4567);
        write_le_u16(&mut ram, ORANGE_BLUE_BARRIER_STATE, 0x5678);
        write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF, 0x0079);
        write_le_u16(&mut ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT, 0x000a);

        let mut map16 = OverworldMap16State::load_from_ram(&ram);
        {
            let mut bridge = NativeOverworldMap16BridgeMut::new(&mut map16, &mut ram);
            bridge.set_active_load(OverworldMap16LoadState {
                src_off: 0x3456,
                dst_off: 0x0078,
                y_unit: 0x000a,
            });
            bridge.set_previous_load(OverworldMap16LoadState {
                src_off: 0x4567,
                dst_off: 0x0089,
                y_unit: 0x000b,
            });
            bridge.set_special_exit_src_off(0x5678);
            bridge.set_exit_src_off(0x6789);
            bridge.set_small_scroll_backup(SmallOverworldMap16ScrollBackupState {
                src_off: 0x789a,
                dst_off: 0x009b,
                y_unit: 0x000c,
            });
        }

        assert_eq!(map16.active_load.src_off, 0x3456);
        assert_eq!(map16.active_load.dst_off, 0x0078);
        assert_eq!(map16.active_load.y_unit, 0x000a);
        assert_eq!(map16.previous_load.src_off, 0x4567);
        assert_eq!(map16.previous_load.dst_off, 0x0089);
        assert_eq!(map16.previous_load.y_unit, 0x000b);
        assert_eq!(map16.special_exit_src_off, 0x5678);
        assert_eq!(map16.exit_src_off, 0x6789);
        assert_eq!(map16.small_scroll_backup.src_off, 0x789a);
        assert_eq!(map16.small_scroll_backup.dst_off, 0x009b);
        assert_eq!(map16.small_scroll_backup.y_unit, 0x000c);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF), 0x3456);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF), 0x0078);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT), 0x000a);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_PREV), 0x4567);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_DST_OFF_PREV), 0x0089);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_Y_UNIT_PREV), 0x000b);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_SPEXIT), 0x5678);
        assert_eq!(read_le_u16(&ram, MAP16_LOAD_SRC_OFF_EXIT), 0x6789);
        assert_eq!(read_le_u16(&ram, ORANGE_BLUE_BARRIER_STATE), 0x789a);
        assert_eq!(
            read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_DST_OFF),
            0x009b
        );
        assert_eq!(
            read_le_u16(&ram, SMALL_OW_SCROLL_BACKUP_MAP16_Y_UNIT),
            0x000c
        );
    }

    #[test]
    fn overworld_entrance_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TRIGGER_SPECIAL_ENTRANCE] = 5;
        ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = 9;

        let mut entrance = OverworldEntranceState::load_from_ram(&ram);
        assert_eq!(entrance.special_entrance_trigger, 5);
        assert_eq!(entrance.sequence_counter, 9);

        entrance.special_entrance_trigger = 2;
        entrance.sequence_counter = 7;
        entrance.write_to_ram(&mut ram);

        assert_eq!(ram[TRIGGER_SPECIAL_ENTRANCE], 2);
        assert_eq!(ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER], 7);
    }

    #[test]
    fn native_overworld_entrance_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TRIGGER_SPECIAL_ENTRANCE] = 5;
        ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER] = 0xff;

        let mut entrance = OverworldEntranceState::load_from_ram(&ram);
        {
            let mut bridge = NativeOverworldEntranceBridgeMut::new(&mut entrance, &mut ram);
            bridge.set_special_entrance_trigger(3);
            assert_eq!(bridge.increment_sequence_counter(), 0);
            assert_eq!(bridge.decrement_sequence_counter(), 0xff);
            bridge.clear_special_entrance_trigger();
            bridge.clear_sequence_counter();
        }

        assert_eq!(entrance.special_entrance_trigger, 0);
        assert_eq!(entrance.sequence_counter, 0);
        assert_eq!(ram[TRIGGER_SPECIAL_ENTRANCE], 0);
        assert_eq!(ram[OVERWORLD_ENTRANCE_SEQUENCE_COUNTER], 0);
    }

    #[test]
    fn overworld_exit_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_EXIT, 0x0123);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_SPEXIT, 0x0045);

        let mut exit = OverworldExitState::load_from_ram(&ram);
        assert_eq!(exit.exit_screen, 0x0123);
        assert_eq!(exit.special_exit_screen, 0x0045);

        exit.exit_screen = 0x0067;
        exit.special_exit_screen = 0x0089;
        exit.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_EXIT), 0x0067);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_SPEXIT), 0x0089);
    }

    #[test]
    fn native_overworld_exit_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_EXIT, 0x0111);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_INDEX_SPEXIT, 0x0222);

        let mut exit = OverworldExitState::load_from_ram(&ram);
        {
            let mut bridge = NativeOverworldExitBridgeMut::new(&mut exit, &mut ram);
            bridge.set_exit_screen(0x0033);
            bridge.set_special_exit_screen(0x0044);
        }

        assert_eq!(exit.exit_screen, 0x0033);
        assert_eq!(exit.special_exit_screen, 0x0044);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_EXIT), 0x0033);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_INDEX_SPEXIT), 0x0044);
    }

    #[test]
    fn overworld_transition_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, 0x0302);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, 0x0108);
        ram[OVERWORLD_TRANSITION_DIR] = 6;
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANSITION, 0x0203);
        ram[TRANSITION_COUNTER] = 9;
        ram[OW_COUNTDOWN_TRANSITION] = 12;
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV, 0x0004);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV, 0x0002);
        ram[OVERWORLD_SCREEN_TRANSITION_PREV] = 7;

        let mut transition = OverworldTransitionState::load_from_ram(&ram);
        assert_eq!(transition.edge_direction_bits(), 2);
        assert_eq!(transition.edge_direction_bits, 0x0302);
        assert_eq!(transition.direction_bits(), 8);
        assert_eq!(transition.direction_bits_word(), 0x0108);
        assert_eq!(transition.direction_enum(), 6);
        assert!(transition.has_direction_bits());
        assert_eq!(transition.screen_transition(), 3);
        assert_eq!(transition.screen_transition_word(), 0x0203);
        assert_eq!(transition.transition_counter, 9);
        assert_eq!(transition.countdown(), 12);
        assert_eq!(transition.previous_direction_bits, 4);
        assert_eq!(transition.previous_direction_bits2, 2);
        assert_eq!(transition.previous_screen_transition, 7);

        transition.edge_direction_bits = 0x0003;
        transition.direction_bits = 0x0001;
        transition.direction_enum = 4;
        transition.screen_transition = 0x0002;
        transition.transition_counter = 5;
        transition.countdown = 11;
        transition.previous_direction_bits = 0x0008;
        transition.previous_direction_bits2 = 0x0004;
        transition.previous_screen_transition = 6;
        transition.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS), 3);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2), 1);
        assert_eq!(ram[OVERWORLD_TRANSITION_DIR], 4);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANSITION), 2);
        assert_eq!(ram[TRANSITION_COUNTER], 5);
        assert_eq!(ram[OW_COUNTDOWN_TRANSITION], 11);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV), 8);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV), 4);
        assert_eq!(ram[OVERWORLD_SCREEN_TRANSITION_PREV], 6);
    }

    #[test]
    fn native_overworld_transition_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS, 0x0102);
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2, 0x0108);
        ram[OVERWORLD_TRANSITION_DIR] = 6;
        write_le_u16(&mut ram, OVERWORLD_SCREEN_TRANSITION, 0x0203);
        ram[TRANSITION_COUNTER] = 9;
        ram[OW_COUNTDOWN_TRANSITION] = 1;

        let mut transition = OverworldTransitionState::load_from_ram(&ram);
        {
            let mut bridge = NativeOverworldTransitionBridgeMut::new(&mut transition, &mut ram);
            bridge.and_direction_bits(0x0b);
            bridge.or_direction_bits(0x04);
            assert_eq!(bridge.or_direction_bits_word(0x0100), 0x010c);
            bridge.set_direction_enum(4);
            bridge.set_screen_transition(5);
            bridge.increment_transition_counter();
            assert_eq!(bridge.decrement_countdown(), 0);
            bridge.set_countdown(12);
            bridge.save_previous_direction_bits();
            bridge.set_edge_direction_bits(0x04);
            bridge.clear_direction_bits_word();
            bridge.restore_previous_direction_bits();
            bridge.set_previous_screen_transition(6);
        }

        assert_eq!(transition.edge_direction_bits(), 2);
        assert_eq!(transition.edge_direction_bits, 2);
        assert_eq!(transition.direction_bits_word(), 0x010c);
        assert_eq!(transition.direction_enum(), 4);
        assert_eq!(transition.screen_transition_word(), 0x0205);
        assert_eq!(transition.transition_counter, 10);
        assert_eq!(transition.countdown(), 12);
        assert_eq!(transition.previous_direction_bits, 2);
        assert_eq!(transition.previous_direction_bits2, 0x010c);
        assert_eq!(transition.previous_screen_transition, 6);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS), 2);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2), 0x010c);
        assert_eq!(ram[OVERWORLD_TRANSITION_DIR], 4);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANSITION), 0x0205);
        assert_eq!(ram[TRANSITION_COUNTER], 10);
        assert_eq!(ram[OW_COUNTDOWN_TRANSITION], 12);
        assert_eq!(read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS_PREV), 2);
        assert_eq!(
            read_le_u16(&ram, OVERWORLD_SCREEN_TRANS_DIR_BITS2_PREV),
            0x010c
        );
        assert_eq!(ram[OVERWORLD_SCREEN_TRANSITION_PREV], 6);
    }

    #[test]
    fn display_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[INIDISP_COPY] = 0x0f;
        ram[NMI_BOOLEAN] = 1;
        ram[NMI_DISABLE_CORE_UPDATES] = 4;
        ram[NMI_SUBROUTINE_INDEX] = 11;
        ram[NMI_LOAD_BG_FROM_VRAM] = 3;
        ram[NMI_UPDATE_TILEMAP_DST] = 0x50;
        write_le_u16(&mut ram, NMI_UPDATE_TILEMAP_SRC, 0x0200);
        ram[BGMODE_COPY] = 7;
        ram[TM_COPY] = 0x16;
        ram[TS_COPY] = 0x01;
        ram[W12SEL_COPY] = 0x33;
        ram[W34SEL_COPY] = 3;
        ram[WOBJSEL_COPY] = 0xb0;
        ram[TMW_COPY] = 0x16;
        ram[TSW_COPY] = 1;
        ram[NMI_COPY_PACKETS_FLAG] = 1;
        ram[NMI_FLAG_UPDATE_POLYHEDRAL] = 0xff;
        ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 9;
        ram[NMI_THREAD_ACTIVE] = 1;
        write_le_u16(&mut ram, POLY_THREAD_STACK, 0x01f2);
        ram[IRQ_FLAG] = 0x80;
        ram[VIRQ_TRIGGER] = 0x90;
        ram[DMA_HEAD_POINTER] = 0x20;
        ram[DMA_BODY_POINTER] = 0xa0;
        ram[OAM_BUF] = 0xca;
        ram[OAM_BUF + 1] = 0xfe;
        ram[HDMAEN_COPY] = 0xc0;
        ram[MOSAIC_COPY] = 0x73;
        ram[MOSAIC_LEVEL] = 0x70;
        ram[MOSAIC_TARGET_LEVEL] = 0x1f;
        ram[MOSAIC_INC_OR_DEC] = 1;
        write_le_u16(&mut ram, NMI_LOAD_TARGET_ADDR, 0x2146);
        write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x0124);
        ram[crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF] = 0xfa;
        ram[crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF + 1] = 0xce;
        write_le_u16(
            &mut ram,
            crate::game_state::constants::nmi::OVERWORLD_TILE_ATTR_BUFFER + 6,
            0x4567,
        );
        ram[crate::game_state::constants::nmi::STRIPE_BUFFER_021B] = 0x56;
        ram[crate::game_state::constants::nmi::STRIPE_BUFFER_021B + 1] = 0x78;
        ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER] = 0x9a;
        ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER + 1] = 0xbc;
        ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER_1] = 0xde;
        ram[crate::game_state::constants::nmi::BG_CHAR_BUFFER_1 + 1] = 0xf0;
        ram[crate::game_state::constants::nmi::BG_CHAR_HALF_BUFFER] = 0x13;
        ram[crate::game_state::constants::nmi::BG_CHAR_HALF_BUFFER + 1] = 0x57;
        ram[crate::game_state::constants::nmi::BG1_WALL_TOP_BUFFER] = 0x24;
        ram[crate::game_state::constants::nmi::BG1_WALL_TOP_BUFFER + 1] = 0x68;
        ram[crate::game_state::constants::nmi::BG1_WALL_BOTTOM_BUFFER] = 0xac;
        ram[crate::game_state::constants::nmi::BG1_WALL_BOTTOM_BUFFER + 1] = 0xe0;
        ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_BUFFER] = 0x31;
        ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_BUFFER + 1] = 0x42;
        ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_TAIL_BUFFER] = 0x53;
        ram[crate::game_state::constants::nmi::GAME_OVER_TEXT_TAIL_BUFFER + 1] = 0x64;
        ram[POLYHEDRAL_BUFFER] = 0x75;
        ram[POLYHEDRAL_BUFFER + 1] = 0x86;
        write_le_u16(
            &mut ram,
            crate::game_state::constants::nmi::ARBITRARY_TILEMAP_DST_BUFFER + 4,
            0x789a,
        );
        ram[DUNGEON_BG2_ATTR_TABLE] = 0xa5;
        ram[DUNGEON_BG2_ATTR_TABLE + 1] = 0x5a;
        ram[DUNGEON_BG1_ATTR_TABLE] = 0xc3;
        ram[DUNGEON_BG1_ATTR_TABLE + 1] = 0x3c;
        ram[0x4567] = 0x81;
        ram[0x4568] = 0x18;
        write_le_u16(&mut ram, messaging_constants::MESSAGE_DMA_DST_ADDR, 0x6040);
        write_le_u16(&mut ram, messaging_constants::MESSAGE_DMA_TILE_BASE, 0x4841);
        write_le_u16(
            &mut ram,
            messaging_constants::MESSAGE_DMA_TILE_LIMIT,
            0x007f,
        );
        write_le_u16(
            &mut ram,
            messaging_constants::MESSAGE_DMA_TILE_SENTINEL,
            0xffff,
        );
        ram[HUD_TILE_INDICES_BUFFER] = 0xbe;
        ram[HUD_TILE_INDICES_BUFFER + 1] = 0xef;
        ram[STAR_TILE_RESTORE_PHASE] = 1;
        write_le_u16(&mut ram, ANIMATED_TILE_DATA_SRC, 0xa680);
        write_le_u16(&mut ram, ANIMATED_TILE_VRAM_ADDR, 0x3b00);
        write_le_u16(&mut ram, ATTRACT_VRAM_DST, 0x0168);
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_X, 0x0120);
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y, 0x0140);
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y_RADIUS, 0x0030);
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_X_RADIUS, 0x0040);
        write_le_u16(&mut ram, WATERGATE_SPOTLIGHT_Y_UPPER, 0x0050);
        ram[WATERGATE_POINTER] = 0x06;
        write_le_u16(&mut ram, WATERGATE_POS, 0x0780);
        ram[0xa680] = 0xde;
        ram[0xa681] = 0xad;

        let mut display = DisplayState::load_from_ram(&ram);
        assert_eq!(display.screen_brightness, 0x0f);
        assert_eq!(display.nmi_update_latch, 1);
        assert!(display.nmi_update_is_latched());
        assert_eq!(display.core_update_disable_flag, 4);
        assert!(display.core_updates_are_disabled());
        assert_eq!(display.pending_nmi_subroutine, 11);
        assert_eq!(display.bg_vram_load_mode, 3);
        assert!(display.has_bg_vram_load());
        assert_eq!(display.pending_tilemap_update_destination_page, 0x50);
        assert!(display.has_pending_tilemap_update());
        assert_eq!(display.pending_tilemap_update_vram_destination(), 0x5000);
        assert_eq!(display.pending_tilemap_update_source_offset, 0x0200);
        assert_eq!(
            display.pending_tilemap_update_source_address(),
            crate::game_state::constants::nmi::BG_CHAR_BUFFER + 0x0200
        );
        assert_eq!(display.bg_mode, 7);
        assert_eq!(display.main_screen_layers, 0x16);
        assert_eq!(display.sub_screen_layers, 0x01);
        assert_eq!(display.layer_masks_word(), 0x0116);
        assert_eq!(display.bg12_window_selection, 0x33);
        assert_eq!(display.bg34_window_selection, 3);
        assert_eq!(display.object_color_window_selection, 0xb0);
        assert_eq!(display.main_screen_window_layers, 0x16);
        assert_eq!(display.sub_screen_window_layers, 1);
        assert_eq!(display.nmi_copy_packets_request, 1);
        assert!(display.has_nmi_copy_packets_request());
        assert_eq!(display.pending_polyhedral_update, 0xff);
        assert!(display.has_pending_polyhedral_update());
        assert_eq!(display.chr_halfslot_request, 9);
        assert!(display.has_chr_halfslot_request());
        assert!(display.nmi_thread_active);
        assert_eq!(display.nmi_thread_stack_pointer, 0x01f2);
        assert!(display.nmi_thread_uses_poly_stack());
        assert_eq!(display.irq_control_flag, 0x80);
        assert!(display.has_irq_control_flag());
        assert!(display.irq_control_has_vcounter_marker());
        assert_eq!(display.vertical_irq_trigger, 0x90);
        assert_eq!(display.sprite_dma_head_pointer, 0x20);
        assert_eq!(display.sprite_dma_body_pointer, 0xa0);
        assert_eq!(&display.sprite_oam_shadow_buffer(&ram)[..2], &[0xca, 0xfe]);
        assert_eq!(display.hdma_enable_mask, 0xc0);
        assert!(display.is_hdma_channel_enabled(6));
        assert!(display.is_hdma_channel_enabled(7));
        assert!(!display.is_hdma_channel_enabled(5));
        assert_eq!(display.mosaic_copy, 0x73);
        assert_eq!(display.mosaic_level, 0x70);
        assert_eq!(display.mosaic_target_level, 0x1f);
        assert_eq!(display.mosaic_target_level_word(), 0x1f);
        assert_eq!(display.mosaic_direction, 1);
        assert_eq!(display.nmi_load_target_address, 0x2146);
        assert_eq!(display.nmi_load_target_page(), 0x46);
        assert_eq!(display.vram_upload_cursor, 0x0124);
        assert_eq!(display.vram_upload_cursor_usize(), 0x0124);
        assert_eq!(
            display.current_vram_upload_data_address(),
            VRAM_UPLOAD_DATA + 0x0124
        );
        assert_eq!(&display.nmi_vram_packet_buffer(&ram)[..2], &[0xfa, 0xce]);
        assert_eq!(display.overworld_tile_upload_word(&ram, 0), 0xcefa);
        assert_eq!(display.overworld_tile_attribute_word(&ram, 3), 0x4567);
        assert_eq!(
            &display.tilemap_upload_stripe_buffer(&ram)[..2],
            &[0x24, 0x01]
        );
        assert_eq!(
            &display.secondary_stripe_upload_buffer(&ram)[..2],
            &[0x56, 0x78]
        );
        assert_eq!(
            &display.background_character_buffer(&ram)[..2],
            &[0x9a, 0xbc]
        );
        assert_eq!(
            &display.background_character_secondary_buffer(&ram)[..2],
            &[0xde, 0xf0]
        );
        assert_eq!(
            &display.background_character_half_buffer(&ram)[..2],
            &[0x13, 0x57]
        );
        assert_eq!(
            &display.bg1_wall_top_tilemap_buffer(&ram)[..2],
            &[0x24, 0x68]
        );
        assert_eq!(
            &display.bg1_wall_bottom_tilemap_buffer(&ram)[..2],
            &[0xac, 0xe0]
        );
        assert_eq!(
            &display.game_over_text_tile_buffer(&ram)[..2],
            &[0x31, 0x42]
        );
        assert_eq!(
            &display.game_over_text_tail_tile_buffer(&ram)[..2],
            &[0x53, 0x64]
        );
        assert_eq!(&display.polyhedral_tile_buffer(&ram)[..2], &[0x75, 0x86]);
        assert_eq!(display.arbitrary_tilemap_destination(&ram, 2), 0x789a);
        assert_eq!(
            &display.dungeon_bg2_attribute_table(&ram)[..2],
            &[0xa5, 0x5a]
        );
        assert_eq!(
            &display.dungeon_bg1_attribute_table(&ram)[..2],
            &[0xc3, 0x3c]
        );
        assert_eq!(
            display.vram_dma_source_bytes(&ram, 0x4567, 2),
            &[0x81, 0x18]
        );
        assert_eq!(display.message_dma_destination_address, 0x6040);
        assert_eq!(display.message_dma_tile_base, 0x4841);
        assert_eq!(display.message_dma_tile_limit, 0x007f);
        assert_eq!(display.message_dma_tile_sentinel, 0xffff);
        assert_eq!(&display.message_dma_tile_indices(&ram)[..2], &[0xbe, 0xef]);
        assert_eq!(display.star_tile_restore_phase, 1);
        assert_eq!(display.star_tile_restore_source_offsets(), (32, 0));
        assert_eq!(display.animated_tile_data_source_address, 0xa680);
        assert_eq!(display.animated_tile_data_source_usize(), 0xa680);
        assert_eq!(&display.animated_tile_data(&ram)[..2], &[0xde, 0xad]);
        assert!(display.has_animated_tile_data_source());
        assert_eq!(display.animated_tile_vram_destination_address, 0x3b00);
        assert_eq!(display.animated_tile_vram_destination_usize(), 0x3b00);
        assert_eq!(display.attract_vram_destination_address, 0x0168);
        assert!(!display.attract_vram_destination_high_is_clear());
        assert_eq!(display.water_hdma_window.window_x(), 0x0120);
        assert_eq!(display.water_hdma_window.window_y(), 0x0140);
        assert_eq!(display.water_hdma_window.window_y_radius(), 0x0030);
        assert_eq!(display.water_hdma_window.window_x_radius(), 0x0040);
        assert_eq!(
            display.water_hdma_window.watergate_spotlight_y_upper(),
            0x0050
        );
        assert_eq!(display.water_hdma_window.watergate_pointer(), 0x06);
        assert_eq!(display.water_hdma_window.watergate_tilemap_pos_x2(), 0x0780);

        display.screen_brightness = 0x80;
        display.nmi_update_latch = 0;
        display.core_update_disable_flag = 0;
        display.pending_nmi_subroutine = 0;
        display.bg_vram_load_mode = 0;
        display.pending_tilemap_update_destination_page = 0x40;
        display.pending_tilemap_update_source_offset = 0x0600;
        display.bg_mode = 9;
        display.main_screen_layers = 0x11;
        display.sub_screen_layers = 0;
        display.bg12_window_selection = 0;
        display.bg34_window_selection = 0;
        display.object_color_window_selection = 0x30;
        display.main_screen_window_layers = 3;
        display.sub_screen_window_layers = 0;
        display.nmi_copy_packets_request = 0;
        display.pending_polyhedral_update = 0;
        display.chr_halfslot_request = 0;
        display.nmi_thread_active = false;
        display.nmi_thread_stack_pointer = 0x1f31;
        display.irq_control_flag = 0;
        display.vertical_irq_trigger = 0x70;
        display.sprite_dma_head_pointer = 0x40;
        display.sprite_dma_body_pointer = 0x80;
        display.hdma_enable_mask = 0x80;
        display.mosaic_copy = 3;
        display.mosaic_level = 0x20;
        display.mosaic_target_level = 0;
        display.mosaic_direction = 0;
        display.nmi_load_target_address = 0x0080;
        display.vram_upload_cursor = 0x0042;
        display.message_dma_destination_address = 0x6080;
        display.message_dma_tile_base = 0x4842;
        display.message_dma_tile_limit = 0x0080;
        display.message_dma_tile_sentinel = 0xfffe;
        display.star_tile_restore_phase = 0;
        display.animated_tile_data_source_address = 0xac80;
        display.animated_tile_vram_destination_address = 0x3c00;
        display.attract_vram_destination_address = 0x0068;
        display.water_hdma_window.set_window_x(0x0220);
        display.water_hdma_window.set_window_y(0x0240);
        display.water_hdma_window.set_window_y_radius_byte(0x31);
        display.water_hdma_window.set_window_x_radius(0x0048);
        display
            .water_hdma_window
            .set_watergate_spotlight_y_upper(0x0058);
        display.water_hdma_window.set_watergate_pointer(0x07);
        display
            .water_hdma_window
            .set_watergate_tilemap_pos_x2(0x0880);
        display.write_to_ram(&mut ram);

        assert_eq!(ram[INIDISP_COPY], 0x80);
        assert_eq!(ram[NMI_BOOLEAN], 0);
        assert_eq!(ram[NMI_DISABLE_CORE_UPDATES], 0);
        assert_eq!(ram[NMI_SUBROUTINE_INDEX], 0);
        assert_eq!(ram[NMI_LOAD_BG_FROM_VRAM], 0);
        assert_eq!(ram[NMI_UPDATE_TILEMAP_DST], 0x40);
        assert_eq!(read_le_u16(&ram, NMI_UPDATE_TILEMAP_SRC), 0x0600);
        assert_eq!(ram[BGMODE_COPY], 9);
        assert_eq!(ram[TM_COPY], 0x11);
        assert_eq!(ram[TS_COPY], 0);
        assert_eq!(ram[W12SEL_COPY], 0);
        assert_eq!(ram[W34SEL_COPY], 0);
        assert_eq!(ram[WOBJSEL_COPY], 0x30);
        assert_eq!(ram[TMW_COPY], 3);
        assert_eq!(ram[TSW_COPY], 0);
        assert_eq!(ram[NMI_COPY_PACKETS_FLAG], 0);
        assert_eq!(
            read_le_u16(&ram, messaging_constants::MESSAGE_DMA_DST_ADDR),
            0x6080
        );
        assert_eq!(
            read_le_u16(&ram, messaging_constants::MESSAGE_DMA_TILE_BASE),
            0x4842
        );
        assert_eq!(
            read_le_u16(&ram, messaging_constants::MESSAGE_DMA_TILE_LIMIT),
            0x0080
        );
        assert_eq!(
            read_le_u16(&ram, messaging_constants::MESSAGE_DMA_TILE_SENTINEL),
            0xfffe
        );
        assert_eq!(ram[NMI_FLAG_UPDATE_POLYHEDRAL], 0);
        assert_eq!(ram[LOAD_CHR_HALFSLOT_EVEN_ODD], 0);
        assert_eq!(ram[NMI_THREAD_ACTIVE], 0);
        assert_eq!(read_le_u16(&ram, POLY_THREAD_STACK), 0x1f31);
        assert_eq!(ram[IRQ_FLAG], 0);
        assert_eq!(ram[VIRQ_TRIGGER], 0x70);
        assert_eq!(ram[DMA_HEAD_POINTER], 0x40);
        assert_eq!(ram[DMA_BODY_POINTER], 0x80);
        assert_eq!(ram[HDMAEN_COPY], 0x80);
        assert_eq!(ram[MOSAIC_COPY], 3);
        assert_eq!(ram[MOSAIC_LEVEL], 0x20);
        assert_eq!(ram[MOSAIC_TARGET_LEVEL], 0);
        assert_eq!(ram[MOSAIC_INC_OR_DEC], 0);
        assert_eq!(read_le_u16(&ram, NMI_LOAD_TARGET_ADDR), 0x0080);
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET), 0x0042);
        assert_eq!(ram[STAR_TILE_RESTORE_PHASE], 0);
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_DATA_SRC), 0xac80);
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_VRAM_ADDR), 0x3c00);
        assert_eq!(read_le_u16(&ram, ATTRACT_VRAM_DST), 0x0068);
        assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X), 0x0220);
        assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_Y), 0x0240);
        assert_eq!(ram[WATER_HDMA_WINDOW_Y_RADIUS], 0x31);
        assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X_RADIUS), 0x0048);
        assert_eq!(read_le_u16(&ram, WATERGATE_SPOTLIGHT_Y_UPPER), 0x0058);
        assert_eq!(ram[WATERGATE_POINTER], 0x07);
        assert_eq!(read_le_u16(&ram, WATERGATE_POS), 0x0880);
    }

    #[test]
    fn native_attract_vram_destination_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, ATTRACT_VRAM_DST, 0x0160);

        let mut display = DisplayState::default();
        {
            let mut bridge = NativeAttractVramDestinationBridgeMut::new(&mut display, &mut ram);
            bridge.set_page_offset(0x70);
            bridge.decrement_page_offset();
            bridge.set_address(0x0068);
            assert_eq!(bridge.decrement_address(), 0x0067);
            bridge.clear_address();
            bridge.set_address(0x0068);
        }

        assert_eq!(display.attract_vram_destination_address, 0x0068);
        assert!(display.attract_vram_destination_high_is_clear());
        assert_eq!(read_le_u16(&ram, ATTRACT_VRAM_DST), 0x0068);
    }

    #[test]
    fn water_hdma_window_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_X, 0x0120);
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y, 0x0140);
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y_RADIUS, 0x0230);
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_X_RADIUS, 0x0040);
        write_le_u16(&mut ram, WATERGATE_SPOTLIGHT_Y_UPPER, 0x0050);
        ram[WATERGATE_POINTER] = 0x06;
        write_le_u16(&mut ram, WATERGATE_POS, 0x0780);

        let mut water = WaterHdmaWindowState::load_from_ram(&ram);
        assert_eq!(water.window_x(), 0x0120);
        assert_eq!(water.window_y(), 0x0140);
        assert_eq!(water.window_y_radius(), 0x0230);
        assert_eq!(water.window_x_radius(), 0x0040);
        assert_eq!(water.watergate_spotlight_y_upper(), 0x0050);
        assert_eq!(water.watergate_pointer(), 0x06);
        assert_eq!(water.watergate_tilemap_pos_x2(), 0x0780);

        water.set_window_x(0x0220);
        water.set_window_y(0x0240);
        water.set_window_x_radius(0x0048);
        water.set_window_y_radius_byte(0x31);
        water.decrement_watergate_spotlight_y_upper();
        assert_eq!(water.increment_watergate_pointer(), 0x07);
        water.set_watergate_tilemap_pos_x2(0x0880);
        water.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X), 0x0220);
        assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_Y), 0x0240);
        assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_Y_RADIUS), 0x0231);
        assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X_RADIUS), 0x0048);
        assert_eq!(read_le_u16(&ram, WATERGATE_SPOTLIGHT_Y_UPPER), 0x004f);
        assert_eq!(ram[WATERGATE_POINTER], 0x07);
        assert_eq!(read_le_u16(&ram, WATERGATE_POS), 0x0880);
    }

    #[test]
    fn native_water_hdma_window_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_X, 0x0120);
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y, 0x0140);
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_Y_RADIUS, 0x0230);
        write_le_u16(&mut ram, WATER_HDMA_WINDOW_X_RADIUS, 0x0040);
        write_le_u16(&mut ram, WATERGATE_SPOTLIGHT_Y_UPPER, 0x0050);
        ram[WATERGATE_POINTER] = 0x06;
        write_le_u16(&mut ram, WATERGATE_POS, 0x0780);
        write_le_u16(&mut ram, SPOTLIGHT_Y_UPPER, 0x1111);
        write_le_u16(&mut ram, SPOTLIGHT_WINDOW_Y_BUFFER, 0x2210);

        let mut display = DisplayState::default();
        {
            let mut bridge = NativeWaterHdmaWindowBridgeMut::new(&mut display, &mut ram);
            bridge.set_window_x(0x0220);
            bridge.set_window_y(0x0240);
            bridge.set_window_x_radius(0x0048);
            bridge.set_window_y_radius_byte(0x31);
            assert_eq!(bridge.decrement_watergate_spotlight_y_upper(), 0x004f);
            bridge.set_watergate_spotlight_y_upper(0x0058);
            bridge.set_watergate_pointer(0x07);
            assert_eq!(bridge.increment_watergate_pointer(), 0x08);
            bridge.set_watergate_tilemap_pos_x2(0x0880);
            assert_eq!(bridge.advance_watergate_window_y_radius(), 0x51);
        }

        assert_eq!(display.water_hdma_window.window_x(), 0x0220);
        assert_eq!(display.water_hdma_window.window_y(), 0x0240);
        assert_eq!(display.water_hdma_window.window_y_radius(), 0x0251);
        assert_eq!(display.water_hdma_window.window_x_radius(), 0x0048);
        assert_eq!(
            display.water_hdma_window.watergate_spotlight_y_upper(),
            0x0058
        );
        assert_eq!(display.water_hdma_window.watergate_pointer(), 0x08);
        assert_eq!(display.water_hdma_window.watergate_tilemap_pos_x2(), 0x0880);
        assert_eq!(display.spotlight_hdma.y_upper(), 0x0058);
        assert_eq!(display.spotlight_hdma.window_y_buffer_byte(), 0x11);
        assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X), 0x0220);
        assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_Y), 0x0240);
        assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_Y_RADIUS), 0x0251);
        assert_eq!(read_le_u16(&ram, WATER_HDMA_WINDOW_X_RADIUS), 0x0048);
        assert_eq!(read_le_u16(&ram, WATERGATE_SPOTLIGHT_Y_UPPER), 0x0058);
        assert_eq!(ram[WATERGATE_POINTER], 0x08);
        assert_eq!(read_le_u16(&ram, WATERGATE_POS), 0x0880);
        assert_eq!(ram[SPOTLIGHT_Y_UPPER], 0x58);
        assert_eq!(ram[SPOTLIGHT_WINDOW_Y_BUFFER], 0x11);
    }

    #[test]
    fn native_spotlight_hdma_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, SPOTLIGHT_Y_LOWER, 0x0010);
        write_le_u16(&mut ram, SPOTLIGHT_Y_UPPER, 0x0020);
        write_le_u16(&mut ram, SPOTLIGHT_WINDOW_RADIUS, 0x1234);
        write_le_u16(&mut ram, SPOTLIGHT_WINDOW_STATE, 0x5678);
        write_le_u16(&mut ram, SPOTLIGHT_WINDOW_Y_BUFFER, 0x9abc);
        write_le_u16(&mut ram, HDMA_TABLE_DYNAMIC + 6, 0xbeef);

        let mut spotlight = SpotlightHdmaState::default();
        {
            let mut bridge = NativeSpotlightHdmaBridgeMut::new(&mut spotlight, &mut ram);
            bridge.set_y_lower(0x0030);
            bridge.set_y_upper(0x0040);
            bridge.set_window_radius_byte(0x80);
            bridge.shr_window_radius_byte(1);
            bridge.add_window_radius_byte(0x10);
            bridge.set_window_state_byte(0x02);
            assert_eq!(bridge.decrement_window_y_buffer(), 0x9abb);
            bridge.set_hdma_table_dynamic_entry(3, 0xcafe);
            bridge.clear_hdma_table_dynamic_range(3, 1);
        }

        assert_eq!(spotlight.y_lower(), 0x0030);
        assert_eq!(spotlight.y_upper(), 0x0040);
        assert_eq!(spotlight.window_radius(), 0x1250);
        assert_eq!(spotlight.window_state(), 0x5602);
        assert_eq!(spotlight.window_y_buffer(), 0x9abb);
        assert_eq!(spotlight.hdma_table_dynamic_entry(3), 0);
        assert_eq!(read_le_u16(&ram, SPOTLIGHT_Y_LOWER), 0x0030);
        assert_eq!(read_le_u16(&ram, SPOTLIGHT_Y_UPPER), 0x0040);
        assert_eq!(read_le_u16(&ram, SPOTLIGHT_WINDOW_RADIUS), 0x1250);
        assert_eq!(read_le_u16(&ram, SPOTLIGHT_WINDOW_STATE), 0x5602);
        assert_eq!(read_le_u16(&ram, SPOTLIGHT_WINDOW_Y_BUFFER), 0x9abb);
        assert_eq!(read_le_u16(&ram, HDMA_TABLE_DYNAMIC + 6), 0);
    }

    #[test]
    fn overworld_palette_backup_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP] = 0x12;
        ram[OVERWORLD_PAL_AUX3_BP7_BACKUP] = 0x34;
        ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP] = 0x56;

        let mut backup = OverworldPaletteBackupState::load_from_ram(&ram);
        assert_eq!(backup.main_indoors(), 0x12);
        assert_eq!(backup.aux3_bg_palette_7(), 0x34);
        assert_eq!(backup.main_indoors_copy(), 0x56);

        backup.set_main_indoors(0x9a);
        backup.set_aux3_bg_palette_7(0xbc);
        backup.set_main_indoors_copy(0xde);
        backup.write_to_ram(&mut ram);

        assert_eq!(ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP], 0x9a);
        assert_eq!(ram[OVERWORLD_PAL_AUX3_BP7_BACKUP], 0xbc);
        assert_eq!(ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP], 0xde);
    }

    #[test]
    fn native_overworld_palette_backup_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP] = 0x12;
        ram[OVERWORLD_PAL_AUX3_BP7_BACKUP] = 0x34;
        ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP] = 0x56;

        let mut backup = OverworldPaletteBackupState::default();
        {
            let mut bridge = NativeOverworldPaletteBackupBridgeMut::new(&mut backup, &mut ram);
            bridge.set_main_indoors_backup(0x9a);
            bridge.set_aux3_bg_palette_7_backup(0xbc);
            bridge.set_main_indoors_copy_backup(0xde);
        }

        assert_eq!(backup.main_indoors(), 0x9a);
        assert_eq!(backup.aux3_bg_palette_7(), 0xbc);
        assert_eq!(backup.main_indoors_copy(), 0xde);
        assert_eq!(ram[OVERWORLD_PAL_MAIN_INDOORS_BACKUP], 0x9a);
        assert_eq!(ram[OVERWORLD_PAL_AUX3_BP7_BACKUP], 0xbc);
        assert_eq!(ram[OVERWORLD_PAL_MAIN_INDOORS_COPY_BACKUP], 0xde);
    }

    #[test]
    fn palette_buffer_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MAIN_PALETTE_BUFFER + 4, 0x1234);
        write_le_u16(&mut ram, AUX_PALETTE_BUFFER + 6, 0x5678);
        ram[AUX_PALETTE_BUFFER + 255] = 0x9a;
        ram[MAPBAK_PALETTE + 511] = 0xbc;
        write_le_u16(&mut ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x0200);
        ram[PALETTE_SP0L] = 1;
        ram[PALETTE_SP5L] = 2;
        ram[PALETTE_SP6L] = 3;
        ram[PALETTE_MAIN_INDOORS] = 4;
        ram[HUD_PALETTE] = 5;
        ram[PALETTE_SP6R_INDOORS] = 6;
        ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI] = 7;
        ram[OVERWORLD_PALETTE_AUX3_BP7_LO] = 8;
        ram[OVERWORLD_PALETTE_MODE] = 9;

        let palette = PaletteBufferState::load_from_ram(&ram);
        assert_eq!(palette.main_color(2), 0x1234);
        assert_eq!(palette.aux_color(3), 0x5678);
        assert_eq!(palette.aux_visible_slice()[255], 0x9a);
        assert_eq!(palette.overworld_palette_backup()[511], 0xbc);
        assert_eq!(palette.overworld_aux_or_main_offset(), 0x0200);

        let mut projected = vec![0; WRAM_SIZE];
        palette.write_to_ram(&mut projected);
        assert_eq!(PaletteBufferState::load_from_ram(&projected), palette);
        assert_eq!(projected[PALETTE_SP0L], 1);
        assert_eq!(projected[OVERWORLD_PALETTE_MODE], 9);
    }

    #[test]
    fn native_palette_buffer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0xff; WRAM_SIZE];
        write_le_u16(&mut ram, OVERWORLD_PALETTE_AUX_OR_MAIN, 0x12ab);

        let mut display = DisplayState::default();
        {
            let mut bridge = NativePaletteBufferBridgeMut::new(&mut display, &mut ram);
            bridge.set_main_color(2, 0x1234);
            bridge.set_aux_color(3, 0x5678);
            bridge.keep_overworld_aux_or_main_low_byte();
            bridge.select_overworld_aux_palette_offset();
            bridge.copy_aux_visible_from(&vec![0x22; 256]);
            bridge.copy_main_palette_bytes(&[0x11, 0x22, 0x33, 0x44], 4);
            bridge.backup_overworld_palette_from(&vec![0x77; 512]);
            bridge.clear_aux_sprite_subpalettes();
            bridge.set_sp0l(1);
            bridge.set_sp5l(2);
            bridge.set_sp6l(3);
            bridge.set_palette_main_indoors(4);
            bridge.set_hud_palette(5);
            bridge.set_sp6r_indoors(6);
            bridge.set_overworld_palette_aux2_hi(7);
            bridge.set_overworld_palette_aux3_lo(8);
            bridge.set_bg_tile_animation_countdown(0x9abc);
            bridge.set_overworld_palette_mode(9);
        }

        assert_eq!(display.palette_buffer.main_color(0), 0x2211);
        assert_eq!(display.palette_buffer.main_color(1), 0x4433);
        assert_eq!(display.palette_buffer.aux_color(3), 0x2222);
        assert_eq!(display.palette_buffer.aux_visible_slice()[0], 0x22);
        assert_eq!(
            display.palette_buffer.aux_full_slice()[0x180..0x200],
            [0; 0x80]
        );
        assert_eq!(display.palette_buffer.overworld_palette_backup()[511], 0x77);
        assert_eq!(
            display.palette_buffer.overworld_aux_or_main_offset(),
            0x0200
        );
        assert_eq!(display.bg_tile_animation_countdown, 0x9abc);
        assert_eq!(read_le_u16(&ram, MAIN_PALETTE_BUFFER), 0x2211);
        assert_eq!(read_le_u16(&ram, AUX_PALETTE_BUFFER + 6), 0x2222);
        assert_eq!(ram[MAPBAK_PALETTE + 511], 0x77);
        assert_eq!(read_le_u16(&ram, OVERWORLD_PALETTE_AUX_OR_MAIN), 0x0200);
        assert_eq!(ram[PALETTE_SP0L], 1);
        assert_eq!(ram[PALETTE_SP5L], 2);
        assert_eq!(ram[PALETTE_SP6L], 3);
        assert_eq!(ram[PALETTE_MAIN_INDOORS], 4);
        assert_eq!(ram[HUD_PALETTE], 5);
        assert_eq!(ram[PALETTE_SP6R_INDOORS], 6);
        assert_eq!(ram[OVERWORLD_PALETTE_AUX2_BP5TO7_HI], 7);
        assert_eq!(ram[OVERWORLD_PALETTE_AUX3_BP7_LO], 8);
        assert_eq!(read_le_u16(&ram, BG_TILE_ANIMATION_COUNTDOWN), 0x9abc);
        assert_eq!(ram[OVERWORLD_PALETTE_MODE], 9);
    }

    #[test]
    fn palette_filter_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, PALETTE_FILTER_COUNTDOWN, 0x1204);
        write_le_u16(&mut ram, DARKENING_OR_LIGHTENING_SCREEN, 0x34ff);
        ram[CGWSEL_COPY] = 0x20;
        ram[CGADSUB_COPY] = 0x31;
        ram[CGADSUB_COPY + 1] = 0x42;
        ram[COLDATA_COPY0] = 0x21;
        ram[COLDATA_COPY1] = 0x43;
        ram[COLDATA_COPY2] = 0x85;

        let palette_filter = PaletteFilterState::load_from_ram(&ram);
        assert_eq!(palette_filter.countdown(), 4);
        assert_eq!(palette_filter.countdown_word(), 0x1204);
        assert_eq!(palette_filter.darkening_or_lightening_screen(), 0xff);
        assert_eq!(palette_filter.darkening_or_lightening_screen_word(), 0x34ff);
        assert_eq!(palette_filter.color_window_selection(), 0x20);
        assert_eq!(palette_filter.color_math_control(), 0x31);
        assert_eq!(palette_filter.color_window_and_math_word(), 0x3120);
        assert_eq!(palette_filter.color_math_control_word(), 0x4231);
        assert_eq!(palette_filter.fixed_color_red(), 0x21);
        assert_eq!(palette_filter.fixed_color_green(), 0x43);
        assert_eq!(palette_filter.fixed_color_blue(), 0x85);
        assert_eq!(palette_filter.fixed_color_component(0), 0x21);
        assert_eq!(palette_filter.fixed_color_component(3), 0);

        let mut projected = vec![0; WRAM_SIZE];
        palette_filter.write_to_ram(&mut projected);
        assert_eq!(read_le_u16(&projected, PALETTE_FILTER_COUNTDOWN), 0x1204);
        assert_eq!(
            read_le_u16(&projected, DARKENING_OR_LIGHTENING_SCREEN),
            0x34ff
        );
        assert_eq!(projected[CGWSEL_COPY], 0x20);
        assert_eq!(projected[CGADSUB_COPY], 0x31);
        assert_eq!(projected[COLDATA_COPY0], 0x21);
        assert_eq!(projected[COLDATA_COPY1], 0x43);
        assert_eq!(projected[COLDATA_COPY2], 0x85);
    }

    #[test]
    fn native_palette_filter_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, PALETTE_FILTER_COUNTDOWN, 0x1200);
        write_le_u16(&mut ram, DARKENING_OR_LIGHTENING_SCREEN, 0x3401);
        ram[CGWSEL_COPY] = 0x20;
        ram[CGADSUB_COPY] = 0x31;
        ram[CGADSUB_COPY + 1] = 0x42;
        ram[COLDATA_COPY0] = 0x20;
        ram[COLDATA_COPY1] = 0x40;
        ram[COLDATA_COPY2] = 0x80;

        let mut display = DisplayState::default();
        {
            let mut bridge = NativePaletteFilterBridgeMut::new(&mut display, &mut ram);
            bridge.increment_countdown();
            bridge.decrement_countdown();
            bridge.set_countdown_word(0x5607);
            bridge.xor_darkening_or_lightening_screen(0xff);
            bridge.set_darkening_or_lightening_screen_word(0x7809);
            bridge.set_color_window_and_math_word(0x3322);
            bridge.set_color_window_selection(0x24);
            bridge.set_color_math_control(0x35);
            bridge.or_fixed_color_red(0x01);
            bridge.subtract_fixed_color_red(2);
            bridge.set_fixed_color_green(0x50);
            bridge.or_fixed_color_green(0x0f);
            bridge.subtract_fixed_color_green(1);
            bridge.set_fixed_color_blue(0x90);
            bridge.or_fixed_color_blue(0x0f);
            bridge.subtract_fixed_color_blue(1);
            bridge.set_fixed_color_component(2, 0x88);
            bridge.or_fixed_color_component(0, 0x10);
            bridge.set_fixed_color_red(0x22);
        }

        assert_eq!(display.palette_filter.countdown_word(), 0x5607);
        assert_eq!(
            display.palette_filter.darkening_or_lightening_screen_word(),
            0x7809
        );
        assert_eq!(display.palette_filter.color_window_and_math_word(), 0x3524);
        assert_eq!(display.palette_filter.color_math_control_word(), 0x4235);
        assert_eq!(display.palette_filter.fixed_color_red(), 0x22);
        assert_eq!(display.palette_filter.fixed_color_green(), 0x5e);
        assert_eq!(display.palette_filter.fixed_color_blue(), 0x88);
        assert_eq!(read_le_u16(&ram, PALETTE_FILTER_COUNTDOWN), 0x5607);
        assert_eq!(read_le_u16(&ram, DARKENING_OR_LIGHTENING_SCREEN), 0x7809);
        assert_eq!(ram[CGWSEL_COPY], 0x24);
        assert_eq!(ram[CGADSUB_COPY], 0x35);
        assert_eq!(ram[CGADSUB_COPY + 1], 0x42);
        assert_eq!(ram[COLDATA_COPY0], 0x22);
        assert_eq!(ram[COLDATA_COPY1], 0x5e);
        assert_eq!(ram[COLDATA_COPY2], 0x88);
    }

    #[test]
    fn trinexx_palette_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TRINEXX_RED_SHELL_PALETTE_DELAY] = 2;
        ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] = 4;
        ram[TRINEXX_RED_SHELL_PALETTE_STEP] = 6;
        ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] = 8;

        let mut palette = TrinexxPaletteState::load_from_ram(&ram);
        assert_eq!(
            palette,
            TrinexxPaletteState {
                red_shell_delay: 2,
                blue_shell_delay: 4,
                red_shell_step: 6,
                blue_shell_step: 8,
            }
        );

        palette.decrement_red_shell_delay();
        palette.decrement_blue_shell_delay();
        assert_eq!(palette.increment_red_shell_step(), 7);
        assert_eq!(palette.increment_blue_shell_step(), 9);
        palette.write_to_ram(&mut ram);

        assert_eq!(ram[TRINEXX_RED_SHELL_PALETTE_DELAY], 1);
        assert_eq!(ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY], 3);
        assert_eq!(ram[TRINEXX_RED_SHELL_PALETTE_STEP], 7);
        assert_eq!(ram[TRINEXX_BLUE_SHELL_PALETTE_STEP], 9);
    }

    #[test]
    fn native_trinexx_palette_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TRINEXX_RED_SHELL_PALETTE_DELAY] = 0;
        ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY] = 0;
        ram[TRINEXX_RED_SHELL_PALETTE_STEP] = 0xff;
        ram[TRINEXX_BLUE_SHELL_PALETTE_STEP] = 0xfe;

        let mut display = DisplayState::default();
        {
            let mut bridge = NativeTrinexxPaletteBridgeMut::new(&mut display, &mut ram);
            bridge.set_red_shell_delay(3);
            bridge.set_blue_shell_delay(4);
            bridge.decrement_red_shell_delay();
            bridge.decrement_blue_shell_delay();
            assert_eq!(bridge.increment_red_shell_step(), 0);
            assert_eq!(bridge.increment_blue_shell_step(), 0xff);
            bridge.set_red_shell_step(2);
            bridge.set_blue_shell_step(5);
        }

        assert_eq!(
            display.trinexx_palette,
            TrinexxPaletteState {
                red_shell_delay: 2,
                blue_shell_delay: 3,
                red_shell_step: 2,
                blue_shell_step: 5,
            }
        );
        assert_eq!(ram[TRINEXX_RED_SHELL_PALETTE_DELAY], 2);
        assert_eq!(ram[TRINEXX_BLUE_SHELL_PALETTE_DELAY], 3);
        assert_eq!(ram[TRINEXX_RED_SHELL_PALETTE_STEP], 2);
        assert_eq!(ram[TRINEXX_BLUE_SHELL_PALETTE_STEP], 5);
    }

    #[test]
    fn dialogue_message_index_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DIALOGUE_MESSAGE_INDEX, 0x0123);

        let mut message_index = DialogueMessageIndexState::load_from_ram(&ram);
        assert_eq!(message_index.value(), 0x0123);

        message_index.set_value(0x0140);
        message_index.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, DIALOGUE_MESSAGE_INDEX), 0x0140);
    }

    #[test]
    fn native_dialogue_message_index_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, DIALOGUE_MESSAGE_INDEX, 0x0123);

        let mut message_index = DialogueMessageIndexState::default();
        {
            let mut bridge = NativeDialogueMessageIndexBridgeMut::new(&mut message_index, &mut ram);
            bridge.set_value(0x0140);
        }

        assert_eq!(message_index.value(), 0x0140);
        assert_eq!(read_le_u16(&ram, DIALOGUE_MESSAGE_INDEX), 0x0140);
    }

    #[test]
    fn select_file_menu_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, SELECTFILE_SAVE_SLOT_FLAGS, 1);
        write_le_u16(&mut ram, SELECTFILE_SAVE_SLOT_FLAGS + 2, 2);
        write_le_u16(&mut ram, SELECTFILE_SAVE_SLOT_FLAGS + 4, 3);
        ram[SELECT_FILE_CURSOR_WORK] = 4;
        ram[SELECT_FILE_TRANSITION_WORK] = 7;
        write_le_u16(&mut ram, SELECT_FILE_TARGET_WORK, 0x1234);
        write_le_u16(&mut ram, SELECT_FILE_COPY_SOURCE_SLOT_X2, 0x1204);
        ram[SELECT_FILE_REMEMBERED_CURSOR] = 2;
        write_le_u16(&mut ram, SELECT_FILE_NAME_SCROLL_X, 0x01f0);
        ram[SELECT_FILE_NAME_COLUMN] = 5;
        ram[SELECT_FILE_NAME_CURSOR_Y] = 0x83;
        ram[SELECT_FILE_NAME_SLOT] = 3;
        ram[SELECT_FILE_NAME_SCROLL_X_STEP] = 8;
        ram[SELECT_FILE_NAME_SCROLL_Y_STEP] = 9;
        ram[SELECT_FILE_NAME_ROW] = 6;
        ram[SELECT_FILE_NAME_SCROLL_X_DIRECTION] = 1;

        let mut menu = SelectFileMenuState::load_from_ram(&ram);
        assert_eq!(menu.save_slot_flags(), [1, 2, 3]);
        assert!(menu.any_save_slot_flag());
        assert_eq!(menu.cursor(), 4);
        assert_eq!(menu.choice(0), 0x34);
        assert_eq!(menu.target_word(), 0x1234);
        assert_eq!(menu.copy_source_slot_x2(), 0x1204);
        assert_eq!(menu.copy_source_slot(), 0x0902);
        assert_eq!(menu.name_scroll_x(), 0x01f0);
        assert!(menu.is_name_scrolling());
        menu.clear_name_entry_state();
        menu.write_to_ram(&mut ram);

        assert_eq!(ram[SELECT_FILE_NAME_COLUMN], 0);
        assert_eq!(ram[SELECT_FILE_NAME_SLOT], 0);
        assert_eq!(ram[SELECT_FILE_NAME_ROW], 0);
        assert_eq!(ram[SELECT_FILE_CHOICE_WORK], 0);
        assert_eq!(ram[SELECT_FILE_COPY_SOURCE_SLOT_X2], 0);
        assert_eq!(ram[SELECT_FILE_COPY_SOURCE_SLOT_X2 + 1], 0x12);
        assert_eq!(ram[SELECT_FILE_NAME_CURSOR_Y], 0x83);
        assert_eq!(read_le_u16(&ram, SELECT_FILE_NAME_SCROLL_X), 0x01f0);
    }

    #[test]
    fn native_select_file_menu_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[SELECT_FILE_CURSOR_WORK] = 0xff;
        ram[SELECT_FILE_NAME_SLOT] = 0;
        ram[SELECT_FILE_NAME_CURSOR_Y] = 0x80;
        ram[SELECT_FILE_NAME_SCROLL_X_STEP] = 0xfe;

        let mut menu = SelectFileMenuState::default();
        {
            let mut bridge = NativeSelectFileMenuBridgeMut::new(&mut menu, &mut ram);
            assert_eq!(bridge.increment_cursor(), 0);
            bridge.remember_current_cursor();
            bridge.set_cursor(3);
            bridge.restore_remembered_cursor();
            bridge.set_target_word(0x1234);
            bridge.set_copy_source_slot(2);
            assert_eq!(bridge.move_name_slot_left_wrapped(), 5);
            assert_eq!(bridge.move_name_slot_right_wrapped(), 0);
            assert!(bridge.step_name_cursor_y_toward(0x84));
            assert_eq!(bridge.advance_name_scroll_x_step_by(4), 2);
            bridge.mark_save_slot_present(1);
            bridge.clear_transition_scratch();
        }

        assert_eq!(menu.cursor(), 0);
        assert_eq!(menu.remembered_cursor(), 0);
        assert_eq!(menu.target_word(), 0x1234);
        assert_eq!(menu.copy_source_slot_x2(), 4);
        assert_eq!(menu.name_slot(), 0);
        assert_eq!(menu.name_cursor_y(), 0x82);
        assert_eq!(menu.name_scroll_x_step(), 2);
        assert_eq!(menu.save_slot_flag(1), 1);
        assert_eq!(ram[SELECT_FILE_CURSOR_WORK], 0);
        assert_eq!(ram[SELECT_FILE_REMEMBERED_CURSOR], 0);
        assert_eq!(read_le_u16(&ram, SELECT_FILE_TARGET_WORK), 0x1234);
        assert_eq!(read_le_u16(&ram, SELECT_FILE_COPY_SOURCE_SLOT_X2), 4);
        assert_eq!(ram[SELECT_FILE_NAME_SLOT], 0);
        assert_eq!(ram[SELECT_FILE_NAME_CURSOR_Y], 0x82);
        assert_eq!(ram[SELECT_FILE_NAME_SCROLL_X_STEP], 2);
        assert_eq!(read_le_u16(&ram, SELECTFILE_SAVE_SLOT_FLAGS + 2), 1);
        assert_eq!(ram[SELECT_FILE_TRANSITION_WORK], 0);
    }

    #[test]
    fn multiselect_choice_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MULTISELECT_CHOICE, 0x0204);
        ram[MULTISELECT_CHOICE_BACKUP] = 0x07;

        let mut choice = MultiselectChoiceState::load_from_ram(&ram);
        assert_eq!(choice.value(), 0x04);
        assert_eq!(
            MultiselectChoiceRead::new(&choice, &MessagingRuntimeState::load_from_ram(&ram))
                .value_word(),
            0x0204
        );
        assert_eq!(choice.backup(), 0x07);

        choice.increment_value();
        choice.save_backup();
        choice.decrement_value();
        choice.restore_backup();
        choice.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, MULTISELECT_CHOICE), 0x0205);
        assert_eq!(ram[MULTISELECT_CHOICE_BACKUP], 0x05);
    }

    #[test]
    fn native_multiselect_choice_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MULTISELECT_CHOICE, 0x0204);
        ram[MULTISELECT_CHOICE_BACKUP] = 0x07;

        let mut choice = MultiselectChoiceState::default();
        {
            let mut bridge = NativeMultiselectChoiceBridgeMut::new(&mut choice, &mut ram);
            bridge.increment_value();
            bridge.save_backup();
            bridge.set_value(0x01);
            bridge.restore_backup();
            bridge.decrement_value();
        }

        assert_eq!(choice.value(), 0x04);
        assert_eq!(
            MultiselectChoiceRead::new(&choice, &MessagingRuntimeState::load_from_ram(&ram))
                .value_word(),
            0x0204
        );
        assert_eq!(choice.backup(), 0x05);
        assert_eq!(read_le_u16(&ram, MULTISELECT_CHOICE), 0x0204);
        assert_eq!(ram[MULTISELECT_CHOICE_BACKUP], 0x05);
    }

    #[test]
    fn dialogue_number_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DIALOGUE_NUMBER_LO] = 0x12;
        ram[DIALOGUE_NUMBER_HI] = 0x34;

        let mut number = DialogueNumberState::load_from_ram(&ram);
        assert_eq!(number.packed_digits(0), 0x12);
        assert_eq!(number.packed_digits(1), 0x34);
        assert_eq!(number.packed_digits(2), 0);

        number.set_low_pair(0x56);
        number.set_high_pair(0x78);
        number.write_to_ram(&mut ram);

        assert_eq!(ram[DIALOGUE_NUMBER_LO], 0x56);
        assert_eq!(ram[DIALOGUE_NUMBER_HI], 0x78);
    }

    #[test]
    fn native_dialogue_number_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DIALOGUE_NUMBER_LO] = 0x12;
        ram[DIALOGUE_NUMBER_HI] = 0x34;

        let mut number = DialogueNumberState::default();
        {
            let mut bridge = NativeDialogueNumberBridgeMut::new(&mut number, &mut ram);
            bridge.set_low_pair(0x56);
            bridge.set_high_pair(0x78);
            bridge.set_packed_digits(0x9a, 0xbc);
        }

        assert_eq!(number.packed_digits(0), 0x9a);
        assert_eq!(number.packed_digits(1), 0xbc);
        assert_eq!(ram[DIALOGUE_NUMBER_LO], 0x9a);
        assert_eq!(ram[DIALOGUE_NUMBER_HI], 0xbc);
    }

    #[test]
    fn dialogue_source_offset_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DIALOGUE_MSG_SRC_OFFS] = 0xaa;
        ram[DIALOGUE_MSG_SRC_OFFS + 1] = 0xbb;
        ram[DIALOGUE_MSG_SRC_OFFS + 2] = 0x0e;

        let mut source_offset = DialogueSourceOffsetState::load_from_ram(&ram);
        assert_eq!(source_offset.bank_offset_low_nibble(), 0x0e);
        assert_eq!(source_offset.increment_bank_offset_low_nibble(), 0x0f);
        assert_eq!(source_offset.increment_bank_offset_low_nibble(), 0x10);
        source_offset.write_to_ram(&mut ram);

        assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS], 0xaa);
        assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS + 1], 0xbb);
        assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS + 2], 0x10);
    }

    #[test]
    fn native_dialogue_source_offset_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[DIALOGUE_MSG_SRC_OFFS] = 0xaa;
        ram[DIALOGUE_MSG_SRC_OFFS + 1] = 0xbb;
        ram[DIALOGUE_MSG_SRC_OFFS + 2] = 0xff;

        let mut source_offset = DialogueSourceOffsetState::default();
        {
            let mut bridge = NativeDialogueSourceOffsetBridgeMut::new(&mut source_offset, &mut ram);
            assert_eq!(bridge.increment_bank_offset_low_nibble(), 0);
            assert_eq!(bridge.increment_bank_offset_low_nibble(), 1);
        }

        assert_eq!(source_offset.bank_offset_low_nibble(), 1);
        assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS], 0xaa);
        assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS + 1], 0xbb);
        assert_eq!(ram[DIALOGUE_MSG_SRC_OFFS + 2], 1);
    }

    #[test]
    fn decoded_message_text_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MESSAGING_TEXT_BUFFER] = 0x12;
        ram[MESSAGING_TEXT_BUFFER + 1] = 0x34;
        ram[MESSAGING_TEXT_BUFFER + 2] = 0x56;

        let mut text = DecodedMessageTextState::load_from_ram(&ram);
        assert_eq!(text.byte(0), 0x12);
        assert_eq!(text.next_byte(0), Some(0x34));
        assert_eq!(text.byte(usize::MAX), 0);

        assert_eq!(text.load_decoded_dialogue(&[1, 2, 3, 4]), 4);
        assert_eq!(
            text.write_decoded_text_at(MESSAGING_TEXT_BUFFER + 2, &[0xaa, 0xbb]),
            2
        );
        text.write_to_ram(&mut ram);

        assert_eq!(
            &ram[MESSAGING_TEXT_BUFFER..MESSAGING_TEXT_BUFFER + 4],
            &[1, 2, 0xaa, 0xbb]
        );
    }

    #[test]
    fn dialogue_pointer_table_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[TEXT_DIALOGUE_POINTERS + 3] = 0x12;
        ram[TEXT_DIALOGUE_POINTERS + 4] = 0x34;
        ram[TEXT_DIALOGUE_POINTERS + 5] = 0x56;

        let mut pointers = DialoguePointerTableState::load_from_ram(&ram);
        assert_eq!(pointers.pointer(1), 0x563412);
        assert_eq!(pointers.pointer(398), 0);

        pointers.set_pointer(1, 0xffaabbcc);
        pointers.write_to_ram(&mut ram);

        assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 3], 0xcc);
        assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 4], 0xbb);
        assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 5], 0xaa);
    }

    #[test]
    fn native_decoded_message_text_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[MESSAGING_TEXT_BUFFER] = 0x12;
        ram[TEXT_DIALOGUE_POINTERS] = 0x11;
        ram[TEXT_DIALOGUE_POINTERS + 1] = 0x22;
        ram[TEXT_DIALOGUE_POINTERS + 2] = 0x33;

        let mut messaging = MessagingState::default();
        {
            let mut bridge = NativeDecodedMessageTextBridgeMut::new(&mut messaging, &mut ram);
            assert_eq!(bridge.load_decoded_dialogue(&[1, 2, 3]), 3);
            assert_eq!(
                bridge.write_decoded_text_at(MESSAGING_TEXT_BUFFER + 1, &[0xaa, 0xbb]),
                2
            );
            bridge.set_dialogue_pointer(0, 0xffc0de);
        }

        assert_eq!(messaging.decoded_text.byte(0), 1);
        assert_eq!(messaging.decoded_text.byte(1), 0xaa);
        assert_eq!(messaging.decoded_text.byte(2), 0xbb);
        assert_eq!(messaging.dialogue_pointers.pointer(0), 0xffc0de);
        assert_eq!(
            &ram[MESSAGING_TEXT_BUFFER..MESSAGING_TEXT_BUFFER + 3],
            &[1, 0xaa, 0xbb]
        );
        assert_eq!(ram[TEXT_DIALOGUE_POINTERS], 0xde);
        assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 1], 0xc0);
        assert_eq!(ram[TEXT_DIALOGUE_POINTERS + 2], 0xff);
    }

    #[test]
    fn messaging_render_buffer_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MESSAGING_RENDER_BUFFER + 4, 0x1234);
        ram[MESSAGING_RENDER_BUFFER + 8] = 0xaa;

        let mut render_buffer = MessagingRenderBufferState::load_from_ram(&ram);
        assert_eq!(render_buffer.word(2), 0x1234);
        assert_eq!(render_buffer.word_at_byte_offset(8), 0x00aa);
        assert_eq!(render_buffer.word_at_byte_offset(0x7df), 0);

        render_buffer.set_word(2, 0x5678);
        render_buffer.xor_mask(8, 0x0f);
        render_buffer.clear_mask(8, 0xf0);
        render_buffer.fill_word_range(4, 2, 0x1111);
        render_buffer.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 4), 0x5678);
        assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 8), 0x1111);
        assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 10), 0x1111);
    }

    #[test]
    fn native_messaging_render_buffer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, MESSAGING_RENDER_BUFFER + 2, 0x1111);
        ram[MESSAGING_RENDER_BUFFER + 6] = 0xf0;

        let mut render_buffer = MessagingRenderBufferState::default();
        {
            let mut bridge =
                NativeMessagingRenderBufferBridgeMut::new(&mut render_buffer, &mut ram);
            bridge.set_word(1, 0x2222);
            bridge.set_word_at_byte_offset(4, 0x3333);
            bridge.xor_mask(6, 0x0f);
            bridge.clear_mask(6, 0xf0);
            bridge.fill_word_range(4, 2, 0x4444);
            bridge.clear_range(2);
        }

        assert_eq!(render_buffer.word(0), 0);
        assert_eq!(render_buffer.word(1), 0x2222);
        assert_eq!(render_buffer.word_at_byte_offset(4), 0x3333);
        assert_eq!(render_buffer.word_at_byte_offset(8), 0x4444);
        assert_eq!(render_buffer.word_at_byte_offset(10), 0x4444);
        assert_eq!(ram[MESSAGING_RENDER_BUFFER], 0);
        assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 2), 0x2222);
        assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 4), 0x3333);
        assert_eq!(ram[MESSAGING_RENDER_BUFFER + 6], 0x0f);
        assert_eq!(read_le_u16(&ram, MESSAGING_RENDER_BUFFER + 8), 0x4444);
    }

    #[test]
    fn vwf_render_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[VWF_ARR + 3] = 0x22;
        write_le_u16(&mut ram, VWF_GLYPH_CURSOR, 0x0040);
        write_le_u16(&mut ram, VWF_FLAG_NEXT_LINE, 1);
        write_le_u16(&mut ram, VWF_CURLINE, 4);
        write_le_u16(&mut ram, VWF_LINE_PTR, 0x0540);
        write_le_u16(&mut ram, VWF_TILE_BUFFER + 0x0c2, 0x3456);

        let mut vwf = VwfRenderState::load_from_ram(&ram);
        assert_eq!(vwf.glyph_advance_prefix_sum(3), 0x22);
        assert_eq!(vwf.glyph_cursor(), 0x0040);
        assert_eq!(vwf.glyph_cursor_usize(), 0x40);
        assert_eq!(vwf.next_line_requested(), 1);
        assert_eq!(vwf.current_line(), 4);
        assert_eq!(vwf.line_render_offset(), 0x0540);
        assert_eq!(vwf.tile_word_at_byte_offset(0x0c2), 0x3456);
        assert_eq!(vwf.tile_word_at_byte_offset(0x0fb), 0);

        vwf.set_next_glyph_advance_prefix_sum(3, 0x33);
        vwf.set_glyph_cursor(0x0080);
        vwf.request_next_line(1);
        vwf.set_current_line(2);
        vwf.set_line_render_offset(0x02a0);
        vwf.set_tile_word_at_byte_offset(0x0c4, 0x789a);
        vwf.write_to_ram(&mut ram);

        assert_eq!(ram[VWF_ARR + 4], 0x33);
        assert_eq!(read_le_u16(&ram, VWF_GLYPH_CURSOR), 0x0080);
        assert_eq!(read_le_u16(&ram, VWF_FLAG_NEXT_LINE), 1);
        assert_eq!(read_le_u16(&ram, VWF_CURLINE), 2);
        assert_eq!(read_le_u16(&ram, VWF_LINE_PTR), 0x02a0);
        assert_eq!(read_le_u16(&ram, VWF_TILE_BUFFER + 0x0c4), 0x789a);
    }

    #[test]
    fn native_vwf_render_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[VWF_ARR + 2] = 0x10;
        write_le_u16(&mut ram, VWF_GLYPH_CURSOR, 0x0002);
        write_le_u16(&mut ram, VWF_FLAG_NEXT_LINE, 1);
        write_le_u16(&mut ram, VWF_CURLINE, 2);
        write_le_u16(&mut ram, VWF_LINE_PTR, 0x02a0);

        let mut vwf = VwfRenderState::default();
        {
            let mut bridge = NativeVwfRenderBridgeMut::new(&mut vwf, &mut ram);
            bridge.set_next_glyph_advance_prefix_sum(2, 0x18);
            assert_eq!(bridge.increment_glyph_cursor(), 3);
            bridge.set_glyph_cursor(0x0040);
            bridge.clear_glyph_cursor();
            bridge.request_next_line(1);
            bridge.clear_next_line_request();
            bridge.set_current_line(4);
            bridge.set_line_render_offset(0x0540);
            bridge.set_tile_word_at_byte_offset(0x0ec, 0xabcd);
        }

        assert_eq!(vwf.glyph_advance_prefix_sum(3), 0x18);
        assert_eq!(vwf.glyph_cursor(), 0);
        assert_eq!(vwf.next_line_requested(), 0);
        assert_eq!(vwf.current_line(), 4);
        assert_eq!(vwf.line_render_offset(), 0x0540);
        assert_eq!(vwf.tile_word_at_byte_offset(0x0ec), 0xabcd);
        assert_eq!(ram[VWF_ARR + 3], 0x18);
        assert_eq!(read_le_u16(&ram, VWF_GLYPH_CURSOR), 0);
        assert_eq!(read_le_u16(&ram, VWF_FLAG_NEXT_LINE), 0);
        assert_eq!(read_le_u16(&ram, VWF_CURLINE), 4);
        assert_eq!(read_le_u16(&ram, VWF_LINE_PTR), 0x0540);
        assert_eq!(read_le_u16(&ram, VWF_TILE_BUFFER + 0x0ec), 0xabcd);
    }

    #[test]
    fn hud_inventory_order_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        for index in 0..24 {
            ram[HUD_INVENTORY_ORDER + index] = 24 - index as u8;
        }

        let mut order = HudInventoryOrderState::load_from_ram(&ram);
        assert!(order.is_custom());
        assert_eq!(order.item(0), 24);
        assert_eq!(order.item(23), 1);
        assert_eq!(order.item(24), 0);

        order.initialize_default_order(24);
        order.swap_items(0, 23);
        order.write_to_ram(&mut ram);

        assert_eq!(ram[HUD_INVENTORY_ORDER], 24);
        assert_eq!(ram[HUD_INVENTORY_ORDER + 23], 1);
    }

    #[test]
    fn native_hud_inventory_order_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut display = DisplayState::default();

        {
            let mut bridge = NativeHudInventoryOrderBridgeMut::new(&mut display, &mut ram);
            bridge.initialize_default_order(24);
            bridge.swap_items(1, 22);
        }

        assert_eq!(display.hud_inventory_order.item(0), 1);
        assert_eq!(display.hud_inventory_order.item(1), 23);
        assert_eq!(display.hud_inventory_order.item(22), 2);
        assert_eq!(read_le_u16(&ram, HUD_INVENTORY_ORDER), 0x1701);
        assert_eq!(ram[HUD_INVENTORY_ORDER + 22], 2);
    }

    #[test]
    fn hud_runtime_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[SUPER_BOMB_INDICATOR_TIMER] = 7;
        ram[SUPER_BOMB_INDICATOR_COUNTER] = 2;
        ram[RUPEE_SFX_SOUND_DELAY] = 5;
        ram[IS_DOING_HEART_ANIMATION] = 1;
        ram[HEART_REFILL_COUNTDOWN] = 6;
        ram[HEART_REFILL_ANIM_SUBPOS] = 0x80;
        ram[FLASHING_CIRCLE_TIMER] = 0x10;
        ram[MENU_PREV_JOYPAD_H] = 0x40;
        ram[EQUIPMENT_MENU_EXIT_STATE] = 3;
        ram[BOTTLE_MENU_ROW] = 9;
        ram[HDR_DUNGEON_DARK_WITH_LANTERN] = 1;
        ram[HUD_MODULE_TICK_COUNTER] = 0x33;

        let runtime = HudRuntimeState::load_from_ram(&ram);
        assert_eq!(runtime.super_bomb_indicator_timer(), 7);
        assert_eq!(runtime.super_bomb_indicator_counter(), 2);
        assert_eq!(runtime.rupee_sfx_sound_delay(), 5);
        assert!(runtime.is_doing_heart_animation());
        assert_eq!(runtime.is_doing_heart_animation_raw(), 1);
        assert_eq!(runtime.heart_refill_countdown(), 6);
        assert_eq!(runtime.heart_refill_anim_subpos(), 0x80);
        assert_eq!(runtime.flashing_circle_timer(), 0x10);
        assert_eq!(runtime.prev_joypad_h(), 0x40);
        assert_eq!(runtime.equipment_menu_exit_state(), 3);
        assert_eq!(runtime.bottle_menu_row(), 9);
        assert!(runtime.dungeon_dark_with_lantern());
        assert_eq!(runtime.tick_counter(), 0x33);

        let mut projected = vec![0; WRAM_SIZE];
        runtime.write_to_ram(&mut projected);
        assert_eq!(projected[SUPER_BOMB_INDICATOR_TIMER], 7);
        assert_eq!(projected[SUPER_BOMB_INDICATOR_COUNTER], 2);
        assert_eq!(projected[RUPEE_SFX_SOUND_DELAY], 5);
        assert_eq!(projected[IS_DOING_HEART_ANIMATION], 1);
        assert_eq!(projected[HEART_REFILL_COUNTDOWN], 6);
        assert_eq!(projected[HEART_REFILL_ANIM_SUBPOS], 0x80);
        assert_eq!(projected[FLASHING_CIRCLE_TIMER], 0x10);
        assert_eq!(projected[MENU_PREV_JOYPAD_H], 0x40);
        assert_eq!(projected[EQUIPMENT_MENU_EXIT_STATE], 3);
        assert_eq!(projected[BOTTLE_MENU_ROW], 9);
        assert_eq!(projected[HDR_DUNGEON_DARK_WITH_LANTERN], 1);
        assert_eq!(projected[HUD_MODULE_TICK_COUNTER], 0x33);
    }

    #[test]
    fn hud_tilemap_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, HUD_FLOOR_CHANGED_TIMER, 0x1234);
        write_le_u16(&mut ram, HUD_TILE_INDICES_BUFFER + 4, 0xbeef);
        write_le_u16(&mut ram, MOVING_WALL_REPLACEMENT_BUFFER - 2, 0xabcd);

        let tilemap = HudTilemapState::load_from_ram(&ram);
        assert_eq!(tilemap.floor_changed_timer_low(), 0x34);
        assert_eq!(tilemap.tile_word(2), 0xbeef);
        assert_eq!(
            tilemap.tile_word((MOVING_WALL_REPLACEMENT_BUFFER - HUD_TILE_INDICES_BUFFER) / 2 - 1),
            0xabcd
        );
        assert_eq!(
            tilemap.tile_word((MOVING_WALL_REPLACEMENT_BUFFER - HUD_TILE_INDICES_BUFFER) / 2),
            0
        );

        let mut projected = vec![0; WRAM_SIZE];
        tilemap.write_to_ram(&mut projected);
        assert_eq!(HudTilemapState::load_from_ram(&projected), tilemap);
    }

    #[test]
    fn native_hud_state_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[BOTTLE_MENU_ROW] = 5;
        ram[HUD_TILE_INDICES_BUFFER + 4] = 0x34;
        ram[HUD_TILE_INDICES_BUFFER + 5] = 0x12;
        let mut display = DisplayState::default();

        {
            let mut bridge = NativeHudStateBridgeMut::new(&mut display, &mut ram);
            bridge.set_super_bomb_indicator_timer(8);
            bridge.set_super_bomb_indicator_counter(3);
            bridge.set_rupee_sfx_sound_delay(4);
            bridge.set_is_doing_heart_animation(1);
            bridge.set_heart_refill_countdown(7);
            bridge.set_heart_refill_anim_subpos(0x20);
            bridge.set_flashing_circle_timer(0x10);
            bridge.set_prev_joypad_h(0x80);
            bridge.set_equipment_menu_exit_state(2);
            assert_eq!(bridge.decrement_bottle_menu_row(), 4);
            bridge.set_dungeon_dark_with_lantern();
            bridge.set_tick_counter(0x44);
            bridge.set_floor_changed_timer(0x1234);
            bridge.clear_floor_changed_timer_low();
            bridge.set_tile_word(2, 0xbeef);
            bridge.clear_is_doing_heart_animation();
            bridge.clear_prev_joypad_h();
        }

        assert_eq!(display.hud_runtime.super_bomb_indicator_timer(), 8);
        assert_eq!(display.hud_runtime.super_bomb_indicator_counter(), 3);
        assert_eq!(display.hud_runtime.rupee_sfx_sound_delay(), 4);
        assert!(!display.hud_runtime.is_doing_heart_animation());
        assert_eq!(display.hud_runtime.heart_refill_countdown(), 7);
        assert_eq!(display.hud_runtime.heart_refill_anim_subpos(), 0x20);
        assert_eq!(display.hud_runtime.flashing_circle_timer(), 0x10);
        assert_eq!(display.hud_runtime.prev_joypad_h(), 0);
        assert_eq!(display.hud_runtime.equipment_menu_exit_state(), 2);
        assert_eq!(display.hud_runtime.bottle_menu_row(), 4);
        assert!(display.hud_runtime.dungeon_dark_with_lantern());
        assert_eq!(display.hud_runtime.tick_counter(), 0x44);
        assert_eq!(display.hud_tilemap.floor_changed_timer_low(), 0);
        assert_eq!(display.hud_tilemap.tile_word(2), 0xbeef);
        assert_eq!(ram[SUPER_BOMB_INDICATOR_TIMER], 8);
        assert_eq!(ram[SUPER_BOMB_INDICATOR_COUNTER], 3);
        assert_eq!(ram[RUPEE_SFX_SOUND_DELAY], 4);
        assert_eq!(ram[IS_DOING_HEART_ANIMATION], 0);
        assert_eq!(ram[MENU_PREV_JOYPAD_H], 0);
        assert_eq!(ram[HUD_FLOOR_CHANGED_TIMER], 0);
        assert_eq!(ram[HUD_FLOOR_CHANGED_TIMER + 1], 0x12);
        assert_eq!(read_le_u16(&ram, HUD_TILE_INDICES_BUFFER + 4), 0xbeef);
    }

    #[test]
    fn ending_credit_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, ENDING_WHICH_DUNG, 5);
        write_le_u16(&mut ram, ENDING_CREDIT_DIGIT_CHAR, 0x3cf6);

        let mut credits = EndingCreditState::load_from_ram(&ram);
        assert_eq!(credits.palace_death_count_digit_step, 5);
        assert_eq!(credits.palace_death_count_index(), 2);
        assert_eq!(credits.digit_tile_base_index(), 1);
        assert!(credits.should_write_digit_for_scroll_y(0x200, 0x290));
        assert_eq!(credits.death_count_digit_tile_base, 0x3cf6);

        credits.clear_palace_death_count_digit_step();
        credits.death_count_digit_tile_base = 0x3ce6;
        credits.advance_palace_death_count_digit_step();
        credits.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, ENDING_WHICH_DUNG), 1);
        assert_eq!(read_le_u16(&ram, ENDING_CREDIT_DIGIT_CHAR), 0x3ce6);
    }

    #[test]
    fn intro_scene_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[INTRO_WANT_DOUBLE_RET] = 1;
        write_le_u16(&mut ram, INTRO_SPRITE_ALLOC, 0x0800);
        write_le_u16(&mut ram, TRIFORCE_CTR, 0x01c0);

        let mut intro = IntroSceneState::load_from_ram(&ram);
        assert!(intro.triangle_motion_is_paused());
        assert_eq!(intro.sprite_oam_cursor, 0x0800);
        assert_eq!(intro.triforce_countdown, 0x01c0);
        assert_eq!(intro.allocate_oam_entries(3), 0x0800);
        assert_eq!(intro.sprite_oam_cursor, 0x080c);
        intro.resume_triangle_motion();
        intro.decrement_triforce_countdown();
        intro.write_to_ram(&mut ram);

        assert_eq!(ram[INTRO_WANT_DOUBLE_RET], 0);
        assert_eq!(read_le_u16(&ram, INTRO_SPRITE_ALLOC), 0x080c);
        assert_eq!(read_le_u16(&ram, TRIFORCE_CTR), 0x01bf);
    }

    #[test]
    fn native_intro_scene_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[INTRO_WANT_DOUBLE_RET] = 0;
        write_le_u16(&mut ram, INTRO_SPRITE_ALLOC, 0x0800);
        write_le_u16(&mut ram, TRIFORCE_CTR, 0);

        let mut intro = IntroSceneState::default();
        {
            let mut bridge = NativeIntroSceneBridgeMut::new(&mut intro, &mut ram);
            bridge.pause_triangle_motion();
            assert_eq!(bridge.allocate_oam_entries(2), 0x0800);
            bridge.set_triforce_countdown(0x0001);
            bridge.decrement_triforce_countdown();
            bridge.resume_triangle_motion();
            bridge.set_sprite_oam_cursor(0x0900);
        }

        assert_eq!(
            intro,
            IntroSceneState {
                triangle_motion_pause: 0,
                sprite_oam_cursor: 0x0900,
                triforce_countdown: 0,
            }
        );
        assert_eq!(ram[INTRO_WANT_DOUBLE_RET], 0);
        assert_eq!(read_le_u16(&ram, INTRO_SPRITE_ALLOC), 0x0900);
        assert_eq!(read_le_u16(&ram, TRIFORCE_CTR), 0);
    }

    #[test]
    fn native_ending_credit_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, ENDING_WHICH_DUNG, 0xffff);
        write_le_u16(&mut ram, ENDING_CREDIT_DIGIT_CHAR, 0x1111);

        let mut credits = EndingCreditState::default();
        {
            let mut bridge = NativeEndingCreditBridgeMut::new(&mut credits, &mut ram);
            bridge.clear_palace_death_count_digit_step();
            bridge.advance_palace_death_count_digit_step();
            bridge.set_death_count_digit_tile_base(0x3cf6);
            bridge.set_palace_death_count_digit_step(4);
        }

        assert_eq!(credits.palace_death_count_digit_step, 4);
        assert_eq!(credits.death_count_digit_tile_base, 0x3cf6);
        assert_eq!(read_le_u16(&ram, ENDING_WHICH_DUNG), 4);
        assert_eq!(read_le_u16(&ram, ENDING_CREDIT_DIGIT_CHAR), 0x3cf6);
    }

    #[test]
    fn native_vram_upload_buffer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x0010);

        let mut display = DisplayState::default();
        {
            let mut bridge = NativeVramUploadBufferBridgeMut::new(&mut display, &mut ram);
            bridge.advance_offset_by(0x20);
            bridge.clear_offset();
            bridge.set_offset(0x0034);
            bridge.write_buffer_byte(40, 0xaa);
            bridge.write_buffer_word(42, 0xbbcc);
            bridge.write_tilemap_word(80, 0x1234);
            bridge.write_overworld_vram_word(3, 0x5678);
            bridge.write_absolute_byte(0x2000, 0xdd);
            bridge.write_absolute_word(0x2002, 0xeeff);
            bridge.copy_buffer_bytes(44, &[1, 2, 3, 4]);
            bridge.terminate_buffer_at(48);
            bridge.write_level_label_tiles(&[0x11; 14], &[0x22; 14]);
            bridge.write_map16_update_packet(0x2100, 0x1234, [0x1000, 0x1001, 0x1002, 0x1003]);
            bridge.write_single_tile_stripe_packet(0x2120, 0x3456, 0x2000);
            bridge.write_tile_stripe_sentinel(0x2130);
        }

        assert_eq!(display.vram_upload_cursor, 0x0034);
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET), 0x0034);
        assert_eq!(ram[VRAM_UPLOAD_DATA], 0x11);
        assert_eq!(ram[VRAM_UPLOAD_DATA + 13], 0x11);
        assert_eq!(ram[VRAM_UPLOAD_DATA + 16], 0x22);
        assert_eq!(ram[VRAM_UPLOAD_DATA + 29], 0x22);
        assert_eq!(ram[VRAM_UPLOAD_DATA + 32], 0xff);
        assert_eq!(ram[VRAM_UPLOAD_DATA + 40], 0xaa);
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_DATA + 42), 0xbbcc);
        assert_eq!(
            &ram[VRAM_UPLOAD_DATA + 44..VRAM_UPLOAD_DATA + 48],
            &[1, 2, 3, 4]
        );
        assert_eq!(ram[VRAM_UPLOAD_DATA + 48], 0xff);
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET + 80), 0x1234);
        assert_eq!(read_le_u16(&ram, UVRAM_DATA + 6), 0x5678);
        assert_eq!(ram[0x2000], 0xdd);
        assert_eq!(read_le_u16(&ram, 0x2002), 0xeeff);
        assert_eq!(read_le_u16(&ram, 0x2100), 0x3412);
        assert_eq!(read_le_u16(&ram, 0x2102), 0x0300);
        assert_eq!(read_le_u16(&ram, 0x2104), 0x1000);
        assert_eq!(read_le_u16(&ram, 0x2106), 0x1001);
        assert_eq!(read_le_u16(&ram, 0x2108), 0x5412);
        assert_eq!(read_le_u16(&ram, 0x210a), 0x0300);
        assert_eq!(read_le_u16(&ram, 0x210c), 0x1002);
        assert_eq!(read_le_u16(&ram, 0x210e), 0x1003);
        assert_eq!(read_le_u16(&ram, 0x2110), 0xffff);
        assert_eq!(read_le_u16(&ram, 0x2120), 0x3456);
        assert_eq!(read_le_u16(&ram, 0x2122), 0x0100);
        assert_eq!(read_le_u16(&ram, 0x2124), 0x2000);
        assert_eq!(read_le_u16(&ram, 0x2130), 0xffff);
    }

    #[test]
    fn link_dma_source_slots_read_named_source_addresses() {
        let mut ram = vec![0; WRAM_SIZE];
        let slots = [
            (LinkDmaSourceSlot::BodyTop, DMA_SOURCE_ADDR_3),
            (LinkDmaSourceSlot::BodyBottom, DMA_SOURCE_ADDR_0),
            (LinkDmaSourceSlot::HeadTop, DMA_SOURCE_ADDR_4),
            (LinkDmaSourceSlot::HeadBottom, DMA_SOURCE_ADDR_1),
            (LinkDmaSourceSlot::HandLeft, DMA_SOURCE_ADDR_5),
            (LinkDmaSourceSlot::HandRight, DMA_SOURCE_ADDR_2),
            (LinkDmaSourceSlot::SwordUpper, DMA_SOURCE_ADDR_6),
            (LinkDmaSourceSlot::SwordLower, DMA_SOURCE_ADDR_11),
            (LinkDmaSourceSlot::ShieldUpper, DMA_SOURCE_ADDR_7),
            (LinkDmaSourceSlot::ShieldLower, DMA_SOURCE_ADDR_12),
            (LinkDmaSourceSlot::AuxUpper, DMA_SOURCE_ADDR_8),
            (LinkDmaSourceSlot::AuxLower, DMA_SOURCE_ADDR_13),
            (LinkDmaSourceSlot::PushUpper, DMA_SOURCE_ADDR_10),
            (LinkDmaSourceSlot::PushLower, DMA_SOURCE_ADDR_15),
            (LinkDmaSourceSlot::AnimatedTileUpper, DMA_SOURCE_ADDR_9),
            (LinkDmaSourceSlot::AnimatedTileLower, DMA_SOURCE_ADDR_14),
            (LinkDmaSourceSlot::HeadPointerUpper, DMA_SOURCE_ADDR_16),
            (LinkDmaSourceSlot::HeadPointerLower, DMA_SOURCE_ADDR_18),
            (LinkDmaSourceSlot::BodyPointerUpper, DMA_SOURCE_ADDR_17),
            (LinkDmaSourceSlot::BodyPointerLower, DMA_SOURCE_ADDR_19),
            (LinkDmaSourceSlot::TravelBirdUpper, DMA_SOURCE_ADDR_20),
            (LinkDmaSourceSlot::TravelBirdLower, DMA_SOURCE_ADDR_21),
        ];

        for (index, (_, address)) in slots.iter().copied().enumerate() {
            write_le_u16(&mut ram, address, 0x9000 + index as u16);
        }

        let display = DisplayState::load_from_ram(&ram);
        for (index, (slot, _)) in slots.iter().copied().enumerate() {
            assert_eq!(display.link_dma_source(slot), 0x9000 + index as u16);
        }

        let mut projected = vec![0; WRAM_SIZE];
        display.write_to_ram(&mut projected);
        for (index, (_, address)) in slots.iter().copied().enumerate() {
            assert_eq!(read_le_u16(&projected, address), 0x9000 + index as u16);
        }
    }

    #[test]
    fn native_display_bridge_syncs_seeded_ram_and_dual_writes_brightness() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[INIDISP_COPY] = 4;
        ram[NMI_BOOLEAN] = 1;
        ram[NMI_DISABLE_CORE_UPDATES] = 2;
        ram[NMI_SUBROUTINE_INDEX] = 6;
        ram[NMI_LOAD_BG_FROM_VRAM] = 2;
        ram[NMI_UPDATE_TILEMAP_DST] = 0x50;
        write_le_u16(&mut ram, NMI_UPDATE_TILEMAP_SRC, 0x0200);
        ram[BGMODE_COPY] = 7;
        ram[TM_COPY] = 0x16;
        ram[TS_COPY] = 0x01;
        ram[W12SEL_COPY] = 0x33;
        ram[W34SEL_COPY] = 3;
        ram[WOBJSEL_COPY] = 0xb0;
        ram[TMW_COPY] = 0x16;
        ram[TSW_COPY] = 1;
        ram[NMI_COPY_PACKETS_FLAG] = 1;
        ram[NMI_FLAG_UPDATE_POLYHEDRAL] = 0xff;
        ram[LOAD_CHR_HALFSLOT_EVEN_ODD] = 3;
        ram[NMI_THREAD_ACTIVE] = 1;
        write_le_u16(&mut ram, POLY_THREAD_STACK, 0x01f2);
        ram[IRQ_FLAG] = 0x80;
        ram[VIRQ_TRIGGER] = 0x90;
        ram[DMA_HEAD_POINTER] = 0x20;
        ram[DMA_BODY_POINTER] = 0xa0;
        ram[HDMAEN_COPY] = 0xc0;
        ram[MOSAIC_COPY] = 0x73;
        ram[MOSAIC_LEVEL] = 0x70;
        ram[MOSAIC_TARGET_LEVEL] = 0x1f;
        ram[MOSAIC_INC_OR_DEC] = 1;
        write_le_u16(&mut ram, NMI_LOAD_TARGET_ADDR, 0x2146);
        write_le_u16(&mut ram, VRAM_UPLOAD_OFFSET, 0x0010);
        ram[INCREMENTAL_COUNTER_FOR_VRAM] = 0xfe;
        write_le_u16(&mut ram, messaging_constants::MESSAGE_DMA_DST_ADDR, 0x6040);
        ram[OVERWORLD_FIXED_COLOR_PLUSMINUS] = 0x20;
        ram[FLAG_TRAVEL_BIRD] = 0x04;
        ram[STAR_TILE_RESTORE_PHASE] = 7;
        write_le_u16(&mut ram, ANIMATED_TILE_DATA_SRC, 0xa680);
        write_le_u16(&mut ram, ANIMATED_TILE_VRAM_ADDR, 0x3b00);

        let mut display = DisplayState::default();
        {
            let mut bridge = NativeDisplayStateBridgeMut::new(&mut display, &mut ram);
            bridge.increment_screen_brightness();
            bridge.decrement_screen_brightness();
            bridge.set_screen_brightness(0x80);
            bridge.clear_nmi_update_latch();
            bridge.latch_nmi_update();
            bridge.clear_core_update_disable_flag();
            bridge.set_core_update_disable_flag(7);
            assert_eq!(bridge.take_pending_nmi_subroutine(), 6);
            bridge.set_pending_nmi_subroutine(11);
            bridge.clear_bg_vram_load_mode();
            bridge.set_bg_vram_load_mode(5);
            bridge.queue_tilemap_update(0x52, 0x0400);
            bridge.clear_pending_tilemap_update_destination();
            bridge.queue_tilemap_update(0x54, 0x0800);
            bridge.set_bg_mode(9);
            bridge.set_layer_masks_word(0x0116);
            bridge.and_main_screen_layers(0x15);
            bridge.or_main_screen_layers(0x01);
            bridge.and_sub_screen_layers(0x0f);
            bridge.or_sub_screen_layers(0x10);
            bridge.clear_sub_screen_layers_word();
            bridge.set_main_screen_layers(0x11);
            bridge.set_sub_screen_layers(0x02);
            bridge.set_window_layer_masks(0x33, 3, 0x33, 0x11, 0x02);
            bridge.set_bg12_window_selection(0x11);
            bridge.set_bg34_window_selection(0x22);
            bridge.set_object_color_window_selection(0x30);
            bridge.set_main_screen_window_layers(0x04);
            bridge.set_sub_screen_window_layers(0x05);
            bridge.clear_window_main_sub_masks();
            bridge.set_window_layer_masks(0x33, 3, 0x33, 0x11, 0x02);
            bridge.clear_nmi_copy_packets_request();
            bridge.request_nmi_copy_packets();
            bridge.set_nmi_copy_packets_request(3);
            bridge.clear_pending_polyhedral_update();
            bridge.request_polyhedral_nmi_update();
            bridge.increment_chr_halfslot_request();
            bridge.clear_chr_halfslot_request();
            bridge.set_chr_halfslot_request(12);
            bridge.deactivate_nmi_thread();
            bridge.activate_nmi_thread();
            bridge.set_nmi_thread_stack_pointer(0x1f31);
            bridge.clear_irq_control_flag();
            bridge.set_irq_control_flag(0xff);
            bridge.set_vertical_irq_trigger(0x70);
            bridge.set_sprite_dma_head_pointer(0x40);
            bridge.set_sprite_dma_body_pointer(0x80);
            bridge.clear_hdma_enable_mask();
            bridge.set_hdma_enable_mask(0x80);
            bridge.set_mosaic_level(0x40);
            assert_eq!(bridge.increment_mosaic_level_by(0x10), 0x50);
            assert_eq!(bridge.decrement_mosaic_level_by(0x20), 0x30);
            bridge.set_mosaic_copy_from_level_or(3);
            bridge.set_mosaic_target_level_word(0x001f);
            bridge.clear_mosaic_target_level_word();
            bridge.set_mosaic_target_level(0x0f);
            bridge.set_mosaic_direction(1);
            bridge.clear_mosaic_direction();
            bridge.set_nmi_load_target_page(0x80);
            bridge.set_nmi_load_target_address(0x1234);
            assert_eq!(bridge.increment_vram_upload_counter(), 0xff);
            assert_eq!(bridge.increment_vram_upload_counter(), 0);
            bridge.reset_incremental_vram_upload_counter();
            bridge.set_link_body_dma_sources(0x9000, 0x9001);
            bridge.set_link_head_dma_sources(0x9002, 0x9003);
            bridge.set_link_hand_dma_sources(0x9004, 0x9005);
            bridge.set_link_sword_dma_sources(0x9006, 0x9007);
            bridge.set_link_shield_dma_sources(0x9008, 0x9009);
            bridge.set_link_aux_dma_sources(0x900a, 0x900b);
            bridge.set_link_push_dma_sources(0x900c, 0x900d);
            bridge.set_link_animated_tile_dma_sources(0x900e, 0x900f);
            bridge.set_link_head_pointer_dma_sources(0x9010, 0x9011);
            bridge.set_link_body_pointer_dma_sources(0x9012, 0x9013);
            bridge.set_travel_bird_dma_sources(0x9014, 0x9015);
            bridge.reset_bg_tile_animation_countdown(0xffff);
            bridge.set_message_dma_destination_address(0x6080);
            bridge.set_message_dma_tile_base(0x4841);
            bridge.set_message_dma_tile_limit(0x007f);
            bridge.set_message_dma_tile_sentinel(0xffff);
            bridge.set_overworld_fixed_color_adjustment(0x30);
            bridge.set_travel_bird_tile_offset(0x08);
            bridge.clear_star_tile_restore_phase();
            bridge.set_animated_tile_data_source_address(0xac80);
            bridge.set_animated_tile_vram_destination_address(0x3c00);
            bridge.set_overworld_tile_attribute_word(7, 0x1234);
            bridge.set_overworld_tile_upload_word(2, 0x5678);
            bridge.terminate_overworld_tile_upload_words(3);
            bridge.copy_tilemap_upload_stripe_bytes(&[0xaa, 0xbb, 0xcc]);
        }

        assert_eq!(display.screen_brightness, 0x80);
        assert_eq!(display.nmi_update_latch, 1);
        assert_eq!(display.core_update_disable_flag, 7);
        assert_eq!(display.pending_nmi_subroutine, 11);
        assert_eq!(display.bg_vram_load_mode, 5);
        assert_eq!(display.pending_tilemap_update_destination_page, 0x54);
        assert!(display.has_pending_tilemap_update());
        assert_eq!(display.pending_tilemap_update_vram_destination(), 0x5400);
        assert_eq!(display.pending_tilemap_update_source_offset, 0x0800);
        assert_eq!(
            display.pending_tilemap_update_source_address(),
            crate::game_state::constants::nmi::BG_CHAR_BUFFER + 0x0800
        );
        assert_eq!(display.bg_mode, 9);
        assert_eq!(display.main_screen_layers, 0x11);
        assert_eq!(display.sub_screen_layers, 0x02);
        assert_eq!(display.layer_masks_word(), 0x0211);
        assert_eq!(display.bg12_window_selection, 0x33);
        assert_eq!(display.bg34_window_selection, 3);
        assert_eq!(display.object_color_window_selection, 0x33);
        assert_eq!(display.main_screen_window_layers, 0x11);
        assert_eq!(display.sub_screen_window_layers, 0x02);
        assert_eq!(display.nmi_copy_packets_request, 3);
        assert_eq!(display.pending_polyhedral_update, 0xff);
        assert!(display.has_pending_polyhedral_update());
        assert_eq!(display.chr_halfslot_request, 12);
        assert!(display.nmi_thread_active);
        assert_eq!(display.nmi_thread_stack_pointer, 0x1f31);
        assert!(!display.nmi_thread_uses_poly_stack());
        assert_eq!(display.irq_control_flag, 0xff);
        assert!(display.irq_control_has_vcounter_marker());
        assert_eq!(display.vertical_irq_trigger, 0x70);
        assert_eq!(display.sprite_dma_head_pointer, 0x40);
        assert_eq!(display.sprite_dma_body_pointer, 0x80);
        assert_eq!(display.hdma_enable_mask, 0x80);
        assert!(display.is_hdma_channel_enabled(7));
        assert!(!display.is_hdma_channel_enabled(6));
        assert_eq!(display.mosaic_level, 0x30);
        assert_eq!(display.mosaic_copy, 0x33);
        assert_eq!(display.mosaic_target_level, 0x0f);
        assert_eq!(display.mosaic_direction, 0);
        assert_eq!(display.nmi_load_target_address, 0x1234);
        assert_eq!(display.vram_upload_cursor, 0xbbaa);
        assert_eq!(display.incremental_vram_upload_counter, 0);
        assert_eq!(display.incremental_vram_upload_counter_usize(), 0);
        assert_eq!(display.link_dma_source(LinkDmaSourceSlot::BodyTop), 0x9000);
        assert_eq!(
            display.link_dma_source(LinkDmaSourceSlot::BodyBottom),
            0x9001
        );
        assert_eq!(display.link_dma_source(LinkDmaSourceSlot::HeadTop), 0x9002);
        assert_eq!(
            display.link_dma_source(LinkDmaSourceSlot::HeadBottom),
            0x9003
        );
        assert_eq!(display.link_dma_source(LinkDmaSourceSlot::HandLeft), 0x9004);
        assert_eq!(
            display.link_dma_source(LinkDmaSourceSlot::HandRight),
            0x9005
        );
        assert_eq!(
            display.link_dma_source(LinkDmaSourceSlot::TravelBirdUpper),
            0x9014
        );
        assert_eq!(
            display.link_dma_source(LinkDmaSourceSlot::TravelBirdLower),
            0x9015
        );
        assert_eq!(display.bg_tile_animation_countdown, 0xffff);
        assert_eq!(display.message_dma_destination_address, 0x6080);
        assert_eq!(display.message_dma_destination_address_usize(), 0x6080);
        assert_eq!(display.message_dma_tile_base, 0x4841);
        assert_eq!(display.message_dma_tile_limit, 0x007f);
        assert_eq!(display.message_dma_tile_sentinel, 0xffff);
        assert_eq!(display.overworld_fixed_color_adjustment, 0x30);
        assert_eq!(display.travel_bird_tile_offset, 0x08);
        assert!(display.has_travel_bird_tile_upload());
        assert_eq!(display.star_tile_restore_phase, 0);
        assert_eq!(display.star_tile_restore_source_offsets(), (0, 32));
        assert_eq!(display.animated_tile_data_source_address, 0xac80);
        assert_eq!(display.animated_tile_data_source_usize(), 0xac80);
        assert!(display.has_animated_tile_data_source());
        assert_eq!(display.animated_tile_vram_destination_address, 0x3c00);
        assert_eq!(display.animated_tile_vram_destination_usize(), 0x3c00);
        assert_eq!(ram[INIDISP_COPY], 0x80);
        assert_eq!(ram[NMI_BOOLEAN], 1);
        assert_eq!(ram[NMI_DISABLE_CORE_UPDATES], 7);
        assert_eq!(ram[NMI_SUBROUTINE_INDEX], 11);
        assert_eq!(ram[NMI_LOAD_BG_FROM_VRAM], 5);
        assert_eq!(ram[NMI_UPDATE_TILEMAP_DST], 0x54);
        assert_eq!(read_le_u16(&ram, NMI_UPDATE_TILEMAP_SRC), 0x0800);
        assert_eq!(ram[BGMODE_COPY], 9);
        assert_eq!(ram[TM_COPY], 0x11);
        assert_eq!(ram[TS_COPY], 0x02);
        assert_eq!(ram[W12SEL_COPY], 0x33);
        assert_eq!(ram[W34SEL_COPY], 3);
        assert_eq!(ram[WOBJSEL_COPY], 0x33);
        assert_eq!(ram[TMW_COPY], 0x11);
        assert_eq!(ram[TSW_COPY], 0x02);
        assert_eq!(ram[NMI_COPY_PACKETS_FLAG], 3);
        assert_eq!(ram[NMI_FLAG_UPDATE_POLYHEDRAL], 0xff);
        assert_eq!(ram[LOAD_CHR_HALFSLOT_EVEN_ODD], 12);
        assert_eq!(ram[NMI_THREAD_ACTIVE], 1);
        assert_eq!(read_le_u16(&ram, POLY_THREAD_STACK), 0x1f31);
        assert_eq!(ram[IRQ_FLAG], 0xff);
        assert_eq!(ram[VIRQ_TRIGGER], 0x70);
        assert_eq!(ram[DMA_HEAD_POINTER], 0x40);
        assert_eq!(ram[DMA_BODY_POINTER], 0x80);
        assert_eq!(ram[HDMAEN_COPY], 0x80);
        assert_eq!(ram[MOSAIC_LEVEL], 0x30);
        assert_eq!(ram[MOSAIC_COPY], 0x33);
        assert_eq!(ram[MOSAIC_TARGET_LEVEL], 0x0f);
        assert_eq!(ram[MOSAIC_INC_OR_DEC], 0);
        assert_eq!(read_le_u16(&ram, NMI_LOAD_TARGET_ADDR), 0x1234);
        assert_eq!(read_le_u16(&ram, VRAM_UPLOAD_OFFSET), 0xbbaa);
        assert_eq!(
            ram[crate::game_state::constants::nmi::VRAM_UPLOAD_DATA],
            0xcc
        );
        assert_eq!(
            read_le_u16(
                &ram,
                crate::game_state::constants::nmi::OVERWORLD_TILE_ATTR_BUFFER + 14
            ),
            0x1234
        );
        assert_eq!(
            read_le_u16(
                &ram,
                crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF + 4
            ),
            0x5678
        );
        assert_eq!(
            read_le_u16(
                &ram,
                crate::game_state::constants::nmi::VRAM_UPLOAD_TILE_BUF + 6
            ),
            0xffff
        );
        assert_eq!(ram[INCREMENTAL_COUNTER_FOR_VRAM], 0);
        assert_eq!(ram[STAR_TILE_RESTORE_PHASE], 0);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_3), 0x9000);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_0), 0x9001);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_4), 0x9002);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_1), 0x9003);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_5), 0x9004);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_2), 0x9005);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_20), 0x9014);
        assert_eq!(read_le_u16(&ram, DMA_SOURCE_ADDR_21), 0x9015);
        assert_eq!(read_le_u16(&ram, BG_TILE_ANIMATION_COUNTDOWN), 0xffff);
        assert_eq!(
            read_le_u16(&ram, messaging_constants::MESSAGE_DMA_DST_ADDR),
            0x6080
        );
        assert_eq!(ram[OVERWORLD_FIXED_COLOR_PLUSMINUS], 0x30);
        assert_eq!(ram[FLAG_TRAVEL_BIRD], 0x08);
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_DATA_SRC), 0xac80);
        assert_eq!(read_le_u16(&ram, ANIMATED_TILE_VRAM_ADDR), 0x3c00);
    }

    #[test]
    fn shared_message_timer_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, SHARED_MESSAGE_TIMER, 0x0200);

        let mut timer = SharedMessageTimerState::load_from_ram(&ram);
        assert_eq!(timer.timer, 0x0200);
        assert_eq!(timer.tick(), 0x01ff);
        timer.clear();
        timer.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, SHARED_MESSAGE_TIMER), 0);
    }

    #[test]
    fn native_shared_message_timer_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, SHARED_MESSAGE_TIMER, 0x0000);

        let mut timer = SharedMessageTimerState::default();
        {
            let mut bridge = NativeSharedMessageTimerBridgeMut::new(&mut timer, &mut ram);
            assert_eq!(bridge.tick(), 0xffff);
            bridge.start(0x0df3);
            bridge.clear();
            bridge.start(0x0040);
        }

        assert_eq!(timer.timer, 0x0040);
        assert_eq!(read_le_u16(&ram, SHARED_MESSAGE_TIMER), 0x0040);
    }

    #[test]
    fn ppu_scroll_copy_state_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, BG1_H_SCROLL_COPY, 0x1234);
        write_le_u16(&mut ram, BG2_X_SCROLL, 0x0100);
        write_le_u16(&mut ram, BG2_Y_SCROLL, 0x0200);
        write_le_u16(&mut ram, MAPBAK_CGWSEL, 0xabcd);
        ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 4].copy_from_slice(&[1, 2, 3, 4]);

        let mut scroll = PpuScrollCopyState::load_from_ram(&ram);
        assert_eq!(scroll.bg1_h_copy(), 0x1234);
        assert_eq!(scroll.bg1_h_copy_low(), 0x34);
        assert_eq!(scroll.bg2_h_copy2(), 0x0100);
        assert_eq!(scroll.bg2_v_copy2(), 0x0200);
        assert_eq!(scroll.mapbak_cgwsel_word(), 0xabcd);
        assert_eq!(&scroll.mapbak_palette_slice()[..4], &[1, 2, 3, 4]);

        scroll.add_bg2_h_copy2(0x10);
        scroll.add_bg2_copy2_for_axis_signed(true, -1);
        scroll.set_mapbak_cgwsel(0x55);
        scroll.copy_mapbak_palette_from(&[9, 8, 7]);
        scroll.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, BG2_X_SCROLL), 0x0110);
        assert_eq!(read_le_u16(&ram, BG2_Y_SCROLL), 0x01ff);
        assert_eq!(read_le_u16(&ram, MAPBAK_CGWSEL), 0xab55);
        assert_eq!(&ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 4], &[9, 8, 7, 4]);
    }

    #[test]
    fn native_ppu_scroll_copy_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, BG2_X_SCROLL, 0x0060);
        write_le_u16(&mut ram, BG2_Y_SCROLL, 0x0070);
        write_le_u16(&mut ram, CAMERA_Y_COORD_SCROLL_LOW, 0x1001);
        write_le_u16(&mut ram, CAMERA_X_COORD_SCROLL_LOW, 0x2002);
        ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 4].copy_from_slice(&[1, 2, 3, 4]);

        let mut scroll = PpuScrollCopyState::default();
        {
            let mut bridge = NativePpuScrollCopyBridgeMut::new(&mut scroll, &mut ram);
            bridge.cache_bg2_live_scroll();
            bridge.cache_camera_scroll();
            bridge.copy_bg2_live_to_bg1_live();
            bridge.copy_mapbak_palette_from(&[5, 6, 7]);
        }

        assert_eq!(scroll.bg1_h_copy2(), 0x0060);
        assert_eq!(scroll.bg1_v_copy2(), 0x0070);
        assert_eq!(read_le_u16(&ram, BG1_X_SCROLL), 0x0060);
        assert_eq!(read_le_u16(&ram, BG1_Y_SCROLL), 0x0070);
        assert_eq!(read_le_u16(&ram, BG2_H_SCROLL_COPY2_CACHED), 0x0060);
        assert_eq!(read_le_u16(&ram, BG2_V_SCROLL_COPY2_CACHED), 0x0070);
        assert_eq!(read_le_u16(&ram, CAMERA_Y_COORD_SCROLL_LOW_CACHED), 0x1001);
        assert_eq!(read_le_u16(&ram, CAMERA_X_COORD_SCROLL_LOW_CACHED), 0x2002);
        assert_eq!(&scroll.mapbak_palette_slice()[..4], &[5, 6, 7, 4]);
        assert_eq!(&ram[MAPBAK_PALETTE..MAPBAK_PALETTE + 4], &[5, 6, 7, 4]);
    }

    #[test]
    fn native_bg1_movement_accumulator_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, BG1_MOVE_CALC_BUFFER, 0x1203);

        let mut accumulator = Bg1MovementAccumulatorState::load_from_ram(&ram);
        {
            let mut bridge = NativeBg1MovementAccumulatorBridgeMut::new(&mut accumulator, &mut ram);
            bridge.set_y_subpixel(0x44);
            assert_eq!(bridge.advance_x_subpixel(0xf1), 0x0103);
        }

        assert_eq!(accumulator.x_subpixel(), 0x03);
        assert_eq!(accumulator.y_subpixel(), 0x44);
        assert_eq!(read_le_u16(&ram, BG1_MOVE_CALC_BUFFER), 0x0344);
    }

    #[test]
    fn native_dungeon_entrance_backup_bridge_caches_themes_and_clears_high_bytes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[OVERWORLD_TILE_THEME_INDEX] = 0x11;
        ram[MAIN_TILE_THEME_INDEX] = 0x22;
        ram[AUX_TILE_THEME_INDEX] = 0x33;
        ram[SPRITE_GRAPHICS_INDEX] = 0x44;
        ram[OVERWORLD_SCREEN_INDEX + 1] = 0xaa;
        ram[OVERLAY_INDEX + 1] = 0xbb;

        let mut backup = DungeonEntranceBackupState::default();
        {
            let mut bridge = NativeDungeonEntranceBackupBridgeMut::new(&mut backup, &mut ram);
            bridge.cache_exit_tile_themes();
            bridge.clear_overworld_screen_high();
            bridge.clear_overlay_high();
        }

        assert_eq!(backup.exit_tile_theme(0), 0x11);
        assert_eq!(backup.exit_tile_theme(1), 0x22);
        assert_eq!(backup.exit_tile_theme(2), 0x33);
        assert_eq!(backup.exit_tile_theme(3), 0x44);
        assert_eq!(backup.overworld_screen_high(), 0);
        assert_eq!(backup.overlay_high(), 0);
        assert_eq!(
            &ram[OVERWORLD_EXIT_TILE_THEME_INDEX..OVERWORLD_EXIT_TILE_THEME_INDEX + 4],
            &[0x11, 0x22, 0x33, 0x44]
        );
        assert_eq!(ram[OVERWORLD_SCREEN_INDEX + 1], 0);
        assert_eq!(ram[OVERLAY_INDEX + 1], 0);
    }

    #[test]
    fn native_overworld_sprite_flag_bridges_dual_write_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[OVERWORLD_SPRITE_PRESENCE + 3] = 0x12;
        ram[OVERWORLD_SPRITE_WAS_LOADED + 4] = 0b1010_0000;

        let mut presence = OverworldSpritePresenceState::default();
        {
            let mut bridge = NativeOverworldSpritePresenceBridgeMut::new(&mut presence, &mut ram);
            bridge.set_marker(3, 0x34);
        }
        assert_eq!(presence.marker(3), 0x34);
        assert_eq!(ram[OVERWORLD_SPRITE_PRESENCE + 3], 0x34);

        let mut loaded = OverworldSpriteLoadedState::default();
        {
            let mut bridge = NativeOverworldSpriteLoadedBridgeMut::new(&mut loaded, &mut ram);
            bridge.clear_loaded_mask(32, 0b0010_0000);
            bridge.set_loaded_mask(32, 0b0000_0010);
        }
        assert!(loaded.is_loaded(32, 0b0000_0010));
        assert!(!loaded.is_loaded(32, 0b0010_0000));
        assert_eq!(ram[OVERWORLD_SPRITE_WAS_LOADED + 4], 0b1000_0010);
    }

    #[test]
    fn room_bounds_loads_from_and_projects_to_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, ROOM_BOUNDS, 0x0010);
        write_le_u16(&mut ram, ROOM_BOUNDS + 2, 0x0020);
        write_le_u16(&mut ram, ROOM_BOUNDS + 8, 0x0030);
        write_le_u16(&mut ram, ROOM_BOUNDS + 10, 0x0040);

        let mut bounds = RoomBoundsState::load_from_ram(&ram);
        assert_eq!(bounds.y_bound(0), 0x0010);
        assert_eq!(bounds.y_bound(1), 0x0020);
        assert_eq!(bounds.x_bound(0), 0x0030);
        assert_eq!(bounds.x_bound(1), 0x0040);

        bounds.set_y_bound(2, 0x3000);
        bounds.set_x_bound(3, 0x4000);
        bounds.write_to_ram(&mut ram);

        assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 4), 0x3000);
        assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 14), 0x4000);
    }

    #[test]
    fn native_room_bounds_bridge_dual_writes_changes_from_native_state() {
        let mut ram = vec![0; WRAM_SIZE];
        write_le_u16(&mut ram, SCRATCH_0, 0x0aaa);
        write_le_u16(&mut ram, SCRATCH_1, 0x0bbb);

        let mut bounds = RoomBoundsState::default();
        bounds.set_y_bound(0, 0x0010);
        bounds.set_y_bound(1, 0x0020);
        bounds.set_y_bound(2, 0x0030);
        bounds.set_y_bound(3, 0x0040);
        bounds.set_x_bound(0, 0x0030);
        bounds.set_x_bound(1, 0x0040);
        bounds.set_x_bound(2, 0x0050);
        bounds.set_x_bound(3, 0x0060);
        bounds.write_to_ram(&mut ram);
        {
            let mut bridge = NativeRoomBoundsBridgeMut::new(&mut bounds, &mut ram);
            bridge.add_y_bounds_a(0x0005);
            bridge.add_x_bounds_b(0x0007);
            bridge.copy_y_bound_from(1, SCRATCH_0);
            bridge.copy_x_bound_from(0, SCRATCH_1);
            bridge.set_packed_bounds(0x1000, 0x2000, 0x3000, 0x4000);
        }

        assert_eq!(bounds.packed_top(), 0x1000);
        assert_eq!(bounds.packed_bottom(), 0x2000);
        assert_eq!(bounds.packed_left(), 0x3000);
        assert_eq!(bounds.packed_right(), 0x4000);
        assert_eq!(bounds.x_bound(0), 0x0bbb);
        assert_eq!(bounds.x_bound(1), 0x0047);
        assert_eq!(read_le_u16(&ram, ROOM_BOUNDS), 0x1000);
        assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 2), 0x2000);
        assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 4), 0x3000);
        assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 6), 0x4000);
        assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 8), 0x0bbb);
        assert_eq!(read_le_u16(&ram, ROOM_BOUNDS + 10), 0x0047);
    }

    #[test]
    fn native_room_bounds_bridge_projects_native_state_over_stale_ram() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut bounds = RoomBoundsState::default();
        bounds.set_y_bound(0, 0x0010);
        bounds.set_y_bound(1, 0x0020);
        bounds.set_y_bound(2, 0x0030);
        bounds.set_y_bound(3, 0x0040);
        bounds.set_x_bound(0, 0x0050);
        bounds.set_x_bound(1, 0x0060);
        bounds.set_x_bound(2, 0x0070);
        bounds.set_x_bound(3, 0x0080);
        bounds.write_to_ram(&mut ram);

        write_le_u16(&mut ram, ROOM_BOUNDS, 0xaaaa);
        write_le_u16(&mut ram, ROOM_BOUNDS + 8, 0xbbbb);

        {
            let mut bridge = NativeRoomBoundsBridgeMut::new(&mut bounds, &mut ram);
            bridge.set_y_bound(1, 0x1234);
        }

        assert_eq!(bounds.y_bound(0), 0x0010);
        assert_eq!(bounds.y_bound(1), 0x1234);
        assert_eq!(bounds.x_bound(0), 0x0050);
        assert_eq!(RoomBoundsState::load_from_ram(&ram), bounds);
    }

    #[test]
    fn native_failed_spin_sparkle_spawn_bridge_dual_writes_spawn_record() {
        let mut ram = vec![0; WRAM_SIZE];
        let mut spawn = FailedSpinSparkleSpawnState::default();
        {
            let mut bridge = NativeFailedSpinSparkleSpawnBridgeMut::new(&mut spawn, &mut ram);
            bridge.write_failed_spin_sparkle(0x07, 0x1234, 0x5678);
        }

        assert_eq!(spawn.step(), 0x07);
        assert_eq!(spawn.timer(), 4);
        assert_eq!(spawn.aux_timer(), 3);
        assert_eq!(spawn.x(), 0x1234);
        assert_eq!(spawn.y(), 0x5678);
        assert_eq!(ram[ANCILLA_ITEM_TO_LINK - 1], 0);
        assert_eq!(ram[ANCILLA_STEP - 1], 0x07);
        assert_eq!(ram[ANCILLA_TIMER - 1], 4);
        assert_eq!(ram[ANCILLA_AUX_TIMER - 1], 3);
        assert_eq!(ram[ANCILLA_X_LO - 1], 0x34);
        assert_eq!(ram[ANCILLA_X_HI - 1], 0x12);
        assert_eq!(ram[ANCILLA_Y_LO - 1], 0x78);
        assert_eq!(ram[ANCILLA_Y_HI - 1], 0x56);
    }

    #[test]
    fn native_garnish_runtime_bridge_syncs_seeded_ram_and_dual_writes_changes() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[GARNISH_ACTIVE] = 0x03;
        ram[OVERWORLD_BOULDER_TRAP_COUNT] = 0xff;
        ram[OVERWORLD_BOULDER_TRAP_TIMER] = 0x7f;
        ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH] = 0x22;
        ram[REPULSESPARK_ANIM_DELAY] = 0x00;
        write_le_u16(&mut ram, SPRCOLL_X_BASE, 0x1234);
        write_le_u16(&mut ram, SPRCOLL_Y_BASE, 0x5678);

        let mut garnish = GarnishRuntimeState::default();
        {
            let mut bridge = NativeGarnishRuntimeBridgeMut::new(&mut garnish, &mut ram);
            bridge.set_active_type(0x0a);
            bridge.increment_boulder_trap_count();
            assert_eq!(bridge.increment_boulder_trap_timer(), 0x80);
            bridge.set_sprcoll_x_size(0x0102);
            bridge.set_sprcoll_y_size(0x0304);
            bridge.set_sprcoll_x_base(0x1112);
            bridge.set_sprcoll_y_base(0x1314);
            assert_eq!(bridge.decrement_repulsespark_anim_delay(), 0xff);
            bridge.clear_haunted_grove_flute_event_latch();
        }

        assert_eq!(garnish.active_type(), 0x0a);
        assert_eq!(garnish.boulder_trap_count(), 0x00);
        assert_eq!(garnish.boulder_trap_timer(), 0x80);
        assert_eq!(garnish.sprcoll_x_size(), 0x0102);
        assert_eq!(garnish.sprcoll_y_size(), 0x0304);
        assert_eq!(garnish.sprcoll_x_word(), 0x1112);
        assert_eq!(garnish.sprcoll_y_word(), 0x1314);
        assert_eq!(garnish.repulsespark_anim_delay(), 0xff);
        assert_eq!(garnish.haunted_grove_flute_event_latch(), 0);
        assert_eq!(ram[GARNISH_ACTIVE], 0x0a);
        assert_eq!(ram[OVERWORLD_BOULDER_TRAP_COUNT], 0);
        assert_eq!(ram[OVERWORLD_BOULDER_TRAP_TIMER], 0x80);
        assert_eq!(read_le_u16(&ram, SPRCOLL_X_SIZE), 0x0102);
        assert_eq!(read_le_u16(&ram, SPRCOLL_Y_SIZE), 0x0304);
        assert_eq!(read_le_u16(&ram, SPRCOLL_X_BASE), 0x1112);
        assert_eq!(read_le_u16(&ram, SPRCOLL_Y_BASE), 0x1314);
        assert_eq!(ram[REPULSESPARK_ANIM_DELAY], 0xff);
        assert_eq!(ram[HAUNTED_GROVE_FLUTE_EVENT_LATCH], 0);
    }

    #[test]
    fn native_follower_runtime_bridge_preserves_overlapping_timer_tail_byte() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[FOLLOWER_INDICATOR] = 0x04;
        ram[TAGALONG_DATA_INDEX] = 0x13;
        ram[TAGALONG_HOOKSHOT_INTERLOCK] = 0x02;
        ram[TIMER_TAGALONG_REACQUIRE] = 0x34;
        ram[FOLLOWER_TAIL_WRITE_INDEX] = 0x12;
        ram[TAGALONG_ANIM_FRAME_COUNTER] = 0x02;
        write_le_u16(&mut ram, FOLLOWER_SAVED_Y, 0x5678);
        write_le_u16(&mut ram, FOLLOWER_SAVED_X, 0x9abc);

        let mut follower = FollowerRuntimeState::default();
        {
            let mut bridge = NativeFollowerRuntimeBridgeMut::new(&mut follower, &mut ram);
            bridge.set_reacquire_timer(0xabcd);
            bridge.increment_tail_write_index();
            bridge.set_hookshot_release_tail_index_from_tail_write_index();
            bridge.advance_data_index_wrapping_at_20();
            bridge.increment_and_cycle_draw_anim_frame();
            bridge.set_saved_y(0x1112);
            bridge.set_saved_x(0x1314);
            bridge.set_palette_swap_flag(0x80);
        }

        assert_eq!(follower.reacquire_timer_low(), 0xcd);
        assert_eq!(follower.tail_write_index(), 0xac);
        assert_eq!(follower.reacquire_timer(), 0xaccd);
        assert_eq!(follower.hookshot_release_tail_index(), 0xac);
        assert_eq!(follower.data_index(), 0);
        assert_eq!(follower.draw_anim_frame(), 0);
        assert_eq!(follower.saved_y(), 0x1112);
        assert_eq!(follower.saved_x(), 0x1314);
        assert_eq!(follower.palette_swap_flag(), 0x80);
        assert_eq!(ram[TIMER_TAGALONG_REACQUIRE], 0xcd);
        assert_eq!(ram[FOLLOWER_TAIL_WRITE_INDEX], 0xac);
        assert_eq!(read_le_u16(&ram, TIMER_TAGALONG_REACQUIRE), 0xaccd);
        assert_eq!(ram[FOLLOWER_HOOKSHOT_RELEASE_TAIL_INDEX], 0xac);
        assert_eq!(ram[TAGALONG_DATA_INDEX], 0);
        assert_eq!(ram[TAGALONG_ANIM_FRAME_COUNTER], 0);
        assert_eq!(read_le_u16(&ram, FOLLOWER_SAVED_Y), 0x1112);
        assert_eq!(read_le_u16(&ram, FOLLOWER_SAVED_X), 0x1314);
        assert_eq!(ram[FOLLOWER_PALETTE_SWAP_FLAG], 0x80);
    }

    #[test]
    fn native_poly_structured_bridges_dual_write_projection_face_and_edge_state() {
        let mut ram = vec![0; WRAM_SIZE];
        ram[POLY_PROJECTED_X + 3] = 0x11;
        ram[POLY_PROJECTED_Y + 3] = 0x22;
        ram[POLY_FACE_COORDS] = 0x04;
        ram[POLY_FACE_COORDS + 5] = 0xaa;
        ram[POLY_TOTAL_NUM_STEPS] = 0x02;
        ram[POLY_X0_CUR] = 0x10;
        ram[POLY_Y0_CUR] = 0x20;
        ram[POLY_X1_CUR] = 0x30;
        ram[POLY_Y1_CUR] = 0x40;

        let mut projected = PolyProjectedVerticesState::default();
        let mut face = PolyFaceCoordsState::default();
        let mut edge = PolyRasterEdgeState::default();

        {
            let mut bridge = NativePolyProjectedVerticesBridgeMut::new(&mut projected, &mut ram);
            bridge.set_position(3, 0x55, 0x66);
        }
        {
            let mut bridge = NativePolyFaceCoordsBridgeMut::new(&mut face, &mut ram);
            bridge.set_xy_coords_count(0x08);
            bridge.set_coord(5, 0xbb);
        }
        {
            let mut bridge = NativePolyRasterEdgeBridgeMut::new(&mut edge, &mut ram);
            bridge.set_left_current(0x01, 0x02);
            bridge.set_right_target(0x03, 0x04);
            bridge.set_both_cur_vertex_idx(0x09);
            assert_eq!(bridge.decrement_total_num_steps(), 1);
            bridge.increment_y0_cur();
        }

        assert_eq!(projected.x(3), 0x55);
        assert_eq!(projected.y(3), 0x66);
        assert_eq!(face.xy_coords_count(), 0x08);
        assert_eq!(face.coord(5), 0xbb);
        assert_eq!(edge.x0_cur(), 0x01);
        assert_eq!(edge.y0_cur(), 0x03);
        assert_eq!(edge.x1_target(), 0x03);
        assert_eq!(edge.y1_trigger(), 0x04);
        assert_eq!(edge.cur_vertex_idx0(), 0x09);
        assert_eq!(edge.cur_vertex_idx1(), 0x09);
        assert_eq!(edge.total_num_steps(), 1);
        assert_eq!(ram[POLY_PROJECTED_X + 3], 0x55);
        assert_eq!(ram[POLY_PROJECTED_Y + 3], 0x66);
        assert_eq!(ram[POLY_FACE_COORDS], 0x08);
        assert_eq!(ram[POLY_FACE_COORDS + 5], 0xbb);
        assert_eq!(ram[POLY_X0_CUR], 0x01);
        assert_eq!(ram[POLY_Y0_CUR], 0x03);
        assert_eq!(ram[POLY_X1_TARGET], 0x03);
        assert_eq!(ram[POLY_Y1_TRIG], 0x04);
        assert_eq!(ram[POLY_CUR_VERTEX_IDX0], 0x09);
        assert_eq!(ram[POLY_CUR_VERTEX_IDX1], 0x09);
        assert_eq!(ram[POLY_TOTAL_NUM_STEPS], 1);
    }
}
