//! Temporary pinned-Snes9x adapter for Zelda-level semantic receipts.
//!
//! Emulator PCs and WRAM addresses are allowed only in this replaceable host
//! adapter. Translated gameplay receives the typed values from `zelda3`.

use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zelda3::{
    CachedSpriteCacheField, CachedSpriteExecutionProgress, CachedSpriteExecutionProgressReceipt,
    DialogueExecutionProgress, DungeonLoadSpritesCpuProgress, DungeonResetSpritesCpuProgress,
    DungeonResetSpritesProgressReceipt, DungeonSpriteDisableCpuProgress,
    DungeonSpriteLoadCheckpoint, ItemReceiptGraphicsCaller, ItemReceiptGraphicsProgressReceipt,
    JoypadPublication, MainLoopInterruption, MainLoopProgress, NmiPpuRegisterOperands,
    NmiUpdateGate, OriginalTimingBoundary, OriginalTimingSemanticReceipt,
    OverworldSpriteReloadProgress, PreOverworldStageCompletion, SaveMenuInitializationProgress,
    SourceCallProgress, SpotlightTableBuildCheckpoint, SpotlightTableBuildProgress,
    SpotlightTableBuildProgressReceipt, SpriteMainProgress, SpriteResetAllProgress,
    SpriteResetAllProgressReceipt,
};

const TRACE_PATH_ENV: &str = "ZELDA3_SNES9X_TRACE";
const TRACE_EVENTS_ENV: &str = "ZELDA3_SNES9X_TRACE_EVENTS";
const TRACE_WRAM_ENV: &str = "ZELDA3_SNES9X_TRACE_WRAM";
const TRACE_PCS_ENV: &str = "ZELDA3_SNES9X_TRACE_PCS";
const REQUIRED_TRACE_EVENTS: &[&str] = &["frame", "nmi", "nmi-resume", "wram", "rom-rng", "pc"];

const FRAME_COUNTER: u16 = 0x001a;
const NMI_UPDATE_LATCH: u16 = 0x0012;
// The pinned ROM has two source paths through `Interrupt_NMI`. The ordinary
// path finishes its final PPU write at $00:8221 and reaches REP at $00:8225;
// the active IRQ/poly-thread path jumps through `NMI_SwitchThread`, finishes
// its final PPU write at $00:82c4, and reaches REP at $00:82c7. At either REP
// all Zelda-visible NMI work is complete and only register/stack restoration
// remains. These addresses are private adapter provenance;
// gameplay sees only `NmiHandlerCompleted`.
const NMI_HANDLER_COMPLETE_PCS: [u32; 2] = [0x0000_8225, 0x0000_82c7];
// ROM $00:8051 is `INC $1a`, the first statement of ZeldaRunGameLoop.
// The generic WRAM hook observes the instruction's post-write PC.
const ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC: u32 = 0x008053;
// ROM $00:805d is `STZ $12`, the final semantic operation in
// ZeldaRunGameLoop's unconditional `NMI_PrepareSprites(); nmi_boolean = 0`
// suffix. The generic WRAM hook observes the instruction's post-write PC.
// This exact publication remains valid when the CPU immediately switches to
// Zelda's poly stack, where neither a main-wait PC nor a host-return PC can
// prove the completed source call.
const ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC: u32 = 0x00805f;
// In pinned ROM source, $00:f375 is the JSR to
// IrisSpotlight_CalculateCircleValue. The current loop iteration has loaded
// its input and conditionally decremented spotlight_var4, but has not yet
// calculated or stored either HDMA-table word.
const IRIS_SPOTLIGHT_CIRCLE_VALUE_CALL_PC: u32 = 0x00f375;
// ROM $00:f364 is the direct-page store which initializes the current loop
// iteration's `r8 = 0xff`. At this instruction boundary the store and every
// table publication for the iteration remain pending. The adapter derives the
// completed C iteration count from the source `r6` cursor captured at the NMI
// handler entry; gameplay never observes this ROM address or scratch register.
const IRIS_SPOTLIGHT_ITERATION_VALUE_STORE_PC: u32 = 0x00f364;
// The pure circle helper has returned and the source loop has doubled its
// upper cursor, but neither HDMA-table store has executed. Rewind this
// emulator-private instruction boundary to the same resumable C checkpoint as
// the helper call: recalculating the pure value cannot replay a publication.
const IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC: u32 = 0x00f383;
// ROM $00:f392 is the long WRAM store of the already-calculated circle value
// to `hdma_table_dynamic[r6]`. The upper-cursor store at $00:f383 is complete;
// this lower store and the loop-cursor update remain pending.
const IRIS_SPOTLIGHT_LOWER_TABLE_WRITE_PC: u32 = 0x00f392;
// At $00:f39a both table stores are complete and the source loop has compared
// its upper cursor with the vertical center. The branch and any cursor update
// remain pending. The adapter converts the private X register into the two C
// cursors and exports only that resumable statement boundary.
const IRIS_SPOTLIGHT_LOOP_COMPLETION_BRANCH_PC: u32 = 0x00f39a;
// The branch above was not taken and the source has incremented its upper
// cursor at $00:f39c. At $00:f39e only the paired lower-cursor decrement is
// still pending before the next C loop iteration.
const IRIS_SPOTLIGHT_LOWER_CURSOR_DECREMENT_PC: u32 = 0x00f39e;
// `$00:f3a0` is the loop-back JMP reached only after the false completion
// branch incremented r4 and decremented r6.  The current C iteration is fully
// published and the next iteration has not initialized its value yet.  This
// is therefore the same backend-neutral checkpoint as entering the next
// iteration, with one additional completed iteration.
const IRIS_SPOTLIGHT_NEXT_ITERATION_PC: u32 = 0x00f3a0;
// Source memcpy at $00:f3b4..$00:f3c4 copies 224 words from
// `hdma_table_dynamic` to `hdma_table_unused`. These are the only instruction
// boundaries within that loop; the adapter converts X into a copied-word
// count so gameplay never observes a CPU register or ROM address.
const IRIS_SPOTLIGHT_COPY_INIT_PC: u32 = 0x00f3b4;
const IRIS_SPOTLIGHT_COPY_LOAD_PC: u32 = 0x00f3b7;
const IRIS_SPOTLIGHT_COPY_STORE_PC: u32 = 0x00f3bb;
const IRIS_SPOTLIGHT_COPY_FIRST_INCREMENT_PC: u32 = 0x00f3be;
const IRIS_SPOTLIGHT_COPY_SECOND_INCREMENT_PC: u32 = 0x00f3bf;
const IRIS_SPOTLIGHT_COPY_COMPARE_PC: u32 = 0x00f3c0;
const IRIS_SPOTLIGHT_COPY_BRANCH_PC: u32 = 0x00f3c3;
const IRIS_SPOTLIGHT_COPY_COMPLETE_PC: u32 = 0x00f3c5;
// `IrisSpotlight_CalculateCircleValue` is a pure C helper. An NMI may suspend
// inside it after the caller has decremented `spotlight_var4`, but before the
// helper returns a value or either table word is written. The adapter rewinds
// that pure helper to its source-call boundary; gameplay receives only the
// pending circle input, never this ROM range.
/// `IrisSpotlight_ResetTable` ($00:F427) clears the 224-word dynamic HDMA
/// table as seven interleaved `STA $1Bxx,X` stripes per iteration, X running
/// from $3E down to 0 by two. The loop body spans the first store through the
/// `BPL` back-branch; an NMI accepted inside it leaves the goal transition's
/// remaining stores and the caller suffix pending.
const IRIS_SPOTLIGHT_RESET_TABLE_FIRST_STORE_PC: u32 = 0x00f42f;
const IRIS_SPOTLIGHT_RESET_TABLE_FIRST_DEX_PC: u32 = 0x00f444;
const IRIS_SPOTLIGHT_RESET_TABLE_SECOND_DEX_PC: u32 = 0x00f445;
const IRIS_SPOTLIGHT_RESET_TABLE_BRANCH_PC: u32 = 0x00f446;
const IRIS_SPOTLIGHT_RESET_TABLE_INITIAL_X: u16 = 0x3e;
const IRIS_SPOTLIGHT_RESET_TABLE_STORES_PER_ITERATION: u16 = 7;
const IRIS_SPOTLIGHT_CIRCLE_VALUE_START_PC: u32 = 0x00f4cc;
const IRIS_SPOTLIGHT_CIRCLE_VALUE_END_PC: u32 = 0x00f53e;
pub(crate) const SPOTLIGHT_VAR4_LOW_ADDRESS: usize = 0x067a;
pub(crate) const SPOTLIGHT_LOWER_CURSOR_ADDRESS: usize = 0x0006;
const NMI_HANDLER_ENTRY_PC: u32 = 0x0080c9;
const DUNGEON_CACHE_TRANS_SPRITES_START_PC: u32 = 0x09c176;
const DUNGEON_CACHE_TRANS_SPRITES_END_PC: u32 = 0x09c244;
const DUNGEON_RESET_SPRITES_CLEAR_PC: u32 = 0x09c244;
const SPRITE_DISABLE_ALL_END_PC: u32 = 0x09c290;
const SPRITE_DISABLE_ALL_FINAL_GARNISH_PC: u32 = 0x09c281;
const GARNISH_TYPE_SLOT_ZERO: u16 = 0x0b00;
const ANCILLA_TYPE_BASE: u16 = 0x0c4a;
const ANCILLA_PICKUP_FLAG: u16 = 0x02ec;
const SPRITE_LIMIT_INSTANCE: u16 = 0x0b6a;
const DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC: u32 = 0x09c38c;
const DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC: u32 = 0x09c3a9;
const DUNGEON_LOAD_SINGLE_SPRITE_END_PC: u32 = 0x09c400;
// `Module_PreDungeon` calls `Sprite_ResetAll` at $02:8347; the return address
// exposed by the pinned trace is $02:834b. The shared reset routine itself is
// adapter-private provenance; gameplay receives only its semantic checkpoint.
const MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC: u32 = 0x02_834b;
const SPRITE_RESET_ALL_NO_DISABLE_START_PC: u32 = 0x09_c452;
const SPRITE_RESET_ALL_END_PC: u32 = 0x09_c499;
const NMI_PREPARE_SPRITES_START_PC: u32 = 0x0085fc;
// The executable body ends at the pinned ROM's shared `JumpTableLocal`
// helper.  The following address range also contains `JumpTableLong`,
// `Startup_InitializeMemory`, and embedded tables before the next translated
// C function at $00:8901.  Those helpers can be called from unrelated source
// stacks, so classifying through the next C symbol would turn a private PC
// overlap into a false `NMI_PrepareSprites` receipt.
const NMI_PREPARE_SPRITES_END_PC: u32 = 0x008781;
// Before the first store for one unrolled four-byte group, every previously
// visited extended-OAM group is complete and the current group is still
// unpublished. The backend-private Y register identifies that resumable
// source cursor; translated gameplay never sees the register or PC.
const NMI_PREPARE_EXTENDED_OAM_GROUP_BEFORE_STORE_START_PC: u32 = 0x008602;
// `$8614` is the first STA opcode. NMI acceptance is instruction-boundary
// atomic, so observing this PC still proves the current group is unpublished;
// `$8615/$8616` are operand bytes and cannot be acceptance PCs.
const NMI_PREPARE_EXTENDED_OAM_GROUP_BEFORE_STORE_END_PC: u32 = 0x008615;
const LINK_OAM_START_PC: u32 = 0x0da18e;
const LINK_OAM_END_PC: u32 = 0x0dadb6;
// The generic PC trace uses these private pinned-ROM boundaries to translate
// the descending `Sprite_Main` loop into a Zelda-level resumable slot receipt.
// Neither address nor the CPU X register crosses the adapter boundary.
const SPRITE_MAIN_ENTRY_PC: u32 = 0x068328;
const SPRITE_EXECUTE_SINGLE_ENTRY_PC: u32 = 0x0684e2;
const SPRITE_SLOT_RETURN_PC: u32 = 0x0683a7;
const SPRITE_MAIN_RETURN_PC: u32 = 0x028842;
// Final subtype store in the state-10 `Chicken_IncrSubtype2(k, 3)` call.
// The graphics store and the rest of the current Cucco handler remain pending.
const CUCCO_SUBTYPE_INCREMENT_PUBLICATION_PCS: [u32; 5] =
    [0x06_a6e5, 0x06_a6e8, 0x06_a6eb, 0x06_a6ee, 0x06_a6f1];
// Final graphics-generation store in `Chicken_IncrSubtype2_3` ($86:a6e5).
// The following source call is `Sprite_ReturnIfLifted`; an NMI may interrupt
// that unfinished tail after all Cucco animation writes are already visible.
const CUCCO_ANIMATION_PUBLICATION_PC: u32 = 0x06_a6fa;
// `Cucco_Flee` calls `Chicken_IncrSubtype2` only after its XY movement,
// `sprite_z = 0`, and optional velocity retarget have all completed. This
// private call site becomes a source-level movement-completion receipt.
const CUCCO_FLEE_SUBTYPE_HELPER_CALL_PC: u32 = 0x06_a724;
// The active-Cucco branch enters `Sprite_MoveXY` here. The PC is private
// adapter provenance; gameplay receives only the C assignment boundary.
const ACTIVE_CUCCO_MOVEMENT_CALL_PC: u32 = 0x06_a628;
const SPRITE_X_SUBPIXEL_BASE: u16 = 0x0d70;
const SPRITE_X_LOW_BASE: u16 = 0x0d10;
const SPRITE_X_HIGH_BASE: u16 = 0x0d30;
const SPRITE_Y_SUBPIXEL_BASE: u16 = 0x0d60;
const SPRITE_GRAPHICS_BASE: u16 = 0x0dc0;
const SPRITE_SUBTYPE2_BASE: u16 = 0x0e80;
// `PrepareEnemyDrop` stores the replacement sprite type immediately before
// entering `SpritePrep_BigKey_load_graphics`. The private ROM address is used
// only to translate that source statement into a typed gameplay receipt.
const BIG_KEY_DROP_TYPE_PUBLICATION_PC: u32 = 0x06_f9d4;
const BIG_KEY_DROP_SPRITE_TYPE: u8 = 0xe5;
// Pinned Link_HandleVelocity has a second, earlier source-equivalent boundary:
// after `$87:e274 LDA link_player_handler_state`, the saved PC is `$87:e276`
// and no Zelda state has been changed yet.  The following CMP/branch is also
// side-effect free.  Keep this private adapter range separate from the wider
// Link_MovePosition range so unrelated Link_HandleVelocity branches cannot be
// mistaken for the same semantic checkpoint.
const LINK_VELOCITY_BEFORE_STATE_BRANCH_START_PC: u32 = 0x07e276;
const LINK_VELOCITY_BEFORE_STATE_BRANCH_END_PC: u32 = 0x07e27a;
// Pinned Link_MovePosition ($87:e370) copies Link's current coordinates and
// safe-return bytes before its first coordinate integration store at $87:e3af.
// Bank $07 is the executing LoROM mirror observed by the maintained core.
const LINK_POSITION_BEFORE_COORDINATES_START_PC: u32 = 0x07e370;
const LINK_POSITION_BEFORE_COORDINATES_END_PC: u32 = 0x07e3af;
// Link_MovePosition's axis loop between `STA $2A,y` (the subpixel store) and
// `ADC $20,x` (the coordinate add): the current axis' subpixel is published,
// its coordinate is not. X names the pass (4 = z, 2 = y, 0 = x).
const LINK_POSITION_AFTER_SUBPIXEL_START_PC: u32 = 0x07e3b2;
const LINK_POSITION_AFTER_SUBPIXEL_END_PC: u32 = 0x07e3c6;
const VWF_RENDER_SINGLE_START_PC: u32 = 0x0ecab8;
const VWF_RENDER_SINGLE_END_PC: u32 = 0x0ecd1a;
const UNCACHE_SPRITE_START_PC: u32 = 0x1dea00;
const UNCACHE_SPRITE_RESTORE_START_PC: u32 = 0x1deb06;
const UNCACHE_SPRITE_END_PC: u32 = 0x1deb68;
const SPRITE_STATE_BASE: u16 = 0x0dd0;
const SPRITE_Y_HIGH_BASE: u16 = 0x0d20;
const SPRITE_N_WORD_BASE: u16 = 0x0bc0;
const SPRITE_TYPE_BASE: u16 = 0x0e20;
const SPRITE_DIE_ACTION_BASE: u16 = 0x0f20;
const OVERWORLD_SPRITE_SCAN_START_PC: u32 = 0x09c55e;
const OVERWORLD_SPRITE_SCAN_END_PC: u32 = 0x09c881;
const OVERWORLD_LOAD_SINGLE_SPRITE_START_PC: u32 = 0x09c770;
const OVERWORLD_LOAD_SINGLE_SPRITE_END_PC: u32 = 0x09c80b;
// BottleVendor case 2 calls Link_ReceiveItem synchronously at $85:eb1d.
// The call's graphics decompressor may span several host returns; $85:eb21 is
// the first instruction of the caller suffix after that JSL returns.  These
// addresses remain private adapter provenance.
// BottleVendor_GrantBottle is commonly symbolicated through the $85 LoROM
// mirror, but the cold route executes the source call through PB=$05.  Trace
// the actual runtime addresses; the typed receipt below deliberately hides
// this emulator-private provenance from translated gameplay.
const BOTTLE_VENDOR_ITEM_RECEIPT_CALL_PC: u32 = 0x05eb1d;
const BOTTLE_VENDOR_ITEM_RECEIPT_RETURN_PC: u32 = 0x05eb21;
// SickKid case 2 performs the same synchronous source call at $06:b9cc. The
// caller suffix begins at $06:b9d0 with PLX, then advances the Sick Kid state
// and releases Link. These PCs remain private adapter provenance.
const SICK_KID_ITEM_RECEIPT_CALL_PC: u32 = 0x06b9cc;
const SICK_KID_ITEM_RECEIPT_RETURN_PC: u32 = 0x06b9d0;
// `Link_ReceiveItem` is shared by direct sprite pickups as well as the
// caller-specific BottleVendor/SickKid paths above. At $07:9a0b its nested
// `AncillaAdd_ItemReceipt` graphics call has returned; the remaining HUD/dash
// suffix is still ordinary synchronous Zelda code.
const LINK_RECEIVE_ITEM_ENTRY_PC: u32 = 0x0799ad;
const LINK_RECEIVE_ITEM_GRAPHICS_RETURN_PC: u32 = 0x079a0b;
// Uncle_InPassage case 1 calls Link_ReceiveItem at $85:df49 and resumes its
// state/progress suffix at $85:df4d. The route executes the $05 LoROM mirror;
// these addresses stay private to the oracle adapter.
const UNCLE_PASSAGE_ITEM_RECEIPT_CALL_PC: u32 = 0x05df49;
const UNCLE_PASSAGE_ITEM_RECEIPT_RETURN_PC: u32 = 0x05df4d;

// Live-slot statement order in UncacheAndExecuteSprite. These addresses are
// Snes9x-adapter provenance only; the emitted receipt carries semantic counts.
const CACHED_SPRITE_LIVE_FIELDS: [u16; 24] = [
    0x0dd0, 0x0e20, 0x0d10, 0x0d30, 0x0d00, 0x0d20, 0x0dc0, 0x0d90, 0x0eb0, 0x0f50, 0x0b89, 0x0de0,
    0x0e40, 0x0f20, 0x0d80, 0x0e60, 0x0da0, 0x0db0, 0x0e90, 0x0e80, 0x0f70, 0x0df0, 0xf9c2, 0x0ba0,
];

const CACHE_FIELD_WRITES: [(CachedSpriteCacheField, u16); 25] = [
    (CachedSpriteCacheField::StateClear, 0x1d00),
    (CachedSpriteCacheField::Type, 0x1d10),
    (CachedSpriteCacheField::XLow, 0x1d20),
    (CachedSpriteCacheField::Graphics, 0x1d60),
    (CachedSpriteCacheField::XHigh, 0x1d30),
    (CachedSpriteCacheField::YLow, 0x1d40),
    (CachedSpriteCacheField::YHigh, 0x1d50),
    (CachedSpriteCacheField::State, 0x1d00),
    (CachedSpriteCacheField::A, 0x1d70),
    (CachedSpriteCacheField::HeadDirection, 0x1d80),
    (CachedSpriteCacheField::OamFlags, 0x1d90),
    (CachedSpriteCacheField::ObjPriority, 0x1da0),
    (CachedSpriteCacheField::D, 0x1db0),
    (CachedSpriteCacheField::Flags2, 0x1dc0),
    (CachedSpriteCacheField::Floor, 0x1dd0),
    (CachedSpriteCacheField::SpawnedFlag, 0x1de0),
    (CachedSpriteCacheField::Flags3, 0x1df0),
    (CachedSpriteCacheField::B, 0xfa5c),
    (CachedSpriteCacheField::C, 0xfa6c),
    (CachedSpriteCacheField::E, 0xfa7c),
    (CachedSpriteCacheField::Subtype2, 0xfa8c),
    (CachedSpriteCacheField::HeightAboveShadow, 0xfa9c),
    (CachedSpriteCacheField::DelayMain, 0xfaac),
    (CachedSpriteCacheField::I, 0xfacc),
    (CachedSpriteCacheField::IgnoreProjectile, 0xfadc),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CacheWriteProgress {
    slot: u8,
    next_field_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CachedSpriteExecutionTracker {
    slot: u8,
    copied_fields: u8,
    restored_fields: u8,
    restore_started: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct OverworldSpriteActivationTracker {
    slot: u8,
    block_low: Option<u8>,
    block_high: Option<u8>,
    sprite_type: Option<u8>,
    state_published: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct SpriteMainExecutionTracker {
    current_slot: Option<u8>,
    last_completed_slot: Option<u8>,
    #[serde(default)]
    cucco_subtype_increments: Option<(u8, u8, u8)>,
    #[serde(default)]
    cucco_helper_ordinal: u8,
    #[serde(default)]
    cucco_flee_movement: Option<(u8, u8)>,
    #[serde(default)]
    active_cucco_movement: Option<(u8, u8)>,
    #[serde(default)]
    active_cucco_x_publications: u8,
    #[serde(default)]
    active_cucco_y_subpixel: Option<(u8, u8)>,
    #[serde(default)]
    cucco_animation_slot: Option<(u8, u8)>,
    #[serde(default)]
    big_key_drop_graphics_slot: Option<u8>,
}

impl SpriteMainExecutionTracker {
    fn progress(self) -> SpriteMainProgress {
        if let Some(slot) = self.big_key_drop_graphics_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "big-key graphics publication outlived its active sprite slot",
            );
            assert_eq!(
                self.cucco_animation_slot, None,
                "one sprite slot published two incompatible partial checkpoints",
            );
            assert_eq!(
                self.cucco_subtype_increments, None,
                "one sprite slot published two incompatible partial checkpoints",
            );
            return SpriteMainProgress::BigKeyDropGraphicsStarted(slot);
        }
        if let Some((slot, helper_ordinal)) = self.cucco_animation_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Cucco animation publication outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterCuccoGraphicsPublication {
                slot,
                helper_ordinal,
            };
        }
        if let Some((slot, helper_ordinal)) = self.cucco_flee_movement {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Cucco flee movement publication outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterCuccoFleeMovement {
                slot,
                helper_ordinal,
            };
        }
        if let Some((slot, helper_ordinal)) = self.active_cucco_y_subpixel {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Cucco flee Y-subpixel publication outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterActiveCuccoYSubpixel {
                slot,
                helper_ordinal,
            };
        }
        if let Some((slot, helper_ordinal)) = self.active_cucco_movement {
            if self.active_cucco_x_publications == 3 {
                assert_eq!(
                    self.current_slot,
                    Some(slot),
                    "active Cucco X publication outlived its active sprite slot",
                );
                return SpriteMainProgress::AfterActiveCuccoX {
                    slot,
                    helper_ordinal,
                };
            }
        }
        if let Some((slot, helper_ordinal, completed)) = self.cucco_subtype_increments {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Cucco subtype publication outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterCuccoSubtypeIncrements {
                slot,
                helper_ordinal,
                completed,
            };
        }
        self.last_completed_slot.map_or(
            SpriteMainProgress::BeforeFirstSlot,
            SpriteMainProgress::AfterSlot,
        )
    }

    fn interruption(self) -> MainLoopInterruption {
        match self.progress() {
            SpriteMainProgress::BeforeFirstSlot => MainLoopInterruption::SpriteMainBeforeFirstSlot,
            SpriteMainProgress::AfterSlot(slot) => MainLoopInterruption::SpriteMainAfterSlot(slot),
            SpriteMainProgress::AfterActiveCuccoX {
                slot,
                helper_ordinal,
            } => MainLoopInterruption::SpriteMainAfterActiveCuccoX {
                slot,
                helper_ordinal,
            },
            SpriteMainProgress::AfterActiveCuccoYSubpixel {
                slot,
                helper_ordinal,
            } => MainLoopInterruption::SpriteMainAfterActiveCuccoYSubpixel {
                slot,
                helper_ordinal,
            },
            SpriteMainProgress::AfterCuccoFleeMovement {
                slot,
                helper_ordinal,
            } => MainLoopInterruption::SpriteMainAfterCuccoFleeMovement {
                slot,
                helper_ordinal,
            },
            SpriteMainProgress::AfterCuccoSubtypeIncrements {
                slot,
                helper_ordinal,
                completed,
            } => MainLoopInterruption::SpriteMainAfterCuccoSubtypeIncrements {
                slot,
                helper_ordinal,
                completed,
            },
            SpriteMainProgress::AfterCuccoGraphicsPublication {
                slot,
                helper_ordinal,
            } => MainLoopInterruption::SpriteMainAfterCuccoGraphicsPublication {
                slot,
                helper_ordinal,
            },
            SpriteMainProgress::BigKeyDropGraphicsStarted(slot) => {
                MainLoopInterruption::SpriteMainBigKeyDropGraphicsStarted(slot)
            }
        }
    }
}

impl CachedSpriteExecutionTracker {
    fn from_observed_write(pc: u32, slot: u8, field_index: usize) -> Self {
        if pc >= UNCACHE_SPRITE_RESTORE_START_PC {
            Self {
                slot,
                copied_fields: CACHED_SPRITE_LIVE_FIELDS.len() as u8,
                restored_fields: (CACHED_SPRITE_LIVE_FIELDS.len() - field_index) as u8,
                restore_started: true,
            }
        } else {
            Self {
                slot,
                copied_fields: (field_index + 1) as u8,
                restored_fields: 0,
                restore_started: false,
            }
        }
    }

    fn observe_write(&mut self, pc: u32, slot: u8, field_index: usize) -> Result<bool, String> {
        if slot != self.slot {
            return Err(format!(
                "Snes9x UncacheAndExecuteSprite slot changed from {} to {slot}",
                self.slot
            ));
        }
        if pc >= UNCACHE_SPRITE_RESTORE_START_PC && !self.restore_started {
            self.restore_started = true;
            self.restored_fields = 0;
        }
        if self.restore_started {
            let expected = CACHED_SPRITE_LIVE_FIELDS
                .len()
                .checked_sub(usize::from(self.restored_fields) + 1)
                .ok_or("Snes9x UncacheAndExecuteSprite restored past the final live field")?;
            if field_index != expected {
                return Err(format!(
                    "Snes9x UncacheAndExecuteSprite restore expected field {expected}, observed {field_index}"
                ));
            }
            self.restored_fields = self.restored_fields.saturating_add(1);
            Ok(usize::from(self.restored_fields) == CACHED_SPRITE_LIVE_FIELDS.len())
        } else {
            let expected = usize::from(self.copied_fields);
            if field_index != expected {
                return Err(format!(
                    "Snes9x UncacheAndExecuteSprite load expected field {expected}, observed {field_index}"
                ));
            }
            self.copied_fields = self.copied_fields.saturating_add(1);
            Ok(false)
        }
    }

    fn receipt(self) -> CachedSpriteExecutionProgress {
        if self.restore_started {
            CachedSpriteExecutionProgress::Restoring {
                slot: self.slot,
                live_fields: (CACHED_SPRITE_LIVE_FIELDS.len() - usize::from(self.restored_fields))
                    as u8,
            }
        } else {
            CachedSpriteExecutionProgress::Loading {
                slot: self.slot,
                copied_fields: self.copied_fields,
            }
        }
    }
}

pub(crate) struct Snes9xOracleSemanticTrace {
    /// Seed warm-up (oracle-seeded segment starts): the decoder began at a
    /// savestate captured mid-publication, so an NMI handler completion whose
    /// acceptance predates the trace is dropped instead of rejected. Cleared
    /// by the harness once Rust is seeded at a clean run boundary.
    seed_warmup_active: bool,
    path: PathBuf,
    offset: u64,
    cache_write_progress: Option<CacheWriteProgress>,
    normal_load_ordinal: Option<u16>,
    pending_reset_progress: Option<DungeonResetSpritesCpuProgress>,
    cached_sprite_execution: Option<CachedSpriteExecutionTracker>,
    overworld_presence_published: bool,
    overworld_sprite_activation: Option<OverworldSpriteActivationTracker>,
    pending_spotlight_helper_nmi: Option<RawTraceEvent>,
    /// Index of the `NmiAccepted` receipt published for the pending helper
    /// NMI in the current host vector; host-local, never checkpointed.
    pending_spotlight_helper_nmi_acceptance_index: Option<usize>,
    item_receipt_caller: Option<ItemReceiptGraphicsCaller>,
    sprite_main_execution: Option<SpriteMainExecutionTracker>,
    zelda_run_game_loop_call_active: bool,
    nmi_publication_pending: bool,
    pending_nmi_update_gate: Option<NmiUpdateGate>,
    nmi_resume_targets: Vec<(u32, u16)>,
    synthesized_nmi_resume: Option<(u32, u16)>,
    /// Host-local acceptance operands drained with the ordered receipt vector.
    /// They never enter the cross-host semantic decoder checkpoint.
    host_nmi_ppu_register_operands: Vec<NmiPpuRegisterOperands>,
}

const SEMANTIC_TRACE_CHECKPOINT_SCHEMA: u32 = 9;

/// Emulator-private continuation state for the typed semantic adapter.
///
/// This is persisted beside a paired oracle state, never exposed to translated
/// gameplay. It prevents a resumed trace from forgetting source work which
/// crossed the host boundary, such as an accepted NMI whose publication runs
/// during the next `retro_run`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Snes9xOracleSemanticTraceCheckpoint {
    schema: u32,
    cache_write_progress: Option<CacheWriteProgress>,
    normal_load_ordinal: Option<u16>,
    pending_reset_progress: Option<DungeonResetSpritesCpuProgress>,
    cached_sprite_execution: Option<CachedSpriteExecutionTracker>,
    overworld_presence_published: bool,
    overworld_sprite_activation: Option<OverworldSpriteActivationTracker>,
    pending_spotlight_helper_nmi: Option<RawTraceEvent>,
    item_receipt_caller: Option<ItemReceiptGraphicsCaller>,
    #[serde(default)]
    sprite_main_execution: Option<SpriteMainExecutionTracker>,
    #[serde(default)]
    zelda_run_game_loop_call_active: bool,
    nmi_publication_pending: bool,
    #[serde(default)]
    pending_nmi_update_gate: Option<NmiUpdateGate>,
    nmi_resume_targets: Vec<(u32, u16)>,
    synthesized_nmi_resume: Option<(u32, u16)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct RawTraceEvent {
    event: String,
    #[serde(default)]
    stage: Option<String>,
    #[serde(default)]
    run: Option<u64>,
    #[serde(default)]
    pc: Option<u32>,
    #[serde(default)]
    s: Option<u16>,
    #[serde(default)]
    return_address: Option<u32>,
    #[serde(default)]
    a: Option<u16>,
    #[serde(default)]
    main: Option<u8>,
    #[serde(default)]
    sub: Option<u8>,
    #[serde(default)]
    subsub: Option<u8>,
    #[serde(default)]
    frame_counter: Option<u8>,
    #[serde(default)]
    nmi_latch: Option<u8>,
    #[serde(default)]
    link_y: Option<u16>,
    #[serde(default)]
    bg2_v: Option<u16>,
    #[serde(default)]
    spotlight_radius: Option<u16>,
    #[serde(default)]
    spotlight_var4_low: Option<u8>,
    #[serde(default)]
    spotlight_lower_cursor: Option<u16>,
    #[serde(default)]
    joypad_high: Option<u8>,
    #[serde(default)]
    joypad_low: Option<u8>,
    #[serde(default)]
    joypad_high_filtered: Option<u8>,
    #[serde(default)]
    joypad_low_filtered: Option<u8>,
    #[serde(default)]
    x: Option<u16>,
    #[serde(default)]
    y: Option<u16>,
    #[serde(default)]
    address: Option<u16>,
    #[serde(default)]
    value: Option<u8>,
    #[serde(default)]
    nmi_ppu_register_operands: Option<[u8; 31]>,
}

impl RawTraceEvent {
    fn nmi_ppu_register_operands(&self) -> Result<NmiPpuRegisterOperands, String> {
        let bytes = self
            .nmi_ppu_register_operands
            .ok_or("Snes9x NMI receipt omitted Zelda's WritePpuRegisters acceptance operands")?;
        let word = |low: usize| u16::from_le_bytes([bytes[low], bytes[low + 1]]);
        Ok(NmiPpuRegisterOperands {
            window_selection: [bytes[0], bytes[1], bytes[2]],
            color_window_selection: bytes[3],
            color_math_control: bytes[4],
            fixed_color: [bytes[5], bytes[6], bytes[7]],
            screen_layers: [bytes[8], bytes[9], bytes[10], bytes[11]],
            bg_scroll: [word(12), word(14), word(16), word(18), word(20), word(22)],
            screen_brightness: bytes[24],
            mosaic: bytes[25],
            bg_mode: bytes[26],
            mode7_center: [word(27), word(29)],
        })
    }

    fn joypad_publication(&self) -> Result<Option<JoypadPublication>, String> {
        match (
            self.joypad_high,
            self.joypad_low,
            self.joypad_high_filtered,
            self.joypad_low_filtered,
        ) {
            (None, None, None, None) => Ok(None),
            (Some(high), Some(low), Some(high_filtered), Some(low_filtered)) => {
                Ok(Some(JoypadPublication {
                    high,
                    low,
                    high_filtered,
                    low_filtered,
                }))
            }
            _ => Err(
                "Snes9x NMI publication receipt omitted part of Zelda's joypad state".to_string(),
            ),
        }
    }
}

fn nmi_resume_target(event: &RawTraceEvent) -> Result<(u32, u16), String> {
    Ok((
        event
            .pc
            .ok_or("Snes9x NMI receipt omitted interrupted PC")?
            & 0x00ff_ffff,
        event
            .s
            .ok_or("Snes9x NMI receipt omitted interrupted stack pointer")?,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HostFrameState {
    run: u64,
    pc: u32,
    x: Option<u16>,
    main: u8,
    sub: u8,
    subsub: u8,
    frame_counter: u8,
    nmi_latch: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpotlightCallCompletion {
    EntryReturned,
    RecurringCallerReachedLinkOam,
    RecurringCallerReturnedToMainWait,
    OverworldGoalCallerReturned,
}

#[derive(Default)]
struct HostFrameWindow {
    entry: Option<HostFrameState>,
    returned: Option<HostFrameState>,
    vwf_nmi_observed: bool,
    main_loop_starts: u8,
    main_loop_common_suffix_completed: bool,
    /// The host began inside the previous iteration's common suffix, before
    /// its `$12` clear (entry at `$00:805D`, route host 511525): that leading
    /// completion belongs to the carried iteration and a fresh iteration may
    /// complete its own suffix later in the same host.
    leading_common_suffix_completed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainLoopCompletionProof {
    CommonSuffixCompleted,
}

fn main_loop_started_by_event(event: &RawTraceEvent) -> bool {
    event.event == "wram-write"
        && event.address == Some(FRAME_COUNTER)
        && event.pc.map(|pc| pc & 0x00ff_ffff) == Some(ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC)
}

impl HostFrameWindow {
    fn spotlight_call_completion(&self) -> Option<SpotlightCallCompletion> {
        if matches!(
            (self.entry, self.returned),
            (
                Some(HostFrameState {
                    main: 0x0f,
                    sub: 0,
                    ..
                }),
                Some(HostFrameState {
                    main: 0x0f,
                    sub: 1,
                    ..
                })
            )
        ) {
            return Some(SpotlightCallCompletion::EntryReturned);
        }
        if matches!(
            (self.entry, self.returned),
            (
                Some(HostFrameState {
                    pc: entry_pc,
                    main: 0x0f,
                    sub: 1,
                    nmi_latch: entry_nmi_latch,
                    ..
                }),
                Some(HostFrameState {
                    pc: returned_pc,
                    main: 0x0f,
                    sub: 1,
                    nmi_latch: returned_nmi_latch,
                    ..
                })
            ) if self.main_loop_starts == 0
                && !zelda_main_wait_pc(entry_pc)
                && (zelda_main_wait_pc(returned_pc)
                    || (entry_nmi_latch != 0 && returned_nmi_latch == 0)
                    // The host can end inside the Open NMI handler that
                    // follows the return, with the handler's own latch write
                    // already visible (route host 597513); the observed
                    // common-suffix $12 clear is the same return proof.
                    || self.main_loop_common_suffix_completed)
        ) {
            return Some(SpotlightCallCompletion::RecurringCallerReturnedToMainWait);
        }
        if matches!(
            (self.entry, self.returned),
            (
                Some(HostFrameState {
                    pc: entry_pc,
                    main: 0x0f,
                    sub: 1,
                    ..
                }),
                Some(HostFrameState {
                    pc: returned_pc,
                    main: 0x0f,
                    sub: 1,
                    ..
                })
            ) if self.main_loop_starts == 0
                && !zelda_main_wait_pc(entry_pc)
                && main_loop_interruption_for_source_state(
                    returned_pc,
                    Some(0x0f),
                    Some(1),
                    None,
                )
                    == Some(MainLoopInterruption::LinkOam)
        ) {
            return Some(SpotlightCallCompletion::RecurringCallerReachedLinkOam);
        }
        if matches!(
            (self.entry, self.returned),
            (
                Some(HostFrameState {
                    pc: entry_pc,
                    main: 0x10,
                    sub: 1,
                    nmi_latch: entry_nmi_latch,
                    ..
                }),
                Some(HostFrameState {
                    pc: returned_pc,
                    main: returned_main,
                    nmi_latch: returned_nmi_latch,
                    ..
                })
            ) if self.main_loop_starts == 0
                && returned_main != 0x10
                && !zelda_main_wait_pc(entry_pc)
                && (zelda_main_wait_pc(returned_pc)
                    || (entry_nmi_latch != 0 && returned_nmi_latch == 0))
        ) {
            return Some(SpotlightCallCompletion::OverworldGoalCallerReturned);
        }
        None
    }

    /// Observe one raw source event and report an exact source-call completion
    /// at the event's ordered position. The caller preserves that boundary
    /// relative to NMI acceptance without exporting CPU or WRAM provenance.
    fn observe(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<Option<MainLoopCompletionProof>, String> {
        if main_loop_started_by_event(event) {
            self.main_loop_starts = self
                .main_loop_starts
                .checked_add(1)
                .ok_or("Snes9x host call overflowed its ZeldaRunGameLoop start count")?;
        }
        let common_suffix_completed = event.event == "wram-write"
            && event.address == Some(NMI_UPDATE_LATCH)
            && event.pc.map(|pc| pc & 0x00ff_ffff)
                == Some(ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC);
        if common_suffix_completed {
            if event.value != Some(0) {
                return Err(format!(
                    "Snes9x ZeldaRunGameLoop common suffix published invalid $12 value {:?}",
                    event.value,
                ));
            }
            if self.main_loop_common_suffix_completed
                && !(self.leading_common_suffix_completed && self.main_loop_starts == 1)
            {
                return Err(format!(
                    "Snes9x host call completed ZeldaRunGameLoop's common suffix twice (main_loop_starts={}, entry={:?}, event pc={:?})",
                    self.main_loop_starts, self.entry, event.pc,
                ));
            }
            if self.main_loop_starts == 0 && !self.main_loop_common_suffix_completed {
                self.leading_common_suffix_completed = true;
            }
            self.main_loop_common_suffix_completed = true;
        }
        if event.event == "nmi"
            && event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
                (VWF_RENDER_SINGLE_START_PC..VWF_RENDER_SINGLE_END_PC).contains(&pc)
            })
        {
            self.vwf_nmi_observed = true;
        }
        if event.event != "frame" {
            return Ok(if common_suffix_completed {
                Some(MainLoopCompletionProof::CommonSuffixCompleted)
            } else {
                None
            });
        }
        let stage = event
            .stage
            .as_deref()
            .ok_or("Snes9x frame receipt omitted its stage")?;
        let state = HostFrameState {
            run: event.run.ok_or("Snes9x frame receipt omitted its run")?,
            pc: event
                .pc
                .ok_or("Snes9x frame receipt omitted its program counter")?
                & 0x00ff_ffff,
            x: event.x,
            main: event
                .main
                .ok_or("Snes9x frame receipt omitted Zelda main module")?,
            sub: event
                .sub
                .ok_or("Snes9x frame receipt omitted Zelda submodule")?,
            subsub: event
                .subsub
                .ok_or("Snes9x frame receipt omitted Zelda subsubmodule")?,
            frame_counter: event
                .frame_counter
                .ok_or("Snes9x frame receipt omitted Zelda frame counter")?,
            nmi_latch: event
                .nmi_latch
                .ok_or("Snes9x frame receipt omitted Zelda NMI latch")?,
        };
        match stage {
            "entry" if self.entry.replace(state).is_none() => {}
            "return" if self.returned.replace(state).is_none() => {}
            "entry" | "return" => Err(format!(
                "Snes9x host call published duplicate frame/{stage} receipts"
            ))?,
            _ => {}
        }
        Ok(None)
    }

    fn finish(
        self,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
        dialogue_message_read_position: Option<u16>,
        zelda_run_game_loop_call_active_at_entry: bool,
    ) -> Result<(), String> {
        let entry = self
            .entry
            .ok_or("Snes9x host call omitted frame/entry receipt")?;
        let returned = self
            .returned
            .ok_or("Snes9x host call omitted frame/return receipt")?;
        if entry.run != returned.run {
            return Err(format!(
                "Snes9x frame receipt run changed within one host call: {} -> {}",
                entry.run, returned.run
            ));
        }
        let main_loop_progress = match self.main_loop_starts {
            0 if zelda_run_game_loop_call_active_at_entry => {
                Some(MainLoopProgress::CallStackContinued)
            }
            0 => None,
            1 => Some(MainLoopProgress::IterationStarted),
            starts => {
                return Err(format!(
                    "Snes9x host call started ZeldaRunGameLoop {starts} times; expected zero or one"
                ));
            }
        };
        if let Some(main_loop_progress) = main_loop_progress {
            let progress_already_emitted = receipts.iter().any(|receipt| {
                *receipt == OriginalTimingSemanticReceipt::MainLoopProgress(main_loop_progress)
            });
            if !progress_already_emitted {
                receipts.push(OriginalTimingSemanticReceipt::MainLoopProgress(
                    main_loop_progress,
                ));
            }
        }
        let spotlight_call_completion = self.spotlight_call_completion();
        if std::env::var_os("ZELDA3_DEBUG_SPOTLIGHT_DECODE").is_some() {
            eprintln!(
                "[SPOTLIGHT-DECODE] entry={:?} returned={:?} main_loop_starts={} suffix_completed={} completion={:?}",
                self.entry,
                self.returned,
                self.main_loop_starts,
                self.main_loop_common_suffix_completed,
                spotlight_call_completion,
            );
        }
        if spotlight_call_completion
            == Some(SpotlightCallCompletion::RecurringCallerReturnedToMainWait)
        {
            // The backend-private entry/return PCs prove that the suspended
            // Module0F caller resumed through its Link/OAM and sprite-
            // preparation suffix before this host returned. The equivalent
            // source-owned proof is ZeldaRunGameLoop's nmi_boolean transition:
            // C clears it only after Module_MainRouting and
            // NMI_PrepareSprites return. That proof remains valid when the
            // following NMI is accepted before S9xMainLoop returns and the
            // final private PC is consequently the NMI handler rather than
            // the main wait. Export only the source-call fact; translated
            // gameplay sees neither backend PC nor latch.
            receipts
                .push(OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait);
        }
        if let Some(phase) = main_loop_interruption_for_source_state(
            returned.pc,
            Some(returned.main),
            Some(returned.sub),
            returned.x,
        )
        .filter(|phase| {
            // Module0F's ENTRY host (submodule 0 -> 1) is owned by the
            // spotlight-iteration model, which already places the whole Link
            // movement on the following host; only the recurring caller
            // publishes the mid-loop Link position boundary (route hosts
            // 179577 vs 179586).
            !(matches!(
                phase,
                MainLoopInterruption::LinkPositionAfterSubpixel { .. }
            ) && entry.main == 0x0f
                && entry.sub == 0)
        }) {
            let observed = receipts
                .iter()
                .filter_map(|receipt| match receipt {
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(observed) => Some(*observed),
                    _ => None,
                })
                .collect::<Vec<_>>();
            match observed.as_slice() {
                [] => receipts.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(phase)),
                [existing] if *existing == phase => {}
                [existing] => {
                    return Err(format!(
                        "Snes9x host call observed conflicting main-loop interruption phases: {existing:?} then {phase:?} at return"
                    ));
                }
                _ => {
                    return Err(
                        "Snes9x host call published multiple main-loop interruption receipts"
                            .to_string(),
                    );
                }
            }
        }
        // Module_PreDungeon publishes module 07/0f from the overworld entrance
        // (Module06) and from the spawn-select reload, which re-enters through
        // Module05's loader (route host 160528: the publication precedes the
        // NMI-masked song-bank upload exactly as at the Module06 entrances).
        if matches!(entry.main, 5 | 6) && returned.main == 7 && returned.sub == 15 {
            receipts.push(OriginalTimingSemanticReceipt::PreDungeonModuleReturned);
        }
        if entry.main == 14 && entry.sub == 11 && entry.subsub == 0 {
            let progress = match (returned.main, returned.sub, returned.subsub) {
                (14, 11, 0) => SaveMenuInitializationProgress::InProgress,
                (14, 11, 1) => SaveMenuInitializationProgress::Completed,
                state => {
                    return Err(format!(
                        "Snes9x save-menu initialization returned in unexpected state {state:?}"
                    ));
                }
            };
            receipts.push(OriginalTimingSemanticReceipt::SaveMenuInitializationProgress(progress));
        }
        if self.vwf_nmi_observed
            && entry.main == 14
            && returned.main == 14
            && entry.frame_counter == returned.frame_counter
        {
            let message_read_position = dialogue_message_read_position
                .ok_or("Snes9x dialogue continuation omitted its semantic message read position")?;
            receipts.push(OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                    message_read_position,
                },
            ));
        }
        if entry.main == 14 && returned.main != 14 {
            receipts.push(OriginalTimingSemanticReceipt::DialogueClosed);
        }
        let pre_overworld = match (entry.main, entry.sub, returned.main, returned.sub) {
            (8, 0, 8, 1) => Some(PreOverworldStageCompletion::PropertiesReturned),
            (8, 1, 8, 2) => Some(PreOverworldStageCompletion::OverlaysReturned),
            (8, 2, 16, 0) => Some(PreOverworldStageCompletion::ScreenBuildReturned),
            _ => None,
        };
        if let Some(stage) = pre_overworld {
            receipts.push(OriginalTimingSemanticReceipt::PreOverworldStageCompleted(
                stage,
            ));
        }
        // Module09_LoadNewMapAndGFX serves both transition lanes: submodule
        // $03 -> $04 and its mosaic twin $11 -> $12 (route host 197641).
        if entry.main == 9
            && returned.main == 9
            && matches!(entry.sub, 3 | 0x11)
            && returned.sub == entry.sub + 1
        {
            receipts.push(OriginalTimingSemanticReceipt::OverworldMapQuadrantsPublished);
        }
        if entry.main == 9 && entry.sub == 0x20 && returned.main == 9 && returned.sub == 0x21 {
            receipts.push(OriginalTimingSemanticReceipt::WorldMapOverlayReloadReturned);
        }
        if entry.main == 9 && entry.sub == 0x21 && returned.main == 9 && returned.sub == 0x22 {
            receipts.push(OriginalTimingSemanticReceipt::WorldMapAmbientMap8Returned);
        }
        if spotlight_call_completion == Some(SpotlightCallCompletion::EntryReturned) {
            receipts.push(OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned);
        }
        if spotlight_call_completion == Some(SpotlightCallCompletion::OverworldGoalCallerReturned) {
            receipts.push(OriginalTimingSemanticReceipt::OverworldSpotlightGoalCallerReturned);
        }
        if entry.main == 9
            && matches!(entry.sub, 4 | 18)
            && returned.main == 9
            && returned.sub == entry.sub.wrapping_add(1)
        {
            receipts.push(
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::ReloadReturned,
                ),
            );
        }
        Ok(())
    }
}

impl Snes9xOracleSemanticTrace {
    /// Configure the existing generic trace before the core is loaded. A
    /// caller-provided trace remains authoritative; this only adds the generic
    /// domains/ranges required by the semantic adapter.
    pub(crate) fn configure(session_dir: Option<&Path>) -> Result<Self, String> {
        let path = env::var_os(TRACE_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                session_dir
                    .map(|dir| dir.join("snes9x-semantic-live.jsonl"))
                    .unwrap_or_else(|| {
                        env::temp_dir().join(format!(
                            "zelda3-snes9x-semantic-live-{}.jsonl",
                            std::process::id()
                        ))
                    })
            });
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "create Snes9x semantic trace directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        if !path.exists() {
            File::create(&path).map_err(|error| {
                format!("create Snes9x semantic trace {}: {error}", path.display())
            })?;
        }
        unsafe {
            env::set_var(TRACE_PATH_ENV, &path);
            env::set_var(
                TRACE_EVENTS_ENV,
                append_csv(
                    env::var(TRACE_EVENTS_ENV).ok().as_deref(),
                    REQUIRED_TRACE_EVENTS,
                ),
            );
            env::set_var(
                TRACE_PCS_ENV,
                append_csv(
                    env::var(TRACE_PCS_ENV).ok().as_deref(),
                    &[
                        "028842", "05df49", "05df4d", "05eb1d", "05eb21", "068328", "0683a7",
                        "0684e2", "06a628", "06a724", "06b9cc", "06b9d0", "0799ad", "079a0b",
                        "008225", "0082c7",
                    ],
                ),
            );
            // Watch only the Zelda arrays used by these semantic domains.
            // Seeing every later Dungeon_LoadSingleSprite destination still
            // lets the adapter invalidate a YHigh candidate before NMI,
            // without tracing unrelated writes from whole WRAM pages.
            env::set_var(
                TRACE_WRAM_ENV,
                append_csv(
                    env::var(TRACE_WRAM_ENV).ok().as_deref(),
                    &[
                        "0012",
                        "001a",
                        "0020-002f",
                        "02ec",
                        "0b00-0b1d",
                        "0b6a",
                        "0b89-0b98",
                        "0ba0-0baf",
                        "0bc0-0bdf",
                        "0c4a-0c53",
                        "0d00-0d3f",
                        "0d60-0d7f",
                        "0d80-0dff",
                        "0e20-0e2f",
                        "0e40-0e4f",
                        "0e60-0e6f",
                        "0e80-0e9f",
                        "0eb0-0ebf",
                        "0f20-0f2f",
                        "0f50-0f5f",
                        "0f70-0f7f",
                        "0fba",
                        "1d00-1dff",
                        "f9c2-f9d1",
                        "fa5c-fabb",
                        "facc-faeb",
                    ],
                ),
            );
        }
        Ok(Self {
            path,
            // The trace core opens its configured output for a fresh session
            // during load, so semantic consumption always starts at byte 0.
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            seed_warmup_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        })
    }

    /// Begin the oracle-seeded warm-up (see `seed_warmup_active`).
    pub(crate) fn begin_seed_warmup(&mut self) {
        self.seed_warmup_active = true;
    }

    /// End the oracle-seeded warm-up once Rust is seeded at a clean boundary.
    pub(crate) fn end_seed_warmup(&mut self) {
        self.seed_warmup_active = false;
    }

    /// Whether an accepted NMI's publication is still pending at the last
    /// decoded run boundary (its handler completes in the next run).
    pub(crate) fn nmi_publication_pending(&self) -> bool {
        self.nmi_publication_pending
    }

    pub(crate) fn backing_path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn checkpoint(&self) -> Snes9xOracleSemanticTraceCheckpoint {
        Snes9xOracleSemanticTraceCheckpoint {
            schema: SEMANTIC_TRACE_CHECKPOINT_SCHEMA,
            cache_write_progress: self.cache_write_progress,
            normal_load_ordinal: self.normal_load_ordinal,
            pending_reset_progress: self.pending_reset_progress,
            cached_sprite_execution: self.cached_sprite_execution,
            overworld_presence_published: self.overworld_presence_published,
            overworld_sprite_activation: self.overworld_sprite_activation,
            pending_spotlight_helper_nmi: self.pending_spotlight_helper_nmi.clone(),
            item_receipt_caller: self.item_receipt_caller,
            sprite_main_execution: self.sprite_main_execution,
            zelda_run_game_loop_call_active: self.zelda_run_game_loop_call_active,
            nmi_publication_pending: self.nmi_publication_pending,
            pending_nmi_update_gate: self.pending_nmi_update_gate,
            nmi_resume_targets: self.nmi_resume_targets.clone(),
            synthesized_nmi_resume: self.synthesized_nmi_resume,
        }
    }

    pub(crate) fn restore_checkpoint(
        &mut self,
        checkpoint: Snes9xOracleSemanticTraceCheckpoint,
    ) -> Result<(), String> {
        if checkpoint.schema != SEMANTIC_TRACE_CHECKPOINT_SCHEMA {
            return Err(format!(
                "unsupported Snes9x semantic trace checkpoint schema {}",
                checkpoint.schema
            ));
        }
        if checkpoint.nmi_publication_pending != checkpoint.pending_nmi_update_gate.is_some() {
            return Err(
                "Snes9x semantic checkpoint NMI pending marker disagrees with its update gate"
                    .to_string(),
            );
        }
        if checkpoint.nmi_publication_pending && checkpoint.nmi_resume_targets.is_empty() {
            return Err(
                "Snes9x semantic checkpoint has pending NMI publication without a resume target"
                    .to_string(),
            );
        }
        for &(pc, stack) in &checkpoint.nmi_resume_targets {
            // The 65816 stack pointer is a full 16-bit register in native
            // mode. Pinned Snes9x uses Registers.S.W for native pushes/pulls
            // and constrains SH to $01 only while CheckEmulation() is true.
            // Zelda deliberately runs source work on native stacks outside
            // page one, so every value representable by this u16 is valid.
            if pc > 0x00ff_ffff {
                return Err(format!(
                    "Snes9x semantic checkpoint has invalid NMI resume target ${pc:08x}/S=${stack:04x}"
                ));
            }
        }
        if checkpoint
            .synthesized_nmi_resume
            .is_some_and(|(pc, _stack)| pc > 0x00ff_ffff)
        {
            return Err(
                "Snes9x semantic checkpoint has invalid synthesized NMI resume".to_string(),
            );
        }

        self.cache_write_progress = checkpoint.cache_write_progress;
        self.normal_load_ordinal = checkpoint.normal_load_ordinal;
        self.pending_reset_progress = checkpoint.pending_reset_progress;
        self.cached_sprite_execution = checkpoint.cached_sprite_execution;
        self.overworld_presence_published = checkpoint.overworld_presence_published;
        self.overworld_sprite_activation = checkpoint.overworld_sprite_activation;
        self.pending_spotlight_helper_nmi = checkpoint.pending_spotlight_helper_nmi;
        self.pending_spotlight_helper_nmi_acceptance_index = None;
        self.item_receipt_caller = checkpoint.item_receipt_caller;
        self.sprite_main_execution = checkpoint.sprite_main_execution;
        self.zelda_run_game_loop_call_active = checkpoint.zelda_run_game_loop_call_active;
        self.nmi_publication_pending = checkpoint.nmi_publication_pending;
        self.pending_nmi_update_gate = checkpoint.pending_nmi_update_gate;
        self.nmi_resume_targets = checkpoint.nmi_resume_targets;
        self.synthesized_nmi_resume = checkpoint.synthesized_nmi_resume;
        Ok(())
    }

    pub(crate) fn read_after_host_call(
        &mut self,
        dialogue_message_read_position: Option<u16>,
        spotlight_var4_low_at_return: Option<u8>,
        spotlight_lower_cursor_at_return: Option<u16>,
    ) -> Result<Vec<OriginalTimingSemanticReceipt>, String> {
        if !self.host_nmi_ppu_register_operands.is_empty() {
            return Err("prior Snes9x host NMI acceptance operands were not consumed".to_string());
        }
        let mut file = File::open(&self.path).map_err(|error| {
            format!(
                "open Snes9x semantic trace {}: {error}",
                self.path.display()
            )
        })?;
        file.seek(SeekFrom::Start(self.offset)).map_err(|error| {
            format!(
                "seek Snes9x semantic trace {}: {error}",
                self.path.display()
            )
        })?;
        let mut reader = BufReader::new(file);
        let mut line = String::new();
        let mut receipts = Vec::new();
        let mut host_frame = HostFrameWindow::default();
        let zelda_run_game_loop_call_active_at_entry = self.zelda_run_game_loop_call_active;
        let mut returned_event = None;
        loop {
            line.clear();
            let bytes = reader.read_line(&mut line).map_err(|error| {
                format!(
                    "read Snes9x semantic trace {}: {error}",
                    self.path.display()
                )
            })?;
            if bytes == 0 {
                break;
            }
            self.offset = self.offset.saturating_add(bytes as u64);
            let event: RawTraceEvent = serde_json::from_str(&line).map_err(|error| {
                format!(
                    "parse Snes9x semantic trace at byte {}: {error}",
                    self.offset
                )
            })?;
            if event.event == "frame" && event.stage.as_deref() == Some("return") {
                returned_event = Some(event.clone());
            }
            let main_loop_started = main_loop_started_by_event(&event);
            if main_loop_started && self.zelda_run_game_loop_call_active {
                return Err(
                    "Snes9x started ZeldaRunGameLoop before the prior call completed its common suffix"
                        .to_string(),
                );
            }
            let main_loop_common_suffix_completed = event.event == "wram-write"
                && event.address == Some(NMI_UPDATE_LATCH)
                && event.pc.map(|pc| pc & 0x00ff_ffff)
                    == Some(ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC);
            if main_loop_common_suffix_completed && !self.zelda_run_game_loop_call_active {
                return Err(
                    "Snes9x completed ZeldaRunGameLoop's common suffix without an active source call"
                        .to_string(),
                );
            }
            if main_loop_started {
                // Preserve the source write's exact position relative to NMI
                // acceptance/completion. HostFrameWindow still proves there
                // is at most one such write and supplies CallStackContinued
                // only when no iteration began.
                receipts.push(OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::IterationStarted,
                ));
            }
            if let Some(completion) = host_frame.observe(&event)? {
                if host_frame.main_loop_starts == 0
                    && !receipts.iter().any(|receipt| {
                        matches!(receipt, OriginalTimingSemanticReceipt::MainLoopProgress(_))
                    })
                {
                    // A resumed ZeldaRunGameLoop has no frame-counter write in
                    // this host. Publish its progress immediately before the
                    // exact common-suffix/return fact so later NMI phases stay
                    // on the correct side of the source boundary.
                    receipts.push(OriginalTimingSemanticReceipt::MainLoopProgress(
                        MainLoopProgress::CallStackContinued,
                    ));
                }
                let MainLoopCompletionProof::CommonSuffixCompleted = completion;
                receipts.push(OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted);
            }
            if main_loop_started {
                self.zelda_run_game_loop_call_active = true;
            }
            if main_loop_common_suffix_completed {
                self.zelda_run_game_loop_call_active = false;
            }
            self.consume_event(event, &mut receipts)?;
        }
        if let Some(returned_event) = returned_event.as_ref() {
            self.finish_pending_spotlight_helper_nmi(
                returned_event,
                spotlight_var4_low_at_return,
                spotlight_lower_cursor_at_return,
                host_frame.spotlight_call_completion(),
                &mut receipts,
            )?;
            publish_spotlight_host_return_progress(
                returned_event,
                spotlight_var4_low_at_return,
                spotlight_lower_cursor_at_return,
                &mut receipts,
            )?;
            if publish_pre_dungeon_sprite_reset_progress(
                returned_event,
                OriginalTimingBoundary::HostReturn,
                &mut receipts,
            )? {
                // The shared Sprite_DisableAll candidate belongs to the
                // enclosing Sprite_ResetAll call identified above, not to the
                // later Dungeon_ResetSprites call. Keep the domains separate.
                self.pending_reset_progress = None;
            }
        }
        // `retro_run` may return at the SCAN_KEYS boundary without accepting
        // an NMI.  A synchronous Zelda call can therefore remain suspended at
        // a source-visible write even though no `nmi` trace row closed the
        // interval.  Publish that same semantic progress at every host return;
        // the following host reconstructs continuation order from its next
        // observed write, so no CPU address or call-stack state escapes this
        // adapter.
        self.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
        self.flush_item_receipt_progress(&mut receipts);
        host_frame.finish(
            &mut receipts,
            dialogue_message_read_position,
            zelda_run_game_loop_call_active_at_entry,
        )?;
        if receipts.iter().any(|receipt| {
            matches!(
                receipt,
                OriginalTimingSemanticReceipt::PreOverworldStageCompleted(
                    PreOverworldStageCompletion::PropertiesReturned
                )
            )
        }) {
            self.overworld_presence_published = false;
            self.overworld_sprite_activation = None;
        }
        Ok(receipts)
    }

    fn finish_pending_spotlight_helper_nmi(
        &mut self,
        returned_event: &RawTraceEvent,
        spotlight_var4_low_at_return: Option<u8>,
        spotlight_lower_cursor_at_return: Option<u16>,
        spotlight_call_completion: Option<SpotlightCallCompletion>,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    ) -> Result<(), String> {
        let Some(helper_nmi) = self.pending_spotlight_helper_nmi.take() else {
            self.pending_spotlight_helper_nmi_acceptance_index = None;
            return Ok(());
        };
        let helper_acceptance_index = self.pending_spotlight_helper_nmi_acceptance_index.take();
        let returned_pc = returned_event
            .pc
            .ok_or("Snes9x helper-interrupted host return omitted PC")?
            & 0x00ff_ffff;
        // An enclosing typed source completion supersedes the helper's
        // earlier timing checkpoint even when a later NMI leaves the final
        // host PC inside the handler. Decide that ownership before using the
        // final private PC to validate a genuinely suspended helper.
        let superseded = matches!(
            (spotlight_call_completion, helper_nmi.main, helper_nmi.sub),
            (
                Some(SpotlightCallCompletion::EntryReturned),
                Some(0x0f),
                Some(0)
            ) | (
                Some(SpotlightCallCompletion::RecurringCallerReachedLinkOam),
                Some(0x0f),
                Some(1)
            ) | (
                Some(SpotlightCallCompletion::RecurringCallerReturnedToMainWait),
                Some(0x0f),
                Some(1)
            ) | (
                Some(SpotlightCallCompletion::OverworldGoalCallerReturned),
                Some(0x10),
                Some(1)
            )
        );
        let link_position_interruption_at_return =
            matches!((helper_nmi.main, helper_nmi.sub), (Some(0x0f), Some(1)))
                && matches!(
                    main_loop_interruption_for_source_state(
                        returned_pc,
                        Some(0x0f),
                Some(1),
                returned_event.x,
            ),
            Some(
                MainLoopInterruption::LinkPositionBeforeCoordinates
                    | MainLoopInterruption::LinkPositionAfterSubpixel { .. }
            )
        );
        if superseded || link_position_interruption_at_return {
            // A host return inside Module0F's Link movement proves the
            // recurring caller's table build completed after the helper NMI;
            // the movement interruption receipt owns this host (route host
            // 179586).
            return Ok(());
        }
        // The helper NMI's own handler completed inside this host when a
        // completion follows its acceptance: the interrupted table build
        // resumed here and the recurring caller returned to the main wait
        // (route host 182702, the recurring Module10 opening). The checkpoint
        // then belongs directly after that acceptance, ahead of the handler
        // completion, exactly as a projection-copy checkpoint accepted at the
        // same position is published. A host that instead ends inside a later
        // NMI's handler keeps the established trailing position (route hosts
        // 37587 and 182709).
        let resumed_acceptance = helper_acceptance_index.filter(|&index| {
            receipts[index + 1..].iter().any(|receipt| {
                matches!(receipt, OriginalTimingSemanticReceipt::NmiHandlerCompleted)
            })
        });
        if returned_pc != NMI_HANDLER_ENTRY_PC {
            if let Some(acceptance) = resumed_acceptance.filter(|_| {
                helper_nmi.spotlight_var4_low.is_some()
                    && helper_nmi.spotlight_lower_cursor.is_some()
            }) {
                // The event binds its own scratch, so the checkpoint decodes
                // without the host-return scratch.
                let progress = spotlight_table_build_progress(&helper_nmi, None, None)?
                    .ok_or("Snes9x spotlight helper NMI did not decode to table progress")?;
                receipts.insert(
                    acceptance + 1,
                    OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                        SpotlightTableBuildProgressReceipt {
                            progress,
                            boundary: OriginalTimingBoundary::NmiAccepted,
                        },
                    ),
                );
                return Ok(());
            }
            // `Dungeon_PrepExitWithSpotlight` increments submodule 0 -> 1
            // only after `IrisSpotlight_close` returns. On recurring calls,
            // ZeldaRunGameLoop clears its NMI latch only after
            // Module_MainRouting and NMI_PrepareSprites return. When either
            // enclosing source completion is published in this same host
            // call, the intermediate table checkpoint has been superseded by
            // the stronger receipt emitted by `HostFrameWindow::finish`.
            return Err(format!(
                "Snes9x spotlight helper NMI did not return at the source NMI entry: ${returned_pc:06x}"
            ));
        }
        let progress = spotlight_table_build_progress(
            &helper_nmi,
            spotlight_var4_low_at_return,
            spotlight_lower_cursor_at_return,
        )?
        .ok_or("Snes9x spotlight helper NMI did not decode to table progress")?;
        receipts.push(OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
            SpotlightTableBuildProgressReceipt {
                progress,
                boundary: OriginalTimingBoundary::NmiAccepted,
            },
        ));
        Ok(())
    }

    /// Remove the emulator-private raw trace after its typed receipt ledger is
    /// complete. Callers deliberately invoke this only on success so a failed
    /// capture retains the narrow source evidence needed for diagnosis.
    pub(crate) fn remove_backing_file(&self) -> Result<(), String> {
        fs::remove_file(&self.path).map_err(|error| {
            format!(
                "remove completed Snes9x semantic trace {}: {error}",
                self.path.display()
            )
        })
    }

    pub(crate) fn take_host_nmi_ppu_register_operands(&mut self) -> Vec<NmiPpuRegisterOperands> {
        std::mem::take(&mut self.host_nmi_ppu_register_operands)
    }

    fn flush_reset_progress(
        &mut self,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
        boundary: OriginalTimingBoundary,
    ) {
        if let Some(progress) = self.pending_reset_progress.take() {
            receipts.push(OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                DungeonResetSpritesProgressReceipt { progress, boundary },
            ));
        }
    }

    fn flush_host_boundary_progress(
        &mut self,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
        boundary: OriginalTimingBoundary,
    ) {
        self.flush_reset_progress(receipts, boundary);
        if let Some(progress) = self.cached_sprite_execution.take() {
            receipts.push(
                OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
                    CachedSpriteExecutionProgressReceipt {
                        progress: progress.receipt(),
                        boundary,
                    },
                ),
            );
        }
        if boundary == OriginalTimingBoundary::HostReturn {
            // `retro_run` can stop at SCAN_KEYS while the interrupted source
            // call is still inside Sprite_Main, without accepting an NMI in
            // that host interval. Preserve the furthest returned C statement
            // as the one host-boundary fact. Earlier same-host resume facts
            // are superseded by this later checkpoint; NMI lifecycle receipts
            // remain independently ordered in the ledger.
            receipts.retain(|receipt| {
                !matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::SpriteMainProgressed(_)
                )
            });
            if let Some(execution) = self.sprite_main_execution {
                receipts.push(OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    execution.progress(),
                ));
            }
        }
    }

    fn flush_item_receipt_progress(&self, receipts: &mut Vec<OriginalTimingSemanticReceipt>) {
        if let Some(caller) = self.item_receipt_caller {
            receipts.push(OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller,
                    progress: SourceCallProgress::Suspended,
                },
            ));
        }
    }

    fn consume_event(
        &mut self,
        event: RawTraceEvent,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    ) -> Result<(), String> {
        if event.event == "nmi" && self.nmi_publication_pending {
            return Err(
                "Snes9x accepted a second NMI before the first published its updates".to_string(),
            );
        }
        let resumed_nmi_context = self.reconcile_nmi_context_resume(&event)?;
        if resumed_nmi_context {
            retire_resumed_main_loop_interruption(
                receipts,
                self.sprite_main_execution
                    .map(|execution| execution.progress()),
            )?;
        }
        match event.event.as_str() {
            "pc" => {
                let pc = event.pc.ok_or("Snes9x PC receipt omitted PC")? & 0x00ff_ffff;
                match pc {
                    SPRITE_MAIN_ENTRY_PC => {
                        // A fresh entry is also a source-level proof that any
                        // prior call returned. Different module callers have
                        // different private return PCs, so the generic adapter
                        // closes the old tracker here instead of enumerating
                        // every call site.
                        if let Some(caller) = self.item_receipt_caller.take() {
                            receipts.push(
                                OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                                    ItemReceiptGraphicsProgressReceipt {
                                        caller,
                                        progress: SourceCallProgress::Returned,
                                    },
                                ),
                            );
                        }
                        if self.sprite_main_execution.take().is_some() {
                            receipts.push(OriginalTimingSemanticReceipt::SpriteMainReturned);
                        }
                        self.sprite_main_execution = Some(SpriteMainExecutionTracker::default());
                    }
                    SPRITE_EXECUTE_SINGLE_ENTRY_PC => {
                        // ExecuteCachedSprites calls this leaf directly after
                        // Sprite_Main's descending loop has returned. Those
                        // calls belong to the separate cached-sprite tracker
                        // and must not reopen or corrupt the Sprite_Main
                        // cursor. Only an active Sprite_Main entry owns this
                        // receipt domain.
                        if let Some(execution) = self.sprite_main_execution.as_mut() {
                            let slot = u8::try_from(
                                event
                                    .x
                                    .ok_or("Snes9x Sprite_ExecuteSingle receipt omitted slot X")?,
                            )
                            .map_err(|_| "Snes9x Sprite_ExecuteSingle slot exceeded one byte")?;
                            if slot >= 16 {
                                return Err(format!(
                                    "Snes9x Sprite_ExecuteSingle used invalid slot {slot}"
                                ));
                            }
                            execution.current_slot = Some(slot);
                            execution.cucco_subtype_increments = None;
                            execution.cucco_animation_slot = None;
                            execution.cucco_flee_movement = None;
                            execution.active_cucco_movement = None;
                            execution.active_cucco_x_publications = 0;
                            execution.active_cucco_y_subpixel = None;
                            execution.cucco_helper_ordinal = 0;
                            execution.big_key_drop_graphics_slot = None;
                        }
                    }
                    SPRITE_SLOT_RETURN_PC => {
                        let execution = self
                            .sprite_main_execution
                            .as_mut()
                            .ok_or("Snes9x returned one sprite slot outside Sprite_Main")?;
                        let slot = execution
                            .current_slot
                            .ok_or("Snes9x returned one Sprite_Main slot before entering it")?;
                        execution.last_completed_slot = Some(slot);
                        execution.cucco_subtype_increments = None;
                        execution.cucco_animation_slot = None;
                        execution.cucco_flee_movement = None;
                        execution.active_cucco_movement = None;
                        execution.active_cucco_x_publications = 0;
                        execution.active_cucco_y_subpixel = None;
                        execution.cucco_helper_ordinal = 0;
                        execution.big_key_drop_graphics_slot = None;
                        // Slot zero is the final iteration of the descending C
                        // loop. No caller-specific return address is needed to
                        // prove that later NMIs are outside Sprite_Main.
                        if slot == 0 {
                            self.sprite_main_execution = None;
                            receipts.push(OriginalTimingSemanticReceipt::SpriteMainReturned);
                        }
                    }
                    SPRITE_MAIN_RETURN_PC => {
                        // Slot zero closes the descending loop as soon as its
                        // source call returns. The later common caller-return
                        // marker is therefore idempotent for a complete loop,
                        // while still closing early-return paths which never
                        // entered all sixteen slots.
                        if self.sprite_main_execution.take().is_some() {
                            receipts.push(OriginalTimingSemanticReceipt::SpriteMainReturned);
                        }
                    }
                    ACTIVE_CUCCO_MOVEMENT_CALL_PC | CUCCO_FLEE_SUBTYPE_HELPER_CALL_PC
                        if self.sprite_main_execution.is_none() && event.main == Some(0x1a) =>
                    {
                        // Module1A credits scenes call `SpriteActive_Main`
                        // directly (route host 1573154); those Cucco helpers
                        // publish no Sprite_Main receipts.
                    }
                    ACTIVE_CUCCO_MOVEMENT_CALL_PC => {
                        let execution = self
                            .sprite_main_execution
                            .as_mut()
                            .ok_or("Snes9x entered active Cucco movement outside Sprite_Main")?;
                        let slot = execution
                            .current_slot
                            .ok_or("Snes9x entered active Cucco movement before entering a slot")?;
                        if event.x != Some(u16::from(slot)) {
                            return Err(format!(
                                "Snes9x active Cucco movement disagreed on slot {slot}: x={:?}",
                                event.x,
                            ));
                        }
                        if execution.cucco_subtype_increments.is_some()
                            || execution.cucco_flee_movement.is_some()
                            || execution.active_cucco_movement.is_some()
                            || execution.active_cucco_x_publications != 0
                            || execution.active_cucco_y_subpixel.is_some()
                        {
                            return Err(
                                "Snes9x restarted active Cucco movement with unfinished work"
                                    .to_string(),
                            );
                        }
                        if execution.cucco_animation_slot.take().is_some() {
                            execution.cucco_helper_ordinal = execution
                                .cucco_helper_ordinal
                                .checked_add(1)
                                .ok_or("Snes9x Cucco helper ordinal overflowed")?;
                        }
                        execution.active_cucco_movement =
                            Some((slot, execution.cucco_helper_ordinal));
                        execution.active_cucco_x_publications = 0;
                    }
                    CUCCO_FLEE_SUBTYPE_HELPER_CALL_PC => {
                        let execution = self
                            .sprite_main_execution
                            .as_mut()
                            .ok_or("Snes9x completed Cucco flee movement outside Sprite_Main")?;
                        let slot = execution
                            .current_slot
                            .ok_or("Snes9x completed Cucco flee movement before entering a slot")?;
                        if event.x != Some(u16::from(slot)) {
                            return Err(format!(
                                "Snes9x Cucco flee movement disagreed on slot {slot}: x={:?}",
                                event.x,
                            ));
                        }
                        if execution.cucco_subtype_increments.is_some()
                            || execution.cucco_flee_movement.is_some()
                        {
                            return Err(
                                "Snes9x entered Cucco flee helper with an unfinished helper"
                                    .to_string(),
                            );
                        }
                        if execution.cucco_animation_slot.take().is_some() {
                            execution.cucco_helper_ordinal = execution
                                .cucco_helper_ordinal
                                .checked_add(1)
                                .ok_or("Snes9x Cucco helper ordinal overflowed")?;
                        }
                        execution.active_cucco_y_subpixel = None;
                        execution.active_cucco_x_publications = 0;
                        execution.cucco_flee_movement =
                            Some((slot, execution.cucco_helper_ordinal));
                    }
                    _ => {}
                }
                match pc {
                    pc if NMI_HANDLER_COMPLETE_PCS.contains(&pc) => {
                        if !self.nmi_publication_pending {
                            if !self.seed_warmup_active {
                                return Err(
                                    "Snes9x reached NMI publication completion without an accepted NMI"
                                        .to_string(),
                                );
                            }
                            // Seed warm-up: the NMI was accepted before the
                            // seeded oracle state was captured; its completion
                            // belongs to no host this decoder saw. Drop it.
                        } else {
                            let update_gate = self.pending_nmi_update_gate.ok_or(
                                "Snes9x NMI completion lost its accepted update-gate disposition",
                            )?;
                            let joypad_publication = if update_gate == NmiUpdateGate::Open {
                                Some(event.joypad_publication()?.ok_or(
                                    "open Snes9x NMI completion omitted Zelda joypad publication",
                                )?)
                            } else {
                                None
                            };
                            self.pending_nmi_update_gate = None;
                            self.nmi_publication_pending = false;
                            receipts.push(OriginalTimingSemanticReceipt::NmiHandlerCompleted);
                            if let Some(publication) = joypad_publication {
                                receipts.push(OriginalTimingSemanticReceipt::JoypadPublication(
                                    publication,
                                ));
                            }
                        }
                    }
                    BOTTLE_VENDOR_ITEM_RECEIPT_CALL_PC | SICK_KID_ITEM_RECEIPT_CALL_PC => {
                        let slot = u8::try_from(
                            event
                                .x
                                .ok_or("Snes9x Sprite_Main item call omitted sprite slot")?,
                        )
                        .map_err(|_| "Snes9x Sprite_Main sprite slot exceeded one byte")?;
                        if slot >= 16 {
                            return Err(format!(
                                "Snes9x Sprite_Main item call used invalid sprite slot {slot}"
                            ));
                        }
                        let caller = ItemReceiptGraphicsCaller::SpriteMain { slot };
                        if let Some(active) = self.item_receipt_caller.replace(caller) {
                            return Err(format!(
                                "Snes9x entered a second Sprite_Main item receipt in slot {slot} while {active:?} remained suspended"
                            ));
                        }
                    }
                    BOTTLE_VENDOR_ITEM_RECEIPT_RETURN_PC | SICK_KID_ITEM_RECEIPT_RETURN_PC => {
                        let caller = self.item_receipt_caller.take().ok_or(
                            "Snes9x returned from a Sprite_Main item receipt without an active caller",
                        )?;
                        if !matches!(caller, ItemReceiptGraphicsCaller::SpriteMain { .. }) {
                            return Err(format!(
                                "Snes9x returned from a Sprite_Main item receipt while {caller:?} remained active"
                            ));
                        }
                        receipts.push(OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                            ItemReceiptGraphicsProgressReceipt {
                                caller,
                                progress: SourceCallProgress::Returned,
                            },
                        ));
                    }
                    LINK_RECEIVE_ITEM_ENTRY_PC => {
                        // Caller-specific item paths install their tracker at
                        // the outer C call site. Otherwise a direct item
                        // pickup inside Sprite_Main owns the same synchronous
                        // graphics boundary and the active source slot is its
                        // semantic caller identity.
                        if self.item_receipt_caller.is_none() {
                            if let Some(execution) = self.sprite_main_execution {
                                if let Some(slot) = execution.current_slot {
                                    self.item_receipt_caller =
                                        Some(ItemReceiptGraphicsCaller::SpriteMainDirect { slot });
                                } else if execution.last_completed_slot.is_none() {
                                    // Sprite_Main's prefix (Ancilla_Main) ran
                                    // the falling milestone item's receipt
                                    // before the slot loop began (route host
                                    // 1142850); X is the ancilla slot.
                                    let slot = u8::try_from(
                                        event.x.ok_or(
                                            "Snes9x ancilla item receipt omitted its slot X",
                                        )?,
                                    )
                                    .map_err(|_| {
                                        "Snes9x ancilla item receipt slot exceeded one byte"
                                    })?;
                                    self.item_receipt_caller =
                                        Some(ItemReceiptGraphicsCaller::SpriteMainAncilla { slot });
                                }
                            }
                        }
                    }
                    LINK_RECEIVE_ITEM_GRAPHICS_RETURN_PC => {
                        if matches!(
                            self.item_receipt_caller,
                            Some(
                                ItemReceiptGraphicsCaller::SpriteMainDirect { .. }
                                    | ItemReceiptGraphicsCaller::SpriteMainAncilla { .. }
                            )
                        ) {
                            let caller = self
                                .item_receipt_caller
                                .take()
                                .expect("direct Sprite_Main item receipt was matched above");
                            receipts.push(
                                OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                                    ItemReceiptGraphicsProgressReceipt {
                                        caller,
                                        progress: SourceCallProgress::Returned,
                                    },
                                ),
                            );
                        }
                    }
                    UNCLE_PASSAGE_ITEM_RECEIPT_CALL_PC => {
                        let slot = u8::try_from(
                            event
                                .x
                                .ok_or("Snes9x Uncle item call omitted sprite slot X")?,
                        )
                        .map_err(|_| "Snes9x Uncle item-call slot exceeded one byte")?;
                        if slot >= 16 {
                            return Err(format!(
                                "Snes9x Uncle item call used invalid sprite slot {slot}"
                            ));
                        }
                        let caller = ItemReceiptGraphicsCaller::UnclePassage { slot };
                        if let Some(active) = self.item_receipt_caller.replace(caller) {
                            return Err(format!(
                                "Snes9x entered Uncle item receipt in slot {slot} while {active:?} remained suspended"
                            ));
                        }
                    }
                    UNCLE_PASSAGE_ITEM_RECEIPT_RETURN_PC => {
                        let caller = self.item_receipt_caller.take().ok_or(
                            "Snes9x returned from Uncle item receipt without an active caller",
                        )?;
                        if !matches!(caller, ItemReceiptGraphicsCaller::UnclePassage { .. }) {
                            return Err(format!(
                                "Snes9x returned from Uncle item receipt while {caller:?} remained active"
                            ));
                        }
                        receipts.push(OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                            ItemReceiptGraphicsProgressReceipt {
                                caller,
                                progress: SourceCallProgress::Returned,
                            },
                        ));
                    }
                    _ => {}
                }
            }
            "wram-write" => {
                let pc = event.pc.ok_or("Snes9x WRAM write omitted PC")? & 0x00ff_ffff;
                let address = event.address.ok_or("Snes9x WRAM write omitted address")?;
                if let Some(execution) = self.sprite_main_execution.as_mut() {
                    if let Some((slot, helper_ordinal)) = execution.active_cucco_movement {
                        let expected_x_address = match execution.active_cucco_x_publications {
                            0 => Some(SPRITE_X_SUBPIXEL_BASE + u16::from(slot)),
                            1 => Some(SPRITE_X_LOW_BASE + u16::from(slot)),
                            2 => Some(SPRITE_X_HIGH_BASE + u16::from(slot)),
                            3 => None,
                            count => {
                                return Err(format!(
                                    "Snes9x active Cucco published invalid X field count {count}",
                                ));
                            }
                        };
                        if expected_x_address == Some(address) {
                            execution.active_cucco_x_publications += 1;
                        } else if [
                            SPRITE_X_SUBPIXEL_BASE + u16::from(slot),
                            SPRITE_X_LOW_BASE + u16::from(slot),
                            SPRITE_X_HIGH_BASE + u16::from(slot),
                        ]
                        .contains(&address)
                        {
                            return Err(format!(
                                "Snes9x active Cucco X publications were out of source order at ${address:04x}",
                            ));
                        }
                        if address == SPRITE_Y_SUBPIXEL_BASE + u16::from(slot) {
                            if execution.active_cucco_x_publications != 3 {
                                return Err(
                                    "Snes9x published active Cucco Y before Sprite_MoveX returned"
                                        .to_string(),
                                );
                            }
                            if execution.active_cucco_y_subpixel.is_some() {
                                return Err(
                                    "Snes9x published active Cucco Y subpixel twice".to_string()
                                );
                            }
                            execution.active_cucco_y_subpixel = Some((slot, helper_ordinal));
                        }
                    }
                }
                let credits_direct_sprite_call =
                    self.sprite_main_execution.is_none() && event.main == Some(0x1a);
                if let Some(increment_index) = CUCCO_SUBTYPE_INCREMENT_PUBLICATION_PCS
                    .iter()
                    .position(|&increment_pc| pc == increment_pc)
                    .filter(|_| !credits_direct_sprite_call)
                {
                    let execution = self
                        .sprite_main_execution
                        .as_mut()
                        .ok_or("Snes9x published a Cucco subtype increment outside Sprite_Main")?;
                    let slot = execution.current_slot.ok_or(
                        "Snes9x published a Cucco subtype increment before entering a slot",
                    )?;
                    if event.x != Some(u16::from(slot))
                        || address != SPRITE_SUBTYPE2_BASE + u16::from(slot)
                    {
                        return Err(format!(
                            "Snes9x Cucco subtype publication disagreed on slot {slot}: x={:?}, address=${address:04x}",
                            event.x,
                        ));
                    }
                    if increment_index == 0 {
                        if execution.cucco_subtype_increments.is_some() {
                            return Err(
                                "Snes9x restarted a Cucco helper before publishing graphics"
                                    .to_string(),
                            );
                        }
                        if execution.cucco_animation_slot.take().is_some() {
                            execution.cucco_helper_ordinal = execution
                                .cucco_helper_ordinal
                                .checked_add(1)
                                .ok_or("Snes9x Cucco helper ordinal overflowed")?;
                        }
                        execution.cucco_flee_movement = None;
                        execution.active_cucco_movement = None;
                        execution.active_cucco_x_publications = 0;
                        execution.active_cucco_y_subpixel = None;
                    }
                    let completed = u8::try_from(increment_index + 1)
                        .expect("the Cucco increment table has five entries");
                    execution.cucco_subtype_increments =
                        Some((slot, execution.cucco_helper_ordinal, completed));
                }
                if pc == CUCCO_ANIMATION_PUBLICATION_PC && !credits_direct_sprite_call {
                    let execution = self
                        .sprite_main_execution
                        .as_mut()
                        .ok_or("Snes9x published Cucco animation outside Sprite_Main")?;
                    let slot = execution
                        .current_slot
                        .ok_or("Snes9x published Cucco animation before entering a slot")?;
                    if event.x != Some(u16::from(slot))
                        || address != SPRITE_GRAPHICS_BASE + u16::from(slot)
                    {
                        return Err(format!(
                            "Snes9x Cucco animation publication disagreed on slot {slot}: x={:?}, address=${address:04x}",
                            event.x,
                        ));
                    }
                    execution.cucco_subtype_increments = None;
                    execution.cucco_flee_movement = None;
                    execution.active_cucco_movement = None;
                    execution.active_cucco_x_publications = 0;
                    execution.active_cucco_y_subpixel = None;
                    execution.cucco_animation_slot = Some((slot, execution.cucco_helper_ordinal));
                }
                if pc == BIG_KEY_DROP_TYPE_PUBLICATION_PC {
                    let value = event
                        .value
                        .ok_or("Snes9x enemy-drop type publication omitted value")?;
                    // This is the shared `sprite_type[k] = item` statement.
                    // Only `$e5` takes the following big-key graphics branch;
                    // ordinary prize/key drops remain outside this domain.
                    if value == BIG_KEY_DROP_SPRITE_TYPE {
                        let execution = self
                            .sprite_main_execution
                            .as_mut()
                            .ok_or("Snes9x entered big-key graphics outside Sprite_Main")?;
                        let slot = execution
                            .current_slot
                            .ok_or("Snes9x entered big-key graphics before entering a slot")?;
                        if event.x != Some(u16::from(slot))
                            || address != SPRITE_TYPE_BASE + u16::from(slot)
                        {
                            return Err(format!(
                                "Snes9x big-key type publication disagreed on slot {slot}: x={:?}, address=${address:04x}, value=${value:02x}",
                                event.x,
                            ));
                        }
                        execution.big_key_drop_graphics_slot = Some(slot);
                    }
                }
                self.observe_overworld_sprite_publication(&event, pc, address, receipts)?;
                let disable_progress = sprite_disable_progress(pc, address, event.value)?;
                // `SpritesDisabled` is a candidate for a host boundary at the
                // final Sprite_DisableAll write, not a durable description of
                // the rest of Dungeon_ResetSprites.  Once any later source
                // write is observed, execution has advanced beyond that exact
                // statement.  Drop the candidate fail-closed; a more precise
                // cache/load receipt below may replace it.
                if matches!(
                    self.pending_reset_progress,
                    Some(DungeonResetSpritesCpuProgress::SpritesDisabled)
                ) && !(pc == SPRITE_DISABLE_ALL_FINAL_GARNISH_PC
                    && event.x == Some(0)
                    && address == GARNISH_TYPE_SLOT_ZERO)
                {
                    self.pending_reset_progress = None;
                }
                if matches!(
                    self.pending_reset_progress,
                    Some(DungeonResetSpritesCpuProgress::Disable(_))
                ) && disable_progress.is_none()
                {
                    self.pending_reset_progress = None;
                }
                let cached_sprite_write = (UNCACHE_SPRITE_START_PC..UNCACHE_SPRITE_END_PC)
                    .contains(&pc)
                    .then(|| cached_sprite_live_field(address))
                    .flatten();
                if let Some((field_index, slot)) = cached_sprite_write {
                    if let Some(progress) = self.cached_sprite_execution.as_mut() {
                        if progress.observe_write(pc, slot, field_index)? {
                            self.cached_sprite_execution = None;
                        }
                    } else {
                        self.cached_sprite_execution =
                            Some(CachedSpriteExecutionTracker::from_observed_write(
                                pc,
                                slot,
                                field_index,
                            ));
                    }
                } else if let Some(progress) = disable_progress {
                    self.cache_write_progress = None;
                    self.normal_load_ordinal = None;
                    self.pending_reset_progress =
                        Some(DungeonResetSpritesCpuProgress::Disable(progress));
                } else if pc == SPRITE_DISABLE_ALL_FINAL_GARNISH_PC
                    && event.x == Some(0)
                    && address == GARNISH_TYPE_SLOT_ZERO
                {
                    self.pending_reset_progress =
                        Some(DungeonResetSpritesCpuProgress::SpritesDisabled);
                } else if (DUNGEON_CACHE_TRANS_SPRITES_START_PC..DUNGEON_CACHE_TRANS_SPRITES_END_PC)
                    .contains(&pc)
                    && CACHE_FIELD_WRITES
                        .iter()
                        .any(|&(_, base)| (base..base + 16).contains(&address))
                {
                    let slot = u8::try_from(
                        event
                            .x
                            .ok_or("Snes9x Dungeon_CacheTransSprites write omitted X")?,
                    )
                    .map_err(|_| "Snes9x Dungeon_CacheTransSprites X exceeded one byte")?;
                    if slot >= 16 {
                        return Err(format!(
                            "Snes9x Dungeon_CacheTransSprites slot {slot} is outside 0..16"
                        ));
                    }
                    let progress = match self.cache_write_progress {
                        Some(progress) if progress.slot == slot => progress,
                        Some(progress) if slot < progress.slot => CacheWriteProgress {
                            slot,
                            next_field_index: 0,
                        },
                        // A completed call may have no later traced reset
                        // write (Sprite_DisableAll stores only active slots).
                        // The next source call is nevertheless unambiguous:
                        // its descending C loop begins with slot 15's
                        // StateClear publication.
                        Some(_) if slot == 15 && address == 0x1d0f => CacheWriteProgress {
                            slot,
                            next_field_index: 0,
                        },
                        Some(progress) => {
                            return Err(format!(
                                "Snes9x Dungeon_CacheTransSprites slot order advanced from {} to {slot}",
                                progress.slot
                            ));
                        }
                        None => CacheWriteProgress {
                            slot,
                            next_field_index: 0,
                        },
                    };
                    let &(field, base) = CACHE_FIELD_WRITES
                        .get(progress.next_field_index)
                        .ok_or("Snes9x Dungeon_CacheTransSprites wrote past the final field")?;
                    let expected_address = base + u16::from(slot);
                    if address != expected_address {
                        return Err(format!(
                            "Snes9x Dungeon_CacheTransSprites field {field:?} for slot {slot} expected ${expected_address:04x}, observed ${address:04x}"
                        ));
                    }
                    self.cache_write_progress = Some(CacheWriteProgress {
                        slot,
                        next_field_index: progress.next_field_index + 1,
                    });
                    self.pending_reset_progress =
                        Some(DungeonResetSpritesCpuProgress::Cache { slot, field });
                } else if pc == DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC
                    && (SPRITE_STATE_BASE..SPRITE_STATE_BASE + 16).contains(&address)
                {
                    let slot = (address - SPRITE_STATE_BASE) as u8;
                    if event.x != Some(u16::from(slot)) {
                        return Err(format!(
                            "Snes9x Dungeon_LoadSingleSprite state write disagrees on slot: x={:?}, address=${address:04x}",
                            event.x
                        ));
                    }
                    self.normal_load_ordinal = Some(
                        self.normal_load_ordinal
                            .map(|ordinal| ordinal.saturating_add(1))
                            .unwrap_or(0),
                    );
                    self.pending_reset_progress = None;
                } else if (DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC..DUNGEON_LOAD_SINGLE_SPRITE_END_PC)
                    .contains(&pc)
                {
                    self.pending_reset_progress = None;
                    if pc == DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC
                        && (SPRITE_Y_HIGH_BASE..SPRITE_Y_HIGH_BASE + 16).contains(&address)
                    {
                        let slot = (address - SPRITE_Y_HIGH_BASE) as u8;
                        if event.x != Some(u16::from(slot)) {
                            return Err(format!(
                                "Snes9x Dungeon_LoadSingleSprite YHigh write disagrees on slot: x={:?}, address=${address:04x}",
                                event.x
                            ));
                        }
                        let normal_load_ordinal = self.normal_load_ordinal.ok_or(
                            "Snes9x observed Dungeon_LoadSingleSprite YHigh before record state",
                        )?;
                        self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::Load(
                            DungeonLoadSpritesCpuProgress {
                                normal_load_ordinal,
                                slot,
                                checkpoint: DungeonSpriteLoadCheckpoint::YHigh,
                            },
                        ));
                    }
                }
            }
            "nmi" => {
                let ppu_register_operands = event.nmi_ppu_register_operands()?;
                let target = nmi_resume_target(&event)?;
                let update_gate = match event
                    .nmi_latch
                    .ok_or("Snes9x NMI receipt omitted Zelda's software update latch")?
                {
                    0 => NmiUpdateGate::Open,
                    _ => NmiUpdateGate::LatchHeld,
                };
                // Validate the source-stage cursor before changing any
                // cross-host NMI ownership or publishing partial receipts.
                let main_loop_interruption = main_loop_interruption_for_event(&event)?;
                if matches!(
                    main_loop_interruption,
                    Some(MainLoopInterruption::SpritePreparationExtendedOamPacking { .. })
                ) && update_gate != NmiUpdateGate::LatchHeld
                {
                    return Err(
                        "Snes9x extended-OAM packing interruption observed an open Zelda NMI latch"
                            .to_string(),
                    );
                }
                self.nmi_publication_pending = true;
                self.pending_nmi_update_gate = Some(update_gate);
                self.nmi_resume_targets.push(target);
                self.host_nmi_ppu_register_operands
                    .push(ppu_register_operands);
                if publish_pre_dungeon_sprite_reset_progress(
                    &event,
                    OriginalTimingBoundary::NmiAccepted,
                    receipts,
                )? {
                    // `Sprite_DisableAll` is shared by `Sprite_ResetAll` and
                    // `Dungeon_ResetSprites`. The interrupted PC and the
                    // innermost source return address prove this execution is
                    // the former, so the generic reset candidate must not
                    // escape into the wrong semantic domain below.
                    self.pending_reset_progress = None;
                }
                if event.main == Some(8)
                    && event.sub == Some(0)
                    && event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
                        (OVERWORLD_SPRITE_SCAN_START_PC..OVERWORLD_SPRITE_SCAN_END_PC).contains(&pc)
                    })
                {
                    self.publish_overworld_presence(receipts);
                }
                self.flush_host_boundary_progress(receipts, OriginalTimingBoundary::NmiAccepted);
                receipts.push(OriginalTimingSemanticReceipt::NmiAccepted(update_gate));
                if let Some(execution) = self.sprite_main_execution {
                    let interruption = match self.item_receipt_caller {
                        Some(
                            ItemReceiptGraphicsCaller::SpriteMain { slot }
                            | ItemReceiptGraphicsCaller::UnclePassage { slot },
                        ) => {
                            if execution.current_slot != Some(slot) {
                                return Err(format!(
                                    "Snes9x item-receipt caller slot {slot} disagreed with active Sprite_Main slot {:?}",
                                    execution.current_slot,
                                ));
                            }
                            MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(slot)
                        }
                        _ => execution.interruption(),
                    };
                    receipts.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(
                        interruption,
                    ));
                }
                let progress_requires_return_scratch = spotlight_receipt_domain(&event)
                    && event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
                        pc == IRIS_SPOTLIGHT_ITERATION_VALUE_STORE_PC
                            || pc == IRIS_SPOTLIGHT_NEXT_ITERATION_PC
                            || (IRIS_SPOTLIGHT_CIRCLE_VALUE_START_PC
                                ..IRIS_SPOTLIGHT_CIRCLE_VALUE_END_PC)
                                .contains(&pc)
                    });
                if progress_requires_return_scratch {
                    if self
                        .pending_spotlight_helper_nmi
                        .replace(event.clone())
                        .is_some()
                    {
                        return Err(
                            "Snes9x host call accepted multiple NMIs inside the spotlight helper"
                                .to_string(),
                        );
                    }
                    self.pending_spotlight_helper_nmi_acceptance_index =
                        receipts.iter().rposition(|receipt| {
                            matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted(_))
                        });
                } else if let Some(progress) = spotlight_table_build_progress(&event, None, None)? {
                    receipts.push(OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                        SpotlightTableBuildProgressReceipt {
                            progress,
                            boundary: OriginalTimingBoundary::NmiAccepted,
                        },
                    ));
                }
                if let Some(phase) = main_loop_interruption {
                    receipts.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(phase));
                }
            }
            "nmi-resume" => {
                // Reconciliation above validates the private stack-qualified
                // context return. It emits no gameplay receipt: Zelda may
                // publish the NMI and switch to the main stack long before the
                // interrupted context resumes.
            }
            _ => {}
        }
        Ok(())
    }

    fn reconcile_nmi_context_resume(&mut self, event: &RawTraceEvent) -> Result<bool, String> {
        let is_direct_resume = event.event == "nmi-resume";
        let can_observe_resume = is_direct_resume
            || event.event == "nmi"
            || event.event == "pc"
            || event.event == "frame";
        if !can_observe_resume {
            return Ok(false);
        }

        if event.event == "nmi" {
            // Zelda3TraceNmi replaces the maintained core's one pending
            // direct-resume marker. The host stack remains authoritative for
            // older nested targets, but no later direct row can acknowledge a
            // completion synthesized before this acceptance.
            self.synthesized_nmi_resume = None;
        }

        let position = match (event.pc, event.s) {
            (Some(pc), Some(s)) => (pc & 0x00ff_ffff, s),
            _ if self.nmi_resume_targets.is_empty() && !is_direct_resume => return Ok(false),
            _ => {
                return Err(format!(
                    "Snes9x {} receipt omitted the PC/stack required for NMI context-resume ownership",
                    event.event
                ));
            }
        };

        if is_direct_resume && self.synthesized_nmi_resume == Some(position) {
            self.synthesized_nmi_resume = None;
            return Ok(false);
        }

        if self.nmi_resume_targets.last().copied() == Some(position) {
            self.nmi_resume_targets.pop();
            if event.event == "frame" {
                // S9xMainLoop can return on SCAN_KEYS before
                // Zelda3TraceInstruction emits nmi-resume. Remember the exact
                // position so that later direct marker is consumed once.
                self.synthesized_nmi_resume = Some(position);
            }
            return Ok(true);
        }

        if is_direct_resume {
            return Err(format!(
                    "Snes9x NMI context resume at ${:06x}/S=${:04x} did not match the active target {:?}",
                position.0,
                position.1,
                self.nmi_resume_targets.last()
            ));
        }
        Ok(false)
    }

    fn publish_overworld_presence(&mut self, receipts: &mut Vec<OriginalTimingSemanticReceipt>) {
        if !self.overworld_presence_published {
            self.overworld_presence_published = true;
            receipts.push(
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::PresencePublished,
                ),
            );
        }
    }

    fn observe_overworld_sprite_publication(
        &mut self,
        event: &RawTraceEvent,
        pc: u32,
        address: u16,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    ) -> Result<(), String> {
        if event.main != Some(8)
            || event.sub != Some(0)
            || !(OVERWORLD_LOAD_SINGLE_SPRITE_START_PC..OVERWORLD_LOAD_SINGLE_SPRITE_END_PC)
                .contains(&pc)
        {
            return Ok(());
        }
        let value = event
            .value
            .ok_or("Snes9x overworld sprite write omitted value")?;

        if (SPRITE_N_WORD_BASE..SPRITE_N_WORD_BASE + 32).contains(&address) {
            let byte_offset = address - SPRITE_N_WORD_BASE;
            let slot = (byte_offset / 2) as u8;
            if event.x != Some(u16::from(slot) * 2) {
                return Err(format!(
                    "Snes9x overworld sprite block write disagrees on word index: x={:?}, address=${address:04x}",
                    event.x
                ));
            }
            self.publish_overworld_presence(receipts);
            let tracker =
                self.overworld_sprite_activation
                    .get_or_insert(OverworldSpriteActivationTracker {
                        slot,
                        block_low: None,
                        block_high: None,
                        sprite_type: None,
                        state_published: false,
                    });
            if tracker.slot != slot {
                return Err(format!(
                    "Snes9x overworld sprite activation changed slot from {} to {slot}",
                    tracker.slot
                ));
            }
            let destination = if byte_offset & 1 == 0 {
                &mut tracker.block_low
            } else {
                &mut tracker.block_high
            };
            if destination.replace(value).is_some() {
                return Err(format!(
                    "Snes9x overworld sprite activation rewrote block byte at ${address:04x}"
                ));
            }
            return Ok(());
        }

        let Some(tracker) = self.overworld_sprite_activation.as_mut() else {
            return Ok(());
        };
        let expected_slot = u16::from(tracker.slot);
        if address == SPRITE_TYPE_BASE + expected_slot {
            if tracker.sprite_type.replace(value).is_some() {
                return Err("Snes9x overworld sprite activation rewrote its type".to_string());
            }
        } else if address == SPRITE_STATE_BASE + expected_slot {
            if value != 8 {
                return Err(format!(
                    "Snes9x overworld sprite activation published state {value}, expected 8"
                ));
            }
            tracker.state_published = true;
        } else if address == SPRITE_DIE_ACTION_BASE + expected_slot {
            if value != 0 {
                return Err(format!(
                    "Snes9x overworld sprite activation published die action {value}, expected 0"
                ));
            }
            let completed = self
                .overworld_sprite_activation
                .take()
                .expect("checked above");
            let block = u16::from(
                completed
                    .block_low
                    .ok_or("Snes9x overworld sprite activation omitted block low byte")?,
            ) | (u16::from(
                completed
                    .block_high
                    .ok_or("Snes9x overworld sprite activation omitted block high byte")?,
            ) << 8);
            let sprite_type = completed
                .sprite_type
                .ok_or("Snes9x overworld sprite activation omitted type")?;
            if !completed.state_published {
                return Err("Snes9x overworld sprite activation omitted state publication".into());
            }
            receipts.push(
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::SpriteActivated {
                        block,
                        slot: completed.slot,
                        sprite_type,
                    },
                ),
            );
        }
        Ok(())
    }
}

fn publish_spotlight_host_return_progress(
    returned_event: &RawTraceEvent,
    spotlight_var4_low: Option<u8>,
    spotlight_lower_cursor: Option<u16>,
    receipts: &mut Vec<OriginalTimingSemanticReceipt>,
) -> Result<(), String> {
    let Some(progress) =
        spotlight_table_build_progress(returned_event, spotlight_var4_low, spotlight_lower_cursor)?
    else {
        return Ok(());
    };
    // Host return is the latest source-visible state in this interval. If an
    // earlier NMI exposed the same synchronous C call, replace that checkpoint
    // rather than presenting two competing resumptions to native gameplay.
    receipts.retain(|receipt| {
        !matches!(
            receipt,
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(_)
        )
    });
    receipts.push(OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
        SpotlightTableBuildProgressReceipt {
            progress,
            boundary: OriginalTimingBoundary::HostReturn,
        },
    ));
    Ok(())
}

fn publish_pre_dungeon_sprite_reset_progress(
    event: &RawTraceEvent,
    boundary: OriginalTimingBoundary,
    receipts: &mut Vec<OriginalTimingSemanticReceipt>,
) -> Result<bool, String> {
    let pc = event.pc.map(|pc| pc & 0x00ff_ffff);
    let return_address = event.return_address.map(|address| address & 0x00ff_ffff);
    if !pc.is_some_and(|pc| {
        (SPRITE_RESET_ALL_NO_DISABLE_START_PC..SPRITE_RESET_ALL_END_PC).contains(&pc)
    }) || return_address != Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC)
    {
        return Ok(false);
    }
    // The return address is the source-owned caller proof. `Module_PreDungeon`
    // is both main-module 6's dispatch target and a direct callee of the
    // module-5 selected-game and module-27 spawn-select loaders. Those direct
    // callers deliberately retain their current module byte until
    // `Module_PreDungeon` publishes module 7 near its return, so the incidental
    // module/submodule state cannot identify this call boundary.
    receipts.retain(|receipt| {
        !matches!(
            receipt,
            OriginalTimingSemanticReceipt::SpriteResetAllProgress(_)
        )
    });
    receipts.push(OriginalTimingSemanticReceipt::SpriteResetAllProgress(
        SpriteResetAllProgressReceipt {
            progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
            boundary,
        },
    ));
    Ok(true)
}

fn spotlight_table_build_progress(
    event: &RawTraceEvent,
    spotlight_var4_low: Option<u8>,
    spotlight_lower_cursor: Option<u16>,
) -> Result<Option<SpotlightTableBuildProgress>, String> {
    // New traces bind these volatile direct-page values to the exact event.
    // The host-end reads are retained only as a backward-compatible fallback:
    // NMI handling or a same-host resume may legitimately repurpose scratch
    // before `retro_run` returns.
    let spotlight_var4_low = event.spotlight_var4_low.or(spotlight_var4_low);
    let spotlight_lower_cursor = event.spotlight_lower_cursor.or(spotlight_lower_cursor);
    let pc = event.pc.map(|pc| pc & 0x00ff_ffff);
    let inside_circle_value = pc.is_some_and(|pc| {
        (IRIS_SPOTLIGHT_CIRCLE_VALUE_START_PC..IRIS_SPOTLIGHT_CIRCLE_VALUE_END_PC).contains(&pc)
    });
    if !inside_circle_value
        && !matches!(
            pc,
            Some(
                IRIS_SPOTLIGHT_ITERATION_VALUE_STORE_PC
                    | IRIS_SPOTLIGHT_CIRCLE_VALUE_CALL_PC
                    | IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC
                    | IRIS_SPOTLIGHT_LOWER_TABLE_WRITE_PC
                    | IRIS_SPOTLIGHT_LOOP_COMPLETION_BRANCH_PC
                    | IRIS_SPOTLIGHT_LOWER_CURSOR_DECREMENT_PC
                    | IRIS_SPOTLIGHT_NEXT_ITERATION_PC
                    | IRIS_SPOTLIGHT_COPY_INIT_PC
                    | IRIS_SPOTLIGHT_COPY_LOAD_PC
                    | IRIS_SPOTLIGHT_COPY_STORE_PC
                    | IRIS_SPOTLIGHT_COPY_FIRST_INCREMENT_PC
                    | IRIS_SPOTLIGHT_COPY_SECOND_INCREMENT_PC
                    | IRIS_SPOTLIGHT_COPY_COMPARE_PC
                    | IRIS_SPOTLIGHT_COPY_BRANCH_PC
                    | IRIS_SPOTLIGHT_COPY_COMPLETE_PC
            )
        )
    {
        return Ok(None);
    }
    if !spotlight_receipt_domain(event) {
        // The same C routine also serves dungeon-landing and menu callers.
        // Their native continuations do not consume this opening/closing
        // receipt domain, so leave those legitimate executions to their own
        // authority instead of leaking a shared ROM address across domains.
        return Ok(None);
    }
    let link_y = event
        .link_y
        .ok_or("Snes9x spotlight checkpoint omitted Link Y")?;
    let bg2_v = event
        .bg2_v
        .ok_or("Snes9x spotlight checkpoint omitted BG2 vertical scroll")?;
    let radius = event
        .spotlight_radius
        .ok_or("Snes9x spotlight checkpoint omitted radius")?;
    // Source C initializes r6=max(2*r14,224), then decrements it once after
    // every completed loop iteration. These source invariants recover the
    // exact statement progress without exposing CPU registers to gameplay.
    let vertical_center = link_y.wrapping_sub(bg2_v).wrapping_add(12);
    let initial_lower_cursor = vertical_center.wrapping_mul(2).max(224);
    let y_upper = vertical_center.wrapping_add(radius);
    let iterations_before_iris = if initial_lower_cursor < y_upper {
        0
    } else {
        initial_lower_cursor.wrapping_sub(y_upper).wrapping_add(1)
    };
    let total_iterations = vertical_center
        .wrapping_sub(
            vertical_center
                .wrapping_mul(2)
                .wrapping_sub(initial_lower_cursor),
        )
        .wrapping_add(1);
    let iteration_initialization_checkpoint = matches!(
        pc,
        Some(IRIS_SPOTLIGHT_ITERATION_VALUE_STORE_PC | IRIS_SPOTLIGHT_NEXT_ITERATION_PC)
    );
    let projection_checkpoint = !inside_circle_value
        && !matches!(
            pc,
            Some(
                IRIS_SPOTLIGHT_ITERATION_VALUE_STORE_PC
                    | IRIS_SPOTLIGHT_CIRCLE_VALUE_CALL_PC
                    | IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC
                    | IRIS_SPOTLIGHT_LOWER_TABLE_WRITE_PC
                    | IRIS_SPOTLIGHT_LOOP_COMPLETION_BRANCH_PC
                    | IRIS_SPOTLIGHT_LOWER_CURSOR_DECREMENT_PC
                    | IRIS_SPOTLIGHT_NEXT_ITERATION_PC
            )
        );
    let (completed_iterations, checkpoint) = if iteration_initialization_checkpoint {
        let lower_cursor = spotlight_lower_cursor
            .ok_or("Snes9x spotlight iteration-start checkpoint omitted the source lower cursor")?;
        let completed_iterations = initial_lower_cursor
            .checked_sub(lower_cursor)
            .ok_or("Snes9x spotlight iteration-start cursor exceeded its source initial value")?;
        let active_iterations = completed_iterations.saturating_sub(iterations_before_iris);
        let expected_var4 = radius
            .checked_sub(active_iterations)
            .ok_or("Snes9x spotlight iteration-start cursor exceeded its source radius")?;
        if spotlight_var4_low != Some(expected_var4 as u8) {
            return Err(format!(
                "Snes9x spotlight iteration-start cursor derived spotlight_var4 {expected_var4}, observed {spotlight_var4_low:?}",
            ));
        }
        (
            completed_iterations,
            SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
        )
    } else if inside_circle_value
        || matches!(
            pc,
            Some(IRIS_SPOTLIGHT_CIRCLE_VALUE_CALL_PC | IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC)
        )
    {
        let (pending_circle_input, completed_iterations) = if pc
            == Some(IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC)
        {
            // At this statement the pure helper has returned, but neither
            // HDMA-table word has been published. The source loop's r4
            // cursor has not advanced yet, so X/2 identifies the current
            // iteration without relying on a host-side WRAM snapshot.
            let doubled_upper_cursor = event
                .x
                .ok_or("Snes9x spotlight upper-table checkpoint omitted X")?;
            if doubled_upper_cursor & 1 != 0 {
                return Err(format!(
                        "Snes9x spotlight upper cursor encoded an odd table byte offset {doubled_upper_cursor}",
                    ));
            }
            // Both cursors are signed source values: when the iris center
            // sits in the upper half of the screen the initial r4 is
            // negative and X carries its two's-complement doubling (route
            // host 196210, center 84 -> r4 starts at -56).
            let initial_upper_cursor = (vertical_center as i16)
                .wrapping_mul(2)
                .wrapping_sub(initial_lower_cursor as i16);
            let completed_iterations = u16::try_from(
                ((doubled_upper_cursor as i16) >> 1).wrapping_sub(initial_upper_cursor),
            )
            .map_err(|_| "Snes9x spotlight upper cursor preceded its source initial value")?;
            let active_iris_iterations =
                completed_iterations
                    .checked_sub(iterations_before_iris)
                    .ok_or("Snes9x spotlight upper cursor preceded its first iris iteration")?;
            let pending_circle_input = radius
                .checked_sub(active_iris_iterations)
                .and_then(|input| u8::try_from(input).ok())
                .ok_or("Snes9x spotlight upper cursor exceeded its source radius")?;
            if let Some(var4) = spotlight_var4_low {
                let var4_input = var4.checked_add(1).ok_or(
                    "Snes9x spotlight pure-circle checkpoint overflowed its pending input",
                )?;
                if var4_input != pending_circle_input {
                    return Err(format!(
                            "Snes9x spotlight upper cursor derived input {pending_circle_input} but spotlight_var4 derived {var4_input}",
                        ));
                }
            }
            (pending_circle_input, completed_iterations)
        } else {
            let pending_circle_input = if inside_circle_value {
                spotlight_var4_low
                    .ok_or("Snes9x spotlight pure-circle checkpoint omitted spotlight_var4")?
                    .checked_add(1)
                    .ok_or("Snes9x spotlight pure-circle checkpoint overflowed its pending input")?
            } else {
                u8::try_from(
                    event
                        .a
                        .ok_or("Snes9x spotlight checkpoint omitted accumulator")?,
                )
                .map_err(|_| "Snes9x spotlight circle input exceeded one byte")?
            };
            let completed_iterations = iterations_before_iris
                .checked_add(
                    radius
                        .checked_sub(u16::from(pending_circle_input))
                        .ok_or("Snes9x spotlight circle input exceeded its source radius")?,
                )
                .ok_or("Snes9x spotlight iteration count overflowed")?;
            (pending_circle_input, completed_iterations)
        };
        if pending_circle_input == 0 || u16::from(pending_circle_input) > radius {
            return Err(format!(
                "Snes9x spotlight circle input {pending_circle_input} is not derivable from radius {radius}",
            ));
        }
        (
            completed_iterations,
            SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                pending_circle_input,
            },
        )
    } else if pc == Some(IRIS_SPOTLIGHT_LOWER_TABLE_WRITE_PC) {
        let doubled_lower_cursor = event
            .x
            .ok_or("Snes9x spotlight lower-table checkpoint omitted X")?;
        if doubled_lower_cursor & 1 != 0 {
            return Err(format!(
                "Snes9x spotlight lower cursor encoded an odd table byte offset {doubled_lower_cursor}",
            ));
        }
        let lower_cursor = doubled_lower_cursor >> 1;
        let completed_iterations = initial_lower_cursor
            .checked_sub(lower_cursor)
            .ok_or("Snes9x spotlight lower cursor exceeded its source initial value")?;
        let circle_value = event
            .a
            .ok_or("Snes9x spotlight lower-table checkpoint omitted circle value")?;
        (
            completed_iterations,
            SpotlightTableBuildCheckpoint::BeforeLowerTableWrite {
                lower_cursor,
                circle_value,
            },
        )
    } else if matches!(
        pc,
        Some(IRIS_SPOTLIGHT_LOOP_COMPLETION_BRANCH_PC | IRIS_SPOTLIGHT_LOWER_CURSOR_DECREMENT_PC)
    ) {
        let observed_x = event
            .x
            .ok_or("Snes9x spotlight loop-test checkpoint omitted X")?;
        let initial_upper_cursor = vertical_center
            .wrapping_mul(2)
            .wrapping_sub(initial_lower_cursor);
        // The assembly uses X for each visible long-indexed table store. The
        // lower store follows the upper store, so X contains 2*r6 when the
        // lower cursor is visible, otherwise 2*r4 when only the upper cursor
        // is visible. When BOTH rows are clipped (`r4 >= 240 && r6 >= 240`,
        // the off-screen head of a large iris), no store reloads X and it
        // retains `IrisSpotlight_CalculateCircleValue`'s helper-table index
        // `t = ((input << 8) / radius) >> 1` from `$00:F4CC`'s TAX — an
        // odd value is legal there (route frame 111852, X=13). Reconstruct
        // the unique C loop iteration whose source register trace leaves
        // the observed value; do not assume X always owns one particular
        // local cursor.
        let mut matched = None;
        // Pass 1: store-retained values. The ROM's store guard compares the
        // DOUBLED cursor against #$01C0: rows at index 224..239 are skipped
        // by the loop (the epilogue clears them separately), so X is
        // reloaded only for cursors below 224.
        for completed_iterations in 0..total_iterations {
            let upper_cursor = initial_upper_cursor.wrapping_add(completed_iterations);
            let lower_cursor = initial_lower_cursor.wrapping_sub(completed_iterations);
            let retained_x = if lower_cursor < 224 {
                Some(lower_cursor * 2)
            } else if upper_cursor < 224 {
                Some(upper_cursor * 2)
            } else {
                None
            };
            if retained_x == Some(observed_x) {
                if matched
                    .replace((completed_iterations, upper_cursor, lower_cursor))
                    .is_some()
                {
                    return Err(format!(
                        "Snes9x spotlight loop-test X {observed_x} maps to multiple source iterations",
                    ));
                }
            }
        }
        // Pass 2: when no visible store reloaded X for the observed value,
        // both rows were clipped and X retains the circle helper's
        // quantized table index `t = ((input << 8) / radius) >> 1` from
        // `$00:F4CC`'s TAX (route frame 111852, X = 13 at input 11 of
        // radius 105).
        if matched.is_none() && radius != 0 {
            for completed_iterations in iterations_before_iris..total_iterations {
                let upper_cursor = initial_upper_cursor.wrapping_add(completed_iterations);
                let lower_cursor = initial_lower_cursor.wrapping_sub(completed_iterations);
                if lower_cursor < 224 || upper_cursor < 224 {
                    continue;
                }
                let active_iterations = completed_iterations - iterations_before_iris;
                let pending_circle_input = radius.saturating_sub(active_iterations);
                let helper_index = (u32::from(pending_circle_input) << 8) / u32::from(radius) >> 1;
                if helper_index as u16 == observed_x {
                    if matched
                        .replace((completed_iterations, upper_cursor, lower_cursor))
                        .is_some()
                    {
                        return Err(format!(
                            "Snes9x spotlight loop-test helper index {observed_x} maps to multiple clipped source iterations",
                        ));
                    }
                }
            }
        }
        let (completed_iterations, upper_cursor, lower_cursor) = matched.ok_or_else(|| {
            format!(
                "Snes9x spotlight loop-test X {observed_x} is not produced by a visible source table store (radius={radius} vc={vertical_center} init_lower={initial_lower_cursor} before_iris={iterations_before_iris} total={total_iterations} a={:?} y={:?})",
                event.a, event.y,
            )
        })?;
        let (completed_iterations, checkpoint) =
            if pc == Some(IRIS_SPOTLIGHT_LOWER_CURSOR_DECREMENT_PC) {
                if upper_cursor == vertical_center {
                    return Err(
                        "Snes9x spotlight lower-cursor decrement followed a completed source loop"
                            .to_string(),
                    );
                }
                (
                    completed_iterations,
                    SpotlightTableBuildCheckpoint::BeforeLowerCursorDecrement {
                        upper_cursor: upper_cursor.wrapping_add(1),
                        lower_cursor,
                    },
                )
            } else {
                (
                    completed_iterations,
                    SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                        upper_cursor,
                        lower_cursor,
                    },
                )
            };
        (completed_iterations, checkpoint)
    } else {
        let copied_bytes = match pc.expect("recognized spotlight copy PC") {
            IRIS_SPOTLIGHT_COPY_INIT_PC => 0,
            IRIS_SPOTLIGHT_COPY_LOAD_PC | IRIS_SPOTLIGHT_COPY_STORE_PC => event
                .x
                .ok_or("Snes9x spotlight projection checkpoint omitted X")?,
            IRIS_SPOTLIGHT_COPY_FIRST_INCREMENT_PC => event
                .x
                .ok_or("Snes9x spotlight projection checkpoint omitted X")?
                .checked_add(2)
                .ok_or("Snes9x spotlight projection byte count overflowed")?,
            IRIS_SPOTLIGHT_COPY_SECOND_INCREMENT_PC => event
                .x
                .ok_or("Snes9x spotlight projection checkpoint omitted X")?
                .checked_add(1)
                .ok_or("Snes9x spotlight projection byte count overflowed")?,
            IRIS_SPOTLIGHT_COPY_COMPARE_PC | IRIS_SPOTLIGHT_COPY_BRANCH_PC => event
                .x
                .ok_or("Snes9x spotlight projection checkpoint omitted X")?,
            IRIS_SPOTLIGHT_COPY_COMPLETE_PC => 448,
            _ => unreachable!("spotlight projection PC was filtered above"),
        };
        if copied_bytes & 1 != 0 || copied_bytes > 448 {
            return Err(format!(
                "Snes9x spotlight projection encoded invalid copied-byte count {copied_bytes}",
            ));
        }
        (
            total_iterations,
            SpotlightTableBuildCheckpoint::ProjectionCopy {
                copied_words: copied_bytes >> 1,
            },
        )
    };
    if !projection_checkpoint && completed_iterations >= total_iterations {
        return Err(format!(
            "Snes9x spotlight checkpoint derived {completed_iterations} completed iterations for a {total_iterations}-iteration table",
        ));
    }
    Ok(Some(SpotlightTableBuildProgress {
        completed_iterations,
        checkpoint,
    }))
}

fn spotlight_receipt_domain(event: &RawTraceEvent) -> bool {
    matches!(
        (event.main, event.sub),
        (Some(0x0f), Some(0 | 1)) | (Some(0x10), Some(0 | 1))
    )
}

fn zelda_main_wait_pc(pc: u32) -> bool {
    matches!(pc, 0x00_8034 | 0x00_8036)
}

fn main_loop_interruption_for_pc(pc: u32) -> Option<MainLoopInterruption> {
    if (LINK_OAM_START_PC..LINK_OAM_END_PC).contains(&pc) {
        Some(MainLoopInterruption::LinkOam)
    } else if (NMI_PREPARE_SPRITES_START_PC..NMI_PREPARE_SPRITES_END_PC).contains(&pc) {
        Some(MainLoopInterruption::SpritePreparation)
    } else {
        None
    }
}

fn main_loop_interruption_for_source_state(
    pc: u32,
    main: Option<u8>,
    sub: Option<u8>,
    x: Option<u16>,
) -> Option<MainLoopInterruption> {
    if main == Some(0x0f)
        && sub == Some(1)
        && ((LINK_VELOCITY_BEFORE_STATE_BRANCH_START_PC..LINK_VELOCITY_BEFORE_STATE_BRANCH_END_PC)
            .contains(&pc)
            || (LINK_POSITION_BEFORE_COORDINATES_START_PC..LINK_POSITION_BEFORE_COORDINATES_END_PC)
                .contains(&pc))
    {
        Some(MainLoopInterruption::LinkPositionBeforeCoordinates)
    } else if main == Some(0x0f)
        && sub == Some(1)
        && (LINK_POSITION_AFTER_SUBPIXEL_START_PC..LINK_POSITION_AFTER_SUBPIXEL_END_PC)
            .contains(&pc)
    {
        let pass = u8::try_from(x?)
            .ok()
            .filter(|pass| matches!(pass, 0 | 2 | 4))?;
        Some(MainLoopInterruption::LinkPositionAfterSubpixel { pass })
    } else if matches!((main, sub), (Some(0x0f | 0x10), Some(0 | 1)))
        && (IRIS_SPOTLIGHT_RESET_TABLE_FIRST_STORE_PC..=IRIS_SPOTLIGHT_RESET_TABLE_BRANCH_PC)
            .contains(&pc)
    {
        let completed_stores = spotlight_reset_table_completed_stores(pc, x?)?;
        Some(MainLoopInterruption::SpotlightGoalResetTable { completed_stores })
    } else {
        main_loop_interruption_for_pc(pc)
    }
}

/// Recover how many of `IrisSpotlight_ResetTable`'s 224 source-order stores
/// completed before an interruption at `pc` with loop register `x`.
fn spotlight_reset_table_completed_stores(pc: u32, x: u16) -> Option<u8> {
    // Every store is three bytes long; the seven stores sit at consecutive
    // three-byte offsets from the first one.
    let (iteration_x, stores_in_iteration) = if pc < IRIS_SPOTLIGHT_RESET_TABLE_FIRST_DEX_PC {
        let offset = pc - IRIS_SPOTLIGHT_RESET_TABLE_FIRST_STORE_PC;
        if offset % 3 != 0 {
            return None;
        }
        (x, offset / 3)
    } else if pc == IRIS_SPOTLIGHT_RESET_TABLE_FIRST_DEX_PC {
        (
            x,
            u32::from(IRIS_SPOTLIGHT_RESET_TABLE_STORES_PER_ITERATION),
        )
    } else if pc == IRIS_SPOTLIGHT_RESET_TABLE_SECOND_DEX_PC {
        (
            x.wrapping_add(1),
            u32::from(IRIS_SPOTLIGHT_RESET_TABLE_STORES_PER_ITERATION),
        )
    } else {
        (
            x.wrapping_add(2),
            u32::from(IRIS_SPOTLIGHT_RESET_TABLE_STORES_PER_ITERATION),
        )
    };
    if iteration_x > IRIS_SPOTLIGHT_RESET_TABLE_INITIAL_X || iteration_x % 2 != 0 {
        return None;
    }
    let completed_iterations = (IRIS_SPOTLIGHT_RESET_TABLE_INITIAL_X - iteration_x) / 2;
    let completed = u32::from(completed_iterations)
        * u32::from(IRIS_SPOTLIGHT_RESET_TABLE_STORES_PER_ITERATION)
        + stores_in_iteration;
    u8::try_from(completed).ok()
}

fn main_loop_interruption_for_event(
    event: &RawTraceEvent,
) -> Result<Option<MainLoopInterruption>, String> {
    let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
        return Ok(None);
    };
    if (NMI_PREPARE_EXTENDED_OAM_GROUP_BEFORE_STORE_START_PC
        ..NMI_PREPARE_EXTENDED_OAM_GROUP_BEFORE_STORE_END_PC)
        .contains(&pc)
    {
        let next_group_start = u8::try_from(
            event
                .y
                .ok_or("Snes9x extended-OAM packing interruption omitted source cursor Y")?,
        )
        .map_err(|_| "Snes9x extended-OAM packing cursor exceeded one byte")?;
        if next_group_start > 28 || next_group_start & 3 != 0 {
            return Err(format!(
                "Snes9x extended-OAM packing interruption used invalid group cursor {next_group_start}",
            ));
        }
        let source_x = event
            .x
            .ok_or("Snes9x extended-OAM packing interruption omitted source cursor X")?;
        if source_x != u16::from(next_group_start) * 4 {
            return Err(format!(
                "Snes9x extended-OAM packing cursors disagreed: y={next_group_start}, x={source_x}",
            ));
        }
        return Ok(Some(
            MainLoopInterruption::SpritePreparationExtendedOamPacking { next_group_start },
        ));
    }
    Ok(main_loop_interruption_for_source_state(
        pc, event.main, event.sub, event.x,
    ))
}

/// Remove the source-call interruption belonging to an NMI whose exact
/// stack-qualified context resumed within this same host interval.
///
/// A pinned-Snes9x `retro_run` can accept an NMI just after entry, return to
/// the interrupted C call, and then accept the following field's NMI before
/// the host call returns. The first interruption is not a surviving gameplay
/// boundary; its ordered NMI acceptance/publication receipts remain, while
/// only the later still-suspended interruption is exported to gameplay.
fn retire_resumed_main_loop_interruption(
    receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    resumed_sprite_progress: Option<SpriteMainProgress>,
) -> Result<(), String> {
    let Some(last_acceptance) = receipts
        .iter()
        .rposition(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted(_)))
    else {
        if let Some(progress) = resumed_sprite_progress {
            receipts.push(OriginalTimingSemanticReceipt::SpriteMainProgressed(
                progress,
            ));
        }
        return Ok(());
    };
    let interruptions = receipts
        .iter()
        .enumerate()
        .skip(last_acceptance + 1)
        .filter_map(|(index, receipt)| {
            matches!(
                receipt,
                OriginalTimingSemanticReceipt::MainLoopInterrupted(_)
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    match interruptions.as_slice() {
        [] => {
            if let Some(progress) = resumed_sprite_progress {
                receipts.push(OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    progress,
                ));
            }
            Ok(())
        }
        [index] => {
            let OriginalTimingSemanticReceipt::MainLoopInterrupted(interruption) = receipts[*index]
            else {
                unreachable!("interruption index changed receipt kind")
            };
            let progress = match interruption {
                MainLoopInterruption::SpriteMainBeforeFirstSlot => {
                    Some(SpriteMainProgress::BeforeFirstSlot)
                }
                MainLoopInterruption::SpriteMainAfterSlot(slot) => {
                    Some(SpriteMainProgress::AfterSlot(slot))
                }
                MainLoopInterruption::SpriteMainAfterActiveCuccoX {
                    slot,
                    helper_ordinal,
                } => Some(SpriteMainProgress::AfterActiveCuccoX {
                    slot,
                    helper_ordinal,
                }),
                MainLoopInterruption::SpriteMainAfterActiveCuccoYSubpixel {
                    slot,
                    helper_ordinal,
                } => Some(SpriteMainProgress::AfterActiveCuccoYSubpixel {
                    slot,
                    helper_ordinal,
                }),
                MainLoopInterruption::SpriteMainAfterCuccoSubtypeIncrements {
                    slot,
                    helper_ordinal,
                    completed,
                } => Some(SpriteMainProgress::AfterCuccoSubtypeIncrements {
                    slot,
                    helper_ordinal,
                    completed,
                }),
                MainLoopInterruption::SpriteMainAfterCuccoGraphicsPublication {
                    slot,
                    helper_ordinal,
                } => Some(SpriteMainProgress::AfterCuccoGraphicsPublication {
                    slot,
                    helper_ordinal,
                }),
                MainLoopInterruption::SpriteMainBigKeyDropGraphicsStarted(slot) => {
                    Some(SpriteMainProgress::BigKeyDropGraphicsStarted(slot))
                }
                _ => None,
            };
            if let Some(progress) = progress {
                receipts[*index] = OriginalTimingSemanticReceipt::SpriteMainProgressed(progress);
            } else {
                receipts.remove(*index);
            }
            Ok(())
        }
        _ => Err(
            "Snes9x published multiple main-loop interruptions for one accepted NMI".to_string(),
        ),
    }
}

fn sprite_disable_progress(
    pc: u32,
    address: u16,
    value: Option<u8>,
) -> Result<Option<DungeonSpriteDisableCpuProgress>, String> {
    if !(DUNGEON_RESET_SPRITES_CLEAR_PC..SPRITE_DISABLE_ALL_END_PC).contains(&pc) {
        return Ok(None);
    }
    let progress = if pc == DUNGEON_RESET_SPRITES_CLEAR_PC
        && (SPRITE_STATE_BASE..SPRITE_STATE_BASE + 16).contains(&address)
    {
        Some(DungeonSpriteDisableCpuProgress::SpriteStatesThrough {
            slot: (address - SPRITE_STATE_BASE) as u8,
        })
    } else if (ANCILLA_TYPE_BASE..ANCILLA_TYPE_BASE + 10).contains(&address) {
        Some(DungeonSpriteDisableCpuProgress::AncillasThrough {
            slot: (address - ANCILLA_TYPE_BASE) as u8,
        })
    } else if address == ANCILLA_PICKUP_FLAG {
        Some(DungeonSpriteDisableCpuProgress::AncillaPickupFlagCleared)
    } else if address == SPRITE_LIMIT_INSTANCE {
        Some(DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared)
    } else {
        None
    };
    if progress.is_some() && value != Some(0) {
        return Err(format!(
            "Snes9x Sprite_DisableAll progress wrote nonzero value {:?} to ${address:04x}",
            value
        ));
    }
    Ok(progress)
}

fn cached_sprite_live_field(address: u16) -> Option<(usize, u8)> {
    CACHED_SPRITE_LIVE_FIELDS
        .iter()
        .enumerate()
        .find_map(|(field_index, &base)| {
            (base..base + 16)
                .contains(&address)
                .then(|| (field_index, (address - base) as u8))
        })
}

fn append_csv(existing: Option<&str>, required: &[&str]) -> String {
    let mut values = existing
        .into_iter()
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    for &required in required {
        if !values.iter().any(|value| value == required) {
            values.push(required.to_string());
        }
    }
    values.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    const NMI_HANDLER_COMPLETE_PC: u32 = NMI_HANDLER_COMPLETE_PCS[0];

    #[test]
    fn semantic_trace_configuration_observes_acceptance_publication_and_context_resume() {
        assert!(REQUIRED_TRACE_EVENTS.contains(&"nmi"));
        assert!(REQUIRED_TRACE_EVENTS.contains(&"nmi-resume"));
        assert_eq!(NMI_UPDATE_LATCH, 0x0012);
        assert_eq!(ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC, 0x00805f);
        assert_eq!(NMI_HANDLER_COMPLETE_PCS, [0x0000_8225, 0x0000_82c7]);
        assert_eq!(
            append_csv(Some("dma,nmi"), REQUIRED_TRACE_EVENTS),
            "dma,nmi,frame,nmi-resume,wram,rom-rng,pc",
        );
    }

    #[test]
    fn nmi_publication_emits_exact_zelda_joypad_bytes() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        source
            .consume_event(raw("nmi", Some(0x008036), None, None), &mut receipts)
            .unwrap();
        let mut completion = raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f8);
        completion.joypad_high = Some(0x05);
        completion.joypad_low = Some(0x80);
        completion.joypad_high_filtered = Some(0x04);
        completion.joypad_low_filtered = Some(0x80);
        source.consume_event(completion, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0x05,
                    low: 0x80,
                    high_filtered: 0x04,
                    low_filtered: 0x80,
                }),
            ]
        );
    }

    #[test]
    fn partial_nmi_joypad_publication_fails_before_committing_completion() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        source
            .consume_event(raw("nmi", Some(0x008036), None, None), &mut receipts)
            .unwrap();
        let mut completion = raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f8);
        completion.joypad_high = Some(0x05);
        completion.joypad_low = None;
        completion.joypad_high_filtered = None;
        completion.joypad_low_filtered = None;

        assert!(source
            .consume_event(completion, &mut receipts)
            .unwrap_err()
            .contains("omitted part"));
        assert!(source.nmi_publication_pending);
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open
            )]
        );
    }

    #[test]
    fn semantic_trace_checkpoint_preserves_cross_host_nmi_ownership() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        source
            .consume_event(raw("nmi", Some(0x008036), None, None), &mut receipts)
            .unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open
            )]
        );

        let bytes = serde_json::to_vec(&source.checkpoint()).unwrap();
        let checkpoint = serde_json::from_slice(&bytes).unwrap();
        let mut resumed = empty_semantic_tracker();
        resumed.restore_checkpoint(checkpoint).unwrap();
        let mut resumed_receipts = Vec::new();
        publish_nmi(&mut resumed, &mut resumed_receipts);

        assert_eq!(
            resumed_receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0,
                    low: 0,
                    high_filtered: 0,
                    low_filtered: 0,
                }),
            ]
        );
        assert!(!resumed.nmi_publication_pending);
        assert_eq!(resumed.nmi_resume_targets, source.nmi_resume_targets);
    }

    #[test]
    fn cross_host_latch_held_nmi_completes_without_joypad_publication() {
        let mut source = empty_semantic_tracker();
        let mut accepted = raw("nmi", Some(0x0c_ce3c), None, None);
        accepted.nmi_latch = Some(1);
        let mut receipts = Vec::new();
        source.consume_event(accepted, &mut receipts).unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::LatchHeld,
            )],
        );

        let checkpoint =
            serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
        let mut resumed = empty_semantic_tracker();
        resumed.restore_checkpoint(checkpoint).unwrap();
        let mut completion = raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f8);
        // The trace event still carries Zelda's unchanged joypad bytes. A held
        // `$12` gate proves NMI_ReadJoypads did not publish them this handler.
        completion.joypad_high = Some(0xa5);
        completion.joypad_low = Some(0x5a);
        completion.joypad_high_filtered = Some(0x81);
        completion.joypad_low_filtered = Some(0x42);
        let mut resumed_receipts = Vec::new();
        resumed
            .consume_event(completion, &mut resumed_receipts)
            .unwrap();

        assert_eq!(
            resumed_receipts,
            vec![OriginalTimingSemanticReceipt::NmiHandlerCompleted],
        );
        assert!(!resumed.nmi_publication_pending);
        assert!(resumed.pending_nmi_update_gate.is_none());
    }

    #[test]
    fn semantic_trace_checkpoint_preserves_native_mode_nmi_stack_context() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        source
            .consume_event(raw_at("nmi", 0x09fe65, 0x1f34), &mut receipts)
            .unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open
            )]
        );

        let bytes = serde_json::to_vec(&source.checkpoint()).unwrap();
        let checkpoint = serde_json::from_slice(&bytes).unwrap();
        let mut resumed = empty_semantic_tracker();
        resumed.restore_checkpoint(checkpoint).unwrap();

        let mut resumed_receipts = Vec::new();
        publish_nmi(&mut resumed, &mut resumed_receipts);
        resumed
            .consume_event(
                raw_at("nmi-resume", 0x09fe65, 0x1f34),
                &mut resumed_receipts,
            )
            .unwrap();

        assert_eq!(
            resumed_receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0,
                    low: 0,
                    high_filtered: 0,
                    low_filtered: 0,
                }),
            ]
        );
        assert!(resumed.nmi_resume_targets.is_empty());
    }

    #[test]
    fn sprite_main_nmi_exports_only_the_last_completed_source_slot() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        source
            .consume_event(
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(15), None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(SPRITE_SLOT_RETURN_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(14), None),
                &mut receipts,
            )
            .unwrap();

        // Persisting between the source slot return and interrupt must retain
        // the semantic loop cursor without exporting CPU state to gameplay.
        let checkpoint =
            serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
        let mut resumed = empty_semantic_tracker();
        resumed.restore_checkpoint(checkpoint).unwrap();
        resumed
            .consume_event(raw("nmi", Some(0x06_f80f), Some(14), None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterSlot(15),
                ),
            ],
        );
    }

    #[test]
    fn nmi_acceptance_decodes_complete_ppu_register_operands_and_rejects_absence() {
        let mut event = raw("nmi", None, None, None);
        event.nmi_ppu_register_operands = Some(std::array::from_fn(|index| index as u8));
        let operands = event.nmi_ppu_register_operands().unwrap();
        assert_eq!(operands.window_selection, [0, 1, 2]);
        assert_eq!(operands.fixed_color, [5, 6, 7]);
        assert_eq!(operands.screen_layers, [8, 9, 10, 11]);
        assert_eq!(
            operands.bg_scroll,
            [0x0d0c, 0x0f0e, 0x1110, 0x1312, 0x1514, 0x1716]
        );
        assert_eq!(operands.screen_brightness, 24);
        assert_eq!(operands.mosaic, 25);
        assert_eq!(operands.bg_mode, 26);
        assert_eq!(operands.mode7_center, [0x1c1b, 0x1e1d]);

        event.nmi_ppu_register_operands = None;
        assert!(event.nmi_ppu_register_operands().is_err());
    }

    #[test]
    fn resumed_entry_nmi_preserves_sprite_progress_without_duplicate_interruption() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        for (pc, x) in [
            (SPRITE_MAIN_ENTRY_PC, None),
            (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(15)),
            (SPRITE_SLOT_RETURN_PC, None),
            (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(14)),
        ] {
            source
                .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
                .unwrap();
        }

        source
            .consume_event(raw_at("nmi", 0x06_f80f, 0x01ff), &mut receipts)
            .unwrap();
        source
            .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
            .unwrap();
        source
            .consume_event(raw_at("nmi-resume", 0x06_f80f, 0x01ff), &mut receipts)
            .unwrap();

        source
            .consume_event(
                raw("pc", Some(SPRITE_SLOT_RETURN_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(13), None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(raw_at("nmi", 0x06_f80f, 0x01ff), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(SpriteMainProgress::AfterSlot(
                    15
                ),),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterSlot(14),
                ),
            ],
        );
    }

    #[test]
    fn cross_host_nmi_resume_emits_progress_from_the_persistent_sprite_loop() {
        let mut source = empty_semantic_tracker();
        let mut previous_host = Vec::new();
        for (pc, x) in [
            (SPRITE_MAIN_ENTRY_PC, None),
            (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(15)),
            (SPRITE_SLOT_RETURN_PC, None),
            (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(14)),
        ] {
            source
                .consume_event(raw("pc", Some(pc), x, None), &mut previous_host)
                .unwrap();
        }
        source
            .consume_event(raw_at("nmi", 0x06_f80f, 0x01ff), &mut previous_host)
            .unwrap();
        assert_eq!(
            source
                .sprite_main_execution
                .map(|execution| execution.progress()),
            Some(SpriteMainProgress::AfterSlot(15)),
        );

        let checkpoint =
            serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
        let mut resumed = empty_semantic_tracker();
        resumed.restore_checkpoint(checkpoint).unwrap();
        assert_eq!(
            resumed
                .sprite_main_execution
                .map(|execution| execution.progress()),
            Some(SpriteMainProgress::AfterSlot(15)),
        );
        let mut current_host = Vec::new();
        resumed
            .consume_event(
                raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3),
                &mut current_host,
            )
            .unwrap();
        assert_eq!(resumed.nmi_resume_targets, vec![(0x06_f80f, 0x01ff)]);
        assert_eq!(resumed.synthesized_nmi_resume, None);
        assert_eq!(
            resumed
                .sprite_main_execution
                .map(|execution| execution.progress()),
            Some(SpriteMainProgress::AfterSlot(15)),
        );
        resumed
            .consume_event(raw_at("nmi-resume", 0x06_f80f, 0x01ff), &mut current_host)
            .unwrap();

        assert_eq!(
            current_host,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(SpriteMainProgress::AfterSlot(
                    15
                ),),
            ],
        );
    }

    #[test]
    fn host_return_publishes_the_active_sprite_main_checkpoint_without_an_nmi() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        for (pc, x) in [
            (SPRITE_MAIN_ENTRY_PC, None),
            (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(4)),
            (SPRITE_SLOT_RETURN_PC, None),
            (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(3)),
        ] {
            source
                .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
                .unwrap();
        }

        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterSlot(4),
            )],
        );
        assert_eq!(
            source
                .sprite_main_execution
                .map(|execution| execution.progress()),
            Some(SpriteMainProgress::AfterSlot(4)),
        );
    }

    #[test]
    fn host_return_coalesces_resumed_sprite_progress_to_the_latest_checkpoint() {
        let mut source = empty_semantic_tracker();
        source.sprite_main_execution = Some(SpriteMainExecutionTracker {
            current_slot: Some(2),
            last_completed_slot: Some(3),
            cucco_subtype_increments: None,
            cucco_helper_ordinal: 0,
            cucco_flee_movement: None,
            active_cucco_movement: None,
            active_cucco_x_publications: 0,
            active_cucco_y_subpixel: None,
            cucco_animation_slot: None,
            big_key_drop_graphics_slot: None,
        });
        let mut receipts = vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
            SpriteMainProgress::AfterSlot(4),
        )];

        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterSlot(3),
            )],
        );
    }

    #[test]
    fn big_key_type_publication_becomes_a_typed_partial_slot_checkpoint() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        source
            .consume_event(
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(2), None),
                &mut receipts,
            )
            .unwrap();
        let mut publication = raw(
            "wram-write",
            Some(BIG_KEY_DROP_TYPE_PUBLICATION_PC),
            Some(2),
            Some(SPRITE_TYPE_BASE + 2),
        );
        publication.value = Some(BIG_KEY_DROP_SPRITE_TYPE);
        source.consume_event(publication, &mut receipts).unwrap();

        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::BigKeyDropGraphicsStarted(2),
            )],
        );
        assert_eq!(
            source
                .sprite_main_execution
                .map(|execution| execution.interruption()),
            Some(MainLoopInterruption::SpriteMainBigKeyDropGraphicsStarted(2)),
        );
    }

    #[test]
    fn ordinary_enemy_drop_type_does_not_enter_the_big_key_receipt_domain() {
        let mut source = empty_semantic_tracker();
        source.sprite_main_execution = Some(SpriteMainExecutionTracker {
            current_slot: Some(1),
            ..SpriteMainExecutionTracker::default()
        });
        let mut receipts = Vec::new();
        let mut publication = raw(
            "wram-write",
            Some(BIG_KEY_DROP_TYPE_PUBLICATION_PC),
            Some(1),
            Some(SPRITE_TYPE_BASE + 1),
        );
        publication.value = Some(0xd8);

        source.consume_event(publication, &mut receipts).unwrap();

        assert_eq!(
            source
                .sprite_main_execution
                .map(|execution| execution.progress()),
            Some(SpriteMainProgress::BeforeFirstSlot),
        );
        assert!(receipts.is_empty());
    }

    #[test]
    fn sprite_main_slot_zero_then_common_return_closes_the_tracker_once() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        for (pc, x) in [
            (SPRITE_MAIN_ENTRY_PC, None),
            (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(0)),
            (SPRITE_SLOT_RETURN_PC, None),
            (SPRITE_MAIN_RETURN_PC, None),
        ] {
            source
                .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
                .unwrap();
        }

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainReturned],
        );
        assert!(source.sprite_main_execution.is_none());
    }

    #[test]
    fn cached_sprite_execute_single_after_sprite_main_does_not_reopen_the_loop() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        for (pc, x) in [
            (SPRITE_MAIN_ENTRY_PC, None),
            (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(0)),
            (SPRITE_SLOT_RETURN_PC, None),
            // ExecuteCachedSprites invokes the leaf directly after the
            // descending Sprite_Main loop has closed.
            (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(9)),
        ] {
            source
                .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
                .unwrap();
        }

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainReturned],
        );
        assert!(source.sprite_main_execution.is_none());
    }

    #[test]
    fn fresh_sprite_main_entry_proves_prior_item_receipt_caller_returned() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        source
            .consume_event(
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw(
                    "pc",
                    Some(BOTTLE_VENDOR_ITEM_RECEIPT_CALL_PC),
                    Some(0),
                    None,
                ),
                &mut receipts,
            )
            .unwrap();

        // A later top-level Sprite_Main entry cannot coexist with the prior
        // synchronous item call on the source CPU stack. It is therefore a
        // stronger semantic return proof than any caller-specific PC marker.
        source
            .consume_event(
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                &mut receipts,
            )
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                    ItemReceiptGraphicsProgressReceipt {
                        caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 0 },
                        progress: SourceCallProgress::Returned,
                    },
                ),
                OriginalTimingSemanticReceipt::SpriteMainReturned,
            ],
        );
        assert!(source.item_receipt_caller.is_none());
    }

    #[test]
    fn sprite_main_nmi_exports_cucco_animation_publication_before_lift_tail_returns() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        source
            .consume_event(
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(6), None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw(
                    "wram-write",
                    Some(CUCCO_ANIMATION_PUBLICATION_PC),
                    Some(6),
                    Some(SPRITE_GRAPHICS_BASE + 6),
                ),
                &mut receipts,
            )
            .unwrap();

        // The semantic cursor is part of the paired oracle checkpoint. A
        // resume between the publication and the NMI must not regress to the
        // prior fully-returned slot.
        let checkpoint =
            serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
        let mut resumed = empty_semantic_tracker();
        resumed.restore_checkpoint(checkpoint).unwrap();
        resumed
            .consume_event(raw("nmi", Some(0x06_f80f), Some(6), None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterCuccoGraphicsPublication {
                        slot: 6,
                        helper_ordinal: 0,
                    },
                ),
            ],
        );
    }

    #[test]
    fn sprite_main_nmi_after_cucco_flee_movement_keeps_the_subtype_helper_pending() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        source
            .consume_event(
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(2), None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(CUCCO_FLEE_SUBTYPE_HELPER_CALL_PC), Some(2), None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(raw("nmi", Some(0x06_a724), Some(2), None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterCuccoFleeMovement {
                        slot: 2,
                        helper_ordinal: 0,
                    },
                ),
            ],
        );
    }

    #[test]
    fn sprite_main_host_return_after_active_cucco_x_keeps_y_movement_pending() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(1), None),
            raw("pc", Some(ACTIVE_CUCCO_MOVEMENT_CALL_PC), Some(1), None),
            raw(
                "wram-write",
                Some(0x06_e94e),
                Some(0x11),
                Some(SPRITE_X_SUBPIXEL_BASE + 1),
            ),
            raw(
                "wram-write",
                Some(0x06_e964),
                Some(0x11),
                Some(SPRITE_X_LOW_BASE + 1),
            ),
            raw(
                "wram-write",
                Some(0x06_e96b),
                Some(0x11),
                Some(SPRITE_X_HIGH_BASE + 1),
            ),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterActiveCuccoX {
                    slot: 1,
                    helper_ordinal: 0,
                },
            )],
        );
    }

    #[test]
    fn sprite_main_host_return_after_active_cucco_y_subpixel_keeps_coordinate_suffix_pending() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        source
            .consume_event(
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(2), None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(ACTIVE_CUCCO_MOVEMENT_CALL_PC), Some(2), None),
                &mut receipts,
            )
            .unwrap();
        for address in [
            SPRITE_X_SUBPIXEL_BASE + 2,
            SPRITE_X_LOW_BASE + 2,
            SPRITE_X_HIGH_BASE + 2,
        ] {
            source
                .consume_event(
                    raw("wram-write", Some(0x06_e94e), Some(0x12), Some(address)),
                    &mut receipts,
                )
                .unwrap();
        }
        source
            .consume_event(
                raw(
                    "wram-write",
                    Some(0x06_e94e),
                    Some(0x12),
                    Some(SPRITE_Y_SUBPIXEL_BASE + 2),
                ),
                &mut receipts,
            )
            .unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterActiveCuccoYSubpixel {
                    slot: 2,
                    helper_ordinal: 0,
                },
            )],
        );
    }

    #[test]
    fn sprite_main_nmi_after_three_shared_cucco_increments_keeps_the_helper_pending() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        source
            .consume_event(
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(5), None),
                &mut receipts,
            )
            .unwrap();
        for pc in CUCCO_SUBTYPE_INCREMENT_PUBLICATION_PCS.iter().take(3) {
            source
                .consume_event(
                    raw(
                        "wram-write",
                        Some(*pc),
                        Some(5),
                        Some(SPRITE_SUBTYPE2_BASE + 5),
                    ),
                    &mut receipts,
                )
                .unwrap();
        }
        source
            .consume_event(raw("nmi", Some(0x06_a6eb), Some(5), None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterCuccoSubtypeIncrements {
                        slot: 5,
                        helper_ordinal: 0,
                        completed: 3,
                    },
                ),
            ],
        );
    }

    #[test]
    fn sprite_main_nmi_before_the_first_slot_is_not_a_cucco_publication() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        source
            .consume_event(
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(raw("nmi", Some(0x06_f80f), None, None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainBeforeFirstSlot,
                ),
            ],
        );
    }

    fn raw(event: &str, pc: Option<u32>, x: Option<u16>, address: Option<u16>) -> RawTraceEvent {
        let pc = pc.or_else(|| matches!(event, "nmi" | "nmi-resume").then_some(0x008000));
        RawTraceEvent {
            event: event.to_string(),
            stage: None,
            run: None,
            pc,
            s: matches!(event, "nmi" | "nmi-resume").then_some(0x01ff),
            return_address: None,
            a: None,
            main: None,
            sub: None,
            subsub: None,
            frame_counter: None,
            nmi_latch: matches!(event, "nmi").then_some(0),
            link_y: None,
            bg2_v: None,
            spotlight_radius: None,
            spotlight_var4_low: None,
            spotlight_lower_cursor: None,
            joypad_high: None,
            joypad_low: None,
            joypad_high_filtered: None,
            joypad_low_filtered: None,
            x,
            y: None,
            address,
            value: address.map(|_| 0),
            nmi_ppu_register_operands: matches!(event, "nmi").then_some([0; 31]),
        }
    }

    fn raw_at(event: &str, pc: u32, s: u16) -> RawTraceEvent {
        let mut event = raw(event, Some(pc), None, None);
        event.s = Some(s);
        if event.event == "pc" && NMI_HANDLER_COMPLETE_PCS.contains(&(pc & 0x00ff_ffff)) {
            event.joypad_high = Some(0);
            event.joypad_low = Some(0);
            event.joypad_high_filtered = Some(0);
            event.joypad_low_filtered = Some(0);
        }
        event
    }

    fn empty_semantic_tracker() -> Snes9xOracleSemanticTrace {
        Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        }
    }

    fn publish_nmi(
        tracker: &mut Snes9xOracleSemanticTrace,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    ) {
        let mut completion = raw_at("pc", NMI_HANDLER_COMPLETE_PCS[0], 0x01f3);
        completion.joypad_high = Some(0);
        completion.joypad_low = Some(0);
        completion.joypad_high_filtered = Some(0);
        completion.joypad_low_filtered = Some(0);
        tracker.consume_event(completion, receipts).unwrap();
    }

    fn zero_joypad_publication() -> OriginalTimingSemanticReceipt {
        OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
            high: 0,
            low: 0,
            high_filtered: 0,
            low_filtered: 0,
        })
    }

    fn frame(stage: &str, run: u64, main: u8, frame_counter: u8) -> RawTraceEvent {
        RawTraceEvent {
            event: "frame".to_string(),
            stage: Some(stage.to_string()),
            run: Some(run),
            pc: Some(0),
            s: Some(0x01ff),
            return_address: None,
            a: None,
            main: Some(main),
            sub: Some(0),
            subsub: Some(0),
            frame_counter: Some(frame_counter),
            nmi_latch: Some(0),
            link_y: None,
            bg2_v: None,
            spotlight_radius: None,
            spotlight_var4_low: None,
            spotlight_lower_cursor: None,
            joypad_high: None,
            joypad_low: None,
            joypad_high_filtered: None,
            joypad_low_filtered: None,
            x: None,
            y: None,
            address: None,
            value: None,
            nmi_ppu_register_operands: None,
        }
    }

    fn frame_with_sub(stage: &str, run: u64, main: u8, sub: u8) -> RawTraceEvent {
        let mut event = frame(stage, run, main, 0);
        event.sub = Some(sub);
        event
    }

    fn main_loop_start() -> RawTraceEvent {
        raw(
            "wram-write",
            Some(ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC),
            None,
            Some(FRAME_COUNTER),
        )
    }

    fn main_loop_common_suffix_completion() -> RawTraceEvent {
        let mut event = raw(
            "wram-write",
            Some(ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC),
            None,
            Some(NMI_UPDATE_LATCH),
        );
        event.value = Some(0);
        event
    }

    fn write_semantic_trace(path: &Path, events: &[serde_json::Value]) {
        fs::write(
            path,
            events
                .iter()
                .map(serde_json::Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
                + "\n",
        )
        .unwrap();
    }

    #[test]
    fn cold_boot_without_a_zelda_run_game_loop_start_emits_no_progress() {
        let path = env::temp_dir().join(format!(
            "zelda3-snes9x-cold-bootstrap-no-main-progress-{}.jsonl",
            std::process::id()
        ));
        write_semantic_trace(
            &path,
            &[
                serde_json::json!({
                    "event": "frame", "stage": "entry", "run": 0,
                    "pc": 0x008000, "s": 0x01ff, "main": 0x55,
                    "sub": 0x55, "subsub": 0x55, "frame_counter": 0x55,
                    "nmi_latch": 0x55
                }),
                serde_json::json!({
                    "event": "frame", "stage": "return", "run": 0,
                    "pc": 0x0088b3, "s": 0x01fa, "main": 0x55,
                    "sub": 0x55, "subsub": 0x55, "frame_counter": 0x55,
                    "nmi_latch": 0x55
                }),
            ],
        );
        let mut tracker = empty_semantic_tracker();
        tracker.path = path.clone();

        let receipts = tracker.read_after_host_call(None, None, None).unwrap();
        fs::remove_file(path).unwrap();

        assert!(receipts.is_empty());
        assert!(!tracker.zelda_run_game_loop_call_active);
    }

    #[test]
    fn main_loop_call_ownership_survives_checkpoint_into_continued_suffix_host() {
        let start_path = env::temp_dir().join(format!(
            "zelda3-snes9x-main-start-before-checkpoint-{}.jsonl",
            std::process::id()
        ));
        write_semantic_trace(
            &start_path,
            &[
                serde_json::json!({
                    "event": "frame", "stage": "entry", "run": 81,
                    "pc": 0x008034, "s": 0x01ff, "main": 0,
                    "sub": 0, "subsub": 0, "frame_counter": 0,
                    "nmi_latch": 0
                }),
                serde_json::json!({
                    "event": "wram-write", "run": 81,
                    "pc": ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC,
                    "s": 0x01ff, "address": FRAME_COUNTER, "value": 1
                }),
                serde_json::json!({
                    "event": "frame", "stage": "return", "run": 81,
                    "pc": 0x0c_c1db, "s": 0x01f4, "main": 0,
                    "sub": 1, "subsub": 0, "frame_counter": 1,
                    "nmi_latch": 1
                }),
            ],
        );
        let mut source = empty_semantic_tracker();
        source.path = start_path.clone();
        assert_eq!(
            source.read_after_host_call(None, None, None).unwrap(),
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                MainLoopProgress::IterationStarted,
            )],
        );
        fs::remove_file(start_path).unwrap();
        assert!(source.zelda_run_game_loop_call_active);

        let checkpoint: Snes9xOracleSemanticTraceCheckpoint =
            serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
        let suffix_path = env::temp_dir().join(format!(
            "zelda3-snes9x-main-suffix-after-checkpoint-{}.jsonl",
            std::process::id()
        ));
        write_semantic_trace(
            &suffix_path,
            &[
                serde_json::json!({
                    "event": "frame", "stage": "entry", "run": 82,
                    "pc": 0x0c_c1db, "s": 0x01f4, "main": 0,
                    "sub": 1, "subsub": 0, "frame_counter": 1,
                    "nmi_latch": 1
                }),
                serde_json::json!({
                    "event": "nmi", "run": 82, "pc": 0x0c_c1df,
                    "s": 0x01f4, "main": 0, "sub": 1, "nmi_latch": 1,
                    "nmi_ppu_register_operands": vec![0; 31]
                }),
                serde_json::json!({
                    "event": "pc", "run": 82,
                    "pc": NMI_HANDLER_COMPLETE_PCS[0], "s": 0x01e8
                }),
                serde_json::json!({
                    "event": "nmi-resume", "run": 82,
                    "pc": 0x0c_c1df, "s": 0x01f4
                }),
                serde_json::json!({
                    "event": "wram-write", "run": 82,
                    "pc": ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC,
                    "s": 0x01ff, "address": NMI_UPDATE_LATCH, "value": 0
                }),
                serde_json::json!({
                    "event": "frame", "stage": "return", "run": 82,
                    "pc": 0x008034, "s": 0x01ff, "main": 0,
                    "sub": 1, "subsub": 0, "frame_counter": 1,
                    "nmi_latch": 0
                }),
            ],
        );
        let mut resumed = empty_semantic_tracker();
        resumed.path = suffix_path.clone();
        resumed.restore_checkpoint(checkpoint).unwrap();

        let receipts = resumed.read_after_host_call(None, None, None).unwrap();
        fs::remove_file(suffix_path).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        );
        assert!(!resumed.zelda_run_game_loop_call_active);
    }

    #[test]
    fn main_loop_call_ownership_rejects_double_start_and_orphan_suffix() {
        for (active, event, message) in [
            (
                true,
                main_loop_start(),
                "before the prior call completed its common suffix",
            ),
            (
                false,
                main_loop_common_suffix_completion(),
                "without an active source call",
            ),
        ] {
            let path = env::temp_dir().join(format!(
                "zelda3-snes9x-invalid-main-loop-transition-{}-{active}.jsonl",
                std::process::id()
            ));
            let mut entry = frame("entry", 1, 0, 0);
            entry.pc = Some(0x008034);
            let mut returned = frame("return", 1, 0, 0);
            returned.pc = Some(0x008034);
            write_semantic_trace(
                &path,
                &[
                    serde_json::to_value(entry).unwrap(),
                    serde_json::to_value(event).unwrap(),
                    serde_json::to_value(returned).unwrap(),
                ],
            );
            let mut tracker = empty_semantic_tracker();
            tracker.path = path.clone();
            tracker.zelda_run_game_loop_call_active = active;

            let error = tracker.read_after_host_call(None, None, None).unwrap_err();
            fs::remove_file(path).unwrap();

            assert!(error.contains(message), "unexpected error: {error}");
        }
    }

    #[test]
    fn completed_old_call_then_new_start_preserves_both_ordered_progress_facts() {
        let path = env::temp_dir().join(format!(
            "zelda3-snes9x-main-suffix-then-new-start-{}.jsonl",
            std::process::id()
        ));
        write_semantic_trace(
            &path,
            &[
                serde_json::to_value(frame("entry", 3, 0, 1)).unwrap(),
                serde_json::to_value(main_loop_common_suffix_completion()).unwrap(),
                serde_json::to_value(main_loop_start()).unwrap(),
                serde_json::to_value(frame("return", 3, 0, 2)).unwrap(),
            ],
        );
        let mut tracker = empty_semantic_tracker();
        tracker.path = path.clone();
        tracker.zelda_run_game_loop_call_active = true;

        let receipts = tracker.read_after_host_call(None, None, None).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted),
            ],
        );
        assert!(tracker.zelda_run_game_loop_call_active);
    }

    #[test]
    fn sprite_item_receipt_call_progress_hides_cpu_provenance_and_preserves_call_order() {
        let path = env::temp_dir().join("unused-snes9x-item-receipt-progress-test.jsonl");
        let mut tracker = Snes9xOracleSemanticTrace {
            path,
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();

        tracker
            .consume_event(
                raw(
                    "pc",
                    // The pinned cold trace executes the symbol's $05 LoROM
                    // mirror even though source listings commonly print $85.
                    Some(0x05eb1d),
                    Some(12),
                    None,
                ),
                &mut receipts,
            )
            .unwrap();
        tracker.flush_item_receipt_progress(&mut receipts);
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 12 },
                    progress: SourceCallProgress::Suspended,
                }
            )],
        );

        receipts.clear();
        tracker.flush_item_receipt_progress(&mut receipts);
        assert_eq!(
            receipts[0],
            OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 12 },
                    progress: SourceCallProgress::Suspended,
                }
            ),
        );

        receipts.clear();
        tracker
            .consume_event(raw("pc", Some(0x05eb21), None, None), &mut receipts)
            .unwrap();
        tracker.flush_item_receipt_progress(&mut receipts);
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 12 },
                    progress: SourceCallProgress::Returned,
                }
            )],
        );

        receipts.clear();
        tracker
            .consume_event(
                raw("pc", Some(SICK_KID_ITEM_RECEIPT_CALL_PC), Some(3), None),
                &mut receipts,
            )
            .unwrap();
        tracker.flush_item_receipt_progress(&mut receipts);
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 3 },
                    progress: SourceCallProgress::Suspended,
                }
            )],
        );

        receipts.clear();
        tracker
            .consume_event(
                raw("pc", Some(SICK_KID_ITEM_RECEIPT_RETURN_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 3 },
                    progress: SourceCallProgress::Returned,
                }
            )],
        );

        receipts.clear();
        tracker
            .consume_event(
                raw(
                    "pc",
                    Some(UNCLE_PASSAGE_ITEM_RECEIPT_CALL_PC),
                    Some(1),
                    None,
                ),
                &mut receipts,
            )
            .unwrap();
        tracker.flush_item_receipt_progress(&mut receipts);
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::UnclePassage { slot: 1 },
                    progress: SourceCallProgress::Suspended,
                }
            )],
        );

        receipts.clear();
        tracker
            .consume_event(
                raw("pc", Some(UNCLE_PASSAGE_ITEM_RECEIPT_RETURN_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::UnclePassage { slot: 1 },
                    progress: SourceCallProgress::Returned,
                }
            )],
        );
    }

    #[test]
    fn direct_sprite_item_graphics_return_is_distinct_from_outer_sprite_main_return() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for (pc, x) in [
            (SPRITE_MAIN_ENTRY_PC, None),
            (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(3)),
            (LINK_RECEIVE_ITEM_ENTRY_PC, None),
        ] {
            tracker
                .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
                .unwrap();
        }
        tracker.flush_item_receipt_progress(&mut receipts);
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                ItemReceiptGraphicsProgressReceipt {
                    caller: ItemReceiptGraphicsCaller::SpriteMainDirect { slot: 3 },
                    progress: SourceCallProgress::Suspended,
                },
            )],
        );

        receipts.clear();
        tracker
            .consume_event(
                raw("pc", Some(LINK_RECEIVE_ITEM_GRAPHICS_RETURN_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        tracker
            .consume_event(
                raw("pc", Some(SPRITE_SLOT_RETURN_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        for slot in (0..3).rev() {
            tracker
                .consume_event(
                    raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(slot), None),
                    &mut receipts,
                )
                .unwrap();
            tracker
                .consume_event(
                    raw("pc", Some(SPRITE_SLOT_RETURN_PC), None, None),
                    &mut receipts,
                )
                .unwrap();
        }

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                    ItemReceiptGraphicsProgressReceipt {
                        caller: ItemReceiptGraphicsCaller::SpriteMainDirect { slot: 3 },
                        progress: SourceCallProgress::Returned,
                    },
                ),
                OriginalTimingSemanticReceipt::SpriteMainReturned,
            ],
        );
        assert!(tracker.item_receipt_caller.is_none());
        assert!(tracker.sprite_main_execution.is_none());
    }

    fn save_menu_frame(stage: &str, run: u64, subsub: u8, frame_counter: u8) -> RawTraceEvent {
        let mut event = frame_with_sub(stage, run, 14, 11);
        event.subsub = Some(subsub);
        event.frame_counter = Some(frame_counter);
        event
    }

    #[test]
    fn save_menu_initialization_reports_source_call_progress_without_cpu_state() {
        let cases = [
            (
                save_menu_frame("entry", 11616, 0, 13),
                save_menu_frame("return", 11616, 0, 14),
                MainLoopProgress::IterationStarted,
                SaveMenuInitializationProgress::InProgress,
            ),
            (
                save_menu_frame("entry", 11621, 0, 14),
                save_menu_frame("return", 11621, 1, 14),
                MainLoopProgress::CallStackContinued,
                SaveMenuInitializationProgress::Completed,
            ),
        ];

        for (entry, returned, main_progress, save_progress) in cases {
            let mut host = HostFrameWindow::default();
            host.observe(&entry).unwrap();
            if main_progress == MainLoopProgress::IterationStarted {
                host.observe(&main_loop_start()).unwrap();
            }
            host.observe(&returned).unwrap();
            let mut receipts = Vec::new();
            host.finish(
                &mut receipts,
                None,
                main_progress == MainLoopProgress::CallStackContinued,
            )
            .unwrap();
            assert_eq!(
                receipts,
                vec![
                    OriginalTimingSemanticReceipt::MainLoopProgress(main_progress),
                    OriginalTimingSemanticReceipt::SaveMenuInitializationProgress(save_progress),
                ]
            );
        }
    }

    #[test]
    fn main_wait_return_without_exact_common_suffix_does_not_claim_completion() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 1923, 14, 3);
        entry.pc = Some(0x00_8034);
        entry.frame_counter = Some(220);
        let mut returned = frame_with_sub("return", 1923, 14, 3);
        returned.pc = Some(0x00_8034);
        returned.frame_counter = Some(221);

        host.observe(&entry).unwrap();
        host.observe(&main_loop_start()).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, false).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                MainLoopProgress::IterationStarted,
            )],
        );
    }

    #[test]
    fn main_wait_nmi_without_exact_common_suffix_does_not_claim_completion() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 326, 14, 3);
        entry.pc = Some(0x00_80c9);
        entry.frame_counter = Some(223);
        let mut accepted = raw("nmi", Some(0x00_8036), None, None);
        accepted.run = Some(326);
        let mut returned = frame_with_sub("return", 326, 14, 3);
        returned.pc = Some(0x00_80c9);
        returned.frame_counter = Some(224);

        host.observe(&entry).unwrap();
        host.observe(&main_loop_start()).unwrap();
        host.observe(&accepted).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, false).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                MainLoopProgress::IterationStarted,
            )],
        );
    }

    #[test]
    fn continued_wait_return_without_exact_common_suffix_does_not_claim_completion() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 1059, 4, 1);
        entry.pc = Some(0x0c_ce3a);
        entry.nmi_latch = Some(1);
        let mut accepted = raw("nmi", Some(0x0c_ce3c), None, None);
        accepted.run = Some(1059);
        let mut returned = frame_with_sub("return", 1059, 4, 2);
        returned.pc = Some(0x00_8034);
        returned.nmi_latch = Some(0);

        host.observe(&entry).unwrap();
        host.observe(&accepted).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                MainLoopProgress::CallStackContinued,
            )],
        );
    }

    #[test]
    fn continued_wait_nmi_without_exact_common_suffix_does_not_claim_completion() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 84, 0, 1);
        entry.pc = Some(0x0c_c1d3);
        entry.nmi_latch = Some(1);
        let mut interrupted = raw("nmi", Some(0x0c_c1d7), None, None);
        interrupted.run = Some(84);
        interrupted.nmi_latch = Some(1);
        let mut following = raw("nmi", Some(0x00_8034), None, None);
        following.run = Some(84);
        following.nmi_latch = Some(0);
        let mut returned = frame_with_sub("return", 84, 0, 1);
        returned.pc = Some(0x00_80c9);
        returned.nmi_latch = Some(0);

        assert_eq!(host.observe(&entry).unwrap(), None);
        assert_eq!(host.observe(&interrupted).unwrap(), None);
        let mut receipts = vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::NmiHandlerCompleted,
        ];
        assert_eq!(host.observe(&following).unwrap(), None);
        receipts.push(OriginalTimingSemanticReceipt::NmiAccepted(
            NmiUpdateGate::Open,
        ));
        assert_eq!(host.observe(&returned).unwrap(), None);
        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
            ],
        );
    }

    #[test]
    fn continued_iteration_still_inside_module_omits_terminal_completion() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 1058, 4, 1);
        entry.pc = Some(0x0c_ce38);
        entry.nmi_latch = Some(1);
        let mut returned = frame_with_sub("return", 1058, 4, 1);
        returned.pc = Some(0x0c_ce3a);
        returned.nmi_latch = Some(1);

        host.observe(&entry).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                MainLoopProgress::CallStackContinued,
            )],
        );
    }

    #[test]
    fn leading_nmi_at_main_wait_does_not_complete_the_later_iteration() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 327, 14, 3);
        entry.pc = Some(0x00_8034);
        let leading = raw("nmi", Some(0x00_8036), None, None);
        let mut returned = frame_with_sub("return", 327, 14, 3);
        returned.pc = Some(LINK_OAM_START_PC);

        host.observe(&entry).unwrap();
        host.observe(&leading).unwrap();
        host.observe(&main_loop_start()).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, false).unwrap();

        assert!(
            !receipts.contains(&OriginalTimingSemanticReceipt::MainLoopIterationReturnedToWait,)
        );
    }

    #[test]
    fn unchanged_dialogue_iteration_with_vwf_nmi_becomes_a_semantic_hold() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame("entry", 36018, 14, 0x35)).unwrap();
        host.observe(&raw(
            "nmi",
            Some(VWF_RENDER_SINGLE_START_PC + 0x11),
            None,
            None,
        ))
        .unwrap();
        host.observe(&frame("return", 36018, 14, 0x35)).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, Some(0x0037), true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                    DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                        message_read_position: 0x0037,
                    },
                ),
            ],
        );
    }

    #[test]
    fn dialogue_caller_return_pc_outside_vwf_range_omits_semantic_hold() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame("entry", 2333, 14, 0x8f)).unwrap();
        host.observe(&raw("nmi", Some(0x0e_c58b), None, None))
            .unwrap();
        host.observe(&frame("return", 2333, 14, 0x8f)).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, Some(0x0037), true).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                MainLoopProgress::CallStackContinued,
            )],
            "$0e:c58b is Text_LoadCharacterBuffer's caller, not the VWF render loop",
        );
    }

    #[test]
    fn dialogue_terminal_common_suffix_keeps_progress_without_vwf_endpoint() {
        let path = env::temp_dir().join(format!(
            "zelda3-snes9x-dialogue-terminal-common-suffix-{}.jsonl",
            std::process::id()
        ));
        let events = [
            serde_json::json!({
                "event": "frame", "stage": "entry", "run": 2334,
                "pc": NMI_HANDLER_ENTRY_PC, "s": 0x01ee, "main": 14,
                "sub": 2, "subsub": 0, "frame_counter": 0x8f,
                "nmi_latch": 1
            }),
            serde_json::json!({
                "event": "pc", "run": 2334,
                "pc": NMI_HANDLER_COMPLETE_PCS[0], "s": 0x01e5
            }),
            serde_json::json!({
                "event": "nmi-resume", "run": 2334,
                "pc": 0x0e_c58b, "s": 0x01f2
            }),
            serde_json::json!({
                "event": "wram-write", "run": 2334,
                "pc": ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC,
                "s": 0x01ff, "address": NMI_UPDATE_LATCH, "value": 0
            }),
            serde_json::json!({
                "event": "frame", "stage": "return", "run": 2334,
                "pc": 0x00_8034, "s": 0x01ff, "main": 14, "sub": 2,
                "subsub": 0, "frame_counter": 0x8f, "nmi_latch": 0
            }),
        ];
        fs::write(
            &path,
            events
                .iter()
                .map(serde_json::Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
                + "\n",
        )
        .unwrap();
        let mut tracker = empty_semantic_tracker();
        tracker.path = path.clone();
        tracker.zelda_run_game_loop_call_active = true;
        tracker.nmi_publication_pending = true;
        tracker.pending_nmi_update_gate = Some(NmiUpdateGate::LatchHeld);
        tracker.nmi_resume_targets.push((0x0e_c58b, 0x01f2));

        let receipts = tracker
            .read_after_host_call(Some(0x0037), None, None)
            .unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        );
        assert!(!tracker.nmi_publication_pending);
        assert!(tracker.pending_nmi_update_gate.is_none());
        assert!(tracker.nmi_resume_targets.is_empty());
    }

    #[test]
    fn pre_overworld_register_transitions_become_source_stage_receipts() {
        let cases = [
            (8, 0, 8, 1, PreOverworldStageCompletion::PropertiesReturned),
            (8, 1, 8, 2, PreOverworldStageCompletion::OverlaysReturned),
            (
                8,
                2,
                16,
                0,
                PreOverworldStageCompletion::ScreenBuildReturned,
            ),
        ];
        for (entry_main, entry_sub, return_main, return_sub, expected) in cases {
            let mut host = HostFrameWindow::default();
            host.observe(&frame_with_sub("entry", 7, entry_main, entry_sub))
                .unwrap();
            host.observe(&frame_with_sub("return", 7, return_main, return_sub))
                .unwrap();
            let mut receipts = Vec::new();
            host.finish(&mut receipts, None, true).unwrap();
            assert_eq!(
                receipts,
                vec![
                    OriginalTimingSemanticReceipt::MainLoopProgress(
                        MainLoopProgress::CallStackContinued,
                    ),
                    OriginalTimingSemanticReceipt::PreOverworldStageCompleted(expected),
                ]
            );
        }
    }

    #[test]
    fn overworld_sprite_reload_return_becomes_a_backend_neutral_completion_receipt() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 38517, 9, 4)).unwrap();
        host.observe(&frame_with_sub("return", 38517, 9, 5))
            .unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::ReloadReturned,
                ),
            ],
        );
    }

    #[test]
    fn overworld_map_quadrant_publication_becomes_a_backend_neutral_receipt() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 39261, 9, 3)).unwrap();
        host.observe(&frame_with_sub("return", 39261, 9, 4))
            .unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::OverworldMapQuadrantsPublished,
            ],
        );
    }

    #[test]
    fn world_map_ambient_map8_return_becomes_a_backend_neutral_receipt() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 6173, 9, 0x21))
            .unwrap();
        host.observe(&frame_with_sub("return", 6173, 9, 0x22))
            .unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::WorldMapAmbientMap8Returned,
            ],
        );
    }

    #[test]
    fn world_map_overlay_reload_return_becomes_a_backend_neutral_receipt() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 6168, 9, 0x20))
            .unwrap();
        host.observe(&frame_with_sub("return", 6168, 9, 0x21))
            .unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::WorldMapOverlayReloadReturned,
            ],
        );
    }

    #[test]
    fn dungeon_exit_spotlight_entry_return_becomes_a_backend_neutral_receipt() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 39630, 15, 0))
            .unwrap();
        host.observe(&frame_with_sub("return", 39630, 15, 1))
            .unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned,
            ],
        );
    }

    #[test]
    fn recurring_spotlight_caller_return_to_main_wait_is_a_semantic_receipt() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 4789, 15, 1);
        entry.pc = Some(NMI_HANDLER_ENTRY_PC);
        let mut returned = frame_with_sub("return", 4789, 15, 1);
        returned.pc = Some(0x00_8036);
        host.observe(&entry).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
            ],
        );

        let mut idle = HostFrameWindow::default();
        let mut idle_entry = frame_with_sub("entry", 4790, 15, 1);
        idle_entry.pc = Some(0x00_8036);
        let mut idle_return = frame_with_sub("return", 4790, 15, 1);
        idle_return.pc = Some(0x00_8036);
        idle.observe(&idle_entry).unwrap();
        idle.observe(&idle_return).unwrap();
        let mut idle_receipts = Vec::new();
        idle.finish(&mut idle_receipts, None, false).unwrap();
        assert!(
            idle_receipts.is_empty(),
            "an idle main-wait host invented a suspended C caller",
        );
    }

    #[test]
    fn recurring_spotlight_caller_return_before_following_nmi_is_a_semantic_receipt() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 39634, 15, 1);
        entry.pc = Some(0x00_f3bf);
        entry.nmi_latch = Some(1);
        let mut returned = frame_with_sub("return", 39634, 15, 1);
        returned.pc = Some(NMI_HANDLER_ENTRY_PC);
        returned.nmi_latch = Some(0);
        host.observe(&entry).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
            ],
            "ZeldaRunGameLoop clears nmi_boolean only after Module0F and NMI_PrepareSprites return",
        );

        let mut interrupted = HostFrameWindow::default();
        let mut interrupted_entry = frame_with_sub("entry", 39635, 15, 1);
        interrupted_entry.pc = Some(0x00_f3bf);
        interrupted_entry.nmi_latch = Some(1);
        let mut interrupted_return = frame_with_sub("return", 39635, 15, 1);
        interrupted_return.pc = Some(NMI_HANDLER_ENTRY_PC);
        interrupted_return.nmi_latch = Some(1);
        interrupted.observe(&interrupted_entry).unwrap();
        interrupted.observe(&interrupted_return).unwrap();
        let mut interrupted_receipts = Vec::new();
        interrupted
            .finish(&mut interrupted_receipts, None, true)
            .unwrap();
        assert_eq!(
            interrupted_receipts,
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                MainLoopProgress::CallStackContinued,
            )],
            "an NMI accepted while Module0F remains active cannot fabricate a caller return",
        );
    }

    #[test]
    fn overworld_sprite_writes_become_backend_neutral_publication_receipts() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();
        let slot = 13u8;
        let writes = [
            (SPRITE_N_WORD_BASE + u16::from(slot) * 2, 0x98),
            (SPRITE_N_WORD_BASE + u16::from(slot) * 2 + 1, 0x01),
            (SPRITE_TYPE_BASE + u16::from(slot), 0xac),
            (SPRITE_STATE_BASE + u16::from(slot), 8),
            (SPRITE_DIE_ACTION_BASE + u16::from(slot), 0),
        ];
        for (index, (address, value)) in writes.into_iter().enumerate() {
            let mut event = raw(
                "wram-write",
                Some(OVERWORLD_LOAD_SINGLE_SPRITE_START_PC + 1),
                Some(if index < 2 {
                    u16::from(slot) * 2
                } else {
                    u16::from(slot)
                }),
                Some(address),
            );
            event.main = Some(8);
            event.sub = Some(0);
            event.value = Some(value);
            tracker.consume_event(event, &mut receipts).unwrap();
        }

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::PresencePublished,
                ),
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::SpriteActivated {
                        block: 0x0198,
                        slot,
                        sprite_type: 0xac,
                    },
                ),
            ],
        );
    }

    #[test]
    fn nmi_inside_overworld_sprite_scan_publishes_presence_once() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();
        for _ in 0..2 {
            let mut event = raw("nmi", Some(OVERWORLD_SPRITE_SCAN_START_PC + 1), None, None);
            event.main = Some(8);
            event.sub = Some(0);
            tracker.consume_event(event, &mut receipts).unwrap();
            publish_nmi(&mut tracker, &mut receipts);
        }

        assert_eq!(
            receipts
                .iter()
                .filter(|receipt| matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                        OverworldSpriteReloadProgress::PresencePublished
                    )
                ))
                .count(),
            1,
        );
        assert_eq!(
            receipts
                .iter()
                .filter(|receipt| matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open)
                ))
                .count(),
            2,
        );
    }

    #[test]
    fn pre_dungeon_return_is_distinct_from_the_next_main_iteration() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 49414, 6, 0);
        entry.frame_counter = Some(218);
        let mut returned = frame_with_sub("return", 49414, 7, 15);
        returned.frame_counter = Some(218);
        host.observe(&entry).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::PreDungeonModuleReturned,
            ],
        );
    }

    #[test]
    fn dialogue_iteration_that_advanced_is_not_reported_as_a_resumed_hold() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame("entry", 36014, 14, 0x34)).unwrap();
        host.observe(&main_loop_start()).unwrap();
        host.observe(&raw("nmi", Some(VWF_RENDER_SINGLE_END_PC - 1), None, None))
            .unwrap();
        host.observe(&frame("return", 36014, 14, 0x35)).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, false).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                MainLoopProgress::IterationStarted,
            )],
        );
    }

    #[test]
    fn dialogue_return_to_gameplay_becomes_a_backend_neutral_close_receipt() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 9, 14, 2)).unwrap();
        host.observe(&main_loop_start()).unwrap();
        let mut returned = frame_with_sub("return", 9, 9, 0);
        returned.frame_counter = Some(1);
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, false).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted,),
                OriginalTimingSemanticReceipt::DialogueClosed,
            ],
        );
    }

    #[test]
    fn host_interval_rejects_multiple_main_loop_starts() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame("entry", 7, 9, 0xfe)).unwrap();
        host.observe(&main_loop_start()).unwrap();
        host.observe(&main_loop_start()).unwrap();
        host.observe(&frame("return", 7, 9, 0x00)).unwrap();
        let mut receipts = Vec::new();

        assert_eq!(
            host.finish(&mut receipts, None, false).unwrap_err(),
            "Snes9x host call started ZeldaRunGameLoop 2 times; expected zero or one",
        );
        assert!(receipts.is_empty());
    }

    #[test]
    fn cold_initialization_frame_counter_clear_is_not_a_main_loop_start() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame("entry", 80, 0x55, 0x55);
        entry.pc = Some(0x0087cb);
        entry.nmi_latch = Some(0x55);
        host.observe(&entry).unwrap();
        host.observe(&raw(
            "wram-write",
            Some(0x0087ce),
            None,
            Some(FRAME_COUNTER),
        ))
        .unwrap();
        let mut returned = frame("return", 80, 0, 0);
        returned.pc = Some(0x008034);
        returned.nmi_latch = Some(0);
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, false).unwrap();

        assert!(receipts.is_empty());
    }

    #[test]
    fn nmi_inside_common_sprite_preparation_becomes_a_backend_neutral_receipt() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw("nmi", Some(0x00_8751), None, None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpritePreparation,
                ),
            ],
        );
    }

    #[test]
    fn nmi_inside_extended_oam_pack_exports_only_the_resumable_source_cursor() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("nmi", Some(0x00_860f), None, None);
        event.y = Some(4);
        event.x = Some(16);
        event.nmi_latch = Some(1);

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpritePreparationExtendedOamPacking {
                        next_group_start: 4,
                    },
                ),
            ],
        );
    }

    #[test]
    fn extended_oam_first_store_opcode_is_still_an_unpublished_group_boundary() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("nmi", Some(0x00_8614), None, None);
        event.y = Some(4);
        event.x = Some(16);
        event.nmi_latch = Some(1);

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpritePreparationExtendedOamPacking {
                        next_group_start: 4,
                    },
                ),
            ],
        );
    }

    #[test]
    fn extended_oam_pack_receipt_rejects_a_missing_cursor_before_mutation() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("nmi", Some(0x00_860f), None, None);
        event.x = Some(16);
        event.nmi_latch = Some(1);

        let error = tracker.consume_event(event, &mut receipts).unwrap_err();

        assert!(error.contains("omitted source cursor Y"));
        assert!(receipts.is_empty());
        assert!(!tracker.nmi_publication_pending);
        assert_eq!(tracker.pending_nmi_update_gate, None);
        assert!(tracker.nmi_resume_targets.is_empty());
    }

    #[test]
    fn extended_oam_pack_receipt_rejects_an_invalid_cursor_before_mutation() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("nmi", Some(0x00_860f), None, None);
        event.y = Some(6);
        event.x = Some(24);
        event.nmi_latch = Some(1);

        let error = tracker.consume_event(event, &mut receipts).unwrap_err();

        assert!(error.contains("invalid group cursor 6"));
        assert!(receipts.is_empty());
        assert!(!tracker.nmi_publication_pending);
        assert_eq!(tracker.pending_nmi_update_gate, None);
        assert!(tracker.nmi_resume_targets.is_empty());
    }

    #[test]
    fn extended_oam_pack_receipt_rejects_disagreeing_source_cursors_before_mutation() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("nmi", Some(0x00_860f), None, None);
        event.y = Some(4);
        event.x = Some(12);
        event.nmi_latch = Some(1);

        let error = tracker.consume_event(event, &mut receipts).unwrap_err();

        assert!(error.contains("cursors disagreed: y=4, x=12"));
        assert!(receipts.is_empty());
        assert!(!tracker.nmi_publication_pending);
        assert_eq!(tracker.pending_nmi_update_gate, None);
        assert!(tracker.nmi_resume_targets.is_empty());
    }

    #[test]
    fn extended_oam_pack_receipt_requires_a_held_latch_before_mutation() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("nmi", Some(0x00_860f), None, None);
        event.y = Some(4);
        event.x = Some(16);
        event.nmi_latch = Some(0);

        let error = tracker.consume_event(event, &mut receipts).unwrap_err();

        assert!(error.contains("observed an open Zelda NMI latch"));
        assert!(receipts.is_empty());
        assert!(!tracker.nmi_publication_pending);
        assert_eq!(tracker.pending_nmi_update_gate, None);
        assert!(tracker.nmi_resume_targets.is_empty());
    }

    #[test]
    fn shared_jump_table_inside_the_symbol_gap_keeps_the_active_source_receipt() {
        let mut tracker = empty_semantic_tracker();
        tracker.sprite_main_execution = Some(SpriteMainExecutionTracker {
            current_slot: Some(0),
            last_completed_slot: Some(1),
            cucco_subtype_increments: None,
            cucco_helper_ordinal: 0,
            cucco_flee_movement: None,
            active_cucco_movement: None,
            active_cucco_x_publications: 0,
            active_cucco_y_subpixel: None,
            cucco_animation_slot: None,
            big_key_drop_graphics_slot: None,
        });
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw("nmi", Some(0x00_8799), Some(0), None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterSlot(1),
                ),
            ],
        );
    }

    #[test]
    fn nmi_inside_caller_specific_item_graphics_keeps_the_current_sprite_slot_pending() {
        let mut tracker = empty_semantic_tracker();
        tracker.sprite_main_execution = Some(SpriteMainExecutionTracker {
            current_slot: Some(12),
            last_completed_slot: Some(13),
            cucco_subtype_increments: None,
            cucco_helper_ordinal: 0,
            cucco_flee_movement: None,
            active_cucco_movement: None,
            active_cucco_x_publications: 0,
            active_cucco_y_subpixel: None,
            cucco_animation_slot: None,
            big_key_drop_graphics_slot: None,
        });
        tracker.item_receipt_caller = Some(ItemReceiptGraphicsCaller::SpriteMain { slot: 12 });
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw("nmi", Some(0x07_99f0), Some(12), None), &mut receipts)
            .unwrap();
        tracker.flush_item_receipt_progress(&mut receipts);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainItemReceiptGraphicsStarted(12),
                ),
                OriginalTimingSemanticReceipt::ItemReceiptGraphicsProgress(
                    ItemReceiptGraphicsProgressReceipt {
                        caller: ItemReceiptGraphicsCaller::SpriteMain { slot: 12 },
                        progress: SourceCallProgress::Suspended,
                    },
                ),
            ],
        );
    }

    #[test]
    fn nmi_inside_link_oam_becomes_a_backend_neutral_receipt() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw("nmi", Some(0x0d_a9d0), None, None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam,),
            ],
        );
    }

    #[test]
    fn nmi_before_link_coordinate_publication_becomes_a_backend_neutral_receipt() {
        for pc in [0x07_e276, 0x07_e381] {
            let mut tracker = Snes9xOracleSemanticTrace {
                path: PathBuf::new(),
                offset: 0,
                cache_write_progress: None,
                normal_load_ordinal: None,
                pending_reset_progress: None,
                cached_sprite_execution: None,
                overworld_presence_published: false,
                overworld_sprite_activation: None,
                pending_spotlight_helper_nmi: None,
                pending_spotlight_helper_nmi_acceptance_index: None,
                seed_warmup_active: false,
                item_receipt_caller: None,
                sprite_main_execution: None,
                zelda_run_game_loop_call_active: false,
                nmi_publication_pending: false,
                pending_nmi_update_gate: None,
                nmi_resume_targets: Vec::new(),
                synthesized_nmi_resume: None,
                host_nmi_ppu_register_operands: Vec::new(),
            };
            let mut event = raw("nmi", Some(pc), None, None);
            event.main = Some(0x0f);
            event.sub = Some(1);
            let mut receipts = Vec::new();

            tracker.consume_event(event, &mut receipts).unwrap();

            assert_eq!(
                receipts,
                vec![
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(
                        MainLoopInterruption::LinkPositionBeforeCoordinates,
                    ),
                ],
                "source PC {pc:06x}",
            );
        }
    }

    #[test]
    fn nmi_inside_spotlight_circle_build_reports_exact_c_statement_progress() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut event = raw("nmi", Some(IRIS_SPOTLIGHT_CIRCLE_VALUE_CALL_PC), None, None);
        event.a = Some(30);
        event.main = Some(0x0f);
        event.sub = Some(0);
        event.link_y = Some(1012);
        event.bg2_v = Some(786);
        event.spotlight_radius = Some(126);
        let mut receipts = Vec::new();

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    SpotlightTableBuildProgressReceipt {
                        progress: SpotlightTableBuildProgress {
                            completed_iterations: 209,
                            checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                                pending_circle_input: 30,
                            },
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
            ],
        );
    }

    #[test]
    fn nmi_before_spotlight_iteration_initialization_reports_exact_c_progress() {
        let mut tracker = empty_semantic_tracker();
        let mut interrupted = raw(
            "nmi",
            Some(IRIS_SPOTLIGHT_ITERATION_VALUE_STORE_PC),
            Some(100),
            None,
        );
        interrupted.a = Some(0x00ff);
        interrupted.main = Some(0x0f);
        interrupted.sub = Some(0);
        interrupted.link_y = Some(8209);
        interrupted.bg2_v = Some(8191);
        interrupted.spotlight_radius = Some(126);
        let mut receipts = Vec::new();

        tracker.consume_event(interrupted, &mut receipts).unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open
            )]
        );

        let mut returned = frame_with_sub("return", 49_001, 0x0f, 0);
        returned.pc = Some(NMI_HANDLER_ENTRY_PC);
        tracker
            .finish_pending_spotlight_helper_nmi(&returned, Some(20), Some(49), None, &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    SpotlightTableBuildProgressReceipt {
                        progress: SpotlightTableBuildProgress {
                            completed_iterations: 175,
                            checkpoint:
                                SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
            ]
        );
    }

    #[test]
    fn nmi_inside_pure_circle_helper_rewinds_to_its_c_call_boundary() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        // The source loop loads t=126, decrements spotlight_var4 to 125, then
        // enters the pure circle helper. The authoritative frame-40977 NMI is
        // accepted at $00:f4da after the divider operands are written but
        // before a result or either HDMA-table word is published. The generic
        // WRAM write trace does not cover this direct-mapped store, so the
        // adapter reads the current source variable from WRAM at the immediate
        // NMI-entry return instead of retaining an older host's value.
        let mut interrupted = raw("nmi", Some(0x00_f4da), Some(182), None);
        interrupted.main = Some(0x0f);
        interrupted.sub = Some(0);
        interrupted.a = Some(126);
        interrupted.link_y = Some(8180);
        interrupted.bg2_v = Some(7954);
        interrupted.spotlight_radius = Some(126);
        let mut receipts = Vec::new();

        tracker.consume_event(interrupted, &mut receipts).unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open
            )],
        );
        let mut returned = frame("return", 40977, 0x0f, 253);
        returned.pc = Some(NMI_HANDLER_ENTRY_PC);
        tracker
            .finish_pending_spotlight_helper_nmi(&returned, Some(125), None, None, &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    SpotlightTableBuildProgressReceipt {
                        progress: SpotlightTableBuildProgress {
                            completed_iterations: 113,
                            checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                                pending_circle_input: 126,
                            },
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
            ],
        );
    }

    #[test]
    fn completed_spotlight_entry_supersedes_helper_interruption_progress() {
        // Pinned frame 50186 accepts NMI inside
        // IrisSpotlight_CalculateCircleValue at $00:f52d, then resumes the
        // same Module0F call through Dungeon_PrepExitWithSpotlight's
        // submodule increment before returning inside LinkOam_Main. The C
        // submodule transition proves the entire IrisSpotlight_close call has
        // returned, so replaying its earlier pure-helper checkpoint would
        // move source execution backwards.
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 3186, 0x0f, 0);
        entry.pc = Some(0x00_f52b);
        host.observe(&entry).unwrap();

        let mut interrupted = raw("nmi", Some(0x00_f52d), None, None);
        interrupted.main = Some(0x0f);
        interrupted.sub = Some(0);
        interrupted.a = Some(0x0174);
        interrupted.link_y = Some(8692);
        interrupted.bg2_v = Some(8465);
        interrupted.spotlight_radius = Some(126);
        let mut receipts = Vec::new();
        tracker
            .consume_event(interrupted.clone(), &mut receipts)
            .unwrap();
        host.observe(&interrupted).unwrap();

        let mut returned = frame_with_sub("return", 3186, 0x0f, 1);
        returned.pc = Some(0x0d_a38c);
        host.observe(&returned).unwrap();
        tracker
            .finish_pending_spotlight_helper_nmi(
                &returned,
                Some(119),
                None,
                host.spotlight_call_completion(),
                &mut receipts,
            )
            .unwrap();
        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam,),
                OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned,
            ],
        );
    }

    #[test]
    fn completed_recurring_spotlight_call_supersedes_helper_interruption_progress() {
        // Pinned frame 50192 enters this host while the recurring Module0F
        // spotlight caller is suspended at $00:f4fd, accepts NMI at $00:f500,
        // and returns at Zelda's $00:8036 main wait. The source-owned
        // nmi_boolean/main-wait transition proves Module_MainRouting and its
        // Link/OAM + NMI_PrepareSprites suffix returned, which supersedes the
        // intermediate pure-helper checkpoint.
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 3192, 0x0f, 1);
        entry.pc = Some(0x00_f4fd);
        entry.nmi_latch = Some(1);
        host.observe(&entry).unwrap();

        let mut interrupted = raw("nmi", Some(0x00_f500), None, None);
        interrupted.main = Some(0x0f);
        interrupted.sub = Some(1);
        interrupted.a = Some(110);
        interrupted.link_y = Some(8692);
        interrupted.bg2_v = Some(8465);
        interrupted.spotlight_radius = Some(112);
        let mut receipts = Vec::new();
        tracker
            .consume_event(interrupted.clone(), &mut receipts)
            .unwrap();
        host.observe(&interrupted).unwrap();

        let mut returned = frame_with_sub("return", 3192, 0x0f, 1);
        returned.pc = Some(0x00_8036);
        returned.nmi_latch = Some(0);
        host.observe(&returned).unwrap();
        tracker
            .finish_pending_spotlight_helper_nmi(
                &returned,
                Some(105),
                None,
                host.spotlight_call_completion(),
                &mut receipts,
            )
            .unwrap();
        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
            ],
        );
    }

    #[test]
    fn terminal_recurring_spotlight_double_nmi_suppresses_the_deferred_helper_checkpoint() {
        // Pinned run4791 accepts its leading Held NMI inside the circle helper,
        // completes that handler and the whole recurring Module0F caller, then
        // accepts one trailing Open NMI before the host returns inside its
        // handler. The deferred helper checkpoint belongs to the leading Held
        // boundary and is superseded by the stronger caller-return fact.
        let mut tracker = empty_semantic_tracker();
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 4_791, 0x0f, 1);
        entry.pc = Some(0x00_f4d4);
        entry.nmi_latch = Some(1);
        host.observe(&entry).unwrap();

        let mut leading = raw("nmi", Some(0x00_f4d7), Some(0x01f0), None);
        leading.main = Some(0x0f);
        leading.sub = Some(1);
        leading.nmi_latch = Some(1);
        leading.a = Some(105);
        leading.link_y = Some(8692);
        leading.bg2_v = Some(8466);
        leading.spotlight_radius = Some(105);
        let mut receipts = Vec::new();
        tracker
            .consume_event(leading.clone(), &mut receipts)
            .unwrap();
        host.observe(&leading).unwrap();
        publish_nmi(&mut tracker, &mut receipts);

        receipts.push(OriginalTimingSemanticReceipt::MainLoopProgress(
            MainLoopProgress::CallStackContinued,
        ));
        let suffix = main_loop_common_suffix_completion();
        assert_eq!(
            host.observe(&suffix).unwrap(),
            Some(MainLoopCompletionProof::CommonSuffixCompleted),
        );
        receipts.push(OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted);

        let mut trailing = raw("nmi", Some(0x00_8034), Some(0x01ff), None);
        trailing.main = Some(0x0f);
        trailing.sub = Some(1);
        trailing.nmi_latch = Some(0);
        tracker
            .consume_event(trailing.clone(), &mut receipts)
            .unwrap();
        host.observe(&trailing).unwrap();

        let mut returned = frame_with_sub("return", 4_791, 0x0f, 1);
        returned.pc = Some(NMI_HANDLER_ENTRY_PC);
        returned.nmi_latch = Some(0);
        host.observe(&returned).unwrap();
        assert_eq!(
            host.spotlight_call_completion(),
            Some(SpotlightCallCompletion::RecurringCallerReturnedToMainWait),
        );
        tracker
            .finish_pending_spotlight_helper_nmi(
                &returned,
                Some(9),
                Some(247),
                host.spotlight_call_completion(),
                &mut receipts,
            )
            .unwrap();
        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::DungeonExitSpotlightCallerReturnedToMainWait,
            ],
        );
    }

    #[test]
    fn nonterminal_final_handler_return_preserves_acceptance_then_deferred_helper_progress() {
        let mut tracker = empty_semantic_tracker();
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 4_790, 0x0f, 1);
        entry.pc = Some(0x00_f4d4);
        entry.nmi_latch = Some(1);
        host.observe(&entry).unwrap();

        let mut interrupted = raw("nmi", Some(0x00_f4d7), Some(0x01f0), None);
        interrupted.main = Some(0x0f);
        interrupted.sub = Some(1);
        interrupted.nmi_latch = Some(1);
        interrupted.a = Some(105);
        interrupted.link_y = Some(8692);
        interrupted.bg2_v = Some(8466);
        interrupted.spotlight_radius = Some(105);
        let mut receipts = Vec::new();
        tracker
            .consume_event(interrupted.clone(), &mut receipts)
            .unwrap();
        host.observe(&interrupted).unwrap();

        let mut returned = frame_with_sub("return", 4_790, 0x0f, 1);
        returned.pc = Some(NMI_HANDLER_ENTRY_PC);
        returned.nmi_latch = Some(1);
        host.observe(&returned).unwrap();
        assert_eq!(host.spotlight_call_completion(), None);
        tracker
            .finish_pending_spotlight_helper_nmi(&returned, Some(9), Some(247), None, &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    SpotlightTableBuildProgressReceipt {
                        progress: SpotlightTableBuildProgress {
                            completed_iterations: 229,
                            checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                                pending_circle_input: 10,
                            },
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
            ],
        );
    }

    #[test]
    fn recurring_spotlight_link_oam_interruption_supersedes_helper_progress() {
        // Pinned frame 56931 enters a recurring Module0F call at $00:f516,
        // accepts NMI inside the pure table helper at $00:f518, and returns
        // after the resumed caller has reached LinkOam_Main. The helper
        // checkpoint is no longer the source boundary: the stronger semantic
        // fact is that the enclosing caller reached Link OAM and remains
        // suspended there.
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 9931, 0x0f, 1);
        entry.pc = Some(0x00_f516);
        entry.nmi_latch = Some(1);
        host.observe(&entry).unwrap();

        let mut interrupted = raw("nmi", Some(0x00_f518), None, None);
        interrupted.main = Some(0x0f);
        interrupted.sub = Some(1);
        interrupted.a = Some(0xffa2);
        interrupted.x = Some(45);
        interrupted.link_y = Some(9204);
        interrupted.bg2_v = Some(8978);
        interrupted.spotlight_radius = Some(119);
        let mut receipts = Vec::new();
        tracker
            .consume_event(interrupted.clone(), &mut receipts)
            .unwrap();
        host.observe(&interrupted).unwrap();

        let mut returned = frame_with_sub("return", 9931, 0x0f, 1);
        returned.pc = Some(0x0d_a38f);
        returned.nmi_latch = Some(1);
        host.observe(&returned).unwrap();
        tracker
            .finish_pending_spotlight_helper_nmi(
                &returned,
                Some(112),
                None,
                host.spotlight_call_completion(),
                &mut receipts,
            )
            .unwrap();
        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam,),
            ],
        );
    }

    #[test]
    fn completed_overworld_spotlight_goal_supersedes_helper_interruption_progress() {
        // Pinned frame 54309 enters in recurring Module10 at $00:f4e6,
        // accepts NMI inside the pure helper at $00:f4e8, then returns at
        // Zelda's main wait after IrisSpotlight_ConfigureTable restored the
        // saved module and OpenSpotlight_Next2 selected its source submodule.
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 7309, 0x10, 1);
        entry.pc = Some(0x00_f4e6);
        entry.nmi_latch = Some(1);
        host.observe(&entry).unwrap();

        let mut interrupted = raw("nmi", Some(0x00_f4e8), None, None);
        interrupted.main = Some(0x10);
        interrupted.sub = Some(1);
        interrupted.a = Some(9);
        interrupted.link_y = Some(2359);
        interrupted.bg2_v = Some(2278);
        interrupted.spotlight_radius = Some(119);
        let mut receipts = Vec::new();
        tracker
            .consume_event(interrupted.clone(), &mut receipts)
            .unwrap();
        host.observe(&interrupted).unwrap();

        let mut returned = frame_with_sub("return", 7309, 9, 10);
        returned.pc = Some(0x00_8036);
        returned.nmi_latch = Some(0);
        host.observe(&returned).unwrap();
        tracker
            .finish_pending_spotlight_helper_nmi(
                &returned,
                Some(126),
                None,
                host.spotlight_call_completion(),
                &mut receipts,
            )
            .unwrap();
        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::OverworldSpotlightGoalCallerReturned,
            ],
        );
    }

    #[test]
    fn helper_interruption_rejects_unproven_non_nmi_host_return() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: Some({
                let mut event = raw("nmi", Some(0x00_f52d), None, None);
                event.main = Some(0x0f);
                event.sub = Some(0);
                event
            }),
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut returned = frame_with_sub("return", 3186, 0x0f, 0);
        returned.pc = Some(0x0d_a38c);

        assert_eq!(
            tracker
                .finish_pending_spotlight_helper_nmi(
                    &returned,
                    Some(119),
                    None,
                    None,
                    &mut Vec::new(),
                )
                .unwrap_err(),
            "Snes9x spotlight helper NMI did not return at the source NMI entry: $0da38c",
        );
    }

    #[test]
    fn host_return_before_upper_spotlight_write_rewinds_only_the_pure_helper() {
        // Pinned host call 43805 returns at $00:f383. The circle helper has
        // completed, X holds 2*r4, and spotlight_var4 has already decremented,
        // but the upper and lower HDMA-table stores are both still pending.
        // Re-entering at BeforeCircleCalculation repeats only the pure helper
        // and preserves the exact C publication boundary.
        let mut returned = raw(
            "frame",
            Some(IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC),
            Some(424),
            None,
        );
        returned.stage = Some("return".to_string());
        returned.main = Some(0x0f);
        returned.sub = Some(0);
        returned.link_y = Some(9204);
        returned.bg2_v = Some(8978);
        returned.spotlight_radius = Some(126);
        let mut receipts = Vec::new();

        publish_spotlight_host_return_progress(&returned, Some(26), None, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 212,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                            pending_circle_input: 27,
                        },
                    },
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            )],
        );
    }

    #[test]
    fn nmi_after_spotlight_cursor_update_resumes_at_the_next_c_iteration() {
        // Pinned cold-route frame 56,927 accepts NMI at $00:f3a0.  The false
        // loop-completion branch has already advanced both source cursors, but
        // the loop-back JMP has not begun the next C iteration. The source r6
        // cursor is 286 and spotlight_var4 is 49, which independently prove
        // exactly 190 completed iterations. X is deliberately not consulted:
        // some valid geometries execute no visible table store and retain an
        // unrelated value there.
        let mut event = raw("nmi", Some(IRIS_SPOTLIGHT_NEXT_ITERATION_PC), None, None);
        event.main = Some(0x0f);
        event.sub = Some(0);
        event.link_y = Some(9204);
        event.bg2_v = Some(8978);
        event.spotlight_radius = Some(126);
        event.spotlight_var4_low = Some(49);
        event.spotlight_lower_cursor = Some(286);

        assert_eq!(
            // Volatile host-end scratch is intentionally different; the
            // event-bound source values remain authoritative.
            spotlight_table_build_progress(&event, Some(0), Some(600)).unwrap(),
            Some(SpotlightTableBuildProgress {
                completed_iterations: 190,
                checkpoint: SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
            }),
        );
    }

    #[test]
    fn nmi_before_upper_spotlight_write_derives_input_from_source_cursor() {
        // Pinned frame 54151 accepts NMI at $00:f383 after the pure circle
        // helper returned but before the upper-table store. C has not advanced
        // r4 yet, so X=$0196 means upper cursor 203. With the first iris
        // iteration at 127 and radius 112, the pending helper input is exactly
        // 112 - (203 - 127) = 36. The accumulator already holds the helper
        // result and must not be interpreted as its input.
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut interrupted = raw(
            "nmi",
            Some(IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC),
            Some(406),
            None,
        );
        interrupted.main = Some(0x0f);
        interrupted.sub = Some(1);
        interrupted.a = Some(0xff00);
        interrupted.link_y = Some(9204);
        interrupted.bg2_v = Some(8978);
        interrupted.spotlight_radius = Some(112);
        let mut receipts = Vec::new();

        tracker.consume_event(interrupted, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    SpotlightTableBuildProgressReceipt {
                        progress: SpotlightTableBuildProgress {
                            completed_iterations: 203,
                            checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                                pending_circle_input: 36,
                            },
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
            ],
        );
    }

    #[test]
    fn nmi_between_spotlight_upper_and_lower_writes_reports_exact_c_statement_progress() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut event = raw(
            "nmi",
            Some(IRIS_SPOTLIGHT_LOWER_TABLE_WRITE_PC),
            Some(78),
            None,
        );
        event.a = Some(0xff00);
        event.main = Some(0x10);
        event.sub = Some(1);
        event.link_y = Some(1044);
        event.bg2_v = Some(1024);
        event.spotlight_radius = Some(119);
        let mut receipts = Vec::new();

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    SpotlightTableBuildProgressReceipt {
                        progress: SpotlightTableBuildProgress {
                            completed_iterations: 185,
                            checkpoint: SpotlightTableBuildCheckpoint::BeforeLowerTableWrite {
                                lower_cursor: 39,
                                circle_value: 0xff00,
                            },
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
            ],
        );
    }

    #[test]
    fn shared_circle_helper_in_dungeon_caller_does_not_cross_receipt_domains() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        // Pinned run 2327 is inside the same pure helper, but the enclosing
        // source caller is Module 7's dungeon-landing spotlight. That caller
        // has its own native continuation and must not receive the Module0F/10
        // opening/closing receipt merely because the ROM shares this helper.
        let mut interrupted = raw("nmi", Some(0x00_f4fd), Some(2), None);
        interrupted.main = Some(7);
        interrupted.sub = Some(15);
        interrupted.a = Some(119);
        interrupted.link_y = Some(8538);
        interrupted.bg2_v = Some(8464);
        interrupted.spotlight_radius = Some(119);
        let mut receipts = Vec::new();

        tracker.consume_event(interrupted, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open
            )],
        );
        assert!(tracker.pending_spotlight_helper_nmi.is_none());
    }

    #[test]
    fn nmi_after_spotlight_projection_store_reports_copied_word_prefix() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        // At $f3be the absolute,X store at $f3bb has completed with X=$0138.
        // Bytes 0..=$0139, or 157 words, are therefore already published.
        let mut event = raw(
            "nmi",
            Some(IRIS_SPOTLIGHT_COPY_FIRST_INCREMENT_PC),
            Some(0x0138),
            None,
        );
        event.main = Some(0x0f);
        event.sub = Some(0);
        event.link_y = Some(1704);
        event.bg2_v = Some(1610);
        event.spotlight_radius = Some(126);
        let mut receipts = Vec::new();

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    SpotlightTableBuildProgressReceipt {
                        progress: SpotlightTableBuildProgress {
                            completed_iterations: 119,
                            checkpoint: SpotlightTableBuildCheckpoint::ProjectionCopy {
                                copied_words: 157,
                            },
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
            ],
        );
    }

    #[test]
    fn nmi_at_spotlight_loop_completion_test_reports_published_iteration() {
        // The pinned failing cold host reaches $f39a after both table stores.
        // X still holds 2*r6 (222), while the source-derived upper cursor is
        // 107. The branch/cursor tail remains pending and must not cause the
        // translated owner to replay either table publication.
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut event = raw(
            "nmi",
            Some(IRIS_SPOTLIGHT_LOOP_COMPLETION_BRANCH_PC),
            Some(222),
            None,
        );
        event.main = Some(0x0f);
        event.sub = Some(0);
        event.link_y = Some(2168);
        event.bg2_v = Some(2071);
        event.spotlight_radius = Some(126);
        let mut receipts = Vec::new();

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    SpotlightTableBuildProgressReceipt {
                        progress: SpotlightTableBuildProgress {
                            completed_iterations: 113,
                            checkpoint: SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                                upper_cursor: 107,
                                lower_cursor: 111,
                            },
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
            ],
        );
    }

    #[test]
    fn loop_completion_uses_the_upper_cursor_when_the_lower_store_is_offscreen() {
        // Pinned cold host 37,589 reaches $f39a with r4=217 and r6=259.
        // Because r6 is offscreen, the lower store is skipped and X retains
        // the preceding upper-store byte offset 2*r4=434.
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut event = raw(
            "nmi",
            Some(IRIS_SPOTLIGHT_LOOP_COMPLETION_BRANCH_PC),
            Some(434),
            None,
        );
        event.main = Some(0x0f);
        event.sub = Some(1);
        event.link_y = Some(1012);
        event.bg2_v = Some(786);
        event.spotlight_radius = Some(112);
        let mut receipts = Vec::new();

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    SpotlightTableBuildProgressReceipt {
                        progress: SpotlightTableBuildProgress {
                            completed_iterations: 217,
                            checkpoint: SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                                upper_cursor: 217,
                                lower_cursor: 259,
                            },
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
            ],
        );
    }

    #[test]
    fn lower_cursor_decrement_checkpoint_preserves_the_source_statement_boundary() {
        // Cold host 54,145 returns at $f39e after INC r4 and before DEC r6.
        // The lower row is offscreen, so X retains the upper-table byte offset
        // from the just-published iteration.
        let mut event = raw(
            "frame",
            Some(IRIS_SPOTLIGHT_LOWER_CURSOR_DECREMENT_PC),
            Some(378),
            None,
        );
        event.stage = Some("return".to_string());
        event.run = Some(54_145);
        event.main = Some(0x0f);
        event.sub = Some(0);
        event.subsub = Some(0);
        event.frame_counter = Some(155);
        event.nmi_latch = Some(1);
        event.link_y = Some(9204);
        event.bg2_v = Some(8978);
        event.spotlight_radius = Some(126);
        let mut receipts = Vec::new();

        publish_spotlight_host_return_progress(&event, Some(0), None, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 189,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeLowerCursorDecrement {
                            upper_cursor: 190,
                            lower_cursor: 287,
                        },
                    },
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            )],
        );
    }

    #[test]
    fn shared_spotlight_copy_in_another_caller_does_not_cross_receipt_domains() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut event = raw(
            "nmi",
            Some(IRIS_SPOTLIGHT_COPY_FIRST_INCREMENT_PC),
            Some(0x0138),
            None,
        );
        event.main = Some(7);
        event.sub = Some(15);
        let mut receipts = Vec::new();

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open
            )]
        );
    }

    #[test]
    fn recurring_close_projection_store_remains_pending_in_its_semantic_receipt() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        // At $f3bb the long load is complete, but the absolute,X store has
        // not executed. X=$011e therefore denotes exactly 143 copied words.
        let mut event = raw(
            "nmi",
            Some(IRIS_SPOTLIGHT_COPY_STORE_PC),
            Some(0x011e),
            None,
        );
        event.main = Some(0x0f);
        event.sub = Some(1);
        event.link_y = Some(1703);
        event.bg2_v = Some(1610);
        event.spotlight_radius = Some(119);
        let mut receipts = Vec::new();

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    SpotlightTableBuildProgressReceipt {
                        progress: SpotlightTableBuildProgress {
                            completed_iterations: 120,
                            checkpoint: SpotlightTableBuildCheckpoint::ProjectionCopy {
                                copied_words: 143,
                            },
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
            ],
        );
    }

    #[test]
    fn host_return_replaces_earlier_spotlight_progress_with_latest_c_statement() {
        // The authoritative run returns at $f3bf after the second increment.
        // X=$01b7 means bytes 0..=$01b7, or 220 words, are already copied.
        let mut returned = raw(
            "frame",
            Some(IRIS_SPOTLIGHT_COPY_SECOND_INCREMENT_PC),
            Some(0x01b7),
            None,
        );
        returned.stage = Some("return".to_string());
        returned.main = Some(0x0f);
        returned.sub = Some(1);
        returned.link_y = Some(1703);
        returned.bg2_v = Some(1610);
        returned.spotlight_radius = Some(112);
        let mut receipts = vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 120,
                        checkpoint: SpotlightTableBuildCheckpoint::ProjectionCopy {
                            copied_words: 143,
                        },
                    },
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            ),
        ];

        publish_spotlight_host_return_progress(&returned, None, None, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                    SpotlightTableBuildProgressReceipt {
                        progress: SpotlightTableBuildProgress {
                            completed_iterations: 120,
                            checkpoint: SpotlightTableBuildCheckpoint::ProjectionCopy {
                                copied_words: 220,
                            },
                        },
                        boundary: OriginalTimingBoundary::HostReturn,
                    },
                ),
            ],
        );
    }

    #[test]
    fn pre_dungeon_sprite_reset_return_reports_completed_disable_semantically() {
        let mut returned = frame_with_sub("return", 39_722, 6, 0);
        returned.pc = Some(0x09_c47f);
        returned.return_address = Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC);
        let mut receipts = Vec::new();

        assert!(publish_pre_dungeon_sprite_reset_progress(
            &returned,
            OriginalTimingBoundary::HostReturn,
            &mut receipts,
        )
        .unwrap());

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteResetAllProgress(
                SpriteResetAllProgressReceipt {
                    progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            )],
        );

        returned.return_address = Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC + 4);
        let mut wrong_caller = Vec::new();
        assert!(!publish_pre_dungeon_sprite_reset_progress(
            &returned,
            OriginalTimingBoundary::HostReturn,
            &mut wrong_caller,
        )
        .unwrap());
        assert!(wrong_caller.is_empty());
    }

    #[test]
    fn pre_dungeon_sprite_reset_caller_proof_is_independent_of_entry_module() {
        for main in [5, 6, 27] {
            let mut returned = frame_with_sub("return", 2_291, main, 0);
            returned.pc = Some(0x09_c47f);
            returned.return_address = Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC);
            let mut receipts = Vec::new();

            assert!(publish_pre_dungeon_sprite_reset_progress(
                &returned,
                OriginalTimingBoundary::HostReturn,
                &mut receipts,
            )
            .unwrap());
            assert_eq!(
                receipts,
                vec![OriginalTimingSemanticReceipt::SpriteResetAllProgress(
                    SpriteResetAllProgressReceipt {
                        progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
                        boundary: OriginalTimingBoundary::HostReturn,
                    },
                )],
            );
        }
    }

    #[test]
    fn pre_dungeon_nmi_inside_reset_suffix_routes_completed_disable_to_its_source_caller() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: Some(DungeonResetSpritesCpuProgress::SpritesDisabled),
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        // Pinned Snes9x frame 48,536 accepts NMI at $09:c47b inside
        // Sprite_ResetAll_noDisable with the innermost return address $02:834b,
        // the Module_PreDungeon statement immediately after Sprite_ResetAll.
        let mut event = raw("nmi", Some(0x09_c47b), Some(0x0ed7), None);
        event.return_address = Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC);
        event.main = Some(6);
        event.sub = Some(0);
        let mut receipts = Vec::new();

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::SpriteResetAllProgress(
                    SpriteResetAllProgressReceipt {
                        progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        );
        assert_eq!(tracker.pending_reset_progress, None);
    }

    #[test]
    fn host_return_inside_link_oam_becomes_a_backend_neutral_receipt() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 11_561, 7, 15);
        entry.subsub = Some(0);
        entry.frame_counter = Some(186);
        entry.pc = Some(0x00_8036);
        let mut returned = frame_with_sub("return", 11_561, 7, 15);
        returned.subsub = Some(1);
        returned.frame_counter = Some(187);
        returned.pc = Some(0x0d_a49b);

        host.observe(&entry).unwrap();
        host.observe(&main_loop_start()).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, false).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted,),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam,),
            ],
        );
    }

    #[test]
    fn nmi_acceptance_and_publication_remain_distinct_ordered_receipts() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw("nmi", None, None, None), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("nmi", 0x008010, 0x01f0), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        );
    }

    #[test]
    fn ordinary_and_threaded_nmi_paths_share_one_publication_receipt() {
        for &publication_pc in &NMI_HANDLER_COMPLETE_PCS {
            let mut tracker = empty_semantic_tracker();
            let mut receipts = Vec::new();

            tracker
                .consume_event(raw("nmi", None, None, None), &mut receipts)
                .unwrap();
            tracker
                .consume_event(raw_at("pc", publication_pc, 0x01f3), &mut receipts)
                .unwrap();

            assert_eq!(
                receipts,
                vec![
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                    OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                    zero_joypad_publication(),
                ],
            );
            assert!(!tracker.nmi_publication_pending);
        }
    }

    #[test]
    fn frame_boundary_context_resume_is_private_and_deduplicates_the_direct_marker() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("frame", 0x008123, 0x01f8), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("nmi-resume", 0x008123, 0x01f8), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
            ]
        );
        assert!(tracker.nmi_resume_targets.is_empty());
        assert_eq!(tracker.synthesized_nmi_resume, None);
    }

    #[test]
    fn new_nmi_at_restored_context_validates_the_old_resume_without_republishing_it() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ]
        );
        assert_eq!(tracker.nmi_resume_targets, vec![(0x008123, 0x01f8)]);
    }

    #[test]
    fn nested_nmi_contexts_resume_in_stack_order_without_gameplay_receipts() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("nmi", 0x0080d0, 0x01f0), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("nmi-resume", 0x0080d0, 0x01f0), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("pc", 0x008123, 0x01f8), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
            ]
        );
        assert!(tracker.nmi_resume_targets.is_empty());
    }

    #[test]
    fn mismatched_direct_nmi_completion_fails_closed_without_losing_target() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
            .unwrap();
        tracker
            .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
            .unwrap();
        let error = tracker
            .consume_event(raw_at("nmi-resume", 0x008124, 0x01f8), &mut receipts)
            .unwrap_err();

        assert!(error.contains("did not match the active target"));
        assert_eq!(tracker.nmi_resume_targets, vec![(0x008123, 0x01f8)]);
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
            ]
        );
    }

    #[test]
    fn publication_without_acceptance_fails_closed() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();

        let error = tracker
            .consume_event(raw_at("pc", NMI_HANDLER_COMPLETE_PC, 0x01f3), &mut receipts)
            .unwrap_err();

        assert!(error.contains("without an accepted NMI"));
        assert!(receipts.is_empty());
        assert!(!tracker.nmi_publication_pending);
    }

    #[test]
    fn second_acceptance_before_publication_fails_without_losing_context() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        tracker
            .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
            .unwrap();

        let error = tracker
            .consume_event(raw_at("nmi", 0x008123, 0x01f8), &mut receipts)
            .unwrap_err();

        assert!(error.contains("before the first published"));
        assert_eq!(tracker.nmi_resume_targets, vec![(0x008123, 0x01f8)]);
        assert!(tracker.nmi_publication_pending);
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open
            )]
        );
    }

    #[test]
    fn live_trace_preserves_main_loop_start_between_leading_and_trailing_nmi_phases() {
        let path = env::temp_dir().join(format!(
            "zelda3-snes9x-ordered-main-loop-receipt-{}.jsonl",
            std::process::id()
        ));
        let events = [
            serde_json::json!({
                "event": "frame", "stage": "entry", "run": 7,
                "pc": 0x008123, "s": 0x01f8, "main": 15, "sub": 1,
                "subsub": 0, "frame_counter": 9, "nmi_latch": 1
            }),
            serde_json::json!({
                "event": "nmi", "run": 7, "pc": 0x008123, "s": 0x01f8,
                "main": 15, "sub": 1, "nmi_latch": 0,
                "nmi_ppu_register_operands": vec![0; 31]
            }),
            serde_json::json!({
                "event": "pc", "run": 7, "pc": NMI_HANDLER_COMPLETE_PC,
                "s": 0x01f3,
                "joypad_high": 0, "joypad_low": 0,
                "joypad_high_filtered": 0, "joypad_low_filtered": 0
            }),
            serde_json::json!({
                "event": "wram-write", "run": 7,
                "pc": ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC,
                "s": 0x01f8, "address": FRAME_COUNTER, "value": 10
            }),
            serde_json::json!({
                "event": "nmi-resume", "run": 7, "pc": 0x008123, "s": 0x01f8
            }),
            serde_json::json!({
                "event": "nmi", "run": 7, "pc": 0x009000, "s": 0x01f7,
                "main": 15, "sub": 1, "nmi_latch": 1,
                    "nmi_ppu_register_operands": vec![0; 31]
            }),
            serde_json::json!({
                "event": "frame", "stage": "return", "run": 7,
                "pc": NMI_HANDLER_ENTRY_PC, "s": 0x01f3, "main": 15, "sub": 1,
                "subsub": 0, "frame_counter": 10, "nmi_latch": 1
            }),
        ];
        let encoded = events
            .iter()
            .map(serde_json::Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        fs::write(&path, encoded).unwrap();
        let mut tracker = Snes9xOracleSemanticTrace {
            path: path.clone(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };

        let receipts = tracker.read_after_host_call(None, None, None).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
                OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted,),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            ]
        );
    }

    #[test]
    fn live_trace_publishes_exact_common_suffix_before_poly_nmi_and_across_next_host() {
        let path = env::temp_dir().join(format!(
            "zelda3-snes9x-poly-common-suffix-receipt-{}.jsonl",
            std::process::id()
        ));
        let first_host = [
            serde_json::json!({
                "event": "frame", "stage": "entry", "run": 889,
                "pc": 0x008034, "s": 0x01ff, "main": 0, "sub": 7,
                "subsub": 0, "frame_counter": 147, "nmi_latch": 0
            }),
            serde_json::json!({
                "event": "nmi", "run": 889, "pc": 0x008036, "s": 0x01ff,
                "main": 0, "sub": 7, "nmi_latch": 0,
                    "nmi_ppu_register_operands": vec![0; 31]
            }),
            serde_json::json!({
                "event": "pc", "run": 889, "pc": NMI_HANDLER_COMPLETE_PC,
                "s": 0x01f2,
                "joypad_high": 0, "joypad_low": 0,
                "joypad_high_filtered": 0, "joypad_low_filtered": 0
            }),
            serde_json::json!({
                "event": "nmi-resume", "run": 889, "pc": 0x008036, "s": 0x01ff
            }),
            serde_json::json!({
                "event": "wram-write", "run": 889,
                "pc": ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC,
                "s": 0x01ff, "address": FRAME_COUNTER, "value": 148
            }),
            serde_json::json!({
                "event": "wram-write", "run": 889,
                "pc": ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC,
                "s": 0x01ff, "address": NMI_UPDATE_LATCH, "value": 0
            }),
            serde_json::json!({
                "event": "nmi", "run": 889, "pc": 0x09fd18, "s": 0x1f39,
                "main": 0, "sub": 7, "nmi_latch": 0,
                    "nmi_ppu_register_operands": vec![0; 31]
            }),
            serde_json::json!({
                "event": "frame", "stage": "return", "run": 889,
                "pc": NMI_HANDLER_ENTRY_PC, "s": 0x1f35, "main": 0, "sub": 7,
                "subsub": 0, "frame_counter": 148, "nmi_latch": 0
            }),
        ];
        fs::write(
            &path,
            first_host
                .iter()
                .map(serde_json::Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
                + "\n",
        )
        .unwrap();
        let mut tracker = Snes9xOracleSemanticTrace {
            path: path.clone(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };

        let receipts = tracker.read_after_host_call(None, None, None).unwrap();
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
                OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted,),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        );
        assert!(tracker.nmi_publication_pending);
        assert_eq!(tracker.take_host_nmi_ppu_register_operands().len(), 2);

        let second_host = [
            serde_json::json!({
                "event": "frame", "stage": "entry", "run": 890,
                "pc": NMI_HANDLER_ENTRY_PC, "s": 0x1f35, "main": 0, "sub": 7,
                "subsub": 0, "frame_counter": 148, "nmi_latch": 0
            }),
            serde_json::json!({
                "event": "pc", "run": 890, "pc": NMI_HANDLER_COMPLETE_PCS[1],
                "s": 0x1f2c,
                "joypad_high": 0, "joypad_low": 0,
                "joypad_high_filtered": 0, "joypad_low_filtered": 0
            }),
            serde_json::json!({
                "event": "wram-write", "run": 890,
                "pc": ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC,
                "s": 0x01ff, "address": FRAME_COUNTER, "value": 149
            }),
            serde_json::json!({
                "event": "wram-write", "run": 890,
                "pc": ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC,
                "s": 0x01ff, "address": NMI_UPDATE_LATCH, "value": 0
            }),
            serde_json::json!({
                "event": "nmi-resume", "run": 890, "pc": 0x09fd18, "s": 0x1f39
            }),
            serde_json::json!({
                "event": "frame", "stage": "return", "run": 890,
                "pc": 0x09fe63, "s": 0x1f34, "main": 0, "sub": 7,
                "subsub": 0, "frame_counter": 149, "nmi_latch": 0
            }),
        ];
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap();
        for event in second_host {
            writeln!(file, "{}", event).unwrap();
        }
        drop(file);

        let receipts = tracker.read_after_host_call(None, None, None).unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                zero_joypad_publication(),
                OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted,),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
            ],
        );
        assert!(!tracker.nmi_publication_pending);
        assert!(tracker.nmi_resume_targets.is_empty());
    }

    #[test]
    fn continued_trace_publishes_progress_and_exact_suffix_before_following_open_nmi() {
        let path = env::temp_dir().join(format!(
            "zelda3-snes9x-continued-common-suffix-receipt-{}.jsonl",
            std::process::id()
        ));
        let events = [
            serde_json::json!({
                "event": "frame", "stage": "entry", "run": 1059,
                "pc": 0x0cce3a, "s": 0x01f3, "main": 4, "sub": 1,
                "subsub": 0, "frame_counter": 1, "nmi_latch": 1
            }),
            serde_json::json!({
                "event": "nmi", "run": 1059, "pc": 0x0cce3c,
                "s": 0x01f3, "main": 4, "sub": 1, "nmi_latch": 1,
                    "nmi_ppu_register_operands": vec![0; 31]
            }),
            serde_json::json!({
                "event": "pc", "run": 1059, "pc": NMI_HANDLER_COMPLETE_PC,
                "s": 0x01f2
            }),
            serde_json::json!({
                "event": "nmi-resume", "run": 1059, "pc": 0x0cce3c,
                "s": 0x01f3
            }),
            serde_json::json!({
                "event": "wram-write", "run": 1059,
                "pc": ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC,
                "s": 0x01ff, "address": NMI_UPDATE_LATCH, "value": 0
            }),
            serde_json::json!({
                "event": "nmi", "run": 1059, "pc": 0x008034,
                "s": 0x01ff, "main": 4, "sub": 2, "nmi_latch": 0,
                    "nmi_ppu_register_operands": vec![0; 31]
            }),
            serde_json::json!({
                "event": "frame", "stage": "return", "run": 1059,
                "pc": NMI_HANDLER_ENTRY_PC, "s": 0x01fb, "main": 4,
                "sub": 2, "subsub": 0, "frame_counter": 1, "nmi_latch": 0
            }),
        ];
        fs::write(
            &path,
            events
                .iter()
                .map(serde_json::Value::to_string)
                .collect::<Vec<_>>()
                .join("\n")
                + "\n",
        )
        .unwrap();
        let mut tracker = Snes9xOracleSemanticTrace {
            path: path.clone(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: true,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };

        let receipts = tracker.read_after_host_call(None, None, None).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        );
        assert!(tracker.nmi_publication_pending);
        assert_eq!(tracker.pending_nmi_update_gate, Some(NmiUpdateGate::Open));
    }

    #[test]
    fn common_suffix_receipt_requires_exact_post_write_pc_value_and_is_unique() {
        let mut host = HostFrameWindow::default();
        let mut wrong_pc = main_loop_common_suffix_completion();
        wrong_pc.pc = Some(ZELDA_RUN_GAME_LOOP_COMMON_SUFFIX_WRITE_PC - 1);
        assert_eq!(host.observe(&wrong_pc).unwrap(), None);

        let mut wrong_value = main_loop_common_suffix_completion();
        wrong_value.value = Some(1);
        assert!(host
            .observe(&wrong_value)
            .unwrap_err()
            .contains("invalid $12 value"));
        assert!(!host.main_loop_common_suffix_completed);

        assert_eq!(
            host.observe(&main_loop_common_suffix_completion()).unwrap(),
            Some(MainLoopCompletionProof::CommonSuffixCompleted),
        );
        assert!(host
            .observe(&main_loop_common_suffix_completion())
            .unwrap_err()
            .contains("common suffix twice"));
    }

    #[test]
    fn cached_sprite_load_and_restore_writes_become_semantic_progress() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();
        for &base in &CACHED_SPRITE_LIVE_FIELDS[..4] {
            tracker
                .consume_event(
                    raw(
                        "wram-write",
                        Some(UNCACHE_SPRITE_START_PC + 0x20),
                        None,
                        Some(base + 7),
                    ),
                    &mut receipts,
                )
                .unwrap();
        }
        tracker
            .consume_event(raw("nmi", None, None, None), &mut receipts)
            .unwrap();
        publish_nmi(&mut tracker, &mut receipts);

        for &base in CACHED_SPRITE_LIVE_FIELDS.iter().rev().take(4) {
            tracker
                .consume_event(
                    raw(
                        "wram-write",
                        Some(UNCACHE_SPRITE_RESTORE_START_PC),
                        None,
                        Some(base + 7),
                    ),
                    &mut receipts,
                )
                .unwrap();
        }
        tracker
            .consume_event(raw_at("nmi", 0x008010, 0x01f0), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
                    CachedSpriteExecutionProgressReceipt {
                        progress: CachedSpriteExecutionProgress::Loading {
                            slot: 7,
                            copied_fields: 4,
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0,
                    low: 0,
                    high_filtered: 0,
                    low_filtered: 0,
                }),
                OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
                    CachedSpriteExecutionProgressReceipt {
                        progress: CachedSpriteExecutionProgress::Restoring {
                            slot: 7,
                            live_fields: 20,
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        );
    }

    #[test]
    fn cached_sprite_progress_at_scan_keys_return_keeps_host_return_ownership() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(UNCACHE_SPRITE_RESTORE_START_PC),
                    None,
                    Some(CACHED_SPRITE_LIVE_FIELDS[7] + 2),
                ),
                &mut receipts,
            )
            .unwrap();

        tracker.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
                    CachedSpriteExecutionProgressReceipt {
                        progress: CachedSpriteExecutionProgress::Restoring {
                            slot: 2,
                            live_fields: 7,
                        },
                        boundary: OriginalTimingBoundary::HostReturn,
                    },
                ),
            ],
        );
        assert_eq!(tracker.cached_sprite_execution, None);
    }

    #[test]
    fn source_y_high_then_nmi_becomes_one_typed_progress_receipt() {
        let path = env::temp_dir().join("unused-snes9x-semantic-test.jsonl");
        let mut tracker = Snes9xOracleSemanticTrace {
            path,
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(DUNGEON_RESET_SPRITES_CLEAR_PC),
                    Some(0),
                    Some(0x0dd0),
                ),
                &mut receipts,
            )
            .unwrap();
        for slot in [0, 1] {
            tracker
                .consume_event(
                    raw(
                        "wram-write",
                        Some(DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC),
                        Some(slot),
                        Some(SPRITE_STATE_BASE + slot),
                    ),
                    &mut receipts,
                )
                .unwrap();
            tracker
                .consume_event(
                    raw(
                        "wram-write",
                        Some(DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC),
                        Some(slot),
                        Some(SPRITE_Y_HIGH_BASE + slot),
                    ),
                    &mut receipts,
                )
                .unwrap();
            if slot == 0 {
                tracker
                    .consume_event(
                        raw(
                            "wram-write",
                            Some(0x09c3b6),
                            Some(slot),
                            Some(0x0d10 + slot),
                        ),
                        &mut receipts,
                    )
                    .unwrap();
            }
        }
        tracker
            .consume_event(raw("nmi", None, None, None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                    DungeonResetSpritesProgressReceipt {
                        progress: DungeonResetSpritesCpuProgress::Load(
                            DungeonLoadSpritesCpuProgress {
                                normal_load_ordinal: 1,
                                slot: 1,
                                checkpoint: DungeonSpriteLoadCheckpoint::YHigh,
                            },
                        ),
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        );
    }

    #[test]
    fn cache_short_branch_then_nmi_becomes_a_typed_field_receipt() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();
        for &(field, base) in &CACHE_FIELD_WRITES[..=6] {
            tracker
                .consume_event(
                    raw(
                        "wram-write",
                        Some(DUNGEON_CACHE_TRANS_SPRITES_START_PC + 9),
                        Some(15),
                        Some(base + 15),
                    ),
                    &mut receipts,
                )
                .unwrap_or_else(|error| panic!("failed to consume {field:?}: {error}"));
        }
        tracker
            .consume_event(raw("nmi", None, None, None), &mut receipts)
            .unwrap();
        publish_nmi(&mut tracker, &mut receipts);
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(DUNGEON_CACHE_TRANS_SPRITES_START_PC + 9),
                    Some(14),
                    Some(0x1d0e),
                ),
                &mut receipts,
            )
            .unwrap();
        tracker
            .consume_event(raw_at("nmi", 0x008010, 0x01f0), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                    DungeonResetSpritesProgressReceipt {
                        progress: DungeonResetSpritesCpuProgress::Cache {
                            slot: 15,
                            field: CachedSpriteCacheField::YHigh,
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::NmiHandlerCompleted,
                OriginalTimingSemanticReceipt::JoypadPublication(JoypadPublication {
                    high: 0,
                    low: 0,
                    high_filtered: 0,
                    low_filtered: 0,
                }),
                OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                    DungeonResetSpritesProgressReceipt {
                        progress: DungeonResetSpritesCpuProgress::Cache {
                            slot: 14,
                            field: CachedSpriteCacheField::StateClear,
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        );
    }

    #[test]
    fn host_return_without_nmi_publishes_the_completed_sprite_disable_prefix() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(SPRITE_DISABLE_ALL_FINAL_GARNISH_PC),
                    Some(0),
                    Some(GARNISH_TYPE_SLOT_ZERO),
                ),
                &mut receipts,
            )
            .unwrap();
        assert!(receipts.is_empty());

        tracker.flush_reset_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                DungeonResetSpritesProgressReceipt {
                    progress: DungeonResetSpritesCpuProgress::SpritesDisabled,
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            )],
        );
        let mut next_host = Vec::new();
        tracker.flush_reset_progress(&mut next_host, OriginalTimingBoundary::HostReturn);
        assert!(next_host.is_empty(), "host-return receipts are one-shot");
    }

    #[test]
    fn sprite_disable_progress_refines_across_host_return_then_nmi() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut first_host = Vec::new();
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(DUNGEON_RESET_SPRITES_CLEAR_PC),
                    Some(0),
                    Some(SPRITE_STATE_BASE),
                ),
                &mut first_host,
            )
            .unwrap();
        for slot in (0..10).rev() {
            tracker
                .consume_event(
                    raw(
                        "wram-write",
                        Some(DUNGEON_RESET_SPRITES_CLEAR_PC + 5),
                        Some(slot),
                        Some(ANCILLA_TYPE_BASE + slot),
                    ),
                    &mut first_host,
                )
                .unwrap();
        }
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(DUNGEON_RESET_SPRITES_CLEAR_PC + 0x0e),
                    Some(0xff),
                    Some(ANCILLA_PICKUP_FLAG),
                ),
                &mut first_host,
            )
            .unwrap();
        tracker.flush_reset_progress(&mut first_host, OriginalTimingBoundary::HostReturn);
        assert_eq!(
            first_host,
            vec![OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                DungeonResetSpritesProgressReceipt {
                    progress: DungeonResetSpritesCpuProgress::Disable(
                        DungeonSpriteDisableCpuProgress::AncillaPickupFlagCleared,
                    ),
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            )],
        );

        let mut second_host = Vec::new();
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(DUNGEON_RESET_SPRITES_CLEAR_PC + 0x11),
                    Some(0xff),
                    Some(SPRITE_LIMIT_INSTANCE),
                ),
                &mut second_host,
            )
            .unwrap();
        tracker
            .consume_event(raw("nmi", None, None, None), &mut second_host)
            .unwrap();
        assert_eq!(
            second_host,
            vec![
                OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                    DungeonResetSpritesProgressReceipt {
                        progress: DungeonResetSpritesCpuProgress::Disable(
                            DungeonSpriteDisableCpuProgress::SpriteLimitInstanceCleared,
                        ),
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        );
    }

    #[test]
    fn pinned_route_receipt_decodes_to_normal_load_one_slot_one_y_high() {
        let mut trace = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();
        for line in include_str!(
            "../../external/snes9x-libretro/fixtures/zelda3-dungeon-reset-sprites-yhigh-nmi.jsonl"
        )
        .lines()
        {
            let mut event: RawTraceEvent = serde_json::from_str(line).unwrap();
            if event.event == "nmi" && event.s.is_none() {
                // This older reduced fixture ends at acceptance and predates
                // preservation of the already-present source stack and update-
                // gate fields. A later full trace of this same route boundary
                // independently preserves the omitted source values: S=$01f2
                // and Zelda's `$12` latch held. This fixture cannot exercise
                // completion ownership; restore only those corroborated fields
                // to drive its terminal acceptance through the stricter adapter.
                event.s = Some(0x01f2);
                event.nmi_latch = Some(1);
                event.nmi_ppu_register_operands = Some([0; 31]);
            }
            trace.consume_event(event, &mut receipts).unwrap();
        }

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                    DungeonResetSpritesProgressReceipt {
                        progress: DungeonResetSpritesCpuProgress::Load(
                            DungeonLoadSpritesCpuProgress {
                                normal_load_ordinal: 1,
                                slot: 1,
                                checkpoint: DungeonSpriteLoadCheckpoint::YHigh,
                            },
                        ),
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            ],
        );
    }

    #[test]
    fn later_source_write_invalidates_the_y_high_candidate() {
        let path = env::temp_dir().join("unused-snes9x-semantic-test.jsonl");
        let mut tracker = Snes9xOracleSemanticTrace {
            path,
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: Some(0),
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC),
                    Some(0),
                    Some(SPRITE_Y_HIGH_BASE),
                ),
                &mut receipts,
            )
            .unwrap();
        tracker
            .consume_event(
                raw("wram-write", Some(0x09c3b6), Some(0), Some(0x0d10)),
                &mut receipts,
            )
            .unwrap();
        tracker
            .consume_event(raw("nmi", None, None, None), &mut receipts)
            .unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open
            )]
        );
    }

    #[test]
    fn later_source_write_invalidates_the_sprite_disable_candidate() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            pending_spotlight_helper_nmi: None,
            pending_spotlight_helper_nmi_acceptance_index: None,
            seed_warmup_active: false,
            item_receipt_caller: None,
            sprite_main_execution: None,
            zelda_run_game_loop_call_active: false,
            nmi_publication_pending: false,
            pending_nmi_update_gate: None,
            nmi_resume_targets: Vec::new(),
            synthesized_nmi_resume: None,
            host_nmi_ppu_register_operands: Vec::new(),
        };
        let mut receipts = Vec::new();
        tracker
            .consume_event(
                raw(
                    "wram-write",
                    Some(SPRITE_DISABLE_ALL_FINAL_GARNISH_PC),
                    Some(0),
                    Some(GARNISH_TYPE_SLOT_ZERO),
                ),
                &mut receipts,
            )
            .unwrap();
        // The pinned route immediately continues into post-disable
        // bookkeeping at $09:C12C before loading sprites and consuming RNG.
        tracker
            .consume_event(
                raw("wram-write", Some(0x09c12c), Some(0xff), Some(0x0fba)),
                &mut receipts,
            )
            .unwrap();
        tracker
            .consume_event(raw("nmi", None, None, None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::Open
            )]
        );
    }

    #[test]
    fn csv_extension_is_deduplicated_and_preserves_existing_domains() {
        assert_eq!(
            append_csv(Some("frame,wram"), &["nmi", "wram"]),
            "frame,wram,nmi"
        );
    }
}
