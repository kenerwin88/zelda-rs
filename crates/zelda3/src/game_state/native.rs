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

use crate::game_state::constants::MAIN_MODULE;

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
pub use sprites::CachedSpriteCacheField;
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
    /// Report bytes in the `$7F5800` ancilla scratch area that two projected native
    /// states would write with *different* values in the same projection.
    ///
    /// C aliases this window across mutually-exclusive effects (swordbeam / ether /
    /// quake / bombos / break-tower-seal / weathervane debris / happiness-pond rupees)
    /// and never re-stamps: whichever effect is running writes it and the others are
    /// dormant. The native migration models it with eight states that ALL bulk-project
    /// every frame, so last-writer-wins. That is only an actual clobber when two of them
    /// disagree about a byte, which this detects directly instead of by inspection.
    ///
    /// Driven by `ZELDA3_ASSERT_SCRATCH_CONFLICTS`; each entry is
    /// `(addr, winner, winner_value, loser, loser_value)` in projection order.
    ///
    /// The window defaults to the ancilla scratch and can be retargeted at any other
    /// shared range with `ZELDA3_SCRATCH_CONFLICT_RANGE=0xLO-0xHI` (HI exclusive), which
    /// is how the other C-aliased overlaps get measured rather than argued about.
    fn scratch_conflict_window() -> (usize, usize) {
        const DEFAULT: (usize, usize) = (0x15800, 0x15880);
        let Ok(spec) = std::env::var("ZELDA3_SCRATCH_CONFLICT_RANGE") else {
            return DEFAULT;
        };
        let parse = |t: &str| {
            let t = t.trim();
            usize::from_str_radix(t.strip_prefix("0x").unwrap_or(t), 16).ok()
        };
        match spec.split_once('-') {
            Some((a, b)) => match (parse(a), parse(b)) {
                (Some(a), Some(b)) if a < b => (a, b),
                _ => DEFAULT,
            },
            None => DEFAULT,
        }
    }

    pub(crate) fn report_scratch_conflicts(
        &self,
        ram: &[u8],
    ) -> Vec<(usize, &'static str, u8, &'static str, u8)> {
        let (lo, hi) = Self::scratch_conflict_window();

        // Which bytes a state WRITES, and with what — independent of whether the value
        // happens to match RAM. Run it twice over opposite sentinel fills; a byte it writes
        // lands on the same value both times, a byte it leaves alone keeps the sentinel.
        // (Comparing against live RAM instead would miss an owner writing the current
        // value, which is exactly the case that makes a shared byte look single-owned.)
        let claims = |f: &dyn Fn(&mut [u8])| -> Vec<(usize, u8)> {
            let mut zeros = ram.to_vec();
            let mut ones = ram.to_vec();
            zeros[lo..hi].fill(0x00);
            ones[lo..hi].fill(0xff);
            f(&mut zeros);
            f(&mut ones);
            (lo..hi)
                .filter(|&a| zeros[a] == ones[a])
                .map(|a| (a, zeros[a]))
                .collect()
        };

        // Attribute by top-level projection group, in GameState::write_to_ram order, so the
        // detector names an owner for ANY range — not just the states it was first written
        // for. Group granularity is enough to point at the file to read next.
        // Groups named in ZELDA3_SCRATCH_CONFLICT_SPLIT are expanded into their sub-states.
        // Needed whenever the two owners of a byte live in the SAME group: the group's own
        // write_to_ram has already resolved them last-writer-wins, so at group granularity
        // the conflict is invisible (this hid the 0x74 follower_link/tile_detection clobber
        // and the 0xb69 world.transient overrun until the group was split).
        let split = std::env::var("ZELDA3_SCRATCH_CONFLICT_SPLIT").unwrap_or_default();
        let split: Vec<&str> = split.split(',').map(|s| s.trim()).collect();
        let all: [(&'static str, &dyn Fn(&mut [u8])); 27] = [
            ("frame", &|r: &mut [u8]| self.frame.write_to_ram(r)),
            ("system_signals", &|r: &mut [u8]| {
                self.system_signals.write_to_ram(r)
            }),
            ("enhanced_features", &|r: &mut [u8]| {
                self.enhanced_features.write_to_ram(r)
            }),
            ("scratch_counter", &|r: &mut [u8]| {
                self.scratch_counter.write_to_ram(r)
            }),
            ("minigame", &|r: &mut [u8]| self.minigame.write_to_ram(r)),
            ("intro_sword", &|r: &mut [u8]| {
                self.intro_sword.write_to_ram(r)
            }),
            ("archery_game", &|r: &mut [u8]| {
                self.archery_game.write_to_ram(r)
            }),
            ("sprite_battle", &|r: &mut [u8]| {
                self.sprite_battle.write_to_ram(r)
            }),
            ("memorized_tiles", &|r: &mut [u8]| {
                self.memorized_tiles.write_to_ram(r)
            }),
            ("save_load_transfer", &|r: &mut [u8]| {
                self.save_load_transfer.write_to_ram(r)
            }),
            ("dungeon_map_display", &|r: &mut [u8]| {
                self.dungeon_map_display.write_to_ram(r)
            }),
            ("dungeon", &|r: &mut [u8]| self.dungeon.write_to_ram(r)),
            ("sprites", &|r: &mut [u8]| self.sprites.write_to_ram(r)),
            ("player.special_exit_position", &|r: &mut [u8]| {
                self.player.special_exit_position.write_to_ram(r)
            }),
            ("player.follower_link", &|r: &mut [u8]| {
                self.player.follower_link.write_to_ram(r)
            }),
            ("player.swim_acceleration", &|r: &mut [u8]| {
                self.player.swim_acceleration.write_to_ram(r)
            }),
            ("player.pushed_block", &|r: &mut [u8]| {
                self.player.pushed_block.write_to_ram(r)
            }),
            ("player.bg1_movement_accumulator", &|r: &mut [u8]| {
                self.player.bg1_movement_accumulator.write_to_ram(r)
            }),
            ("player.tile_detection", &|r: &mut [u8]| {
                self.player.tile_detection.write_to_ram(r)
            }),
            ("inventory", &|r: &mut [u8]| self.inventory.write_to_ram(r)),
            ("ending", &|r: &mut [u8]| self.ending.write_to_ram(r)),
            ("messaging", &|r: &mut [u8]| self.messaging.write_to_ram(r)),
            ("world", &|r: &mut [u8]| self.world.write_to_ram(r)),
            ("poly", &|r: &mut [u8]| self.poly.write_to_ram(r)),
            ("display", &|r: &mut [u8]| self.display.write_to_ram(r)),
            ("effects", &|r: &mut [u8]| self.effects.write_to_ram(r)),
            ("oam", &|r: &mut [u8]| self.oam.write_to_ram(r)),
        ];
        let owners: Vec<(&'static str, &dyn Fn(&mut [u8]))> = all
            .into_iter()
            .filter(|(name, _)| {
                // Keep a group unless it was split; keep a sub-state only when its group was.
                match name.split_once('.') {
                    Some((group, _)) => split.contains(&group),
                    None => !split.contains(name),
                }
            })
            .collect();

        // Two signals, selected by ZELDA3_SCRATCH_CONFLICT_MODE:
        //
        // `shared` (default) — a genuine dual-ownership clobber: two projected groups both
        //   WRITE the byte and disagree about its value, so the later one silently discards
        //   the earlier one's. This is the precise question for a shared address.
        // `stomp` — the broader "the frame-wide projection overwrote a live RAM byte".
        //   Useful when hunting an unknown writer, but on a byte with a single legitimate
        //   owner it also fires on ordinary native-vs-RAM drift, so it is not by itself
        //   evidence of dual ownership.
        let stomp_mode = std::env::var("ZELDA3_SCRATCH_CONFLICT_MODE")
            .map(|m| m == "stomp")
            .unwrap_or(false);

        if stomp_mode {
            let mut probe = ram.to_vec();
            self.write_to_ram(&mut probe);
            let stomped: Vec<usize> = (lo..hi).filter(|&a| probe[a] != ram[a]).collect();
            if stomped.is_empty() {
                return Vec::new();
            }
            let mut claimed: std::collections::BTreeMap<usize, (&'static str, u8)> =
                Default::default();
            for (name, project) in owners {
                for (addr, value) in claims(project) {
                    claimed.insert(addr, (name, value));
                }
            }
            return stomped
                .into_iter()
                .map(|addr| {
                    let (name, value) = claimed
                        .get(&addr)
                        .copied()
                        .unwrap_or(("<unattributed>", probe[addr]));
                    (addr, name, value, "live-ram", ram[addr])
                })
                .collect();
        }

        let mut claimed: std::collections::BTreeMap<usize, (&'static str, u8)> = Default::default();
        let mut out = Vec::new();
        for (name, project) in owners {
            for (addr, value) in claims(project) {
                if let Some(&(prev_name, prev_value)) = claimed.get(&addr) {
                    if prev_value != value {
                        out.push((addr, name, value, prev_name, prev_value));
                    }
                }
                claimed.insert(addr, (name, value));
            }
        }
        out
    }

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
        if ram_byte(ram, MAIN_MODULE) != 7 {
            check!(save_load_transfer);
        }
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
