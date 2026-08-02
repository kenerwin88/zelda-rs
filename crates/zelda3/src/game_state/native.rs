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
mod system;
mod world;

pub(crate) use display::{
    palette_provenance_check_mode, DisplayState, GraphicsDecompressionScratch, HudStateRead,
    HudTilemapState, LinkDmaSourceSlot, LinkDmaSources, NativeAttractVramDestinationBridgeMut,
    NativeDisplayStateBridgeMut, NativeHudInventoryOrderBridgeMut, NativeHudStateBridgeMut,
    NativeOverworldPaletteBackupBridgeMut, NativePaletteBufferBridgeMut,
    NativePaletteFilterBridgeMut, NativePpuScrollCopyBridgeMut, NativeSpotlightHdmaBridgeMut,
    NativeTrinexxPaletteBridgeMut, NativeVramUploadBufferBridgeMut, NativeWaterHdmaWindowBridgeMut,
    PaletteSliceSource, PaletteTransform, PpuScrollCopyState, ProvenanceCheckMode,
    SpotlightHdmaState,
};
pub(crate) use dungeon::{
    loaded_room_data_word, DungeonStairList, DungeonState, NativeDungeonBg2AttributeBridgeMut,
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
    lanmola_flat_trail_entry_from_ram, BlastWallExplosionSlotState, BlastWallFireballSlotState,
    BlastWallFragmentSlotState, BombosBlastState, BombosFireColumnState, EffectState,
    HappinessPondRupeeSlotState, HappinessPondRupeeSnapshot, HistoryPositionState,
    LanmolaFlatTrailEntry, LanmolaSegmentMotionState, NativeBeamosLaserHistoryBridgeMut,
    NativeBlastWallBridgeMut, NativeBlastWallExplosionBridgeMut, NativeBlastWallFireballBridgeMut,
    NativeBlastWallFragmentBridgeMut, NativeBombosBlastBridgeMut, NativeBombosFireColumnBridgeMut,
    NativeBombosSpellBridgeMut, NativeDiggingGamePrizeBridgeMut, NativeDoorDebrisBridgeMut,
    NativeEffectAngleScratchBridgeMut, NativeHappinessPondRupeeBridgeMut,
    NativeLanmolaSegmentMotionBridgeMut, NativeMoldormHistoryBridgeMut, NativeQuakeBoltBridgeMut,
    NativeQuakeSpellBridgeMut, NativeSkullWoodsFireBridgeMut, NativeSkullWoodsFireSlotBridgeMut,
    NativeSwamolaHistoryBridgeMut, NativeSwamolaTargetBridgeMut, NativeTowerSealBridgeMut,
    NativeTowerSealOrbitBridgeMut, NativeTowerSealSparkleBridgeMut,
    NativeWeatherVaneDebrisBridgeMut, QuakeBoltSlotState, SkullWoodsFireSlotState,
    TowerSealOrbitState, TowerSealSparkleState, WeatherVaneDebrisSlotState,
};
pub(crate) use ending::{
    AttractSceneState, EndingState, IntroActorRead, IntroActorState, NativeAttractSceneBridgeMut,
    NativeEndingCreditBridgeMut, NativeIntroActorBridgeMut, NativeIntroSceneBridgeMut,
};
pub(crate) use frame::{FrameState, NativeFrameStateBridgeMut};
pub(crate) use inventory::{
    InventoryState, NativeDungeonKeySlotsBridgeMut, NativeInventoryItemsBridgeMut,
    NativeMirrorWarpBridgeMut, NativePlayerResourcesBridgeMut, NativeSaveProgressBridgeMut,
    SaveProgressState,
};
pub(crate) use messaging::{
    MessagingRenderBufferState, MessagingState, MultiselectChoiceRead,
    NativeDecodedMessageTextBridgeMut, NativeDialogueMessageIndexBridgeMut,
    NativeDialogueNumberBridgeMut, NativeDialogueSourceOffsetBridgeMut,
    NativeMessagingRenderBufferBridgeMut, NativeMessagingRuntimeBridgeMut,
    NativeMultiselectChoiceBridgeMut, NativeSelectFileMenuBridgeMut,
    NativeSharedMessageTimerBridgeMut, NativeVwfRenderBridgeMut, SelectFileMenuState,
    VwfRenderState,
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
    Bg1MovementAccumulatorState, FollowerLinkState, NativeBg1MovementAccumulatorBridgeMut,
    NativeFollowerLinkBridgeMut, NativePushedBlockBridgeMut, NativeSpecialExitPositionBridgeMut,
    NativeSwimAccelerationBridgeMut, NativeTileDetectionBridgeMut, PlayerState,
};
pub(crate) use poly::{
    NativePolyFaceCoordsBridgeMut, NativePolyProjectedVerticesBridgeMut,
    NativePolyRasterEdgeBridgeMut, NativePolyRuntimeBridgeMut, PolyState,
};
pub(crate) use sprites::{
    BossHomePositionRead, CachedSpriteRead, NativeAncillaSlotBridgeMut, NativeAncillaSlotView,
    NativeArmosKnightHomePositionBridgeMut, NativeArrghusPuffHomePositionBridgeMut,
    NativeCachedSpriteBridgeMut, NativeChainChompHistoryBridgeMut,
    NativeDualLayerTileCacheBridgeMut, NativeEnemyDamageSubclassTableBridgeMut,
    NativeEtherOrbitBridgeMut, NativeFailedSpinSparkleSpawnBridgeMut,
    NativeFollowerRuntimeBridgeMut, NativeGarnishRuntimeBridgeMut, NativeGarnishSlotBridgeMut,
    NativeGarnishSlotView, NativeMazeGameTimerBridgeMut, NativeOverlordSlotBridgeMut,
    NativeOverlordSlotView, NativeOverworldSpriteLoadedBridgeMut,
    NativeOverworldSpritePresenceBridgeMut, NativePrizeDropCycleBridgeMut,
    NativeSpriteDrawWorkPositionBridgeMut, NativeSpriteHitboxWorkOffsetBridgeMut,
    NativeSpriteSlotBridgeMut, NativeSpriteSlotView, NativeSpriteSystemBridgeMut,
    NativeSpriteWorkspaceBridgeMut, NativeTagalongSlotBridgeMut, SpriteState, TagalongSlotRead,
    OVERWORLD_SPRITE_FLAG_COUNT,
};
pub(crate) use system::{
    MsuResumeInfoState, MsuResumeSlot, NativeSystemSignalsBridgeMut, SystemSignalsState,
    SystemWorkArea,
};
pub(crate) use world::{
    BirdTravelDestinationState, NativeBirdTravelDestinationBridgeMut,
    NativeOverworldConfigTableBridgeMut, NativeOverworldEntranceBridgeMut,
    NativeOverworldEventInfoBridgeMut, NativeOverworldExitBridgeMut, NativeOverworldMap16BridgeMut,
    NativeOverworldMapUiBridgeMut, NativeOverworldMapZoomBridgeMut,
    NativeOverworldScreenSizeBridgeMut, NativeOverworldScrollDeltaBridgeMut,
    NativeOverworldTransitionBridgeMut, NativeRoomBoundsBridgeMut, NativeWeatherVaneBridgeMut,
    NativeWorldCameraBoundariesBridgeMut, NativeWorldLocationBridgeMut,
    NativeWorldPaletteThemeBridgeMut, NativeWorldRegionBridgeMut, NativeWorldScrollBridgeMut,
    NativeWorldTransientBridgeMut, OverworldConfigTableRead, OverworldMap16Decode,
    OverworldMap16DecodeScratch, OverworldMap16SourcePage, WorldLocationState, WorldState,
};
pub use world::{OverworldMap16LoadState, SmallOverworldMap16ScrollBackupState};

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

    /// Compare every native sub-state to a fresh `load_from_ram(ram)` and return the
    /// names of those that differ. A non-empty result means the native model has drifted
    /// out of sync with RAM — the dominant bug class in this migration (a stale native
    /// field that will re-project over RAM, or RAM written directly without updating the
    /// native model, e.g. the cached-sprite uncache / clear_room_parser_words family).
    ///
    /// Some sub-states legitimately differ mid-frame (gated/mode-reuse projections that
    /// only mirror RAM in one game mode), so a few names form a stable baseline; a real
    /// coherence bug shows up as a state going incoherent at the step that introduced it.
    /// Driven by `replay_trace_ram_watch` under `ZELDA3_ASSERT_NATIVE_COHERENT`.
    pub(crate) fn report_incoherent_with_ram(&self, ram: &[u8]) -> Vec<&'static str> {
        let fresh = Self::load_from_ram(ram);
        let mut out = Vec::new();
        macro_rules! check {
            ($field:ident) => {
                if self.$field != fresh.$field {
                    out.push(stringify!($field));
                }
            };
        }
        check!(frame);
        check!(system_signals);
        check!(enhanced_features);
        check!(scratch_counter);
        check!(minigame);
        check!(intro_sword);
        check!(archery_game);
        check!(sprite_battle);
        check!(memorized_tiles);
        check!(dungeon_secret);
        check!(save_load_transfer);
        if self.dungeon_map_display != fresh.dungeon_map_display {
            out.extend(self.dungeon_map_display.report_incoherent_with_ram(ram));
        }
        // sprites/dungeon/world are the most composite/bug-prone states — drill to the leaf so a
        // faithful leaf (e.g. sprite_slots, world.location) flagging stands out from intentionally
        // noisy gated/mode-reuse composites.
        if self.dungeon != fresh.dungeon {
            out.extend(self.dungeon.report_incoherent_with_ram(ram));
        }
        if self.sprites != fresh.sprites {
            out.extend(self.sprites.report_incoherent_with_ram(ram));
        }
        check!(player);
        check!(inventory);
        if self.world != fresh.world {
            out.extend(self.world.report_incoherent_with_ram(ram));
            if self.world.transient != fresh.world.transient {
                out.extend(self.world.transient.report_incoherent_with_ram(ram));
            }
        }
        check!(poly);
        check!(display);
        check!(effects);
        check!(ending);
        check!(messaging);
        check!(oam);
        out
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
        // DUNGEON_SECRET_PENDING_KIND / OVERWORLD_SECRET_SUBST_CTR are mode-reused
        // scratch. The explicit dungeon-secret bridge owns write-through updates, but
        // the frame-wide projection must preserve C's stale scratch bytes.
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
mod native_tests;
