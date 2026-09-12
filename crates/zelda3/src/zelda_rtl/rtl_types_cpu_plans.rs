//! Types split out of `zelda_rtl.rs` by family (types_cpu_plans).
//! Mechanical move: definitions are unchanged; items and struct fields
//! became `pub(crate)` so the parent module and its siblings see them.

use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct StripeUploadWork {
    pub(crate) packets: usize,
    pub(crate) transfer_bytes: usize,
    pub(crate) fixed_source_packets: usize,
    pub(crate) vertical_packets: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OverworldAuxGraphicsWorkload {
    pub(crate) background_packs_to_decompress: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OverworldAuxGraphicsTiming {
    pub(crate) load_nmi_slices: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OverworldMapGraphicsWorkload {
    pub(crate) map32_definition_changes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OverworldMapAndSpriteGraphicsTiming {
    pub(crate) quadrant_load_nmi_slices: u8,
    pub(crate) map16_to_map8_tail_nmi_slices: u8,
    pub(crate) scroll_map_and_sprite_gfx_tail_nmi_slices: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OverworldSpriteReloadWorkload {
    pub(crate) sprite_records: usize,
    pub(crate) in_bounds_proximity_checks: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverworldSpriteReloadEntryPhase {
    OrdinaryModuleIteration,
    VblankEdgeAfterGraphicsTail,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreMainNmiResume {
    OverworldAuxGraphicsReturn,
    OverworldSpriteReloadReturn {
        scanout: OverworldSpriteReloadResumeScanout,
    },
    DungeonSupertileQuadrantUploads,
    DungeonSupertileQuadrantUploadsAfterHeldNmi,
    DungeonSupertileCallerReturnNmi,
    DungeonSupertileNextIterationAfterLeadingNmi,
    DungeonModuleCallerCompletedBeforeNextNmi,
}

/// Caller suffixes that resume before the next fresh module iteration.
///
/// These continuations are mutually exclusive on the 65816 call stack. Keep
/// that invariant explicit instead of representing one program counter with
/// several booleans that could all be true at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreMainCallerContinuation {
    DialogueVwfReturn,
    FileSelectCheckerboardUpload,
    NamePlayerTilemapUpload,
    DungeonFadedFilterSecondPalettePass { resumed_phase: ModuleCpuPhase },
    SpiralStairsSecondPaletteFilter,
    SpiralStairsSecondGrayscalePaletteFilter,
}

/// Shared `ZeldaRunGameLoop` caller suffix which remains below an interrupted
/// `Module_MainRouting` call. This is a source-level continuation, not an NMI
/// cadence rule: once the module returns, the 65816 runs exactly one
/// `NMI_PrepareSprites` call and clears `$12` before the next accepted NMI can
/// publish display memory or joypads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MainLoopCommonSuffixContinuation {
    PrepareSpritesAndClearNmiLatch,
    ResumeSpritePreparationExtendedOamPackingAndClearNmiLatch { next_group_start: u8 },
    ResumeSpritePreparationBytePackingAndClearNmiLatch { progress: SpritePreparationProgress },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpritePreparationProgress {
    ExtendedOam(ExtendedOamPackingProgress),
    PointerTail(SpritePreparationPointerProgress),
}

/// Instruction-boundary progress through $874e-$8780. Each STA commits a
/// complete source word; the three pointer pairs precede SEP and RTS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SpritePreparationPointerProgress {
    pub(crate) master_cycles: u16,
}

impl SpritePreparationPointerProgress {
    pub(crate) fn completed_words(self) -> usize {
        assert!(self.master_cycles <= 610);
        [118, 182, 300, 364, 482, 546].into_iter()
            .filter(|&cycles| cycles <= self.master_cycles).count()
    }
}

/// A suspended pass of $0085FE-$00865A. Stores within a pass commit in
/// ascending byte order; passes themselves visit 28,24,...,0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ExtendedOamPackingProgress {
    pub(crate) group_start: u8,
    pub(crate) completed_bytes: u8,
    pub(crate) group_master_cycles: u16,
}

impl ExtendedOamPackingProgress {
    pub(crate) fn before_group(group_start: u8) -> Self {
        Self { group_start, completed_bytes: 0, group_master_cycles: 0 }
    }

    pub(crate) fn validate(self) {
        assert!(self.group_start <= 28 && self.group_start & 3 == 0);
        assert!(self.completed_bytes <= 4);
        let group_total = if self.group_start == 0 { 1128 } else { 1134 };
        assert!(self.group_master_cycles <= group_total);
        // TYA/ASL/ASL/TAX56, then each packed byte's four reads, six
        // shifts and store250. A store is committed only after its access.
        let stores = [306, 556, 806, 1056];
        assert_eq!(usize::from(self.completed_bytes), stores.into_iter()
            .filter(|&cycle| cycle <= self.group_master_cycles).count());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeOverworldSongUpload {
    CommandAt { host: u32, position: CpuRasterPosition },
    AwaitReturn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NmiPhase {
    BeforeNmi,
    AfterNmi,
}

impl NmiPhase {
    pub(crate) const fn return_bg_scroll_generation(self) -> DisplayBgScrollGeneration {
        match self {
            Self::BeforeNmi => DisplayBgScrollGeneration::ComposeLiveAfterNmi,
            Self::AfterNmi => DisplayBgScrollGeneration::RetainCapturedBeforeNmi,
        }
    }
}

impl PreMainNmiResume {
    /// These continuations carry only the native estimate that a pending NMI
    /// publishes before the next `ZeldaRunGameLoop` iteration. Once the Live
    /// authority reports that fresh iteration, the source boundary has
    /// completed this timing shadow; the translated module iteration still
    /// runs below from the semantic receipt.
    pub(crate) const fn is_timing_shadow_completed_by_fresh_iteration(self) -> bool {
        matches!(
            self,
            Self::DungeonSupertileQuadrantUploads
                | Self::DungeonSupertileQuadrantUploadsAfterHeldNmi
                | Self::DungeonSupertileNextIterationAfterLeadingNmi
                // The reload return's remaining semantics are the display
                // generations its leading NMI consumes; the fresh-iteration
                // plan stages them before its handler (route host 38519).
                | Self::OverworldSpriteReloadReturn { .. }
        )
    }

    pub(crate) const fn nmi_latch_clear_phase(self) -> Option<NmiPhase> {
        match self {
            Self::DungeonSupertileQuadrantUploads
            | Self::DungeonSupertileQuadrantUploadsAfterHeldNmi
            | Self::DungeonSupertileNextIterationAfterLeadingNmi => Some(NmiPhase::BeforeNmi),
            Self::DungeonSupertileCallerReturnNmi
            | Self::DungeonModuleCallerCompletedBeforeNextNmi => Some(NmiPhase::AfterNmi),
            Self::OverworldAuxGraphicsReturn | Self::OverworldSpriteReloadReturn { .. } => None,
        }
    }

    pub(crate) const fn continues_after_main(self, frame: crate::game_state::FrameState) -> bool {
        if frame.main_module != 7 {
            return false;
        }
        matches!(
            self,
            Self::DungeonSupertileQuadrantUploads
                | Self::DungeonSupertileQuadrantUploadsAfterHeldNmi
        ) && ((frame.submodule == 2 && matches!(frame.subsubmodule, 5..=15))
            || (frame.submodule == 0x0e && matches!(frame.subsubmodule, 8..=15)))
    }

    pub(crate) const fn scanout_generations(self) -> PreMainNmiScanoutGenerations {
        match self {
            Self::OverworldAuxGraphicsReturn => PreMainNmiScanoutGenerations {
                publication: DisplaySnapshotPublication::PublishCaptured,
                vram: DisplayVramGeneration::ComposeLiveAfterNmi,
                animated_bg: None,
                bg_scroll: DisplayBgScrollGeneration::ComposeLiveAfterNmi,
                obj: None,
            },
            Self::OverworldSpriteReloadReturn { scanout } => scanout.generations(),
            Self::DungeonSupertileQuadrantUploads
            | Self::DungeonSupertileQuadrantUploadsAfterHeldNmi
            | Self::DungeonSupertileCallerReturnNmi
            | Self::DungeonSupertileNextIterationAfterLeadingNmi
            | Self::DungeonModuleCallerCompletedBeforeNextNmi => PreMainNmiScanoutGenerations {
                publication: DisplaySnapshotPublication::PublishCaptured,
                vram: DisplayVramGeneration::ComposeLiveAfterNmi,
                // This continuation captures before a leading hardware NMI.
                // Its animated-CHR DMA and BG scroll-register writes therefore
                // belong to the active scanout, unlike the ordinary
                // main-then-next-NMI cadence.
                animated_bg: Some(AnimatedBgScanoutGeneration::LiveAfterNmi),
                bg_scroll: DisplayBgScrollGeneration::ComposeLiveAfterNmi,
                obj: Some(ObjScanoutGenerations {
                    oam: OamScanoutSource::ComposeLiveAfterNmi,
                    // Link's CHR DMA runs in the same leading NMI as the
                    // animated-BG transfer. The resumed main thread does not
                    // make that upload belong to the following scanout.
                    link_obj: GraphicsDmaGeneration::LiveAfterMain,
                    link_obj_sources: GraphicsDmaGeneration::LiveAfterMain,
                }),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DialogueInitializationCpuPlan {
    pub(crate) prefix_nmi_crossings: u8,
    pub(crate) nmi_crossings: u8,
    pub(crate) final_interrupted_pc: u32,
    pub(crate) return_scanline: u16,
    pub(crate) return_master_cycle: u16,
    pub(crate) following_main_nmi_uses_host_animated_bg_operands: bool,
}

impl DialogueInitializationCpuPlan {
    pub(crate) const fn prefix_nmi_crossings(self) -> u8 {
        self.prefix_nmi_crossings
    }

    pub(crate) const fn caller_nmi_crossings(self) -> u8 {
        self.nmi_crossings - self.prefix_nmi_crossings
    }

    pub(crate) const fn schedule_key(self) -> (u8, u8, bool) {
        (
            self.prefix_nmi_crossings,
            self.nmi_crossings,
            self.following_main_nmi_uses_host_animated_bg_operands,
        )
    }

    pub(crate) const fn following_main_nmi_uses_host_animated_bg_operands(self) -> bool {
        self.following_main_nmi_uses_host_animated_bg_operands
    }

    pub(crate) const fn diagnostic(self) -> (u8, u8, u32, u16, u16, bool) {
        (
            self.prefix_nmi_crossings,
            self.nmi_crossings,
            self.final_interrupted_pc,
            self.return_scanline,
            self.return_master_cycle,
            self.following_main_nmi_uses_host_animated_bg_operands,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DungeonRoomLoadCpuSchedule {
    pub(crate) room_load_nmis: u8,
    pub(crate) auxiliary_graphics_nmis: u8,
    pub(crate) caller_nmis: u8,
    pub(crate) caller_prefix_nmis: u8,
    pub(crate) caller_sprite_main_nmis: u8,
    pub(crate) caller_suffix_nmis: u8,
    pub(crate) sprite_main_boundary: Option<SpriteMainCpuBoundary>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DungeonRoomLoadCpuPlan {
    pub(crate) room_load_return: CpuRasterPosition,
    pub(crate) auxiliary_graphics_entry: CpuRasterPosition,
    pub(crate) auxiliary_graphics_return: CpuRasterPosition,
    pub(crate) caller_return: CpuRasterPosition,
    pub(crate) room_load_nmis: u8,
    pub(crate) auxiliary_graphics_nmis: u8,
    pub(crate) caller_nmis: u8,
    pub(crate) caller_prefix_nmis: u8,
    pub(crate) caller_sprite_main_nmis: u8,
    pub(crate) caller_suffix_nmis: u8,
    pub(crate) sprite_main_boundary: Option<SpriteMainCpuBoundary>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct DungeonSubmoduleCpuSchedule {
    pub(crate) reset_progress: Option<DungeonResetSpritesCpuProgress>,
    pub(crate) submodule_nmis: u8,
    pub(crate) caller_nmis: u8,
    pub(crate) caller_sprite_main_nmis: u8,
    pub(crate) caller_suffix_nmis: u8,
    pub(crate) caller_first_nmi_phase: Option<ModuleCpuPhase>,
    pub(crate) sprite_main_boundary: Option<SpriteMainCpuBoundary>,
    pub(crate) cached_sprite_interruption: Option<CachedSpriteCpuInterruption>,
    pub(crate) reenters_main_loop_before_nmi: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Module09CpuSchedule {
    pub(crate) submodule_nmis: u8,
    pub(crate) caller_nmis: u8,
    pub(crate) caller_sprite_main_nmis: u8,
    pub(crate) caller_suffix_nmis: u8,
    pub(crate) caller_first_nmi_phase: Option<ModuleCpuPhase>,
    pub(crate) sprite_main_boundary: Option<SpriteMainCpuBoundary>,
}

impl DungeonRoomLoadCpuPlan {
    pub(crate) const fn schedule(self) -> DungeonRoomLoadCpuSchedule {
        DungeonRoomLoadCpuSchedule {
            room_load_nmis: self.room_load_nmis,
            auxiliary_graphics_nmis: self.auxiliary_graphics_nmis,
            caller_nmis: self.caller_nmis,
            caller_prefix_nmis: self.caller_prefix_nmis,
            caller_sprite_main_nmis: self.caller_sprite_main_nmis,
            caller_suffix_nmis: self.caller_suffix_nmis,
            sprite_main_boundary: self.sprite_main_boundary,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DungeonQuadrantCpuAdvance {
    CompleteBeforeNmi,
    InterruptedInModule,
    InterruptedAfterModule,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ModuleCpuPhase {
    CompleteBeforeNmi,
    InterruptedBeforeSubmodule,
    InterruptedInSubmodule,
    InterruptedBeforeSpriteMain,
    InterruptedInSpriteMain,
    InterruptedAfterSpriteMain,
    InterruptedInLinkOam,
    InterruptedBeforeNmiPrepareSprites,
    InterruptedInNmiPrepareSprites,
    InterruptedAfterModule,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum SpriteMainCpuBoundary {
    BeforeFirstSlot,
    AfterSlot(u8),
    /// The current slot's shared timer/OAM prefix returned. `state` is absent
    /// on the source receipt and is bound by the native call site before the
    /// continuation is parked, matching Sprite_ExecuteSingle's saved state.
    AfterTimersAndOam {
        slot: u8,
        state: Option<u8>,
    },
    /// Every countdown update in the shared timer/OAM helper completed.
    /// `state` preserves Sprite_ExecuteSingle's pre-helper dispatch value.
    AfterTimerDecrements {
        slot: u8,
        state: Option<u8>,
    },
    /// A type-$23/$24 Bari initializer has completed its property/state and
    /// fixed pre-RNG publications. The RNG-backed aux-delay and lower slots
    /// remain pending.
    BariBeforeRandom(u8),
    /// Zelda's state-8 initializer is suspended inside the shared follower-
    /// graphics decompressor. `saved_follower_indicator` is filled only after
    /// native execution commits the initializer prefix; wire receipts carry
    /// only the source stage and slot.
    FollowerGraphics {
        slot: u8,
        caller: crate::SpriteFollowerGraphicsCaller,
        prefix_completed: bool,
        saved_follower_indicator: Option<u8>,
        stage: crate::RescuedMaidenInitializationStage,
    },
    /// The current type-$ec death slot published its timer/OAM prefix and
    /// terminal state clear. Its OAM-coordinate/garnish suffix and the lower
    /// Sprite_Main slots remain pending.
    AfterThrowableSceneryStateClear(u8),
    /// The active-Cucco branch completed `Sprite_MoveX`; `Sprite_MoveY` and
    /// the caller suffix remain pending.
    AfterActiveCuccoX {
        slot: u8,
        helper_ordinal: u8,
    },
    /// `Cucco_Flee` completed X movement and published only Y's subpixel
    /// assignment. The native caller fills the pending Y low/high result when
    /// it reaches this source statement so resume cannot recompute movement.
    AfterActiveCuccoYSubpixel {
        slot: u8,
        helper_ordinal: u8,
        y_low: Option<u8>,
        y_high: Option<u8>,
    },
    /// A master-sword light beam is suspended at an exact assignment in its
    /// `Sprite_MoveXY` call. Computed coordinate bytes are bound by native
    /// execution before the continuation is parked.
    MasterSwordLightBeamMovement {
        slot: u8,
        checkpoint: crate::SpriteMoveXYCheckpoint,
        continuation: Option<SpriteMoveXYContinuation>,
    },
    /// An indoor Boulder entered `Sprite_MoveXYZ`; its Z move ran and the
    /// XY coordinate bytes are bound natively before parking.
    BoulderMovement {
        slot: u8,
        checkpoint: crate::SpriteMoveXYCheckpoint,
        continuation: Option<SpriteMoveXYContinuation>,
    },
    /// A state-8 prep (`SpritePrep_Spike` / `SpritePrep_RockStal`) entered
    /// `Sprite_MoveY`; the pending y_low/y_high bytes are bound natively.
    InitializePrepMoveY {
        slot: u8,
        checkpoint: crate::SpriteMoveXYCheckpoint,
        continuation: Option<(u8, u8)>,
    },
    /// The same light beam has completed movement and published a prefix of
    /// its frame-gated replacement `Sprite_SpawnDynamically` call.
    MasterSwordLightBeamSpawn {
        slot: u8,
        spawned_slot: u8,
        progress: crate::SpriteDynamicSpawnProgress,
    },
    /// `Cucco_Flee` published its movement/Z/retarget prefix and is about to
    /// enter the shared subtype helper. The helper ordinal distinguishes this
    /// call from the optional state-10 helper in the same sprite invocation.
    AfterCuccoFleeMovement {
        slot: u8,
        helper_ordinal: u8,
    },
    /// The shared Cucco helper published a source-observed prefix of its byte
    /// increments but not its graphics store. `continuation` is absent when
    /// the replaceable timing authority first supplies the statement boundary;
    /// the native helper fills it from the C call site before scheduling.
    AfterCuccoSubtypeIncrements {
        slot: u8,
        helper_ordinal: u8,
        completed: u8,
        total: u8,
        continuation: Option<CuccoSubtypeContinuation>,
    },
    /// The shared Cucco helper published subtype and graphics but has not run
    /// `Sprite_ReturnIfLifted` or its caller suffix. The native call site owns
    /// the typed continuation; the oracle receipt exposes no CPU identity.
    AfterCuccoGraphicsPublication {
        slot: u8,
        helper_ordinal: u8,
        continuation: Option<CuccoSubtypeContinuation>,
    },
    /// `PrepareEnemyDrop` published type `$e5` and entered the synchronous
    /// big-key graphics call. The current slot has not returned.
    BigKeyDropGraphicsStarted(u8),
    /// King Zora published the purchased-flippers spawn and entered the
    /// synchronous `$11` animated-sheet decode. The current slot and every
    /// lower slot remain pending.
    KingZoraFlippersGraphicsStarted(u8),
    /// The current slot entered `Link_ReceiveItem`'s synchronous graphics
    /// call. Its typed item-receipt continuation, rather than the outer loop,
    /// owns the suspended suffix.
    ItemReceiptGraphicsStarted(u8),
    /// Wish Pond case 2 has removed the selected inventory item, spawned the
    /// tossed-item ancilla, and entered that helper's synchronous animated-
    /// sheet decode. The backend receipt carries no call-local data; native
    /// execution binds the exact suffix before parking the continuation.
    WishPondTossedItemGraphics {
        slot: u8,
        continuation: Option<WishPondTossedItemGraphicsContinuation>,
    },
    /// Sprite state/property initialization and Zelda's prep prefix returned,
    /// then NMI interrupted the shared follower-graphics loader. The saved
    /// indicator is filled by the translated prefix before scheduling.
    BeforeZeldaFollowerGraphics(u8),
    AfterZeldaFollowerGraphics {
        slot: u8,
        saved_follower_indicator: u8,
    },
    /// The named slot's state/property initialization and the bonk item's
    /// floor assignment returned, then NMI interrupted the synchronous
    /// `DecodeAnimatedSpriteTile_variable($0e)` sheet decode inside
    /// `SpritePrep_BonkItem` (dungeon room `$107` without the Book of
    /// Mudora). The semantic wire names this caller directly, so native never
    /// infers it from a room, sprite type, or neighboring slot.
    BonkItemGraphicsEntered(u8),
    /// `SpriteDraw_SingleSmall` published the current slot's timer/OAM and
    /// position prefix. The native call site fills the continuation payload
    /// before parking so resume never recomputes source-visible coordinates.
    AfterSingleSmallDrawPosition {
        slot: u8,
        continuation: Option<SingleSmallDrawContinuation>,
    },
    /// The Wallmaster slot has published its carry/send prefix and the fixed
    /// `Sprite_ResetAll_noDisable` stores. The large reset clears, send
    /// suffix, current slot return, and lower slots remain pending.
    AfterWallmasterResetPrefix(u8),
    /// `Sprite_Zazak_Main` published the current animation frame; its body
    /// suffix and lower Sprite_Main slots remain pending.
    ZazakAfterGraphics(u8),
    /// The four leading countdown statements in `Sprite_TimersAndOam`
    /// completed. The hit-timer and aux4 countdown work remains pending.
    AfterPrimaryTimerDecrements {
        slot: u8,
        state: Option<u8>,
    },
    /// The main and aux1 countdown statements in `Sprite_TimersAndOam`
    /// completed. Aux2/aux3 and all later work remain pending.
    AfterMainAndAux1TimerDecrements {
        slot: u8,
        state: Option<u8>,
    },
    /// A guard probe reached the instruction immediately after
    /// `Sprite_PrepOamCoordOrDoubleRet`. The source receipt omits the native
    /// scratch result; the translated prefix fills it before parking.
    ProbeAfterOamCoordinates {
        slot: u8,
        oam_position: Option<(u16, u16)>,
    },
    /// A state-8 slot is suspended inside the source-ordered 40-store
    /// `SpritePrep_ResetProperties` prefix. The source receipt supplies the
    /// count; native execution commits exactly that prefix before parking.
    InitializeResetProperties {
        slot: u8,
        phase: crate::SpriteInitializeResetPropertiesPhase,
        completed_stores: u8,
    },
    InitializeLoadProperties {
        slot: u8,
        phase: crate::SpriteInitializeResetPropertiesPhase,
        completed_stores: u8,
    },
    /// Fire Debirando's fixed initializer prefix completed through the entry
    /// to `Sprite_SpawnDynamically`; the callee has not mutated a free slot.
    FireDebirandoBeforeSpawn(u8),
    /// Fire Debirando's dynamic-spawn helper selected a child slot and
    /// published the named source mutation.
    FireDebirandoSpawn {
        slot: u8,
        spawned_slot: u8,
        progress: crate::SpriteDynamicSpawnProgress,
    },
    /// The dying Trinexx body's explosion spawn selected a child slot and
    /// published the named source mutation.
    TrinexxDeathExplosionSpawn {
        slot: u8,
        spawned_slot: u8,
        progress: crate::SpriteDynamicSpawnProgress,
    },
    /// Agahnim's afterimage spawn (`Sprite_Agahnim_ApplyMotionBlur`) reached
    /// the named `Sprite_SpawnDynamically` publication; `bound` records that
    /// the handler ran its prefix and published it on the interrupted host.
    AgahnimMotionBlurSpawn {
        slot: u8,
        spawned_slot: u8,
        progress: crate::SpriteDynamicSpawnProgress,
        bound: bool,
    },
    /// `Sprite_Trinexx_FinalPhase` state 0 stopped inside
    /// `Sprite_CheckTileCollision`, after the direction probes when
    /// `probes_completed`.
    TrinexxFinalPhaseTileCollision {
        slot: u8,
        probes_completed: bool,
    },
    /// `HelmasaurHardHatBeetleCommon` stopped inside `Sprite_CheckTileCollision`.
    HelmasaurHardHatTileCollision {
        slot: u8,
        stage: crate::SpriteTileCollisionStage,
    },
    /// `Lanmola_Draw` stopped inside its graphics/history prologue.
    LanmolaDrawPrefix {
        slot: u8,
        completed_stores: u8,
    },
    /// `Sprite_TrinexxD_Draw` stopped in body segment `segment` after
    /// `stage` of its per-segment steps.
    TrinexxFinalPhaseDraw {
        slot: u8,
        segment: u8,
        stage: u8,
        continuation: Option<TrinexxFlashingSegmentContinuation>,
    },
    /// `SpriteDraw_Antfairy` published its leading subtype2 increment.
    /// `continuation` is bound by the native sprite call site before parking.
    AfterAntfairySubtype2Increment {
        slot: u8,
        continuation: Option<AntfairyDrawContinuation>,
    },
    /// `Lanmola_Draw` published its graphics/history prefix and subtype2
    /// increment. `continuation` retains the source locals needed by the two
    /// remaining trail loops and is bound before the native caller parks.
    AfterLanmolaSubtype2Increment {
        slot: u8,
        continuation: Option<LanmolaDrawContinuation>,
    },
    /// `HelmasaurHardHatBeetleCommon` published its shared subtype2 increment.
    /// Both callers have completed their draw prefix; only the shared body
    /// suffix and the lower Sprite_Main slots remain pending.
    AfterHelmasaurHardHatBeetleSubtype2Increment {
        slot: u8,
    },
    /// The first nested active call in a state-8 standard-guard initializer
    /// stopped at the final weapon entry's flags store. The semantic wire
    /// carries only the slot; translated execution binds the source locals
    /// and the already-published OAM prefix before parking.
    GuardPrepWeaponFlagsPending {
        slot: u8,
        continuation: Option<GuardPrepWeaponDrawContinuation>,
    },
    /// A state-8 Mini Moldorm has completed its generic initialization and a
    /// source-ordered prefix of the Y-low/Y-high/X-low/X-high history stores.
    MiniMoldormHistory {
        slot: u8,
        completed_stores: u8,
    },
    GuardPrepParryHitbox {
        slot: u8,
        active_call: u8,
        continuation: Option<GuardPrepParryContinuation>,
    },
    GuardAnimation {
        slot: u8,
        checkpoint: crate::GuardAnimationCheckpoint,
        continuation: Option<GuardAnimationContinuation>,
    },
    HogSpearBodyGraphicsPending {
        slot: u8,
    },
    GuardPrepPatrolDelay {
        slot: u8,
        active_call: u8,
        saved_submodule: Option<u8>,
    },
    InitializePrepPending {
        slot: u8,
    },
    GuardPrepTileCollisionReturned {
        slot: u8,
        active_call: u8,
        saved_submodule: Option<u8>,
    },
    AbsorbableHorizontalTileLookup {
        slot: u8,
    },
    WallmasterResetClear {
        slot: u8,
        cleared_bytes: u16,
    },
    AfterHitTimer {
        slot: u8,
        state: Option<u8>,
    },
    PengatorSlidePending {
        slot: u8,
    },
    AntifairyBouncePending {
        slot: u8,
    },
    KholdstareDamagePending {
        slot: u8,
    },
    AfterMainTimerDecrement {
        slot: u8,
        state: Option<u8>,
    },
    AfterZeroHitTimerClear {
        slot: u8,
        state: Option<u8>,
    },
    AbsorbableVerticalTileLookup {
        slot: u8,
    },
    AbsorbableVerticalTileAttributeLoaded {
        slot: u8,
    },
    SwamolaHeadDraw {
        slot: u8,
    },
    SwamolaHeadDrawCompleted {
        slot: u8,
    },
    SwamolaSegmentDraw {
        slot: u8,
        segment: u8,
    },
    VitreousDamagePending {
        slot: u8,
    },
    VitreousAiPending {
        slot: u8,
    },
    VitreousPlayerDamagePending {
        slot: u8,
    },
    MoblinCollisionGeometry {
        slot: u8,
    },
    MoblinAttributeLoaded {
        slot: u8,
    },
    Module09FinalScrollPairPending,
    HappinessPondRupeeGraphicsStarted(u8),
    MiniMoldormAiPending {
        slot: u8,
    },
    BuzzblobAfterXSubpixel {
        slot: u8,
        pending: Option<(u8, u8)>,
    },
    CatfishMedallionGraphicsStarted(u8),
    TrinexxHeadDrawSetup(u8),
    WaterfallGtCutsceneGraphicsStarted(u8),
    TrinexxBreathTileCollisionReturned(u8),
    LaserEyeDrawPrologue(u8),
    /// A Zora fireball entered `Sprite_MoveXY`; its coordinate bytes are
    /// bound natively before parking.
    ZoraFireballMovement {
        slot: u8,
        checkpoint: crate::SpriteMoveXYCheckpoint,
        continuation: Option<SpriteMoveXYContinuation>,
    },
    TrinexxHeadDraw {
        slot: u8,
        segment: u8,
        continuation: Option<TrinexxHeadDrawContinuation>,
    },
    SidenexxNeckTargetLoop {
        slot: u8,
        step: u8,
        continuation: Option<SidenexxNeckLoopContinuation>,
    },
    TrinexxHeadFrontPart {
        slot: u8,
        completed_stores: u8,
        continuation: Option<TrinexxHeadDrawContinuation>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TrinexxHeadDrawContinuation {
    pub(crate) info_x: u16,
    pub(crate) info_y: u16,
    pub(crate) info_flags: u8,
    pub(crate) oam: usize,
    pub(crate) next_segment: u8,
    pub(crate) front_part: Option<(u8, u8, u8)>,
}

/// Sprite_Sidenexx's state-2 neck-target loop locals: the change count in
/// `$01` and whether the current pass stored a step whose count is pending.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SidenexxNeckLoopContinuation {
    pub(crate) n: u8,
    pub(crate) pending_increment: bool,
}

/// `Sprite_Trinexx_CheckDamageToFlashingSegment`'s saved body position (the
/// four PHA'd coordinate bytes) while the flashing segment's position is
/// swapped in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TrinexxFlashingSegmentContinuation {
    pub(crate) old_x: u16,
    pub(crate) old_y: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GuardAnimationContinuation {
    pub(crate) saved_initializer_submodule: Option<u8>,
    pub(crate) graphics: u8,
    pub(crate) direction: u8,
    pub(crate) checkpoint: crate::GuardAnimationCheckpoint,
    pub(crate) poc_x: u16,
    pub(crate) poc_y: u16,
    pub(crate) poc_flags: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GuardPrepParryContinuation {
    pub(crate) saved_submodule: u8,
    pub(crate) hitbox: crate::types::SpriteHitBox,
    pub(crate) disabled_oam_offsets: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SpriteMoveXYContinuation {
    pub(crate) x_low: u8,
    pub(crate) x_high: u8,
    pub(crate) y_low: u8,
    pub(crate) y_high: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct GuardPrepWeaponDrawContinuation {
    pub(crate) saved_submodule: u8,
    pub(crate) guard_bak_graphics: u8,
    pub(crate) guard_bak_direction: u8,
    pub(crate) draw_flags3: u8,
    pub(crate) draw_direction: u8,
    pub(crate) poc_x: u16,
    pub(crate) poc_y: u16,
    pub(crate) poc_flags: u8,
    pub(crate) pending_oam_index: u16,
    pub(crate) pending_oam_x: u16,
    pub(crate) pending_oam_flags: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct WishPondTossedItemGraphicsContinuation {
    pub(crate) ancilla_slot: u8,
    pub(crate) item: u8,
    pub(crate) wish_item: u8,
    pub(crate) graphics: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum CuccoSubtypeContinuation {
    State10,
    ActiveC,
    Hopping,
    Flee,
    CarriedLanding,
    CarriedAirborne,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum AntfairyDrawContinuation {
    BunnyBeam,
    Antifairy,
    AntifairyCircle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct LanmolaDrawContinuation {
    pub(crate) r2: u8,
    pub(crate) r5: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SingleSmallDrawContinuation {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) oam: u16,
    pub(crate) flags: u8,
    pub(crate) visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Module09SpriteMainReturn {
    pub(crate) bg2_x: u16,
    pub(crate) bg2_y: u16,
    pub(crate) bg1_x: u16,
    pub(crate) bg1_y: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Module09ItemReceiptCallerReturn {
    pub(crate) link_oam: Option<player_oam::LinkOamEquipmentContinuation>,
    pub(crate) scroll: Module09SpriteMainReturn,
    /// Some Module09 loaders publish the rain overlay before their suspended
    /// Sprite_Main caller returns. Ordinary player control publishes it after.
    pub(crate) rain_already_published: bool,
    /// Source continuation which remains below Module09's Sprite_Main frame.
    /// The continuous timing owner reports only the semantic Sprite_Main
    /// boundary; translated gameplay retains the caller suffix without
    /// depending on the source PC or raster position.
    pub(crate) after_sprite_main: Module09AfterSpriteMain,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SpriteMainCpuCaller {
    #[default]
    DungeonModule07,
    /// Module07's ordinary Sprite_Main call was interrupted at a boundary
    /// observed by the continuous source owner. The typed boundary determines
    /// whether its dungeon suffix resumes before or after the NMI accepted
    /// by this host; gameplay never consumes the source PC or raster position.
    DungeonModule07Live {
        boundary: OriginalTimingBoundary,
    },
    /// An ordinary Module09 caller whose stack-local scroll restoration and
    /// Link/OAM/HUD/rain suffix remain pending after the interrupted slot loop.
    Module09 {
        boundary: OriginalTimingBoundary,
    },
    WorldMapOverlayReload {
        module09: Module09SpriteMainReturn,
    },
    /// The Module13 boss-victory caller whose Link/OAM suffix remains
    /// pending after the interrupted slot loop (route host 103998).
    BossVictory {
        boundary: OriginalTimingBoundary,
    },
    /// The Module17 save-and-quit caller whose Link/OAM suffix remains
    /// pending after the interrupted slot loop (route host 159333, the
    /// post-save reset into the intro).
    SaveAndQuit {
        boundary: OriginalTimingBoundary,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NmiPrepareSpritesCpuCaller {
    DungeonModule07,
    OverworldModule09,
    DesertPrayer,
    WorldMapOverlayReload,
    /// `Module19_TriforceRoom`'s LinkOam_Main interrupted by vblank after
    /// its case-2 loads (route host 1557676).
    TriforceRoom,
    /// A resumed Module09 long-load caller (aux graphics or a whirlpool
    /// step) whose NMI_PrepareSprites was interrupted at the host boundary.
    Module09LongLoad,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum CachedSpriteCpuInterruption {
    Loading {
        slot: u8,
        copied_fields: u8,
    },
    ExecutingAntfairyAfterSubtype2Increment {
        slot: u8,
        continuation: Option<AntfairyDrawContinuation>,
    },
    Restoring {
        slot: u8,
        live_fields: u8,
    },
}

impl CachedSpriteCpuInterruption {
    pub(crate) const fn slot(self) -> u8 {
        match self {
            Self::Loading { slot, .. }
            | Self::ExecutingAntfairyAfterSubtype2Increment { slot, .. }
            | Self::Restoring { slot, .. } => slot,
        }
    }
}

impl From<crate::CachedSpriteExecutionProgress> for CachedSpriteCpuInterruption {
    fn from(progress: crate::CachedSpriteExecutionProgress) -> Self {
        match progress {
            crate::CachedSpriteExecutionProgress::Loading {
                slot,
                copied_fields,
            } => Self::Loading {
                slot,
                copied_fields,
            },
            crate::CachedSpriteExecutionProgress::Executing { slot, progress } => match progress {
                crate::CachedSpriteExecutionBodyProgress::AfterAntfairySubtype2Increment => {
                    Self::ExecutingAntfairyAfterSubtype2Increment {
                        slot,
                        continuation: None,
                    }
                }
            },
            crate::CachedSpriteExecutionProgress::Restoring { slot, live_fields } => {
                Self::Restoring { slot, live_fields }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CachedSpriteCpuProgress {
    pub(crate) slot: u8,
    pub(crate) copied_fields: u8,
    pub(crate) restored_fields: u8,
    pub(crate) restore_started: bool,
}

impl CachedSpriteCpuProgress {
    pub(crate) fn at_entry(slot: u8) -> Self {
        Self {
            slot,
            copied_fields: 0,
            restored_fields: 0,
            restore_started: false,
        }
    }

    pub(crate) fn observe_pc(&mut self, pc: u32) {
        // $1D:EB06 is the first instruction after Sprite_ExecuteSingle and
        // the pause check; the following PLA/STA pairs restore the displaced
        // live slot in reverse field order through $1D:EB67.
        self.restore_started |= (0x1d_eb06..=0x1d_eb67).contains(&pc);
    }

    pub(crate) fn observe_wram_write(&mut self, address: usize) {
        if self.restore_started {
            let restored = usize::from(self.restored_fields);
            if restored < CACHED_SPRITE_LIVE_FIELDS.len() {
                let field = CACHED_SPRITE_LIVE_FIELDS.len() - 1 - restored;
                if address == CACHED_SPRITE_LIVE_FIELDS[field] + usize::from(self.slot) {
                    self.restored_fields = self.restored_fields.saturating_add(1);
                }
            }
        } else {
            let copied = usize::from(self.copied_fields);
            if copied < CACHED_SPRITE_LIVE_FIELDS.len()
                && address == CACHED_SPRITE_LIVE_FIELDS[copied] + usize::from(self.slot)
            {
                self.copied_fields = self.copied_fields.saturating_add(1);
            }
        }
    }

    pub(crate) fn interruption(self) -> Option<CachedSpriteCpuInterruption> {
        if !self.restore_started
            && usize::from(self.copied_fields) <= CACHED_SPRITE_LIVE_FIELDS.len()
        {
            // A fully copied live slot can remain displaced while its cached
            // sprite handler is executing. Preserve that as Loading(24): the
            // translated continuation has no fields left to copy, but still
            // must finish the handler and restore the displaced live sprite.
            Some(CachedSpriteCpuInterruption::Loading {
                slot: self.slot,
                copied_fields: self.copied_fields,
            })
        } else if self.restore_started
            && usize::from(self.restored_fields) < CACHED_SPRITE_LIVE_FIELDS.len()
        {
            Some(CachedSpriteCpuInterruption::Restoring {
                slot: self.slot,
                live_fields: (CACHED_SPRITE_LIVE_FIELDS.len() - usize::from(self.restored_fields))
                    as u8,
            })
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DungeonModuleCpuAdvance {
    pub(crate) phase: ModuleCpuPhase,
    pub(crate) resumed_phase: Option<ModuleCpuPhase>,
    pub(crate) submodule_nmi_slices: u8,
    pub(crate) subsubmodule: u8,
    pub(crate) palette_countdown: u8,
    pub(crate) sprite_main_boundary: Option<SpriteMainCpuBoundary>,
    pub(crate) cached_sprite_interruption: Option<CachedSpriteCpuInterruption>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DungeonModuleCpuTiming {
    pub(crate) advance: DungeonModuleCpuAdvance,
    /// Rows whose reserved-table copy precedes their HDMA read in the
    /// interrupted field. The copy may straddle the visible field's start.
    pub(crate) spotlight_copy_visible_rows: Option<[bool; SPOTLIGHT_VISIBLE_SCANLINES]>,
    /// Leading rows whose HDMA read beat `IrisSpotlight_ResetTable`, but only
    /// when the reset field remains owned by this caller-return publication.
    pub(crate) spotlight_reset_prefix_scanlines: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DungeonPaletteCpuAdvance {
    pub(crate) work: CpuWorkAdvance,
    pub(crate) pc: u32,
    pub(crate) subsubmodule: u8,
    pub(crate) palette_countdown: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OverworldSpriteReloadResumeBoundary {
    ByReturnPhase(NmiPhase),
    CpuSliceEntryNmiRegisters,
}

impl OverworldSpriteReloadResumeBoundary {
    pub(crate) fn capture_scanout(
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
pub(crate) struct OverworldSpriteReloadTiming {
    pub(crate) load_nmi_slices: u8,
    pub(crate) post_return_hold_nmi_slices: u8,
    pub(crate) return_phase: NmiPhase,
    pub(crate) epilogue_phase: NmiPhase,
    pub(crate) resume_boundary: OverworldSpriteReloadResumeBoundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DungeonSupertileTransitionWork {
    RoomLoad,
    AuxiliarySpriteGraphics,
    /// Legacy scheduler ownership retained until this semantic domain has an
    /// authoritative Dungeon_ResetSprites progress receipt.
    RoomLoadSpriteResetFallback,
    RoomLoadSpriteReset {
        progress: DungeonResetSpritesCpuProgress,
    },
    SpriteConversion,
    RoomLoadCallerResume,
    SpriteConversionCallerResume,
    FilteredQuadrantTilemapBuild,
    QuadrantUploadCallerReturn,
    State13CallerReturn,
    FadedFilterPreCompletionCallerReturn,
    FadedFilterCallerReturn,
    SpiralRoomInitialization,
    SpiralBackgroundSync,
    StraightInterroomRoomInitialization,
    StraightInterroomBgCharacters34,
    StraightInterroomSpriteGraphics,
    FallingBgCharacters34,
    FallingSpriteGraphics,
    SpiralRoomCallerResume,
    SpiralBgCharacters34,
    SpiralSpriteGraphics,
}

/// Source caller whose stack is suspended inside
/// `Dungeon_FlipCrystalPegAttribute`.
///
/// The flip helper is shared, but its return statements are not: UpdatePegs
/// resets Module 7 to its base state, while the selectable-attribute caller
/// resumes the remainder of supertile state 8 before the common Module 7
/// suffix. Keeping that identity with the cursor prevents a terminal receipt
/// from completing the wrong translated caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DungeonPegAttributeFlipCaller {
    UpdatePegs,
    SupertileTransition {
        state_12_cpu_advance: Option<DungeonModuleCpuAdvance>,
        quadrant_cpu_advance: Option<DungeonModuleCpuAdvance>,
    },
    SpiralStairs,
    WarpPad,
    FallingTransition,
    FatInterRoomStairs,
    StraightInterroomStairs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DungeonPegAttributeFlipContinuation {
    pub(crate) caller: DungeonPegAttributeFlipCaller,
    pub(crate) progress: crate::DungeonPegAttributeFlipProgressReceipt,
}

/// CPU caller which resumes after an interrupted palette-filter walk.
///
/// The spiral-stair module's caller tail requeues core DMA after returning
/// from NMI. The straight-interroom modules instead expose the gate cleared by
/// that NMI. Carrying the caller in the continuation preserves the real return
/// path after the shared palette helper has suspended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InterruptedPaletteFilterCaller {
    SpiralStairs,
    StraightInterroomStairs,
    FallingRoom,
}

impl InterruptedPaletteFilterCaller {
    pub(crate) fn from_dungeon_submodule(submodule: u8) -> Self {
        match submodule {
            // Module07_06_FatInterRoomStairs shares Dungeon_SpiralStaircase11/12
            // with the spiral module (route host 402695).
            0x06 | 0x0e => Self::SpiralStairs,
            0x11..=0x13 => Self::StraightInterroomStairs,
            _ => panic!("palette-filter continuation from invalid dungeon submodule {submodule}"),
        }
    }

    pub(crate) const fn requeues_core_dma_after_nmi(self) -> bool {
        matches!(self, Self::SpiralStairs)
    }
}

impl DungeonSupertileTransitionWork {
    pub(crate) const fn nmi_slices(self) -> u8 {
        match self {
            Self::RoomLoad => DUNGEON_SUPERTILE_ROOM_LOAD_NMI_SLICES,
            Self::AuxiliarySpriteGraphics => DUNGEON_SUPERTILE_AUX_SPRITE_GFX_NMI_SLICES,
            // ROM timing mode replaces this placeholder before scheduling.
            Self::RoomLoadSpriteResetFallback | Self::RoomLoadSpriteReset { .. } => 1,
            Self::SpriteConversion => DUNGEON_SUPERTILE_SPRITE_CONVERSION_NMI_SLICES,
            Self::RoomLoadCallerResume | Self::SpriteConversionCallerResume => {
                DUNGEON_SUPERTILE_CALLER_RESUME_NMI_SLICES
            }
            Self::FilteredQuadrantTilemapBuild
            | Self::QuadrantUploadCallerReturn
            | Self::State13CallerReturn
            | Self::FadedFilterPreCompletionCallerReturn
            | Self::FadedFilterCallerReturn => DUNGEON_SUPERTILE_QUADRANT_TILEMAP_NMI_SLICES,
            // ROM timing mode replaces this placeholder before scheduling.
            Self::SpiralRoomInitialization => 1,
            // ROM timing mode replaces this placeholder before scheduling.
            Self::SpiralBackgroundSync => 1,
            Self::StraightInterroomRoomInitialization => {
                DUNGEON_STRAIGHT_INTERROOM_ROOM_INITIALIZATION_NMI_SLICES
            }
            Self::StraightInterroomBgCharacters34 => {
                DUNGEON_STRAIGHT_INTERROOM_BG_CHARACTERS_34_NMI_SLICES
            }
            Self::StraightInterroomSpriteGraphics => {
                DUNGEON_STRAIGHT_INTERROOM_SPRITE_GRAPHICS_NMI_SLICES
            }
            Self::FallingBgCharacters34 => DUNGEON_FALLING_BG_CHARACTERS_34_NMI_SLICES,
            Self::FallingSpriteGraphics => DUNGEON_FALLING_SPRITE_GRAPHICS_NMI_SLICES,
            Self::SpiralRoomCallerResume => DUNGEON_SUPERTILE_CALLER_RESUME_NMI_SLICES,
            Self::SpiralBgCharacters34 => DUNGEON_SPIRAL_BG_CHARACTERS_34_NMI_SLICES,
            Self::SpiralSpriteGraphics => DUNGEON_SPIRAL_SPRITE_GRAPHICS_NMI_SLICES,
        }
    }

    pub(crate) const fn next_module_resumes_after_pre_main_nmi(self) -> bool {
        matches!(self, Self::SpriteConversionCallerResume)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DungeonFallingEntranceWork {
    RoomAndTilesets,
    SpriteGraphics,
}

impl DungeonFallingEntranceWork {
    pub(crate) const fn nmi_slices(self) -> u8 {
        match self {
            Self::RoomAndTilesets => DUNGEON_FALLING_ENTRANCE_ROOM_LOAD_NMI_SLICES,
            Self::SpriteGraphics => DUNGEON_FALLING_ENTRANCE_SPRITE_GFX_NMI_SLICES,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ItemReceiptCaller {
    AtomicCaller,
    /// The ground handler's A-press opened a chest: the caller's remaining
    /// iteration tail is deferred to the receipt's completion slice.
    GroundApress,
    UnclePassage {
        sprite_slot: u8,
    },
    /// A live `Sprite_Main` handler whose synchronous Link_ReceiveItem call
    /// can hold the remaining lower slots and its module caller suffix.
    SpriteMain {
        sprite_slot: u8,
        suffix: SpriteMainItemReceiptSuffix,
    },
    /// The falling milestone item (`Ancilla_MilestoneItemReceipt`, ancilla
    /// type $29) called `Link_ReceiveItem` from Sprite_Main's prefix. The
    /// suspension holds the remaining ancilla slots, the rest of the prefix,
    /// the whole slot loop, and the module caller suffix (route host 1142850).
    AncillaMilestone {
        ancilla_slot: u8,
        suffix: AncillaItemReceiptSuffix,
    },
    /// A live sprite slot called `Link_ReceiveItem` directly from its own
    /// state machine (the source's `SpriteMainDirect` receipt caller). The
    /// suspension holds the same lower slots and module suffix as
    /// [`Self::SpriteMain`].
    SpriteMainDirect {
        sprite_slot: u8,
        suffix: SpriteMainItemReceiptSuffix,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpriteMainItemReceiptSuffix {
    BottleVendor,
    SickKid,
    /// ROM `$86d13c` absorption case 13: `Link_ReceiveItem(0x32)` then the
    /// shared key/big-key savegame-bit tail.
    BigKeyAbsorption,
    /// ROM `$85ef47` Sprite_HeartContainer's crystal branch:
    /// `Link_ReceiveItem(0x3e)` then the savegame-state bit tail (route host
    /// 102905).
    HeartContainerFull,
    /// ROM `$85ef47` Sprite_HeartContainer's piece-upgrade branch:
    /// `Link_ReceiveItem(0x26)` then the obtained-flag tail.
    HeartContainerUpgrade,
    /// ROM `$85f14d` Sprite_Sahasrahla's boots handout:
    /// `Link_ReceiveItem(0x4b)` then the AI-state and map-icon tail (route
    /// host 106127).
    SahasrahlaBoots,
    /// ROM `$85fc04` Sprite_BookOfMudora's pickup: `Link_ReceiveItem(0x1d)`
    /// then the sprite-kill tail (route host 111518).
    BookOfMudora,
    /// ROM `$9ddf54` Sprite_Catfish_QuakeMedallion's handout:
    /// `Link_ReceiveItem(sprite_A, 0)` then the OAM-region/draw tail of the
    /// now-inactive sprite (route host 177047).
    CatfishMedallion,
    /// ROM `$86c44c` Sprite_HappinessPond case 9: `sprite_ai_state = 10`
    /// then `Link_ReceiveItem(gfx, 0)` with nothing after it (route hosts
    /// 180723, 182366).
    HappinessPondReward,
    /// ROM `$86bdd0` Sprite_Hobo_Bum case 2: `Link_ReceiveItem(0x16, 0)`
    /// then the progress-indicator bit tail (route host 187034).
    HoboBottle,
    /// ROM `$85ee78` Sprite_E7_Mushroom: `Link_ReceiveItem(0x29, 0)` with
    /// nothing after it (route host 201575).
    Mushroom,
    /// ROM `$9ee0..` Sprite_OldMan's mirror handout: `Link_ReceiveItem(0x1a,
    /// 0)` then the starting-point, cutscene, and shuffle-away tail (route
    /// host 207229).
    OldManMirror,
    /// ROM `$9ef366` ShopItem_HandleReceipt: `Link_ReceiveItem(item, 0)`, then
    /// the give-item message and rapid terminate for shopkeeper subtypes, and
    /// (for the shield items) the caller's `sprite_flags4 = 0x1c` store (route
    /// host 330113).
    ShopItem {
        flags4: bool,
    },
    /// ROM `$8589..` Sprite_MasterSword case 4: `Link_ReceiveItem(1, 0)` then
    /// the map-icon, pull-state, and AI-state tail (route host 264468).
    MasterSword,
    /// ROM `$85f644` Sprite_BagOfPowder: `Link_ReceiveItem(0xd, 0)` then the
    /// sprite-kill tail (route host 753969).
    PotionShopPowder,
    /// ROM `$85f68e`/`$85f72b`/`$85f7d0` potion cauldrons:
    /// `Link_ReceiveItem(item, 0)` with nothing after it (route host 968296).
    PotionCauldron,
    /// ROM `$86b34e` Smithy_Main case 6 (GiveTemperedSword): `Link_ReceiveItem(2, 0)`
    /// then the progress-indicator-3 bit clear (route host 809101).
    SmithyTemperedSword,
    /// ROM `$85f020` Sprite_HeartPiece's fourth piece: `Link_ReceiveItem(0x26, 0)`
    /// then the sprite-kill and obtained-flag tail (route host 974819).
    HeartPiece,
    /// ROM FluteKid stumpy case 2: `Link_ReceiveItem(0x13, 0)` (the shovel)
    /// then `ai_state = 0` (route host 512301).
    FluteKidShovel,
    /// ROM Locksmith case 3: `Link_ReceiveItem(0x16, 0)` (the chain) then the
    /// progress-indicator-3 bit, `ai_state = 4`, and the follower indicator
    /// clear (route host 1036281).
    LocksmithChain,
}

impl SpriteMainItemReceiptSuffix {
    /// Whether this typed callsite reaches `Link_ReceiveItem` directly from
    /// a sprite's own state machine (the source's `SpriteMainDirect` receipt
    /// caller) rather than through a solicited `SpriteMain` handler call.
    /// Each suffix names one fixed C callsite, so its directness is a
    /// property of the callsite itself.
    pub(crate) const fn caller_is_direct(self) -> bool {
        matches!(
            self,
            Self::BigKeyAbsorption
                | Self::HeartContainerFull
                | Self::HeartContainerUpgrade
                | Self::SahasrahlaBoots
                | Self::BookOfMudora
                | Self::CatfishMedallion
                | Self::HappinessPondReward
                | Self::HoboBottle
                | Self::Mushroom
                | Self::OldManMirror
                | Self::ShopItem { .. }
                | Self::MasterSword
                | Self::PotionShopPowder
                | Self::PotionCauldron
                | Self::SmithyTemperedSword
                | Self::HeartPiece
                | Self::FluteKidShovel
                | Self::LocksmithChain
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GameCallStatus {
    Returned,
    Suspended,
}

impl GameCallStatus {
    pub(crate) const fn is_suspended(self) -> bool {
        matches!(self, Self::Suspended)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ItemReceiptReturn {
    pub(crate) ancilla_slot: u8,
    pub(crate) item: u8,
    pub(crate) chest_position: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DungeonSpriteMainReturn {
    pub(crate) bg2_x: u16,
    pub(crate) bg2_y: u16,
    pub(crate) bg1_x: u16,
    pub(crate) bg1_y: u16,
    pub(crate) link_oam: Option<player_oam::LinkOamEquipmentContinuation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LinkMovePositionReturn {
    pub(crate) old_x: u16,
    pub(crate) old_y: u16,
}

/// `Link_HandleVelocity` has selected its speed. Unpublished actual-velocity
/// components and the movement call remain on the source stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LinkActualVelocityReturn {
    pub(crate) pending_speed_index: Option<u8>,
    pub(crate) completed_clear_stores: u8,
    pub(crate) pending_actual_x: Option<u8>,
    pub(crate) pending_actual_y: Option<u8>,
}

/// `Link_MovePosition` suspended inside its axis loop: the current axis (the
/// loop runs Z when airborne, then X, then Y) has its subpixel stored and still
/// owes `pending_pixel_delta` on its coordinate; later axes are untouched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LinkMovePositionPartial {
    pub(crate) axis: crate::game_state::PlayerAxis,
    pub(crate) pending_pixel_delta: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LinkMovePositionPartialReturn {
    pub(crate) old_x: u16,
    pub(crate) old_y: u16,
    pub(crate) partial: LinkMovePositionPartial,
}

/// `Link_MovePosition` suspended after the current axis' low coordinate byte
/// but before its high coordinate byte. The mixed coordinate is already
/// visible in RAM/native state; `pending_coordinate_high` completes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LinkMovePositionAfterCoordinateLowReturn {
    pub(crate) old_x: u16,
    pub(crate) old_y: u16,
    pub(crate) axis: crate::game_state::PlayerAxis,
    pub(crate) pending_coordinate_high: u8,
}

/// `Link_MovePosition` suspended after both coordinate stores for `axis`.
/// Earlier axes and `axis` are complete; later axes and the movement tail
/// remain pending.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LinkMovePositionAfterCoordinatesReturn {
    pub(crate) old_x: u16,
    pub(crate) old_y: u16,
    pub(crate) axis: crate::game_state::PlayerAxis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ItemReceiptGraphicsContinuation {
    /// The translated caller is still atomic. Its suffix has already run, so
    /// the measured graphics delay only owns the ordinary main-loop epilogue.
    CallerAlreadyCompleted {
        gfx: u8,
        /// `Some` when the receipt was initiated by the ground handler's
        /// A-press (chest open): the ROM blocks inside AncillaAdd_ItemReceipt
        /// for the intervening vblanks, so the ancilla tail, Link_ReceiveItem
        /// tail, and the rest of HandleLink_From1D (movement with the latched
        /// joypad included) all run after the decompression returns. The
        /// completion arm resumes them here.
        ground_apress_tail: Option<ItemReceiptReturn>,
    },
    /// `DecodeAnimatedSpriteTile_variable` interrupted this real ROM call
    /// stack. Resume each typed caller suffix after the graphics routine
    /// returns instead of publishing selected side effects early.
    ResumeUnclePassage {
        receipt: ItemReceiptReturn,
        sprite_slot: u8,
        dungeon: DungeonSpriteMainReturn,
    },
    /// Resume the exact Sprite_Main/module call chain after the timing
    /// authority reports that the synchronous item-receipt graphics call has
    /// returned. Native caller state is carried as a typed source-level return
    /// token; no CPU address or measured frame count is stored here.
    ResumeSpriteMainItemReceipt {
        receipt: ItemReceiptReturn,
        sprite_slot: u8,
        suffix: SpriteMainItemReceiptSuffix,
        caller: SpriteMainItemReceiptCallerReturn,
    },
    /// Resume the falling milestone item's receipt from Sprite_Main's prefix:
    /// the remaining ancilla slots, the prefix tail, the whole slot loop, and
    /// the module caller run after the graphics routine returns.
    ResumeAncillaItemReceipt {
        receipt: ItemReceiptReturn,
        ancilla_slot: u8,
        suffix: AncillaItemReceiptSuffix,
        caller: SpriteMainItemReceiptCallerReturn,
    },
}

/// The C statements an ancilla runs after its synchronous `Link_ReceiveItem`
/// call returns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AncillaItemReceiptSuffix {
    /// `Ancilla_MilestoneItemReceipt` and the dug-up flute return immediately.
    None,
    /// `Ancilla22_ItemReceipt` hands out the fourth heart piece as a heart
    /// container, then clears its own slot and the modal pause flag.
    ClearAncillaAndModalPause,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SpriteMainItemReceiptCallerReturn {
    /// Module 7 retains its scroll restore values and dungeon caller suffix
    /// across its Sprite_Main call.
    Module07(DungeonSpriteMainReturn),
    /// Module 9 retains four stack-local scroll values and rain-publication
    /// ownership across its Sprite_Main call.
    Module09(Module09ItemReceiptCallerReturn),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PreDungeonSpriteResetContinuation {
    #[default]
    Pending,
    /// This selected-game destination does not enter `Module_PreDungeon`.
    NotApplicable,
    SpriteDisableAllCompleted,
    /// `Sprite_ResetAll` returned and the following `Dungeon_ResetSprites`
    /// call published this exact source checkpoint.
    DungeonResetSprites(crate::DungeonResetSpritesCpuProgress),
    SpriteResetAllCompleted,
    DungeonResetSpritesCompleted,
    SpriteDisableAllThrough(u8),
    GarnishDisableThrough(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FluteMenuSelectedScreenStep {
    InitialSpriteReset,
    OverworldReloadReset,
    OverworldReloadScan,
    SelectedScreenSuffix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GameWorkContinuation {
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
    /// `Module07_07`'s subsub-2 `Dungeon_InitializeRoomFromSpecial` room
    /// load runs suspended across many held vblanks (route hosts
    /// 91639-91656); the wire's terminal return owns its completion and the
    /// `subsubmodule += 1` advance.
    FinishDungeonFallingRoomInitialization,
    /// `Module07_18_RescuedMaiden` was interrupted inside its 8,192-store
    /// room-tilemap clear. The source-order prefix has committed and the saved
    /// caller resumes from `completed_stores` after the interrupting NMI.
    FinishRescuedMaidenTilemapClear {
        completed_stores: u16,
    },
    /// `Module07_18_RescuedMaiden` remains suspended inside the two
    /// `LoadFollowerGraphics` decompressions. The carried source-stage cursor
    /// owns the exact committed prefix of the partially-written scratch
    /// buffers until the enclosing cutscene initializer returns.
    FinishRescuedMaidenInitialization {
        stage: crate::RescuedMaidenInitializationStage,
    },
    FinishDungeonSupertileTransition {
        work: DungeonSupertileTransitionWork,
    },
    /// NMI interrupted the selected Module 7 submodule before its indirect
    /// dispatcher call returned. The translated submodule prefix has already
    /// committed; resume the common Module 7 suffix after that hardware NMI.
    FinishDungeonAfterSubmoduleCallerReturn,
    /// NMI interrupted the common Module 7 suffix inside `LinkOam_Main`.
    /// The translated prefix through `Sprite_Main` has completed; resume Link
    /// OAM, HUD, and the main-loop suffix after that exact hardware boundary.
    FinishDungeonPostSpriteMainCallerReturn,
    /// NMI interrupted Module09's caller suffix inside `LinkOam_Main` after
    /// `Sprite_Main` returned. Preserve the four live scroll locals and resume
    /// Link OAM, HUD, rain, and the main-loop suffix without replaying either
    /// the overlay body or `Sprite_Main`.
    FinishModule09LinkOamCallerReturn {
        caller: Module09ItemReceiptCallerReturn,
    },
    /// NMI interrupted the main-loop suffix while it packed the extended OAM
    /// staging bytes. Resume sprite preparation before the next publishable
    /// NMI without replaying Module 7.
    FinishNmiPrepareSpritesCallerReturn {
        caller: NmiPrepareSpritesCpuCaller,
    },
    FinishPreDungeonEntranceLoad {
        sprite_reset: PreDungeonSpriteResetContinuation,
    },
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
    /// `Module19_TriforceRoom` cases 2-4 each block one ZeldaRunGameLoop
    /// iteration for dozens of hosts (credits song-bank upload with NMI
    /// masked, then the overlay/tileset/screen loads under held vblanks);
    /// the case remainder and the module tail run where the wire returns
    /// the iteration (route hosts 1557656-1557723).
    FinishTriforceRoomLoad {
        step: TriforceRoomLoadStep,
    },
    /// `Text_Initialize` reached `Text_LoadCharacterBuffer` after the measured
    /// decompression crossings. The remaining count is measured independently
    /// because compressed message length decides whether that suffix crosses
    /// another NMI.
    FinishDialogueInitializationPrefix {
        caller_nmi_crossings: u8,
    },
    /// `Text_Initialize` crossed its final NMI inside the character-buffer
    /// build. Resume its stores and the Module0E caller suffix on the following
    /// host without inventing another vblank inside the initializer.
    FinishDialogueInitializationCallerReturn,
    /// `Module0E_05_DesertPrayer` was interrupted within the iris-table
    /// builder. The typed progress owns its exact persistent prefix; the
    /// remaining rows, caller-specific tail, Module0E scroll-copy suffix, and
    /// main-loop return remain pending.
    FinishDesertPrayerIris {
        progress: crate::DesertPrayerIrisProgress,
        caller: DesertPrayerIrisCaller,
    },
    FinishDesertPrayerPaletteFilter {
        countdown: u8,
        next_color: u8,
    },
    FinishDungeonSubtilePaletteFilter,
    FinishStraightInterroomFadeoutSuffix,
    FinishStraightInterroomSpriteReset {
        progress: DungeonResetSpritesCpuProgress,
    },
    FinishSpriteMain {
        boundary: SpriteMainCpuBoundary,
        caller: SpriteMainCpuCaller,
    },
    FinishCpuInstructionNmi {
        resume: PreMainNmiResume,
    },
    FinishDungeonCachedSpriteMain {
        boundary: CachedSpriteCpuInterruption,
        live_slot_backup: [u8; 24],
        dungeon: DungeonSpriteMainReturn,
    },
    FinishSpiralStaircasePaletteFilter {
        tail: SpiralStaircasePaletteTail,
        caller: InterruptedPaletteFilterCaller,
    },
    FinishDungeonExitSpotlightEntry {
        table_build: SpotlightTableBuildContinuation,
        iteration: SpotlightIteration,
    },
    FinishDungeonExitSpotlightBuild {
        table_build: SpotlightTableBuildContinuation,
        projection_completed: bool,
        iteration: SpotlightIteration,
    },
    /// A recurring Module0F caller reached `LinkOam_Main` before the live
    /// timing boundary. The table/control and Link-animation prefix have
    /// completed; the following host owns the shared sprite-preparation
    /// suffix without replaying the spotlight call.
    FinishDungeonExitSpotlightLinkOam {
        iteration: SpotlightIteration,
    },
    /// Module0F reached vblank after Player_MovePosition1_ integrated Link's
    /// coordinates but before the moving-floor/conveyor/drag tail returned.
    FinishDungeonExitSpotlightLinkVelocity {
        position_return: LinkMovePositionReturn,
        iteration: SpotlightIteration,
    },
    /// Module0F ended inside `Link_HandleVelocity`'s actual-velocity loop.
    /// The unresolved components are retained; the speed/modifier prefix is
    /// already committed and must not run again.
    FinishDungeonExitSpotlightActualVelocity {
        velocity_return: LinkActualVelocityReturn,
        iteration: SpotlightIteration,
    },
    /// Module0F's submodule call returned before the accepted NMI, but none
    /// of the following Link movement/OAM suffix executed. Resume that whole
    /// source suffix without replaying the completed spotlight entry.
    FinishDungeonExitSpotlightLinkAndOam {
        iteration: SpotlightIteration,
    },
    /// Module0F reached vblank in `Link_MovePosition` before either coordinate
    /// integration store. The following host owns the complete movement and
    /// Link/OAM suffix, so no partially executed native leaf is replayed.
    FinishDungeonExitSpotlightLinkMovement {
        iteration: SpotlightIteration,
    },
    /// Module0F's Link movement was interrupted inside `Link_MovePosition`'s
    /// axis loop after the `pass` axis' subpixel store: that coordinate still
    /// owes `pending_pixel_delta`, the later axes their whole move, and the
    /// caller its sand-drag and Link/OAM suffix (route host 179586).
    FinishDungeonExitSpotlightLinkMovementAfterSubpixel {
        iteration: SpotlightIteration,
        pass: u8,
        pending_pixel_delta: u16,
        old_x: u16,
        old_y: u16,
    },
    /// Module0F's Link movement published the selected axis' coordinate low
    /// byte; the high byte, later axes, and caller suffix remain pending.
    FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow {
        iteration: SpotlightIteration,
        pass: u8,
        pending_coordinate_high: u8,
        old_x: u16,
        old_y: u16,
    },
    /// Module10's Sprite_Main prefix returned, but vblank interrupted the
    /// opening-iris table build or copy before its radius/control suffix.
    FinishOverworldSpotlightBuild {
        table_build: SpotlightTableBuildContinuation,
        phase: OverworldSpotlightBuildPhase,
        projection_completed: bool,
        iteration: SpotlightIteration,
    },
    /// Module10 reached vblank in LinkOam_Main. This is the resumable suffix of
    /// that concrete C caller; its ROM plan also supplies the next Module10
    /// entry raster rather than reusing a fixed per-iteration estimate.
    FinishOverworldSpotlightLinkOam {
        iteration: SpotlightIteration,
    },
    /// Module10's resumed opening build reached its goal inside
    /// `IrisSpotlight_ConfigureTable`, but vblank interrupted
    /// `IrisSpotlight_ResetTable` after `completed_stores` of its 224 source-
    /// order table stores. The remaining stores, the module/submodule and
    /// music transition, `OpenSpotlight_Next2`, LinkOam, and the shared
    /// suffix all belong to the following host (route host 182709).
    FinishOverworldSpotlightGoalResetTable {
        iteration: SpotlightIteration,
        completed_stores: u8,
    },
    /// `Module09_2E_Whirlpool` entered one of its long loads; the load's
    /// completion, the case tail, and Module09's Sprite_Main/HUD suffix belong
    /// to the host where the wire returns the suspended caller.
    FinishModule09LongLoad {
        step: Module09LongLoadStep,
    },
    /// The closing iris reached its goal inside `IrisSpotlight_ConfigureTable`,
    /// but vblank interrupts before `Spotlight_ConfigureTableAndControl` can
    /// restore Link's exit coordinate and call `OpenSpotlight_Next2`.
    FinishDungeonExitSpotlightGoalCaller {
        iteration: SpotlightIteration,
    },
    FinishGameOverSpotlightBuild {
        table_build: SpotlightTableBuildContinuation,
        entry: bool,
        iteration: SpotlightIteration,
    },
    /// The closing Game Over iris reached radius zero and the source was
    /// interrupted inside its six-row constant-palette fill, before the table
    /// reset and submodule-4 initializer.
    FinishGameOverIrisGoalPaletteFill {
        completed_stores: u8,
    },
    FinishSpotlightIteration {
        iteration: SpotlightIteration,
    },
    PreOverworldPropertiesSpriteReset {
        overworld_screen: u8,
        animated_tiles: u8,
    },
    FinishPreOverworldProperties {
        overworld_screen: u8,
        sprite_presence_published: bool,
    },
    /// `FluteMenu_LoadSelectedScreen` is suspended inside the selected bird-
    /// travel destination load. The source wire advances this synchronous C
    /// stack through both sprite-reset calls, the overworld reload scan, and
    /// finally the remaining palette/tile caller suffix.
    FinishFluteMenuSelectedScreen {
        step: FluteMenuSelectedScreenStep,
    },
    FinishPreOverworldOverlays,
    FinishPreOverworldScreenBuild,
    FinishWorldMapExitTilesets,
    FinishWorldMapOverlayReload,
    FinishWorldMapAmbientMap8,
    FinishOverworldAuxGraphics,
    FinishOverworldMosaicSpriteGraphics,
    FinishOverworldMapQuadrants {
        scroll_map_and_sprite_gfx_tail_nmi_slices: u8,
    },
    FinishOverworldScreenMapAndSpriteGraphicsTail,
    FinishOverworldSpriteReloadTail {
        post_return_hold_nmi_slices: u8,
        return_phase: NmiPhase,
        epilogue_phase: NmiPhase,
        resume_scanout: OverworldSpriteReloadResumeScanout,
    },
    HoldOverworldSpriteReloadReturn,
    /// `Overworld_LoadOverlays` is suspended inside its concrete
    /// `Sprite_ReloadAll_Overworld` call. The rebuilt generation is staged;
    /// source activation and return receipts publish it incrementally.
    FinishOverworldLoadOverlaysSpriteReload,
    /// The same `Overworld_LoadOverlays` call has returned from sprite reload
    /// and is suspended in `LoadOverworldOverlay`; its terminal return owns
    /// Module09's Sprite_Main/HUD and shared main-loop suffix.
    FinishOverworldLoadOverlaysOverlay,
    /// Module0F's Link movement reached the end of one `Link_MovePosition`
    /// axis pass. Both coordinate bytes for `pass` are live; later axes and
    /// the caller suffix resume after the boundary.
    FinishDungeonExitSpotlightLinkMovementAfterCoordinates {
        iteration: SpotlightIteration,
        pass: u8,
        old_x: u16,
        old_y: u16,
    },
    /// Module0B/$24 entered `DecodeAnimatedSpriteTile_variable` before
    /// restoring the saved special-overworld coordinates. The completed
    /// decode is staged, while the restore and submodule advance remain
    /// pending until the wire reports the enclosing source-stage return.
    FinishOverworldSpecialExitMosaic,
    /// Module0B/$24 restored the saved special-overworld state and entered
    /// its second `DecodeAnimatedSpriteTile_variable` call. Only that decode,
    /// the submodule advance, and the Module09 caller suffix remain pending.
    FinishOverworldSpecialExitMosaicSecondDecode,
    FinishDungeonPushBlocks {
        dungeon: DungeonSpriteMainReturn,
    },
    /// Module 7's Dungeon_PushBlock_Handler loop resumes from the saved misc
    /// object index; the scroll copies, push-block drawing and Sprite_Main
    /// follow in the same resumed tail.
    FinishDungeonPushBlockHandler {
        /// The interrupted handler loop still has misc objects to run before
        /// the tail (false: the handler returned before the interruption).
        handler_pending: bool,
    },
    /// Death_Func15 has disabled sprites and entered the reset workspace
    /// clear. Death counters and the save/continue caller remain suspended.
    FinishGameOverDeathAfterSpriteReset {
        count_as_death: bool,
    },
    FinishDungeonExitSpotlightControl {
        iteration: SpotlightIteration,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GameWorkCompletionPublication {
    pub(crate) bg_scroll: Option<DisplayBgScrollGeneration>,
    pub(crate) obj: Option<ObjScanoutGenerations>,
}

impl GameWorkContinuation {
    /// Scheduled pre-overworld and world-map loads hold ZeldaRunGameLoop
    /// suspended across several vblanks without saving a translated call
    /// stack of their own. Their Continued hosts still publish the generic
    /// acceptance/progress receipts, and the uninterrupted scheduled-caller
    /// timeline is the sole consumer of that authority.
    pub(crate) const fn consumes_uninterrupted_scheduled_caller_timeline(self) -> bool {
        matches!(
            self,
            Self::PreOverworldPropertiesSpriteReset { .. }
                | Self::FinishPreOverworldProperties { .. }
                | Self::FinishFluteMenuSelectedScreen { .. }
                | Self::FinishPreOverworldOverlays
                | Self::FinishPreOverworldScreenBuild
                | Self::FinishWorldMapLightLoad
                | Self::FinishWorldMapExitTilesets
                | Self::FinishOverworldAuxGraphics
                | Self::FinishOverworldMosaicSpriteGraphics
                | Self::FinishOverworldSpecialExitMosaic
                | Self::FinishOverworldSpecialExitMosaicSecondDecode
                | Self::FinishGameOverDeathAfterSpriteReset { .. }
                | Self::FinishModule09LongLoad { .. }
                | Self::FinishOverworldMapQuadrants { .. }
                | Self::FinishOverworldScreenMapAndSpriteGraphicsTail
                | Self::FinishOverworldSpriteReloadTail { .. }
                | Self::FinishOverworldLoadOverlaysSpriteReload
                | Self::FinishOverworldLoadOverlaysOverlay
                | Self::FinishDungeonFallingEntrance { .. }
                | Self::FinishDungeonFallingRoomInitialization
                | Self::FinishRescuedMaidenTilemapClear { .. }
                | Self::FinishRescuedMaidenInitialization { .. }
                | Self::FinishDesertPrayerIris { .. }
                | Self::FinishDesertPrayerPaletteFilter { .. }
                | Self::FinishPreDungeonEntranceLoad { .. }
                | Self::FinishPreDungeonSongBankTransfer
                | Self::FinishDungeonMapGraphicsPreparation
                | Self::FinishDungeonMapRoomDrawing
                | Self::FinishDungeonMapRecovery
                | Self::FinishTriforceRoomLoad { .. }
        )
    }

    /// Scheduled callers whose terminal main-wait return is consumed by the
    /// shared scheduled-caller return-timeline lane: the saved C call resumes
    /// after its carried or same-host Held handler, runs through
    /// ZeldaRunGameLoop's common suffix, and may carry one trailing Open
    /// acceptance. The dungeon dispatcher additionally proves one Sprite_Main
    /// return; a pre-overworld stage appends its boundary-qualified stage
    /// receipt; the overworld opening Build returns bare.
    pub(crate) const fn scheduled_caller_return_timeline_owns_terminal_return(self) -> bool {
        matches!(
            self,
            Self::FinishDungeonAfterSubmoduleCallerReturn
                | Self::FinishDungeonPushBlocks { .. }
                | Self::FinishDungeonPushBlockHandler { .. }
                | Self::FinishOverworldSpotlightBuild { .. }
                | Self::FinishOverworldSpotlightGoalResetTable { .. }
                | Self::FinishGameOverSpotlightBuild { .. }
                | Self::FinishGameOverIrisGoalPaletteFill { .. }
                | Self::FinishGameOverDeathAfterSpriteReset { .. }
                | Self::FinishWorldMapLightLoad
                | Self::FinishWorldMapExitTilesets
                | Self::FinishWorldMapAmbientMap8
                | Self::FinishOverworldAuxGraphics
                | Self::FinishOverworldMosaicSpriteGraphics
                | Self::FinishOverworldSpecialExitMosaic
                | Self::FinishOverworldSpecialExitMosaicSecondDecode
                | Self::FinishModule09LongLoad { .. }
                | Self::FinishOverworldScreenMapAndSpriteGraphicsTail
                | Self::FinishOverworldSpriteReloadTail { .. }
                | Self::FinishDungeonFallingEntrance { .. }
                | Self::FinishDungeonFallingRoomInitialization
                | Self::FinishRescuedMaidenTilemapClear { .. }
                | Self::FinishRescuedMaidenInitialization { .. }
                | Self::FinishItemReceiptGraphics {
                    continuation: ItemReceiptGraphicsContinuation::ResumeUnclePassage { .. }
                        | ItemReceiptGraphicsContinuation::ResumeSpriteMainItemReceipt { .. }
                        | ItemReceiptGraphicsContinuation::ResumeAncillaItemReceipt { .. },
                }
                | Self::FinishDungeonSubtilePaletteFilter
                | Self::FinishNmiPrepareSpritesCallerReturn { .. }
                | Self::FinishDungeonPostSpriteMainCallerReturn
                | Self::FinishDungeonExitSpotlightGoalCaller { .. }
                | Self::FinishDungeonCachedSpriteMain { .. }
                | Self::FinishSpriteMain {
                    caller: SpriteMainCpuCaller::DungeonModule07
                        | SpriteMainCpuCaller::DungeonModule07Live { .. }
                        | SpriteMainCpuCaller::Module09 { .. }
                        | SpriteMainCpuCaller::BossVictory { .. }
                        | SpriteMainCpuCaller::SaveAndQuit { .. },
                    ..
                }
                | Self::FinishBigKeyDropGraphics { .. }
                | Self::FinishStraightInterroomSpriteReset { .. }
                | Self::FinishDialogueInitializationPrefix { .. }
                | Self::FinishDesertPrayerIris { .. }
                | Self::FinishDesertPrayerPaletteFilter { .. }
                | Self::FinishDungeonExitSpotlightEntry { .. }
                | Self::FinishDungeonExitSpotlightLinkAndOam { .. }
                | Self::FinishDungeonExitSpotlightControl { .. }
                | Self::FinishDungeonExitSpotlightLinkMovement { .. }
                | Self::FinishDungeonExitSpotlightActualVelocity { .. }
                | Self::FinishDungeonExitSpotlightLinkMovementAfterSubpixel { .. }
                | Self::FinishDungeonExitSpotlightLinkMovementAfterCoordinateLow { .. }
                | Self::FinishDungeonExitSpotlightLinkMovementAfterCoordinates { .. }
                | Self::FinishModule09LinkOamCallerReturn { .. }
                | Self::FinishWorldMapOverlayReload
                | Self::FinishOverworldLoadOverlaysSpriteReload
                | Self::FinishOverworldLoadOverlaysOverlay
                | Self::FinishFluteMenuSelectedScreen { .. }
                | Self::FinishPreDungeonEntranceLoad { .. }
                | Self::FinishDungeonSupertileTransition { .. }
                | Self::FinishSpiralStaircasePaletteFilter { .. }
                | Self::FinishDungeonMapGraphicsPreparation
                | Self::FinishDungeonMapRoomDrawing
                | Self::FinishDungeonMapRecovery
                | Self::FinishTriforceRoomLoad { .. }
        ) || self.pre_overworld_stage_completion().is_some()
    }

    /// How many shared `Sprite_Main` loops this scheduled caller's resumed C
    /// body runs before returning to main wait. `None` leaves the count to
    /// the wire (a supertile transition runs zero or one depending on which
    /// stage its saved caller resumed inside).
    pub(crate) const fn scheduled_caller_return_expected_sprite_main_claims(self) -> Option<usize> {
        match self {
            Self::FinishDungeonAfterSubmoduleCallerReturn
            | Self::FinishDungeonPushBlocks { .. }
            | Self::FinishDungeonPushBlockHandler { .. }
            | Self::FinishWorldMapAmbientMap8
            | Self::FinishOverworldAuxGraphics
            | Self::FinishOverworldMosaicSpriteGraphics
            | Self::FinishOverworldSpecialExitMosaic
            | Self::FinishOverworldSpecialExitMosaicSecondDecode
            | Self::FinishOverworldScreenMapAndSpriteGraphicsTail
            | Self::FinishOverworldSpriteReloadTail { .. }
            | Self::FinishItemReceiptGraphics { .. }
            | Self::FinishDungeonSubtilePaletteFilter
            | Self::FinishSpiralStaircasePaletteFilter { .. }
            | Self::FinishBigKeyDropGraphics { .. }
            | Self::FinishStraightInterroomSpriteReset { .. }
            | Self::FinishWorldMapOverlayReload
            | Self::FinishOverworldLoadOverlaysSpriteReload
            | Self::FinishOverworldLoadOverlaysOverlay => Some(1),
            Self::FinishDesertPrayerIris { .. } => Some(0),
            Self::FinishDesertPrayerPaletteFilter { .. } => Some(0),
            // Module15's warp caller runs no Sprite_Main after the load.
            Self::FinishModule09LongLoad { step } => {
                if step.caller_is_module15() && !step.module15_runs_sprite_main_after() {
                    Some(0)
                } else {
                    Some(1)
                }
            }
            Self::FinishDungeonSupertileTransition { .. } => None,
            // The falling room-init caller resumes through Module07_07 and
            // the module tail's unconditional Sprite_Main before returning
            // to main wait (route host 91656).
            Self::FinishDungeonFallingRoomInitialization => Some(1),
            Self::FinishRescuedMaidenTilemapClear { .. } => Some(1),
            Self::FinishRescuedMaidenInitialization { .. } => Some(1),
            // A resumed Sprite_Main body crosses its native slot-zero return
            // during this completion, but the wire may have published the
            // matching SpriteMainReturned on the acceptance host instead of
            // the caller-return host; the wire count is authoritative.
            Self::FinishSpriteMain { .. } => None,
            _ => Some(0),
        }
    }

    /// The boundary-qualified stage receipt the adapter appends at host
    /// finish when this scheduled pre-overworld call returns to main wait.
    pub(crate) const fn pre_overworld_stage_completion(
        self,
    ) -> Option<crate::PreOverworldStageCompletion> {
        match self {
            Self::FinishPreOverworldProperties { .. } => {
                Some(crate::PreOverworldStageCompletion::PropertiesReturned)
            }
            Self::FinishPreOverworldOverlays => {
                Some(crate::PreOverworldStageCompletion::OverlaysReturned)
            }
            Self::FinishPreOverworldScreenBuild => {
                Some(crate::PreOverworldStageCompletion::ScreenBuildReturned)
            }
            _ => None,
        }
    }

    /// The table/control and Link/OAM work owned by these closing spotlight
    /// continuations has already returned.  Their terminal source receipt
    /// therefore owns only the saved main-loop suffix and display projection.
    pub(crate) const fn terminal_spotlight_suffix_only_iteration(
        self,
    ) -> Option<SpotlightIteration> {
        match self {
            Self::FinishSpotlightIteration { iteration }
            | Self::FinishDungeonExitSpotlightLinkOam { iteration } => Some(iteration),
            _ => None,
        }
    }

    pub(crate) const fn can_return_directly_to_same_host_fresh_iteration(self) -> bool {
        matches!(
            self,
            Self::FinishSpotlightIteration { .. }
                | Self::FinishDungeonSupertileTransition {
                    work: DungeonSupertileTransitionWork::State13CallerReturn,
                }
        )
    }

    pub(crate) const fn same_host_fresh_iteration_completion_is_authoritative(self) -> bool {
        matches!(self, Self::FinishSpotlightIteration { .. })
    }

    pub(crate) const fn trailing_nmi_uses_live_oam_shadow(self) -> bool {
        matches!(
            self,
            Self::FinishDungeonAfterSubmoduleCallerReturn
                | Self::FinishDungeonPushBlocks { .. }
                | Self::FinishDungeonPushBlockHandler { .. }
                | Self::FinishDungeonPostSpriteMainCallerReturn
                | Self::FinishModule09LinkOamCallerReturn { .. }
                | Self::FinishNmiPrepareSpritesCallerReturn { .. }
        )
    }

    pub(crate) const fn completion_publication(
        self,
        cpu_slice_entry: BgScrollRegisterScanout,
    ) -> GameWorkCompletionPublication {
        match self {
            Self::FinishOverworldAuxGraphics
            | Self::FinishOverworldMosaicSpriteGraphics
            | Self::FinishOverworldSpecialExitMosaic
            | Self::FinishOverworldSpecialExitMosaicSecondDecode
            | Self::HoldOverworldSpriteReloadReturn => GameWorkCompletionPublication {
                bg_scroll: Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
                obj: None,
            },
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
            Self::FinishDungeonSupertileTransition {
                work:
                    DungeonSupertileTransitionWork::State13CallerReturn
                    | DungeonSupertileTransitionWork::FadedFilterPreCompletionCallerReturn
                    | DungeonSupertileTransitionWork::FadedFilterCallerReturn,
            } => GameWorkCompletionPublication {
                bg_scroll: Some(DisplayBgScrollGeneration::ComposeLiveAfterNmi),
                obj: Some(ObjScanoutGenerations {
                    oam: OamScanoutSource::RetainCapturedBeforeNmi,
                    link_obj: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                    link_obj_sources: GraphicsDmaGeneration::HostBoundaryBeforeMain,
                }),
            },
            _ => GameWorkCompletionPublication {
                bg_scroll: None,
                obj: None,
            },
        }
    }
}

/// `Module19_TriforceRoom`'s blocking cases (route hosts 1557656-1557723).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TriforceRoomLoadStep {
    /// Case 2's `LoadCreditsSongs` upload. Live execution waits for the
    /// source palette checkpoint which proves the upload returned; the
    /// measured 15 hosts remain only a non-live fallback.
    Case2Upload,
    /// Case 2's `Overworld_EnterSpecialArea` after a host boundary exposed a
    /// source-order prefix of the OWBG2 palette load.
    Case2SpecialAreaPalettes {
        room_bak: u8,
        completed_ow_bg2_words: u8,
    },
    /// Case 2's `Overworld_LoadOverlays2` under held vblanks, then the
    /// subsubmodule advance and the module tail (route hosts
    /// 1557672-1557676).
    Case2Overlays,
    /// Case 3: `InitializeTilesets` and the palette loads.
    Case3Tilesets,
    /// Case 4: `Module08_02_LoadAndAdvance` and the Triforce-room setup.
    Case4Screen,
    /// Case 7's `TriforceRoom_PrepGFXSlotForPoly`: a fixed two hosts after
    /// the iteration host; the message index and `Main_ShowTextMessage`
    /// (module $0E) run in its last host (route hosts 1557788-1557789).
    Case7PolyGraphics,
    /// Case 7's `RenderText` initialization under held vblanks, then the
    /// case restores module $19 (route hosts 1557790-1557809, whose return
    /// host also publishes the decoder's dialogue-close fact).
    Case7TextInit,
    /// Case 9's text scroll: one `RenderText` scroll call (five passes)
    /// holds the iteration across the main thread's 81-line slots until the
    /// wire returns it (route hosts 1558261-1558267).
    Case9Scroll,
    /// Cases 8-13 after `AdvancePolyhedral`: the ROM's main thread owns
    /// only the V-IRQ-to-vblank slot, so a text render, palette animation, or
    /// filter step often crosses hosts; its writes (submodule advances)
    /// land where the wire returns the iteration (route hosts 1563689,
    /// 1563886).
    IterationTail { case: u8 },
    /// One Module1A credits iteration whose host publishes no suffix: the
    /// scene loads block the main thread for many hosts (route host 1563904:
    /// entry through `LoadOverworldFromDungeon` and the music/ambient clears
    /// land in the entry host; `DecompressAnimatedOverworldTiles` and
    /// `InitializeTilesets` then hold the iteration 38 hosts, and the
    /// palettes, font, scroll, and text writes land at the return host
    /// 1563942; the dungeon scene at 1564763 likewise completes
    /// `Dungeon_LoadEntrance` in its entry host and returns 45 hosts later).
    /// `prefix` records which entry-host part already ran.
    CreditsIteration { prefix: CreditsEntryHostPrefix },
    /// A blocking credits scene loader returned and published a prefix of
    /// `Credits_AddEndingSequenceText`; the source iteration itself remains
    /// suspended until the common main-loop suffix returns.
    CreditsTextAfterSceneLoad { completed_payload_bytes: u16 },
    /// `EndSequence_32` has copied both SRAM save blocks and reached the
    /// source checksum loop. The accumulator is native semantic state: it
    /// preserves exactly the words read before the host boundary without
    /// leaking a source CPU register into gameplay.
    CreditsEndSequence32AfterSaveChecksum {
        completed_checksum_words: u16,
        accumulated_word_sum: u16,
    },
}

/// The whirlpool-warp steps whose ROM calls span several held vblanks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Module09LongLoadStep {
    /// Case 3's first source phase: `FindPartnerWhirlpoolExit` is suspended in
    /// the second `Sprite_ReloadAll_Overworld` call inside
    /// `Overworld_LoadBirdTravelPos`. The reset generation left by the first
    /// `Sprite_ResetAll` remains live until the source reports individual slot
    /// activations and the nested reload return.
    LoadOverlays2,
    /// Case 5: `Overworld_LoadOverlayAndMap`'s screen build.
    LoadOverlayAndMap,
    /// Case 7: `Module09_LoadAuxGFX`'s auxiliary decompression.
    LoadAuxGraphics,
    /// Case 9: `LoadNewSpriteGFXSet` after the synchronous palette loads.
    SpritePalettesAndGraphics,
    /// Case 12: `ReloadPreviouslyLoadedSheets`' seven-sheet decompression.
    ReloadSheets,
    /// `MirrorWarp_FinalizeAndLoadDestination` (Module09/$23 subsubmodule 4):
    /// the same seven-sheet reload, then the song list and module return
    /// (route host 210261).
    MirrorWarpReloadSheets,
    /// `AnimateMirrorWarp` step 8, `MirrorWarp_LoadSpritesAndColors`: the
    /// main-page Map16ToMap8 conversion spans the wire's held vblanks, and the
    /// palette loads, sprite reset/reload, and portal spawn follow it; the
    /// waving-table builder that called the animation resumes after (route
    /// host 211558). This first phase runs the fixed-size conversion, the
    /// palette loads, and `Sprite_ResetAll` on its estimate; the wire
    /// publishes no boundary inside it.
    MirrorWarpSpriteLoadMap16,
    /// The second phase of `MirrorWarp_LoadSpritesAndColors`: the overworld
    /// sprite reload. Its source return publishes the rebuilt generation but
    /// does not yet authorize the caller's item/player reset or portal spawn.
    MirrorWarpSpriteLoadReload,
    /// The same two phases when Module15's Agahnim warp (submodules 3/4) runs
    /// the animation: that caller has no Sprite_Main/LinkOam suffix, only the
    /// wave-completion submodule advance (route host 315211).
    Module15MirrorWarpSpriteLoadMap16,
    Module15MirrorWarpSpriteLoadReload,
    /// `KillAghanim_Func5`: `Main_ShowTextMessage` switches to Module0E, then
    /// `ReloadPreviouslyLoadedSheets` spans the held vblanks under that
    /// module; the HUD rebuild and the Module15/6 return follow, and the
    /// caller's Sprite_Main/LinkOam suffix runs (route host 315323).
    Module15ReloadSheetsAfterMessage,
    /// The suffix after `Sprite_ReloadAll_Overworld` returned: Link's item
    /// reset, the torch/player reset, portal spawn, and waving-table caller
    /// remainder. These appended variants preserve every existing serialized
    /// scheduler discriminant.
    MirrorWarpSpriteLoadTail,
    Module15MirrorWarpSpriteLoadTail,
    /// Case 3 after `Sprite_ReloadAll_Overworld` returns: Link/ambient and
    /// overlay-selection prefixes are live, while `LoadOverworldOverlay` and
    /// the enclosing Module09 caller remain suspended.
    LoadOverlays2Overlay,
    MirrorWarpInteractiveCleanup {
        slot: u8,
        module15: bool,
    },
    MirrorWarpInteractiveTypeClear {
        slot: u8,
        module15: bool,
    },
    MirrorWarpPortalReset {
        slot: u8,
        completed_stores: u8,
        module15: bool,
    },
    MirrorWarpPortalLoadProperties {
        slot: u8,
        completed_stores: u8,
        module15: bool,
    },
}

impl Module09LongLoadStep {
    /// Whether the live wire's caller return (or its Sprite_Main activity)
    /// decides this step's completion; the remaining steps complete on their
    /// fixed-cost estimate because the wire publishes nothing inside them.
    pub(crate) const fn completes_by_wire(self) -> bool {
        !matches!(
            self,
            Self::MirrorWarpSpriteLoadMap16 | Self::Module15MirrorWarpSpriteLoadMap16
        )
    }

    /// Whether Module15's Agahnim warp, not Module09, owns the caller suffix.
    pub(crate) const fn caller_is_module15(self) -> bool {
        matches!(
            self,
            Self::Module15MirrorWarpSpriteLoadMap16
                | Self::Module15MirrorWarpSpriteLoadReload
                | Self::Module15MirrorWarpSpriteLoadTail
                | Self::Module15ReloadSheetsAfterMessage
                | Self::MirrorWarpInteractiveCleanup { module15: true, .. }
                | Self::MirrorWarpInteractiveTypeClear { module15: true, .. }
                | Self::MirrorWarpPortalReset { module15: true, .. }
                | Self::MirrorWarpPortalLoadProperties { module15: true, .. }
        )
    }

    /// Whether the Module15 caller runs its Sprite_Main/LinkOam suffix after
    /// this step returns (submodule 6 does; the warp animation submodules do
    /// not).
    pub(crate) const fn module15_runs_sprite_main_after(self) -> bool {
        matches!(self, Self::Module15ReloadSheetsAfterMessage)
    }
}

pub struct LoadFuncState<'a> {
    pub(crate) p: &'a [u8],
    pub(crate) pos: usize,
}

impl<'a> LoadFuncState<'a> {
    pub fn new(p: &'a [u8]) -> Self {
        Self { p, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.p.len().saturating_sub(self.pos)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DialogueScrollCompletionTiming {
    AfterReturnBoundary,
    BeforeNextVblank,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DungeonState13PublicationPhase {
    #[default]
    None,
    PreMainQuadrantNmiEntry,
    RecurringMain,
    AtomicCallerReturn,
    CallerReturn,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DungeonFadedFilterPublicationPhase {
    #[default]
    None,
    LandingPaletteCompletion,
    CallerReturn,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub(crate) struct DialogueScrollContinuation(u8);

impl DialogueScrollContinuation {
    pub(crate) const IDLE: Self = Self(0);
    pub(crate) const RETURN_ONLY: Self = Self(1);
    pub(crate) const COPY_REMAINING_PIXELS_THEN_RETURN: Self = Self(2);
    pub(crate) const COPY_REMAINING_PIXELS_BEFORE_VBLANK: Self = Self(3);
    pub(crate) const COMPLETION_PENDING_PUBLICATION: Self = Self(4);

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

    pub(crate) fn is_copying_remaining_pixels(self) -> bool {
        matches!(
            self,
            Self::COPY_REMAINING_PIXELS_THEN_RETURN | Self::COPY_REMAINING_PIXELS_BEFORE_VBLANK
        )
    }

    pub(crate) fn completion_timing(self) -> DialogueScrollCompletionTiming {
        debug_assert!(self.is_copying_remaining_pixels());
        if self == Self::COPY_REMAINING_PIXELS_BEFORE_VBLANK {
            DialogueScrollCompletionTiming::BeforeNextVblank
        } else {
            DialogueScrollCompletionTiming::AfterReturnBoundary
        }
    }

    pub(crate) fn is_return_only(self) -> bool {
        self == Self::RETURN_ONLY
    }

    pub(crate) fn is_completion_pending_publication(self) -> bool {
        self == Self::COMPLETION_PENDING_PUBLICATION
    }

    pub(crate) fn finish_remaining_pixels(
        &mut self,
        completion_timing: DialogueScrollCompletionTiming,
    ) {
        debug_assert!(self.is_copying_remaining_pixels());
        *self = match completion_timing {
            DialogueScrollCompletionTiming::AfterReturnBoundary => Self::RETURN_ONLY,
            DialogueScrollCompletionTiming::BeforeNextVblank => {
                Self::COMPLETION_PENDING_PUBLICATION
            }
        };
    }

    pub(crate) fn publish_early_completion(&mut self) {
        debug_assert!(self.is_completion_pending_publication());
        *self = Self::IDLE;
    }

    pub(crate) fn finish_return(&mut self) {
        debug_assert!(self.is_return_only());
        *self = Self::IDLE;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DialogueScrollPhase {
    Idle,
    CopyingRemainingPixels {
        completion_timing: DialogueScrollCompletionTiming,
    },
    ReturnOnly,
    CompletionPendingPublication,
    CompletionStagedAfterFrozenScanout,
    CompletionStagedAfterSnapshot,
    CompletedScroll,
    RetiredTextDma,
}
