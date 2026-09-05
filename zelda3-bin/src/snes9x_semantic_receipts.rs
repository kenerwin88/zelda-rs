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
    CachedSpriteCacheField, CachedSpriteExecutionBodyProgress, CachedSpriteExecutionProgress,
    CachedSpriteExecutionProgressReceipt, CreditsEndSequence32ProgressReceipt,
    CreditsSceneLoadProgress, CreditsSceneLoadProgressReceipt, DialogueExecutionProgress,
    DungeonFallingEntranceProgress, DungeonLoadSpritesCpuProgress,
    DungeonPegAttributeFlipProgressReceipt, DungeonResetSpritesCpuProgress,
    DungeonResetSpritesProgressReceipt, DungeonSpriteDisableCpuProgress,
    DungeonSpriteLoadCheckpoint, FileSelectGraphicsLowWramClearProgress, ItemReceiptGraphicsCaller,
    ItemReceiptGraphicsProgressReceipt, JoypadPublication, MainLoopInterruption, MainLoopProgress,
    NmiPpuRegisterOperands, NmiUpdateGate, OriginalTimingBoundary, OriginalTimingSemanticReceipt,
    OverworldSpriteReloadProgress, PreOverworldStageCompletion,
    RescuedMaidenInitializationProgressReceipt, RescuedMaidenInitializationStage,
    RescuedMaidenTilemapClearProgressReceipt, SaveMenuInitializationProgress, SourceCallProgress,
    SpotlightTableBuildCheckpoint, SpotlightTableBuildProgress, SpotlightTableBuildProgressReceipt,
    SpriteDynamicSpawnProgress, SpriteFollowerGraphicsCaller, SpriteInitializeResetPropertiesPhase,
    SpriteMainProgress, SpriteMoveXYCheckpoint, SpriteResetAllProgress,
    SpriteResetAllProgressReceipt, TriforceRoomCase2PaletteProgressReceipt,
};

const TRACE_PATH_ENV: &str = "ZELDA3_SNES9X_TRACE";
const TRACE_EVENTS_ENV: &str = "ZELDA3_SNES9X_TRACE_EVENTS";
const TRACE_WRAM_ENV: &str = "ZELDA3_SNES9X_TRACE_WRAM";
const TRACE_PCS_ENV: &str = "ZELDA3_SNES9X_TRACE_PCS";
const DIALOGUE_SCROLL_ENTRY_PC: u32 = 0x0e_cfe2;
const DIALOGUE_SCROLL_PIXEL_COMPLETED_PC: u32 = 0x0e_d088;
const DIALOGUE_SCROLL_RETURN_PC: u32 = 0x0e_d0c2;

#[derive(Default)]
struct DialogueScrollHostWindow {
    completed: Vec<zelda3::DialogueScrollProgressReceipt>,
    active: Option<zelda3::DialogueScrollProgressReceipt>,
}

impl DialogueScrollHostWindow {
    fn observe(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return Ok(());
        };
        if event.event == "pc" {
            match pc {
                DIALOGUE_SCROLL_ENTRY_PC => {
                    if self.active.is_some() {
                        return Err("Snes9x reentered an active dialogue scroll call".to_string());
                    }
                    self.active = Some(zelda3::DialogueScrollProgressReceipt {
                        entered: true,
                        ..Default::default()
                    });
                }
                DIALOGUE_SCROLL_PIXEL_COMPLETED_PC => {
                    let progress = self.active.get_or_insert_with(Default::default);
                    if progress.completed_pixel_passes == 16 {
                        return Err(
                            "Snes9x dialogue scroll exceeded one line in one call".to_string()
                        );
                    }
                    progress.completed_pixel_passes += 1;
                }
                DIALOGUE_SCROLL_RETURN_PC => {
                    let mut progress = self.active.take().unwrap_or_default();
                    progress.returned = true;
                    self.completed.push(progress);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn finish(mut self) -> Vec<zelda3::DialogueScrollProgressReceipt> {
        if let Some(progress) = self.active {
            self.completed.push(progress);
        }
        self.completed
    }
}
const REQUIRED_TRACE_EVENTS: &[&str] = &["frame", "nmi", "nmi-resume", "wram", "rom-rng", "pc"];

const FRAME_COUNTER: u16 = 0x001a;
const NMI_UPDATE_LATCH: u16 = 0x0012;
const SUBMODULE_INDEX: u16 = 0x0011;
const SUBSUBMODULE_INDEX: u16 = 0x00b0;
const CREDITS_SCENE_OVERWORLD_SUBSUBMODULE_INCREMENT_PC: u32 = 0x02_8696;
const CREDITS_ENDING_TEXT_BEFORE_TILE_COPY_PC: u32 = 0x0e_c34b;
const CREDITS_END_SEQUENCE_32_SAVE_CHECKSUM_LOOP_PC: u32 = 0x00_899c;
// Source statements inside Module11_02's long room load. These post-write
// PCs are pinned-adapter provenance only; gameplay receives the typed
// `DungeonFallingEntranceProgress` facts below.
const FALLING_ENTRANCE_ROOM_PARSER_SUBSUB_CLEAR_PC: u32 = 0x02_c5b1;
const FALLING_ENTRANCE_SUBSUB_ADVANCE_PC: u32 = 0x02_9b9d;
const FALLING_ENTRANCE_SONG_BANK_TAIL_PC: u32 = 0x02_9bd7;
// The rescued-maiden transition clears four BG2 and then four BG1 1,024-word
// regions for each even X cursor. Snes9x reports the opcode-postfetch PC which
// the interrupted stack resumes at; these are private adapter coordinates for
// translating that position to an exact source-order store count.
const RESCUED_MAIDEN_TILEMAP_CLEAR_FIRST_STORE_PC: u32 = 0x02_984a;
const RESCUED_MAIDEN_TILEMAP_CLEAR_SECOND_STORE_PC: u32 = 0x02_984e;
const RESCUED_MAIDEN_TILEMAP_CLEAR_THIRD_STORE_PC: u32 = 0x02_9852;
const RESCUED_MAIDEN_TILEMAP_CLEAR_FOURTH_STORE_PC: u32 = 0x02_9856;
const RESCUED_MAIDEN_TILEMAP_CLEAR_FIFTH_STORE_PC: u32 = 0x02_985a;
const RESCUED_MAIDEN_TILEMAP_CLEAR_SIXTH_STORE_PC: u32 = 0x02_985e;
const RESCUED_MAIDEN_TILEMAP_CLEAR_SEVENTH_STORE_PC: u32 = 0x02_9862;
const RESCUED_MAIDEN_TILEMAP_CLEAR_EIGHTH_STORE_PC: u32 = 0x02_9866;
const RESCUED_MAIDEN_TILEMAP_CLEAR_FIRST_INX_PC: u32 = 0x02_986a;
const RESCUED_MAIDEN_TILEMAP_CLEAR_SECOND_INX_PC: u32 = 0x02_986b;
const RESCUED_MAIDEN_TILEMAP_CLEAR_COMPARE_PC: u32 = 0x02_986c;
const RESCUED_MAIDEN_TILEMAP_CLEAR_BRANCH_PC: u32 = 0x02_986f;
// Source call boundaries inside the rescued-maiden state's synchronous
// follower-graphics load. Only these sparse call sites are traced; the exact
// decompressor output cursor comes from Y on the host/NMI boundary event, so
// route-wide tracing does not record one event per output byte.
const RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC: u32 = 0x00_d423;
const RESCUED_MAIDEN_FIRST_FOLLOWER_SHEET_ENTRY_PC: u32 = 0x00_e75c;
const RESCUED_MAIDEN_SECOND_FOLLOWER_SHEET_ENTRY_PC: u32 = 0x00_e766;
const RESCUED_MAIDEN_FOLLOWER_SHEETS_RETURN_PC: u32 = 0x00_d44c;
const FOLLOWER_GRAPHICS_CONVERSION_START_PC: u32 = 0x00_d5ce;
const FOLLOWER_GRAPHICS_CONVERSION_END_PC: u32 = 0x00_d619;
const FOLLOWER_GRAPHICS_CONVERSION_DESTINATION_X: u16 = 0x2940;
const FOLLOWER_GRAPHICS_CONVERSION_STORES: u16 = 32 * 8 * 2;
// Return addresses of the Sprite_Main-owned calls to LoadFollowerGraphics.
// The active slot plus the concrete caller distinguishes the two state-8
// preparation paths, the Old Man's state-8 prep, and Blind Maiden's state-9
// become-follower body from the many other callers of the shared loader.
const SPRITE_PREP_BLIND_MAIDEN_FOLLOWER_GRAPHICS_RETURN_PC: u32 = 0x06_89c2;
const SPRITE_PREP_ZELDA_FOLLOWER_GRAPHICS_RETURN_PC: u32 = 0x05_ebf5;
const SPRITE_BLIND_MAIDEN_BODY_FOLLOWER_GRAPHICS_RETURN_PC: u32 = 0x1e_e8ea;
const SPRITE_PREP_OLD_MAN_FOLLOWER_GRAPHICS_RETURN_PC: u32 = 0x1e_e925;
const SPRITE_PURPLE_CHEST_FOLLOWER_GRAPHICS_RETURN_PC: u32 = 0x1e_e10a;
// `Sprite_Zazak_Main`'s animation publication. The write event proves the
// current slot's source-selected graphics byte committed before the boundary.
const SPRITE_ZAZAK_GRAPHICS_STORE_PC: u32 = 0x1e_91fb;
const RESCUED_MAIDEN_DECOMPRESS_BODY_START_PC: u32 = 0x00_e79e;
const RESCUED_MAIDEN_DECOMPRESS_BODY_END_PC: u32 = 0x00_e851;
// Snes9x reports the post-fetch PC $e7f4 while the preceding `STA [$00],Y`
// has committed but its following INY has not. Every other supported body PC
// exposes Y directly as the number of committed output bytes.
const RESCUED_MAIDEN_DECOMPRESS_STORE_POSTFETCH_PC: u32 = 0x00_e7f4;
const RESCUED_MAIDEN_FOLLOWER_SHEET_BYTES: u16 = 0x0600;
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
// `$00:F378..$00:F382` is the publication-free caller suffix after the pure
// circle helper returns: it loads and doubles the upper cursor, then evaluates
// the upper-table guard. Neither table word has been stored yet, so these
// instruction boundaries resume from the same source checkpoint as the pure
// helper call. `$00:F383` is the first upper-table store itself.
const IRIS_SPOTLIGHT_AFTER_CIRCLE_VALUE_START_PC: u32 = 0x00f378;
// ROM $00:f361 loads the constant for the direct-page store at $00:f364 which
// initializes the current loop iteration's `r8 = 0xff`. From the load through
// that store, the initialization and every table publication for the iteration
// remain pending. The adapter derives the completed C iteration count from the
// source `r6` cursor captured at the boundary; gameplay never observes this ROM
// address or scratch register.
const IRIS_SPOTLIGHT_ITERATION_VALUE_LOAD_PC: u32 = 0x00f361;
const IRIS_SPOTLIGHT_ITERATION_VALUE_STORE_PC: u32 = 0x00f364;
// `$00:F366..$00:F374` only evaluates the new iteration's upper-bound branch
// and, on the active-circle path, tests whether `spotlight_var4` will be
// decremented. The complete `$00:F361..$00:F374` prefix therefore rewinds to
// one backend-neutral iteration-start checkpoint.
// The pure circle helper has returned and the source loop has doubled its
// upper cursor, but neither HDMA-table store has executed. Rewind this
// emulator-private instruction boundary to the same resumable C checkpoint as
// the helper call: recalculating the pure value cannot replay a publication.
const IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC: u32 = 0x00f383;
// After the upper long store completes, $00:F387..$00:F391 evaluates the
// guarded lower-table write. An NMI here has published the upper word but may
// have repurposed A/X for the guard, so expose the source statement rather
// than trying to recover the prior accumulator value.
const IRIS_SPOTLIGHT_AFTER_UPPER_TABLE_WRITE_START_PC: u32 = 0x00f387;
const IRIS_SPOTLIGHT_LOWER_TABLE_WRITE_PC: u32 = 0x00f392;
// The lower long store has returned at `$00:F396`; LDA/CMP then prepare the
// source loop-completion test whose branch begins at `$00:F39A`. All three
// instruction boundaries name the same source checkpoint: both table writes
// are published and the loop test itself remains pending.
const IRIS_SPOTLIGHT_BEFORE_LOOP_COMPLETION_TEST_START_PC: u32 = 0x00f396;
const DESERT_PRAYER_IRIS_ENTRY_PC: u32 = 0x07ea27;
const DESERT_PRAYER_IRIS_LOWER_Y_PUBLISHED_PC: u32 = 0x07ea40;
const DESERT_PRAYER_IRIS_UPPER_Y_PUBLISHED_PC: u32 = 0x07ea4f;
const DESERT_PRAYER_IRIS_X_CENTER_PUBLISHED_PC: u32 = 0x07ea5b;
const DESERT_PRAYER_IRIS_CURSOR_PUBLISHED_PC: u32 = 0x07ea61;
const DESERT_PRAYER_IRIS_EARLY_ITERATION_END_PC: u32 = 0x07ea79;
const DESERT_PRAYER_IRIS_RADIAL_BRANCH_START_PC: u32 = 0x07ea7c;
const DESERT_PRAYER_IRIS_RADIAL_BRANCH_END_PC: u32 = 0x07ea7f;
const DESERT_PRAYER_IRIS_RADIAL_CALCULATION_START_PC: u32 = 0x07ea97;
const DESERT_PRAYER_IRIS_BEFORE_LOWER_ZERO_WRITE_PC: u32 = 0x07ea9e;
const DESERT_PRAYER_IRIS_PRIMARY_VALUE_START_PC: u32 = 0x07eaa1;
const DESERT_PRAYER_IRIS_PRIMARY_INDEX_IN_X_PC: u32 = 0x07eaca;
const DESERT_PRAYER_IRIS_PRIMARY_TABLE_WRITE_PC: u32 = 0x07eb12;
const DESERT_PRAYER_IRIS_AFTER_PRIMARY_TABLE_WRITE_PC: u32 = 0x07eb15;
const DESERT_PRAYER_IRIS_BEFORE_MIRRORED_TABLE_WRITE_PC: u32 = 0x07eb43;
const DESERT_PRAYER_IRIS_AFTER_ITERATION_PCS: [u32; 4] = [0x07eb4b, 0x07eb4f, 0x07eb52, 0x07eb54];
const DESERT_PRAYER_IRIS_LOOP_COMPLETE_START_PC: u32 = 0x07eb57;
const DESERT_PRAYER_IRIS_STATE4_TAIL_START_PC: u32 = 0x07eb66;
const DESERT_PRAYER_IRIS_SHAPE_HELPER_START_PC: u32 = 0x07ecdc;
const DESERT_PRAYER_IRIS_SHAPE_HELPER_END_PC: u32 = 0x07ed2c;
const PALETTE_FILTER_BEFORE_COLOR_LOAD_PC: u32 = 0x00e9e4;
const PALETTE_FILTER_BEFORE_COLOR_STORE_PC: u32 = 0x00ea30;
// ROM $00:f392 is the long WRAM store of the already-calculated circle value
// to `hdma_table_dynamic[r6]`. The upper-cursor store at $00:f383 is complete;
// this lower store and the loop-cursor update remain pending.
// At $00:f39a both table stores are complete and the source loop has compared
// its upper cursor with the vertical center. The branch and any cursor update
// remain pending. The adapter converts the private X register into the two C
// cursors and exports only that resumable statement boundary.
const IRIS_SPOTLIGHT_LOOP_COMPLETION_BRANCH_PC: u32 = 0x00f39a;
// At the fallthrough INC r4 opcode the branch has completed, but neither
// cursor has changed. Re-evaluating the pure loop test has the same source
// state; keep this boundary in the existing pre-test continuation domain.
const IRIS_SPOTLIGHT_UPPER_CURSOR_INCREMENT_PC: u32 = 0x00f39c;
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
// The completed row loop polls the beam counter before copying its table.
// No projection word is stored at any instruction in this wait loop.
const IRIS_SPOTLIGHT_BEAM_WAIT_PCS: [u32; 6] =
    [0x00f3a3, 0x00f3a6, 0x00f3a9, 0x00f3ac, 0x00f3af, 0x00f3b2];
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
const GAME_OVER_IRIS_PALETTE_FIRST_STORE_PC: u32 = 0x09f37a;
const GAME_OVER_IRIS_PALETTE_SECOND_STORE_PC: u32 = 0x09f37e;
const GAME_OVER_IRIS_PALETTE_THIRD_STORE_PC: u32 = 0x09f382;
const GAME_OVER_IRIS_PALETTE_FOURTH_STORE_PC: u32 = 0x09f386;
const GAME_OVER_IRIS_PALETTE_FIFTH_STORE_PC: u32 = 0x09f38a;
const GAME_OVER_IRIS_PALETTE_SIXTH_STORE_PC: u32 = 0x09f38e;
const GAME_OVER_IRIS_PALETTE_FIRST_INCREMENT_PC: u32 = 0x09f391;
const GAME_OVER_IRIS_PALETTE_SECOND_INCREMENT_PC: u32 = 0x09f392;
const GAME_OVER_IRIS_PALETTE_COMPARE_PC: u32 = 0x09f393;
const GAME_OVER_IRIS_PALETTE_BRANCH_PC: u32 = 0x09f396;
const IRIS_SPOTLIGHT_RESET_TABLE_INITIAL_X: u16 = 0x3e;
const IRIS_SPOTLIGHT_RESET_TABLE_STORES_PER_ITERATION: u16 = 7;
const IRIS_SPOTLIGHT_CIRCLE_VALUE_START_PC: u32 = 0x00f4cc;
const IRIS_SPOTLIGHT_CIRCLE_VALUE_END_PC: u32 = 0x00f53e;
pub(crate) const SPOTLIGHT_VAR4_LOW_ADDRESS: usize = 0x067a;
pub(crate) const SPOTLIGHT_LOWER_CURSOR_ADDRESS: usize = 0x0006;
const NMI_HANDLER_ENTRY_PC: u32 = 0x0080c9;
// `$09:F825` is the common render-call entry reached by Zelda's IRQ-driven
// poly worker after its go/upload-byte loop admits the next frame. The PC is
// private adapter provenance; gameplay receives only the source call-start
// fact, and only for the preemptive dungeon/Triforce users modeled there.
const POLYHEDRAL_RENDER_START_PC: u32 = 0x09f825;
const DUNGEON_CACHE_TRANS_SPRITES_START_PC: u32 = 0x09c176;
const DUNGEON_CACHE_TRANS_SPRITES_END_PC: u32 = 0x09c244;
const DUNGEON_RESET_SPRITES_CLEAR_PC: u32 = 0x09c244;
// After Sprite_DisableAll returns, Dungeon_ResetSprites publishes the paired
// collision sizes and performs a read-only four-entry room-history search.
// An interrupt anywhere in the search can resume from its source start: no
// history mutation has occurred yet. These private PCs distinguish that
// caller checkpoint from the completed callee boundary at $09:C244..C290.
const DUNGEON_RESET_SPRITES_AFTER_DISABLE_PC: u32 = 0x09c124;
const DUNGEON_RESET_SPRITES_COLLISION_Y_STORE_PC: u32 = 0x09c12c;
const DUNGEON_RESET_SPRITES_HISTORY_SEARCH_START_PC: u32 = 0x09c12f;
const DUNGEON_RESET_SPRITES_HISTORY_FIRST_MUTATION_PC: u32 = 0x09c148;
const DUNGEON_RESET_SPRITES_HISTORY_FOUND_PC: u32 = 0x09c16e;
const DUNGEON_RESET_SPRITES_LOAD_CALL_PC: u32 = 0x09c170;
// First instruction after Dungeon_LoadSprites returns to Dungeon_ResetSprites.
// Any partial cache/disable/load cursor is retired here even when the room had
// no new sprite-record writes to supersede it.
const DUNGEON_RESET_SPRITES_RETURN_PC: u32 = 0x09c173;
const SPRITE_DISABLE_ALL_END_PC: u32 = 0x09c290;
const SPRITE_DISABLE_ALL_FINAL_GARNISH_PC: u32 = 0x09c281;
const GARNISH_TYPE_SLOT_ZERO: u16 = 0x0b00;
const ANCILLA_TYPE_BASE: u16 = 0x0c4a;
const ANCILLA_PICKUP_FLAG: u16 = 0x02ec;
const SPRITE_LIMIT_INSTANCE: u16 = 0x0b6a;
const DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC: u32 = 0x09c38c;
const DUNGEON_LOAD_SINGLE_SPRITE_TEMP_Y_PC: u32 = 0x09c391;
const DUNGEON_LOAD_SINGLE_SPRITE_FLOOR_PC: u32 = 0x09c398;
const DUNGEON_LOAD_SINGLE_SPRITE_Y_LOW_PC: u32 = 0x09c3a1;
const DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC: u32 = 0x09c3a9;
const DUNGEON_LOAD_SINGLE_SPRITE_SHARED_X_PC: u32 = 0x09c3af;
const DUNGEON_LOAD_SINGLE_SPRITE_X_LOW_PC: u32 = 0x09c3b6;
const DUNGEON_LOAD_SINGLE_SPRITE_X_HIGH_PC: u32 = 0x09c3be;
const DUNGEON_LOAD_SINGLE_SPRITE_TYPE_PC: u32 = 0x09c3c4;
const DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_CLEAR_PC: u32 = 0x09c3c7;
const DUNGEON_LOAD_SINGLE_SPRITE_TEMP_SUBTYPE_PC: u32 = 0x09c3d1;
const DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_FINAL_PC: u32 = 0x09c3df;
const DUNGEON_LOAD_SINGLE_SPRITE_SPAWN_INDEX_PC: u32 = 0x09c3e4;
const DUNGEON_LOAD_SINGLE_SPRITE_COMPLETE_PC: u32 = 0x09c3e7;
const DUNGEON_LOAD_SINGLE_SPRITE_END_PC: u32 = 0x09c3e8;
// `Module_PreDungeon` calls `Sprite_ResetAll` at $02:8347; the return address
// exposed by the pinned trace is $02:834b. The shared reset routine itself is
// adapter-private provenance; gameplay receives only its semantic checkpoint.
const MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC: u32 = 0x02_834b;
// `Overworld_LoadBirdTravelPos` performs an initial `Sprite_ResetAll`, then
// immediately enters `Sprite_ReloadAll_Overworld`. These caller return PCs
// distinguish its two source reset phases from every other shared caller.
const BIRD_TRAVEL_AFTER_INITIAL_SPRITE_RESET_PC: u32 = 0x02_ecd2;
const BIRD_TRAVEL_AFTER_SPRITE_RELOAD_PC: u32 = 0x02_ecd6;
const SPRITE_RELOAD_AFTER_DISABLE_PC: u32 = 0x09_c4a0;
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
// Return edge of Module0F's indirect submodule call, before its Link suffix.
// The address is private adapter evidence; the receipt exports only the C
// source boundary.
const MODULE0F_AFTER_SUBMODULE_DISPATCH_PC: u32 = 0x02998d;
// JSL Link_HandleVelocity, after Module0F's ripple/speed prefix stores.
const MODULE0F_LINK_VELOCITY_CALL_PC: u32 = 0x0299a0;
// The generic PC trace uses these private pinned-ROM boundaries to translate
// the descending `Sprite_Main` loop into a Zelda-level resumable slot receipt.
// Neither address nor the CPU X register crosses the adapter boundary.
const SPRITE_MAIN_ENTRY_PC: u32 = 0x068328;
const SPRITE_EXECUTE_SINGLE_ENTRY_PC: u32 = 0x0684e2;
const SPRITE_ACTIVE_MAIN_ENTRY_PC: u32 = 0x069271;
// Final weapon entry's flags store in `Guard_AnimateWeapon`. At this opcode
// the entry's X, Y, and character bytes have committed, while flags and the
// following bytewise-extended-OAM store have not.
const GUARD_ANIMATE_WEAPON_FLAGS_STORE_PC: u32 = 0x05cbcd;
// CMP #$80 after the fast parry hitbox and position-mode branch. Include
// its operand byte: Snes9x can expose an in-instruction PC at host return.
const GUARD_PARRY_HITBOX_COMPARE_PC: u32 = 0x06eb94;
// First instruction after Sprite_TimersAndOam's last countdown update. The
// helper's floor/priority suffix and the state-dispatched body remain pending.
// `$06:84A4` is the first instruction after the final countdown update in
// Sprite_TimersAndOam.  Through the helper's RTS at `$06:84B8`, the countdown
// prefix is complete while the floor/priority suffix may still be in flight.
const SPRITE_TIMER_DECREMENTS_COMPLETE_START_PC: u32 = 0x0684a4;
const SPRITE_TIMER_DECREMENTS_COMPLETE_END_PC: u32 = 0x0684b9;
const SPRITE_TIMER_DECREMENTS_TRACE_PC: u32 = 0x0684aa;
// The four leading countdown statements (`delay_main`, aux1, aux2, aux3)
// are complete once the ROM begins loading `sprite_hit_timer`. Include the
// possible mid-instruction host-return PCs through the AND operand; no hit-
// timer statement has published at any address in this interval.
const SPRITE_PRIMARY_TIMER_DECREMENTS_COMPLETE_START_PC: u32 = 0x068444;
const SPRITE_PRIMARY_TIMER_DECREMENTS_COMPLETE_END_PC: u32 = 0x068449;
// When `sprite_hit_timer & $7f` is zero, the branch at `$06:8446` jumps
// directly to the `STZ sprite_hit_timer,X` instruction. Its opcode and operand
// fetches are still before that store publishes, so they name the same exact
// source boundary as the linear hit-timer load/branch interval above.
const SPRITE_PRIMARY_TIMER_DECREMENTS_ZERO_HIT_STORE_START_PC: u32 = 0x068496;
const SPRITE_PRIMARY_TIMER_DECREMENTS_ZERO_HIT_STORE_END_PC: u32 = 0x068499;
// `$06:8432` begins the aux2 load after the main/aux1 countdown statements.
// An NMI may be accepted during that instruction, before aux2 has executed.
const SPRITE_MAIN_AND_AUX1_TIMER_DECREMENTS_COMPLETE_START_PC: u32 = 0x068432;
const SPRITE_MAIN_AND_AUX1_TIMER_DECREMENTS_COMPLETE_END_PC: u32 = 0x068437;
// First instruction after Sprite_ExecuteSingle's shared
// Sprite_TimersAndOam call returns, before the saved dispatch state is
// restored from the stack. The adapter exports only that C-call boundary.
const SPRITE_TIMERS_AND_OAM_RETURN_PC: u32 = 0x0684eb;
const PALETTE_LOAD_MULTIPLE_BEFORE_WORD_COPY_PC: u32 = 0x1bef5f;
const OVERWORLD_PARSE_MAP32_DEFINITION_SECOND_WORD_PC: u32 = 0x02f695;
// `SpritePrep_Bari` has completed its fixed Z publication and room-$ce
// conditional when execution reaches the RNG tail at `$06:8B2B`. A libretro
// host may return on the preceding BNE instruction, so include the complete
// compare/branch range as the same source boundary.
const SPRITE_BARI_BEFORE_RANDOM_START_PC: u32 = 0x068b24;
const SPRITE_BARI_BEFORE_RANDOM_END_PC: u32 = 0x068b2e;
const SPRITE_SLOT_RETURN_PC: u32 = 0x0683a7;
const SPRITE_MAIN_RETURN_PC: u32 = 0x028842;
// `SpriteDraw_Antfairy` has published its source-visible subtype increment
// at this store. Its animation/draw suffix and the caller-specific sprite
// body remain pending; the adapter exports only that semantic statement.
const ANTFAIRY_SUBTYPE2_INCREMENT_PC: u32 = 0x1df39b;
// `HelmasaurHardHatBeetleCommon` has passed its inactive check and published
// the subtype2 increment shared by Mini Helmasaur and Hardhat Beetle.
const HELMASAUR_HARD_HAT_BEETLE_SUBTYPE2_INCREMENT_PC: u32 = 0x06a473;
// `Lanmola_Draw` has published its graphics/history prefix and the leading
// subtype2 increment at this source store. Its remaining draw and AI body are
// still pending.
const LANMOLA_SUBTYPE2_INCREMENT_PC: u32 = 0x05a6bd;
const DUNGEON_PEG_FLIP_LOOP_START_PC: u32 = 0x01c22f;
const DUNGEON_PEG_FLIP_BANK_B_PC: u32 = 0x01c241;
const DUNGEON_PEG_FLIP_BANK_C_PC: u32 = 0x01c253;
const DUNGEON_PEG_FLIP_BANK_D_PC: u32 = 0x01c265;
const DUNGEON_PEG_FLIP_DECREMENT_PC: u32 = 0x01c277;
const DUNGEON_PEG_FLIP_BRANCH_PC: u32 = 0x01c278;
const DUNGEON_PEG_FLIP_INDEX_EXHAUSTED_PC: u32 = 0x01c27a;
const DUNGEON_PEG_FLIP_RETURN_PC: u32 = 0x01c27d;
// `SpriteDraw_SingleSmall` has published its X coordinate, extended-OAM
// size/X bit, and visible Y coordinate when execution reaches this CLC. The
// character/flags stores and optional shadow remain pending. Keep the pinned
// PC private to the adapter; gameplay receives only the source statement.
const SPRITE_SINGLE_SMALL_AFTER_POSITION_PC: u32 = 0x06dd1a;
// Probe has returned from Sprite_PrepOamCoordOrDoubleRet and is about to test
// its prepared coordinates for off-screen removal. Movement, collision, and
// proximity work are complete; keep the instruction address in this adapter.
const SPRITE_PROBE_AFTER_OAM_COORDINATES_PC: u32 = 0x05c21d;
// WallMaster_SendPlayerToLastEntrance calls Sprite_ResetAll from $0B:FFAF;
// the return address proves this is the nested Wallmaster call rather than
// one of the other users of the shared reset routine. At $09:C47B the fixed
// Sprite_ResetAll_noDisable stores are complete and the first large clear
// store has not executed yet.
const WALLMASTER_RESET_AFTER_FIXED_PREFIX_PC: u32 = 0x09c47b;
const WALLMASTER_AFTER_SPRITE_RESET_PC: u32 = 0x0bffb3;
// `ThrowableScenery_ScatterIntoDebris`'s small-debris branch publishes the
// current slot's terminal state clear here, before calling the OAM-coordinate
// helper and optionally publishing one garnish. This is a resumable C
// statement boundary; neither the PC nor the current-slot register leaves the
// semantic adapter.
const THROWABLE_SCENERY_STATE_CLEAR_PC: u32 = 0x06aca4;
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
// `Sprite_62_MasterSword` subtype 2 enters `Sprite_MoveXY` after its draw and
// nonzero-A branch. This private call-site PC distinguishes the light-beam
// caller from the many other users of the shared movement helper.
const MASTER_SWORD_LIGHT_BEAM_MOVEMENT_CALL_PC: u32 = 0x05_8af3;
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
// The shared animated-sprite decoder has many callers. Its pinned entry PC
// plus King Zora's return address proves that the purchased-flippers spawn
// and every field publication before the `$11` decode have completed.
const DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC: u32 = 0x00_d4ed;
// Module0B/$24 enters the shared decoder a second time only after
// `LoadOverworldFromSpecialOverworld` has restored the special-exit state.
// JSL leaves the address of its final operand on the native stack, so the
// pinned caller proof is $02:AECA for the call whose opcode is $02:AEC7.
const SPECIAL_EXIT_MOSAIC_SECOND_DECODE_RETURN_ADDRESS: u32 = 0x02_aeca;
const ZORA_FLIPPERS_GRAPHICS_RETURN_ADDRESS: u32 = 0x1d_e1e9;
// `SpritePrep_BonkItem`'s room-$107 branch calls the same decoder after its
// state/property initialization and floor assignment have returned. The
// pinned return address distinguishes that semantic caller from every other
// animated-sheet user without exporting either address to gameplay.
const BONK_ITEM_GRAPHICS_RETURN_ADDRESS: u32 = 0x06_8d17;
// `AncillaAdd_TossedPondItem` enters the same decoder after Wish Pond case 2
// has removed the selected item and successfully spawned its ancilla. Its
// private return address identifies that exact source caller.
const WISH_POND_TOSSED_ITEM_GRAPHICS_RETURN_ADDRESS: u32 = 0x09_8a64;
// Pinned Link_HandleVelocity has a second, earlier source-equivalent boundary.
// From `$87:e275 LDA link_player_handler_state` through the following
// `$87:e27e BEQ`, the routine only reads the handler/movement flags and selects
// a branch; no Zelda state has changed yet. Snes9x can expose an operand-byte
// PC while `retro_run` returns between CPU cycles (route host 142141 exposes
// `$87:e27d`, the high operand byte of `LDA link_flag_moving`), so cover the
// complete instructions rather than only their opcode boundaries. Keep this
// private adapter range separate from the wider Link_MovePosition range so
// later Link_HandleVelocity branches which mutate gameplay state cannot be
// mistaken for the same semantic checkpoint.
const LINK_VELOCITY_BEFORE_STATE_BRANCH_START_PC: u32 = 0x07e275;
const LINK_VELOCITY_BEFORE_STATE_BRANCH_END_PC: u32 = 0x07e280;
// After Link_HandleVelocity has selected its speed-table index, `$87:e2c8`
// stores only the call-local scratch byte. The next instruction at `$87:e2ca`
// is the first gameplay-state store (`STZ link_actual_vel_y`). An interrupt
// whose saved PC lies in this interval therefore exposes the same semantic
// boundary: Module0F's outer speed/ripple prefix has run, but Link's velocity
// and coordinates have not changed yet (route host 65295).
const LINK_VELOCITY_AFTER_SPEED_SELECTION_START_PC: u32 = 0x07e2c8;
const LINK_VELOCITY_BEFORE_FIRST_STATE_STORE_END_PC: u32 = 0x07e2cc;
// Link_HandleVelocity resolves actual velocity in X-then-Y source order.
// The source cursor is 1 for horizontal and 0 for vertical. `$87:e344`
// begins each pass and `$87:e357` is its first persistent store, so the
// interrupted cursor identifies which components remain unpublished.
const LINK_ACTUAL_VELOCITY_PASS_START_PC: u32 = 0x07e344;
const LINK_ACTUAL_VELOCITY_BEFORE_STORE_END_PC: u32 = 0x07e359;
// Pinned Link_MovePosition ($87:e370) copies Link's current coordinates and
// safe-return bytes before its first coordinate integration store at $87:e3af.
// Bank $07 is the executing LoROM mirror observed by the maintained core.
const LINK_POSITION_BEFORE_COORDINATES_START_PC: u32 = 0x07e370;
const LINK_POSITION_BEFORE_COORDINATES_END_PC: u32 = 0x07e3af;
// Link_MovePosition's axis loop between `STA $2A,y` (the subpixel store) and
// `ADC $20,x` (the coordinate add): the current axis' subpixel is published,
// its coordinate is not. X names the pass (4 = z, 2 = x, 0 = y).
const LINK_POSITION_AFTER_SUBPIXEL_START_PC: u32 = 0x07e3b2;
const LINK_POSITION_AFTER_SUBPIXEL_END_PC: u32 = 0x07e3ca;
// After `STA $20,x` has published the coordinate low byte, the source still
// owes the high-byte add/store. A return PC at $87:E3CA through $87:E3CE is
// exactly this mixed-coordinate interval (route host 179583).
const LINK_POSITION_AFTER_COORDINATE_LOW_START_PC: u32 = 0x07e3ca;
const LINK_POSITION_AFTER_COORDINATE_LOW_END_PC: u32 = 0x07e3cf;
// After `STA $21,x` publishes the current axis' high coordinate byte, the
// loop still owes its cursor decrements, later axes, and movement tail. X
// identifies the just-completed pass through the loop epilogue. Once the
// final Y pass has decremented X below zero, its PC identifies that completed
// pass even though X no longer does (route host 76632).
const LINK_POSITION_AFTER_COORDINATES_START_PC: u32 = 0x07e3cf;
const LINK_POSITION_AFTER_COORDINATES_END_PC: u32 = 0x07e3d5;
const SPRITE_PREP_RESET_PROPERTIES_START_PC: u32 = 0x0db871;
const SPRITE_PREP_RESET_PROPERTIES_ACCUMULATOR_CLEAR_PC: u32 = 0x0db8da;
const SPRITE_PREP_RESET_PROPERTIES_LONG_STORES_START_PC: u32 = 0x0db8dc;
const SPRITE_PREP_RESET_PROPERTIES_RETURN_PC: u32 = 0x0db8f0;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_RETURN_ADDRESS: u32 = 0x0db81b;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_PC: u32 = 0x0db81c;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS2_PC: u32 = 0x0db829;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_HEALTH_PC: u32 = 0x0db82f;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS4_PC: u32 = 0x0db835;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS5_PC: u32 = 0x0db83b;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_DEFLECTION_PC: u32 = 0x0db841;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_BUMP_DAMAGE_PC: u32 = 0x0db847;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS_PC: u32 = 0x0db84d;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_ROOM_PC: u32 = 0x0db85a;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS3_PC: u32 = 0x0db869;
const SPRITE_PREP_LOAD_PROPERTIES_AFTER_OAM_FLAGS_PC: u32 = 0x0db86e;
const SPRITE_PREP_LOAD_PROPERTIES_RETURN_PC: u32 = 0x0db870;
// `SpritePrep_MiniMoldorm_bounce` publishes 32 history entries as four
// source-ordered byte stores (Y low/high, then X low/high). The live adapter
// translates the private instruction/cursor position into a store count.
const SPRITE_PREP_MINI_MOLDORM_HISTORY_LOOP_START_PC: u32 = 0x1df282;
const SPRITE_PREP_MINI_MOLDORM_HISTORY_Y_HIGH_LOAD_PC: u32 = 0x1df289;
const SPRITE_PREP_MINI_MOLDORM_HISTORY_X_LOW_LOAD_PC: u32 = 0x1df290;
const SPRITE_PREP_MINI_MOLDORM_HISTORY_X_HIGH_LOAD_PC: u32 = 0x1df297;
const SPRITE_PREP_MINI_MOLDORM_HISTORY_INCREMENT_PC: u32 = 0x1df29e;
const SPRITE_PREP_MINI_MOLDORM_HISTORY_LOOP_TEST_PC: u32 = 0x1df29f;
const SPRITE_PREP_MINI_MOLDORM_HISTORY_RETURN_START_PC: u32 = 0x1df2a3;
const SPRITE_PREP_MINI_MOLDORM_HISTORY_END_PC: u32 = 0x1df2a5;
// Fire Debirando's state-8 initializer converts type $64 to $63 before its
// nested second SpritePrep_LoadProperties call. The following reset shares
// the generic helper's PC and immediate return address with the initial load,
// so this source write distinguishes the caller phase.
const SPRITE_PREP_FIRE_DEBIRANDO_TYPE_STORE_PC: u32 = 0x068b43;
const SPRITE_PREP_FIRE_DEBIRANDO_SPAWN_RETURN_ADDRESS: u32 = 0x068b5f;
const SPRITE_SPAWN_DYNAMICALLY_ENTRY_PC: u32 = 0x1df65d;
const SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC: u32 = 0x1df66f;
const SPRITE_SPAWN_DYNAMICALLY_STATE_STORE_PC: u32 = 0x1df674;
const SPRITE_SPAWN_DYNAMICALLY_IDENTITY_STORE_PC: u32 = 0x1df6b8;
const SPRITE_SPAWN_DYNAMICALLY_FLOOR_STORE_PC: u32 = 0x1df6bf;
const SPRITE_SPAWN_DYNAMICALLY_DIRECTION_STORE_PC: u32 = 0x1df6c5;
const SPRITE_SPAWN_DYNAMICALLY_DIE_ACTION_STORE_PC: u32 = 0x1df6cc;
const SPRITE_SPAWN_DYNAMICALLY_SUBTYPE_STORE_PC: u32 = 0x1df6cf;
const VWF_RENDER_SINGLE_START_PC: u32 = 0x0ecab8;
// The shared body entered by VWF_RenderSingle after its per-glyph prefix. A
// host return in this range proves that the current decoder byte's click and
// line-transition statements have committed even though the decoder cursor
// itself has not advanced yet.
const VWF_RENDER_SINGLE_BODY_START_PC: u32 = 0x0ecb5e;
const VWF_RENDER_SINGLE_END_PC: u32 = 0x0ecd1a;
const UNCACHE_SPRITE_START_PC: u32 = 0x1dea00;
const UNCACHE_SPRITE_RESTORE_START_PC: u32 = 0x1deb06;
const UNCACHE_SPRITE_END_PC: u32 = 0x1deb68;
const SPRITE_STATE_BASE: u16 = 0x0dd0;
const SPRITE_Y_LOW_BASE: u16 = 0x0d00;
const SPRITE_Y_HIGH_BASE: u16 = 0x0d20;
const SPRITE_N_WORD_BASE: u16 = 0x0bc0;
const SPRITE_TYPE_BASE: u16 = 0x0e20;
const SPRITE_SUBTYPE_BASE: u16 = 0x0e30;
const SPRITE_FLOOR_BASE: u16 = 0x0f20;
const SPRITE_DIRECTION_BASE: u16 = 0x0de0;
const SPRITE_N_BASE: u16 = 0x0bc0;
const SPRITE_DIE_ACTION_BASE: u16 = 0x0cba;
const DUNGEON_LOAD_TEMP_Y: u16 = 0x0fb5;
const DUNGEON_LOAD_SHARED_X: u16 = 0x0fb6;
const OVERWORLD_SPRITE_SCAN_START_PC: u32 = 0x09c55e;
const OVERWORLD_SPRITE_SCAN_END_PC: u32 = 0x09c881;
const OVERWORLD_LOAD_SINGLE_SPRITE_START_PC: u32 = 0x09c770;
const OVERWORLD_LOAD_SINGLE_SPRITE_END_PC: u32 = 0x09c80b;
// `Overworld_LoadOverlays` calls `Sprite_ReloadAll_Overworld` at $02:af0b;
// the JSL returns to $02:af12. Track that source call from its callee entry
// through the callee's $09:c4aa RTL rather than inferring ownership from the
// module/submodule bytes, which remain unchanged while the call spans hosts.
const OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC: u32 = 0x09c499;
const OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_RETURN_PC: u32 = 0x09c4aa;
const OVERWORLD_LOAD_OVERLAYS_AFTER_SPRITE_RELOAD_PC: u32 = 0x02af12;
// `PreOverworld_LoadProperties` calls `Sprite_ReloadAll_Overworld`; its JSL
// returns here after the reset, presence publication, and proximity scan.
const PRE_OVERWORLD_AFTER_SPRITE_RELOAD_PC: u32 = 0x0284dd;
// `MirrorWarp_LoadSpritesAndColors` calls the same reload at $02:b3ef; the
// JSL returns to $02:b3f3. The module remains Module09/$23 throughout the
// cross-host call, so only this concrete return address proves ownership.
const MIRROR_WARP_AFTER_SPRITE_RELOAD_PC: u32 = 0x02b3f3;
// Death_Func15 has returned from Death_Func31 and published its reset module,
// Link coordinate, and scroll stores when it reaches the source-ordered
// `memset(save_dung_info, 0, ...)` loop. The loop and song upload remain live.
const SAVE_QUIT_RESET_DUNGEON_INFO_CLEAR_ENTRY_PC: u32 = 0x09f63f;
// Graphics decompression clears its shared low-WRAM workspace with three
// descending 16-bit STZ stores at $02:80d0/$02:80d3/$02:80d6. Reaching $02:80dd
// proves the complete $0d00-$0fff range is zero. During file-select loading
// this aliases the live sprite arrays while the graphics caller remains live.
const FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_AFTER_PAGE_D_PC: u32 = 0x0280d3;
const FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_AFTER_PAGE_E_PC: u32 = 0x0280d6;
const FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_AFTER_PAGE_F_PC: u32 = 0x0280d9;
const FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_RETURN_PC: u32 = 0x0280dd;
const SELECTED_GAME_LOAD_MESSAGE_INTERFACE_RETURN_PC: u32 = 0x0ffdc3;
const MODULE05_AFTER_SHOW_TEXT_MESSAGE_PC: u32 = 0x0281f5;
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
    #[serde(default)]
    body_progress: Option<CachedSpriteExecutionBodyProgress>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum RescuedMaidenInitializationTrackerPhase {
    FirstFollowerSheet,
    SecondFollowerSheet,
    /// Both sheets returned. A host boundary here would require a separate
    /// conversion/caller-suffix checkpoint and is rejected fail-closed.
    Converting,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct RescuedMaidenInitializationTracker {
    phase: RescuedMaidenInitializationTrackerPhase,
    completed_bytes: u16,
}

impl RescuedMaidenInitializationTracker {
    const fn first_sheet() -> Self {
        Self {
            phase: RescuedMaidenInitializationTrackerPhase::FirstFollowerSheet,
            completed_bytes: 0,
        }
    }

    fn begin_second_sheet(&mut self) -> Result<(), String> {
        if self.phase != RescuedMaidenInitializationTrackerPhase::FirstFollowerSheet {
            return Err(format!(
                "Snes9x rescued-maiden follower graphics entered the second sheet from {:?}",
                self.phase,
            ));
        }
        self.phase = RescuedMaidenInitializationTrackerPhase::SecondFollowerSheet;
        self.completed_bytes = 0;
        Ok(())
    }

    fn begin_conversion(&mut self) -> Result<(), String> {
        if self.phase != RescuedMaidenInitializationTrackerPhase::SecondFollowerSheet {
            return Err(format!(
                "Snes9x rescued-maiden follower graphics returned from {:?}",
                self.phase,
            ));
        }
        self.phase = RescuedMaidenInitializationTrackerPhase::Converting;
        self.completed_bytes = 0;
        Ok(())
    }

    fn observe_boundary(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if self.phase == RescuedMaidenInitializationTrackerPhase::Converting {
            if let Some(completed_stores) = follower_graphics_conversion_completed_stores(event)? {
                if completed_stores < self.completed_bytes {
                    return Err(format!(
                        "Snes9x follower-graphics conversion cursor moved backward: {} -> {completed_stores}",
                        self.completed_bytes,
                    ));
                }
                self.completed_bytes = completed_stores;
            }
            return Ok(());
        }
        let pc = event
            .pc
            .ok_or("Snes9x rescued-maiden decompressor boundary omitted PC")?
            & 0x00ff_ffff;
        if !(RESCUED_MAIDEN_DECOMPRESS_BODY_START_PC..=RESCUED_MAIDEN_DECOMPRESS_BODY_END_PC)
            .contains(&pc)
        {
            // An accepted NMI may leave the host inside its handler. The
            // acceptance event already captured the interrupted cursor.
            return Ok(());
        }
        let y = event
            .y
            .ok_or("Snes9x rescued-maiden decompressor boundary omitted Y")?;
        let completed_bytes = if pc == RESCUED_MAIDEN_DECOMPRESS_STORE_POSTFETCH_PC {
            y.checked_add(1)
                .ok_or("Snes9x rescued-maiden decompressor cursor overflowed")?
        } else {
            y
        };
        if completed_bytes > RESCUED_MAIDEN_FOLLOWER_SHEET_BYTES {
            return Err(format!(
                "Snes9x rescued-maiden decompressor exceeded one sheet: {completed_bytes}",
            ));
        }
        if completed_bytes < self.completed_bytes {
            return Err(format!(
                "Snes9x rescued-maiden decompressor cursor moved backward: {} -> {completed_bytes}",
                self.completed_bytes,
            ));
        }
        self.completed_bytes = completed_bytes;
        Ok(())
    }

    fn host_return_receipt(self) -> Result<RescuedMaidenInitializationProgressReceipt, String> {
        let stage = match self.phase {
            RescuedMaidenInitializationTrackerPhase::FirstFollowerSheet => {
                RescuedMaidenInitializationStage::FirstFollowerSheet {
                    completed_bytes: self.completed_bytes,
                }
            }
            RescuedMaidenInitializationTrackerPhase::SecondFollowerSheet => {
                RescuedMaidenInitializationStage::SecondFollowerSheet {
                    completed_bytes: self.completed_bytes,
                }
            }
            RescuedMaidenInitializationTrackerPhase::Converting => {
                RescuedMaidenInitializationStage::Conversion {
                    completed_stores: self.completed_bytes,
                }
            }
        };
        Ok(RescuedMaidenInitializationProgressReceipt {
            stage,
            boundary: OriginalTimingBoundary::HostReturn,
        })
    }
}

/// Convert the pinned converter's private instruction/X position into the
/// exact prefix of its 512 source-order 16-bit publications. X advances by
/// two bytes per row and by a further sixteen bytes between 32-byte tiles;
/// each row publishes its low/high word before its upper-plane word.
fn follower_graphics_conversion_completed_stores(
    event: &RawTraceEvent,
) -> Result<Option<u16>, String> {
    let pc = event
        .pc
        .ok_or("Snes9x follower-graphics conversion boundary omitted PC")?
        & 0x00ff_ffff;
    if !(FOLLOWER_GRAPHICS_CONVERSION_START_PC..=FOLLOWER_GRAPHICS_CONVERSION_END_PC).contains(&pc)
    {
        return Ok(None);
    }
    if pc >= 0x00_d618 {
        return Ok(Some(FOLLOWER_GRAPHICS_CONVERSION_STORES));
    }
    let x = event
        .x
        .ok_or("Snes9x follower-graphics conversion boundary omitted X")?;
    let delta = x
        .checked_sub(FOLLOWER_GRAPHICS_CONVERSION_DESTINATION_X)
        .ok_or_else(|| {
            format!("Snes9x follower-graphics conversion X preceded its destination: ${x:04x}")
        })?;
    if delta > 0x0400 {
        return Err(format!(
            "Snes9x follower-graphics conversion X exceeded its destination: ${x:04x}",
        ));
    }
    let tile = delta / 0x20;
    let within_tile = delta % 0x20;
    if within_tile > 0x10 || within_tile & 1 != 0 {
        return Err(format!(
            "Snes9x follower-graphics conversion used invalid row cursor X=${x:04x}",
        ));
    }
    let row = tile * 8 + within_tile / 2;
    let before_current_instruction = row * 2;
    let committed_in_row = match pc {
        0x00_d5db..=0x00_d5dd | 0x00_d5f2..=0x00_d5f4 => 0,
        0x00_d5e1..=0x00_d5ea | 0x00_d5f8..=0x00_d601 => 1,
        0x00_d5ee..=0x00_d5f1 | 0x00_d605..=0x00_d60b => 2,
        _ => 0,
    };
    let completed = before_current_instruction
        .checked_add(committed_in_row)
        .ok_or("Snes9x follower-graphics conversion cursor overflowed")?;
    if completed > FOLLOWER_GRAPHICS_CONVERSION_STORES {
        return Err(format!(
            "Snes9x follower-graphics conversion exceeded {} stores: {completed}",
            FOLLOWER_GRAPHICS_CONVERSION_STORES,
        ));
    }
    Ok(Some(completed))
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
    dispatch_trampoline_return: Option<u8>,
    #[serde(default)]
    timers_and_oam_slot: Option<u8>,
    #[serde(default)]
    timers_and_oam_dispatch_state: Option<u8>,
    #[serde(default)]
    initialize_active_main_calls: u8,
    #[serde(default)]
    guard_prep_parry_hitbox: Option<(u8, u8)>,
    guard_prep_patrol_delay: Option<(u8, u8)>,
    guard_prep_tile_collision_return: Option<(u8, u8)>,
    #[serde(default)]
    guard_animation_checkpoint: Option<(u8, zelda3::GuardAnimationCheckpoint)>,
    hog_spear_body_graphics_pending: Option<u8>,
    absorbable_body_active: bool,
    absorbable_horizontal_lookup: Option<u8>,
    absorbable_vertical_lookup: Option<u8>,
    absorbable_vertical_attribute_loaded: Option<u8>,
    swamola_segment: Option<u8>,
    vitreous_minions_seen: bool,
    vitreous_player_damage_pending: Option<u8>,
    vitreous_ai_pending: Option<u8>,
    vitreous_damage_pending: Option<u8>,
    swamola_head_prepared: bool,
    swamola_head_draw_completed: Option<u8>,
    swamola_head_draw: Option<u8>,
    swamola_segment_draw: Option<(u8, u8)>,
    pengator_slide_pending: Option<u8>,
    antifairy_bounce_pending: Option<u8>,
    kholdstare_subtype_decremented: bool,
    kholdstare_damage_pending: Option<u8>,
    initialize_prep_pending: Option<u8>,
    #[serde(default)]
    guard_animation_pose_slot: Option<u8>,
    #[serde(default)]
    guard_prep_weapon_flags_pending_slot: Option<u8>,
    #[serde(default)]
    mini_moldorm_history: Option<(u8, u8)>,
    #[serde(default)]
    initialize_reset_properties: Option<(u8, SpriteInitializeResetPropertiesPhase, u8)>,
    #[serde(default)]
    initialize_load_properties: Option<(u8, SpriteInitializeResetPropertiesPhase, u8)>,
    #[serde(default)]
    fire_debirando_property_reload: bool,
    #[serde(default)]
    fire_debirando_before_spawn_slot: Option<u8>,
    #[serde(default)]
    fire_debirando_spawn: Option<(u8, u8, SpriteDynamicSpawnProgress)>,
    #[serde(default)]
    antfairy_subtype2_increment_slot: Option<u8>,
    #[serde(default)]
    lanmola_subtype2_increment_slot: Option<u8>,
    #[serde(default)]
    helmasaur_hard_hat_beetle_subtype2_increment_slot: Option<u8>,
    #[serde(default)]
    timer_decrements_slot: Option<u8>,
    #[serde(default)]
    primary_timer_decrements_slot: Option<u8>,
    #[serde(default)]
    hit_timer_slot: Option<u8>,
    #[serde(default)]
    main_and_aux1_timer_decrements_slot: Option<u8>,
    main_timer_decrement_slot: Option<u8>,
    zero_hit_timer_clear_slot: Option<u8>,
    #[serde(default)]
    bari_before_random_slot: Option<u8>,
    #[serde(default)]
    throwable_scenery_state_clear_slot: Option<u8>,
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
    master_sword_light_beam_movement: Option<(u8, u8)>,
    #[serde(default)]
    master_sword_light_beam_spawn: Option<(u8, u8, SpriteDynamicSpawnProgress)>,
    #[serde(default)]
    cucco_animation_slot: Option<(u8, u8)>,
    #[serde(default)]
    big_key_drop_graphics_slot: Option<u8>,
    #[serde(default)]
    king_zora_flippers_graphics_slot: Option<u8>,
    #[serde(default)]
    bonk_item_graphics_slot: Option<u8>,
    #[serde(default)]
    wish_pond_tossed_item_graphics_slot: Option<u8>,
    #[serde(default)]
    single_small_draw_position_slot: Option<u8>,
    #[serde(default)]
    probe_after_oam_coordinates_slot: Option<u8>,
    #[serde(default)]
    wallmaster_reset_prefix_slot: Option<u8>,
    wallmaster_reset_cleared_bytes: Option<u16>,
    #[serde(default)]
    zazak_graphics_slot: Option<u8>,
    #[serde(default)]
    follower_graphics: Option<(
        SpriteFollowerGraphicsCaller,
        RescuedMaidenInitializationTracker,
    )>,
}

impl SpriteMainExecutionTracker {
    fn observe_dispatch_trampoline_return(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        // SpriteActive4_Trampoline returned from its long dispatcher. Its
        // remaining RTS reaches Sprite_Main's slot return without any store.
        if event.pc.map(|pc| pc & 0xff_ffff) == Some(0x06_bff8)
            && event
                .return_address
                .is_some_and(|address| address & 0xffff == 0x83a6)
        {
            let slot = self
                .current_slot
                .ok_or("sprite trampoline return has no active slot")?;
            if event.x != Some(u16::from(slot)) {
                return Err("sprite trampoline return disagrees with active slot".into());
            }
            self.dispatch_trampoline_return = Some(slot);
        }
        Ok(())
    }

    fn observe_kholdstare_damage_pending(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        // DEC $0E80,X at $1E:9537 identifies Kholdstare's active body.
        if event.event == "wram-write" && event.pc == Some(0x1e_953a) {
            let slot = self
                .current_slot
                .ok_or("Kholdstare decrement lost its slot")?;
            if event.x != Some(u16::from(slot))
                || event.address != Some(0x0e80 + u16::from(slot))
                || self.timers_and_oam_dispatch_state != Some(9)
            {
                return Err("Kholdstare decrement disagrees with its active caller".into());
            }
            self.kholdstare_subtype_decremented = true;
        }
        // CheckIfHitBoxesOverlap has pushed X and loaded its axis cursor.
        // Beneath saved X, $F2D0 proves Sprite_CheckDamageFromLink's JSR.
        // Its hitbox setup is local computation; no damage effect has run.
        if self.kholdstare_subtype_decremented
            && event.pc == Some(0x06_f839)
            && event.return_address.map(|stack| stack >> 8) == Some(0xf2d0)
        {
            let slot = self.current_slot.ok_or("Kholdstare damage lost its slot")?;
            if event.stack1 != Some(slot) || event.x != Some(1) {
                return Err("Kholdstare hitbox checkpoint disagrees with its saved slot".into());
            }
            self.kholdstare_damage_pending = Some(slot);
        }
        Ok(())
    }

    fn observe_antifairy_bounce_pending(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        // Shared bounce entry belongs to Antifairy only under its own JSL.
        if event.pc != Some(0x1d_c778) || event.return_address != Some(0x06_a53e) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Antifairy bounce lost its active slot")?;
        if event.x != Some(u16::from(slot)) || self.timers_and_oam_dispatch_state != Some(9) {
            return Err("Antifairy bounce checkpoint disagrees with its active caller".into());
        }
        self.antifairy_bounce_pending = Some(slot);
        Ok(())
    }

    fn observe_pengator_slide_pending(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        // Pengator_Slide has only tested the sparkle cadence and Z; its first
        // RNG call and every slide-specific persistent store remain pending.
        if !matches!(event.pc, Some(0x1e_a271..=0x1e_a279)) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Pengator slide lost its active slot")?;
        if event.x != Some(u16::from(slot)) || self.timers_and_oam_dispatch_state != Some(9) {
            return Err("Pengator slide checkpoint disagrees with its active caller".into());
        }
        self.pengator_slide_pending = Some(slot);
        Ok(())
    }

    fn observe_vitreous_damage_pending(&mut self, event: &RawTraceEvent) {
        // The shared pair returned from damage-from-Link and entered the
        // damage-to-Link leaf, before that leaf's first persistent effect.
        if self.vitreous_minions_seen
            && event.pc.map(|pc| pc & 0xff_ffff) == Some(0x06_f145)
            && event.return_address.map(|pc| pc & 0xff_ffff) == Some(0x1d_f126)
            && self.current_slot.map(u16::from) == event.x
        {
            self.vitreous_player_damage_pending = self.current_slot;
        }
        // The long jump-table helper popped the low return byte into Y;
        // bank/high bytes remain on the stack, proving Vitreous's AI dispatch.
        if self.vitreous_minions_seen
            && event.pc.map(|pc| pc & 0xff_ffff) == Some(0x00_8788)
            && event.y == Some(0xe4)
            && event
                .return_address
                .is_some_and(|stack| stack & 0xffff == 0x1de4)
            && self.current_slot.map(u16::from) == event.x
        {
            self.vitreous_ai_pending = self.current_slot;
        }
        // The minion cadence call returned and the shared damage wrapper
        // only pushed DB. Both damage directions and Vitreous AI are pending.
        let entry = event.pc.map(|pc| pc & 0xff_ffff) == Some(0x06_f2ab)
            && event.return_address.map(|pc| pc & 0xff_ffff) == Some(0xc2_141d);
        // Sprite_SetupHitBox has only computed local hitbox coordinates.
        // Saved Y precedes the damage caller's JSR return $F2CD on the stack.
        let hitbox = event.pc.map(|pc| pc & 0xff_ffff) == Some(0x06_f82d)
            && event.return_address.map(|stack| stack >> 8) == Some(0xf2cd);
        // Player_SetupActionHitBox computes only local geometry. Before PHX
        // the caller slot is in X; afterward it is the saved stack byte.
        let action_hitbox = event
            .pc
            .is_some_and(|pc| (0x06_f5e0..0x06_f645).contains(&(pc & 0xff_ffff)))
            && event.return_address.is_some_and(|stack| {
                (stack & 0xffff == 0xf2ca && self.current_slot.map(u16::from) == event.x)
                    || (stack >> 8 == 0xf2ca && self.current_slot == Some(stack as u8))
            });
        if self.vitreous_minions_seen
            && (((entry || hitbox) && self.current_slot.map(u16::from) == event.x) || action_hitbox)
        {
            self.vitreous_damage_pending = self.current_slot;
        }
    }

    fn observe_swamola_segment_draw(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        // PrepOamCoord's off-screen path has applied its side effects and
        // is discarding the near return address. Swamola's history is pending.
        if self.swamola_head_prepared
            && event.pc.map(|pc| pc & 0xff_ffff) == Some(0x06_e492)
            && event.return_address.map(|pc| pc & 0xff_ffff) == Some(0xdb_f5dc)
            && self.current_slot.map(u16::from) == event.x
        {
            self.swamola_head_draw_completed = self.current_slot;
        }
        // The Swamola caller has stored head graphics and flags, but the
        // JSL has only entered SpriteDraw_SingleLarge; history is still pending.
        if event.pc.map(|pc| pc & 0xff_ffff) == Some(0x06_dbf0)
            && event.return_address.map(|pc| pc & 0xff_ffff) == Some(0x1d_9f8b)
            && self.current_slot.map(u16::from) == event.x
        {
            self.swamola_head_draw = self.current_slot;
        }
        if event.pc.map(|pc| pc & 0xff_ffff) == Some(0x06_e442)
            && event
                .return_address
                .is_some_and(|address| address & 0xffff == 0xdc12)
            && self.current_slot.map(u16::from) == event.x
        {
            if let Some(segment) = self.swamola_segment.filter(|&segment| segment < 4) {
                self.swamola_segment_draw = Some((
                    self.current_slot.ok_or("Swamola draw lost its slot")?,
                    segment,
                ));
            }
        }
        Ok(())
    }

    fn observe_absorbable_tile_lookup(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        // PHX before the simplified-attribute table: the attribute leaf has
        // returned, while every collision-dependent sprite effect is pending.
        if self.absorbable_body_active
            && event.pc.map(|pc| pc & 0xff_ffff) == Some(0x06_e812)
            && event
                .return_address
                .is_some_and(|address| address & 0xffff == 0xe5f0)
            && self.current_slot.map(u16::from) == event.x
        {
            self.absorbable_vertical_attribute_loaded = self.current_slot;
        }
        // Sprite_CheckTileProperty's JSR reaches the attribute leaf before
        // any tile-dependent effects. X is still the owning sprite here.
        let attribute_entry = event.pc.map(|pc| pc & 0xff_ffff) == Some(0x06_e883)
            && event
                .return_address
                .is_some_and(|address| address & 0xffff == 0xe7a0)
            && self.current_slot.map(u16::from) == event.x;
        let attribute_lookup = event
            .pc
            .is_some_and(|pc| (0x00_882e..0x00_8888).contains(&(pc & 0xff_ffff)))
            && event.return_address.map(|pc| pc & 0xff_ffff) == Some(0x06_e8cd);
        if self.absorbable_body_active
            && (attribute_entry
                || attribute_lookup
                || (event
                    .pc
                    .is_some_and(|pc| (0x06_e775..=0x06_e782).contains(&(pc & 0xff_ffff)))
                    && event
                        .return_address
                        .is_some_and(|address| address & 0xffff == 0xe5f0)
                    && self.current_slot.map(u16::from) == event.x))
            && event.y.is_some_and(|y| matches!(y & 7, 0 | 2))
        {
            self.absorbable_vertical_lookup = self.current_slot;
        }
        if self.absorbable_body_active
            && (attribute_entry || attribute_lookup)
            && event.y.is_some_and(|y| matches!(y & 7, 4 | 6))
        {
            let slot = self
                .current_slot
                .ok_or("absorbable lookup lost its source slot")?;
            self.absorbable_horizontal_lookup = Some(slot);
        }
        Ok(())
    }
    fn observe_guard_prep_patrol_delay(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if !matches!(
            event.pc.map(|pc| pc & 0xff_ffff),
            Some(0x05_c412 | 0x05_c415)
        ) || self.timers_and_oam_dispatch_state != Some(8)
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("guard patrol checkpoint has no current slot")?;
        if event.x != Some(u16::from(slot)) || !(1..=2).contains(&self.initialize_active_main_calls)
        {
            return Err("guard patrol checkpoint lacks initializer call authority".into());
        }
        self.initialize_prep_pending = None;
        self.initialize_reset_properties = None;
        self.initialize_load_properties = None;
        self.guard_prep_patrol_delay = Some((slot, self.initialize_active_main_calls));
        Ok(())
    }

    fn observe_guard_prep_tile_collision_return(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        if !matches!(
            event.pc.map(|pc| pc & 0xff_ffff),
            Some(0x06_e49d | 0x06_e4a0)
        ) || event.return_address.map(|pc| pc & 0xff_ffff) != Some(0x05_b890)
            || self.timers_and_oam_dispatch_state != Some(8)
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("guard tile-collision checkpoint has no current slot")?;
        if event.x != Some(u16::from(slot)) || !(1..=2).contains(&self.initialize_active_main_calls)
        {
            return Err("guard tile-collision checkpoint lacks initializer call authority".into());
        }
        self.initialize_prep_pending = None;
        self.initialize_reset_properties = None;
        self.initialize_load_properties = None;
        self.guard_prep_tile_collision_return = Some((slot, self.initialize_active_main_calls));
        Ok(())
    }

    fn observe_guard_prep_parry_hitbox(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if !event.pc.is_some_and(|pc| {
            // LDA $44 and its following CMP #$80 have the same completed
            // hitbox prefix. Neither instruction publishes gameplay state.
            (GUARD_PARRY_HITBOX_COMPARE_PC - 2..GUARD_PARRY_HITBOX_COMPARE_PC + 2)
                .contains(&(pc & 0x00ff_ffff))
        }) || self.timers_and_oam_dispatch_state != Some(8)
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("guard parry checkpoint has no current slot")?;
        if event.x != Some(u16::from(slot)) || !(1..=2).contains(&self.initialize_active_main_calls)
        {
            return Err(
                "guard parry checkpoint lacks its initializer active-call authority".into(),
            );
        }
        self.initialize_prep_pending = None;
        self.initialize_reset_properties = None;
        self.initialize_load_properties = None;
        self.guard_prep_weapon_flags_pending_slot = None;
        self.guard_prep_parry_hitbox = Some((slot, self.initialize_active_main_calls));
        Ok(())
    }

    fn observe_mini_moldorm_history(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if self.timers_and_oam_dispatch_state != Some(8) {
            return Ok(());
        }
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return Ok(());
        };
        if !(SPRITE_PREP_MINI_MOLDORM_HISTORY_LOOP_START_PC
            ..SPRITE_PREP_MINI_MOLDORM_HISTORY_END_PC)
            .contains(&pc)
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x entered Mini Moldorm history initialization before a sprite slot")?;
        if event.y != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x Mini Moldorm history initializer disagreed on slot: tracker={slot}, y={:?}",
                event.y,
            ));
        }
        let base = u16::from(slot) * 32;
        let completed_stores = if pc >= SPRITE_PREP_MINI_MOLDORM_HISTORY_RETURN_START_PC {
            128
        } else {
            let cursor = event
                .x
                .ok_or("Snes9x Mini Moldorm history initializer omitted cursor X")?;
            let entry = cursor.checked_sub(base).ok_or_else(|| {
                format!("Snes9x Mini Moldorm history cursor preceded slot {slot}: x=${cursor:04x}")
            })?;
            let completed = if pc >= SPRITE_PREP_MINI_MOLDORM_HISTORY_LOOP_TEST_PC {
                u32::from(entry) * 4
            } else {
                if entry >= 32 {
                    return Err(format!(
                        "Snes9x Mini Moldorm history cursor exceeded slot {slot}: x=${cursor:04x}"
                    ));
                }
                let component = if pc < SPRITE_PREP_MINI_MOLDORM_HISTORY_Y_HIGH_LOAD_PC {
                    0
                } else if pc < SPRITE_PREP_MINI_MOLDORM_HISTORY_X_LOW_LOAD_PC {
                    1
                } else if pc < SPRITE_PREP_MINI_MOLDORM_HISTORY_X_HIGH_LOAD_PC {
                    2
                } else if pc < SPRITE_PREP_MINI_MOLDORM_HISTORY_INCREMENT_PC {
                    3
                } else {
                    4
                };
                u32::from(entry) * 4 + component
            };
            u8::try_from(completed).map_err(|_| {
                format!("Snes9x Mini Moldorm history progress exceeded 128 stores: {completed}")
            })?
        };
        if completed_stores > 128 {
            return Err(format!(
                "Snes9x Mini Moldorm history progress exceeded 128 stores: {completed_stores}"
            ));
        }
        self.initialize_prep_pending = None;
        self.initialize_reset_properties = None;
        self.initialize_load_properties = None;
        self.mini_moldorm_history = Some((slot, completed_stores));
        Ok(())
    }

    fn observe_guard_prep_weapon_flags_pending(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        if event.pc.map(|pc| pc & 0x00ff_ffff) != Some(GUARD_ANIMATE_WEAPON_FLAGS_STORE_PC) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x reached guard weapon draw before a Sprite_Main slot")?;
        if self.timers_and_oam_dispatch_state != Some(8)
            || self.initialize_active_main_calls != 1
            || event.x != Some(0)
            || event.sub != Some(0)
        {
            return Err(format!(
                "Snes9x guard weapon checkpoint lacked its state-8 first-active-call authority: slot={slot} dispatch={:?} active_calls={} x={:?} sub={:?}",
                self.timers_and_oam_dispatch_state,
                self.initialize_active_main_calls,
                event.x,
                event.sub,
            ));
        }
        self.guard_prep_weapon_flags_pending_slot = Some(slot);
        Ok(())
    }

    fn observe_hog_spear_body_graphics_pending(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        // JSR $05:CC38 enters the shared animation helper. Its raw two-byte
        // return is CC3A; the third stack byte belongs to the enclosing caller.
        if !event
            .pc
            .is_some_and(|pc| (0x05_c457..=0x05_c46c).contains(&(pc & 0xff_ffff)))
            || event.return_address.map(|pc| pc & 0xffff) != Some(0xcc3a)
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Hog Spear body endpoint has no active slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err("Hog Spear body endpoint disagrees with its active slot".into());
        }
        self.hog_spear_body_graphics_pending = Some(slot);
        Ok(())
    }

    fn observe_guard_animation_checkpoint(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if event.pc == Some(0x05_c243) && self.timers_and_oam_dispatch_state == Some(9) {
            let slot = self
                .current_slot
                .ok_or("guard draw return has no current slot")?;
            if event.x != Some(u16::from(slot)) {
                return Err("guard draw return disagreed with its active caller".into());
            }
            self.guard_animation_checkpoint =
                Some((slot, zelda3::GuardAnimationCheckpoint::DrawReturned));
            return Ok(());
        }
        if event.event == "wram-write"
            && event.pc == Some(0x05_c240)
            && self.timers_and_oam_dispatch_state == Some(9)
        {
            let slot = self
                .current_slot
                .ok_or("guard temporary pose has no current slot")?;
            if event.address != Some(SPRITE_GRAPHICS_BASE + u16::from(slot))
                || event.x != Some(u16::from(slot))
            {
                return Err("guard temporary pose store disagreed with its active caller".into());
            }
            self.guard_animation_pose_slot = Some(slot);
            return Ok(());
        }
        if !matches!(
            event.pc.map(|pc| pc & 0x00ff_ffff),
            Some(
                0x05_cbaa
                | 0x05_cb86..=0x05_cb8c
                | 0x05_c711
                | 0x05_c713
                | 0x05_c717
                | 0x05_c719
                | 0x05_c71c
                | 0x05_c721..=0x05_c729
                | 0x05_ca29
                | 0x05_ca43..=0x05_ca4b
                | 0x05_ca6b
                | 0x05_ca71
                | 0x05_ca74
                | 0x05_ca77..=0x05_ca9f,
            )
        ) || self.timers_and_oam_dispatch_state != Some(9)
            || self.guard_animation_pose_slot != self.current_slot
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("guard weapon checkpoint has no active slot")?;
        if matches!(event.pc, Some(0x05_c721..=0x05_c729)) {
            let expected_y = if event.pc.unwrap() <= 0x05_c724 { 3 } else { 0 };
            if event.y != Some(expected_y) {
                return Err("guard head extended checkpoint has an invalid OAM cursor".into());
            }
            self.guard_animation_checkpoint =
                Some((slot, zelda3::GuardAnimationCheckpoint::HeadExtendedPending));
            return Ok(());
        }
        if matches!(
            event.pc,
            Some(0x05_c711 | 0x05_c713 | 0x05_c717 | 0x05_c719 | 0x05_c71c)
        ) {
            let expected_y = if matches!(event.pc, Some(0x05_c711 | 0x05_c713)) {
                1
            } else {
                2
            };
            if event.y != Some(expected_y) {
                return Err("guard head flags checkpoint has an invalid OAM cursor".into());
            }
            self.guard_animation_checkpoint = Some((
                slot,
                if matches!(event.pc, Some(0x05_c711 | 0x05_c713 | 0x05_c717)) {
                    zelda3::GuardAnimationCheckpoint::HeadCharacterPending
                } else {
                    zelda3::GuardAnimationCheckpoint::HeadFlagsPending
                },
            ));
            return Ok(());
        }
        let x = event
            .x
            .ok_or("guard weapon checkpoint omitted its table cursor")?;
        use zelda3::GuardAnimationCheckpoint as Stage;
        let y = event.y.ok_or("guard draw omitted its OAM cursor")?;
        let checkpoint = match event.pc {
            Some(0x05_ca29) if x < 56 && y & 3 == 0 => Stage::BodyBeforeEntry {
                entry: (x & 3) as u8,
            },
            // The body table index has doubled, but the first coordinate
            // store has not run. This is the same native publication prefix.
            Some(0x05_ca43..=0x05_ca4b) if x < 112 && x & 1 == 0 && y & 3 == 0 => {
                Stage::BodyBeforeEntry {
                    entry: ((x >> 1) & 3) as u8,
                }
            }
            Some(0x05_ca6b) if x < 112 && x & 1 == 0 && y & 3 == 1 => Stage::BodyCoordinates {
                entry: ((x >> 1) & 3) as u8,
            },
            Some(0x05_ca71 | 0x05_ca74) if x < 56 && y & 3 == 1 => Stage::BodyCoordinates {
                entry: (x & 3) as u8,
            },
            Some(0x05_ca77..=0x05_ca9f) if x < 56 && matches!(y & 3, 2 | 3) => {
                Stage::BodyFlagsPending {
                    entry: (x & 3) as u8,
                }
            }
            Some(0x05_cb86..=0x05_cb8c) if x < 56 && x & 1 == 0 && y & 3 == 0 => {
                Stage::WeaponBeforeCoordinates {
                    entry: ((x >> 1) & 1) as u8,
                }
            }
            Some(0x05_cbaa) if x < 56 && x & 1 == 0 && y & 3 == 1 => Stage::WeaponCoordinates {
                entry: ((x >> 1) & 1) as u8,
            },
            _ => {
                return Err(format!(
                    "guard draw checkpoint has invalid cursors: pc={:?} x={x} y={y}",
                    event.pc
                ))
            }
        };
        self.guard_animation_checkpoint = Some((slot, checkpoint));
        Ok(())
    }

    fn observe_bari_before_random(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if !event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
            (SPRITE_BARI_BEFORE_RANDOM_START_PC..SPRITE_BARI_BEFORE_RANDOM_END_PC).contains(&pc)
        }) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x reached the Bari pre-RNG boundary before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x Bari pre-RNG boundary disagreed on slot: tracker={slot}, x={:?}",
                event.x,
            ));
        }
        self.bari_before_random_slot = Some(slot);
        Ok(())
    }

    fn observe_timer_decrements(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if !event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
            (SPRITE_TIMER_DECREMENTS_COMPLETE_START_PC..SPRITE_TIMER_DECREMENTS_COMPLETE_END_PC)
                .contains(&pc)
        }) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x completed sprite timer decrements before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x sprite timer decrement boundary disagreed on slot: tracker={slot}, x={:?}",
                event.x,
            ));
        }
        self.timer_decrements_slot = Some(slot);
        Ok(())
    }

    fn observe_primary_timer_decrements(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if !event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
            (SPRITE_PRIMARY_TIMER_DECREMENTS_COMPLETE_START_PC
                ..SPRITE_PRIMARY_TIMER_DECREMENTS_COMPLETE_END_PC)
                .contains(&pc)
                || (SPRITE_PRIMARY_TIMER_DECREMENTS_ZERO_HIT_STORE_START_PC
                    ..SPRITE_PRIMARY_TIMER_DECREMENTS_ZERO_HIT_STORE_END_PC)
                    .contains(&pc)
        }) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x completed primary sprite timer decrements before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x primary sprite timer decrement boundary disagreed on slot: tracker={slot}, x={:?}",
                event.x,
            ));
        }
        self.primary_timer_decrements_slot = Some(slot);
        Ok(())
    }

    /// The hit timer and its priority update are complete; aux4 has not run.
    fn observe_hit_timer(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        // LDA aux4 / BEQ / the following DEC opcode are all before the
        // countdown store. Operand fetches from the load are safe as well.
        if !event
            .pc
            .map(|pc| pc & 0xffffff)
            .is_some_and(|pc| (0x06_849c..=0x06_84a1).contains(&pc))
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("hit timer returned before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err("hit timer return disagreed on sprite slot".to_string());
        }
        self.hit_timer_slot = Some(slot);
        Ok(())
    }

    fn observe_main_and_aux1_timer_decrements(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        if !event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
            (SPRITE_MAIN_AND_AUX1_TIMER_DECREMENTS_COMPLETE_START_PC
                ..SPRITE_MAIN_AND_AUX1_TIMER_DECREMENTS_COMPLETE_END_PC)
                .contains(&pc)
        }) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x completed main/aux1 sprite timer decrements before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x main/aux1 sprite timer decrement boundary disagreed on slot: tracker={slot}, x={:?}",
                event.x,
            ));
        }
        self.main_and_aux1_timer_decrements_slot = Some(slot);
        Ok(())
    }

    fn observe_main_timer_decrement(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if !event
            .pc
            .map(|pc| pc & 0x00ff_ffff)
            .is_some_and(|pc| (0x06_8429..0x06_8431).contains(&pc))
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x completed main sprite timer decrements before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x main sprite timer decrement boundary disagreed on slot: tracker={slot}, x={:?}",
                event.x,
            ));
        }
        self.main_timer_decrement_slot = Some(slot);
        Ok(())
    }

    fn observe_zero_hit_timer_clear(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if !event
            .pc
            .map(|pc| pc & 0x00ff_ffff)
            .is_some_and(|pc| (0x06_8499..0x06_849c).contains(&pc))
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x completed main sprite timer decrements before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x main sprite timer decrement boundary disagreed on slot: tracker={slot}, x={:?}",
                event.x,
            ));
        }
        self.zero_hit_timer_clear_slot = Some(slot);
        Ok(())
    }

    fn observe_timers_and_oam_return(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if event.pc.map(|pc| pc & 0x00ff_ffff) != Some(SPRITE_TIMERS_AND_OAM_RETURN_PC) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x returned from Sprite_TimersAndOam before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x Sprite_TimersAndOam return disagreed on slot: tracker={slot}, x={:?}",
                event.x,
            ));
        }
        self.timers_and_oam_slot = Some(slot);
        self.timers_and_oam_dispatch_state = event.stack1;
        Ok(())
    }

    fn observe_antfairy_subtype2_increment(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if event.pc.map(|pc| pc & 0x00ff_ffff) != Some(ANTFAIRY_SUBTYPE2_INCREMENT_PC) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x published Antfairy subtype2 before entering a sprite slot")?;
        if event.x != Some(u16::from(slot))
            || event.address != Some(SPRITE_SUBTYPE2_BASE + u16::from(slot))
        {
            return Err(format!(
                "Snes9x Antfairy subtype2 publication disagreed on slot {slot}: x={:?}, address={:?}",
                event.x, event.address,
            ));
        }
        if self
            .antfairy_subtype2_increment_slot
            .replace(slot)
            .is_some()
        {
            return Err(
                "Snes9x published the Antfairy subtype2 increment twice in one slot".into(),
            );
        }
        Ok(())
    }

    fn observe_lanmola_subtype2_increment(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if event.pc.map(|pc| pc & 0x00ff_ffff) != Some(LANMOLA_SUBTYPE2_INCREMENT_PC) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x published Lanmola subtype2 before entering a sprite slot")?;
        if event.x != Some(u16::from(slot))
            || event.address != Some(SPRITE_SUBTYPE2_BASE + u16::from(slot))
        {
            return Err(format!(
                "Snes9x Lanmola subtype2 publication disagreed on slot {slot}: x={:?}, address={:?}",
                event.x, event.address,
            ));
        }
        if self.lanmola_subtype2_increment_slot.replace(slot).is_some() {
            return Err("Snes9x published the Lanmola subtype2 increment twice in one slot".into());
        }
        Ok(())
    }

    fn observe_helmasaur_hard_hat_beetle_subtype2_increment(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        if event.pc.map(|pc| pc & 0x00ff_ffff)
            != Some(HELMASAUR_HARD_HAT_BEETLE_SUBTYPE2_INCREMENT_PC)
        {
            return Ok(());
        }
        let slot = self.current_slot.ok_or(
            "Snes9x published the Helmasaur/Hardhat subtype2 increment before entering a sprite slot",
        )?;
        if event.x != Some(u16::from(slot))
            || event.address != Some(SPRITE_SUBTYPE2_BASE + u16::from(slot))
        {
            return Err(format!(
                "Snes9x Helmasaur/Hardhat subtype2 publication disagreed on slot {slot}: x={:?}, address={:?}",
                event.x, event.address,
            ));
        }
        if self
            .helmasaur_hard_hat_beetle_subtype2_increment_slot
            .replace(slot)
            .is_some()
        {
            return Err(
                "Snes9x published the Helmasaur/Hardhat subtype2 increment twice in one slot"
                    .into(),
            );
        }
        Ok(())
    }

    fn observe_initialize_reset_properties(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return Ok(());
        };
        let Some(completed_stores) = sprite_prep_reset_properties_completed_stores(pc) else {
            return Ok(());
        };
        if event.return_address.map(|pc| pc & 0x00ff_ffff)
            != Some(SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_RETURN_ADDRESS)
            || self.timers_and_oam_dispatch_state != Some(8)
        {
            return Ok(());
        }
        if self
            .fire_debirando_spawn
            .is_some_and(|(_, spawned_slot, _)| event.x == Some(u16::from(spawned_slot)))
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x reset state-8 sprite properties before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x state-8 property reset disagreed on slot: tracker={slot}, x={:?}",
                event.x,
            ));
        }
        let phase = if self.fire_debirando_property_reload {
            SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion
        } else {
            SpriteInitializeResetPropertiesPhase::InitialPropertyLoad
        };
        self.initialize_prep_pending = None;
        self.initialize_reset_properties = Some((slot, phase, completed_stores));
        Ok(())
    }

    fn observe_initialize_prep_pending(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        let pc = event.pc.map(|pc| pc & 0xff_ffff);
        // FireBar's private prep entry is still before its first INC store;
        // the initializer has already published properties and state 9.
        let initializer_dispatch = matches!(pc, Some(0x06_8654 | 0x06_8657 | 0x06_91b4))
            || (pc == Some(0x00_8781)
                && event.return_address.map(|pc| pc & 0xff_ffff) == Some(0x06_865a));
        if !initializer_dispatch || self.timers_and_oam_dispatch_state != Some(8) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("sprite initializer dispatch has no active slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err("sprite initializer dispatch disagrees with its active slot".into());
        }
        self.initialize_prep_pending = None;
        self.initialize_reset_properties = None;
        self.initialize_load_properties = None;
        self.initialize_prep_pending = Some(slot);
        Ok(())
    }

    fn observe_initialize_load_properties(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return Ok(());
        };
        let Some(completed_stores) = sprite_prep_load_properties_completed_stores(pc) else {
            return Ok(());
        };
        if self.timers_and_oam_dispatch_state != Some(8) {
            return Ok(());
        }
        if self
            .fire_debirando_spawn
            .is_some_and(|(_, spawned_slot, _)| event.x == Some(u16::from(spawned_slot)))
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x loaded state-8 sprite properties before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x state-8 property load disagreed on slot: tracker={slot}, x={:?}",
                event.x,
            ));
        }
        let phase = if self.fire_debirando_property_reload {
            SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion
        } else {
            SpriteInitializeResetPropertiesPhase::InitialPropertyLoad
        };
        self.initialize_prep_pending = None;
        self.initialize_reset_properties = None;
        self.initialize_load_properties = Some((slot, phase, completed_stores));
        Ok(())
    }

    fn observe_fire_debirando_before_spawn(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return Ok(());
        };
        if !(SPRITE_SPAWN_DYNAMICALLY_ENTRY_PC..=SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC)
            .contains(&pc)
            || event.return_address.map(|pc| pc & 0x00ff_ffff)
                != Some(SPRITE_PREP_FIRE_DEBIRANDO_SPAWN_RETURN_ADDRESS)
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x entered Fire Debirando spawn before a sprite slot")?;
        if !self.fire_debirando_property_reload || event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x Fire Debirando spawn disagreed on slot {slot}: converted={}, x={:?}",
                self.fire_debirando_property_reload, event.x,
            ));
        }
        self.initialize_prep_pending = None;
        self.initialize_reset_properties = None;
        self.initialize_load_properties = None;
        self.fire_debirando_before_spawn_slot = Some(slot);
        Ok(())
    }

    fn observe_fire_debirando_spawn_write(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        let pc = event.pc.ok_or("Snes9x WRAM write omitted PC")? & 0x00ff_ffff;
        let address = event.address.ok_or("Snes9x WRAM write omitted address")?;

        if pc == SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC && self.fire_debirando_property_reload
        {
            let slot = self
                .current_slot
                .ok_or("Snes9x spawned Fire Debirando child before a sprite slot")?;
            let spawned_slot = u8::try_from(
                event
                    .y
                    .ok_or("Snes9x Fire Debirando spawn type write omitted slot Y")?,
            )
            .map_err(|_| "Snes9x Fire Debirando spawned slot exceeded one byte")?;
            if event.x != Some(u16::from(slot))
                || spawned_slot >= 16
                || address != SPRITE_TYPE_BASE + u16::from(spawned_slot)
                || event.value != Some(0x64)
            {
                return Err(format!(
                    "Snes9x Fire Debirando spawn type publication disagreed: parent={slot}, x={:?}, spawned={spawned_slot}, address=${address:04x}, value={:?}",
                    event.x, event.value,
                ));
            }
            self.fire_debirando_before_spawn_slot = None;
            self.fire_debirando_spawn = Some((
                slot,
                spawned_slot,
                SpriteDynamicSpawnProgress::TypePublished,
            ));
            return Ok(());
        }

        observe_dynamic_spawn_progress_write(
            &mut self.fire_debirando_spawn,
            event,
            "Fire Debirando",
        )
    }

    fn observe_fire_debirando_spawn_boundary(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        observe_dynamic_spawn_progress_boundary(&mut self.fire_debirando_spawn, event)
    }

    fn observe_master_sword_light_beam_spawn_write(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        let pc = event.pc.ok_or("Snes9x WRAM write omitted PC")? & 0x00ff_ffff;
        let address = event.address.ok_or("Snes9x WRAM write omitted address")?;
        if pc == SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC
            && self.master_sword_light_beam_movement.is_some()
        {
            let slot = self
                .current_slot
                .ok_or("Snes9x spawned a replacement light beam before a sprite slot")?;
            let spawned_slot = u8::try_from(
                event
                    .y
                    .ok_or("Snes9x replacement light-beam type write omitted slot Y")?,
            )
            .map_err(|_| "Snes9x replacement light-beam slot exceeded one byte")?;
            if event.x != Some(u16::from(slot))
                || spawned_slot >= 16
                || address != SPRITE_TYPE_BASE + u16::from(spawned_slot)
                || event.value != Some(0x62)
            {
                return Err(format!(
                    "Snes9x replacement light-beam type publication disagreed: parent={slot}, x={:?}, spawned={spawned_slot}, address=${address:04x}, value={:?}",
                    event.x, event.value,
                ));
            }
            self.master_sword_light_beam_movement = None;
            self.master_sword_light_beam_spawn = Some((
                slot,
                spawned_slot,
                SpriteDynamicSpawnProgress::TypePublished,
            ));
            return Ok(());
        }
        observe_dynamic_spawn_progress_write(
            &mut self.master_sword_light_beam_spawn,
            event,
            "replacement light beam",
        )
    }

    fn observe_master_sword_light_beam_spawn_boundary(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        observe_dynamic_spawn_progress_boundary(&mut self.master_sword_light_beam_spawn, event)
    }

    fn observe_wallmaster_reset_prefix(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        let pc = event.pc.map(|pc| pc & 0x00ff_ffff);
        if !matches!(
            pc,
            Some(WALLMASTER_RESET_AFTER_FIXED_PREFIX_PC | 0x09_c47f | 0x09_c480)
        ) || event.return_address.map(|pc| pc & 0x00ff_ffff)
            != Some(WALLMASTER_AFTER_SPRITE_RESET_PC)
        {
            return Ok(());
        }
        let cleared_bytes = match (pc, event.x) {
            (Some(WALLMASTER_RESET_AFTER_FIXED_PREFIX_PC), Some(0xfff)) => None,
            (Some(0x09_c47b | 0x09_c480), Some(x)) if x <= 0xfff => Some(0xfff - x),
            (Some(0x09_c47f), Some(x)) if x <= 0xfff => Some(0x1000 - x),
            (Some(0x09_c480), Some(0xffff)) => Some(0x1000),
            _ => return Err("invalid Wallmaster reset clear cursor".to_string()),
        };
        self.wallmaster_reset_cleared_bytes = cleared_bytes;
        let slot = self
            .current_slot
            .ok_or("Snes9x reached the Wallmaster reset prefix before a sprite slot")?;
        self.wallmaster_reset_prefix_slot = Some(slot);
        Ok(())
    }

    fn observe_single_small_draw_position(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if event.pc.map(|pc| pc & 0x00ff_ffff) != Some(SPRITE_SINGLE_SMALL_AFTER_POSITION_PC) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x reached the single-small draw position before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x single-small draw position disagreed on slot: tracker={slot}, x={:?}",
                event.x,
            ));
        }
        self.single_small_draw_position_slot = Some(slot);
        Ok(())
    }

    fn observe_probe_after_oam_coordinates(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if event.pc.map(|pc| pc & 0x00ff_ffff) != Some(SPRITE_PROBE_AFTER_OAM_COORDINATES_PC) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x reached the guard-probe OAM boundary before a sprite slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err(format!(
                "Snes9x guard-probe OAM boundary disagreed on slot: tracker={slot}, x={:?}",
                event.x,
            ));
        }
        self.probe_after_oam_coordinates_slot = Some(slot);
        Ok(())
    }

    fn observe_zazak_graphics(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if event.pc.map(|pc| pc & 0x00ff_ffff) != Some(SPRITE_ZAZAK_GRAPHICS_STORE_PC) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Snes9x published Zazak graphics before a Sprite_Main slot")?;
        if event.x != Some(u16::from(slot))
            || event.address != Some(SPRITE_GRAPHICS_BASE + u16::from(slot))
        {
            return Err(format!(
                "Snes9x Zazak graphics publication disagreed on slot: tracker={slot}, x={:?}, address={:?}",
                event.x, event.address,
            ));
        }
        self.zazak_graphics_slot = Some(slot);
        Ok(())
    }

    fn progress(self) -> SpriteMainProgress {
        if let Some(slot) = self.dispatch_trampoline_return {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::AfterSlot(slot);
        }
        if let Some((caller, tracker)) = self.follower_graphics {
            let slot = self
                .current_slot
                .expect("Zelda follower graphics outlived its active Sprite_Main slot");
            let stage = match tracker.phase {
                RescuedMaidenInitializationTrackerPhase::FirstFollowerSheet => {
                    RescuedMaidenInitializationStage::FirstFollowerSheet {
                        completed_bytes: tracker.completed_bytes,
                    }
                }
                RescuedMaidenInitializationTrackerPhase::SecondFollowerSheet => {
                    RescuedMaidenInitializationStage::SecondFollowerSheet {
                        completed_bytes: tracker.completed_bytes,
                    }
                }
                RescuedMaidenInitializationTrackerPhase::Converting => {
                    RescuedMaidenInitializationStage::Conversion {
                        completed_stores: tracker.completed_bytes,
                    }
                }
            };
            return SpriteMainProgress::FollowerGraphics {
                slot,
                caller,
                stage,
            };
        }
        if let Some(slot) = self.wallmaster_reset_prefix_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Wallmaster reset prefix outlived its active sprite slot",
            );
            if let Some(cleared_bytes) = self.wallmaster_reset_cleared_bytes {
                return SpriteMainProgress::WallmasterResetClear {
                    slot,
                    cleared_bytes,
                };
            }
            return SpriteMainProgress::AfterWallmasterResetPrefix(slot);
        }
        if let Some(slot) = self.zazak_graphics_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Zazak graphics publication outlived its active sprite slot",
            );
            return SpriteMainProgress::ZazakAfterGraphics(slot);
        }
        if let Some((slot, completed_stores)) = self.mini_moldorm_history {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Mini Moldorm history progress outlived its active sprite slot",
            );
            return SpriteMainProgress::MiniMoldormHistory {
                slot,
                completed_stores,
            };
        }
        if let Some(slot) = self.bari_before_random_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Bari pre-RNG boundary outlived its active sprite slot",
            );
            return SpriteMainProgress::BariBeforeRandom(slot);
        }
        if let Some(slot) = self.single_small_draw_position_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "single-small draw position publication outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterSingleSmallDrawPosition(slot);
        }
        if let Some(slot) = self.probe_after_oam_coordinates_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "guard-probe OAM boundary outlived its active sprite slot",
            );
            return SpriteMainProgress::ProbeAfterOamCoordinates(slot);
        }
        if let Some((slot, spawned_slot, progress)) = self.fire_debirando_spawn {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Fire Debirando dynamic spawn outlived its active sprite slot",
            );
            return SpriteMainProgress::FireDebirandoSpawn {
                slot,
                spawned_slot,
                progress,
            };
        }
        if let Some(slot) = self.initialize_prep_pending {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::InitializePrepPending(slot);
        }
        if let Some((slot, phase, completed_stores)) = self.initialize_load_properties {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "state-8 property load outlived its active sprite slot",
            );
            return SpriteMainProgress::InitializeLoadProperties {
                slot,
                phase,
                completed_stores,
            };
        }
        if let Some((slot, phase, completed_stores)) = self.initialize_reset_properties {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "state-8 property reset outlived its active sprite slot",
            );
            return SpriteMainProgress::InitializeResetProperties {
                slot,
                phase,
                completed_stores,
            };
        }
        if let Some(slot) = self.fire_debirando_before_spawn_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Fire Debirando spawn boundary outlived its active sprite slot",
            );
            return SpriteMainProgress::FireDebirandoBeforeSpawn(slot);
        }
        if let Some(slot) = self.kholdstare_damage_pending {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::KholdstareDamagePending(slot);
        }
        if let Some(slot) = self.antifairy_bounce_pending {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::AntifairyBouncePending(slot);
        }
        if let Some(slot) = self.antfairy_subtype2_increment_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Antfairy subtype2 publication outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterAntfairySubtype2Increment(slot);
        }
        if let Some(slot) = self.lanmola_subtype2_increment_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Lanmola subtype2 publication outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterLanmolaSubtype2Increment(slot);
        }
        if let Some(slot) = self.helmasaur_hard_hat_beetle_subtype2_increment_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Helmasaur/Hardhat subtype2 publication outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterHelmasaurHardHatBeetleSubtype2Increment(slot);
        }
        if let Some(slot) = self.throwable_scenery_state_clear_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "throwable-scenery state clear outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterThrowableSceneryStateClear(slot);
        }
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
        if let Some(slot) = self.king_zora_flippers_graphics_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "King Zora flippers graphics publication outlived its active sprite slot",
            );
            assert_eq!(
                self.cucco_animation_slot, None,
                "one sprite slot published two incompatible partial checkpoints",
            );
            assert_eq!(
                self.cucco_subtype_increments, None,
                "one sprite slot published two incompatible partial checkpoints",
            );
            return SpriteMainProgress::KingZoraFlippersGraphicsStarted(slot);
        }
        if let Some(slot) = self.bonk_item_graphics_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "bonk-item graphics entry outlived its active sprite slot",
            );
            return SpriteMainProgress::BonkItemGraphicsStarted(slot);
        }
        if let Some(slot) = self.wish_pond_tossed_item_graphics_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Wish Pond tossed-item graphics entry outlived its active sprite slot",
            );
            return SpriteMainProgress::WishPondTossedItemGraphicsStarted(slot);
        }
        if let Some(slot) = self.hog_spear_body_graphics_pending {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::HogSpearBodyGraphicsPending(slot);
        }
        if let Some(slot) = self.vitreous_ai_pending {
            return SpriteMainProgress::VitreousAiPending(slot);
        }
        if let Some(slot) = self.vitreous_player_damage_pending {
            return SpriteMainProgress::VitreousPlayerDamagePending(slot);
        }
        if let Some(slot) = self.vitreous_damage_pending {
            return SpriteMainProgress::VitreousDamagePending(slot);
        }
        if let Some(slot) = self.swamola_head_draw_completed {
            return SpriteMainProgress::SwamolaHeadDrawCompleted(slot);
        }
        if let Some(slot) = self.swamola_head_draw {
            return SpriteMainProgress::SwamolaHeadDraw(slot);
        }
        if let Some((slot, segment)) = self.swamola_segment_draw {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::SwamolaSegmentDraw { slot, segment };
        }
        if let Some(slot) = self.absorbable_horizontal_lookup {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::AbsorbableHorizontalTileLookup(slot);
        }
        if let Some(slot) = self.absorbable_vertical_attribute_loaded {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::AbsorbableVerticalTileAttributeLoaded(slot);
        }
        if let Some(slot) = self.absorbable_vertical_lookup {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::AbsorbableVerticalTileLookup(slot);
        }
        if let Some(slot) = self.pengator_slide_pending {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::PengatorSlidePending(slot);
        }
        if let Some((slot, checkpoint)) = self.guard_animation_checkpoint {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::GuardAnimation { slot, checkpoint };
        }
        if let Some(slot) = self.guard_prep_weapon_flags_pending_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "guard prep weapon checkpoint outlived its active sprite slot",
            );
            return SpriteMainProgress::GuardPrepWeaponFlagsPending(slot);
        }
        if let Some((slot, active_call)) = self.guard_prep_tile_collision_return {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::GuardPrepTileCollisionReturned { slot, active_call };
        }
        if let Some((slot, active_call)) = self.guard_prep_patrol_delay {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::GuardPrepPatrolDelay { slot, active_call };
        }
        if let Some((slot, active_call)) = self.guard_prep_parry_hitbox {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::GuardPrepParryHitbox { slot, active_call };
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
        if let Some((slot, spawned_slot, progress)) = self.master_sword_light_beam_spawn {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "master-sword replacement spawn outlived its active sprite slot",
            );
            return SpriteMainProgress::MasterSwordLightBeamSpawn {
                slot,
                spawned_slot,
                progress,
            };
        }
        if let Some((slot, checkpoint_ordinal)) = self.master_sword_light_beam_movement {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "master-sword light-beam movement outlived its active sprite slot",
            );
            let checkpoint = match checkpoint_ordinal {
                0 => SpriteMoveXYCheckpoint::BeforeMovement,
                1 => SpriteMoveXYCheckpoint::AfterXSubpixel,
                2 => SpriteMoveXYCheckpoint::AfterXLow,
                3 => SpriteMoveXYCheckpoint::AfterXHigh,
                4 => SpriteMoveXYCheckpoint::AfterYSubpixel,
                5 => SpriteMoveXYCheckpoint::AfterYLow,
                6 => SpriteMoveXYCheckpoint::AfterYHigh,
                count => panic!("invalid master-sword movement store count {count}"),
            };
            return SpriteMainProgress::MasterSwordLightBeamMovement { slot, checkpoint };
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
        if let Some(slot) = self.timers_and_oam_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "timer/OAM return outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterTimersAndOam(slot);
        }
        if let Some(slot) = self.timer_decrements_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "timer decrement boundary outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterTimerDecrements(slot);
        }
        if let Some(slot) = self.hit_timer_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "hit timer decrement boundary outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterHitTimer(slot);
        }
        if let Some(slot) = self.zero_hit_timer_clear_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "main timer decrement boundary outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterZeroHitTimerClear(slot);
        }
        if let Some(slot) = self.primary_timer_decrements_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "primary timer decrement boundary outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterPrimaryTimerDecrements(slot);
        }
        if let Some(slot) = self.main_and_aux1_timer_decrements_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "main/aux1 timer decrement boundary outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterMainAndAux1TimerDecrements(slot);
        }
        if let Some(slot) = self.main_timer_decrement_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "main timer decrement boundary outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterMainTimerDecrement(slot);
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
            SpriteMainProgress::AfterTimersAndOam(slot) => {
                MainLoopInterruption::SpriteMainAfterTimersAndOam(slot)
            }
            SpriteMainProgress::AfterTimerDecrements(slot) => {
                MainLoopInterruption::SpriteMainAfterTimerDecrements(slot)
            }
            SpriteMainProgress::AfterPrimaryTimerDecrements(slot) => {
                MainLoopInterruption::SpriteMainAfterPrimaryTimerDecrements(slot)
            }
            SpriteMainProgress::AfterHitTimer(slot) => {
                MainLoopInterruption::SpriteMainAfterHitTimer(slot)
            }
            SpriteMainProgress::AfterMainAndAux1TimerDecrements(slot) => {
                MainLoopInterruption::SpriteMainAfterMainAndAux1TimerDecrements(slot)
            }
            SpriteMainProgress::AfterMainTimerDecrement(slot) => {
                MainLoopInterruption::SpriteMainAfterMainTimerDecrement(slot)
            }
            SpriteMainProgress::AfterZeroHitTimerClear(slot) => {
                MainLoopInterruption::SpriteMainAfterZeroHitTimerClear(slot)
            }
            SpriteMainProgress::BariBeforeRandom(slot) => {
                MainLoopInterruption::SpriteMainBariBeforeRandom(slot)
            }
            SpriteMainProgress::FollowerGraphics {
                slot,
                caller,
                stage,
            } => MainLoopInterruption::SpriteMainFollowerGraphics {
                slot,
                caller,
                stage,
            },
            SpriteMainProgress::AfterThrowableSceneryStateClear(slot) => {
                MainLoopInterruption::SpriteMainAfterThrowableSceneryStateClear(slot)
            }
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
            SpriteMainProgress::MasterSwordLightBeamMovement { slot, checkpoint } => {
                MainLoopInterruption::SpriteMainMasterSwordLightBeamMovement { slot, checkpoint }
            }
            SpriteMainProgress::MasterSwordLightBeamSpawn {
                slot,
                spawned_slot,
                progress,
            } => MainLoopInterruption::SpriteMainMasterSwordLightBeamSpawn {
                slot,
                spawned_slot,
                progress,
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
            SpriteMainProgress::KingZoraFlippersGraphicsStarted(slot) => {
                MainLoopInterruption::SpriteMainKingZoraFlippersGraphicsStarted(slot)
            }
            SpriteMainProgress::AfterSingleSmallDrawPosition(slot) => {
                MainLoopInterruption::SpriteMainAfterSingleSmallDrawPosition(slot)
            }
            SpriteMainProgress::WallmasterResetClear {
                slot,
                cleared_bytes,
            } => MainLoopInterruption::SpriteMainWallmasterResetClear {
                slot,
                cleared_bytes,
            },
            SpriteMainProgress::AfterWallmasterResetPrefix(slot) => {
                MainLoopInterruption::SpriteMainAfterWallmasterResetPrefix(slot)
            }
            SpriteMainProgress::ZazakAfterGraphics(slot) => {
                MainLoopInterruption::SpriteMainZazakAfterGraphics(slot)
            }
            SpriteMainProgress::BonkItemGraphicsStarted(slot) => {
                MainLoopInterruption::SpriteMainBonkItemGraphicsStarted(slot)
            }
            SpriteMainProgress::WishPondTossedItemGraphicsStarted(slot) => {
                MainLoopInterruption::SpriteMainWishPondTossedItemGraphicsStarted(slot)
            }
            SpriteMainProgress::ProbeAfterOamCoordinates(slot) => {
                MainLoopInterruption::SpriteMainProbeAfterOamCoordinates(slot)
            }
            SpriteMainProgress::InitializeResetProperties {
                slot,
                phase,
                completed_stores,
            } => MainLoopInterruption::SpriteMainInitializeResetProperties {
                slot,
                phase,
                completed_stores,
            },
            SpriteMainProgress::InitializeLoadProperties {
                slot,
                phase,
                completed_stores,
            } => MainLoopInterruption::SpriteMainInitializeLoadProperties {
                slot,
                phase,
                completed_stores,
            },
            SpriteMainProgress::FireDebirandoBeforeSpawn(slot) => {
                MainLoopInterruption::SpriteMainFireDebirandoBeforeSpawn(slot)
            }
            SpriteMainProgress::FireDebirandoSpawn {
                slot,
                spawned_slot,
                progress,
            } => MainLoopInterruption::SpriteMainFireDebirandoSpawn {
                slot,
                spawned_slot,
                progress,
            },
            SpriteMainProgress::AfterAntfairySubtype2Increment(slot) => {
                MainLoopInterruption::SpriteMainAfterAntfairySubtype2Increment(slot)
            }
            SpriteMainProgress::AfterLanmolaSubtype2Increment(slot) => {
                MainLoopInterruption::SpriteMainAfterLanmolaSubtype2Increment(slot)
            }
            SpriteMainProgress::InitializePrepPending(slot) => {
                MainLoopInterruption::SpriteMainInitializePrepPending(slot)
            }
            SpriteMainProgress::HogSpearBodyGraphicsPending(slot) => {
                MainLoopInterruption::SpriteMainHogSpearBodyGraphicsPending(slot)
            }
            SpriteMainProgress::AbsorbableHorizontalTileLookup(slot) => {
                MainLoopInterruption::SpriteMainAbsorbableHorizontalTileLookup(slot)
            }
            SpriteMainProgress::AbsorbableVerticalTileLookup(slot) => {
                MainLoopInterruption::SpriteMainAbsorbableVerticalTileLookup(slot)
            }
            SpriteMainProgress::AbsorbableVerticalTileAttributeLoaded(slot) => {
                MainLoopInterruption::SpriteMainAbsorbableVerticalTileAttributeLoaded(slot)
            }
            SpriteMainProgress::SwamolaHeadDraw(slot) => {
                MainLoopInterruption::SpriteMainSwamolaHeadDraw(slot)
            }
            SpriteMainProgress::SwamolaHeadDrawCompleted(slot) => {
                MainLoopInterruption::SpriteMainSwamolaHeadDrawCompleted(slot)
            }
            SpriteMainProgress::VitreousDamagePending(slot) => {
                MainLoopInterruption::SpriteMainVitreousDamagePending(slot)
            }
            SpriteMainProgress::VitreousAiPending(slot) => {
                MainLoopInterruption::SpriteMainVitreousAiPending(slot)
            }
            SpriteMainProgress::VitreousPlayerDamagePending(slot) => {
                MainLoopInterruption::SpriteMainVitreousPlayerDamagePending(slot)
            }
            SpriteMainProgress::SwamolaSegmentDraw { slot, segment } => {
                MainLoopInterruption::SpriteMainSwamolaSegmentDraw { slot, segment }
            }
            SpriteMainProgress::PengatorSlidePending(slot) => {
                MainLoopInterruption::SpriteMainPengatorSlidePending(slot)
            }
            SpriteMainProgress::AntifairyBouncePending(slot) => {
                MainLoopInterruption::SpriteMainAntifairyBouncePending(slot)
            }
            SpriteMainProgress::KholdstareDamagePending(slot) => {
                MainLoopInterruption::SpriteMainKholdstareDamagePending(slot)
            }
            SpriteMainProgress::AfterHelmasaurHardHatBeetleSubtype2Increment(slot) => {
                MainLoopInterruption::SpriteMainAfterHelmasaurHardHatBeetleSubtype2Increment(slot)
            }
            SpriteMainProgress::GuardPrepWeaponFlagsPending(slot) => {
                MainLoopInterruption::SpriteMainGuardPrepWeaponFlagsPending(slot)
            }
            SpriteMainProgress::GuardAnimation { slot, checkpoint } => {
                MainLoopInterruption::SpriteMainGuardAnimation { slot, checkpoint }
            }
            SpriteMainProgress::GuardPrepPatrolDelay { slot, active_call } => {
                MainLoopInterruption::SpriteMainGuardPrepPatrolDelay { slot, active_call }
            }
            SpriteMainProgress::GuardPrepTileCollisionReturned { slot, active_call } => {
                MainLoopInterruption::SpriteMainGuardPrepTileCollisionReturned { slot, active_call }
            }
            SpriteMainProgress::GuardPrepParryHitbox { slot, active_call } => {
                MainLoopInterruption::SpriteMainGuardPrepParryHitbox { slot, active_call }
            }
            SpriteMainProgress::MiniMoldormHistory {
                slot,
                completed_stores,
            } => MainLoopInterruption::SpriteMainMiniMoldormHistory {
                slot,
                completed_stores,
            },
        }
    }
}

fn sprite_prep_load_properties_completed_stores(pc: u32) -> Option<u8> {
    let checkpoints = [
        (SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_PC, 0),
        (SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS2_PC, 1),
        (SPRITE_PREP_LOAD_PROPERTIES_AFTER_HEALTH_PC, 2),
        (SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS4_PC, 3),
        (SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS5_PC, 4),
        (SPRITE_PREP_LOAD_PROPERTIES_AFTER_DEFLECTION_PC, 5),
        (SPRITE_PREP_LOAD_PROPERTIES_AFTER_BUMP_DAMAGE_PC, 6),
        (SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS_PC, 7),
        (SPRITE_PREP_LOAD_PROPERTIES_AFTER_ROOM_PC, 8),
        (SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS3_PC, 9),
        (SPRITE_PREP_LOAD_PROPERTIES_AFTER_OAM_FLAGS_PC, 10),
    ];
    if !(SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_PC..=SPRITE_PREP_LOAD_PROPERTIES_RETURN_PC)
        .contains(&pc)
    {
        return None;
    }
    checkpoints
        .iter()
        .rev()
        .find_map(|&(start, completed)| (pc >= start).then_some(completed))
}

fn sprite_prep_reset_properties_completed_stores(pc: u32) -> Option<u8> {
    if (SPRITE_PREP_RESET_PROPERTIES_START_PC..SPRITE_PREP_RESET_PROPERTIES_ACCUMULATOR_CLEAR_PC)
        .contains(&pc)
    {
        let offset = pc - SPRITE_PREP_RESET_PROPERTIES_START_PC;
        return (offset % 3 == 0).then_some((offset / 3) as u8);
    }
    if (SPRITE_PREP_RESET_PROPERTIES_ACCUMULATOR_CLEAR_PC
        ..SPRITE_PREP_RESET_PROPERTIES_LONG_STORES_START_PC)
        .contains(&pc)
    {
        return Some(35);
    }
    if (SPRITE_PREP_RESET_PROPERTIES_LONG_STORES_START_PC..SPRITE_PREP_RESET_PROPERTIES_RETURN_PC)
        .contains(&pc)
    {
        let offset = pc - SPRITE_PREP_RESET_PROPERTIES_LONG_STORES_START_PC;
        return (offset % 4 == 0).then_some(35 + (offset / 4) as u8);
    }
    (pc == SPRITE_PREP_RESET_PROPERTIES_RETURN_PC).then_some(40)
}

impl CachedSpriteExecutionTracker {
    fn from_observed_write(pc: u32, slot: u8, field_index: usize) -> Self {
        if pc >= UNCACHE_SPRITE_RESTORE_START_PC {
            Self {
                slot,
                copied_fields: CACHED_SPRITE_LIVE_FIELDS.len() as u8,
                restored_fields: (CACHED_SPRITE_LIVE_FIELDS.len() - field_index) as u8,
                restore_started: true,
                body_progress: None,
            }
        } else {
            Self {
                slot,
                copied_fields: (field_index + 1) as u8,
                restored_fields: 0,
                restore_started: false,
                body_progress: None,
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
        } else if let Some(progress) = self.body_progress {
            CachedSpriteExecutionProgress::Executing {
                slot: self.slot,
                progress,
            }
        } else {
            CachedSpriteExecutionProgress::Loading {
                slot: self.slot,
                copied_fields: self.copied_fields,
            }
        }
    }
}

fn observe_dynamic_spawn_progress_write(
    tracker: &mut Option<(u8, u8, SpriteDynamicSpawnProgress)>,
    event: &RawTraceEvent,
    caller: &str,
) -> Result<(), String> {
    let Some((slot, spawned_slot, _)) = *tracker else {
        return Ok(());
    };
    let pc = event.pc.ok_or("Snes9x WRAM write omitted PC")? & 0x00ff_ffff;
    let address = event.address.ok_or("Snes9x WRAM write omitted address")?;
    let spawned_address = |base: u16| base + u16::from(spawned_slot);
    let progress = if pc == SPRITE_SPAWN_DYNAMICALLY_STATE_STORE_PC {
        if event.y != Some(u16::from(spawned_slot))
            || address != spawned_address(SPRITE_STATE_BASE)
            || event.value != Some(9)
        {
            return Err(format!(
                "Snes9x {caller} spawn state publication disagreed on slot {spawned_slot}"
            ));
        }
        Some(SpriteDynamicSpawnProgress::StatePublished)
    } else if let Some(completed_stores) = sprite_prep_reset_properties_completed_stores(pc) {
        (event.x == Some(u16::from(spawned_slot)))
            .then_some(SpriteDynamicSpawnProgress::ResetProperties { completed_stores })
    } else if let Some(completed_stores) = sprite_prep_load_properties_completed_stores(pc) {
        (event.x == Some(u16::from(spawned_slot)))
            .then_some(SpriteDynamicSpawnProgress::LoadProperties { completed_stores })
    } else {
        match pc {
            SPRITE_SPAWN_DYNAMICALLY_IDENTITY_STORE_PC => {
                let indoor_address = SPRITE_N_BASE + u16::from(spawned_slot);
                let outdoor_low_address = SPRITE_N_BASE + u16::from(spawned_slot) * 2;
                let outdoor_high_address = SPRITE_N_BASE + u16::from(spawned_slot) * 2 + 1;
                if !matches!(
                    address,
                    a if a == indoor_address
                        || a == outdoor_low_address
                        || a == outdoor_high_address
                ) || event.value != Some(0xff)
                {
                    return Err(format!(
                        "Snes9x {caller} spawn identity publication disagreed on slot {spawned_slot}: address=${address:04x}, value={:?}",
                        event.value,
                    ));
                }
                // Outdoors this is one 16-bit CPU store and the trace emits
                // its low and high WRAM writes separately. NMI cannot split
                // the instruction, so the low byte validates provenance but
                // only the high byte publishes the completed C assignment.
                (address != outdoor_low_address || address == indoor_address)
                    .then_some(SpriteDynamicSpawnProgress::IdentityPublished)
            }
            SPRITE_SPAWN_DYNAMICALLY_FLOOR_STORE_PC => {
                if event.y != Some(u16::from(spawned_slot))
                    || address != spawned_address(SPRITE_FLOOR_BASE)
                {
                    return Err(format!(
                        "Snes9x {caller} spawn floor publication disagreed on slot {spawned_slot}"
                    ));
                }
                Some(SpriteDynamicSpawnProgress::FloorPublished)
            }
            SPRITE_SPAWN_DYNAMICALLY_DIRECTION_STORE_PC => {
                if event.y != Some(u16::from(spawned_slot))
                    || address != spawned_address(SPRITE_DIRECTION_BASE)
                {
                    return Err(format!(
                        "Snes9x {caller} spawn direction publication disagreed on slot {spawned_slot}"
                    ));
                }
                Some(SpriteDynamicSpawnProgress::DirectionPublished)
            }
            SPRITE_SPAWN_DYNAMICALLY_DIE_ACTION_STORE_PC => {
                if event.y != Some(u16::from(spawned_slot))
                    || address != spawned_address(SPRITE_DIE_ACTION_BASE)
                {
                    return Err(format!(
                        "Snes9x {caller} spawn die-action publication disagreed on slot {spawned_slot}"
                    ));
                }
                Some(SpriteDynamicSpawnProgress::DieActionCleared)
            }
            SPRITE_SPAWN_DYNAMICALLY_SUBTYPE_STORE_PC => {
                if event.y != Some(u16::from(spawned_slot))
                    || address != spawned_address(SPRITE_SUBTYPE_BASE)
                {
                    return Err(format!(
                        "Snes9x {caller} spawn subtype publication disagreed on slot {spawned_slot}"
                    ));
                }
                Some(SpriteDynamicSpawnProgress::SubtypeCleared)
            }
            _ => None,
        }
    };
    if let Some(progress) = progress {
        *tracker = Some((slot, spawned_slot, progress));
    }
    Ok(())
}

fn observe_dynamic_spawn_progress_boundary(
    tracker: &mut Option<(u8, u8, SpriteDynamicSpawnProgress)>,
    event: &RawTraceEvent,
) -> Result<(), String> {
    let Some((slot, spawned_slot, _)) = *tracker else {
        return Ok(());
    };
    if event.x != Some(u16::from(spawned_slot)) {
        return Ok(());
    }
    let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
        return Ok(());
    };
    let progress = if let Some(completed_stores) = sprite_prep_reset_properties_completed_stores(pc)
    {
        Some(SpriteDynamicSpawnProgress::ResetProperties { completed_stores })
    } else {
        sprite_prep_load_properties_completed_stores(pc)
            .map(|completed_stores| SpriteDynamicSpawnProgress::LoadProperties { completed_stores })
    };
    if let Some(progress) = progress {
        *tracker = Some((slot, spawned_slot, progress));
    }
    Ok(())
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
    last_host_return_reset_progress: Option<DungeonResetSpritesCpuProgress>,
    cached_sprite_execution: Option<CachedSpriteExecutionTracker>,
    overworld_presence_published: bool,
    overworld_sprite_activation: Option<OverworldSpriteActivationTracker>,
    overworld_load_overlays_sprite_reload_active: bool,
    /// The active overworld reload's inner `Sprite_ResetAll_noDisable` has
    /// already exposed its one source completion edge. The call can remain
    /// suspended in that routine for another host, so this must not be
    /// inferred anew from every later NMI/host return at the same PC range.
    overworld_sprite_reload_reset_published: bool,
    rescued_maiden_initialization: Option<RescuedMaidenInitializationTracker>,
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
    host_dialogue_scroll_progress: Vec<zelda3::DialogueScrollProgressReceipt>,
}

const SEMANTIC_TRACE_CHECKPOINT_SCHEMA: u32 = 19;

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
    #[serde(default)]
    last_host_return_reset_progress: Option<DungeonResetSpritesCpuProgress>,
    cached_sprite_execution: Option<CachedSpriteExecutionTracker>,
    overworld_presence_published: bool,
    overworld_sprite_activation: Option<OverworldSpriteActivationTracker>,
    #[serde(default)]
    overworld_load_overlays_sprite_reload_active: bool,
    #[serde(default)]
    overworld_sprite_reload_reset_published: bool,
    #[serde(default)]
    rescued_maiden_initialization: Option<RescuedMaidenInitializationTracker>,
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
    stack1: Option<u8>,
    #[serde(default)]
    a: Option<u16>,
    #[serde(default)]
    main: Option<u8>,
    #[serde(default)]
    sub: Option<u8>,
    #[serde(default)]
    subsub: Option<u8>,
    #[serde(default)]
    room: Option<u16>,
    #[serde(default)]
    frame_counter: Option<u8>,
    #[serde(default)]
    nmi_latch: Option<u8>,
    #[serde(default)]
    link_y: Option<u16>,
    #[serde(default)]
    bg2_v: Option<u16>,
    #[serde(default)]
    bg2_h: Option<u16>,
    #[serde(default)]
    spotlight_radius: Option<u16>,
    #[serde(default)]
    spotlight_var4_low: Option<u8>,
    palette_countdown: Option<u8>,
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

fn dungeon_peg_attribute_flip_progress(
    event: &RawTraceEvent,
    boundary: OriginalTimingBoundary,
) -> Result<Option<DungeonPegAttributeFlipProgressReceipt>, String> {
    let selectable_caller = event.main == Some(7)
        && matches!(
            (event.sub, event.subsub),
            (Some(2), Some(1..=u8::MAX))
                | (Some(6), Some(3..=u8::MAX))
                | (Some(7), Some(6..=u8::MAX))
                | (Some(0x0e), Some(7..=u8::MAX))
                | (Some(0x11..=0x13), Some(3..=u8::MAX))
                | (Some(0x15), Some(3..=u8::MAX))
        );
    let update_pegs_caller =
        (event.main, event.sub, event.subsub) == (Some(7), Some(0x16), Some(0x10));
    if !selectable_caller && !update_pegs_caller {
        return Ok(None);
    }
    let pc = event.pc.map(|pc| pc & 0x00ff_ffff);
    let completed_banks = match pc {
        Some(pc) if (DUNGEON_PEG_FLIP_LOOP_START_PC..DUNGEON_PEG_FLIP_BANK_B_PC).contains(&pc) => 0,
        Some(pc) if (DUNGEON_PEG_FLIP_BANK_B_PC..DUNGEON_PEG_FLIP_BANK_C_PC).contains(&pc) => 1,
        Some(pc) if (DUNGEON_PEG_FLIP_BANK_C_PC..DUNGEON_PEG_FLIP_BANK_D_PC).contains(&pc) => 2,
        Some(pc) if (DUNGEON_PEG_FLIP_BANK_D_PC..DUNGEON_PEG_FLIP_DECREMENT_PC).contains(&pc) => 3,
        Some(DUNGEON_PEG_FLIP_DECREMENT_PC) => 4,
        // DEX has already selected the next index at the branch. X=$ffff is
        // the source's exact exhausted-loop cursor.
        Some(DUNGEON_PEG_FLIP_BRANCH_PC) => 0,
        Some(pc)
            if (DUNGEON_PEG_FLIP_INDEX_EXHAUSTED_PC..DUNGEON_PEG_FLIP_RETURN_PC).contains(&pc) =>
        {
            0
        }
        _ => return Ok(None),
    };
    let index = event
        .x
        .ok_or("Snes9x peg-attribute flip boundary omitted source index X")?;
    if index > 0x07ff && index != 0xffff {
        return Err(format!(
            "Snes9x peg-attribute flip used invalid source index ${index:04x}",
        ));
    }
    if index == 0xffff
        && !matches!(
            pc,
            Some(DUNGEON_PEG_FLIP_BRANCH_PC)
                | Some(DUNGEON_PEG_FLIP_INDEX_EXHAUSTED_PC..DUNGEON_PEG_FLIP_RETURN_PC)
        )
    {
        return Err(format!(
            "Snes9x peg-attribute flip exposed exhausted X at invalid PC ${:06x}",
            pc.unwrap_or_default(),
        ));
    }
    Ok(Some(DungeonPegAttributeFlipProgressReceipt {
        index,
        completed_banks,
        boundary,
    }))
}

fn file_select_graphics_low_wram_clear_progress(
    event: &RawTraceEvent,
) -> Result<Option<FileSelectGraphicsLowWramClearProgress>, String> {
    // `Intro_ValidateSram` has one source caller, Module_SelectFile_0. Its
    // long graphics load leaves `$b0` as scratch (`0xd6` on this route), so
    // subsubmodule is not caller authority here.
    if (event.main, event.sub) != (Some(1), Some(1)) {
        return Ok(None);
    }
    let completed_page_stores = match event.pc.map(|pc| pc & 0x00ff_ffff) {
        Some(FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_AFTER_PAGE_D_PC) => 1,
        Some(FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_AFTER_PAGE_E_PC) => 2,
        Some(FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_AFTER_PAGE_F_PC) => 3,
        _ => return Ok(None),
    };
    let word_offset = event
        .x
        .ok_or("Snes9x file-select low-WRAM clear checkpoint omitted X")?;
    if word_offset > 0xfe || word_offset & 1 != 0 {
        return Err(format!(
            "Snes9x file-select low-WRAM clear checkpoint has invalid word offset ${word_offset:04x}",
        ));
    }
    Ok(Some(FileSelectGraphicsLowWramClearProgress {
        word_offset: word_offset as u8,
        completed_page_stores,
    }))
}

fn publish_file_select_graphics_low_wram_clear_progress(
    receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    progress: FileSelectGraphicsLowWramClearProgress,
) {
    // Only the furthest cumulative prefix at a given source boundary matters.
    // Replacing it here also retains its exact position relative to any NMI
    // lifecycle events which follow the last observed store.
    receipts.retain(|receipt| {
        !matches!(
            receipt,
            OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramClearProgress(_)
        )
    });
    receipts.push(OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramClearProgress(progress));
}

fn special_exit_mosaic_restore_checkpoint(event: &RawTraceEvent) -> Result<bool, String> {
    if event.event != "pc"
        || event.pc.map(|pc| pc & 0x00ff_ffff) != Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC)
        || event.return_address.map(|pc| pc & 0x00ff_ffff)
            != Some(SPECIAL_EXIT_MOSAIC_SECOND_DECODE_RETURN_ADDRESS)
    {
        return Ok(false);
    }
    if !matches!((event.main, event.sub), (Some(0x0b), Some(0x24))) {
        return Err(format!(
            "Snes9x special-exit second decode entered outside Module0B/$24: main={:?} sub={:?}",
            event.main, event.sub,
        ));
    }
    Ok(true)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HostFrameState {
    run: u64,
    pc: u32,
    a: Option<u16>,
    x: Option<u16>,
    y: Option<u16>,
    main: u8,
    sub: u8,
    subsub: u8,
    frame_counter: u8,
    nmi_latch: u8,
    bg2_h: Option<u16>,
    bg2_v: Option<u16>,
    link_y: Option<u16>,
    spotlight_radius: Option<u16>,
    spotlight_var4_low: Option<u8>,
    palette_countdown: Option<u8>,
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
    mirror_portal_spawn_slot: Option<u8>,
    mirror_portal_reset_progress: Option<u8>,
    entry: Option<HostFrameState>,
    returned: Option<HostFrameState>,
    vwf_nmi_observed: bool,
    /// Whether the most recently accepted NMI interrupted the committed body
    /// of `VWF_RenderSingle`. When a host ends at NMI entry, that interrupted
    /// PC is the terminal source position; the frame-return PC itself cannot
    /// describe which part of the suspended glyph call already ran.
    last_nmi_interrupted_vwf_glyph_body: bool,
    main_loop_starts: u8,
    main_loop_common_suffix_completed: bool,
    /// The host began inside the previous iteration's common suffix, before
    /// its `$12` clear (entry at `$00:805D`, route host 511525): that leading
    /// completion belongs to the carried iteration and a fresh iteration may
    /// complete its own suffix later in the same host.
    leading_common_suffix_completed: bool,
    /// This host executed inside the concrete `Overworld_LoadOverlays` sprite
    /// reload call. Cross-host ownership is supplied by the semantic decoder;
    /// seeing the private entry PC also sets it for the entry host itself.
    overworld_load_overlays_sprite_reload_active: bool,
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
        if event.event == "wram-write"
            && event.pc == Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC)
            && event.return_address == Some(0x09_afa5)
        {
            let slot = event
                .y
                .filter(|slot| *slot < 16)
                .ok_or("mirror portal spawn omitted its selected slot")?
                as u8;
            if event.x != Some(0xff)
                || event.value != Some(0x6c)
                || event.address != Some(SPRITE_TYPE_BASE + u16::from(slot))
            {
                return Err("mirror portal spawn disagreed with its source caller".into());
            }
            self.mirror_portal_spawn_slot = Some(slot);
        }
        if event.event == "frame"
            && event.stage.as_deref() == Some("return")
            && event.return_address == Some(SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_RETURN_ADDRESS)
        {
            if let Some(slot) = self.mirror_portal_spawn_slot {
                if event.x == Some(u16::from(slot)) {
                    self.mirror_portal_reset_progress = event
                        .pc
                        .and_then(sprite_prep_reset_properties_completed_stores);
                }
            }
        }
        if event.event == "pc"
            && event.pc.map(|pc| pc & 0x00ff_ffff)
                == Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC)
            && event
                .return_address
                .map(|pc| pc & 0x00ff_ffff)
                .is_some_and(|return_pc| {
                    matches!(
                        return_pc,
                        OVERWORLD_LOAD_OVERLAYS_AFTER_SPRITE_RELOAD_PC
                            | BIRD_TRAVEL_AFTER_SPRITE_RELOAD_PC
                            | MIRROR_WARP_AFTER_SPRITE_RELOAD_PC
                            | PRE_OVERWORLD_AFTER_SPRITE_RELOAD_PC
                    )
                })
        {
            self.overworld_load_overlays_sprite_reload_active = true;
        }
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
        // A carried handler can resume the glyph and reach Scroll before
        // this host's next NMI. Its resume PC proves the same suspended VWF
        // caller as an acceptance within the glyph on this host.
        if matches!(event.event.as_str(), "nmi" | "nmi-resume") {
            let interrupted_pc = event.pc.map(|pc| pc & 0x00ff_ffff);
            let interrupted_vwf = interrupted_pc.is_some_and(|pc| {
                (VWF_RENDER_SINGLE_START_PC..VWF_RENDER_SINGLE_END_PC).contains(&pc)
            });
            if interrupted_vwf {
                self.vwf_nmi_observed = true;
            }
            if event.event == "nmi" {
                self.last_nmi_interrupted_vwf_glyph_body = interrupted_pc.is_some_and(|pc| {
                    (VWF_RENDER_SINGLE_BODY_START_PC..VWF_RENDER_SINGLE_END_PC).contains(&pc)
                });
            }
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
            a: event.a,
            x: event.x,
            y: event.y,
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
            bg2_h: event.bg2_h,
            bg2_v: event.bg2_v,
            link_y: event.link_y,
            spotlight_radius: event.spotlight_radius,
            spotlight_var4_low: event.spotlight_var4_low,
            palette_countdown: event.palette_countdown,
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
        // A portal reset belongs to the spawn proven by its private caller.
        if let Some(completed_stores) = self.mirror_portal_reset_progress {
            let slot = self
                .mirror_portal_spawn_slot
                .expect("portal reset has a spawn owner");
            let receipt = receipts
                .iter_mut()
                .find(|receipt| {
                    **receipt
                        == OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                            OverworldSpriteReloadProgress::GenerationReturned,
                        )
                })
                .ok_or("mirror portal reset omitted its enclosing generation return")?;
            *receipt = OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturnedAtPortalReset {
                    slot,
                    completed_stores,
                },
            );
        }
        // Ancilla_TerminateSelectInteractives, immediately before the
        // non-carried-object pickup test. GenerationReturned supplies the
        // enclosing reload provenance; module/submodule identifies its caller.
        if matches!(returned.pc, 0x09_ac9c | 0x09_aca6)
            && ((returned.main == 9 && returned.sub == 0x23)
                || (returned.main == 0x15 && matches!(returned.sub, 3 | 4)))
        {
            let slot = returned
                .x
                .filter(|slot| *slot < 6)
                .ok_or("Snes9x interactive cleanup omitted its valid ancilla slot")?
                as u8;
            let mut found = false;
            for receipt in receipts.iter_mut() {
                if *receipt
                    == OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                        OverworldSpriteReloadProgress::GenerationReturned,
                    )
                {
                    *receipt = OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                        if returned.pc == 0x09_aca6 {
                            OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveTypeClear { slot }
                        } else {
                            OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveCleanup {
                                slot,
                            }
                        },
                    );
                    found = true;
                }
            }
            if !found {
                return Err("Snes9x mirror cleanup checkpoint lacks its generation return".into());
            }
        }
        let overworld_sprite_scan_suspended = (OVERWORLD_SPRITE_SCAN_START_PC
            ..OVERWORLD_SPRITE_SCAN_END_PC)
            .contains(&returned.pc)
            && (self.overworld_load_overlays_sprite_reload_active
                || (entry.main == 9
                    && matches!(entry.sub, 4 | 0x12)
                    && returned.main == entry.main
                    && returned.sub == entry.sub)
                || (entry.main == 8 && entry.sub == 0 && returned.main == 8 && returned.sub == 0));
        if overworld_sprite_scan_suspended
            && self.main_loop_starts == 0
            && entry.bg2_h != returned.bg2_h
        {
            receipts.push(
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::ProximityScanSuspended {
                        bg2_h: returned.bg2_h.ok_or(
                            "Snes9x Overworld_LoadOverlays host return omitted the BG2 scan coordinate",
                        )?,
                    },
                ),
            );
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
        if let Some(phase) = desert_prayer_iris_interruption(
            returned.pc,
            Some(returned.main),
            Some(returned.sub),
            Some(returned.subsub),
            returned.spotlight_radius,
            returned.spotlight_var4_low,
            returned.palette_countdown,
            returned.link_y,
            returned.bg2_v,
            returned.a,
            returned.x,
            returned.y,
        )?
        .or(desert_prayer_palette_filter_interruption(
            returned.pc,
            Some(returned.main),
            Some(returned.sub),
            Some(returned.subsub),
            returned.palette_countdown,
            returned.x,
        )?)
        .or_else(|| {
            main_loop_interruption_for_source_state(
                returned.pc,
                Some(returned.main),
                Some(returned.sub),
                returned.x,
            )
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
        // The selector at $0D:A61A is read-only: all initial LinkOam stores
        // precede it, and equipment drawing plus stair-Y restoration follow it.
        if let Some(progress) = link_oam_stair_progress(returned.pc, Some(returned.sub)) {
            receipts.push(OriginalTimingSemanticReceipt::LinkOamStairProgress(
                progress,
            ));
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
            // A completed enclosing caller supersedes the intermediate VWF
            // decoder endpoint. Stopping at that endpoint would omit the
            // terminal command body (including WAIT/END countdown stores).
            && !receipts.iter().any(|receipt| {
                matches!(receipt, OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted)
            })
        {
            let message_read_position = dialogue_message_read_position
                .ok_or("Snes9x dialogue continuation omitted its semantic message read position")?;
            let current_glyph_started = (VWF_RENDER_SINGLE_BODY_START_PC..VWF_RENDER_SINGLE_END_PC)
                .contains(&returned.pc)
                || (returned.pc == NMI_HANDLER_ENTRY_PC
                    && self.last_nmi_interrupted_vwf_glyph_body);
            let progress = if current_glyph_started {
                DialogueExecutionProgress::ResumedRenderingWithCurrentGlyphStarted {
                    message_read_position,
                }
            } else {
                DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                    message_read_position,
                }
            };
            receipts.push(OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                progress,
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
        if entry.main == 0x0b && entry.sub == 0x24 && returned.main == 0x0b && returned.sub == 0x25
        {
            // A fast terminal host can observe both the second-decode entry
            // and the enclosing caller return. The terminal source fact
            // supersedes the intermediate checkpoint.
            receipts.retain(|receipt| {
                !matches!(
                    receipt,
                    OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicRestored
                )
            });
            receipts.push(OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicReturned);
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
                        "0280d3", "0280d6", "0280d9", "0280dd", "028842", "05df49", "05df4d",
                        "05cbcd", "05eb1d", "05eb21", "068328", "0683a7", "0684e2", "0684aa",
                        "058af3", "0684eb", "069271", "06a628", "06a724", "06b9cc", "06b9d0",
                        "0799ad", "079a0b", "008225", "0082c7", "00d4ed", "09c499", "09c4aa",
                        "09c173", "09f63f", "09f825", "0ffdc3", "00d423", "00e75c", "00e766",
                        "00d44c", "0ecfe2", "0ed088", "0ed0c2", "06d051", "02824d",
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
                        "0011",
                        "001a",
                        "00b0",
                        "0020-002f",
                        "02ec",
                        "0b00-0b1d",
                        "0b6a",
                        "0b89-0b98",
                        "0ba0-0baf",
                        "0bc0-0bdf",
                        "0cba-0cc9",
                        "0c4a-0c53",
                        "0d00-0d3f",
                        "0d60-0d7f",
                        "0d80-0dff",
                        "0e20-0e2f",
                        "0e30-0e3f",
                        "0e40-0e4f",
                        "0e60-0e6f",
                        "0e80-0e9f",
                        "0eb0-0ebf",
                        "0f20-0f2f",
                        "0f50-0f5f",
                        "0f70-0f7f",
                        "0fba",
                        "0fb5-0fb6",
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: self.last_host_return_reset_progress,
            cached_sprite_execution: self.cached_sprite_execution,
            overworld_presence_published: self.overworld_presence_published,
            overworld_sprite_activation: self.overworld_sprite_activation,
            overworld_load_overlays_sprite_reload_active: self
                .overworld_load_overlays_sprite_reload_active,
            overworld_sprite_reload_reset_published: self.overworld_sprite_reload_reset_published,
            rescued_maiden_initialization: self.rescued_maiden_initialization,
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
        self.last_host_return_reset_progress = checkpoint.last_host_return_reset_progress;
        self.cached_sprite_execution = checkpoint.cached_sprite_execution;
        self.overworld_presence_published = checkpoint.overworld_presence_published;
        self.overworld_sprite_activation = checkpoint.overworld_sprite_activation;
        self.overworld_load_overlays_sprite_reload_active =
            checkpoint.overworld_load_overlays_sprite_reload_active;
        self.overworld_sprite_reload_reset_published =
            checkpoint.overworld_sprite_reload_reset_published;
        if checkpoint
            .rescued_maiden_initialization
            .is_some_and(|tracker| tracker.completed_bytes > RESCUED_MAIDEN_FOLLOWER_SHEET_BYTES)
        {
            return Err(
                "Snes9x semantic checkpoint has invalid rescued-maiden decompressor progress"
                    .to_string(),
            );
        }
        self.rescued_maiden_initialization = checkpoint.rescued_maiden_initialization;
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
        if !self.host_dialogue_scroll_progress.is_empty() {
            return Err("prior Snes9x dialogue scroll progress was not consumed".to_string());
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
        let mut dialogue_scroll = DialogueScrollHostWindow::default();
        host_frame.overworld_load_overlays_sprite_reload_active =
            self.overworld_load_overlays_sprite_reload_active;
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
            dialogue_scroll.observe(&event)?;
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
            if event.event == "pc" {
                if let Some(progress) = file_select_graphics_low_wram_clear_progress(&event)? {
                    publish_file_select_graphics_low_wram_clear_progress(&mut receipts, progress);
                }
                if special_exit_mosaic_restore_checkpoint(&event)? {
                    if receipts.iter().any(|receipt| {
                        matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicRestored
                                | OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicReturned
                        )
                    }) {
                        return Err(
                            "Snes9x special-exit mosaic restore checkpoint replayed in one host"
                                .to_string(),
                        );
                    }
                    receipts
                        .push(OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicRestored);
                }
            }
            if main_loop_started {
                self.zelda_run_game_loop_call_active = true;
            }
            if main_loop_common_suffix_completed {
                if let Some(tracker) = self.rescued_maiden_initialization.take() {
                    if tracker.phase != RescuedMaidenInitializationTrackerPhase::Converting {
                        return Err(format!(
                            "Snes9x rescued-maiden caller reached the main-loop suffix from {:?}",
                            tracker.phase,
                        ));
                    }
                }
                self.zelda_run_game_loop_call_active = false;
            }
            self.consume_event(event, &mut receipts)?;
        }
        if let Some(returned_event) = returned_event.as_ref() {
            self.publish_overworld_presence_at_scan_boundary(returned_event, &mut receipts);
            if returned_event.pc.map(|pc| pc & 0xffffff) == Some(0x02_d987)
                && (returned_event.main, returned_event.sub) == (Some(5), Some(0))
            {
                receipts.push(OriginalTimingSemanticReceipt::SelectedGameEntranceBeforeSelection);
            }
            if let Some(progress) = file_select_graphics_low_wram_clear_progress(returned_event)? {
                publish_file_select_graphics_low_wram_clear_progress(&mut receipts, progress);
            }
            if let Some(progress) = credits_scene_load_boundary_progress(
                returned_event,
                OriginalTimingBoundary::HostReturn,
            )? {
                receipts.retain(|receipt| {
                    !matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(_)
                    )
                });
                receipts.push(OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(
                    progress,
                ));
            }
            if let Some(progress) = credits_end_sequence_32_boundary_progress(
                returned_event,
                OriginalTimingBoundary::HostReturn,
            )? {
                receipts.retain(|receipt| {
                    !matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::CreditsEndSequence32Progress(_)
                    )
                });
                receipts.push(OriginalTimingSemanticReceipt::CreditsEndSequence32Progress(
                    progress,
                ));
            }
            if let Some(progress) = rescued_maiden_tilemap_clear_progress(
                returned_event,
                OriginalTimingBoundary::HostReturn,
            )? {
                receipts.push(
                    OriginalTimingSemanticReceipt::RescuedMaidenTilemapClearProgress(progress),
                );
            }
            if let Some(progress) = triforce_room_case2_palette_progress(
                returned_event,
                OriginalTimingBoundary::HostReturn,
            )? {
                receipts.push(
                    OriginalTimingSemanticReceipt::TriforceRoomCase2PaletteProgress(progress),
                );
            }
            if let Some(progress) = dungeon_peg_attribute_flip_progress(
                returned_event,
                OriginalTimingBoundary::HostReturn,
            )? {
                receipts.retain(|receipt| {
                    !matches!(
                        receipt,
                        OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(_)
                    )
                });
                receipts
                    .push(OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(progress));
            }
            if let Some(tracker) = self.rescued_maiden_initialization.as_mut() {
                tracker.observe_boundary(returned_event)?;
                receipts.push(
                    OriginalTimingSemanticReceipt::RescuedMaidenInitializationProgress(
                        tracker.host_return_receipt()?,
                    ),
                );
            }
            if let Some(tracker) = self
                .sprite_main_execution
                .as_mut()
                .and_then(|execution| execution.follower_graphics.as_mut())
            {
                tracker.1.observe_boundary(returned_event)?;
            }
            if let Some(execution) = self.sprite_main_execution.as_mut() {
                execution.observe_guard_prep_weapon_flags_pending(returned_event)?;
                execution.observe_guard_animation_checkpoint(returned_event)?;
                execution.observe_hog_spear_body_graphics_pending(returned_event)?;
                execution.observe_absorbable_tile_lookup(returned_event)?;
                execution.observe_swamola_segment_draw(returned_event)?;
                execution.observe_vitreous_damage_pending(returned_event);
                execution.observe_dispatch_trampoline_return(returned_event)?;
                execution.observe_pengator_slide_pending(returned_event)?;
                execution.observe_antifairy_bounce_pending(returned_event)?;
                execution.observe_kholdstare_damage_pending(returned_event)?;
                execution.observe_guard_prep_parry_hitbox(returned_event)?;
                execution.observe_guard_prep_patrol_delay(returned_event)?;
                execution.observe_guard_prep_tile_collision_return(returned_event)?;
                execution.observe_fire_debirando_spawn_boundary(returned_event)?;
                execution.observe_master_sword_light_beam_spawn_boundary(returned_event)?;
                execution.observe_bari_before_random(returned_event)?;
                execution.observe_main_and_aux1_timer_decrements(returned_event)?;
                execution.observe_main_timer_decrement(returned_event)?;
                execution.observe_zero_hit_timer_clear(returned_event)?;
                execution.observe_primary_timer_decrements(returned_event)?;
                execution.observe_hit_timer(returned_event)?;
                execution.observe_timer_decrements(returned_event)?;
                execution.observe_single_small_draw_position(returned_event)?;
                execution.observe_probe_after_oam_coordinates(returned_event)?;
                execution.observe_initialize_reset_properties(returned_event)?;
                execution.observe_initialize_load_properties(returned_event)?;
                execution.observe_initialize_prep_pending(returned_event)?;
                execution.observe_fire_debirando_before_spawn(returned_event)?;
                execution.observe_zazak_graphics(returned_event)?;
                execution.observe_wallmaster_reset_prefix(returned_event)?;
            }
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
            if dungeon_push_blocks_pending(returned_event) {
                receipts.push(OriginalTimingSemanticReceipt::DungeonPushBlocksPending);
            }
            if publish_pre_dungeon_sprite_reset_progress(
                returned_event,
                OriginalTimingBoundary::HostReturn,
                host_frame.overworld_load_overlays_sprite_reload_active
                    && !self.overworld_sprite_reload_reset_published,
                &mut receipts,
            )? {
                // The shared Sprite_DisableAll candidate belongs to the
                // enclosing Sprite_ResetAll call identified above, not to the
                // later Dungeon_ResetSprites call. Keep the domains separate.
                self.pending_reset_progress = None;
                if host_frame.overworld_load_overlays_sprite_reload_active {
                    self.overworld_sprite_reload_reset_published = true;
                }
            } else if let Some(progress) = dungeon_reset_sprites_caller_progress(returned_event) {
                self.pending_reset_progress = Some(progress);
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
        self.host_dialogue_scroll_progress = dialogue_scroll.finish();
        host_frame.finish(
            &mut receipts,
            dialogue_message_read_position,
            zelda_run_game_loop_call_active_at_entry,
        )?;
        let credits_text_returned_to_sprite_preparation = receipts.iter().any(|receipt| {
            matches!(
                receipt,
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpritePreparation
                        | MainLoopInterruption::SpritePreparationExtendedOamPacking { .. }
                )
            )
        });
        if credits_text_returned_to_sprite_preparation {
            for receipt in &mut receipts {
                if let OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(progress) = receipt {
                    progress.progress = CreditsSceneLoadProgress::EndingTextCompleted;
                }
            }
        }
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
                        MainLoopInterruption::LinkActualVelocity { .. }
                            | MainLoopInterruption::LinkActualVelocityCompleted
                            | MainLoopInterruption::DungeonExitSpotlightTableCompleted
                            | MainLoopInterruption::LinkPositionBeforeCoordinates
                            | MainLoopInterruption::LinkPositionAfterSubpixel { .. }
                            | MainLoopInterruption::LinkPositionAfterCoordinateLow { .. }
                            | MainLoopInterruption::LinkPositionAfterCoordinates { .. }
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

    pub(crate) fn take_host_dialogue_scroll_progress(
        &mut self,
    ) -> Vec<zelda3::DialogueScrollProgressReceipt> {
        std::mem::take(&mut self.host_dialogue_scroll_progress)
    }

    fn flush_reset_progress(
        &mut self,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
        boundary: OriginalTimingBoundary,
    ) {
        let prior_host_return = self.last_host_return_reset_progress.take();
        if let Some(progress) = self.pending_reset_progress.take() {
            // A leading NMI can land in the same non-mutating source range
            // already published at host return. That is a restatement, not
            // a native continuation transition. Genuine new progress survives.
            if boundary == OriginalTimingBoundary::NmiAccepted
                && prior_host_return == Some(progress)
            {
                return;
            }
            if boundary == OriginalTimingBoundary::HostReturn {
                self.last_host_return_reset_progress = Some(progress);
            }
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
            // A caller-specific Sprite_Main/Uncle item-receipt continuation
            // already owns the exact suspended statement and the surrounding
            // Sprite_Main remainder.  Do not also publish an earlier generic
            // slot checkpoint: it is dominated by the typed call boundary and
            // has no independent consumer.  Direct Link_ReceiveItem calls keep
            // the generic checkpoint because their typed receipt owns only the
            // nested graphics call.
            let caller_specific_item_receipt = matches!(
                self.item_receipt_caller,
                Some(
                    ItemReceiptGraphicsCaller::SpriteMain { .. }
                        | ItemReceiptGraphicsCaller::UnclePassage { .. }
                )
            );
            if !caller_specific_item_receipt {
                if let Some(execution) = self.sprite_main_execution {
                    receipts.push(OriginalTimingSemanticReceipt::SpriteMainProgressed(
                        execution.progress(),
                    ));
                }
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
            let resumed_progress = self
                .sprite_main_execution
                .map(|execution| execution.progress());
            retire_resumed_main_loop_interruption(receipts, resumed_progress)?;
            if let Some(execution) = self.sprite_main_execution.as_mut() {
                execution.single_small_draw_position_slot = None;
                execution.wallmaster_reset_prefix_slot = None;
                execution.wallmaster_reset_cleared_bytes = None;
                execution.zazak_graphics_slot = None;
            }
        }
        if let Some(execution) = self.sprite_main_execution.as_mut() {
            execution.observe_mini_moldorm_history(&event)?;
        }
        match event.event.as_str() {
            "pc" => {
                let pc = event.pc.ok_or("Snes9x PC receipt omitted PC")? & 0x00ff_ffff;
                if pc == 0x02_824d
                    && (event.main, event.sub) == (Some(5), Some(0))
                    && event.return_address.map(|pc| pc & 0xffffff) == Some(0x00_8059)
                {
                    // Module05 tail-enters PreDungeon under the game-loop
                    // dispatcher. Starting-point selection also sets main=5,
                    // but calls this body from its own $02:85AD return.
                    receipts.push(OriginalTimingSemanticReceipt::SelectedGameEntranceReturned);
                }
                if pc == DUNGEON_RESET_SPRITES_RETURN_PC {
                    self.pending_reset_progress = None;
                    self.last_host_return_reset_progress = None;
                    self.cache_write_progress = None;
                    self.normal_load_ordinal = None;
                }
                if pc == SAVE_QUIT_RESET_DUNGEON_INFO_CLEAR_ENTRY_PC {
                    if (event.main, event.sub, event.subsub) != (Some(0), Some(10), Some(10)) {
                        return Err(format!(
                            "Snes9x save-quit reset prefix returned with unexpected module state {:?}/{:?}/{:?}",
                            event.main, event.sub, event.subsub,
                        ));
                    }
                    receipts.retain(|receipt| {
                        *receipt != OriginalTimingSemanticReceipt::SaveQuitIntroMemoryReturned
                    });
                    receipts.push(OriginalTimingSemanticReceipt::SaveQuitResetStatePublished);
                }
                if pc == FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_RETURN_PC
                    && (event.main, event.sub) == (Some(1), Some(1))
                {
                    receipts.retain(|receipt| {
                        !matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramClearProgress(
                                _
                            )
                        )
                    });
                    receipts.push(OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramCleared);
                }
                if pc == SELECTED_GAME_LOAD_MESSAGE_INTERFACE_RETURN_PC
                    && event.return_address.map(|pc| pc & 0x00ff_ffff)
                        == Some(MODULE05_AFTER_SHOW_TEXT_MESSAGE_PC)
                    && (event.main, event.sub) == (Some(14), Some(2))
                {
                    receipts.push(
                        OriginalTimingSemanticReceipt::SelectedGameLoadMessageInterfacePublished,
                    );
                }
                if pc == RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC
                    && (event.main, event.sub, event.subsub) == (Some(7), Some(0x18), Some(10))
                {
                    if self.rescued_maiden_initialization.is_some() {
                        return Err(
                            "Snes9x re-entered rescued-maiden follower graphics before its prior call returned"
                                .to_string(),
                        );
                    }
                    self.rescued_maiden_initialization =
                        Some(RescuedMaidenInitializationTracker::first_sheet());
                }
                let sprite_follower_graphics_caller =
                    match event.return_address.map(|pc| pc & 0x00ff_ffff) {
                        Some(SPRITE_PREP_BLIND_MAIDEN_FOLLOWER_GRAPHICS_RETURN_PC) => {
                            Some(SpriteFollowerGraphicsCaller::BlindMaiden)
                        }
                        Some(SPRITE_PREP_ZELDA_FOLLOWER_GRAPHICS_RETURN_PC) => {
                            Some(SpriteFollowerGraphicsCaller::Zelda)
                        }
                        Some(SPRITE_BLIND_MAIDEN_BODY_FOLLOWER_GRAPHICS_RETURN_PC) => {
                            Some(SpriteFollowerGraphicsCaller::BlindMaidenBody)
                        }
                        Some(SPRITE_PREP_OLD_MAN_FOLLOWER_GRAPHICS_RETURN_PC) => {
                            Some(SpriteFollowerGraphicsCaller::OldMan)
                        }
                        Some(SPRITE_PURPLE_CHEST_FOLLOWER_GRAPHICS_RETURN_PC) => {
                            Some(SpriteFollowerGraphicsCaller::PurpleChest)
                        }
                        _ => None,
                    };
                if pc == RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC
                    && sprite_follower_graphics_caller.is_some()
                {
                    let caller = sprite_follower_graphics_caller
                        .expect("checked sprite follower-graphics caller disappeared");
                    let execution = self
                        .sprite_main_execution
                        .as_mut()
                        .ok_or("Snes9x entered sprite follower graphics outside Sprite_Main")?;
                    let slot = execution
                        .current_slot
                        .ok_or("Snes9x entered follower graphics before a Sprite_Main slot")?;
                    if execution.follower_graphics.is_some() {
                        return Err(format!(
                            "Snes9x re-entered {caller:?} follower graphics in slot {slot} before its prior call returned",
                        ));
                    }
                    execution.follower_graphics =
                        Some((caller, RescuedMaidenInitializationTracker::first_sheet()));
                }
                if pc == RESCUED_MAIDEN_FIRST_FOLLOWER_SHEET_ENTRY_PC
                    && (self.rescued_maiden_initialization.is_some()
                        || self
                            .sprite_main_execution
                            .as_ref()
                            .is_some_and(|execution| execution.follower_graphics.is_some()))
                {
                    let purple_chest =
                        self.sprite_main_execution
                            .as_ref()
                            .is_some_and(|execution| {
                                execution
                                    .follower_graphics
                                    .as_ref()
                                    .is_some_and(|(caller, _)| {
                                        *caller == SpriteFollowerGraphicsCaller::PurpleChest
                                    })
                            });
                    let valid_sheet = if purple_chest {
                        event.y == Some(0x58)
                    } else {
                        matches!(event.y, Some(0x64 | 0x66))
                    };
                    if !valid_sheet {
                        return Err(format!(
                            "Snes9x rescued-maiden first follower sheet used unexpected asset {:?}",
                            event.y,
                        ));
                    }
                }
                if pc == RESCUED_MAIDEN_SECOND_FOLLOWER_SHEET_ENTRY_PC {
                    if let Some(tracker) = self.rescued_maiden_initialization.as_mut() {
                        if event.y != Some(0x65) {
                            return Err(format!(
                                "Snes9x rescued-maiden second follower sheet used unexpected asset {:?}",
                                event.y,
                            ));
                        }
                        tracker.begin_second_sheet()?;
                    }
                    if let Some(tracker) = self
                        .sprite_main_execution
                        .as_mut()
                        .and_then(|execution| execution.follower_graphics.as_mut())
                    {
                        if event.y != Some(0x65) {
                            return Err(format!(
                                "Snes9x Zelda's second follower sheet used unexpected asset {:?}",
                                event.y,
                            ));
                        }
                        tracker.1.begin_second_sheet()?;
                    }
                }
                if pc == RESCUED_MAIDEN_FOLLOWER_SHEETS_RETURN_PC {
                    if let Some(tracker) = self.rescued_maiden_initialization.as_mut() {
                        tracker.begin_conversion()?;
                    }
                    if let Some(tracker) = self
                        .sprite_main_execution
                        .as_mut()
                        .and_then(|execution| execution.follower_graphics.as_mut())
                    {
                        tracker.1.begin_conversion()?;
                    }
                }
                if pc == POLYHEDRAL_RENDER_START_PC
                    && matches!(event.main, Some(0x07 | 0x0e | 0x19))
                {
                    receipts.push(OriginalTimingSemanticReceipt::PreemptivePolyhedralRenderStarted);
                }
                if pc == OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC
                    && matches!(
                        event.return_address.map(|pc| pc & 0x00ff_ffff),
                        Some(
                            OVERWORLD_LOAD_OVERLAYS_AFTER_SPRITE_RELOAD_PC
                                | BIRD_TRAVEL_AFTER_SPRITE_RELOAD_PC
                                | MIRROR_WARP_AFTER_SPRITE_RELOAD_PC
                                | PRE_OVERWORLD_AFTER_SPRITE_RELOAD_PC
                        )
                    )
                {
                    if self.overworld_load_overlays_sprite_reload_active {
                        return Err(
                            "Snes9x re-entered Overworld_LoadOverlays sprite reload before its prior call returned"
                                .to_string(),
                        );
                    }
                    self.overworld_load_overlays_sprite_reload_active = true;
                    self.overworld_sprite_reload_reset_published = false;
                    self.overworld_presence_published = false;
                    self.overworld_sprite_activation = None;
                }
                if pc == OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_RETURN_PC
                    && self.overworld_load_overlays_sprite_reload_active
                {
                    self.overworld_load_overlays_sprite_reload_active = false;
                    self.overworld_sprite_reload_reset_published = false;
                    if !matches!((event.main, event.sub), (Some(8), Some(0))) {
                        receipts.push(
                            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                                OverworldSpriteReloadProgress::GenerationReturned,
                            ),
                        );
                    }
                }
                if pc == DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC
                    && event.return_address.map(|pc| pc & 0x00ff_ffff)
                        == Some(ZORA_FLIPPERS_GRAPHICS_RETURN_ADDRESS)
                {
                    let execution = self
                        .sprite_main_execution
                        .as_mut()
                        .ok_or("Snes9x entered King Zora flippers graphics outside Sprite_Main")?;
                    let slot = execution
                        .current_slot
                        .ok_or("Snes9x entered King Zora flippers graphics before a sprite slot")?;
                    execution.king_zora_flippers_graphics_slot = Some(slot);
                }
                if pc == DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC
                    && event.return_address.map(|pc| pc & 0x00ff_ffff)
                        == Some(BONK_ITEM_GRAPHICS_RETURN_ADDRESS)
                {
                    let execution = self
                        .sprite_main_execution
                        .as_mut()
                        .ok_or("Snes9x entered bonk-item graphics outside Sprite_Main")?;
                    let slot = execution
                        .current_slot
                        .ok_or("Snes9x entered bonk-item graphics before a sprite slot")?;
                    execution.bonk_item_graphics_slot = Some(slot);
                }
                if pc == DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC
                    && event.return_address.map(|pc| pc & 0x00ff_ffff)
                        == Some(WISH_POND_TOSSED_ITEM_GRAPHICS_RETURN_ADDRESS)
                {
                    let execution = self.sprite_main_execution.as_mut().ok_or(
                        "Snes9x entered Wish Pond tossed-item graphics outside Sprite_Main",
                    )?;
                    let slot = execution.current_slot.ok_or(
                        "Snes9x entered Wish Pond tossed-item graphics before a sprite slot",
                    )?;
                    execution.wish_pond_tossed_item_graphics_slot = Some(slot);
                }
                match pc {
                    0x06_d051 => {
                        if let Some(execution) = self.sprite_main_execution.as_mut() {
                            if execution.current_slot.map(u16::from) != event.x {
                                return Err(
                                    "absorbable body entry disagrees with its current slot".into(),
                                );
                            }
                            execution.absorbable_body_active = true;
                        }
                    }
                    SPRITE_MAIN_ENTRY_PC => {
                        if let Some(tracker) = self.rescued_maiden_initialization.take() {
                            if tracker.phase != RescuedMaidenInitializationTrackerPhase::Converting
                            {
                                return Err(format!(
                                    "Snes9x rescued-maiden caller entered Sprite_Main from {:?}",
                                    tracker.phase,
                                ));
                            }
                        }
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
                            execution.timers_and_oam_slot = None;
                            execution.timers_and_oam_dispatch_state = None;
                            execution.initialize_active_main_calls = 0;
                            execution.guard_prep_parry_hitbox = None;
                            execution.guard_prep_patrol_delay = None;
                            execution.guard_prep_tile_collision_return = None;
                            execution.guard_animation_checkpoint = None;
                            execution.hog_spear_body_graphics_pending = None;
                            execution.absorbable_body_active = false;
                            execution.absorbable_horizontal_lookup = None;
                            execution.absorbable_vertical_lookup = None;
                            execution.absorbable_vertical_attribute_loaded = None;
                            execution.dispatch_trampoline_return = None;
                            execution.vitreous_minions_seen = false;
                            execution.vitreous_player_damage_pending = None;
                            execution.vitreous_ai_pending = None;
                            execution.vitreous_damage_pending = None;
                            execution.swamola_segment = None;
                            execution.swamola_head_prepared = false;
                            execution.swamola_head_draw_completed = None;
                            execution.swamola_head_draw = None;
                            execution.swamola_segment_draw = None;
                            execution.pengator_slide_pending = None;
                            execution.antifairy_bounce_pending = None;
                            execution.kholdstare_subtype_decremented = false;
                            execution.kholdstare_damage_pending = None;
                            execution.initialize_prep_pending = None;
                            execution.guard_animation_pose_slot = None;
                            execution.guard_prep_weapon_flags_pending_slot = None;
                            execution.mini_moldorm_history = None;
                            execution.initialize_reset_properties = None;
                            execution.initialize_load_properties = None;
                            execution.fire_debirando_property_reload = false;
                            execution.fire_debirando_before_spawn_slot = None;
                            execution.fire_debirando_spawn = None;
                            execution.antfairy_subtype2_increment_slot = None;
                            execution.lanmola_subtype2_increment_slot = None;
                            execution.helmasaur_hard_hat_beetle_subtype2_increment_slot = None;
                            execution.timer_decrements_slot = None;
                            execution.primary_timer_decrements_slot = None;
                            execution.main_timer_decrement_slot = None;
                            execution.zero_hit_timer_clear_slot = None;
                            execution.main_and_aux1_timer_decrements_slot = None;
                            execution.hit_timer_slot = None;
                            execution.bari_before_random_slot = None;
                            execution.throwable_scenery_state_clear_slot = None;
                            execution.cucco_subtype_increments = None;
                            execution.cucco_animation_slot = None;
                            execution.cucco_flee_movement = None;
                            execution.active_cucco_movement = None;
                            execution.active_cucco_x_publications = 0;
                            execution.active_cucco_y_subpixel = None;
                            execution.master_sword_light_beam_movement = None;
                            execution.master_sword_light_beam_spawn = None;
                            execution.cucco_helper_ordinal = 0;
                            execution.big_key_drop_graphics_slot = None;
                            execution.king_zora_flippers_graphics_slot = None;
                            execution.bonk_item_graphics_slot = None;
                            execution.single_small_draw_position_slot = None;
                            execution.probe_after_oam_coordinates_slot = None;
                            execution.wallmaster_reset_prefix_slot = None;
                            execution.wallmaster_reset_cleared_bytes = None;
                            execution.zazak_graphics_slot = None;
                            execution.follower_graphics = None;
                        }
                    }
                    SPRITE_ACTIVE_MAIN_ENTRY_PC => {
                        if let Some(execution) = self.sprite_main_execution.as_mut() {
                            execution.guard_prep_parry_hitbox = None;
                            execution.guard_prep_patrol_delay = None;
                            execution.guard_prep_tile_collision_return = None;
                            execution.guard_animation_checkpoint = None;
                            execution.hog_spear_body_graphics_pending = None;
                            execution.absorbable_body_active = false;
                            execution.absorbable_horizontal_lookup = None;
                            execution.absorbable_vertical_lookup = None;
                            execution.absorbable_vertical_attribute_loaded = None;
                            execution.dispatch_trampoline_return = None;
                            execution.vitreous_minions_seen = false;
                            execution.vitreous_player_damage_pending = None;
                            execution.vitreous_ai_pending = None;
                            execution.vitreous_damage_pending = None;
                            execution.swamola_segment = None;
                            execution.swamola_head_prepared = false;
                            execution.swamola_head_draw_completed = None;
                            execution.swamola_head_draw = None;
                            execution.swamola_segment_draw = None;
                            execution.pengator_slide_pending = None;
                            execution.antifairy_bounce_pending = None;
                            execution.kholdstare_subtype_decremented = false;
                            execution.kholdstare_damage_pending = None;
                            execution.initialize_prep_pending = None;
                            execution.guard_animation_pose_slot = None;
                            if execution.timers_and_oam_dispatch_state == Some(8) {
                                execution.initialize_active_main_calls = execution
                                    .initialize_active_main_calls
                                    .checked_add(1)
                                    .ok_or(
                                        "Snes9x state-8 initializer active-call count overflowed",
                                    )?;
                            }
                        }
                    }
                    SPRITE_TIMERS_AND_OAM_RETURN_PC => {
                        if let Some(execution) = self.sprite_main_execution.as_mut() {
                            execution.observe_bari_before_random(&event)?;
                            execution.observe_timers_and_oam_return(&event)?;
                        }
                    }
                    SPRITE_TIMER_DECREMENTS_TRACE_PC => {
                        if let Some(execution) = self.sprite_main_execution.as_mut() {
                            execution.observe_timer_decrements(&event)?;
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
                        execution.timers_and_oam_slot = None;
                        execution.timers_and_oam_dispatch_state = None;
                        execution.initialize_active_main_calls = 0;
                        execution.guard_prep_parry_hitbox = None;
                        execution.guard_prep_patrol_delay = None;
                        execution.guard_prep_tile_collision_return = None;
                        execution.guard_animation_checkpoint = None;
                        execution.hog_spear_body_graphics_pending = None;
                        execution.absorbable_body_active = false;
                        execution.absorbable_horizontal_lookup = None;
                        execution.absorbable_vertical_lookup = None;
                        execution.absorbable_vertical_attribute_loaded = None;
                        execution.dispatch_trampoline_return = None;
                        execution.vitreous_minions_seen = false;
                        execution.vitreous_player_damage_pending = None;
                        execution.vitreous_ai_pending = None;
                        execution.vitreous_damage_pending = None;
                        execution.swamola_segment = None;
                        execution.swamola_head_prepared = false;
                        execution.swamola_head_draw_completed = None;
                        execution.swamola_head_draw = None;
                        execution.swamola_segment_draw = None;
                        execution.pengator_slide_pending = None;
                        execution.antifairy_bounce_pending = None;
                        execution.kholdstare_subtype_decremented = false;
                        execution.kholdstare_damage_pending = None;
                        execution.initialize_prep_pending = None;
                        execution.guard_animation_pose_slot = None;
                        execution.guard_prep_weapon_flags_pending_slot = None;
                        execution.mini_moldorm_history = None;
                        execution.initialize_reset_properties = None;
                        execution.initialize_load_properties = None;
                        execution.fire_debirando_property_reload = false;
                        execution.fire_debirando_before_spawn_slot = None;
                        execution.fire_debirando_spawn = None;
                        execution.antfairy_subtype2_increment_slot = None;
                        execution.lanmola_subtype2_increment_slot = None;
                        execution.helmasaur_hard_hat_beetle_subtype2_increment_slot = None;
                        execution.timer_decrements_slot = None;
                        execution.primary_timer_decrements_slot = None;
                        execution.main_timer_decrement_slot = None;
                        execution.zero_hit_timer_clear_slot = None;
                        execution.main_and_aux1_timer_decrements_slot = None;
                        execution.hit_timer_slot = None;
                        execution.bari_before_random_slot = None;
                        execution.throwable_scenery_state_clear_slot = None;
                        execution.cucco_subtype_increments = None;
                        execution.cucco_animation_slot = None;
                        execution.cucco_flee_movement = None;
                        execution.active_cucco_movement = None;
                        execution.active_cucco_x_publications = 0;
                        execution.active_cucco_y_subpixel = None;
                        execution.master_sword_light_beam_movement = None;
                        execution.master_sword_light_beam_spawn = None;
                        execution.cucco_helper_ordinal = 0;
                        execution.big_key_drop_graphics_slot = None;
                        execution.king_zora_flippers_graphics_slot = None;
                        execution.bonk_item_graphics_slot = None;
                        execution.single_small_draw_position_slot = None;
                        execution.probe_after_oam_coordinates_slot = None;
                        execution.wallmaster_reset_prefix_slot = None;
                        execution.wallmaster_reset_cleared_bytes = None;
                        execution.zazak_graphics_slot = None;
                        if let Some((caller, tracker)) = execution.follower_graphics.take() {
                            if tracker.phase != RescuedMaidenInitializationTrackerPhase::Converting
                            {
                                return Err(format!(
                                    "Snes9x {caller:?} sprite slot {slot} returned from follower graphics in {:?}",
                                    tracker.phase,
                                ));
                            }
                        }
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
                    MASTER_SWORD_LIGHT_BEAM_MOVEMENT_CALL_PC => {
                        let execution = self.sprite_main_execution.as_mut().ok_or(
                            "Snes9x entered master-sword light-beam movement outside Sprite_Main",
                        )?;
                        let slot = execution.current_slot.ok_or(
                            "Snes9x entered master-sword light-beam movement before a sprite slot",
                        )?;
                        if event.x != Some(u16::from(slot)) {
                            return Err(format!(
                                "Snes9x master-sword light-beam movement disagreed on slot {slot}: x={:?}",
                                event.x,
                            ));
                        }
                        if execution.master_sword_light_beam_movement.is_some() {
                            return Err(
                                "Snes9x restarted master-sword light-beam movement before its slot returned"
                                    .to_string(),
                            );
                        }
                        execution.master_sword_light_beam_movement = Some((slot, 0));
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
                if pc == 0x1d_e5dd {
                    let execution = self
                        .sprite_main_execution
                        .as_mut()
                        .ok_or("Vitreous minion cadence outside Sprite_Main")?;
                    let slot = execution
                        .current_slot
                        .ok_or("Vitreous minion cadence lost slot")?;
                    if event.x != Some(u16::from(slot)) || address != 0x0e80 + u16::from(slot) {
                        return Err("Vitreous cadence disagrees with source slot".into());
                    }
                    execution.vitreous_minions_seen = true;
                }
                if pc == 0x1d_9f88 {
                    let execution = self
                        .sprite_main_execution
                        .as_mut()
                        .ok_or("Swamola head flags outside Sprite_Main")?;
                    let slot = execution.current_slot.ok_or("Swamola head lost slot")?;
                    if event.x != Some(u16::from(slot)) || address != 0x0f50 + u16::from(slot) {
                        return Err("Swamola head flags disagree with source slot".into());
                    }
                    execution.swamola_head_prepared = true;
                }
                if address == 0x0fb6 && matches!(pc, 0x1d_9fd7 | 0x1d_a034) {
                    let execution = self
                        .sprite_main_execution
                        .as_mut()
                        .ok_or("Swamola segment publication outside Sprite_Main")?;
                    if execution.current_slot.map(u16::from) != event.x {
                        return Err("Swamola segment slot disagrees with the active caller".into());
                    }
                    let segment = event
                        .value
                        .ok_or("Swamola segment publication omitted its value")?;
                    if segment > 4 {
                        return Err("Swamola segment exceeds its four-part body".into());
                    }
                    execution.swamola_segment = Some(segment);
                    execution.swamola_head_prepared = false;
                    execution.swamola_head_draw_completed = None;
                    execution.swamola_head_draw = None;
                    execution.swamola_segment_draw = None;
                }
                if pc == CREDITS_SCENE_OVERWORLD_SUBSUBMODULE_INCREMENT_PC
                    && address == SUBSUBMODULE_INDEX
                    && event.main == Some(0x1a)
                {
                    receipts.retain(|receipt| {
                        !matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(_)
                        )
                    });
                    receipts.push(OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(
                        CreditsSceneLoadProgressReceipt {
                            progress: CreditsSceneLoadProgress::SceneLoadCompleted,
                            boundary: OriginalTimingBoundary::HostReturn,
                        },
                    ));
                }
                if let Some(progress) = dungeon_falling_entrance_progress(&event, pc, address)? {
                    receipts.push(
                        OriginalTimingSemanticReceipt::DungeonFallingEntranceProgress(progress),
                    );
                }
                if pc == 0x0c_c25b
                    && address == 0x11
                    && event.return_address == Some(0x0c_f0e8)
                    && event.main == Some(0x17)
                {
                    if event.sub != Some(1) || event.value != Some(2) {
                        return Err(
                            "save-quit intro-memory return has the wrong submodule advance".into(),
                        );
                    }
                    receipts.push(OriginalTimingSemanticReceipt::SaveQuitIntroMemoryReturned);
                }
                if let Some(execution) = self.sprite_main_execution.as_mut() {
                    execution.observe_guard_prep_weapon_flags_pending(&event)?;
                    execution.observe_guard_animation_checkpoint(&event)?;
                    execution.observe_hog_spear_body_graphics_pending(&event)?;
                    execution.observe_absorbable_tile_lookup(&event)?;
                    execution.observe_swamola_segment_draw(&event)?;
                    execution.observe_vitreous_damage_pending(&event);
                    execution.observe_dispatch_trampoline_return(&event)?;
                    execution.observe_pengator_slide_pending(&event)?;
                    execution.observe_antifairy_bounce_pending(&event)?;
                    execution.observe_kholdstare_damage_pending(&event)?;
                    execution.observe_fire_debirando_spawn_write(&event)?;
                    execution.observe_master_sword_light_beam_spawn_write(&event)?;
                    execution.observe_antfairy_subtype2_increment(&event)?;
                    execution.observe_lanmola_subtype2_increment(&event)?;
                    execution.observe_helmasaur_hard_hat_beetle_subtype2_increment(&event)?;
                    execution.observe_zazak_graphics(&event)?;
                    if pc == THROWABLE_SCENERY_STATE_CLEAR_PC {
                        let slot = execution.current_slot.ok_or(
                            "Snes9x cleared throwable scenery before entering a sprite slot",
                        )?;
                        let value = event
                            .value
                            .ok_or("Snes9x throwable-scenery state clear omitted value")?;
                        if event.x != Some(u16::from(slot))
                            || address != SPRITE_STATE_BASE + u16::from(slot)
                            || value != 0
                        {
                            return Err(format!(
                                "Snes9x throwable-scenery state clear disagreed on slot {slot}: x={:?}, address=${address:04x}, value=${value:02x}",
                                event.x,
                            ));
                        }
                        execution.throwable_scenery_state_clear_slot = Some(slot);
                    }
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
                    if let Some((slot, checkpoint_ordinal)) =
                        execution.master_sword_light_beam_movement.as_mut()
                    {
                        let slot = *slot;
                        let expected = [
                            SPRITE_X_SUBPIXEL_BASE,
                            SPRITE_X_LOW_BASE,
                            SPRITE_X_HIGH_BASE,
                            SPRITE_Y_SUBPIXEL_BASE,
                            SPRITE_Y_LOW_BASE,
                            SPRITE_Y_HIGH_BASE,
                        ];
                        let movement_addresses = expected.map(|base| base + u16::from(slot));
                        if let Some(index) = movement_addresses
                            .iter()
                            .position(|&candidate| candidate == address)
                        {
                            // `Sprite_MoveX` and `Sprite_MoveY` each return
                            // without publishing any coordinate assignment
                            // when that axis' velocity is zero. The first
                            // observed Y store can therefore legitimately
                            // follow the call site with no X stores. Preserve
                            // the source checkpoint reached, rather than
                            // counting only writes that happened to execute.
                            let next_ordinal = match (*checkpoint_ordinal, index) {
                                (0, 0) => 1,
                                (1, 1) => 2,
                                (2, 2) => 3,
                                (0 | 3, 3) => 4,
                                (4, 4) => 5,
                                (5, 5) => 6,
                                _ => {
                                    return Err(format!(
                                        "Snes9x master-sword light-beam movement stores were out of source order: checkpoint={} address=${address:04x}",
                                        *checkpoint_ordinal,
                                    ));
                                }
                            };
                            *checkpoint_ordinal = next_ordinal;
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
                if pc == SPRITE_PREP_FIRE_DEBIRANDO_TYPE_STORE_PC {
                    let execution = self
                        .sprite_main_execution
                        .as_mut()
                        .ok_or("Snes9x converted Fire Debirando outside Sprite_Main")?;
                    let slot = execution
                        .current_slot
                        .ok_or("Snes9x converted Fire Debirando before entering a slot")?;
                    if execution.timers_and_oam_dispatch_state != Some(8)
                        || event.x != Some(u16::from(slot))
                        || address != SPRITE_TYPE_BASE + u16::from(slot)
                        || event.value != Some(0x63)
                    {
                        return Err(format!(
                            "Snes9x Fire Debirando conversion disagreed on slot {slot}: dispatch={:?}, x={:?}, address=${address:04x}, value={:?}",
                            execution.timers_and_oam_dispatch_state,
                            event.x,
                            event.value,
                        ));
                    }
                    execution.fire_debirando_property_reload = true;
                    // Any checkpoint from the first property reset is now
                    // superseded by the later source call.
                    execution.initialize_reset_properties = None;
                    execution.initialize_load_properties = None;
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
                if pc == ANTFAIRY_SUBTYPE2_INCREMENT_PC {
                    if let Some(progress) = self.cached_sprite_execution.as_mut() {
                        if progress.restore_started
                            || usize::from(progress.copied_fields)
                                != CACHED_SPRITE_LIVE_FIELDS.len()
                            || event.x != Some(u16::from(progress.slot))
                            || address != SPRITE_SUBTYPE2_BASE + u16::from(progress.slot)
                        {
                            return Err(format!(
                                "Snes9x cached Antfairy subtype publication disagreed with the live-slot swap: tracker={progress:?}, x={:?}, address=${address:04x}",
                                event.x,
                            ));
                        }
                        if progress
                            .body_progress
                            .replace(
                                CachedSpriteExecutionBodyProgress::AfterAntfairySubtype2Increment,
                            )
                            .is_some()
                        {
                            return Err(
                                "Snes9x cached Antfairy published its subtype increment twice"
                                    .to_string(),
                            );
                        }
                    }
                }
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
                    self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::Load(
                        DungeonLoadSpritesCpuProgress {
                            normal_load_ordinal: self.normal_load_ordinal.unwrap(),
                            slot,
                            checkpoint: DungeonSpriteLoadCheckpoint::State,
                        },
                    ));
                } else if (DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC..DUNGEON_LOAD_SINGLE_SPRITE_END_PC)
                    .contains(&pc)
                {
                    let Some((slot, checkpoint)) =
                        dungeon_load_single_sprite_write_progress(pc, address, event.x)?
                    else {
                        return Err(format!(
                            "Snes9x Dungeon_LoadSingleSprite wrote unsupported source field ${address:04x} at ${pc:06x}",
                        ));
                    };
                    let normal_load_ordinal = self.normal_load_ordinal.ok_or(
                        "Snes9x observed Dungeon_LoadSingleSprite field before record state",
                    )?;
                    self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::Load(
                        DungeonLoadSpritesCpuProgress {
                            normal_load_ordinal,
                            slot,
                            checkpoint,
                        },
                    ));
                }
            }
            "nmi" => {
                if event.pc == Some(0x02_dc76)
                    && matches!((event.main, event.sub), (Some(5), Some(0)))
                {
                    receipts
                        .push(OriginalTimingSemanticReceipt::SelectedGameEntranceScrollPublished);
                }
                if let Some(progress) = credits_scene_load_boundary_progress(
                    &event,
                    OriginalTimingBoundary::NmiAccepted,
                )? {
                    receipts.retain(|receipt| {
                        !matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(_)
                        )
                    });
                    receipts.push(OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(
                        progress,
                    ));
                }
                if let Some(progress) = credits_end_sequence_32_boundary_progress(
                    &event,
                    OriginalTimingBoundary::NmiAccepted,
                )? {
                    receipts.retain(|receipt| {
                        !matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::CreditsEndSequence32Progress(_)
                        )
                    });
                    receipts.push(OriginalTimingSemanticReceipt::CreditsEndSequence32Progress(
                        progress,
                    ));
                }
                if let Some(progress) = triforce_room_case2_palette_progress(
                    &event,
                    OriginalTimingBoundary::NmiAccepted,
                )? {
                    receipts.retain(|receipt| {
                        !matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::TriforceRoomCase2PaletteProgress(_)
                        )
                    });
                    receipts.push(
                        OriginalTimingSemanticReceipt::TriforceRoomCase2PaletteProgress(progress),
                    );
                }
                if let Some(progress) = dungeon_peg_attribute_flip_progress(
                    &event,
                    OriginalTimingBoundary::NmiAccepted,
                )? {
                    receipts.retain(|receipt| {
                        !matches!(
                            receipt,
                            OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(_)
                        )
                    });
                    receipts.push(
                        OriginalTimingSemanticReceipt::DungeonPegAttributeFlipProgress(progress),
                    );
                }
                if let Some(tracker) = self.rescued_maiden_initialization.as_mut() {
                    if (event.main, event.sub, event.subsub) != (Some(7), Some(0x18), Some(10)) {
                        return Err(format!(
                            "Snes9x rescued-maiden decompressor escaped its source domain: main={:?}, sub={:?}, subsub={:?}",
                            event.main, event.sub, event.subsub,
                        ));
                    }
                    tracker.observe_boundary(&event)?;
                }
                if let Some(execution) = self.sprite_main_execution.as_mut() {
                    if let Some((_, tracker)) = execution.follower_graphics.as_mut() {
                        tracker.observe_boundary(&event)?;
                    }
                }
                if let Some(execution) = self.sprite_main_execution.as_mut() {
                    execution.observe_guard_prep_weapon_flags_pending(&event)?;
                    execution.observe_guard_animation_checkpoint(&event)?;
                    execution.observe_hog_spear_body_graphics_pending(&event)?;
                    execution.observe_absorbable_tile_lookup(&event)?;
                    execution.observe_swamola_segment_draw(&event)?;
                    execution.observe_vitreous_damage_pending(&event);
                    execution.observe_dispatch_trampoline_return(&event)?;
                    execution.observe_pengator_slide_pending(&event)?;
                    execution.observe_antifairy_bounce_pending(&event)?;
                    execution.observe_kholdstare_damage_pending(&event)?;
                    execution.observe_fire_debirando_spawn_boundary(&event)?;
                    execution.observe_guard_prep_parry_hitbox(&event)?;
                    execution.observe_guard_prep_patrol_delay(&event)?;
                    execution.observe_guard_prep_tile_collision_return(&event)?;
                    execution.observe_bari_before_random(&event)?;
                    execution.observe_main_and_aux1_timer_decrements(&event)?;
                    execution.observe_main_timer_decrement(&event)?;
                    execution.observe_zero_hit_timer_clear(&event)?;
                    execution.observe_primary_timer_decrements(&event)?;
                    execution.observe_hit_timer(&event)?;
                    execution.observe_timer_decrements(&event)?;
                    execution.observe_single_small_draw_position(&event)?;
                    execution.observe_probe_after_oam_coordinates(&event)?;
                    execution.observe_initialize_reset_properties(&event)?;
                    execution.observe_initialize_load_properties(&event)?;
                    execution.observe_initialize_prep_pending(&event)?;
                    execution.observe_fire_debirando_before_spawn(&event)?;
                    execution.observe_zazak_graphics(&event)?;
                    execution.observe_wallmaster_reset_prefix(&event)?;
                }
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
                    self.overworld_load_overlays_sprite_reload_active
                        && !self.overworld_sprite_reload_reset_published,
                    receipts,
                )? {
                    // `Sprite_DisableAll` is shared by `Sprite_ResetAll` and
                    // `Dungeon_ResetSprites`. The interrupted PC and the
                    // innermost source return address prove this execution is
                    // the former, so the generic reset candidate must not
                    // escape into the wrong semantic domain below.
                    self.pending_reset_progress = None;
                    if self.overworld_load_overlays_sprite_reload_active {
                        self.overworld_sprite_reload_reset_published = true;
                    }
                } else if let Some(progress) = dungeon_reset_sprites_caller_progress(&event) {
                    self.pending_reset_progress = Some(progress);
                }
                let overworld_sprite_scan_suspended =
                    self.publish_overworld_presence_at_scan_boundary(&event, receipts);
                if overworld_sprite_scan_suspended {
                    receipts.push(
                        OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                            OverworldSpriteReloadProgress::ProximityScanSuspended {
                                bg2_h: event.bg2_h.ok_or(
                                    "Snes9x overworld scan NMI omitted the BG2 scratch coordinate",
                                )?,
                            },
                        ),
                    );
                }
                self.flush_host_boundary_progress(receipts, OriginalTimingBoundary::NmiAccepted);
                receipts.push(OriginalTimingSemanticReceipt::NmiAccepted(update_gate));
                if event.pc == Some(0x00_e9bc) && event.return_address == Some(0x02_8ea4) {
                    if (event.main, event.sub, event.subsub) != (Some(7), Some(7), Some(15)) {
                        return Err(
                            "falling fade-in palette checkpoint has the wrong source caller".into(),
                        );
                    }
                    receipts.push(
                        OriginalTimingSemanticReceipt::DungeonFallingFadeInPaletteDirectionToggled,
                    );
                }
                if let Some(progress) = rescued_maiden_tilemap_clear_progress(
                    &event,
                    OriginalTimingBoundary::NmiAccepted,
                )? {
                    receipts.push(
                        OriginalTimingSemanticReceipt::RescuedMaidenTilemapClearProgress(progress),
                    );
                }
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
                if let Some(progress) = event
                    .pc
                    .and_then(|pc| link_oam_stair_progress(pc, event.sub))
                {
                    receipts.push(OriginalTimingSemanticReceipt::LinkOamStairProgress(
                        progress,
                    ));
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

    fn publish_overworld_presence_at_scan_boundary(
        &mut self,
        event: &RawTraceEvent,
        receipts: &mut Vec<OriginalTimingSemanticReceipt>,
    ) -> bool {
        // Entering the proximity scan proves the preceding presence load
        // returned, whether this interval ends at NMI or at retro_run's
        // frame boundary. No sprite need have been activated yet.
        // Sprite_Main also calls the proximity helpers. Its active call owns
        // those statements; unchanged Module09/$04 bytes do not prove reload.
        let suspended = self.sprite_main_execution.is_none()
            && (self.overworld_load_overlays_sprite_reload_active
                || (event.main == Some(8) && event.sub == Some(0))
                || (event.main == Some(9) && matches!(event.sub, Some(4 | 0x12))))
            && event.pc.map(|pc| pc & 0x00ff_ffff).is_some_and(|pc| {
                (OVERWORLD_SPRITE_SCAN_START_PC..OVERWORLD_SPRITE_SCAN_END_PC).contains(&pc)
            });
        if suspended {
            self.publish_overworld_presence(receipts);
        }
        suspended
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
        let source_owns_reload_publication = self.overworld_load_overlays_sprite_reload_active
            || matches!((event.main, event.sub), (Some(8), Some(0)))
            || matches!((event.main, event.sub), (Some(9), Some(4 | 0x12)));
        if self.sprite_main_execution.is_some()
            || !source_owns_reload_publication
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
            if (event.main == Some(8) && event.sub == Some(0))
                || (event.main == Some(0x0e)
                    && event.sub == Some(0x0a)
                    && self.overworld_load_overlays_sprite_reload_active)
            {
                self.publish_overworld_presence(receipts);
            }
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
    source_overworld_reload_active: bool,
    receipts: &mut Vec<OriginalTimingSemanticReceipt>,
) -> Result<bool, String> {
    let pc = event.pc.map(|pc| pc & 0x00ff_ffff);
    let return_address = event.return_address.map(|address| address & 0x00ff_ffff);
    let pre_dungeon_caller = return_address == Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC);
    let bird_travel_caller = matches!((event.main, event.sub), (Some(0x0e), Some(0x0a)))
        && matches!(
            return_address,
            Some(BIRD_TRAVEL_AFTER_INITIAL_SPRITE_RESET_PC | SPRITE_RELOAD_AFTER_DISABLE_PC)
        );
    let pre_overworld_caller = source_overworld_reload_active
        && matches!((event.main, event.sub), (Some(8), Some(0)))
        && return_address == Some(SPRITE_RELOAD_AFTER_DISABLE_PC);
    // Death_Func15's JSL at $09:F588 has raw return $09:F58B.
    // Its death counters and save/continue branch follow Sprite_ResetAll.
    let game_over_caller = matches!((event.main, event.sub), (Some(0x12), Some(9)))
        && return_address == Some(0x09_f58b);
    if !pc.is_some_and(|pc| {
        (SPRITE_RESET_ALL_NO_DISABLE_START_PC..SPRITE_RESET_ALL_END_PC).contains(&pc)
    }) || !(pre_dungeon_caller || bird_travel_caller || pre_overworld_caller || game_over_caller)
    {
        return Ok(false);
    }
    // The return address is the source-owned caller proof. `Module_PreDungeon`
    // is both main-module 6's dispatch target and a direct callee of two
    // selected-game loaders. The bird-travel path has two consecutive reset
    // calls; its exact outer and inner return PCs distinguish their phases.
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
    let after_circle_value_before_upper_write = pc.is_some_and(|pc| {
        (IRIS_SPOTLIGHT_AFTER_CIRCLE_VALUE_START_PC..=IRIS_SPOTLIGHT_UPPER_TABLE_WRITE_PC)
            .contains(&pc)
    });
    let after_upper_table_write = pc.is_some_and(|pc| {
        (IRIS_SPOTLIGHT_AFTER_UPPER_TABLE_WRITE_START_PC..IRIS_SPOTLIGHT_LOWER_TABLE_WRITE_PC)
            .contains(&pc)
    });
    let before_loop_completion_test = pc.is_some_and(|pc| {
        (IRIS_SPOTLIGHT_BEFORE_LOOP_COMPLETION_TEST_START_PC
            ..=IRIS_SPOTLIGHT_UPPER_CURSOR_INCREMENT_PC)
            .contains(&pc)
    });
    let before_circle_iteration_prefix = pc.is_some_and(|pc| {
        (IRIS_SPOTLIGHT_ITERATION_VALUE_LOAD_PC..IRIS_SPOTLIGHT_CIRCLE_VALUE_CALL_PC).contains(&pc)
    });
    let before_projection_beam_wait =
        pc.is_some_and(|pc| IRIS_SPOTLIGHT_BEAM_WAIT_PCS.contains(&pc));
    if !inside_circle_value
        && !before_circle_iteration_prefix
        && !after_circle_value_before_upper_write
        && !after_upper_table_write
        && !before_loop_completion_test
        && !before_projection_beam_wait
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
    let iteration_initialization_checkpoint =
        before_circle_iteration_prefix || pc == Some(IRIS_SPOTLIGHT_NEXT_ITERATION_PC);
    let projection_checkpoint = !inside_circle_value
        && !after_upper_table_write
        && !before_loop_completion_test
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
        || after_circle_value_before_upper_write
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
            let pending_circle_input = if inside_circle_value
                || after_circle_value_before_upper_write
            {
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
    } else if after_upper_table_write {
        let lower_cursor = spotlight_lower_cursor
            .ok_or("Snes9x spotlight lower-table guard omitted the source lower cursor")?;
        let completed_iterations = initial_lower_cursor
            .checked_sub(lower_cursor)
            .ok_or("Snes9x spotlight lower-table guard exceeded its source initial cursor")?;
        (
            completed_iterations,
            SpotlightTableBuildCheckpoint::AfterUpperTableWrite { lower_cursor },
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
    } else if before_loop_completion_test || pc == Some(IRIS_SPOTLIGHT_LOWER_CURSOR_DECREMENT_PC) {
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
        // The same X can encode a visible row offset OR the circle helper's
        // quantized index on a later clipped row. Bind candidates to the
        // event-local r6 before interpreting X; never prefer a visible store
        // merely because its numeric offset matches (source host 428504).
        for completed_iterations in 0..total_iterations {
            let upper_cursor = initial_upper_cursor.wrapping_add(completed_iterations);
            let lower_cursor = initial_lower_cursor.wrapping_sub(completed_iterations);
            if spotlight_lower_cursor.is_some_and(|cursor| cursor != lower_cursor) {
                continue;
            }
            let retained_x = if lower_cursor < 224 {
                Some(lower_cursor * 2)
            } else if upper_cursor < 224 {
                Some(upper_cursor * 2)
            } else if radius != 0 && completed_iterations >= iterations_before_iris {
                let active_iterations = completed_iterations - iterations_before_iris;
                let pending_circle_input = radius.saturating_sub(active_iterations);
                Some(((u32::from(pending_circle_input) << 8) / u32::from(radius) >> 1) as u16)
            } else {
                None
            };
            if retained_x == Some(observed_x) {
                if matched
                    .replace((completed_iterations, upper_cursor, lower_cursor))
                    .is_some()
                {
                    return Err(format!(
                        "Snes9x spotlight loop-test X {observed_x} maps to multiple source iterations without a unique source cursor",
                    ));
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
            _ if before_projection_beam_wait => 0,
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
        (Some(0x0f), Some(0 | 1)) | (Some(0x10), Some(0 | 1)) | (Some(0x12), Some(2 | 3))
    )
}

fn zelda_main_wait_pc(pc: u32) -> bool {
    matches!(pc, 0x00_8034 | 0x00_8036)
}

fn dungeon_falling_entrance_progress(
    event: &RawTraceEvent,
    pc: u32,
    address: u16,
) -> Result<Option<DungeonFallingEntranceProgress>, String> {
    // `Dungeon_LoadAndDrawRoom` is shared with Module_PreDungeon. Its parser
    // tail executes the same subsubmodule clear at the same PC while main is
    // still $06; only Module11_02's call owns a falling-entrance continuation.
    // The later caller-local publications have unique PCs and remain
    // fail-closed below if their module state disagrees.
    if pc == FALLING_ENTRANCE_ROOM_PARSER_SUBSUB_CLEAR_PC && event.main != Some(0x11) {
        return Ok(None);
    }
    let (expected_address, expected_value, progress) = match pc {
        FALLING_ENTRANCE_ROOM_PARSER_SUBSUB_CLEAR_PC => (
            SUBSUBMODULE_INDEX,
            0,
            DungeonFallingEntranceProgress::RoomParserClearedSubsubmodule,
        ),
        FALLING_ENTRANCE_SUBSUB_ADVANCE_PC => (
            SUBSUBMODULE_INDEX,
            3,
            DungeonFallingEntranceProgress::RoomLoadAdvancedSubsubmodule,
        ),
        FALLING_ENTRANCE_SONG_BANK_TAIL_PC => (
            SUBMODULE_INDEX,
            7,
            DungeonFallingEntranceProgress::SongBankTailEntered,
        ),
        _ => return Ok(None),
    };
    let value = event
        .value
        .ok_or("Snes9x falling-entrance control publication omitted its value")?;
    if event.main != Some(0x11) || address != expected_address || value != expected_value {
        return Err(format!(
            "Snes9x falling-entrance publication {progress:?} disagreed with its source state: main={:?}, address=${address:04x}, value=${value:02x}",
            event.main,
        ));
    }
    Ok(Some(progress))
}

fn rescued_maiden_tilemap_clear_progress(
    event: &RawTraceEvent,
    boundary: OriginalTimingBoundary,
) -> Result<Option<RescuedMaidenTilemapClearProgressReceipt>, String> {
    let pc = event.pc.ok_or("Snes9x tilemap-clear boundary omitted PC")? & 0x00ff_ffff;
    let next_store = match pc {
        RESCUED_MAIDEN_TILEMAP_CLEAR_FIRST_STORE_PC => Some(0),
        RESCUED_MAIDEN_TILEMAP_CLEAR_SECOND_STORE_PC => Some(1),
        RESCUED_MAIDEN_TILEMAP_CLEAR_THIRD_STORE_PC => Some(2),
        RESCUED_MAIDEN_TILEMAP_CLEAR_FOURTH_STORE_PC => Some(3),
        RESCUED_MAIDEN_TILEMAP_CLEAR_FIFTH_STORE_PC => Some(4),
        RESCUED_MAIDEN_TILEMAP_CLEAR_SIXTH_STORE_PC => Some(5),
        RESCUED_MAIDEN_TILEMAP_CLEAR_SEVENTH_STORE_PC => Some(6),
        RESCUED_MAIDEN_TILEMAP_CLEAR_EIGHTH_STORE_PC => Some(7),
        RESCUED_MAIDEN_TILEMAP_CLEAR_FIRST_INX_PC
        | RESCUED_MAIDEN_TILEMAP_CLEAR_SECOND_INX_PC
        | RESCUED_MAIDEN_TILEMAP_CLEAR_COMPARE_PC
        | RESCUED_MAIDEN_TILEMAP_CLEAR_BRANCH_PC => None,
        _ => return Ok(None),
    };
    if (event.main, event.sub, event.subsub) != (Some(7), Some(0x18), Some(0)) {
        return Err(format!(
            "Snes9x rescued-maiden tilemap-clear PC escaped its source domain: main={:?}, sub={:?}, subsub={:?}",
            event.main, event.sub, event.subsub,
        ));
    }
    let x = event
        .x
        .ok_or("Snes9x rescued-maiden tilemap-clear NMI omitted X")?;
    let completed_stores = if let Some(next_store) = next_store {
        if x > 0x07fe || x & 1 != 0 {
            return Err(format!(
                "Snes9x rescued-maiden store checkpoint used invalid X=${x:04x}",
            ));
        }
        (x / 2)
            .checked_mul(8)
            .and_then(|stores| stores.checked_add(next_store))
            .ok_or("Snes9x rescued-maiden store checkpoint overflowed")?
    } else {
        match pc {
            RESCUED_MAIDEN_TILEMAP_CLEAR_FIRST_INX_PC => {
                if x > 0x07fe || x & 1 != 0 {
                    return Err(format!(
                        "Snes9x rescued-maiden first INX checkpoint used invalid X=${x:04x}",
                    ));
                }
                (x / 2 + 1) * 8
            }
            RESCUED_MAIDEN_TILEMAP_CLEAR_SECOND_INX_PC => {
                if x == 0 || x > 0x07ff || x & 1 == 0 {
                    return Err(format!(
                        "Snes9x rescued-maiden second INX checkpoint used invalid X=${x:04x}",
                    ));
                }
                ((x + 1) / 2) * 8
            }
            RESCUED_MAIDEN_TILEMAP_CLEAR_COMPARE_PC | RESCUED_MAIDEN_TILEMAP_CLEAR_BRANCH_PC => {
                if x > 0x0800 || x & 1 != 0 {
                    return Err(format!(
                        "Snes9x rescued-maiden loop-control checkpoint used invalid X=${x:04x}",
                    ));
                }
                (x / 2) * 8
            }
            _ => unreachable!("store checkpoints were handled above"),
        }
    };
    if completed_stores > 8192 {
        return Err(format!(
            "Snes9x rescued-maiden checkpoint exceeded the 8192-store clear: {completed_stores}",
        ));
    }
    Ok(Some(RescuedMaidenTilemapClearProgressReceipt {
        completed_stores,
        boundary,
    }))
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

fn link_oam_stair_progress(pc: u32, sub: Option<u8>) -> Option<zelda3::LinkOamStairProgress> {
    // LinkOam_Main applies a temporary gameplay Y adjustment only for these
    // two source submodules. This receipt describes that stair-drawing call;
    // other calls continue to use the ordinary LinkOam interruption grammar.
    if !matches!(sub, Some(18 | 19)) {
        return None;
    }
    match pc & 0x00ff_ffff {
        // Initial palette word is stored; follower palette selection and
        // both optional sprite banks have not run yet.
        0x0d_a47e => Some(zelda3::LinkOamStairProgress::PoseSelected),
        0x0d_a61a => Some(zelda3::LinkOamStairProgress::EquipmentSelection),
        // Link DMA index is stored; body coordinates and OAM remain pending.
        0x0d_a992 => Some(zelda3::LinkOamStairProgress::BodySelection),
        0x0d_a8b6 => Some(zelda3::LinkOamStairProgress::ShadowSelection),
        _ => None,
    }
}

fn main_loop_interruption_for_source_state(
    pc: u32,
    main: Option<u8>,
    sub: Option<u8>,
    x: Option<u16>,
) -> Option<MainLoopInterruption> {
    // SEP/PLB/RTL tail after the radius/goal test. Module0F/$01 still
    // identifies the recurring non-goal caller; its control clears follow.
    if main == Some(0x0f) && sub == Some(1) && matches!(pc, 0x00_f423 | 0x00_f425 | 0x00_f426) {
        return Some(MainLoopInterruption::DungeonExitSpotlightTableCompleted);
    }
    if main == Some(0x0f) && sub == Some(1) && pc == MODULE0F_AFTER_SUBMODULE_DISPATCH_PC {
        return Some(MainLoopInterruption::DungeonExitSpotlightAfterSubmodule);
    }
    if main == Some(0x0f) && sub == Some(1) && x == Some(0) && (0x07_e359..=0x07_e361).contains(&pc)
    {
        return Some(MainLoopInterruption::LinkActualVelocityCompleted);
    }
    if main == Some(0x12) && sub == Some(0) {
        if let Some(completed_stores) = game_over_iris_palette_completed_stores(pc, x?) {
            return Some(MainLoopInterruption::GameOverIrisGoalPaletteFill { completed_stores });
        }
    }
    if main == Some(0x0f)
        && sub == Some(1)
        && matches!(x, Some(0 | 1))
        && (LINK_ACTUAL_VELOCITY_PASS_START_PC..LINK_ACTUAL_VELOCITY_BEFORE_STORE_END_PC)
            .contains(&pc)
    {
        Some(MainLoopInterruption::LinkActualVelocity {
            horizontal_resolved: Some(x == Some(0)),
        })
    } else if main == Some(0x0f) && sub == Some(1) && (0x07_e2d2..0x07_e2e8).contains(&pc) {
        // All four STZ stores ($27, $28, $68, $69) have completed.
        // Direction indexing is call-local; the next stateful branch starts
        // after LDA $5b at $07:E2E7. Retain the selected speed for its suffix.
        Some(MainLoopInterruption::LinkActualVelocity {
            horizontal_resolved: None,
        })
    } else if main == Some(0x0f)
        && sub == Some(1)
        && x == Some(0)
        && (0x07_e3a4..=0x07_e3af).contains(&pc)
    {
        // The loop has advanced to Y, but its first subpixel store has not
        // executed. This is exactly the existing completed-X checkpoint;
        // scratch arithmetic between the passes does not publish gameplay.
        Some(MainLoopInterruption::LinkPositionAfterCoordinates { pass: 2 })
    } else if main == Some(0x0f)
        && sub == Some(1)
        && (pc == MODULE0F_LINK_VELOCITY_CALL_PC
            // The JSL has entered the same leaf; its first PHB has not
            // executed. Module0F's speed/ripple stores remain the prefix.
            || pc == 0x07_e245
            || (LINK_VELOCITY_BEFORE_STATE_BRANCH_START_PC
                ..LINK_VELOCITY_BEFORE_STATE_BRANCH_END_PC)
                .contains(&pc)
            || (LINK_VELOCITY_AFTER_SPEED_SELECTION_START_PC
                ..LINK_VELOCITY_BEFORE_FIRST_STATE_STORE_END_PC)
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
    } else if main == Some(0x0f)
        && sub == Some(1)
        && (LINK_POSITION_AFTER_COORDINATE_LOW_START_PC..LINK_POSITION_AFTER_COORDINATE_LOW_END_PC)
            .contains(&pc)
    {
        let pass = u8::try_from(x?)
            .ok()
            .filter(|pass| matches!(pass, 0 | 2 | 4))?;
        Some(MainLoopInterruption::LinkPositionAfterCoordinateLow { pass })
    } else if main == Some(0x0f)
        && sub == Some(1)
        && (LINK_POSITION_AFTER_COORDINATES_START_PC..LINK_POSITION_AFTER_COORDINATES_END_PC)
            .contains(&pc)
    {
        let pass = if pc >= 0x07e3d3 {
            // The only fall-through from the final BPL is the completed Y
            // pass. Both DEX instructions have run, so X is $fffe rather
            // than the semantic pass value zero.
            0
        } else {
            u8::try_from(x?)
                .ok()
                .filter(|pass| matches!(pass, 0 | 2 | 4))?
        };
        Some(MainLoopInterruption::LinkPositionAfterCoordinates { pass })
    } else if matches!(
        (main, sub),
        (Some(0x0f | 0x10), Some(0 | 1)) | (Some(0x12), Some(0))
    ) && (IRIS_SPOTLIGHT_RESET_TABLE_FIRST_STORE_PC
        ..=IRIS_SPOTLIGHT_RESET_TABLE_BRANCH_PC)
        .contains(&pc)
    {
        let completed_stores = spotlight_reset_table_completed_stores(pc, x?)?;
        Some(MainLoopInterruption::SpotlightGoalResetTable { completed_stores })
    } else {
        main_loop_interruption_for_pc(pc)
    }
}

fn game_over_iris_palette_completed_stores(pc: u32, x: u16) -> Option<u8> {
    let stores_in_iteration = match pc {
        GAME_OVER_IRIS_PALETTE_FIRST_STORE_PC => 0,
        GAME_OVER_IRIS_PALETTE_SECOND_STORE_PC => 1,
        GAME_OVER_IRIS_PALETTE_THIRD_STORE_PC => 2,
        GAME_OVER_IRIS_PALETTE_FOURTH_STORE_PC => 3,
        GAME_OVER_IRIS_PALETTE_FIFTH_STORE_PC => 4,
        GAME_OVER_IRIS_PALETTE_SIXTH_STORE_PC => 5,
        GAME_OVER_IRIS_PALETTE_FIRST_INCREMENT_PC | GAME_OVER_IRIS_PALETTE_SECOND_INCREMENT_PC => 6,
        GAME_OVER_IRIS_PALETTE_COMPARE_PC | GAME_OVER_IRIS_PALETTE_BRANCH_PC => 0,
        _ => return None,
    };
    let completed_iterations = match pc {
        GAME_OVER_IRIS_PALETTE_SECOND_INCREMENT_PC => x.checked_sub(1)? / 2,
        GAME_OVER_IRIS_PALETTE_COMPARE_PC | GAME_OVER_IRIS_PALETTE_BRANCH_PC => x / 2,
        _ => x / 2,
    };
    let completed = if matches!(
        pc,
        GAME_OVER_IRIS_PALETTE_COMPARE_PC | GAME_OVER_IRIS_PALETTE_BRANCH_PC
    ) {
        completed_iterations * 6
    } else {
        completed_iterations * 6 + stores_in_iteration
    };
    u8::try_from(completed).ok().filter(|&stores| stores <= 96)
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

fn triforce_room_case2_palette_progress(
    event: &RawTraceEvent,
    boundary: OriginalTimingBoundary,
) -> Result<Option<TriforceRoomCase2PaletteProgressReceipt>, String> {
    let pc = event.pc.map(|pc| pc & 0x00ff_ffff);
    if (event.main, event.sub, event.subsub) != (Some(0x19), Some(0), Some(2)) {
        return Ok(None);
    }

    // Entering the overlay parser is an unambiguous source-order proof that
    // `Overworld_EnterSpecialArea`, including all palette words, returned.
    if event.event == "nmi"
        && pc == Some(OVERWORLD_PARSE_MAP32_DEFINITION_SECOND_WORD_PC)
        && event.room == Some(0x0189)
    {
        return Ok(Some(TriforceRoomCase2PaletteProgressReceipt {
            completed_ow_bg2_words: 21,
            boundary,
        }));
    }
    if pc != Some(PALETTE_LOAD_MULTIPLE_BEFORE_WORD_COPY_PC) || event.room != Some(0x0109) {
        return Ok(None);
    }

    let x = event
        .x
        .ok_or("Triforce case-2 OWBG2 palette progress omitted destination X")?;
    let completed_ow_bg2_words = [0x00b2u16, 0x00d2, 0x00f2]
        .into_iter()
        .enumerate()
        .find_map(|(row, base)| {
            (x >= base && x <= base + 14 && (x - base) & 1 == 0)
                .then_some((row * 7 + usize::from((x - base) / 2)) as u8)
        })
        .ok_or_else(|| {
            format!("Triforce case-2 OWBG2 palette progress used invalid destination X {x:#06x}")
        })?;
    Ok(Some(TriforceRoomCase2PaletteProgressReceipt {
        completed_ow_bg2_words,
        boundary,
    }))
}

fn credits_scene_load_boundary_progress(
    event: &RawTraceEvent,
    boundary: OriginalTimingBoundary,
) -> Result<Option<CreditsSceneLoadProgressReceipt>, String> {
    if event.pc.map(|pc| pc & 0x00ff_ffff) != Some(CREDITS_ENDING_TEXT_BEFORE_TILE_COPY_PC)
        || event.main != Some(0x1a)
    {
        return Ok(None);
    }
    let completed_payload_bytes = event
        .x
        .ok_or("credits ending-text boundary omitted destination X")?;
    if completed_payload_bytes & 1 != 0 {
        return Err(format!(
            "credits ending-text boundary used odd destination X ${completed_payload_bytes:04x}",
        ));
    }
    Ok(Some(CreditsSceneLoadProgressReceipt {
        progress: CreditsSceneLoadProgress::EndingTextPayloadBytes(completed_payload_bytes),
        boundary,
    }))
}

fn credits_end_sequence_32_boundary_progress(
    event: &RawTraceEvent,
    boundary: OriginalTimingBoundary,
) -> Result<Option<CreditsEndSequence32ProgressReceipt>, String> {
    if event.pc.map(|pc| pc & 0x00ff_ffff) != Some(CREDITS_END_SEQUENCE_32_SAVE_CHECKSUM_LOOP_PC)
        || (event.main, event.sub, event.subsub) != (Some(0x1a), Some(0x21), Some(0))
    {
        return Ok(None);
    }
    let checksum_byte_cursor = event
        .x
        .ok_or("credits finale save-checksum boundary omitted source cursor")?;
    if checksum_byte_cursor > 0x4fe || checksum_byte_cursor & 1 != 0 {
        return Err(format!(
            "credits finale save-checksum boundary used invalid cursor ${checksum_byte_cursor:04x}",
        ));
    }
    Ok(Some(CreditsEndSequence32ProgressReceipt {
        completed_checksum_words: checksum_byte_cursor / 2,
        boundary,
    }))
}

fn main_loop_interruption_for_event(
    event: &RawTraceEvent,
) -> Result<Option<MainLoopInterruption>, String> {
    let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
        return Ok(None);
    };
    if let Some(interruption) = desert_prayer_iris_interruption(
        pc,
        event.main,
        event.sub,
        event.subsub,
        event.spotlight_radius,
        event.spotlight_var4_low,
        event.palette_countdown,
        event.link_y,
        event.bg2_v,
        event.a,
        event.x,
        event.y,
    )? {
        return Ok(Some(interruption));
    }
    if let Some(interruption) = desert_prayer_palette_filter_interruption(
        pc,
        event.main,
        event.sub,
        event.subsub,
        event.palette_countdown,
        event.x,
    )? {
        return Ok(Some(interruption));
    }
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

fn desert_prayer_iris_interruption(
    pc: u32,
    main: Option<u8>,
    sub: Option<u8>,
    subsub: Option<u8>,
    spotlight_radius: Option<u16>,
    y_buffer: Option<u8>,
    palette_countdown: Option<u8>,
    link_y: Option<u16>,
    bg2_v: Option<u16>,
    a: Option<u16>,
    x: Option<u16>,
    y: Option<u16>,
) -> Result<Option<MainLoopInterruption>, String> {
    let in_builder =
        (DESERT_PRAYER_IRIS_ENTRY_PC..=DESERT_PRAYER_IRIS_STATE4_TAIL_START_PC).contains(&pc);
    let in_shape_helper = (DESERT_PRAYER_IRIS_SHAPE_HELPER_START_PC
        ..DESERT_PRAYER_IRIS_SHAPE_HELPER_END_PC)
        .contains(&pc);
    if (main, sub) != (Some(0x0e), Some(5)) || (!in_builder && !in_shape_helper) {
        return Ok(None);
    }
    let source_subsubmodule = subsub
        .filter(|subsub| (2..=4).contains(subsub))
        .ok_or_else(|| {
            format!("Snes9x Desert Prayer iris checkpoint used invalid subsubmodule {subsub:?}",)
        })?;
    let radius = spotlight_radius.ok_or("Snes9x Desert Prayer iris checkpoint omitted radius")?;
    if source_subsubmodule != 4 && radius != 0x26 {
        return Err(format!(
            "Snes9x Desert Prayer iris checkpoint used radius {:?}, expected $26",
            spotlight_radius,
        ));
    }
    if radius == 0 || radius >= 0xc0 {
        return Err(format!(
            "Snes9x Desert Prayer iris checkpoint used invalid live radius ${radius:04x}",
        ));
    }
    let y_buffer = y_buffer.ok_or("Snes9x Desert Prayer iris checkpoint omitted row cursor")?;
    if y_buffer == 0 || u16::from(y_buffer) > radius + 1 {
        return Err(format!(
            "Snes9x Desert Prayer iris checkpoint used row cursor {y_buffer} outside radius {radius}",
        ));
    }
    let palette_countdown = palette_countdown
        .ok_or("Snes9x Desert Prayer iris checkpoint omitted palette countdown")?;
    let progress = if in_shape_helper {
        // The helper only computes the current radial pair. No Zelda-owned
        // persistent state is published until its caller reaches the primary
        // HDMA-table store, so every instruction inside it resumes from that
        // source statement boundary.
        zelda3::DesertPrayerIrisProgress::BeforePrimaryTableWrite {
            table_word: desert_prayer_radial_primary_table_word(link_y, bg2_v, y_buffer)?,
            y_buffer,
        }
    } else if pc < DESERT_PRAYER_IRIS_LOWER_Y_PUBLISHED_PC {
        zelda3::DesertPrayerIrisProgress::Setup {
            completed_writes: 0,
        }
    } else if pc < DESERT_PRAYER_IRIS_UPPER_Y_PUBLISHED_PC {
        zelda3::DesertPrayerIrisProgress::Setup {
            completed_writes: 1,
        }
    } else if pc < DESERT_PRAYER_IRIS_X_CENTER_PUBLISHED_PC {
        zelda3::DesertPrayerIrisProgress::Setup {
            completed_writes: 2,
        }
    } else if pc < DESERT_PRAYER_IRIS_CURSOR_PUBLISHED_PC {
        zelda3::DesertPrayerIrisProgress::Setup {
            completed_writes: 3,
        }
    } else if pc <= DESERT_PRAYER_IRIS_EARLY_ITERATION_END_PC
        || (DESERT_PRAYER_IRIS_RADIAL_BRANCH_START_PC..=DESERT_PRAYER_IRIS_RADIAL_BRANCH_END_PC)
            .contains(&pc)
        || (DESERT_PRAYER_IRIS_RADIAL_CALCULATION_START_PC
            ..=DESERT_PRAYER_IRIS_BEFORE_LOWER_ZERO_WRITE_PC)
            .contains(&pc)
    {
        let scanline = desert_prayer_scanline_before_iteration(
            pc,
            link_y,
            bg2_v,
            spotlight_radius,
            y_buffer,
            a,
            x,
            y,
        )?;
        zelda3::DesertPrayerIrisProgress::BeforeIteration { scanline }
    } else if (DESERT_PRAYER_IRIS_PRIMARY_VALUE_START_PC
        ..=DESERT_PRAYER_IRIS_PRIMARY_TABLE_WRITE_PC)
        .contains(&pc)
    {
        zelda3::DesertPrayerIrisProgress::BeforePrimaryTableWrite {
            table_word: if pc >= DESERT_PRAYER_IRIS_PRIMARY_INDEX_IN_X_PC {
                desert_prayer_table_word_from_x(x)?
            } else {
                desert_prayer_radial_primary_table_word(link_y, bg2_v, y_buffer)?
            },
            y_buffer,
        }
    } else if (DESERT_PRAYER_IRIS_AFTER_PRIMARY_TABLE_WRITE_PC
        ..=DESERT_PRAYER_IRIS_BEFORE_MIRRORED_TABLE_WRITE_PC)
        .contains(&pc)
    {
        zelda3::DesertPrayerIrisProgress::AfterPrimaryTableWrite {
            table_word: desert_prayer_primary_table_word_after_store(
                pc, x, link_y, bg2_v, y_buffer,
            )?,
            y_buffer,
        }
    } else if DESERT_PRAYER_IRIS_AFTER_ITERATION_PCS.contains(&pc) {
        let next_scanline = if pc == DESERT_PRAYER_IRIS_AFTER_ITERATION_PCS[0] {
            desert_prayer_table_word_from_x(x)?.wrapping_add(2)
        } else {
            a.ok_or("Snes9x Desert Prayer iris iteration checkpoint omitted next scanline")?
        };
        zelda3::DesertPrayerIrisProgress::AfterIteration {
            next_scanline,
            y_buffer,
        }
    } else if (DESERT_PRAYER_IRIS_LOOP_COMPLETE_START_PC..=DESERT_PRAYER_IRIS_STATE4_TAIL_START_PC)
        .contains(&pc)
    {
        zelda3::DesertPrayerIrisProgress::LoopComplete
    } else {
        return Err(format!(
            "Snes9x Desert Prayer iris NMI stopped at unsupported source statement ${pc:06x}",
        ));
    };
    Ok(Some(MainLoopInterruption::DesertPrayerIris {
        source_subsubmodule,
        palette_countdown,
        radius,
        progress,
    }))
}

fn desert_prayer_table_word_from_x(x: Option<u16>) -> Result<u16, String> {
    let x = x.ok_or("Snes9x Desert Prayer iris checkpoint omitted source cursor X")?;
    if x & 1 != 0 {
        return Err(format!(
            "Snes9x Desert Prayer iris checkpoint used odd table byte cursor ${x:04x}",
        ));
    }
    Ok(x / 2)
}

fn desert_prayer_primary_table_word_after_store(
    pc: u32,
    x: Option<u16>,
    link_y: Option<u16>,
    bg2_v: Option<u16>,
    y_buffer: u8,
) -> Result<u16, String> {
    if pc <= 0x07eb26 {
        return desert_prayer_table_word_from_x(x);
    }
    desert_prayer_radial_primary_table_word(link_y, bg2_v, y_buffer)
}

fn desert_prayer_radial_primary_table_word(
    link_y: Option<u16>,
    bg2_v: Option<u16>,
    y_buffer: u8,
) -> Result<u16, String> {
    let r14 = link_y
        .ok_or("Snes9x Desert Prayer iris checkpoint omitted Link Y")?
        .wrapping_sub(bg2_v.ok_or("Snes9x Desert Prayer iris checkpoint omitted BG2 Y")?)
        .wrapping_add(12);
    Ok(r14.wrapping_sub(u16::from(y_buffer)).wrapping_sub(1))
}

fn desert_prayer_scanline_before_iteration(
    pc: u32,
    link_y: Option<u16>,
    bg2_v: Option<u16>,
    radius: Option<u16>,
    y_buffer: u8,
    a: Option<u16>,
    x: Option<u16>,
    y: Option<u16>,
) -> Result<u16, String> {
    if pc >= DESERT_PRAYER_IRIS_RADIAL_BRANCH_START_PC {
        let r14 = link_y
            .ok_or("Snes9x Desert Prayer iris checkpoint omitted Link Y")?
            .wrapping_sub(bg2_v.ok_or("Snes9x Desert Prayer iris checkpoint omitted BG2 Y")?)
            .wrapping_add(12);
        let lower = r14.wrapping_sub(
            radius.ok_or("Snes9x Desert Prayer iris checkpoint omitted source radius")?,
        );
        return Ok(lower.wrapping_add(u16::from(y_buffer)).wrapping_sub(1));
    }
    if pc >= 0x07ea6f {
        return a.ok_or_else(|| {
            "Snes9x Desert Prayer iris checkpoint omitted scanline accumulator A".to_string()
        });
    }
    let r14 = link_y
        .ok_or("Snes9x Desert Prayer iris checkpoint omitted Link Y")?
        .wrapping_sub(bg2_v.ok_or("Snes9x Desert Prayer iris checkpoint omitted BG2 Y")?)
        .wrapping_add(12);
    let lower = r14
        .wrapping_sub(radius.ok_or("Snes9x Desert Prayer iris checkpoint omitted source radius")?);
    let initial = if lower & 0x8000 != 0 { lower } else { 0 };
    // Once the source has loaded `spotlight_y_lower`, a negative lower bound
    // forces the radial branch for every live iteration.  X and Y are still
    // scratch registers left by the preceding call/host at this boundary;
    // the persistent `spotlight_var4` cursor is the source authority for r4.
    if lower & 0x8000 != 0 && pc >= 0x07ea68 {
        return Ok(lower.wrapping_add(u16::from(y_buffer)).wrapping_sub(1));
    }
    if !matches!(y, Some(0 | 0xff)) {
        return Ok(initial);
    }
    let previous_table_word = desert_prayer_table_word_from_x(x)?;
    let upper = lower.wrapping_add(radius.unwrap() * 2);
    if previous_table_word < lower || previous_table_word >= upper {
        return Ok(previous_table_word.wrapping_add(2));
    }
    if previous_table_word == r14.wrapping_add(u16::from(y_buffer)).wrapping_sub(3) {
        return Ok(lower.wrapping_add(u16::from(y_buffer)).wrapping_sub(1));
    }
    Err(format!(
        "Snes9x Desert Prayer iris checkpoint cannot derive the next scanline from in-window cursor X=${:04x}",
        x.unwrap(),
    ))
}

fn desert_prayer_palette_filter_interruption(
    pc: u32,
    main: Option<u8>,
    sub: Option<u8>,
    subsub: Option<u8>,
    palette_countdown: Option<u8>,
    x: Option<u16>,
) -> Result<Option<MainLoopInterruption>, String> {
    if (main, sub, subsub) != (Some(0x0e), Some(5), Some(3))
        || !(PALETTE_FILTER_BEFORE_COLOR_LOAD_PC..=PALETTE_FILTER_BEFORE_COLOR_STORE_PC)
            .contains(&pc)
    {
        return Ok(None);
    }
    let countdown = palette_countdown
        .ok_or("Snes9x Desert Prayer palette checkpoint omitted source countdown")?;
    let x = x.ok_or("Snes9x Desert Prayer palette checkpoint omitted source cursor X")?;
    if x & 1 != 0 {
        return Err(format!(
            "Snes9x Desert Prayer palette checkpoint used odd byte cursor ${x:04x}",
        ));
    }
    let next_color = u8::try_from(x / 2)
        .map_err(|_| format!("Snes9x Desert Prayer palette cursor exceeded one byte: ${x:04x}"))?;
    if !((0x20..=0xd8).contains(&next_color) || (0xe0..=0xf0).contains(&next_color)) {
        return Err(format!(
            "Snes9x Desert Prayer palette checkpoint used invalid next color ${next_color:02x}",
        ));
    }
    Ok(Some(
        MainLoopInterruption::DesertPrayerPaletteFilterBeforeColor {
            countdown,
            next_color,
        },
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
                MainLoopInterruption::SpriteMainAfterTimersAndOam(slot) => {
                    Some(SpriteMainProgress::AfterTimersAndOam(slot))
                }
                MainLoopInterruption::SpriteMainAfterTimerDecrements(slot) => {
                    Some(SpriteMainProgress::AfterTimerDecrements(slot))
                }
                MainLoopInterruption::SpriteMainAfterPrimaryTimerDecrements(slot) => {
                    Some(SpriteMainProgress::AfterPrimaryTimerDecrements(slot))
                }
                MainLoopInterruption::SpriteMainAfterHitTimer(slot) => {
                    Some(SpriteMainProgress::AfterHitTimer(slot))
                }
                MainLoopInterruption::SpriteMainAfterMainAndAux1TimerDecrements(slot) => {
                    Some(SpriteMainProgress::AfterMainAndAux1TimerDecrements(slot))
                }
                MainLoopInterruption::SpriteMainAfterMainTimerDecrement(slot) => {
                    Some(SpriteMainProgress::AfterMainTimerDecrement(slot))
                }
                MainLoopInterruption::SpriteMainAfterZeroHitTimerClear(slot) => {
                    Some(SpriteMainProgress::AfterZeroHitTimerClear(slot))
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
                MainLoopInterruption::SpriteMainMasterSwordLightBeamMovement {
                    slot,
                    checkpoint,
                } => Some(SpriteMainProgress::MasterSwordLightBeamMovement { slot, checkpoint }),
                MainLoopInterruption::SpriteMainMasterSwordLightBeamSpawn {
                    slot,
                    spawned_slot,
                    progress,
                } => Some(SpriteMainProgress::MasterSwordLightBeamSpawn {
                    slot,
                    spawned_slot,
                    progress,
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
                MainLoopInterruption::SpriteMainBonkItemGraphicsStarted(slot) => {
                    Some(SpriteMainProgress::BonkItemGraphicsStarted(slot))
                }
                MainLoopInterruption::SpriteMainProbeAfterOamCoordinates(slot) => {
                    Some(SpriteMainProgress::ProbeAfterOamCoordinates(slot))
                }
                MainLoopInterruption::SpriteMainInitializeResetProperties {
                    slot,
                    phase,
                    completed_stores,
                } => Some(SpriteMainProgress::InitializeResetProperties {
                    slot,
                    phase,
                    completed_stores,
                }),
                MainLoopInterruption::SpriteMainInitializeLoadProperties {
                    slot,
                    phase,
                    completed_stores,
                } => Some(SpriteMainProgress::InitializeLoadProperties {
                    slot,
                    phase,
                    completed_stores,
                }),
                MainLoopInterruption::SpriteMainFireDebirandoBeforeSpawn(slot) => {
                    Some(SpriteMainProgress::FireDebirandoBeforeSpawn(slot))
                }
                MainLoopInterruption::SpriteMainFireDebirandoSpawn {
                    slot,
                    spawned_slot,
                    progress,
                } => Some(SpriteMainProgress::FireDebirandoSpawn {
                    slot,
                    spawned_slot,
                    progress,
                }),
                MainLoopInterruption::SpriteMainGuardPrepWeaponFlagsPending(slot) => {
                    Some(SpriteMainProgress::GuardPrepWeaponFlagsPending(slot))
                }
                MainLoopInterruption::SpriteMainGuardAnimation { slot, checkpoint } => {
                    Some(SpriteMainProgress::GuardAnimation { slot, checkpoint })
                }
                MainLoopInterruption::SpriteMainGuardPrepPatrolDelay { slot, active_call } => {
                    Some(SpriteMainProgress::GuardPrepPatrolDelay { slot, active_call })
                }
                MainLoopInterruption::SpriteMainGuardPrepTileCollisionReturned {
                    slot,
                    active_call,
                } => Some(SpriteMainProgress::GuardPrepTileCollisionReturned { slot, active_call }),
                MainLoopInterruption::SpriteMainInitializePrepPending(slot) => {
                    Some(SpriteMainProgress::InitializePrepPending(slot))
                }
                MainLoopInterruption::SpriteMainHogSpearBodyGraphicsPending(slot) => {
                    Some(SpriteMainProgress::HogSpearBodyGraphicsPending(slot))
                }
                MainLoopInterruption::SpriteMainAbsorbableHorizontalTileLookup(slot) => {
                    Some(SpriteMainProgress::AbsorbableHorizontalTileLookup(slot))
                }
                MainLoopInterruption::SpriteMainAbsorbableVerticalTileLookup(slot) => {
                    Some(SpriteMainProgress::AbsorbableVerticalTileLookup(slot))
                }
                MainLoopInterruption::SpriteMainAbsorbableVerticalTileAttributeLoaded(slot) => {
                    Some(SpriteMainProgress::AbsorbableVerticalTileAttributeLoaded(
                        slot,
                    ))
                }
                MainLoopInterruption::SpriteMainSwamolaHeadDraw(slot) => {
                    Some(SpriteMainProgress::SwamolaHeadDraw(slot))
                }
                MainLoopInterruption::SpriteMainSwamolaHeadDrawCompleted(slot) => {
                    Some(SpriteMainProgress::SwamolaHeadDrawCompleted(slot))
                }
                MainLoopInterruption::SpriteMainVitreousDamagePending(slot) => {
                    Some(SpriteMainProgress::VitreousDamagePending(slot))
                }
                MainLoopInterruption::SpriteMainVitreousAiPending(slot) => {
                    Some(SpriteMainProgress::VitreousAiPending(slot))
                }
                MainLoopInterruption::SpriteMainVitreousPlayerDamagePending(slot) => {
                    Some(SpriteMainProgress::VitreousPlayerDamagePending(slot))
                }
                MainLoopInterruption::SpriteMainSwamolaSegmentDraw { slot, segment } => {
                    Some(SpriteMainProgress::SwamolaSegmentDraw { slot, segment })
                }
                MainLoopInterruption::SpriteMainPengatorSlidePending(slot) => {
                    Some(SpriteMainProgress::PengatorSlidePending(slot))
                }
                MainLoopInterruption::SpriteMainAntifairyBouncePending(slot) => {
                    Some(SpriteMainProgress::AntifairyBouncePending(slot))
                }
                MainLoopInterruption::SpriteMainKholdstareDamagePending(slot) => {
                    Some(SpriteMainProgress::KholdstareDamagePending(slot))
                }
                MainLoopInterruption::SpriteMainGuardPrepParryHitbox { slot, active_call } => {
                    Some(SpriteMainProgress::GuardPrepParryHitbox { slot, active_call })
                }
                MainLoopInterruption::SpriteMainMiniMoldormHistory {
                    slot,
                    completed_stores,
                } => Some(SpriteMainProgress::MiniMoldormHistory {
                    slot,
                    completed_stores,
                }),
                _ => None,
            };
            if let Some(progress) = progress {
                receipts[*index] = OriginalTimingSemanticReceipt::SpriteMainProgressed(progress);
            } else {
                receipts.remove(*index);
            }
            if interruption == MainLoopInterruption::LinkOam {
                // This exact interrupted source context resumed in the same
                // host. Its partial drawing checkpoint no longer names an
                // outstanding call, just like the enclosing interruption.
                for index in (last_acceptance + 1..receipts.len()).rev() {
                    if matches!(
                        receipts[index],
                        OriginalTimingSemanticReceipt::LinkOamStairProgress(_)
                    ) {
                        receipts.remove(index);
                    }
                }
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

fn dungeon_load_single_sprite_write_progress(
    pc: u32,
    address: u16,
    x: Option<u16>,
) -> Result<Option<(u8, DungeonSpriteLoadCheckpoint)>, String> {
    let slot =
        u8::try_from(x.ok_or("Snes9x Dungeon_LoadSingleSprite field write omitted source slot X")?)
            .map_err(|_| "Snes9x Dungeon_LoadSingleSprite source slot exceeded one byte")?;
    if slot >= 16 {
        return Err(format!(
            "Snes9x Dungeon_LoadSingleSprite used invalid source slot {slot}",
        ));
    }
    let indexed = |base: u16| address == base + u16::from(slot);
    let checkpoint = match pc {
        DUNGEON_LOAD_SINGLE_SPRITE_TEMP_Y_PC if address == DUNGEON_LOAD_TEMP_Y => {
            DungeonSpriteLoadCheckpoint::TempY
        }
        DUNGEON_LOAD_SINGLE_SPRITE_FLOOR_PC if indexed(SPRITE_FLOOR_BASE) => {
            DungeonSpriteLoadCheckpoint::Floor
        }
        DUNGEON_LOAD_SINGLE_SPRITE_Y_LOW_PC if indexed(SPRITE_Y_LOW_BASE) => {
            DungeonSpriteLoadCheckpoint::YLow
        }
        DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC if indexed(SPRITE_Y_HIGH_BASE) => {
            DungeonSpriteLoadCheckpoint::YHigh
        }
        DUNGEON_LOAD_SINGLE_SPRITE_SHARED_X_PC if address == DUNGEON_LOAD_SHARED_X => {
            DungeonSpriteLoadCheckpoint::SharedX
        }
        DUNGEON_LOAD_SINGLE_SPRITE_X_LOW_PC if indexed(SPRITE_X_LOW_BASE) => {
            DungeonSpriteLoadCheckpoint::XLow
        }
        DUNGEON_LOAD_SINGLE_SPRITE_X_HIGH_PC if indexed(SPRITE_X_HIGH_BASE) => {
            DungeonSpriteLoadCheckpoint::XHigh
        }
        DUNGEON_LOAD_SINGLE_SPRITE_TYPE_PC if indexed(SPRITE_TYPE_BASE) => {
            DungeonSpriteLoadCheckpoint::Type
        }
        DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_CLEAR_PC if indexed(SPRITE_SUBTYPE_BASE) => {
            DungeonSpriteLoadCheckpoint::SubtypeClear
        }
        DUNGEON_LOAD_SINGLE_SPRITE_TEMP_SUBTYPE_PC if address == DUNGEON_LOAD_TEMP_Y => {
            DungeonSpriteLoadCheckpoint::TempSubtype
        }
        DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_FINAL_PC if indexed(SPRITE_SUBTYPE_BASE) => {
            DungeonSpriteLoadCheckpoint::SubtypeFinal
        }
        DUNGEON_LOAD_SINGLE_SPRITE_SPAWN_INDEX_PC
            if (SPRITE_N_WORD_BASE..SPRITE_N_WORD_BASE + 16).contains(&address)
                && indexed(SPRITE_N_WORD_BASE) =>
        {
            DungeonSpriteLoadCheckpoint::SpawnIndex
        }
        DUNGEON_LOAD_SINGLE_SPRITE_COMPLETE_PC if indexed(SPRITE_DIE_ACTION_BASE) => {
            DungeonSpriteLoadCheckpoint::Complete
        }
        _ => return Ok(None),
    };
    Ok(Some((slot, checkpoint)))
}

fn dungeon_push_blocks_pending(event: &RawTraceEvent) -> bool {
    // PHB/PHK/PLB and the first block-index read have completed. The saved
    // DB byte precedes Module 7's near return bytes on the native stack.
    event.pc.map(|pc| pc & 0xff_ffff) == Some(0x07_f0b2)
        && event.return_address.map(|pc| pc & 0xff_ffff) == Some(0x88_3d00)
        && event.main == Some(7)
}

fn dungeon_reset_sprites_caller_progress(
    event: &RawTraceEvent,
) -> Option<DungeonResetSpritesCpuProgress> {
    let pc = event.pc? & 0x00ff_ffff;
    // The history rotation has finished. CMP #$ffff / BEQ skips the
    // evicted-room death-mask clear for an empty history entry, leaving no
    // gameplay writes before Dungeon_LoadSprites starts reading its pointer.
    if matches!(pc, 0x09_c163 | 0x09_c166) && event.a == Some(0xffff) {
        return Some(DungeonResetSpritesCpuProgress::LoadBeforeOrigin);
    }
    // First Dungeon_LoadSingleSprite call, after its two INYs and type
    // read, before either the marker branch or normal-slot publication.
    if (0x09_c32b..0x09_c330).contains(&pc) && event.y == Some(3) {
        return Some(DungeonResetSpritesCpuProgress::LoadStarted);
    }
    if (0x09_c290..0x09_c2a6).contains(&pc) {
        return Some(DungeonResetSpritesCpuProgress::LoadBeforeOrigin);
    }
    if (DUNGEON_RESET_SPRITES_AFTER_DISABLE_PC..DUNGEON_RESET_SPRITES_COLLISION_Y_STORE_PC)
        .contains(&pc)
    {
        Some(DungeonResetSpritesCpuProgress::SpritesDisabled)
    } else if (DUNGEON_RESET_SPRITES_COLLISION_Y_STORE_PC
        ..DUNGEON_RESET_SPRITES_HISTORY_SEARCH_START_PC)
        .contains(&pc)
    {
        Some(DungeonResetSpritesCpuProgress::CollisionXSizeSet)
    } else if (DUNGEON_RESET_SPRITES_HISTORY_SEARCH_START_PC
        ..DUNGEON_RESET_SPRITES_HISTORY_FIRST_MUTATION_PC)
        .contains(&pc)
        || (DUNGEON_RESET_SPRITES_HISTORY_FOUND_PC..DUNGEON_RESET_SPRITES_LOAD_CALL_PC)
            .contains(&pc)
    {
        Some(DungeonResetSpritesCpuProgress::RoomHistorySearchStarted)
    } else {
        None
    }
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
    fn desert_prayer_iris_exports_source_statement_progress() {
        assert_eq!(
            desert_prayer_iris_interruption(
                0x07eb31,
                Some(0x0e),
                Some(5),
                Some(2),
                Some(0x26),
                Some(22),
                Some(0),
                Some(3472),
                Some(3374),
                Some(130),
                Some(131),
                Some(255),
            )
            .unwrap(),
            Some(MainLoopInterruption::DesertPrayerIris {
                source_subsubmodule: 2,
                palette_countdown: 0,
                radius: 0x26,
                progress: zelda3::DesertPrayerIrisProgress::AfterPrimaryTableWrite {
                    table_word: 87,
                    y_buffer: 22,
                },
            }),
        );
        assert_eq!(
            desert_prayer_iris_interruption(
                0x07ed1d,
                Some(0x0e),
                Some(5),
                Some(4),
                Some(187),
                Some(69),
                Some(0),
                Some(3472),
                Some(3374),
                Some(0),
                Some(80),
                Some(255),
            )
            .unwrap(),
            Some(MainLoopInterruption::DesertPrayerIris {
                source_subsubmodule: 4,
                palette_countdown: 0,
                radius: 187,
                progress: zelda3::DesertPrayerIrisProgress::BeforePrimaryTableWrite {
                    table_word: 40,
                    y_buffer: 69,
                },
            }),
        );
        assert_eq!(
            desert_prayer_iris_interruption(
                0x07ea6b,
                Some(0x0e),
                Some(5),
                Some(4),
                Some(115),
                Some(1),
                Some(0),
                Some(3472),
                Some(3374),
                Some(0xfffb),
                Some(352),
                Some(0xff),
            )
            .unwrap(),
            Some(MainLoopInterruption::DesertPrayerIris {
                source_subsubmodule: 4,
                palette_countdown: 0,
                radius: 115,
                progress: zelda3::DesertPrayerIrisProgress::BeforeIteration { scanline: 0xfffb },
            }),
        );
        assert_eq!(
            desert_prayer_iris_interruption(
                0x07eb03,
                Some(0x0e),
                Some(5),
                Some(3),
                Some(0x26),
                Some(1),
                Some(11),
                Some(3472),
                Some(3374),
                Some(0xffff),
                Some(444),
                Some(255),
            )
            .unwrap(),
            Some(MainLoopInterruption::DesertPrayerIris {
                source_subsubmodule: 3,
                palette_countdown: 11,
                radius: 0x26,
                progress: zelda3::DesertPrayerIrisProgress::BeforePrimaryTableWrite {
                    table_word: 222,
                    y_buffer: 1,
                },
            }),
        );
        assert_eq!(
            desert_prayer_iris_interruption(
                0x07ea66,
                Some(0x0e),
                Some(5),
                Some(4),
                Some(0x26),
                Some(34),
                Some(0),
                Some(3472),
                Some(3374),
                Some(0x0100),
                Some(0x011a),
                Some(0x00ff),
            )
            .unwrap(),
            Some(MainLoopInterruption::DesertPrayerIris {
                source_subsubmodule: 4,
                palette_countdown: 0,
                radius: 0x26,
                progress: zelda3::DesertPrayerIrisProgress::BeforeIteration { scanline: 105 },
            }),
        );
        assert_eq!(
            desert_prayer_iris_interruption(
                0x07eaa1,
                Some(0x0e),
                Some(5),
                Some(4),
                Some(0x26),
                Some(35),
                Some(0),
                Some(3472),
                Some(3374),
                Some(0),
                Some(117),
                Some(0),
            )
            .unwrap(),
            Some(MainLoopInterruption::DesertPrayerIris {
                source_subsubmodule: 4,
                palette_countdown: 0,
                radius: 0x26,
                progress: zelda3::DesertPrayerIrisProgress::BeforePrimaryTableWrite {
                    table_word: 74,
                    y_buffer: 35,
                },
            }),
        );
        assert_eq!(
            desert_prayer_iris_interruption(
                0x07ea9e,
                Some(0x0e),
                Some(5),
                Some(4),
                Some(0x26),
                Some(35),
                Some(0),
                Some(3472),
                Some(3374),
                Some(0),
                Some(117),
                Some(0),
            )
            .unwrap(),
            Some(MainLoopInterruption::DesertPrayerIris {
                source_subsubmodule: 4,
                palette_countdown: 0,
                radius: 0x26,
                progress: zelda3::DesertPrayerIrisProgress::BeforeIteration { scanline: 106 },
            }),
        );
        assert_eq!(
            desert_prayer_iris_interruption(
                0x07ea77,
                Some(0x0e),
                Some(5),
                Some(4),
                Some(51),
                Some(40),
                Some(0),
                Some(3472),
                Some(3374),
                Some(98),
                Some(294),
                Some(200),
            )
            .unwrap(),
            Some(MainLoopInterruption::DesertPrayerIris {
                source_subsubmodule: 4,
                palette_countdown: 0,
                radius: 51,
                progress: zelda3::DesertPrayerIrisProgress::BeforeIteration { scanline: 98 },
            }),
        );
        assert!(desert_prayer_iris_interruption(
            0x07eb31,
            Some(0x0e),
            Some(5),
            Some(2),
            Some(0x25),
            Some(22),
            Some(0),
            Some(3472),
            Some(3374),
            Some(130),
            Some(131),
            Some(255),
        )
        .unwrap_err()
        .contains("expected $26"));
    }

    #[test]
    fn desert_prayer_palette_filter_exports_next_source_color() {
        assert_eq!(
            desert_prayer_palette_filter_interruption(
                0x00e9e4,
                Some(0x0e),
                Some(5),
                Some(3),
                Some(0),
                Some(0x01aa),
            )
            .unwrap(),
            Some(MainLoopInterruption::DesertPrayerPaletteFilterBeforeColor {
                countdown: 0,
                next_color: 213,
            }),
        );
        assert!(desert_prayer_palette_filter_interruption(
            0x00e9e4,
            Some(0x0e),
            Some(5),
            Some(3),
            None,
            Some(0x01aa),
        )
        .unwrap_err()
        .contains("omitted source countdown"));
    }

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
    fn zero_hit_timer_clear_precedes_priority_store() {
        for pc in 0x06_8499..0x06_849c {
            let mut execution = SpriteMainExecutionTracker {
                current_slot: Some(12),
                last_completed_slot: Some(13),
                primary_timer_decrements_slot: Some(12),
                ..Default::default()
            };
            execution
                .observe_zero_hit_timer_clear(&raw("nmi", Some(pc), Some(12), None))
                .unwrap();
            assert_eq!(
                execution.progress(),
                SpriteMainProgress::AfterZeroHitTimerClear(12)
            );
            execution
                .observe_hit_timer(&raw("nmi", Some(0x06_849c), Some(12), None))
                .unwrap();
            assert_eq!(execution.progress(), SpriteMainProgress::AfterHitTimer(12));
        }
    }

    #[test]
    fn nmi_inside_aux1_load_publishes_only_main_countdown() {
        let mut execution = SpriteMainExecutionTracker {
            current_slot: Some(12),
            last_completed_slot: Some(13),
            ..Default::default()
        };
        let nmi = raw("nmi", Some(0x06_8429), Some(12), None);
        execution.observe_main_timer_decrement(&nmi).unwrap();
        execution.observe_zero_hit_timer_clear(&nmi).unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::AfterMainTimerDecrement(12)
        );
        assert_eq!(
            execution.interruption(),
            MainLoopInterruption::SpriteMainAfterMainTimerDecrement(12)
        );
        execution
            .observe_main_and_aux1_timer_decrements(&raw("nmi", Some(0x06_8432), Some(12), None))
            .unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::AfterMainAndAux1TimerDecrements(12)
        );
    }

    #[test]
    fn nmi_inside_aux2_load_publishes_main_and_aux1_countdowns() {
        let mut execution = SpriteMainExecutionTracker {
            current_slot: Some(0),
            last_completed_slot: Some(1),
            ..SpriteMainExecutionTracker::default()
        };
        let nmi = raw(
            "nmi",
            Some(SPRITE_MAIN_AND_AUX1_TIMER_DECREMENTS_COMPLETE_START_PC + 2),
            Some(0),
            None,
        );

        execution
            .observe_main_and_aux1_timer_decrements(&nmi)
            .unwrap();

        assert_eq!(
            execution.progress(),
            SpriteMainProgress::AfterMainAndAux1TimerDecrements(0),
        );
        assert_eq!(
            execution.interruption(),
            MainLoopInterruption::SpriteMainAfterMainAndAux1TimerDecrements(0),
        );
    }

    #[test]
    fn host_return_inside_hit_timer_load_publishes_primary_countdowns() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        for (pc, x) in [
            (SPRITE_MAIN_ENTRY_PC, None),
            (SPRITE_EXECUTE_SINGLE_ENTRY_PC, Some(0)),
        ] {
            source
                .consume_event(raw("pc", Some(pc), x, None), &mut receipts)
                .unwrap();
        }
        let returned = raw(
            "frame",
            Some(SPRITE_PRIMARY_TIMER_DECREMENTS_COMPLETE_START_PC + 4),
            Some(0),
            None,
        );
        source
            .sprite_main_execution
            .as_mut()
            .unwrap()
            .observe_primary_timer_decrements(&returned)
            .unwrap();

        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterPrimaryTimerDecrements(0),
            )],
        );
    }

    #[test]
    fn host_return_coalesces_resumed_sprite_progress_to_the_latest_checkpoint() {
        let mut source = empty_semantic_tracker();
        source.sprite_main_execution = Some(SpriteMainExecutionTracker {
            current_slot: Some(2),
            last_completed_slot: Some(3),
            timers_and_oam_slot: None,
            timers_and_oam_dispatch_state: None,
            initialize_active_main_calls: 0,
            guard_prep_parry_hitbox: None,
            guard_prep_patrol_delay: None,
            guard_prep_tile_collision_return: None,
            guard_animation_checkpoint: None,
            hog_spear_body_graphics_pending: None,
            absorbable_body_active: false,
            absorbable_horizontal_lookup: None,
            absorbable_vertical_lookup: None,
            absorbable_vertical_attribute_loaded: None,
            swamola_segment: None,
            dispatch_trampoline_return: None,
            vitreous_minions_seen: false,
            vitreous_player_damage_pending: None,
            vitreous_ai_pending: None,
            vitreous_damage_pending: None,
            swamola_head_prepared: false,
            swamola_head_draw_completed: None,
            swamola_head_draw: None,
            swamola_segment_draw: None,
            pengator_slide_pending: None,
            antifairy_bounce_pending: None,
            kholdstare_subtype_decremented: false,
            kholdstare_damage_pending: None,
            initialize_prep_pending: None,
            guard_animation_pose_slot: None,
            guard_prep_weapon_flags_pending_slot: None,
            mini_moldorm_history: None,
            initialize_reset_properties: None,
            initialize_load_properties: None,
            fire_debirando_property_reload: false,
            fire_debirando_before_spawn_slot: None,
            fire_debirando_spawn: None,
            antfairy_subtype2_increment_slot: None,
            lanmola_subtype2_increment_slot: None,
            helmasaur_hard_hat_beetle_subtype2_increment_slot: None,
            timer_decrements_slot: None,
            primary_timer_decrements_slot: None,
            hit_timer_slot: None,
            main_and_aux1_timer_decrements_slot: None,
            main_timer_decrement_slot: None,
            zero_hit_timer_clear_slot: None,
            bari_before_random_slot: None,
            throwable_scenery_state_clear_slot: None,
            cucco_subtype_increments: None,
            cucco_helper_ordinal: 0,
            cucco_flee_movement: None,
            active_cucco_movement: None,
            active_cucco_x_publications: 0,
            active_cucco_y_subpixel: None,
            master_sword_light_beam_movement: None,
            master_sword_light_beam_spawn: None,
            cucco_animation_slot: None,
            big_key_drop_graphics_slot: None,
            king_zora_flippers_graphics_slot: None,
            bonk_item_graphics_slot: None,
            wish_pond_tossed_item_graphics_slot: None,
            single_small_draw_position_slot: None,
            probe_after_oam_coordinates_slot: None,
            wallmaster_reset_prefix_slot: None,
            wallmaster_reset_cleared_bytes: None,
            zazak_graphics_slot: None,
            follower_graphics: None,
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
    fn king_zora_flippers_decoder_entry_becomes_a_typed_partial_slot_checkpoint() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(14), None),
                &mut receipts,
            )
            .unwrap();
        let mut graphics_entry = raw("pc", Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC), None, None);
        graphics_entry.return_address = Some(ZORA_FLIPPERS_GRAPHICS_RETURN_ADDRESS);
        source.consume_event(graphics_entry, &mut receipts).unwrap();

        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::KingZoraFlippersGraphicsStarted(14),
            )],
        );
        assert_eq!(
            source
                .sprite_main_execution
                .map(|execution| execution.interruption()),
            Some(MainLoopInterruption::SpriteMainKingZoraFlippersGraphicsStarted(14)),
        );
    }

    #[test]
    fn bonk_item_decoder_entry_becomes_a_typed_partial_slot_checkpoint() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
                &mut receipts,
            )
            .unwrap();
        let mut graphics_entry = raw("pc", Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC), None, None);
        graphics_entry.return_address = Some(BONK_ITEM_GRAPHICS_RETURN_ADDRESS);
        source.consume_event(graphics_entry, &mut receipts).unwrap();

        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::BonkItemGraphicsStarted(0),
            )],
        );
        assert_eq!(
            source
                .sprite_main_execution
                .map(|execution| execution.interruption()),
            Some(MainLoopInterruption::SpriteMainBonkItemGraphicsStarted(0)),
        );
    }

    #[test]
    fn wish_pond_tossed_item_decoder_entry_retains_the_spawned_prefix() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
                &mut receipts,
            )
            .unwrap();
        let mut graphics_entry = raw("pc", Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC), None, None);
        graphics_entry.return_address = Some(WISH_POND_TOSSED_ITEM_GRAPHICS_RETURN_ADDRESS);
        source.consume_event(graphics_entry, &mut receipts).unwrap();

        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::WishPondTossedItemGraphicsStarted(0),
            )],
        );
        assert_eq!(
            source
                .sprite_main_execution
                .map(|execution| execution.interruption()),
            Some(MainLoopInterruption::SpriteMainWishPondTossedItemGraphicsStarted(0)),
        );
    }

    #[test]
    fn shared_animated_decode_without_king_zora_return_address_is_not_flippers_progress() {
        let mut source = empty_semantic_tracker();
        source.sprite_main_execution = Some(SpriteMainExecutionTracker {
            current_slot: Some(14),
            ..SpriteMainExecutionTracker::default()
        });
        let mut receipts = Vec::new();
        let mut graphics_entry = raw("pc", Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC), None, None);
        graphics_entry.return_address = Some(0x1d_e000);

        source.consume_event(graphics_entry, &mut receipts).unwrap();

        assert_eq!(
            source
                .sprite_main_execution
                .map(|execution| execution.progress()),
            Some(SpriteMainProgress::BeforeFirstSlot),
        );
        assert!(receipts.is_empty());
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
    fn master_sword_light_beam_exports_every_move_xy_assignment_prefix() {
        let checkpoints = [
            SpriteMoveXYCheckpoint::BeforeMovement,
            SpriteMoveXYCheckpoint::AfterXSubpixel,
            SpriteMoveXYCheckpoint::AfterXLow,
            SpriteMoveXYCheckpoint::AfterXHigh,
            SpriteMoveXYCheckpoint::AfterYSubpixel,
            SpriteMoveXYCheckpoint::AfterYLow,
            SpriteMoveXYCheckpoint::AfterYHigh,
        ];
        let addresses = [
            SPRITE_X_SUBPIXEL_BASE + 2,
            SPRITE_X_LOW_BASE + 2,
            SPRITE_X_HIGH_BASE + 2,
            SPRITE_Y_SUBPIXEL_BASE + 2,
            SPRITE_Y_LOW_BASE + 2,
            SPRITE_Y_HIGH_BASE + 2,
        ];
        for (completed, checkpoint) in checkpoints.into_iter().enumerate() {
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
                    raw(
                        "pc",
                        Some(MASTER_SWORD_LIGHT_BEAM_MOVEMENT_CALL_PC),
                        Some(2),
                        None,
                    ),
                    &mut receipts,
                )
                .unwrap();
            for &address in &addresses[..completed] {
                source
                    .consume_event(
                        raw("wram-write", Some(0x05_fa00), Some(2), Some(address)),
                        &mut receipts,
                    )
                    .unwrap();
            }
            source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
            assert_eq!(
                receipts,
                vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::MasterSwordLightBeamMovement {
                        slot: 2,
                        checkpoint,
                    },
                )],
            );
        }
    }

    #[test]
    fn master_sword_light_beam_zero_x_velocity_starts_at_y_assignment() {
        let y_addresses = [
            SPRITE_Y_SUBPIXEL_BASE + 10,
            SPRITE_Y_LOW_BASE + 10,
            SPRITE_Y_HIGH_BASE + 10,
        ];
        let checkpoints = [
            SpriteMoveXYCheckpoint::AfterYSubpixel,
            SpriteMoveXYCheckpoint::AfterYLow,
            SpriteMoveXYCheckpoint::AfterYHigh,
        ];

        for (completed, checkpoint) in (1..=3).zip(checkpoints) {
            let mut source = empty_semantic_tracker();
            let mut receipts = Vec::new();
            for event in [
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(10), None),
                raw(
                    "pc",
                    Some(MASTER_SWORD_LIGHT_BEAM_MOVEMENT_CALL_PC),
                    Some(10),
                    None,
                ),
            ] {
                source.consume_event(event, &mut receipts).unwrap();
            }
            for &address in &y_addresses[..completed] {
                source
                    .consume_event(
                        raw("wram-write", Some(0x05_fa00), Some(10), Some(address)),
                        &mut receipts,
                    )
                    .unwrap();
            }
            source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
            assert_eq!(
                receipts,
                vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::MasterSwordLightBeamMovement {
                        slot: 10,
                        checkpoint,
                    },
                )],
            );
        }
    }

    #[test]
    fn master_sword_replacement_spawn_exports_shared_helper_prefix() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(5), None),
            raw(
                "pc",
                Some(MASTER_SWORD_LIGHT_BEAM_MOVEMENT_CALL_PC),
                Some(5),
                None,
            ),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }

        let mut type_write = raw(
            "wram-write",
            Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC),
            Some(5),
            Some(SPRITE_TYPE_BASE + 3),
        );
        type_write.y = Some(3);
        type_write.value = Some(0x62);
        source.consume_event(type_write, &mut receipts).unwrap();

        let mut state_write = raw(
            "wram-write",
            Some(SPRITE_SPAWN_DYNAMICALLY_STATE_STORE_PC),
            Some(5),
            Some(SPRITE_STATE_BASE + 3),
        );
        state_write.y = Some(3);
        state_write.value = Some(9);
        source.consume_event(state_write, &mut receipts).unwrap();

        source
            .consume_event(
                raw("wram-write", Some(0x0d_b877), Some(3), Some(0x0e93)),
                &mut receipts,
            )
            .unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::MasterSwordLightBeamSpawn {
                    slot: 5,
                    spawned_slot: 3,
                    progress: SpriteDynamicSpawnProgress::ResetProperties {
                        completed_stores: 2,
                    },
                },
            )],
        );
    }

    #[test]
    fn dynamic_spawn_outdoor_identity_publishes_after_atomic_word_store() {
        let mut tracker = Some((5, 4, SpriteDynamicSpawnProgress::StatePublished));
        let mut low = raw(
            "wram-write",
            Some(SPRITE_SPAWN_DYNAMICALLY_IDENTITY_STORE_PC),
            Some(4),
            Some(SPRITE_N_BASE + 8),
        );
        low.value = Some(0xff);
        observe_dynamic_spawn_progress_write(&mut tracker, &low, "test").unwrap();
        assert_eq!(
            tracker,
            Some((5, 4, SpriteDynamicSpawnProgress::StatePublished)),
        );

        let mut high = raw(
            "wram-write",
            Some(SPRITE_SPAWN_DYNAMICALLY_IDENTITY_STORE_PC),
            Some(4),
            Some(SPRITE_N_BASE + 9),
        );
        high.value = Some(0xff);
        observe_dynamic_spawn_progress_write(&mut tracker, &high, "test").unwrap();
        assert_eq!(
            tracker,
            Some((5, 4, SpriteDynamicSpawnProgress::IdentityPublished)),
        );
    }

    #[test]
    fn single_small_draw_nmi_exports_the_published_position_prefix() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(15), None),
            raw(
                "nmi",
                Some(SPRITE_SINGLE_SMALL_AFTER_POSITION_PC),
                Some(15),
                None,
            ),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterSingleSmallDrawPosition(15),
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::AfterSingleSmallDrawPosition(15),
                ),
            ],
        );
    }

    #[test]
    fn guard_probe_nmi_exports_the_completed_oam_coordinate_prefix() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(8), None),
            raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(8), None),
            raw(
                "nmi",
                Some(SPRITE_PROBE_AFTER_OAM_COORDINATES_PC),
                Some(8),
                None,
            ),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainProbeAfterOamCoordinates(8),
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::ProbeAfterOamCoordinates(8),
                ),
            ],
        );
    }

    #[test]
    fn state8_property_reset_nmi_exports_the_exact_completed_store_prefix() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(1), None),
                &mut receipts,
            )
            .unwrap();
        let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(1), None);
        timers_return.stack1 = Some(8);
        source.consume_event(timers_return, &mut receipts).unwrap();
        let mut nmi = raw("nmi", Some(0x0db8ad), Some(1), None);
        nmi.return_address = Some(SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_RETURN_ADDRESS);
        source.consume_event(nmi, &mut receipts).unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainInitializeResetProperties {
                        slot: 1,
                        phase: SpriteInitializeResetPropertiesPhase::InitialPropertyLoad,
                        completed_stores: 20,
                    },
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::InitializeResetProperties {
                        slot: 1,
                        phase: SpriteInitializeResetPropertiesPhase::InitialPropertyLoad,
                        completed_stores: 20,
                    },
                ),
            ],
        );
    }

    #[test]
    fn fire_debirando_nested_property_reset_exports_its_source_phase() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(1), None),
                &mut receipts,
            )
            .unwrap();
        let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(1), None);
        timers_return.stack1 = Some(8);
        source.consume_event(timers_return, &mut receipts).unwrap();
        let mut conversion = raw(
            "wram-write",
            Some(SPRITE_PREP_FIRE_DEBIRANDO_TYPE_STORE_PC),
            Some(1),
            Some(SPRITE_TYPE_BASE + 1),
        );
        conversion.value = Some(0x63);
        source.consume_event(conversion, &mut receipts).unwrap();
        let mut nmi = raw("nmi", Some(0x0db874), Some(1), None);
        nmi.return_address = Some(SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_RETURN_ADDRESS);
        source.consume_event(nmi, &mut receipts).unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainInitializeResetProperties {
                        slot: 1,
                        phase: SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion,
                        completed_stores: 1,
                    },
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::InitializeResetProperties {
                        slot: 1,
                        phase: SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion,
                        completed_stores: 1,
                    },
                ),
            ],
        );
    }

    #[test]
    fn fire_debirando_property_load_exports_its_source_store_cursor() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
                &mut receipts,
            )
            .unwrap();
        let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(0), None);
        timers_return.stack1 = Some(8);
        source.consume_event(timers_return, &mut receipts).unwrap();
        let mut conversion = raw(
            "wram-write",
            Some(SPRITE_PREP_FIRE_DEBIRANDO_TYPE_STORE_PC),
            Some(0),
            Some(SPRITE_TYPE_BASE),
        );
        conversion.value = Some(0x63);
        source.consume_event(conversion, &mut receipts).unwrap();
        let returned = raw(
            "frame",
            Some(SPRITE_PREP_LOAD_PROPERTIES_AFTER_FLAGS3_PC),
            Some(0),
            None,
        );
        source
            .sprite_main_execution
            .as_mut()
            .unwrap()
            .observe_initialize_load_properties(&returned)
            .unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::InitializeLoadProperties {
                    slot: 0,
                    phase: SpriteInitializeResetPropertiesPhase::FireDebirandoTypeConversion,
                    completed_stores: 9,
                },
            )],
        );
    }

    #[test]
    fn fire_debirando_spawn_scan_exports_the_completed_initializer_prefix() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
                &mut receipts,
            )
            .unwrap();
        let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(0), None);
        timers_return.stack1 = Some(8);
        source.consume_event(timers_return, &mut receipts).unwrap();
        let mut conversion = raw(
            "wram-write",
            Some(SPRITE_PREP_FIRE_DEBIRANDO_TYPE_STORE_PC),
            Some(0),
            Some(SPRITE_TYPE_BASE),
        );
        conversion.value = Some(0x63);
        source.consume_event(conversion, &mut receipts).unwrap();
        let mut returned = raw(
            "frame",
            Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC),
            Some(0),
            None,
        );
        returned.return_address = Some(SPRITE_PREP_FIRE_DEBIRANDO_SPAWN_RETURN_ADDRESS);
        source
            .sprite_main_execution
            .as_mut()
            .unwrap()
            .observe_fire_debirando_before_spawn(&returned)
            .unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::FireDebirandoBeforeSpawn(0),
            )],
        );
    }

    #[test]
    fn fire_debirando_dynamic_spawn_exports_the_exact_source_publication() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
                &mut receipts,
            )
            .unwrap();
        let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(0), None);
        timers_return.stack1 = Some(8);
        source.consume_event(timers_return, &mut receipts).unwrap();
        let mut conversion = raw(
            "wram-write",
            Some(SPRITE_PREP_FIRE_DEBIRANDO_TYPE_STORE_PC),
            Some(0),
            Some(SPRITE_TYPE_BASE),
        );
        conversion.value = Some(0x63);
        source.consume_event(conversion, &mut receipts).unwrap();

        let mut child_type = raw(
            "wram-write",
            Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC),
            Some(0),
            Some(SPRITE_TYPE_BASE + 15),
        );
        child_type.y = Some(15);
        child_type.value = Some(0x64);
        source.consume_event(child_type, &mut receipts).unwrap();
        let mut child_state = raw(
            "wram-write",
            Some(SPRITE_SPAWN_DYNAMICALLY_STATE_STORE_PC),
            Some(0),
            Some(SPRITE_STATE_BASE + 15),
        );
        child_state.y = Some(15);
        child_state.value = Some(9);
        source.consume_event(child_state, &mut receipts).unwrap();
        let mut child_floor = raw(
            "wram-write",
            Some(SPRITE_SPAWN_DYNAMICALLY_FLOOR_STORE_PC),
            Some(0),
            Some(SPRITE_FLOOR_BASE + 15),
        );
        child_floor.y = Some(15);
        source.consume_event(child_floor, &mut receipts).unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::FireDebirandoSpawn {
                    slot: 0,
                    spawned_slot: 15,
                    progress: SpriteDynamicSpawnProgress::FloorPublished,
                },
            )],
        );
    }

    #[test]
    fn timer_oam_return_nmi_exports_the_generic_completed_prefix() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(10), None),
            raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(10), None),
            raw("nmi", Some(0x069276), Some(10), None),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterTimersAndOam(10),
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::AfterTimersAndOam(10),
                ),
            ],
        );
    }

    #[test]
    fn mini_moldorm_nmi_exports_exact_history_store_progress() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
                &mut receipts,
            )
            .unwrap();
        let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(0), None);
        timers_return.stack1 = Some(8);
        source.consume_event(timers_return, &mut receipts).unwrap();
        let mut nmi = raw(
            "nmi",
            Some(SPRITE_PREP_MINI_MOLDORM_HISTORY_X_HIGH_LOAD_PC),
            Some(24),
            None,
        );
        nmi.y = Some(0);
        source.consume_event(nmi, &mut receipts).unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        let progress = SpriteMainProgress::MiniMoldormHistory {
            slot: 0,
            completed_stores: 99,
        };
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainMiniMoldormHistory {
                        slot: 0,
                        completed_stores: 99,
                    },
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(progress),
            ],
        );
    }

    #[test]
    fn guard_prep_host_return_exports_the_first_active_calls_pending_weapon_flags() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(12), None),
                &mut receipts,
            )
            .unwrap();
        let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(12), None);
        timers_return.stack1 = Some(8);
        source.consume_event(timers_return, &mut receipts).unwrap();
        source
            .consume_event(
                raw("pc", Some(SPRITE_ACTIVE_MAIN_ENTRY_PC), Some(12), None),
                &mut receipts,
            )
            .unwrap();
        let mut weapon_flags_store = raw(
            "wram-write",
            Some(GUARD_ANIMATE_WEAPON_FLAGS_STORE_PC),
            Some(0),
            Some(0x0803),
        );
        weapon_flags_store.sub = Some(0);
        source
            .consume_event(weapon_flags_store, &mut receipts)
            .unwrap();

        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::GuardPrepWeaponFlagsPending(12),
            )],
        );
    }

    #[test]
    fn active_guard_weapon_nmi_exports_the_unfinished_entry() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(10), None),
                &mut receipts,
            )
            .unwrap();
        let mut timers_return = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(10), None);
        timers_return.stack1 = Some(9);
        source.consume_event(timers_return, &mut receipts).unwrap();
        let mut nmi = raw("nmi", Some(0x05_cbaa), Some(34), None);
        let mut pose = raw(
            "wram-write",
            Some(0x05_c240),
            Some(10),
            Some(SPRITE_GRAPHICS_BASE + 10),
        );
        pose.value = Some(8);
        source.consume_event(pose, &mut receipts).unwrap();
        nmi.y = Some(21);
        source.consume_event(nmi, &mut receipts).unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
        assert!(
            receipts.contains(&OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainGuardAnimation {
                    slot: 10,
                    checkpoint: zelda3::GuardAnimationCheckpoint::WeaponCoordinates { entry: 1 }
                }
            ))
        );
        assert!(
            receipts.contains(&OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::GuardAnimation {
                    slot: 10,
                    checkpoint: zelda3::GuardAnimationCheckpoint::WeaponCoordinates { entry: 1 }
                }
            ))
        );
    }

    #[test]
    fn guard_head_flags_boundary_requires_the_temporary_pose_caller() {
        let mut tracker = SpriteMainExecutionTracker {
            current_slot: Some(11),
            timers_and_oam_dispatch_state: Some(9),
            ..Default::default()
        };
        let mut event = raw("nmi", Some(0x05_c71c), Some(0), None);
        event.y = Some(2);
        tracker.observe_guard_animation_checkpoint(&event).unwrap();
        assert_eq!(tracker.guard_animation_checkpoint, None);
        tracker.guard_animation_pose_slot = Some(11);
        tracker.observe_guard_animation_checkpoint(&event).unwrap();
        assert_eq!(
            tracker.guard_animation_checkpoint,
            Some((11, zelda3::GuardAnimationCheckpoint::HeadFlagsPending))
        );
    }

    #[test]
    fn pengator_slide_entry_proves_movement_before_the_sparkle_rng() {
        let mut tracker = SpriteMainExecutionTracker {
            current_slot: Some(6),
            timers_and_oam_dispatch_state: Some(9),
            ..Default::default()
        };
        let mut event = raw("nmi", Some(0x1e_a279), None, None);
        event.x = Some(6);
        tracker.observe_pengator_slide_pending(&event).unwrap();
        assert_eq!(
            tracker.progress(),
            SpriteMainProgress::PengatorSlidePending(6)
        );
        event.x = Some(5);
        assert!(tracker.observe_pengator_slide_pending(&event).is_err());
    }

    #[test]
    fn kholdstare_damage_checkpoint_requires_body_and_hitbox_caller() {
        let mut tracker = SpriteMainExecutionTracker {
            current_slot: Some(4),
            timers_and_oam_dispatch_state: Some(9),
            ..Default::default()
        };
        let mut endpoint = raw("frame", Some(0x06_f839), None, None);
        endpoint.x = Some(1);
        endpoint.stack1 = Some(4);
        endpoint.return_address = Some(0xf2_d004);
        tracker
            .observe_kholdstare_damage_pending(&endpoint)
            .unwrap();
        assert_eq!(tracker.kholdstare_damage_pending, None);
        let mut decrement = raw("wram-write", Some(0x1e_953a), None, None);
        decrement.x = Some(4);
        decrement.address = Some(0x0e84);
        tracker
            .observe_kholdstare_damage_pending(&decrement)
            .unwrap();
        tracker
            .observe_kholdstare_damage_pending(&endpoint)
            .unwrap();
        assert_eq!(
            tracker.progress(),
            SpriteMainProgress::KholdstareDamagePending(4)
        );
        endpoint.stack1 = Some(3);
        assert!(tracker
            .observe_kholdstare_damage_pending(&endpoint)
            .is_err());
    }

    #[test]
    fn antifairy_bounce_checkpoint_supersedes_draw_progress_for_its_caller_only() {
        let mut tracker = SpriteMainExecutionTracker {
            current_slot: Some(0),
            timers_and_oam_dispatch_state: Some(9),
            antfairy_subtype2_increment_slot: Some(0),
            ..Default::default()
        };
        let mut event = raw("nmi", Some(0x1d_c778), None, None);
        event.x = Some(0);
        event.return_address = Some(0x06_a53d);
        tracker.observe_antifairy_bounce_pending(&event).unwrap();
        assert_eq!(tracker.antifairy_bounce_pending, None);
        event.return_address = Some(0x06_a53e);
        tracker.observe_antifairy_bounce_pending(&event).unwrap();
        assert_eq!(
            tracker.progress(),
            SpriteMainProgress::AntifairyBouncePending(0)
        );
        event.x = Some(1);
        assert!(tracker.observe_antifairy_bounce_pending(&event).is_err());
    }

    #[test]
    fn guard_draw_return_retains_pose_without_requiring_a_pose_store() {
        let mut tracker = SpriteMainExecutionTracker {
            current_slot: Some(10),
            timers_and_oam_dispatch_state: Some(9),
            ..Default::default()
        };
        let mut event = raw("nmi", Some(0x05_c243), None, None);
        event.x = Some(10);
        tracker.observe_guard_animation_checkpoint(&event).unwrap();
        assert_eq!(
            tracker.guard_animation_checkpoint,
            Some((10, zelda3::GuardAnimationCheckpoint::DrawReturned))
        );
        event.x = Some(9);
        assert!(tracker.observe_guard_animation_checkpoint(&event).is_err());
    }

    #[test]
    fn guard_draw_cursors_distinguish_body_and_weapon_store_prefixes() {
        use zelda3::GuardAnimationCheckpoint as Stage;
        for (pc, x, y, expected) in [
            (0x05_c711, 0, 1, Stage::HeadCharacterPending),
            (0x05_c713, 0, 1, Stage::HeadCharacterPending),
            (0x05_ca71, 33, 13, Stage::BodyCoordinates { entry: 1 }),
            (0x05_ca74, 33, 13, Stage::BodyCoordinates { entry: 1 }),
            (0x05_ca77, 35, 6, Stage::BodyFlagsPending { entry: 3 }),
            (0x05_ca8e, 34, 10, Stage::BodyFlagsPending { entry: 2 }),
            (0x05_ca91, 34, 10, Stage::BodyFlagsPending { entry: 2 }),
            (
                0x05_cb8c,
                32,
                24,
                Stage::WeaponBeforeCoordinates { entry: 0 },
            ),
            (0x05_c721, 0, 3, Stage::HeadExtendedPending),
            (0x05_c724, 0, 3, Stage::HeadExtendedPending),
            (0x05_c725, 0, 0, Stage::HeadExtendedPending),
            (0x05_c729, 0, 0, Stage::HeadExtendedPending),
            (0x05_c717, 0, 2, Stage::HeadCharacterPending),
            (0x05_ca9e, 35, 6, Stage::BodyFlagsPending { entry: 3 }),
            (0x05_ca29, 33, 12, Stage::BodyBeforeEntry { entry: 1 }),
            (0x05_ca43, 66, 12, Stage::BodyBeforeEntry { entry: 1 }),
            (0x05_ca4b, 66, 12, Stage::BodyBeforeEntry { entry: 1 }),
            (0x05_ca6b, 66, 13, Stage::BodyCoordinates { entry: 1 }),
            (0x05_ca96, 34, 10, Stage::BodyFlagsPending { entry: 2 }),
            (0x05_ca9f, 35, 7, Stage::BodyFlagsPending { entry: 3 }),
            (
                0x05_cb86,
                32,
                24,
                Stage::WeaponBeforeCoordinates { entry: 0 },
            ),
            (0x05_cbaa, 34, 21, Stage::WeaponCoordinates { entry: 1 }),
        ] {
            let mut tracker = SpriteMainExecutionTracker {
                current_slot: Some(11),
                timers_and_oam_dispatch_state: Some(9),
                guard_animation_pose_slot: Some(11),
                ..Default::default()
            };
            let mut event = raw("nmi", Some(pc), Some(x), None);
            event.y = Some(y);
            tracker.observe_guard_animation_checkpoint(&event).unwrap();
            assert_eq!(tracker.guard_animation_checkpoint, Some((11, expected)));
        }
    }

    #[test]
    fn guard_initializer_parry_nmi_preserves_the_nested_call_ordinal() {
        // Pinned-ROM host 279816: slot 10 dispatches state 8, reaches
        // $069271 twice, and accepts NMI at $06EB94 during call two.
        assert_eq!(GUARD_PARRY_HITBOX_COMPARE_PC, 0x06eb94);
        for (active_call, pc) in [(1, 0x06eb92), (2, 0x06eb92), (1, 0x06eb94), (2, 0x06eb94)] {
            let mut source = empty_semantic_tracker();
            let mut receipts = Vec::new();
            for event in [
                raw("pc", Some(0x068328), None, None),
                raw("pc", Some(0x0684e2), Some(10), None),
            ] {
                source.consume_event(event, &mut receipts).unwrap();
            }
            let mut timers = raw("pc", Some(SPRITE_TIMERS_AND_OAM_RETURN_PC), Some(10), None);
            timers.stack1 = Some(8);
            source.consume_event(timers, &mut receipts).unwrap();
            for _ in 0..active_call {
                source
                    .consume_event(raw("pc", Some(0x069271), Some(10), None), &mut receipts)
                    .unwrap();
            }
            source
                .consume_event(raw("nmi", Some(pc), Some(10), None), &mut receipts)
                .unwrap();
            source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
            assert_eq!(
                receipts,
                vec![
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(
                        MainLoopInterruption::SpriteMainGuardPrepParryHitbox {
                            slot: 10,
                            active_call
                        }
                    ),
                    OriginalTimingSemanticReceipt::SpriteMainProgressed(
                        SpriteMainProgress::GuardPrepParryHitbox {
                            slot: 10,
                            active_call
                        }
                    ),
                ]
            );
        }
    }

    #[test]
    fn timer_decrement_nmi_exports_the_completed_countdown_prefix() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
            // The route frontier returns from retro_run at `$06:84A8`, after
            // countdowns and before the suffix's conditional floor branch.
            raw(
                "nmi",
                Some(SPRITE_TIMER_DECREMENTS_COMPLETE_START_PC + 4),
                Some(0),
                None,
            ),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterTimerDecrements(0),
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::AfterTimerDecrements(0),
                ),
            ],
        );
    }

    #[test]
    fn hit_timer_nmi_retains_the_pending_aux4_update() {
        for pc in [0x06_849c, 0x06_849f, 0x06_84a1] {
            let mut source = empty_semantic_tracker();
            let mut receipts = Vec::new();

            for event in [
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(0), None),
                // The aux4 load follows the completed hit-timer statement.
                raw("nmi", Some(pc), Some(0), None),
            ] {
                source.consume_event(event, &mut receipts).unwrap();
            }
            source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

            assert_eq!(
                receipts,
                vec![
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                    OriginalTimingSemanticReceipt::MainLoopInterrupted(
                        MainLoopInterruption::SpriteMainAfterHitTimer(0),
                    ),
                    OriginalTimingSemanticReceipt::SpriteMainProgressed(
                        SpriteMainProgress::AfterHitTimer(0),
                    ),
                ],
            );
        }
    }

    #[test]
    fn selected_game_entrance_return_requires_its_module05_caller() {
        for (main, caller, expected) in [
            (5, Some(0x00_8059), true),
            (7, Some(0x00_8059), false),
            (5, Some(0x02_85ad), false),
            (5, None, false),
        ] {
            let mut source = empty_semantic_tracker();
            let mut receipts = Vec::new();
            let mut event = raw("pc", Some(0x02_824d), None, None);
            event.main = Some(main);
            event.sub = Some(0);
            event.return_address = caller;
            source.consume_event(event, &mut receipts).unwrap();
            assert_eq!(
                receipts,
                if expected {
                    vec![OriginalTimingSemanticReceipt::SelectedGameEntranceReturned]
                } else {
                    vec![]
                }
            );
        }
    }

    #[test]
    fn zero_hit_timer_branch_nmi_exports_the_primary_countdown_prefix() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(14), None),
            // The zero branch skipped the linear hit-timer interval and the
            // host ended while fetching the first clear instruction.
            raw(
                "nmi",
                Some(SPRITE_PRIMARY_TIMER_DECREMENTS_ZERO_HIT_STORE_START_PC),
                Some(14),
                None,
            ),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterPrimaryTimerDecrements(14),
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::AfterPrimaryTimerDecrements(14),
                ),
            ],
        );
    }

    #[test]
    fn wallmaster_reset_nmi_exports_the_fixed_reset_prefix() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(12), None),
                &mut receipts,
            )
            .unwrap();
        let mut nmi = raw(
            "nmi",
            Some(WALLMASTER_RESET_AFTER_FIXED_PREFIX_PC),
            Some(0xfff),
            None,
        );
        nmi.return_address = Some(WALLMASTER_AFTER_SPRITE_RESET_PC);
        source.consume_event(nmi, &mut receipts).unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterWallmasterResetPrefix(12),
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::AfterWallmasterResetPrefix(12),
                ),
            ],
        );
    }

    #[test]
    fn wallmaster_reset_nmi_exports_the_completed_descending_clear() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(12), None),
                &mut receipts,
            )
            .unwrap();
        let mut nmi = raw("nmi", Some(0x09_c47f), Some(0x039e), None);
        nmi.return_address = Some(WALLMASTER_AFTER_SPRITE_RESET_PC);
        source.consume_event(nmi, &mut receipts).unwrap();
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainWallmasterResetClear {
                        slot: 12,
                        cleared_bytes: 3170
                    },
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::WallmasterResetClear {
                        slot: 12,
                        cleared_bytes: 3170
                    },
                ),
            ],
        );
    }

    #[test]
    fn sprite_main_host_return_exports_throwable_scenery_state_clear() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(7), None),
            raw("pc", Some(SPRITE_SLOT_RETURN_PC), Some(7), None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(6), None),
            raw(
                "wram-write",
                Some(THROWABLE_SCENERY_STATE_CLEAR_PC),
                Some(6),
                Some(SPRITE_STATE_BASE + 6),
            ),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }

        let checkpoint =
            serde_json::from_slice(&serde_json::to_vec(&source.checkpoint()).unwrap()).unwrap();
        let mut resumed = empty_semantic_tracker();
        resumed.restore_checkpoint(checkpoint).unwrap();
        resumed.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterThrowableSceneryStateClear(6),
            )],
        );
    }

    #[test]
    fn sprite_main_nmi_exports_throwable_scenery_state_clear() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(6), None),
            raw(
                "wram-write",
                Some(THROWABLE_SCENERY_STATE_CLEAR_PC),
                Some(6),
                Some(SPRITE_STATE_BASE + 6),
            ),
            raw("nmi", Some(0x06_e465), Some(6), None),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterThrowableSceneryStateClear(6),
                ),
            ],
        );
    }

    #[test]
    fn cached_sprite_nmi_exports_antfairy_subtype_increment() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for &address in &CACHED_SPRITE_LIVE_FIELDS {
            source
                .consume_event(
                    raw(
                        "wram-write",
                        Some(UNCACHE_SPRITE_START_PC),
                        Some(1),
                        Some(address + 1),
                    ),
                    &mut receipts,
                )
                .unwrap();
        }
        source
            .consume_event(
                raw(
                    "wram-write",
                    Some(ANTFAIRY_SUBTYPE2_INCREMENT_PC),
                    Some(1),
                    Some(SPRITE_SUBTYPE2_BASE + 1),
                ),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(
                raw("nmi", Some(0x05_dfb5), Some(0x08e8), None),
                &mut receipts,
            )
            .unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::CachedSpriteExecutionProgress(
                    CachedSpriteExecutionProgressReceipt {
                        progress: CachedSpriteExecutionProgress::Executing {
                            slot: 1,
                            progress:
                                CachedSpriteExecutionBodyProgress::AfterAntfairySubtype2Increment,
                        },
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ],
        );
    }

    #[test]
    fn sprite_main_nmi_exports_antfairy_subtype_increment() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(1), None),
            raw(
                "wram-write",
                Some(ANTFAIRY_SUBTYPE2_INCREMENT_PC),
                Some(1),
                Some(SPRITE_SUBTYPE2_BASE + 1),
            ),
            raw("nmi", Some(0x06_e465), Some(1), None),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterAntfairySubtype2Increment(1),
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::AfterAntfairySubtype2Increment(1),
                ),
            ],
        );
    }

    #[test]
    fn sprite_main_nmi_exports_lanmola_subtype_increment() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(2), None),
            raw(
                "wram-write",
                Some(LANMOLA_SUBTYPE2_INCREMENT_PC),
                Some(2),
                Some(SPRITE_SUBTYPE2_BASE + 2),
            ),
            raw("nmi", Some(0x05_a6c0), Some(2), None),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::SpriteMainAfterLanmolaSubtype2Increment(2),
                ),
                OriginalTimingSemanticReceipt::SpriteMainProgressed(
                    SpriteMainProgress::AfterLanmolaSubtype2Increment(2),
                ),
            ],
        );
    }

    #[test]
    fn initializer_dispatch_jump_table_requires_its_source_caller() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(1), None),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        let execution = source.sprite_main_execution.as_mut().unwrap();
        execution.timers_and_oam_dispatch_state = Some(8);
        let mut event = raw("nmi", Some(0x008781), Some(1), None);
        event.return_address = Some(0x06865a);
        execution.observe_initialize_prep_pending(&event).unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::InitializePrepPending(1)
        );
        execution.initialize_prep_pending = None;
        event.return_address = Some(0x068659);
        execution.observe_initialize_prep_pending(&event).unwrap();
        assert_eq!(execution.initialize_prep_pending, None);
        event.pc = Some(0x06_91b4);
        event.return_address = Some(0x00_83a6);
        execution.observe_initialize_prep_pending(&event).unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::InitializePrepPending(1)
        );
    }

    #[test]
    fn falling_palette_direction_checkpoint_is_bound_to_its_caller() {
        for (caller, expected) in [(0x02_8ea4, true), (0x02_8ea3, false)] {
            let mut source = empty_semantic_tracker();
            let mut receipts = Vec::new();
            let mut event = raw("nmi", Some(0x00_e9bc), Some(480), None);
            event.main = Some(7);
            event.sub = Some(7);
            event.subsub = Some(15);
            event.return_address = Some(caller);
            event.nmi_latch = Some(1);
            source.consume_event(event, &mut receipts).unwrap();
            assert_eq!(
                receipts.contains(
                    &OriginalTimingSemanticReceipt::DungeonFallingFadeInPaletteDirectionToggled
                ),
                expected
            );
        }
    }

    #[test]
    fn guard_patrol_endpoint_retains_the_initializer_active_call() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(9), None),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        let execution = source.sprite_main_execution.as_mut().unwrap();
        execution.timers_and_oam_dispatch_state = Some(8);
        let event = raw("nmi", Some(0x05c415), Some(9), None);
        for active_call in 1..=2 {
            execution.initialize_active_main_calls = active_call;
            execution.observe_guard_prep_patrol_delay(&event).unwrap();
            assert_eq!(
                execution.progress(),
                SpriteMainProgress::GuardPrepPatrolDelay {
                    slot: 9,
                    active_call
                }
            );
        }
        execution.initialize_active_main_calls = 0;
        assert!(execution.observe_guard_prep_patrol_delay(&event).is_err());
        execution.timers_and_oam_dispatch_state = Some(9);
        execution.guard_prep_patrol_delay = None;
        execution.observe_guard_prep_patrol_delay(&event).unwrap();
        assert_eq!(execution.guard_prep_patrol_delay, None);
    }

    #[test]
    fn hog_spear_animation_endpoint_requires_the_exact_two_byte_caller() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(13), None),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        let execution = source.sprite_main_execution.as_mut().unwrap();
        let mut event = raw("frame", Some(0x05c469), Some(13), None);
        event.return_address = Some(0xc8cc3a);
        execution
            .observe_hog_spear_body_graphics_pending(&event)
            .unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::HogSpearBodyGraphicsPending(13)
        );
        execution.hog_spear_body_graphics_pending = None;
        execution.absorbable_body_active = false;
        execution.absorbable_horizontal_lookup = None;
        execution.absorbable_vertical_lookup = None;
        execution.absorbable_vertical_attribute_loaded = None;
        execution.dispatch_trampoline_return = None;
        execution.vitreous_minions_seen = false;
        execution.vitreous_player_damage_pending = None;
        execution.vitreous_ai_pending = None;
        execution.vitreous_damage_pending = None;
        execution.swamola_segment = None;
        execution.swamola_head_prepared = false;
        execution.swamola_head_draw_completed = None;
        execution.swamola_head_draw = None;
        execution.swamola_segment_draw = None;
        execution.pengator_slide_pending = None;
        execution.antifairy_bounce_pending = None;
        execution.kholdstare_subtype_decremented = false;
        execution.kholdstare_damage_pending = None;
        execution.initialize_prep_pending = None;
        event.return_address = Some(0xc8cc3b);
        execution
            .observe_hog_spear_body_graphics_pending(&event)
            .unwrap();
        assert_eq!(execution.hog_spear_body_graphics_pending, None);
    }

    #[test]
    fn guard_initializer_tile_collision_requires_its_nested_call_and_wrapper() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        tracker
            .consume_event(
                raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
                &mut receipts,
            )
            .unwrap();
        tracker
            .consume_event(
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(13), None),
                &mut receipts,
            )
            .unwrap();
        let execution = tracker.sprite_main_execution.as_mut().unwrap();
        execution.timers_and_oam_dispatch_state = Some(8);
        let mut event = raw("frame", Some(0x06_e49d), Some(13), None);
        event.return_address = Some(0x05_b890);
        for active_call in 1..=2 {
            execution.initialize_active_main_calls = active_call;
            execution
                .observe_guard_prep_tile_collision_return(&event)
                .unwrap();
            assert_eq!(
                execution.progress(),
                SpriteMainProgress::GuardPrepTileCollisionReturned {
                    slot: 13,
                    active_call
                }
            );
        }
        execution.initialize_active_main_calls = 0;
        assert!(execution
            .observe_guard_prep_tile_collision_return(&event)
            .is_err());
        event.return_address = Some(0);
        execution.guard_prep_tile_collision_return = None;
        execution
            .observe_guard_prep_tile_collision_return(&event)
            .unwrap();
        assert_eq!(execution.guard_prep_tile_collision_return, None);
    }

    #[test]
    fn absorbable_lookup_checkpoint_requires_body_and_horizontal_direction() {
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
                raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(3), None),
                &mut receipts,
            )
            .unwrap();
        source
            .consume_event(raw("pc", Some(0x06_d051), Some(3), None), &mut receipts)
            .unwrap();
        let execution = source.sprite_main_execution.as_mut().unwrap();
        let mut event = raw("nmi", Some(0x00_8872), Some(170), None);
        event.return_address = Some(0x06_e8cd);
        event.y = Some(6);
        execution.observe_absorbable_tile_lookup(&event).unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::AbsorbableHorizontalTileLookup(3)
        );
        execution.absorbable_horizontal_lookup = None;
        execution.absorbable_vertical_lookup = None;
        execution.absorbable_vertical_attribute_loaded = None;
        execution.dispatch_trampoline_return = None;
        execution.vitreous_minions_seen = false;
        execution.vitreous_player_damage_pending = None;
        execution.vitreous_ai_pending = None;
        execution.vitreous_damage_pending = None;
        execution.swamola_segment = None;
        execution.swamola_head_prepared = false;
        execution.swamola_head_draw_completed = None;
        execution.swamola_head_draw = None;
        execution.swamola_segment_draw = None;
        execution.pengator_slide_pending = None;
        execution.antifairy_bounce_pending = None;
        execution.kholdstare_subtype_decremented = false;
        execution.kholdstare_damage_pending = None;
        event.y = Some(0);
        execution.observe_absorbable_tile_lookup(&event).unwrap();
        assert_eq!(execution.absorbable_horizontal_lookup, None);
        assert_eq!(execution.absorbable_vertical_lookup, Some(3));
        execution.absorbable_vertical_lookup = None;
        execution.absorbable_vertical_attribute_loaded = None;
        execution.dispatch_trampoline_return = None;
        execution.vitreous_minions_seen = false;
        execution.vitreous_player_damage_pending = None;
        execution.vitreous_ai_pending = None;
        execution.vitreous_damage_pending = None;
        execution.swamola_segment = None;
        execution.swamola_head_prepared = false;
        execution.swamola_head_draw_completed = None;
        execution.swamola_head_draw = None;
        execution.swamola_segment_draw = None;
        event.pc = Some(0x06_e782);
        event.return_address = Some(0x03_e5f0);
        event.x = Some(3);
        event.y = Some(8);
        execution.observe_absorbable_tile_lookup(&event).unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::AbsorbableVerticalTileLookup(3)
        );
        execution.absorbable_vertical_lookup = None;
        execution.absorbable_vertical_attribute_loaded = None;
        execution.dispatch_trampoline_return = None;
        execution.vitreous_minions_seen = false;
        execution.vitreous_player_damage_pending = None;
        execution.vitreous_ai_pending = None;
        execution.vitreous_damage_pending = None;
        execution.swamola_segment = None;
        execution.swamola_head_prepared = false;
        execution.swamola_head_draw_completed = None;
        execution.swamola_head_draw = None;
        execution.swamola_segment_draw = None;
        event.return_address = Some(0x03_e5f1);
        execution.observe_absorbable_tile_lookup(&event).unwrap();
        assert_eq!(execution.absorbable_vertical_lookup, None);
        event.pc = Some(0x06_e812);
        event.return_address = Some(0x03_e5f0);
        execution.observe_absorbable_tile_lookup(&event).unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::AbsorbableVerticalTileAttributeLoaded(3)
        );
        execution.absorbable_vertical_attribute_loaded = None;
        execution.dispatch_trampoline_return = None;
        execution.vitreous_minions_seen = false;
        execution.vitreous_player_damage_pending = None;
        execution.vitreous_ai_pending = None;
        execution.vitreous_damage_pending = None;
        execution.swamola_segment = None;
        execution.swamola_head_prepared = false;
        execution.swamola_head_draw_completed = None;
        execution.swamola_head_draw = None;
        execution.swamola_segment_draw = None;
        event.pc = Some(0x06_e883);
        event.return_address = Some(0xba_e7a0);
        event.y = Some(14);
        execution.observe_absorbable_tile_lookup(&event).unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::AbsorbableHorizontalTileLookup(3)
        );
    }

    #[test]
    fn sprite_main_host_return_exports_helmasaur_hard_hat_subtype_increment() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        for event in [
            raw("pc", Some(SPRITE_MAIN_ENTRY_PC), None, None),
            raw("pc", Some(SPRITE_EXECUTE_SINGLE_ENTRY_PC), Some(4), None),
            raw(
                "wram-write",
                Some(HELMASAUR_HARD_HAT_BEETLE_SUBTYPE2_INCREMENT_PC),
                Some(4),
                Some(SPRITE_SUBTYPE2_BASE + 4),
            ),
        ] {
            source.consume_event(event, &mut receipts).unwrap();
        }
        source.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteMainProgressed(
                SpriteMainProgress::AfterHelmasaurHardHatBeetleSubtype2Increment(4),
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

    #[test]
    fn preemptive_poly_render_start_is_module_scoped_source_authority() {
        for main in [0x07, 0x0e, 0x19] {
            let mut source = empty_semantic_tracker();
            let mut receipts = Vec::new();
            let mut event = raw("pc", Some(POLYHEDRAL_RENDER_START_PC), None, None);
            event.main = Some(main);
            source.consume_event(event, &mut receipts).unwrap();
            assert_eq!(
                receipts,
                vec![OriginalTimingSemanticReceipt::PreemptivePolyhedralRenderStarted],
            );
        }

        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut title_event = raw("pc", Some(POLYHEDRAL_RENDER_START_PC), None, None);
        title_event.main = Some(0x00);
        source.consume_event(title_event, &mut receipts).unwrap();
        assert!(receipts.is_empty());
    }

    #[test]
    fn dungeon_reset_caller_pc_maps_to_the_last_published_source_statement() {
        for pc in [0x09_c163, 0x09_c166] {
            let mut event = raw("nmi", Some(pc), None, None);
            event.a = Some(0xffff);
            assert_eq!(
                dungeon_reset_sprites_caller_progress(&event),
                Some(DungeonResetSpritesCpuProgress::LoadBeforeOrigin)
            );
            event.a = Some(0x0123);
            assert_eq!(dungeon_reset_sprites_caller_progress(&event), None);
        }
        let cases = [
            (
                DUNGEON_RESET_SPRITES_AFTER_DISABLE_PC,
                DungeonResetSpritesCpuProgress::SpritesDisabled,
            ),
            (
                DUNGEON_RESET_SPRITES_COLLISION_Y_STORE_PC,
                DungeonResetSpritesCpuProgress::CollisionXSizeSet,
            ),
            (
                DUNGEON_RESET_SPRITES_HISTORY_SEARCH_START_PC,
                DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
            ),
            (
                0x09_c137,
                DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
            ),
            (
                DUNGEON_RESET_SPRITES_HISTORY_FOUND_PC,
                DungeonResetSpritesCpuProgress::RoomHistorySearchStarted,
            ),
        ];
        for (pc, expected) in cases {
            let event = raw("nmi", Some(pc), None, None);
            assert_eq!(
                dungeon_reset_sprites_caller_progress(&event),
                Some(expected)
            );
        }
        assert_eq!(
            dungeon_reset_sprites_caller_progress(&raw(
                "nmi",
                Some(DUNGEON_RESET_SPRITES_HISTORY_FIRST_MUTATION_PC),
                None,
                None,
            )),
            None,
        );
    }

    #[test]
    fn falling_entrance_control_writes_publish_source_stages_without_cpu_provenance() {
        let cases = [
            (
                FALLING_ENTRANCE_ROOM_PARSER_SUBSUB_CLEAR_PC,
                SUBSUBMODULE_INDEX,
                0,
                DungeonFallingEntranceProgress::RoomParserClearedSubsubmodule,
            ),
            (
                FALLING_ENTRANCE_SUBSUB_ADVANCE_PC,
                SUBSUBMODULE_INDEX,
                3,
                DungeonFallingEntranceProgress::RoomLoadAdvancedSubsubmodule,
            ),
            (
                FALLING_ENTRANCE_SONG_BANK_TAIL_PC,
                SUBMODULE_INDEX,
                7,
                DungeonFallingEntranceProgress::SongBankTailEntered,
            ),
        ];

        for (pc, address, value, expected) in cases {
            let mut source = empty_semantic_tracker();
            let mut receipts = Vec::new();
            let mut event = raw("wram-write", Some(pc), None, Some(address));
            event.main = Some(0x11);
            event.value = Some(value);
            source.consume_event(event, &mut receipts).unwrap();
            assert_eq!(
                receipts,
                vec![OriginalTimingSemanticReceipt::DungeonFallingEntranceProgress(expected,)],
            );
        }

        let mut source = empty_semantic_tracker();
        let mut event = raw(
            "wram-write",
            Some(FALLING_ENTRANCE_ROOM_PARSER_SUBSUB_CLEAR_PC),
            None,
            Some(SUBSUBMODULE_INDEX),
        );
        event.main = Some(6);
        let mut receipts = Vec::new();
        source.consume_event(event, &mut receipts).unwrap();
        assert!(
            receipts.is_empty(),
            "Module_PreDungeon's shared room-parser clear is not a falling-entrance publication",
        );
    }

    #[test]
    fn rescued_maiden_nmi_publishes_exact_source_order_tilemap_clear_prefix() {
        let cases = [
            (RESCUED_MAIDEN_TILEMAP_CLEAR_FIRST_STORE_PC, 0x03b4, 3792),
            (RESCUED_MAIDEN_TILEMAP_CLEAR_SIXTH_STORE_PC, 0x03b4, 3797),
            (RESCUED_MAIDEN_TILEMAP_CLEAR_FIRST_INX_PC, 0x03b4, 3800),
            (RESCUED_MAIDEN_TILEMAP_CLEAR_SECOND_INX_PC, 0x03b5, 3800),
            (RESCUED_MAIDEN_TILEMAP_CLEAR_COMPARE_PC, 0x03b6, 3800),
            (RESCUED_MAIDEN_TILEMAP_CLEAR_BRANCH_PC, 0x0800, 8192),
        ];

        for (pc, x, completed_stores) in cases {
            let mut source = empty_semantic_tracker();
            let mut receipts = Vec::new();
            let mut event = raw("nmi", Some(pc), Some(x), None);
            event.main = Some(7);
            event.sub = Some(0x18);
            event.subsub = Some(0);
            event.nmi_latch = Some(1);
            source.consume_event(event, &mut receipts).unwrap();
            assert_eq!(
                receipts,
                vec![
                    OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
                    OriginalTimingSemanticReceipt::RescuedMaidenTilemapClearProgress(
                        RescuedMaidenTilemapClearProgressReceipt {
                            completed_stores,
                            boundary: OriginalTimingBoundary::NmiAccepted,
                        },
                    ),
                ],
            );
        }

        let mut source = empty_semantic_tracker();
        let mut event = raw(
            "nmi",
            Some(RESCUED_MAIDEN_TILEMAP_CLEAR_SIXTH_STORE_PC),
            Some(0x03b4),
            None,
        );
        event.main = Some(7);
        event.sub = Some(0x17);
        event.subsub = Some(0);
        assert!(source.consume_event(event, &mut Vec::new()).is_err());
    }

    #[test]
    fn rescued_maiden_host_return_preserves_the_incomplete_store_loop() {
        let mut event = raw(
            "frame",
            Some(RESCUED_MAIDEN_TILEMAP_CLEAR_SIXTH_STORE_PC),
            Some(0x03b4),
            None,
        );
        event.main = Some(7);
        event.sub = Some(0x18);
        event.subsub = Some(0);
        assert_eq!(
            rescued_maiden_tilemap_clear_progress(&event, OriginalTimingBoundary::HostReturn)
                .unwrap(),
            Some(RescuedMaidenTilemapClearProgressReceipt {
                completed_stores: 3797,
                boundary: OriginalTimingBoundary::HostReturn,
            }),
        );
    }

    #[test]
    fn rescued_maiden_follower_graphics_tracks_exact_sheet_cursors() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();

        let mut load = raw(
            "pc",
            Some(RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC),
            None,
            None,
        );
        load.main = Some(7);
        load.sub = Some(0x18);
        load.subsub = Some(10);
        source.consume_event(load, &mut receipts).unwrap();

        let mut first_entry = raw(
            "pc",
            Some(RESCUED_MAIDEN_FIRST_FOLLOWER_SHEET_ENTRY_PC),
            None,
            None,
        );
        first_entry.y = Some(0x66);
        source.consume_event(first_entry, &mut receipts).unwrap();

        let mut first_boundary = raw("nmi", Some(0x00_e843), None, None);
        first_boundary.main = Some(7);
        first_boundary.sub = Some(0x18);
        first_boundary.subsub = Some(10);
        first_boundary.nmi_latch = Some(1);
        first_boundary.y = Some(1027);
        source.consume_event(first_boundary, &mut receipts).unwrap();
        assert_eq!(
            source
                .rescued_maiden_initialization
                .unwrap()
                .host_return_receipt()
                .unwrap(),
            RescuedMaidenInitializationProgressReceipt {
                stage: RescuedMaidenInitializationStage::FirstFollowerSheet {
                    completed_bytes: 1027,
                },
                boundary: OriginalTimingBoundary::HostReturn,
            },
        );

        let mut tracker = RescuedMaidenInitializationTracker::first_sheet();
        tracker.begin_second_sheet().unwrap();
        let mut second_boundary = raw("frame", Some(0x00_e851), None, None);
        second_boundary.y = Some(189);
        tracker.observe_boundary(&second_boundary).unwrap();
        assert_eq!(
            tracker.host_return_receipt().unwrap().stage,
            RescuedMaidenInitializationStage::SecondFollowerSheet {
                completed_bytes: 189,
            },
        );

        let mut after_store = raw("frame", Some(0x00_e7f4), None, None);
        after_store.y = Some(1343);
        tracker.observe_boundary(&after_store).unwrap();
        assert_eq!(
            tracker.host_return_receipt().unwrap().stage,
            RescuedMaidenInitializationStage::SecondFollowerSheet {
                completed_bytes: 1344,
            },
        );
    }

    #[test]
    fn push_block_checkpoint_requires_the_module07_caller() {
        let mut event = raw("frame", Some(0x07_f0b2), None, None);
        event.main = Some(7);
        event.return_address = Some(0x88_3d00);
        assert!(dungeon_push_blocks_pending(&event));
        event.return_address = Some(0x88_3e00);
        assert!(!dungeon_push_blocks_pending(&event));
        event.return_address = Some(0x88_3d00);
        event.main = Some(9);
        assert!(!dungeon_push_blocks_pending(&event));
    }

    #[test]
    fn vitreous_damage_checkpoint_requires_its_minion_caller() {
        let mut source = empty_semantic_tracker();
        source.sprite_main_execution = Some(SpriteMainExecutionTracker::default());
        source.sprite_main_execution.as_mut().unwrap().current_slot = Some(0);
        let mut returned = raw("frame", Some(0x06_f2ab), Some(0), None);
        returned.return_address = Some(0xc2_141d);
        source
            .sprite_main_execution
            .as_mut()
            .unwrap()
            .observe_vitreous_damage_pending(&returned);
        assert_eq!(
            source
                .sprite_main_execution
                .as_ref()
                .unwrap()
                .vitreous_damage_pending,
            None
        );
        let mut write = raw("wram-write", Some(0x1d_e5dd), Some(0), None);
        write.address = Some(0x0e80);
        write.value = Some(72);
        source.consume_event(write, &mut Vec::new()).unwrap();
        let execution = source.sprite_main_execution.as_mut().unwrap();
        execution.observe_vitreous_damage_pending(&returned);
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::VitreousDamagePending(0)
        );
        execution.vitreous_player_damage_pending = None;
        execution.vitreous_ai_pending = None;
        execution.vitreous_damage_pending = None;
        returned.return_address = Some(0xc2_151d);
        execution.observe_vitreous_damage_pending(&returned);
        assert_eq!(execution.vitreous_damage_pending, None);
        returned.pc = Some(0x06_f82d);
        returned.return_address = Some(0xf2_cd01);
        execution.observe_vitreous_damage_pending(&returned);
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::VitreousDamagePending(0)
        );
        execution.vitreous_damage_pending = None;
        returned.pc = Some(0x06_f5e3);
        returned.return_address = Some(0xaf_f2ca);
        execution.observe_vitreous_damage_pending(&returned);
        assert_eq!(execution.vitreous_damage_pending, Some(0));
        execution.vitreous_damage_pending = None;
        returned.pc = Some(0x06_f600);
        returned.x = Some(9);
        returned.return_address = Some(0xf2_ca00);
        execution.observe_vitreous_damage_pending(&returned);
        assert_eq!(execution.vitreous_damage_pending, Some(0));
        execution.vitreous_damage_pending = None;
        returned.pc = Some(0x06_f645);
        execution.observe_vitreous_damage_pending(&returned);
        assert_eq!(execution.vitreous_damage_pending, None);
        returned.x = Some(0);
        returned.pc = Some(0x00_8788);
        returned.y = Some(0xe4);
        returned.return_address = Some(0x1f_1de4);
        execution.observe_vitreous_damage_pending(&returned);
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::VitreousAiPending(0)
        );
        execution.vitreous_ai_pending = None;
        returned.pc = Some(0x06_f145);
        returned.return_address = Some(0x1d_f126);
        execution.observe_vitreous_damage_pending(&returned);
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::VitreousPlayerDamagePending(0)
        );
    }

    #[test]
    fn dispatcher_rts_supersedes_body_progress_only_for_the_slot_loop_caller() {
        for slot in [0, 1, 15] {
            let mut execution = SpriteMainExecutionTracker {
                current_slot: Some(slot),
                timers_and_oam_slot: Some(slot),
                timers_and_oam_dispatch_state: Some(9),
                ..Default::default()
            };
            let mut event = raw("nmi", Some(0x06_bff8), Some(u16::from(slot)), None);
            event.return_address = Some(0x0083a5);
            execution
                .observe_dispatch_trampoline_return(&event)
                .unwrap();
            assert_eq!(execution.dispatch_trampoline_return, None);
            event.return_address = Some(0x0083a6);
            execution
                .observe_dispatch_trampoline_return(&event)
                .unwrap();
            assert_eq!(execution.progress(), SpriteMainProgress::AfterSlot(slot));
        }
    }

    #[test]
    fn swamola_segment_checkpoint_requires_the_source_loop_and_draw_caller() {
        let mut source = empty_semantic_tracker();
        source.sprite_main_execution = Some(SpriteMainExecutionTracker::default());
        source.sprite_main_execution.as_mut().unwrap().current_slot = Some(4);
        let mut write = raw("wram-write", Some(0x1d_a034), Some(4), None);
        write.address = Some(0x0fb6);
        write.value = Some(1);
        source.consume_event(write, &mut Vec::new()).unwrap();
        let execution = source.sprite_main_execution.as_mut().unwrap();
        let mut returned = raw("frame", Some(0x06_e442), Some(4), None);
        returned.return_address = Some(0xf5_dc12);
        execution.observe_swamola_segment_draw(&returned).unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::SwamolaSegmentDraw {
                slot: 4,
                segment: 1
            }
        );
        execution.swamola_head_prepared = false;
        execution.swamola_head_draw_completed = None;
        execution.swamola_head_draw = None;
        execution.swamola_segment_draw = None;
        returned.return_address = Some(0xf5_dc13);
        execution.observe_swamola_segment_draw(&returned).unwrap();
        assert_eq!(execution.swamola_segment_draw, None);
        returned.pc = Some(0x06_dbf0);
        returned.return_address = Some(0x1d_9f8b);
        execution.observe_swamola_segment_draw(&returned).unwrap();
        assert_eq!(execution.progress(), SpriteMainProgress::SwamolaHeadDraw(4));
        execution.swamola_head_prepared = false;
        execution.swamola_head_draw_completed = None;
        execution.swamola_head_draw = None;
        returned.return_address = Some(0x1d_9f8c);
        execution.observe_swamola_segment_draw(&returned).unwrap();
        assert_eq!(execution.swamola_head_draw, None);
        returned.pc = Some(0x06_e492);
        returned.return_address = Some(0xdb_f5dc);
        execution.observe_swamola_segment_draw(&returned).unwrap();
        assert_eq!(execution.swamola_head_draw_completed, None);
        execution.swamola_head_prepared = true;
        execution.observe_swamola_segment_draw(&returned).unwrap();
        assert_eq!(
            execution.progress(),
            SpriteMainProgress::SwamolaHeadDrawCompleted(4)
        );
    }

    #[test]
    fn purple_chest_follower_graphics_uses_its_exact_body_caller() {
        let mut source = empty_semantic_tracker();
        let mut execution = SpriteMainExecutionTracker::default();
        execution.current_slot = Some(8);
        source.sprite_main_execution = Some(execution);
        let mut event = raw(
            "pc",
            Some(RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC),
            Some(8),
            None,
        );
        event.return_address = Some(SPRITE_PURPLE_CHEST_FOLLOWER_GRAPHICS_RETURN_PC);
        source.consume_event(event, &mut Vec::new()).unwrap();
        assert_eq!(
            source
                .sprite_main_execution
                .as_ref()
                .unwrap()
                .follower_graphics
                .unwrap()
                .0,
            SpriteFollowerGraphicsCaller::PurpleChest
        );
        let mut sheet = raw(
            "pc",
            Some(RESCUED_MAIDEN_FIRST_FOLLOWER_SHEET_ENTRY_PC),
            Some(8),
            None,
        );
        sheet.y = Some(0x58);
        source
            .consume_event(sheet.clone(), &mut Vec::new())
            .unwrap();
        sheet.y = Some(0x66);
        assert!(source.consume_event(sheet, &mut Vec::new()).is_err());
    }

    #[test]
    fn blind_maiden_body_follower_graphics_uses_its_exact_caller() {
        let mut source = empty_semantic_tracker();
        let mut execution = SpriteMainExecutionTracker::default();
        execution.current_slot = Some(0);
        source.sprite_main_execution = Some(execution);

        let mut load = raw(
            "pc",
            Some(RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC),
            Some(0),
            None,
        );
        load.return_address = Some(SPRITE_BLIND_MAIDEN_BODY_FOLLOWER_GRAPHICS_RETURN_PC);
        source.consume_event(load, &mut Vec::new()).unwrap();

        assert_eq!(
            source
                .sprite_main_execution
                .unwrap()
                .follower_graphics
                .unwrap()
                .0,
            SpriteFollowerGraphicsCaller::BlindMaidenBody,
        );
    }

    #[test]
    fn old_man_follower_graphics_uses_its_exact_prep_return() {
        let mut source = empty_semantic_tracker();
        let mut execution = SpriteMainExecutionTracker::default();
        execution.current_slot = Some(8);
        source.sprite_main_execution = Some(execution);

        let mut load = raw(
            "pc",
            Some(RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC),
            Some(8),
            None,
        );
        load.return_address = Some(SPRITE_PREP_OLD_MAN_FOLLOWER_GRAPHICS_RETURN_PC);
        source.consume_event(load, &mut Vec::new()).unwrap();

        let mut boundary = raw("nmi", Some(0x00_e845), None, None);
        boundary.main = Some(7);
        boundary.sub = Some(15);
        boundary.subsub = Some(1);
        boundary.nmi_latch = Some(1);
        boundary.y = Some(1102);
        let mut receipts = Vec::new();
        source.consume_event(boundary, &mut receipts).unwrap();

        assert!(
            receipts.contains(&OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::SpriteMainFollowerGraphics {
                    slot: 8,
                    caller: SpriteFollowerGraphicsCaller::OldMan,
                    stage: RescuedMaidenInitializationStage::FirstFollowerSheet {
                        completed_bytes: 1102,
                    },
                },
            ))
        );
    }

    #[test]
    fn zelda_follower_graphics_uses_the_pinned_sprite_prep_return() {
        let mut source = empty_semantic_tracker();
        let mut execution = SpriteMainExecutionTracker::default();
        execution.current_slot = Some(1);
        source.sprite_main_execution = Some(execution);

        let mut load = raw(
            "pc",
            Some(RESCUED_MAIDEN_LOAD_FOLLOWER_GRAPHICS_ENTRY_PC),
            Some(1),
            None,
        );
        load.return_address = Some(0x05_ebf5);
        source.consume_event(load, &mut Vec::new()).unwrap();

        assert_eq!(
            source
                .sprite_main_execution
                .unwrap()
                .follower_graphics
                .unwrap()
                .0,
            SpriteFollowerGraphicsCaller::Zelda,
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
            stack1: None,
            a: None,
            main: None,
            sub: None,
            subsub: None,
            room: None,
            frame_counter: None,
            nmi_latch: matches!(event, "nmi").then_some(0),
            link_y: None,
            bg2_v: None,
            bg2_h: None,
            spotlight_radius: None,
            spotlight_var4_low: None,
            palette_countdown: None,
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

    #[test]
    fn triforce_case2_palette_progress_uses_source_word_and_return_boundaries() {
        let mut partial = raw(
            "frame",
            Some(PALETTE_LOAD_MULTIPLE_BEFORE_WORD_COPY_PC),
            Some(0x00bc),
            None,
        );
        partial.main = Some(0x19);
        partial.sub = Some(0);
        partial.subsub = Some(2);
        partial.room = Some(0x0109);
        assert_eq!(
            triforce_room_case2_palette_progress(&partial, OriginalTimingBoundary::HostReturn,)
                .unwrap(),
            Some(TriforceRoomCase2PaletteProgressReceipt {
                completed_ow_bg2_words: 5,
                boundary: OriginalTimingBoundary::HostReturn,
            }),
        );

        let mut completed = raw(
            "nmi",
            Some(OVERWORLD_PARSE_MAP32_DEFINITION_SECOND_WORD_PC),
            None,
            None,
        );
        completed.main = Some(0x19);
        completed.sub = Some(0);
        completed.subsub = Some(2);
        completed.room = Some(0x0189);
        assert_eq!(
            triforce_room_case2_palette_progress(&completed, OriginalTimingBoundary::NmiAccepted,)
                .unwrap(),
            Some(TriforceRoomCase2PaletteProgressReceipt {
                completed_ow_bg2_words: 21,
                boundary: OriginalTimingBoundary::NmiAccepted,
            }),
        );
    }

    #[test]
    fn credits_scene_progress_preserves_the_scene_advance_and_text_prefix() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut scene_return = raw(
            "wram-write",
            Some(CREDITS_SCENE_OVERWORLD_SUBSUBMODULE_INCREMENT_PC),
            None,
            Some(SUBSUBMODULE_INDEX),
        );
        scene_return.main = Some(0x1a);
        scene_return.value = Some(1);
        source.consume_event(scene_return, &mut receipts).unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::CreditsSceneLoadProgress(
                CreditsSceneLoadProgressReceipt {
                    progress: CreditsSceneLoadProgress::SceneLoadCompleted,
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            )],
        );

        let mut text = raw(
            "frame",
            Some(CREDITS_ENDING_TEXT_BEFORE_TILE_COPY_PC),
            Some(50),
            None,
        );
        text.main = Some(0x1a);
        assert_eq!(
            credits_scene_load_boundary_progress(&text, OriginalTimingBoundary::HostReturn)
                .unwrap(),
            Some(CreditsSceneLoadProgressReceipt {
                progress: CreditsSceneLoadProgress::EndingTextPayloadBytes(50),
                boundary: OriginalTimingBoundary::HostReturn,
            }),
        );
    }

    #[test]
    fn credits_finale_save_progress_decodes_the_completed_checksum_words() {
        let mut event = raw(
            "frame",
            Some(CREDITS_END_SEQUENCE_32_SAVE_CHECKSUM_LOOP_PC),
            Some(0x016e),
            None,
        );
        event.main = Some(0x1a);
        event.sub = Some(0x21);
        event.subsub = Some(0);
        assert_eq!(
            credits_end_sequence_32_boundary_progress(&event, OriginalTimingBoundary::HostReturn,)
                .unwrap(),
            Some(CreditsEndSequence32ProgressReceipt {
                completed_checksum_words: 183,
                boundary: OriginalTimingBoundary::HostReturn,
            }),
        );
    }

    #[test]
    fn peg_attribute_flip_pc_decodes_to_source_bank_progress() {
        let mut event = raw(
            "frame",
            Some(DUNGEON_PEG_FLIP_BANK_C_PC),
            Some(0x0594),
            None,
        );
        event.main = Some(7);
        event.sub = Some(0x16);
        event.subsub = Some(0x10);

        assert_eq!(
            dungeon_peg_attribute_flip_progress(&event, OriginalTimingBoundary::HostReturn,)
                .unwrap(),
            Some(DungeonPegAttributeFlipProgressReceipt {
                index: 0x0594,
                completed_banks: 2,
                boundary: OriginalTimingBoundary::HostReturn,
            }),
        );

        event.sub = Some(2);
        event.subsub = Some(8);
        assert_eq!(
            dungeon_peg_attribute_flip_progress(&event, OriginalTimingBoundary::HostReturn,)
                .unwrap(),
            Some(DungeonPegAttributeFlipProgressReceipt {
                index: 0x0594,
                completed_banks: 2,
                boundary: OriginalTimingBoundary::HostReturn,
            }),
            "the shared source helper must expose the same cursor to its selectable caller",
        );
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            stack1: None,
            a: None,
            main: Some(main),
            sub: Some(0),
            subsub: Some(0),
            room: None,
            frame_counter: Some(frame_counter),
            nmi_latch: Some(0),
            link_y: None,
            bg2_v: None,
            bg2_h: None,
            spotlight_radius: None,
            spotlight_var4_low: None,
            palette_countdown: None,
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
    fn dialogue_scroll_copy_counts_follow_source_hosts_until_return() {
        // Source hosts 20765-20767 finish the same five-pass call as 2+2+1.
        // The third host alone reaches the scroll RTS. Register values are
        // deliberately absent: this receipt counts source operations only.
        for (entered, copies, returned) in [(true, 2, false), (false, 2, false), (false, 1, true)] {
            let mut host = DialogueScrollHostWindow::default();
            if entered {
                host.observe(&raw("pc", Some(DIALOGUE_SCROLL_ENTRY_PC), None, None))
                    .unwrap();
            }
            for _ in 0..copies {
                host.observe(&raw(
                    "pc",
                    Some(DIALOGUE_SCROLL_PIXEL_COMPLETED_PC),
                    None,
                    None,
                ))
                .unwrap();
            }
            if returned {
                host.observe(&raw("pc", Some(DIALOGUE_SCROLL_RETURN_PC), None, None))
                    .unwrap();
            }
            assert_eq!(
                host.finish(),
                vec![zelda3::DialogueScrollProgressReceipt {
                    entered,
                    completed_pixel_passes: copies,
                    returned,
                }]
            );
        }
    }

    #[test]
    fn dialogue_scroll_observation_does_not_invent_a_completed_copy() {
        let mut host = DialogueScrollHostWindow::default();
        host.observe(&raw(
            "frame",
            Some(DIALOGUE_SCROLL_PIXEL_COMPLETED_PC),
            None,
            None,
        ))
        .unwrap();
        host.observe(&raw(
            "nmi",
            Some(DIALOGUE_SCROLL_PIXEL_COMPLETED_PC),
            None,
            None,
        ))
        .unwrap();
        assert!(host.finish().is_empty());
    }

    #[test]
    fn dialogue_scroll_calls_keep_their_order_within_one_host() {
        let mut host = DialogueScrollHostWindow::default();
        host.observe(&raw(
            "pc",
            Some(DIALOGUE_SCROLL_PIXEL_COMPLETED_PC),
            None,
            None,
        ))
        .unwrap();
        host.observe(&raw("pc", Some(DIALOGUE_SCROLL_RETURN_PC), None, None))
            .unwrap();
        host.observe(&raw("pc", Some(DIALOGUE_SCROLL_ENTRY_PC), None, None))
            .unwrap();
        assert_eq!(
            host.finish(),
            vec![
                zelda3::DialogueScrollProgressReceipt {
                    entered: false,
                    completed_pixel_passes: 1,
                    returned: true
                },
                zelda3::DialogueScrollProgressReceipt {
                    entered: true,
                    completed_pixel_passes: 0,
                    returned: false
                },
            ]
        );
        let mut duplicate = DialogueScrollHostWindow::default();
        duplicate
            .observe(&raw("pc", Some(DIALOGUE_SCROLL_ENTRY_PC), None, None))
            .unwrap();
        assert!(duplicate
            .observe(&raw("pc", Some(DIALOGUE_SCROLL_ENTRY_PC), None, None))
            .is_err());
    }

    #[test]
    fn carried_glyph_resume_preserves_the_endpoint_before_scroll_entry() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 218, 14, 2);
        entry.pc = Some(NMI_HANDLER_ENTRY_PC);
        host.observe(&entry).unwrap();
        host.observe(&raw("nmi-resume", Some(0x0e_cca5), None, None))
            .unwrap();
        host.observe(&raw("pc", Some(DIALOGUE_SCROLL_ENTRY_PC), None, None))
            .unwrap();
        host.observe(&raw("nmi", Some(0x0e_d031), None, None))
            .unwrap();
        let mut returned = frame_with_sub("return", 218, 14, 2);
        returned.pc = Some(NMI_HANDLER_ENTRY_PC);
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, Some(65), true).unwrap();
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued
                ),
                OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                    DialogueExecutionProgress::ResumedRenderingWithoutMainIteration {
                        message_read_position: 65
                    },
                ),
            ]
        );
    }

    #[test]
    fn dialogue_terminal_caller_supersedes_the_decoder_endpoint() {
        // Cold host 20440 resumes a glyph at $0E:CC2F, executes END's
        // countdown decrement, returns through the common suffix, and enters
        // the next NMI. The read cursor remains 108 at the END command.
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 440, 14, 2);
        entry.pc = Some(0x0e_cc2e);
        host.observe(&entry).unwrap();
        host.observe(&raw("nmi", Some(0x0e_cc2f), None, None))
            .unwrap();
        let mut returned = frame_with_sub("return", 440, 14, 2);
        returned.pc = Some(0x00_80c9);
        host.observe(&returned).unwrap();
        let terminal = vec![
            OriginalTimingSemanticReceipt::MainLoopProgress(MainLoopProgress::CallStackContinued),
            OriginalTimingSemanticReceipt::MainLoopCommonSuffixCompleted,
        ];
        let mut receipts = terminal.clone();
        host.finish(&mut receipts, Some(108), true).unwrap();
        assert_eq!(receipts, terminal);
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
    fn dialogue_return_inside_current_glyph_preserves_the_committed_prefix() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame("entry", 7327, 14, 0x35)).unwrap();
        host.observe(&raw("nmi", Some(VWF_RENDER_SINGLE_END_PC - 1), None, None))
            .unwrap();
        let mut returned = frame("return", 7327, 14, 0x35);
        returned.pc = Some(VWF_RENDER_SINGLE_BODY_START_PC);
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, Some(0x003a), true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                    DialogueExecutionProgress::ResumedRenderingWithCurrentGlyphStarted {
                        message_read_position: 0x003a,
                    },
                ),
            ],
        );
    }

    #[test]
    fn dialogue_return_at_nmi_entry_preserves_the_interrupted_glyph_prefix() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame("entry", 7332, 14, 0xbf)).unwrap();
        host.observe(&raw(
            "nmi",
            Some(VWF_RENDER_SINGLE_BODY_START_PC + 0x166),
            None,
            None,
        ))
        .unwrap();
        let mut returned = frame("return", 7332, 14, 0xbf);
        returned.pc = Some(NMI_HANDLER_ENTRY_PC);
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, Some(0x0056), true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::DialogueExecutionProgress(
                    DialogueExecutionProgress::ResumedRenderingWithCurrentGlyphStarted {
                        message_read_position: 0x0056,
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
    fn link_oam_equipment_selector_publishes_its_native_prefix() {
        assert_eq!(
            link_oam_stair_progress(0x0d_a8b6, Some(18)),
            Some(zelda3::LinkOamStairProgress::ShadowSelection)
        );
        assert_eq!(
            link_oam_stair_progress(0x0d_a992, Some(18)),
            Some(zelda3::LinkOamStairProgress::BodySelection)
        );
        assert_eq!(
            link_oam_stair_progress(0x0d_a47e, Some(18)),
            Some(zelda3::LinkOamStairProgress::PoseSelected)
        );
        assert_eq!(link_oam_stair_progress(0x0d_a47b, Some(18)), None);
        assert_eq!(link_oam_stair_progress(0x0d_a47e, Some(1)), None);
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 1, 7, 18)).unwrap();
        let mut returned = frame_with_sub("return", 1, 7, 18);
        returned.pc = Some(0x0d_a61a);
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, true).unwrap();
        assert!(
            receipts.contains(&OriginalTimingSemanticReceipt::MainLoopInterrupted(
                MainLoopInterruption::LinkOam,
            ))
        );
        assert!(
            receipts.contains(&OriginalTimingSemanticReceipt::LinkOamStairProgress(
                zelda3::LinkOamStairProgress::EquipmentSelection
            ))
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
    fn special_exit_mosaic_return_becomes_a_backend_neutral_receipt() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 7426, 0x0b, 0x24))
            .unwrap();
        host.observe(&frame_with_sub("return", 7426, 0x0b, 0x25))
            .unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicReturned,
            ],
        );
    }

    #[test]
    fn special_exit_second_decode_entry_proves_the_restore_prefix() {
        let mut event = raw("pc", Some(DECODE_ANIMATED_SPRITE_TILE_ENTRY_PC), None, None);
        event.return_address = Some(SPECIAL_EXIT_MOSAIC_SECOND_DECODE_RETURN_ADDRESS);
        event.main = Some(0x0b);
        event.sub = Some(0x24);

        assert!(special_exit_mosaic_restore_checkpoint(&event).unwrap());

        event.sub = Some(0x23);
        assert!(special_exit_mosaic_restore_checkpoint(&event).is_err());
    }

    #[test]
    fn special_exit_terminal_return_supersedes_the_restore_checkpoint() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 7426, 0x0b, 0x24))
            .unwrap();
        host.observe(&frame_with_sub("return", 7426, 0x0b, 0x25))
            .unwrap();
        let mut receipts = vec![OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicRestored];

        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::OverworldSpecialExitMosaicReturned,
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
    fn dungeon_exit_spotlight_entry_preserves_its_partial_link_movement() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 24_193, 0x0f, 0))
            .unwrap();
        let mut returned = frame_with_sub("return", 24_193, 0x0f, 1);
        returned.pc = Some(0x07_e3c5);
        returned.x = Some(0);
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();

        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::LinkPositionAfterSubpixel { pass: 0 },
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
    fn overworld_load_overlays_call_identity_owns_cross_host_sprite_publication_and_return() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut entry = raw(
            "pc",
            Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC),
            None,
            None,
        );
        entry.return_address = Some(OVERWORLD_LOAD_OVERLAYS_AFTER_SPRITE_RELOAD_PC);
        entry.main = Some(0x0b);
        entry.sub = Some(0x25);
        tracker.consume_event(entry, &mut receipts).unwrap();
        assert!(tracker.overworld_load_overlays_sprite_reload_active);

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
            event.main = Some(0x0b);
            // The source module bytes do not prove this inner call. Use a
            // different direct-dispatch submodule to ensure the entry/return
            // identity, not the old Module0B/$18 special case, owns it.
            event.sub = Some(0x25);
            event.value = Some(value);
            tracker.consume_event(event, &mut receipts).unwrap();
        }

        let mut returned = raw(
            "pc",
            Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_RETURN_PC),
            None,
            None,
        );
        returned.main = Some(0x0b);
        returned.sub = Some(0x25);
        tracker.consume_event(returned, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::SpriteActivated {
                        block: 0x0198,
                        slot,
                        sprite_type: 0xac,
                    },
                ),
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::GenerationReturned,
                ),
            ],
        );
        assert!(!tracker.overworld_load_overlays_sprite_reload_active);
    }

    #[test]
    fn sprite_main_proximity_helper_does_not_publish_reload_authority() {
        let mut source = empty_semantic_tracker();
        source.sprite_main_execution = Some(SpriteMainExecutionTracker::default());
        let mut event = raw("nmi", Some(0x09_c5fc), Some(0), None);
        event.main = Some(9);
        event.sub = Some(4);
        event.bg2_h = Some(0x08a8);
        let mut receipts = Vec::new();
        assert!(!source.publish_overworld_presence_at_scan_boundary(&event, &mut receipts));
        assert!(receipts.is_empty());
        source.sprite_main_execution = None;
        assert!(source.publish_overworld_presence_at_scan_boundary(&event, &mut receipts));
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::PresencePublished
                )
            ]
        );
    }

    #[test]
    fn mirror_portal_reset_requires_the_private_spawn_caller() {
        for caller in [0x09_afa5, 0x06_8b5f] {
            let mut host = HostFrameWindow::default();
            host.observe(&frame_with_sub("entry", 1, 9, 0x23)).unwrap();
            let mut spawn = raw(
                "wram-write",
                Some(SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC),
                Some(0xff),
                Some(15),
            );
            spawn.return_address = Some(caller);
            spawn.y = Some(15);
            spawn.address = Some(SPRITE_TYPE_BASE + 15);
            spawn.value = Some(0x6c);
            host.observe(&spawn).unwrap();
            let mut returned = frame_with_sub("return", 1, 9, 0x23);
            returned.pc = Some(0x0d_b889);
            returned.x = Some(15);
            returned.return_address = Some(SPRITE_PREP_LOAD_PROPERTIES_AFTER_RESET_RETURN_ADDRESS);
            host.observe(&returned).unwrap();
            let mut receipts = vec![
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::GenerationReturned,
                ),
            ];
            host.finish(&mut receipts, None, true).unwrap();
            let expected = if caller == 0x09_afa5 {
                OverworldSpriteReloadProgress::GenerationReturnedAtPortalReset {
                    slot: 15,
                    completed_stores: 8,
                }
            } else {
                OverworldSpriteReloadProgress::GenerationReturned
            };
            assert!(receipts
                .contains(&OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(expected)));
        }
    }

    #[test]
    fn mirror_cleanup_boundary_retains_generation_return_and_pending_slot() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 1, 9, 0x23)).unwrap();
        let mut returned = frame_with_sub("return", 1, 9, 0x23);
        returned.pc = Some(0x09_ac9c);
        returned.x = Some(4);
        host.observe(&returned).unwrap();
        let mut receipts = vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturned,
            ),
        ];
        host.finish(&mut receipts, None, true).unwrap();
        assert!(receipts.contains(
            &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveCleanup { slot: 4 }
            )
        ));
        assert!(!receipts.contains(
            &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturned
            )
        ));
    }

    #[test]
    fn mirror_type_clear_boundary_defers_portal_spawn() {
        let mut host = HostFrameWindow::default();
        host.observe(&frame_with_sub("entry", 1, 9, 0x23)).unwrap();
        let mut returned = frame_with_sub("return", 1, 9, 0x23);
        returned.pc = Some(0x09_aca6);
        returned.x = Some(1);
        host.observe(&returned).unwrap();
        let mut receipts = vec![
            OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturned,
            ),
        ];
        host.finish(&mut receipts, None, true).unwrap();
        assert!(receipts.contains(
            &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturnedAtInteractiveTypeClear { slot: 1 }
            )
        ));
        assert!(!receipts.contains(
            &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturned
            )
        ));
    }

    #[test]
    fn mirror_warp_call_identity_owns_module09_23_scan_and_generation_return() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut entry = raw(
            "pc",
            Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC),
            None,
            None,
        );
        entry.return_address = Some(MIRROR_WARP_AFTER_SPRITE_RELOAD_PC);
        entry.main = Some(9);
        entry.sub = Some(0x23);
        tracker.consume_event(entry, &mut receipts).unwrap();
        assert!(tracker.overworld_load_overlays_sprite_reload_active);

        let mut nmi = raw(
            "nmi",
            Some(OVERWORLD_SPRITE_SCAN_START_PC + 0x1b0),
            None,
            None,
        );
        nmi.main = Some(9);
        nmi.sub = Some(0x23);
        nmi.nmi_latch = Some(1);
        nmi.bg2_h = Some(0x01fa);
        tracker.consume_event(nmi, &mut receipts).unwrap();
        assert!(receipts.contains(
            &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::PresencePublished,
            ),
        ));
        assert!(receipts.contains(
            &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h: 0x01fa },
            ),
        ));

        publish_nmi(&mut tracker, &mut receipts);
        tracker
            .consume_event(
                raw(
                    "nmi-resume",
                    Some(OVERWORLD_SPRITE_SCAN_START_PC + 0x1b0),
                    None,
                    None,
                ),
                &mut receipts,
            )
            .unwrap();
        let mut returned = raw(
            "pc",
            Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_RETURN_PC),
            None,
            None,
        );
        returned.main = Some(9);
        returned.sub = Some(0x23);
        tracker.consume_event(returned, &mut receipts).unwrap();
        assert!(receipts.contains(
            &OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturned,
            ),
        ));
        assert!(!tracker.overworld_load_overlays_sprite_reload_active);
    }

    #[test]
    fn save_quit_intro_return_is_superseded_by_terminal_reset_state() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw("wram-write", Some(0x0c_c25b), None, None);
        event.address = Some(0x11);
        event.value = Some(2);
        event.main = Some(0x17);
        event.sub = Some(1);
        event.return_address = Some(0x0c_f0e8);
        tracker.consume_event(event, &mut receipts).unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SaveQuitIntroMemoryReturned]
        );
        let mut returned = raw(
            "pc",
            Some(SAVE_QUIT_RESET_DUNGEON_INFO_CLEAR_ENTRY_PC),
            None,
            None,
        );
        returned.main = Some(0);
        returned.sub = Some(10);
        returned.subsub = Some(10);
        tracker.consume_event(returned, &mut receipts).unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SaveQuitResetStatePublished]
        );
    }

    #[test]
    fn save_quit_reset_dungeon_info_clear_entry_publishes_typed_state_boundary() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw(
            "pc",
            Some(SAVE_QUIT_RESET_DUNGEON_INFO_CLEAR_ENTRY_PC),
            None,
            None,
        );
        event.main = Some(0);
        event.sub = Some(10);
        event.subsub = Some(10);

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SaveQuitResetStatePublished],
        );
    }

    #[test]
    fn file_select_graphics_low_wram_store_reports_exact_source_prefix() {
        let mut event = raw(
            "pc",
            Some(FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_AFTER_PAGE_D_PC),
            None,
            None,
        );
        event.main = Some(1);
        event.sub = Some(1);
        event.subsub = Some(0xd6);
        event.x = Some(0x00fe);

        assert_eq!(
            file_select_graphics_low_wram_clear_progress(&event).unwrap(),
            Some(FileSelectGraphicsLowWramClearProgress {
                word_offset: 0xfe,
                completed_page_stores: 1,
            }),
        );

        event.pc = Some(FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_AFTER_PAGE_F_PC);
        event.x = Some(0x00fc);
        assert_eq!(
            file_select_graphics_low_wram_clear_progress(&event).unwrap(),
            Some(FileSelectGraphicsLowWramClearProgress {
                word_offset: 0xfc,
                completed_page_stores: 3,
            }),
        );
    }

    #[test]
    fn file_select_graphics_low_wram_clear_return_publishes_typed_boundary() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw(
            "pc",
            Some(FILE_SELECT_GRAPHICS_LOW_WRAM_CLEAR_RETURN_PC),
            None,
            None,
        );
        event.main = Some(1);
        event.sub = Some(1);
        event.subsub = Some(0xd6);

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::FileSelectGraphicsLowWramCleared],
        );
    }

    #[test]
    fn module05_show_text_message_return_publishes_typed_interface_boundary() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw(
            "pc",
            Some(SELECTED_GAME_LOAD_MESSAGE_INTERFACE_RETURN_PC),
            None,
            None,
        );
        event.return_address = Some(MODULE05_AFTER_SHOW_TEXT_MESSAGE_PC);
        event.main = Some(14);
        event.sub = Some(2);

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SelectedGameLoadMessageInterfacePublished,],
        );
    }

    #[test]
    fn show_text_message_return_from_an_unrelated_caller_is_not_module05_publication() {
        let mut tracker = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut event = raw(
            "pc",
            Some(SELECTED_GAME_LOAD_MESSAGE_INTERFACE_RETURN_PC),
            None,
            None,
        );
        event.return_address = Some(MODULE05_AFTER_SHOW_TEXT_MESSAGE_PC + 1);
        event.main = Some(14);
        event.sub = Some(2);

        tracker.consume_event(event, &mut receipts).unwrap();

        assert!(receipts.is_empty());
    }

    #[test]
    fn overworld_load_overlays_call_identity_survives_semantic_checkpoint() {
        let mut source = empty_semantic_tracker();
        let mut receipts = Vec::new();
        let mut entry = raw(
            "pc",
            Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC),
            None,
            None,
        );
        entry.return_address = Some(OVERWORLD_LOAD_OVERLAYS_AFTER_SPRITE_RELOAD_PC);
        source.consume_event(entry, &mut receipts).unwrap();

        let checkpoint = source.checkpoint();
        let mut resumed = empty_semantic_tracker();
        resumed.restore_checkpoint(checkpoint).unwrap();
        assert!(resumed.overworld_load_overlays_sprite_reload_active);

        resumed
            .consume_event(
                raw(
                    "pc",
                    Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_RETURN_PC),
                    None,
                    None,
                ),
                &mut receipts,
            )
            .unwrap();
        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::GenerationReturned,
                ),
            ],
        );
    }

    #[test]
    fn module0b_held_host_publishes_proximity_scan_scratch_coordinate() {
        let mut host = HostFrameWindow::default();
        host.overworld_load_overlays_sprite_reload_active = true;
        let mut entry = frame_with_sub("entry", 165775, 0x0b, 0x18);
        entry.bg2_h = Some(0x021e);
        let mut returned = frame_with_sub("return", 165775, 0x0b, 0x18);
        returned.pc = Some(OVERWORLD_SPRITE_SCAN_START_PC + 1);
        returned.bg2_h = Some(0x028e);

        host.observe(&entry).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h: 0x028e },
                ),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
            ],
        );
    }

    #[test]
    fn pre_overworld_held_host_publishes_proximity_scan_scratch_coordinate() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 652122, 0x08, 0);
        entry.bg2_h = Some(0x0700);
        let mut returned = frame_with_sub("return", 652122, 0x08, 0);
        returned.pc = Some(OVERWORLD_SPRITE_SCAN_START_PC + 1);
        returned.bg2_h = Some(0x0720);

        host.observe(&entry).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h: 0x0720 },
                ),
                OriginalTimingSemanticReceipt::MainLoopProgress(
                    MainLoopProgress::CallStackContinued,
                ),
            ],
        );
    }

    #[test]
    fn module0b_fresh_iteration_does_not_publish_held_scan_scratch() {
        let mut host = HostFrameWindow::default();
        let mut entry = frame_with_sub("entry", 165774, 0x0b, 0x18);
        entry.bg2_h = Some(0x021e);
        let mut returned = frame_with_sub("return", 165774, 0x0b, 0x18);
        returned.bg2_h = Some(0x022e);

        host.observe(&entry).unwrap();
        let mut reload_entry = raw(
            "pc",
            Some(OVERWORLD_LOAD_OVERLAYS_SPRITE_RELOAD_ENTRY_PC),
            None,
            None,
        );
        reload_entry.return_address = Some(OVERWORLD_LOAD_OVERLAYS_AFTER_SPRITE_RELOAD_PC);
        host.observe(&reload_entry).unwrap();
        host.observe(&main_loop_start()).unwrap();
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        host.finish(&mut receipts, None, true).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::MainLoopProgress(
                MainLoopProgress::IterationStarted,
            )],
        );
    }

    #[test]
    fn unrelated_module09_sprite_writes_do_not_publish_reload_activation() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            event.main = Some(9);
            event.sub = Some(6);
            event.value = Some(value);
            tracker.consume_event(event, &mut receipts).unwrap();
        }

        assert!(receipts.is_empty());
        assert!(tracker.overworld_sprite_activation.is_none());
    }

    #[test]
    fn host_return_inside_flute_scan_publishes_presence_before_scan_progress() {
        let mut tracker = empty_semantic_tracker();
        tracker.overworld_load_overlays_sprite_reload_active = true;
        let mut host = HostFrameWindow::default();
        host.overworld_load_overlays_sprite_reload_active = true;
        host.observe(&frame_with_sub("entry", 1, 0x0e, 0x0a))
            .unwrap();
        let mut returned = frame_with_sub("return", 1, 0x0e, 0x0a);
        returned.pc = Some(0x09_c723);
        returned.bg2_h = Some(128);
        host.observe(&returned).unwrap();
        let mut receipts = Vec::new();
        assert!(tracker.publish_overworld_presence_at_scan_boundary(&returned, &mut receipts));
        host.finish(&mut receipts, None, true).unwrap();
        assert_eq!(
            &receipts[..2],
            &[
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::PresencePublished
                ),
                OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                    OverworldSpriteReloadProgress::ProximityScanSuspended { bg2_h: 128 }
                ),
            ]
        );
        let mut later = Vec::new();
        assert!(tracker.publish_overworld_presence_at_scan_boundary(&returned, &mut later));
        assert!(later.is_empty());
    }

    #[test]
    fn nmi_inside_overworld_sprite_scan_publishes_presence_once() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
        };
        let mut receipts = Vec::new();
        for _ in 0..2 {
            let mut event = raw("nmi", Some(OVERWORLD_SPRITE_SCAN_START_PC + 1), None, None);
            event.main = Some(8);
            event.sub = Some(0);
            event.bg2_h = Some(0x0720);
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            timers_and_oam_slot: None,
            timers_and_oam_dispatch_state: None,
            initialize_active_main_calls: 0,
            guard_prep_parry_hitbox: None,
            guard_prep_patrol_delay: None,
            guard_prep_tile_collision_return: None,
            guard_animation_checkpoint: None,
            hog_spear_body_graphics_pending: None,
            absorbable_body_active: false,
            absorbable_horizontal_lookup: None,
            absorbable_vertical_lookup: None,
            absorbable_vertical_attribute_loaded: None,
            swamola_segment: None,
            dispatch_trampoline_return: None,
            vitreous_minions_seen: false,
            vitreous_player_damage_pending: None,
            vitreous_ai_pending: None,
            vitreous_damage_pending: None,
            swamola_head_prepared: false,
            swamola_head_draw_completed: None,
            swamola_head_draw: None,
            swamola_segment_draw: None,
            pengator_slide_pending: None,
            antifairy_bounce_pending: None,
            kholdstare_subtype_decremented: false,
            kholdstare_damage_pending: None,
            initialize_prep_pending: None,
            guard_animation_pose_slot: None,
            guard_prep_weapon_flags_pending_slot: None,
            mini_moldorm_history: None,
            initialize_reset_properties: None,
            initialize_load_properties: None,
            fire_debirando_property_reload: false,
            fire_debirando_before_spawn_slot: None,
            fire_debirando_spawn: None,
            antfairy_subtype2_increment_slot: None,
            lanmola_subtype2_increment_slot: None,
            helmasaur_hard_hat_beetle_subtype2_increment_slot: None,
            timer_decrements_slot: None,
            primary_timer_decrements_slot: None,
            hit_timer_slot: None,
            main_and_aux1_timer_decrements_slot: None,
            main_timer_decrement_slot: None,
            zero_hit_timer_clear_slot: None,
            bari_before_random_slot: None,
            throwable_scenery_state_clear_slot: None,
            cucco_subtype_increments: None,
            cucco_helper_ordinal: 0,
            cucco_flee_movement: None,
            active_cucco_movement: None,
            active_cucco_x_publications: 0,
            active_cucco_y_subpixel: None,
            master_sword_light_beam_movement: None,
            master_sword_light_beam_spawn: None,
            cucco_animation_slot: None,
            big_key_drop_graphics_slot: None,
            king_zora_flippers_graphics_slot: None,
            bonk_item_graphics_slot: None,
            wish_pond_tossed_item_graphics_slot: None,
            single_small_draw_position_slot: None,
            probe_after_oam_coordinates_slot: None,
            wallmaster_reset_prefix_slot: None,
            wallmaster_reset_cleared_bytes: None,
            zazak_graphics_slot: None,
            follower_graphics: None,
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
            timers_and_oam_slot: None,
            timers_and_oam_dispatch_state: None,
            initialize_active_main_calls: 0,
            guard_prep_parry_hitbox: None,
            guard_prep_patrol_delay: None,
            guard_prep_tile_collision_return: None,
            guard_animation_checkpoint: None,
            hog_spear_body_graphics_pending: None,
            absorbable_body_active: false,
            absorbable_horizontal_lookup: None,
            absorbable_vertical_lookup: None,
            absorbable_vertical_attribute_loaded: None,
            swamola_segment: None,
            dispatch_trampoline_return: None,
            vitreous_minions_seen: false,
            vitreous_player_damage_pending: None,
            vitreous_ai_pending: None,
            vitreous_damage_pending: None,
            swamola_head_prepared: false,
            swamola_head_draw_completed: None,
            swamola_head_draw: None,
            swamola_segment_draw: None,
            pengator_slide_pending: None,
            antifairy_bounce_pending: None,
            kholdstare_subtype_decremented: false,
            kholdstare_damage_pending: None,
            initialize_prep_pending: None,
            guard_animation_pose_slot: None,
            guard_prep_weapon_flags_pending_slot: None,
            mini_moldorm_history: None,
            initialize_reset_properties: None,
            initialize_load_properties: None,
            fire_debirando_property_reload: false,
            fire_debirando_before_spawn_slot: None,
            fire_debirando_spawn: None,
            antfairy_subtype2_increment_slot: None,
            lanmola_subtype2_increment_slot: None,
            helmasaur_hard_hat_beetle_subtype2_increment_slot: None,
            timer_decrements_slot: None,
            primary_timer_decrements_slot: None,
            hit_timer_slot: None,
            main_and_aux1_timer_decrements_slot: None,
            main_timer_decrement_slot: None,
            zero_hit_timer_clear_slot: None,
            bari_before_random_slot: None,
            throwable_scenery_state_clear_slot: None,
            cucco_subtype_increments: None,
            cucco_helper_ordinal: 0,
            cucco_flee_movement: None,
            active_cucco_movement: None,
            active_cucco_x_publications: 0,
            active_cucco_y_subpixel: None,
            master_sword_light_beam_movement: None,
            master_sword_light_beam_spawn: None,
            cucco_animation_slot: None,
            big_key_drop_graphics_slot: None,
            king_zora_flippers_graphics_slot: None,
            bonk_item_graphics_slot: None,
            wish_pond_tossed_item_graphics_slot: None,
            single_small_draw_position_slot: None,
            probe_after_oam_coordinates_slot: None,
            wallmaster_reset_prefix_slot: None,
            wallmaster_reset_cleared_bytes: None,
            zazak_graphics_slot: None,
            follower_graphics: None,
        });
        tracker.item_receipt_caller = Some(ItemReceiptGraphicsCaller::SpriteMain { slot: 12 });
        let mut receipts = Vec::new();

        tracker
            .consume_event(raw("nmi", Some(0x07_99f0), Some(12), None), &mut receipts)
            .unwrap();
        tracker.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
        for pc in [0x07_e275, 0x07_e27d, 0x07_e27f, 0x07_e2ca, 0x07_e381] {
            let mut tracker = Snes9xOracleSemanticTrace {
                path: PathBuf::new(),
                offset: 0,
                cache_write_progress: None,
                normal_load_ordinal: None,
                pending_reset_progress: None,
                last_host_return_reset_progress: None,
                cached_sprite_execution: None,
                overworld_presence_published: false,
                overworld_sprite_activation: None,
                overworld_load_overlays_sprite_reload_active: false,
                overworld_sprite_reload_reset_published: false,
                rescued_maiden_initialization: None,
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
                host_dialogue_scroll_progress: Vec::new(),
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
    fn nmi_after_spotlight_submodule_return_preserves_the_link_suffix() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
        };
        let mut event = raw(
            "nmi",
            Some(MODULE0F_AFTER_SUBMODULE_DISPATCH_PC),
            None,
            None,
        );
        event.main = Some(0x0f);
        event.sub = Some(1);
        let mut receipts = Vec::new();

        tracker.consume_event(event, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
                OriginalTimingSemanticReceipt::MainLoopInterrupted(
                    MainLoopInterruption::DungeonExitSpotlightAfterSubmodule,
                ),
            ],
        );
    }

    #[test]
    fn recurring_spotlight_table_tail_retains_control_and_link_caller() {
        for pc in [0x00_f423, 0x00_f425, 0x00_f426] {
            assert_eq!(
                main_loop_interruption_for_source_state(pc, Some(0x0f), Some(1), Some(0)),
                Some(MainLoopInterruption::DungeonExitSpotlightTableCompleted)
            );
            assert_ne!(
                main_loop_interruption_for_source_state(pc, Some(0x0f), Some(0), Some(0)),
                Some(MainLoopInterruption::DungeonExitSpotlightTableCompleted)
            );
            assert_ne!(
                main_loop_interruption_for_source_state(pc, Some(6), Some(0), Some(0)),
                Some(MainLoopInterruption::DungeonExitSpotlightTableCompleted)
            );
        }
    }

    #[test]
    fn host_boundary_after_actual_x_velocity_retains_the_pending_y_component() {
        for pc in [0x07_e359, 0x07_e35f, 0x07_e361] {
            assert_eq!(
                main_loop_interruption_for_source_state(pc, Some(0x0f), Some(1), Some(0)),
                Some(MainLoopInterruption::LinkActualVelocityCompleted),
            );
            assert_ne!(
                main_loop_interruption_for_source_state(pc, Some(0x0f), Some(1), Some(1)),
                Some(MainLoopInterruption::LinkActualVelocityCompleted),
            );
        }
        assert_eq!(
            main_loop_interruption_for_source_state(0x07_e245, Some(0x0f), Some(1), Some(0)),
            Some(MainLoopInterruption::LinkPositionBeforeCoordinates),
        );
        assert_eq!(
            main_loop_interruption_for_source_state(0x07_e245, Some(9), Some(0), Some(0)),
            None,
        );
        assert_eq!(
            main_loop_interruption_for_source_state(0x07_e2de, Some(0x0f), Some(1), Some(0)),
            Some(MainLoopInterruption::LinkActualVelocity {
                horizontal_resolved: None
            }),
        );
        assert_eq!(
            main_loop_interruption_for_source_state(
                MODULE0F_LINK_VELOCITY_CALL_PC,
                Some(0x0f),
                Some(1),
                Some(0)
            ),
            Some(MainLoopInterruption::LinkPositionBeforeCoordinates),
        );
        assert_eq!(
            main_loop_interruption_for_source_state(0x07_e352, Some(0x0f), Some(1), Some(0),),
            Some(MainLoopInterruption::LinkActualVelocity {
                horizontal_resolved: Some(true)
            }),
        );
        assert_eq!(
            main_loop_interruption_for_source_state(0x07_e352, Some(0x0f), Some(1), Some(1),),
            Some(MainLoopInterruption::LinkActualVelocity {
                horizontal_resolved: Some(false)
            }),
            "the horizontal pass has not completed while X still names it",
        );
    }

    #[test]
    fn first_dungeon_record_inspection_proves_the_completed_reset_prefix() {
        assert_eq!(
            dungeon_reset_sprites_caller_progress(&raw("frame", Some(0x09_c2a0), Some(2), None)),
            Some(DungeonResetSpritesCpuProgress::LoadBeforeOrigin)
        );
        let mut event = raw("frame", Some(0x09_c32e), Some(19), None);
        event.y = Some(3);
        assert_eq!(
            dungeon_reset_sprites_caller_progress(&event),
            Some(DungeonResetSpritesCpuProgress::LoadStarted)
        );
        event.y = Some(6);
        assert_eq!(dungeon_reset_sprites_caller_progress(&event), None);
    }

    #[test]
    fn pending_y_subpixel_store_retains_the_completed_x_pass() {
        for pc in [0x07_e3a4, 0x07_e3af] {
            assert_eq!(
                main_loop_interruption_for_source_state(pc, Some(0x0f), Some(1), Some(0)),
                Some(MainLoopInterruption::LinkPositionAfterCoordinates { pass: 2 })
            );
        }
        assert_eq!(
            main_loop_interruption_for_source_state(0x07_e3af, Some(0x0f), Some(1), Some(2)),
            None
        );
        assert_eq!(
            main_loop_interruption_for_source_state(0x07_e3af, Some(7), Some(1), Some(0)),
            None
        );
    }

    #[test]
    fn resumed_link_oam_context_retires_its_intermediate_drawing_checkpoint() {
        let mut receipts = vec![
            OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::LatchHeld),
            OriginalTimingSemanticReceipt::MainLoopInterrupted(MainLoopInterruption::LinkOam),
            OriginalTimingSemanticReceipt::LinkOamStairProgress(
                zelda3::LinkOamStairProgress::EquipmentSelection,
            ),
        ];
        retire_resumed_main_loop_interruption(&mut receipts, None).unwrap();
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::NmiAccepted(
                NmiUpdateGate::LatchHeld
            )]
        );
    }

    #[test]
    fn link_position_after_coordinates_names_the_completed_axis() {
        assert_eq!(
            main_loop_interruption_for_source_state(
                LINK_POSITION_AFTER_COORDINATES_START_PC,
                Some(0x0f),
                Some(1),
                Some(2),
            ),
            Some(MainLoopInterruption::LinkPositionAfterCoordinates { pass: 2 }),
        );
        assert_eq!(
            main_loop_interruption_for_source_state(
                LINK_POSITION_AFTER_COORDINATES_START_PC,
                Some(0x0f),
                Some(1),
                Some(1),
            ),
            None,
            "the loop's transient first DEX value is not a completed axis",
        );
        assert_eq!(
            main_loop_interruption_for_source_state(0x07e3d3, Some(0x0f), Some(1), Some(0xfffe),),
            Some(MainLoopInterruption::LinkPositionAfterCoordinates { pass: 0 }),
            "the final loop epilogue has committed Y even though X is no longer the pass",
        );
    }

    #[test]
    fn link_position_low_coordinate_store_is_a_distinct_source_boundary() {
        assert_eq!(
            main_loop_interruption_for_source_state(0x07_e3ca, Some(0x0f), Some(1), Some(0),),
            Some(MainLoopInterruption::LinkPositionAfterCoordinateLow { pass: 0 }),
        );
        assert_eq!(
            main_loop_interruption_for_source_state(0x07_e3cd, Some(0x0f), Some(1), Some(2),),
            Some(MainLoopInterruption::LinkPositionAfterCoordinateLow { pass: 2 }),
            "the whole interval before the high-byte store is the same source boundary",
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
    fn host_return_after_pure_circle_helper_precedes_the_first_table_publication() {
        // Route host 682798 returns at $00:F37B: ASL has doubled the upper
        // cursor, but the guarded upper-table STA at $00:F383 is still pending.
        // Re-running the pure helper is state-equivalent; advancing to the
        // table store is not.
        let mut event = frame_with_sub("return", 17_288, 0x0f, 0);
        event.pc = Some(0x00f37b);
        event.a = Some(286);
        event.x = Some(98);
        event.link_y = Some(3060);
        event.bg2_v = Some(2833);
        event.spotlight_radius = Some(126);
        event.spotlight_var4_low = Some(96);
        event.spotlight_lower_cursor = Some(335);

        assert_eq!(
            spotlight_table_build_progress(&event, None, None).unwrap(),
            Some(SpotlightTableBuildProgress {
                completed_iterations: 143,
                checkpoint: SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                    pending_circle_input: 97,
                },
            }),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
    fn host_return_before_spotlight_iteration_value_load_reports_pending_iteration() {
        // Route host 155201 returns at $00:F361 after 172 row-pair iterations.
        // The next iteration has not initialized its local value, while the
        // lower cursor and spotlight scratch independently identify the exact
        // source loop position.
        let mut event = frame_with_sub("return", 155_201, 0x10, 1);
        event.pc = Some(IRIS_SPOTLIGHT_ITERATION_VALUE_LOAD_PC);
        event.link_y = Some(3112);
        event.bg2_v = Some(3072);
        event.spotlight_radius = Some(119);
        event.spotlight_var4_low = Some(1);
        event.spotlight_lower_cursor = Some(52);
        let mut receipts = Vec::new();

        publish_spotlight_host_return_progress(&event, None, None, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 172,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeIterationInitialization,
                    },
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            )],
        );
    }

    #[test]
    fn host_return_after_lower_spotlight_store_keeps_loop_test_pending() {
        // Route host 104340 returns at $00:F398. The visible upper row for
        // cursor 211 has been stored, the lower row at 265 was clipped, and
        // the source loop-completion comparison has not executed yet.
        let mut event = frame_with_sub("return", 104_340, 0x0f, 0);
        event.pc = Some(0x00f398);
        event.x = Some(422);
        event.link_y = Some(6644);
        event.bg2_v = Some(6418);
        event.spotlight_radius = Some(126);
        event.spotlight_var4_low = Some(27);
        event.spotlight_lower_cursor = Some(265);
        let mut receipts = Vec::new();

        publish_spotlight_host_return_progress(&event, None, None, &mut receipts).unwrap();

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpotlightTableBuildProgress(
                SpotlightTableBuildProgressReceipt {
                    progress: SpotlightTableBuildProgress {
                        completed_iterations: 211,
                        checkpoint: SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                            upper_cursor: 211,
                            lower_cursor: 265,
                        },
                    },
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            )],
        );
    }

    #[test]
    fn nmi_in_spotlight_iteration_bound_branch_rewinds_before_publication() {
        // Route host 752654 accepts NMI at $00:F36B. The iteration-local
        // value has been initialized and the upper-bound comparison has run,
        // but the radius scratch decrement, pure circle call, and both HDMA
        // table stores are still pending.
        let mut event = raw("nmi", Some(0x00f36b), None, None);
        event.main = Some(0x10);
        event.sub = Some(1);
        event.link_y = Some(1335);
        event.bg2_v = Some(1254);
        event.spotlight_radius = Some(119);
        event.spotlight_var4_low = Some(4);
        event.spotlight_lower_cursor = Some(96);

        assert_eq!(
            spotlight_table_build_progress(&event, None, None).unwrap(),
            Some(SpotlightTableBuildProgress {
                completed_iterations: 128,
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
    fn spotlight_beam_wait_has_completed_rows_but_no_projection_words() {
        for pc in IRIS_SPOTLIGHT_BEAM_WAIT_PCS {
            let mut event = raw("nmi", Some(pc), Some(178), None);
            event.main = Some(0x10);
            event.sub = Some(1);
            event.link_y = Some(2838);
            event.bg2_v = Some(2761);
            event.spotlight_radius = Some(119);
            assert_eq!(
                spotlight_table_build_progress(&event, None, None).unwrap(),
                Some(SpotlightTableBuildProgress {
                    completed_iterations: 136,
                    checkpoint: SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words: 0 },
                })
            );
        }
    }

    #[test]
    fn clipped_spotlight_helper_index_cannot_alias_an_earlier_visible_row() {
        // At $F396, X=10 is the circle helper index for input8/radius98,
        // not the byte offset of visible row5. r6=246 binds this event to232.
        let mut event = raw("nmi", Some(0x00f396), Some(10), None);
        event.main = Some(0x0f);
        event.sub = Some(1);
        event.link_y = Some(8692);
        event.bg2_v = Some(8465);
        event.spotlight_radius = Some(98);
        event.spotlight_var4_low = Some(7);
        event.spotlight_lower_cursor = Some(246);
        assert_eq!(
            spotlight_table_build_progress(&event, None, None).unwrap(),
            Some(SpotlightTableBuildProgress {
                completed_iterations: 232,
                checkpoint: SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                    upper_cursor: 232,
                    lower_cursor: 246,
                },
            })
        );
        event.spotlight_lower_cursor = None;
        assert!(spotlight_table_build_progress(&event, None, None).is_err());
    }

    #[test]
    fn spotlight_entry_before_upper_increment_keeps_the_loop_pending() {
        // Pinned ROM host 281498 returns at INC r4 ($00:F39C), before
        // that write: center=239, upper=217, lower=261, X=2*upper.
        let mut event = raw("frame", Some(0x00f39c), Some(434), None);
        event.main = Some(0x0f);
        event.sub = Some(0);
        event.link_y = Some(3573);
        event.bg2_v = Some(3346);
        event.spotlight_radius = Some(126);
        event.spotlight_var4_low = Some(22);
        event.spotlight_lower_cursor = Some(261);
        assert_eq!(
            spotlight_table_build_progress(&event, None, None).unwrap(),
            Some(SpotlightTableBuildProgress {
                completed_iterations: 217,
                checkpoint: SpotlightTableBuildCheckpoint::BeforeLoopCompletionTest {
                    upper_cursor: 217,
                    lower_cursor: 261,
                },
            })
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
    fn game_over_sprite_reset_requires_its_exact_death_caller() {
        let mut event = frame_with_sub("return", 609, 0x12, 9);
        event.pc = Some(0x09_c47f);
        event.return_address = Some(0x09_f58b);
        let mut receipts = Vec::new();
        assert!(publish_pre_dungeon_sprite_reset_progress(
            &event,
            OriginalTimingBoundary::NmiAccepted,
            false,
            &mut receipts
        )
        .unwrap());
        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::SpriteResetAllProgress(
                SpriteResetAllProgressReceipt {
                    progress: SpriteResetAllProgress::SpriteDisableAllCompleted,
                    boundary: OriginalTimingBoundary::NmiAccepted
                },
            )]
        );
        event.return_address = Some(0x09_f58c);
        assert!(!publish_pre_dungeon_sprite_reset_progress(
            &event,
            OriginalTimingBoundary::NmiAccepted,
            false,
            &mut Vec::new()
        )
        .unwrap());
        event.return_address = Some(0x09_f58b);
        event.sub = Some(8);
        assert!(!publish_pre_dungeon_sprite_reset_progress(
            &event,
            OriginalTimingBoundary::NmiAccepted,
            false,
            &mut Vec::new()
        )
        .unwrap());
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
            false,
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
            false,
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
                false,
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
    fn completed_dungeon_sprite_record_survives_to_host_return() {
        let mut tracker = empty_semantic_tracker();
        tracker.normal_load_ordinal = Some(5);
        let mut receipts = Vec::new();
        let slot = 6u16;
        let writes = [
            (
                DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC,
                SPRITE_STATE_BASE + slot,
            ),
            (DUNGEON_LOAD_SINGLE_SPRITE_TEMP_Y_PC, DUNGEON_LOAD_TEMP_Y),
            (
                DUNGEON_LOAD_SINGLE_SPRITE_FLOOR_PC,
                SPRITE_FLOOR_BASE + slot,
            ),
            (
                DUNGEON_LOAD_SINGLE_SPRITE_Y_LOW_PC,
                SPRITE_Y_LOW_BASE + slot,
            ),
            (
                DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC,
                SPRITE_Y_HIGH_BASE + slot,
            ),
            (
                DUNGEON_LOAD_SINGLE_SPRITE_SHARED_X_PC,
                DUNGEON_LOAD_SHARED_X,
            ),
            (
                DUNGEON_LOAD_SINGLE_SPRITE_X_LOW_PC,
                SPRITE_X_LOW_BASE + slot,
            ),
            (
                DUNGEON_LOAD_SINGLE_SPRITE_X_HIGH_PC,
                SPRITE_X_HIGH_BASE + slot,
            ),
            (DUNGEON_LOAD_SINGLE_SPRITE_TYPE_PC, SPRITE_TYPE_BASE + slot),
            (
                DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_CLEAR_PC,
                SPRITE_SUBTYPE_BASE + slot,
            ),
            (
                DUNGEON_LOAD_SINGLE_SPRITE_TEMP_SUBTYPE_PC,
                DUNGEON_LOAD_TEMP_Y,
            ),
            (
                DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_FINAL_PC,
                SPRITE_SUBTYPE_BASE + slot,
            ),
            (
                DUNGEON_LOAD_SINGLE_SPRITE_SPAWN_INDEX_PC,
                SPRITE_N_WORD_BASE + slot,
            ),
            (
                DUNGEON_LOAD_SINGLE_SPRITE_COMPLETE_PC,
                SPRITE_DIE_ACTION_BASE + slot,
            ),
        ];
        for (pc, address) in writes {
            tracker
                .consume_event(
                    raw("wram-write", Some(pc), Some(slot), Some(address)),
                    &mut receipts,
                )
                .unwrap();
        }
        tracker.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);

        assert_eq!(
            receipts,
            vec![OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                DungeonResetSpritesProgressReceipt {
                    progress: DungeonResetSpritesCpuProgress::Load(DungeonLoadSpritesCpuProgress {
                        normal_load_ordinal: 6,
                        slot: 6,
                        checkpoint: DungeonSpriteLoadCheckpoint::Complete,
                    },),
                    boundary: OriginalTimingBoundary::HostReturn,
                },
            )],
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
    fn leading_nmi_does_not_republish_the_same_reset_checkpoint() {
        let progress = DungeonResetSpritesCpuProgress::RoomHistorySearchStarted;
        let mut tracker = empty_semantic_tracker();
        tracker.pending_reset_progress = Some(progress);
        let mut first_host = Vec::new();
        tracker.flush_reset_progress(&mut first_host, OriginalTimingBoundary::HostReturn);
        assert_eq!(first_host.len(), 1);
        let checkpoint = tracker.checkpoint();
        let mut resumed = empty_semantic_tracker();
        resumed.restore_checkpoint(checkpoint).unwrap();
        resumed.pending_reset_progress = Some(progress);
        let mut next_host = Vec::new();
        resumed.flush_reset_progress(&mut next_host, OriginalTimingBoundary::NmiAccepted);
        assert!(next_host.is_empty());

        tracker.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::LoadBeforeOrigin);
        tracker.flush_reset_progress(&mut next_host, OriginalTimingBoundary::NmiAccepted);
        assert_eq!(
            next_host,
            vec![OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                DungeonResetSpritesProgressReceipt {
                    progress: DungeonResetSpritesCpuProgress::LoadBeforeOrigin,
                    boundary: OriginalTimingBoundary::NmiAccepted,
                },
            )]
        );
    }

    #[test]
    fn sprite_disable_progress_refines_across_host_return_then_nmi() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
    fn later_source_write_refines_the_y_high_candidate() {
        let path = env::temp_dir().join("unused-snes9x-semantic-test.jsonl");
        let mut tracker = Snes9xOracleSemanticTrace {
            path,
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: Some(0),
            pending_reset_progress: None,
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
            vec![
                OriginalTimingSemanticReceipt::DungeonResetSpritesProgress(
                    DungeonResetSpritesProgressReceipt {
                        progress: DungeonResetSpritesCpuProgress::Load(
                            DungeonLoadSpritesCpuProgress {
                                normal_load_ordinal: 0,
                                slot: 0,
                                checkpoint: DungeonSpriteLoadCheckpoint::XLow,
                            },
                        ),
                        boundary: OriginalTimingBoundary::NmiAccepted,
                    },
                ),
                OriginalTimingSemanticReceipt::NmiAccepted(NmiUpdateGate::Open),
            ]
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
            last_host_return_reset_progress: None,
            cached_sprite_execution: None,
            overworld_presence_published: false,
            overworld_sprite_activation: None,
            overworld_load_overlays_sprite_reload_active: false,
            overworld_sprite_reload_reset_published: false,
            rescued_maiden_initialization: None,
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
            host_dialogue_scroll_progress: Vec::new(),
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
