//! Temporary pinned-Snes9x adapter for Zelda-level semantic receipts.
//!
//! Emulator PCs and WRAM addresses are allowed only in this replaceable host
//! adapter. Translated gameplay receives the typed values from `zelda3`.

use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use zelda3::{
    CachedSpriteCacheField, CachedSpriteExecutionProgress, CachedSpriteExecutionProgressReceipt,
    DialogueExecutionProgress, DungeonLoadSpritesCpuProgress, DungeonResetSpritesCpuProgress,
    DungeonResetSpritesProgressReceipt, DungeonSpriteDisableCpuProgress,
    DungeonSpriteLoadCheckpoint, MainLoopInterruption, MainLoopProgress, OriginalTimingBoundary,
    OriginalTimingSemanticReceipt, OverworldSpriteReloadProgress, PreOverworldStageCompletion,
    SaveMenuInitializationProgress, SpotlightTableBuildCheckpoint, SpotlightTableBuildProgress,
    SpotlightTableBuildProgressReceipt, SpriteResetAllProgress, SpriteResetAllProgressReceipt,
};

const TRACE_PATH_ENV: &str = "ZELDA3_SNES9X_TRACE";
const TRACE_EVENTS_ENV: &str = "ZELDA3_SNES9X_TRACE_EVENTS";
const TRACE_WRAM_ENV: &str = "ZELDA3_SNES9X_TRACE_WRAM";

const FRAME_COUNTER: u16 = 0x001a;
// ROM $00:8051 is `INC $1a`, the first statement of ZeldaRunGameLoop.
// The generic WRAM hook observes the instruction's post-write PC.
const ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC: u32 = 0x008053;
// In pinned ROM source, $00:f375 is the JSR to
// IrisSpotlight_CalculateCircleValue. The current loop iteration has loaded
// its input and conditionally decremented spotlight_var4, but has not yet
// calculated or stored either HDMA-table word.
const IRIS_SPOTLIGHT_CIRCLE_VALUE_CALL_PC: u32 = 0x00f375;
// The pure circle helper has returned and the source loop has doubled its
// upper cursor, but neither HDMA-table store has executed. Rewind this
// emulator-private instruction boundary to the same resumable C checkpoint as
// the helper call: recalculating the pure value cannot replay a publication.
const IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC: u32 = 0x00f383;
// ROM $00:f392 is the long WRAM store of the already-calculated circle value
// to `hdma_table_dynamic[r6]`. The upper-cursor store at $00:f383 is complete;
// this lower store and the loop-cursor update remain pending.
const IRIS_SPOTLIGHT_LOWER_TABLE_WRITE_PC: u32 = 0x00f392;
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
const IRIS_SPOTLIGHT_CIRCLE_VALUE_START_PC: u32 = 0x00f4cc;
const IRIS_SPOTLIGHT_CIRCLE_VALUE_END_PC: u32 = 0x00f53e;
pub(crate) const SPOTLIGHT_VAR4_LOW_ADDRESS: usize = 0x067a;
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
const NMI_PREPARE_SPRITES_END_PC: u32 = 0x008901;
const LINK_OAM_START_PC: u32 = 0x0da18e;
const LINK_OAM_END_PC: u32 = 0x0dadb6;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CacheWriteProgress {
    slot: u8,
    next_field_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CachedSpriteExecutionTracker {
    slot: u8,
    copied_fields: u8,
    restored_fields: u8,
    restore_started: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OverworldSpriteActivationTracker {
    slot: u8,
    block_low: Option<u8>,
    block_high: Option<u8>,
    sprite_type: Option<u8>,
    state_published: bool,
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
    path: PathBuf,
    offset: u64,
    cache_write_progress: Option<CacheWriteProgress>,
    normal_load_ordinal: Option<u16>,
    pending_reset_progress: Option<DungeonResetSpritesCpuProgress>,
    cached_sprite_execution: Option<CachedSpriteExecutionTracker>,
    overworld_presence_published: bool,
    overworld_sprite_activation: Option<OverworldSpriteActivationTracker>,
    pending_spotlight_helper_nmi: Option<RawTraceEvent>,
}

#[derive(Clone, Deserialize)]
struct RawTraceEvent {
    event: String,
    #[serde(default)]
    stage: Option<String>,
    #[serde(default)]
    run: Option<u64>,
    #[serde(default)]
    pc: Option<u32>,
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
    x: Option<u16>,
    #[serde(default)]
    address: Option<u16>,
    #[serde(default)]
    value: Option<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HostFrameState {
    run: u64,
    pc: u32,
    main: u8,
    sub: u8,
    subsub: u8,
    frame_counter: u8,
    nmi_latch: u8,
}

#[derive(Default)]
struct HostFrameWindow {
    entry: Option<HostFrameState>,
    returned: Option<HostFrameState>,
    vwf_nmi_observed: bool,
    main_loop_starts: u8,
}

impl HostFrameWindow {
    fn observe(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if event.event == "wram-write"
            && event.address == Some(FRAME_COUNTER)
            && event.pc.map(|pc| pc & 0x00ff_ffff)
                == Some(ZELDA_RUN_GAME_LOOP_FRAME_COUNTER_WRITE_PC)
        {
            self.main_loop_starts = self
                .main_loop_starts
                .checked_add(1)
                .ok_or("Snes9x host call overflowed its ZeldaRunGameLoop start count")?;
        }
        if event.event == "nmi"
            && event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
                (VWF_RENDER_SINGLE_START_PC..VWF_RENDER_SINGLE_END_PC).contains(&pc)
            })
        {
            self.vwf_nmi_observed = true;
        }
        if event.event != "frame" {
            return Ok(());
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
            "entry" if self.entry.replace(state).is_none() => Ok(()),
            "return" if self.returned.replace(state).is_none() => Ok(()),
            "entry" | "return" => Err(format!(
                "Snes9x host call published duplicate frame/{stage} receipts"
            )),
            _ => Ok(()),
        }
    }

    fn finish(
        self,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
        dialogue_message_read_position: Option<u16>,
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
            0 => MainLoopProgress::CallStackContinued,
            1 => MainLoopProgress::IterationStarted,
            starts => {
                return Err(format!(
                    "Snes9x host call started ZeldaRunGameLoop {starts} times; expected zero or one"
                ));
            }
        };
        receipts.push(OriginalTimingSemanticReceipt::MainLoopProgress(
            main_loop_progress,
        ));
        let resumed_spotlight_caller_returned = self.main_loop_starts == 0
            && entry.main == 0x0f
            && entry.sub == 1
            && returned.main == 0x0f
            && returned.sub == 1
            && !zelda_main_wait_pc(entry.pc)
            && (zelda_main_wait_pc(returned.pc)
                || (entry.nmi_latch != 0 && returned.nmi_latch == 0));
        if resumed_spotlight_caller_returned {
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
        if let Some(phase) = main_loop_interruption_for_pc(returned.pc) {
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
        if entry.main == 6 && returned.main == 7 && returned.sub == 15 {
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
        if entry.main == 9 && entry.sub == 3 && returned.main == 9 && returned.sub == 4 {
            receipts.push(OriginalTimingSemanticReceipt::OverworldMapQuadrantsPublished);
        }
        if entry.main == 15 && entry.sub == 0 && returned.main == 15 && returned.sub == 1 {
            receipts.push(OriginalTimingSemanticReceipt::DungeonExitSpotlightEntryReturned);
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
    /// caller-provided trace remains authoritative; this only adds the two
    /// generic domains/ranges required by the semantic adapter.
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
                    &["frame", "nmi", "wram", "rom-rng"],
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
                        "001a",
                        "02ec",
                        "0b00-0b1d",
                        "0b6a",
                        "0b89-0b98",
                        "0ba0-0baf",
                        "0bc0-0bdf",
                        "0c4a-0c53",
                        "0d00-0d3f",
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
        })
    }

    pub(crate) fn backing_path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn read_after_host_call(
        &mut self,
        dialogue_message_read_position: Option<u16>,
        spotlight_var4_low_at_return: Option<u8>,
    ) -> Result<Vec<OriginalTimingSemanticReceipt>, String> {
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
            host_frame.observe(&event)?;
            self.consume_event(event, &mut receipts)?;
        }
        if let Some(returned_event) = returned_event.as_ref() {
            self.finish_pending_spotlight_helper_nmi(
                returned_event,
                spotlight_var4_low_at_return,
                &mut receipts,
            )?;
            publish_spotlight_host_return_progress(
                returned_event,
                spotlight_var4_low_at_return,
                &mut receipts,
            )?;
            if publish_pre_dungeon_sprite_reset_host_return_progress(returned_event, &mut receipts)?
            {
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
        host_frame.finish(&mut receipts, dialogue_message_read_position)?;
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
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    ) -> Result<(), String> {
        let Some(helper_nmi) = self.pending_spotlight_helper_nmi.take() else {
            return Ok(());
        };
        let returned_pc = returned_event
            .pc
            .ok_or("Snes9x helper-interrupted host return omitted PC")?
            & 0x00ff_ffff;
        if returned_pc != NMI_HANDLER_ENTRY_PC {
            return Err(format!(
                "Snes9x spotlight helper NMI did not return at the source NMI entry: ${returned_pc:06x}"
            ));
        }
        let progress = spotlight_table_build_progress(&helper_nmi, spotlight_var4_low_at_return)?
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
    }

    fn consume_event(
        &mut self,
        event: RawTraceEvent,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    ) -> Result<(), String> {
        match event.event.as_str() {
            "wram-write" => {
                let pc = event.pc.ok_or("Snes9x WRAM write omitted PC")? & 0x00ff_ffff;
                let address = event.address.ok_or("Snes9x WRAM write omitted address")?;
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
                if event.main == Some(8)
                    && event.sub == Some(0)
                    && event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
                        (OVERWORLD_SPRITE_SCAN_START_PC..OVERWORLD_SPRITE_SCAN_END_PC).contains(&pc)
                    })
                {
                    self.publish_overworld_presence(receipts);
                }
                self.flush_host_boundary_progress(receipts, OriginalTimingBoundary::NmiAccepted);
                receipts.push(OriginalTimingSemanticReceipt::NmiAccepted);
                let inside_circle_value = spotlight_receipt_domain(&event)
                    && event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
                        (IRIS_SPOTLIGHT_CIRCLE_VALUE_START_PC..IRIS_SPOTLIGHT_CIRCLE_VALUE_END_PC)
                            .contains(&pc)
                    });
                if inside_circle_value {
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
                } else if let Some(progress) = spotlight_table_build_progress(&event, None)? {
                    receipts.push(OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                        SpotlightTableBuildProgressReceipt {
                            progress,
                            boundary: OriginalTimingBoundary::NmiAccepted,
                        },
                    ));
                }
                if let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) {
                    if let Some(phase) = main_loop_interruption_for_pc(pc) {
                        receipts.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(phase));
                    }
                }
            }
            _ => {}
        }
        Ok(())
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
    receipts: &mut Vec<OriginalTimingSemanticReceipt>,
) -> Result<(), String> {
    let Some(progress) = spotlight_table_build_progress(returned_event, spotlight_var4_low)? else {
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

fn publish_pre_dungeon_sprite_reset_host_return_progress(
    returned_event: &RawTraceEvent,
    receipts: &mut Vec<OriginalTimingSemanticReceipt>,
) -> Result<bool, String> {
    let pc = returned_event.pc.map(|pc| pc & 0x00ff_ffff);
    let return_address = returned_event
        .return_address
        .map(|address| address & 0x00ff_ffff);
    if !pc.is_some_and(|pc| {
        (SPRITE_RESET_ALL_NO_DISABLE_START_PC..SPRITE_RESET_ALL_END_PC).contains(&pc)
    }) || return_address != Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC)
    {
        return Ok(false);
    }
    if (returned_event.main, returned_event.sub) != (Some(6), Some(0)) {
        return Err(format!(
            "Snes9x Module_PreDungeon Sprite_ResetAll return had module {:?}/{:?}",
            returned_event.main, returned_event.sub,
        ));
    }
    receipts.retain(|receipt| {
        !matches!(
            receipt,
            OriginalTimingSemanticReceipt::SpriteResetAllProgress(_)
        )
    });
    receipts.push(OriginalTimingSemanticReceipt::SpriteResetAllProgress(
        SpriteResetAllProgressReceipt {
            progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
            boundary: OriginalTimingBoundary::HostReturn,
        },
    ));
    Ok(true)
}

fn spotlight_table_build_progress(
    event: &RawTraceEvent,
    spotlight_var4_low: Option<u8>,
) -> Result<Option<SpotlightTableBuildProgress>, String> {
    let pc = event.pc.map(|pc| pc & 0x00ff_ffff);
    let inside_circle_value = pc.is_some_and(|pc| {
        (IRIS_SPOTLIGHT_CIRCLE_VALUE_START_PC..IRIS_SPOTLIGHT_CIRCLE_VALUE_END_PC).contains(&pc)
    });
    if !inside_circle_value
        && !matches!(
            pc,
            Some(
                IRIS_SPOTLIGHT_CIRCLE_VALUE_CALL_PC
                    | IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC
                    | IRIS_SPOTLIGHT_LOWER_TABLE_WRITE_PC
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
    let projection_checkpoint = !inside_circle_value
        && !matches!(
            pc,
            Some(
                IRIS_SPOTLIGHT_CIRCLE_VALUE_CALL_PC
                    | IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC
                    | IRIS_SPOTLIGHT_LOWER_TABLE_WRITE_PC
            )
        );
    let (completed_iterations, checkpoint) = if inside_circle_value
        || matches!(
            pc,
            Some(IRIS_SPOTLIGHT_CIRCLE_VALUE_CALL_PC | IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC)
        ) {
        let pending_circle_input =
            if inside_circle_value || pc == Some(IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC) {
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
        if pending_circle_input == 0 || u16::from(pending_circle_input) > radius {
            return Err(format!(
                "Snes9x spotlight circle input {pending_circle_input} is not derivable from radius {radius}",
            ));
        }
        let completed_iterations = iterations_before_iris
            .checked_add(radius - u16::from(pending_circle_input))
            .ok_or("Snes9x spotlight iteration count overflowed")?;
        if pc == Some(IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC) {
            let doubled_upper_cursor = event
                .x
                .ok_or("Snes9x spotlight upper-table checkpoint omitted X")?;
            if doubled_upper_cursor & 1 != 0 {
                return Err(format!(
                    "Snes9x spotlight upper cursor encoded an odd table byte offset {doubled_upper_cursor}",
                ));
            }
            let initial_upper_cursor = vertical_center
                .wrapping_mul(2)
                .wrapping_sub(initial_lower_cursor);
            let cursor_iterations = (doubled_upper_cursor >> 1)
                .checked_sub(initial_upper_cursor)
                .ok_or("Snes9x spotlight upper cursor preceded its source initial value")?;
            if cursor_iterations != completed_iterations {
                return Err(format!(
                    "Snes9x spotlight upper cursor derived {cursor_iterations} iterations but spotlight_var4 derived {completed_iterations}",
                ));
            }
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

    fn raw(event: &str, pc: Option<u32>, x: Option<u16>, address: Option<u16>) -> RawTraceEvent {
        RawTraceEvent {
            event: event.to_string(),
            stage: None,
            run: None,
            pc,
            return_address: None,
            a: None,
            main: None,
            sub: None,
            subsub: None,
            frame_counter: None,
            nmi_latch: None,
            link_y: None,
            bg2_v: None,
            spotlight_radius: None,
            x,
            address,
            value: address.map(|_| 0),
        }
    }

    fn frame(stage: &str, run: u64, main: u8, frame_counter: u8) -> RawTraceEvent {
        RawTraceEvent {
            event: "frame".to_string(),
            stage: Some(stage.to_string()),
            run: Some(run),
            pc: Some(0),
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
            x: None,
            address: None,
            value: None,
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
            host.finish(&mut receipts, None).unwrap();
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
        host.finish(&mut receipts, Some(0x0037)).unwrap();

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
            host.finish(&mut receipts, None).unwrap();
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

        host.finish(&mut receipts, None).unwrap();

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

        host.finish(&mut receipts, None).unwrap();

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
    fn dungeon_exit_spotlight_entry_return_becomes_a_backend_neutral_receipt() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 39630, 15, 0))
            .unwrap();
        host.observe(&frame_with_sub("return", 39630, 15, 1))
            .unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None).unwrap();

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

        host.finish(&mut receipts, None).unwrap();

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
        idle.finish(&mut idle_receipts, None).unwrap();
        assert_eq!(
            idle_receipts,
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                MainLoopProgress::CallStackContinued,
            )],
            "an idle main-wait host did not resume a suspended C caller",
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

        host.finish(&mut receipts, None).unwrap();

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
        interrupted.finish(&mut interrupted_receipts, None).unwrap();
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
        };
        let mut receipts = Vec::new();
        for _ in 0..2 {
            let mut event = raw("nmi", Some(OVERWORLD_SPRITE_SCAN_START_PC + 1), None, None);
            event.main = Some(8);
            event.sub = Some(0);
            tracker.consume_event(event, &mut receipts).unwrap();
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
                .filter(|receipt| matches!(receipt, OriginalTimingSemanticReceipt::NmiAccepted))
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

        host.finish(&mut receipts, None).unwrap();

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
        host.finish(&mut receipts, None).unwrap();

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

        host.finish(&mut receipts, None).unwrap();

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
            host.finish(&mut receipts, None).unwrap_err(),
            "Snes9x host call started ZeldaRunGameLoop 2 times; expected zero or one",
        );
        assert!(receipts.is_empty());
    }

    #[test]
    fn cold_initialization_frame_counter_clear_is_not_a_main_loop_start() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame("entry", 80, 0x55, 0x55)).unwrap();
        host.observe(&raw(
            "wram-write",
            Some(0x0087ce),
            None,
            Some(FRAME_COUNTER),
        ))
        .unwrap();
        host.observe(&frame("return", 80, 0, 0)).unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                MainLoopProgress::CallStackContinued,
            )],
        );
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
        };
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw("nmi", Some(0x00_8751), None, None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted,
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpritePreparation,
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
        };
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw("nmi", Some(0x0d_a9d0), None, None), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted,
                OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam,),
            ],
        );
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
                OriginalTimingSemanticReceipt::NmiAccepted,
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
        assert_eq!(receipts, vec![OriginalTimingSemanticReceipt::NmiAccepted],);
        let mut returned = frame("return", 40977, 0x0f, 253);
        returned.pc = Some(NMI_HANDLER_ENTRY_PC);
        tracker
            .finish_pending_spotlight_helper_nmi(&returned, Some(125), &mut receipts)
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted,
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

        publish_spotlight_host_return_progress(&returned, Some(26), &mut receipts).unwrap();

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
                OriginalTimingSemanticReceipt::NmiAccepted,
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

        assert_eq!(receipts, vec![OriginalTimingSemanticReceipt::NmiAccepted],);
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
                OriginalTimingSemanticReceipt::NmiAccepted,
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

        assert_eq!(receipts, vec![OriginalTimingSemanticReceipt::NmiAccepted]);
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
                OriginalTimingSemanticReceipt::NmiAccepted,
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
            OriginalTimingSemanticReceipt::NmiAccepted,
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

        publish_spotlight_host_return_progress(&returned, None, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted,
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

        assert!(
            publish_pre_dungeon_sprite_reset_host_return_progress(&returned, &mut receipts,)
                .unwrap()
        );

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
        assert!(!publish_pre_dungeon_sprite_reset_host_return_progress(
            &returned,
            &mut wrong_caller,
        )
        .unwrap());
        assert!(wrong_caller.is_empty());
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
        host.finish(&mut receipts, None).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::IterationStarted,),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam,),
            ],
        );
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
            .consume_event(raw("nmi", None, None, None), &mut receipts)
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
                OriginalTimingSemanticReceipt::NmiAccepted,
                OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
                    CachedSpriteExecutionProgressReceipt {
                        progress: CachedSpriteExecutionProgress::Restoring {
                            slot: 7,
                            live_fields: 20,
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted,
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
                OriginalTimingSemanticReceipt::NmiAccepted,
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
            .consume_event(raw("nmi", None, None, None), &mut receipts)
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
                OriginalTimingSemanticReceipt::NmiAccepted,
                OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                    DungeonResetSpritesProgressReceipt {
                        progress: DungeonResetSpritesCpuProgress::Cache {
                            slot: 14,
                            field: CachedSpriteCacheField::StateClear,
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted,
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
                OriginalTimingSemanticReceipt::NmiAccepted,
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
        };
        let mut receipts = Vec::new();
        for line in include_str!(
            "../../external/snes9x-libretro/fixtures/zelda3-dungeon-reset-sprites-yhigh-nmi.jsonl"
        )
        .lines()
        {
            trace
                .consume_event(serde_json::from_str(line).unwrap(), &mut receipts)
                .unwrap();
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
                OriginalTimingSemanticReceipt::NmiAccepted,
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
        assert_eq!(receipts, vec![OriginalTimingSemanticReceipt::NmiAccepted]);
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

        assert_eq!(receipts, vec![OriginalTimingSemanticReceipt::NmiAccepted]);
    }

    #[test]
    fn csv_extension_is_deduplicated_and_preserves_existing_domains() {
        assert_eq!(
            append_csv(Some("frame,wram"), &["nmi", "wram"]),
            "frame,wram,nmi"
        );
    }
}
