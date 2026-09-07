//! Temporary pinned-Snes9x adapter for Zelda-level semantic receipts.
//!
//! Emulator PCs and WRAM addresses are allowed only in this replaceable host
//! adapter. Translated gameplay receives the typed values from `zelda3`.

use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
mod consume_event_arms;

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
    SpriteResetAllProgressReceipt, SpriteTileCollisionStage,
    TriforceRoomCase2PaletteProgressReceipt,
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
// are complete once the ROM begins loading `sprite_hit_timer` at `$06:8441`
// (route host 1514571 returned on that opcode). Include the possible
// mid-instruction host-return PCs through the AND operand; no hit-timer
// statement has published at any address in this interval.
const SPRITE_PRIMARY_TIMER_DECREMENTS_COMPLETE_START_PC: u32 = 0x068441;
const SPRITE_PRIMARY_TIMER_DECREMENTS_COMPLETE_END_PC: u32 = 0x068449;
// When `sprite_hit_timer & $7f` is zero, the branch at `$06:8446` jumps
// directly to the `STZ sprite_hit_timer,X` instruction. Its opcode and operand
// fetches are still before that store publishes, so they name the same exact
// source boundary as the linear hit-timer load/branch interval above.
const SPRITE_PRIMARY_TIMER_DECREMENTS_ZERO_HIT_STORE_START_PC: u32 = 0x068496;
const SPRITE_PRIMARY_TIMER_DECREMENTS_ZERO_HIT_STORE_END_PC: u32 = 0x068499;
// `$06:8431` is the aux2 load after the main/aux1 countdown statements.
// A host may return on its opcode or be interrupted during that instruction,
// before aux2 has executed.
const SPRITE_MAIN_AND_AUX1_TIMER_DECREMENTS_COMPLETE_START_PC: u32 = 0x068431;
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
const SPRITE_OAM_FLAGS_BASE: u16 = 0x0f50;
const SPRITE_OBJECT_PRIORITY_BASE: u16 = 0x0b89;

/// PCs that belong to `LaserEye_Draw` after its prologue stores: the rest
/// of the draw routine, `Sprite_DrawMultiple` ($05:DF6C..$05:E012) and
/// `Sprite_PrepOamCoord*` ($06:E416..$06:E4AA).
fn laser_eye_draw_in_flight_pc(pc: u32) -> bool {
    (0x1e_a71f..=0x1e_a741).contains(&pc)
        || (0x05_df6c..=0x05_e012).contains(&pc)
        || (0x06_e416..=0x06_e4aa).contains(&pc)
}
const SPRITE_Z_BASE: u16 = 0x0f70;
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
/// `SpritePrep_Main`'s per-type jump table ($06:865B, indexed by sprite type):
/// the first instruction of every type-specific prep routine. A host that
/// returns at one of these under Sprite_ExecuteSingle's return has published
/// the properties and state 9 but none of the prep.
/// State-8 prep entries in bank $06 that are a single `JSL` into another
/// bank whose target begins `PHB; PHK; PLB` (the ROM's `_bounce` shape):
/// `(entry, trampoline)`. Nothing of the prep has run until the instruction
/// after `PLB`, so a boundary anywhere in the trampoline prologue is still
/// the pending-prep checkpoint.
const SPRITE_PREP_BOUNCE_TRAMPOLINES: [(u16, u32); 14] = [
    (0x8854, 0x05_f25a),
    (0x886e, 0x05_e675),
    (0x8b03, 0x1e_a4e7),
    (0x8ba2, 0x05_8192),
    (0xbfe5, 0x05_da29),
    (0xbff9, 0x1e_e8f1),
    (0xc026, 0x05_e675),
    (0xc030, 0x05_e675),
    (0xc05d, 0x05_e88e),
    (0xc06c, 0x05_ebc7),
    (0xc07b, 0x05_ee4b),
    (0xc094, 0x05_f521),
    (0xc09e, 0x05_ef01),
    (0xc0a8, 0x05_ef01),
];

/// Whether `pc` sits in a state-8 prep bounce trampoline's `PHB; PHK; PLB`
/// prologue (the next real instruction pending) with the bank-$06 `JSL`
/// return (`entry + 3`, the JSL's last byte) where the trampoline's pushes
/// leave it.
fn sprite_prep_bounce_pending(pc: u32, return_address: Option<u32>, stack4: Option<u8>) -> bool {
    let Some(return_address) = return_address else {
        return false;
    };
    SPRITE_PREP_BOUNCE_TRAMPOLINES
        .iter()
        .any(|&(entry, trampoline)| {
            let rtl_return = u32::from(entry) + 3;
            match pc.checked_sub(trampoline) {
                // PHB pending: the JSL return sits on top.
                Some(0) => return_address & 0x00ff_ffff == 0x06_0000 | rtl_return,
                // PHK pending (B pushed) or the post-PLB instruction pending
                // (K popped back into B).
                Some(1) | Some(3) => {
                    (return_address >> 8) & 0xffff == rtl_return && stack4 == Some(0x06)
                }
                // PLB pending: K and B sit above the return.
                Some(2) => {
                    (return_address >> 16) & 0xff == rtl_return & 0xff
                        && stack4 == Some((rtl_return >> 8) as u8)
                }
                _ => false,
            }
        })
}

const SPRITE_PREP_ENTRY_PCS: [u16; 256] = [
    0x8969, 0x897e, 0x8873, 0x0000, 0x8859, 0x8873, 0x8859, 0x886d, 0x8f71, 0x8f8a, 0x8f71, 0x8873,
    0x8873, 0x8873, 0x8873, 0x8910, 0x8873, 0x8873, 0x8873, 0x9151, 0x8bc4, 0x8ef2, 0x8cde, 0x8873,
    0x89d3, 0x8991, 0x8a79, 0x8873, 0x8b12, 0x8ba7, 0x9064, 0x8d7f, 0x8873, 0x8b34, 0x8873, 0x8b1c,
    0x8b1c, 0x9043, 0x9122, 0x8873, 0x8c9e, 0x8cc1, 0x8ba7, 0x8dfd, 0x8dda, 0x8ba7, 0x9075, 0x8ba7,
    0x8ba7, 0x8de0, 0x8ba7, 0x8bcf, 0xc026, 0xc030, 0x8ba7, 0x8ba7, 0x8873, 0x8d59, 0x8dda, 0x8cf2,
    0x8ba7, 0x886e, 0x8873, 0x8873, 0x891b, 0x8fd6, 0x8fd6, 0x8fd6, 0x9001, 0x9001, 0x9001, 0x9001,
    0x9001, 0x9001, 0x9001, 0x8b81, 0x8fa7, 0x8fb0, 0x8b08, 0x8b0c, 0x8873, 0x8f6c, 0x8f0f, 0x8f3f,
    0x8f95, 0x8fa2, 0x8fc9, 0x8f4d, 0x8873, 0x8ec1, 0x8ed2, 0x8e85, 0x8e85, 0x8e4f, 0x8e53, 0x8e42,
    0x8e46, 0x8873, 0x8e30, 0x8b4a, 0x8b3e, 0x8ba2, 0x8b93, 0x8b93, 0x8b93, 0x8b93, 0x8873, 0x8873,
    0x8873, 0x8878, 0x88aa, 0x888e, 0x91ae, 0x8de9, 0x8ba7, 0xbfe5, 0xc05d, 0x8ba7, 0xc06c, 0x8ef2,
    0x8dd1, 0x916e, 0x9195, 0x91ae, 0x8b2e, 0x8f9d, 0x91b4, 0x91b4, 0x91ae, 0x91ae, 0x9248, 0x91af,
    0x91af, 0x91ae, 0x8e6b, 0x91ae, 0x922f, 0x91ae, 0x91d7, 0x91ae, 0x91f1, 0x91fa, 0x91ae, 0x91e8,
    0x91ae, 0x91ae, 0x91c5, 0x9107, 0x8873, 0x8b03, 0x8b03, 0x8b03, 0x8b03, 0x8873, 0x8fb0, 0x8873,
    0x8af3, 0x8af0, 0x8bb2, 0x8bab, 0x8bab, 0x90cc, 0x90fa, 0x90f0, 0x8f08, 0x90d5, 0x90d5, 0x90e0,
    0x89d8, 0x89d8, 0x899b, 0x924d, 0x916e, 0xbff9, 0x8873, 0x8873, 0x8873, 0x8873, 0x9175, 0x90d6,
    0x8a59, 0x89df, 0x8d46, 0x899c, 0x8873, 0x8a51, 0x8cd5, 0x8bf1, 0x8ba7, 0x894d, 0x88fd, 0x8873,
    0x892c, 0x893b, 0x8873, 0x8901, 0x8873, 0x8ba7, 0x8ba7, 0x88df, 0x8dc6, 0x8d94, 0x8dc1, 0x91ba,
    0x91ba, 0x91ba, 0x88c7, 0x88c0, 0x8873, 0x8873, 0x8ba7, 0x91dc, 0x8ba7, 0x8bbf, 0x88cf, 0x88cf,
    0x916a, 0x916a, 0x916a, 0x916a, 0x916a, 0x916a, 0x916a, 0x916a, 0x916a, 0x916a, 0x916a, 0x915c,
    0x9262, 0x924e, 0x9174, 0xc07b, 0xc085, 0xc094, 0xc09e, 0xc0a8, 0x850f, 0x8873, 0x8841, 0x8873,
    0x8873, 0x8873, 0x8854, 0x00bd, 0x180d, 0x0369, 0x009d, 0xbd0d, 0x0d10, 0x6918, 0x9d08, 0x0d10,
    0x2260, 0xf25a, 0x6005, 0x8ead,
];
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
    hog_spear_active_body: bool,
    buzzblob_movement: Option<(u8, bool)>,
    trinexx_head_draw: Option<(u8, u8)>,
    /// The last traced `$0FB5` store: TrinexxHead_Draw's segment counter.
    trinexx_segment_counter: Option<u8>,
    trinexx_head_draw_setup: Option<u8>,
    trinexx_breath_tile_collision: Option<u8>,
    /// The active slot's type handler returned into the bank-$1D dispatcher
    /// wrapper ($1D:C21A) or its $06:BFF4 trampoline; only returns remain
    /// before Sprite_Main's slot loop, so the slot is complete.
    handler_returned_slot: Option<u8>,
    /// The active slot ran `Sprite_Trinexx_FinalPhase` state 0's `--sprite_A`
    /// store ($1D:AE77) this run.
    trinexx_final_phase_case0: Option<u8>,
    trinexx_final_phase_tile_collision: Option<(u8, bool)>,
    helmasaur_hard_hat_tile_collision: Option<(u8, SpriteTileCollisionStage)>,
    /// The last traced `$0FB6` store: Sprite_TrinexxD_Draw's segment counter
    /// while its loop runs.
    trinexx_d_draw_counter: Option<u8>,
    /// The active slot began Sprite_TrinexxD_Draw's body loop (`STA $0FB6`
    /// of 0 at $1D:AF91) this run.
    trinexx_d_draw_active: Option<u8>,
    trinexx_final_phase_draw: Option<(u8, u8, u8)>,
    sidenexx_neck_target: Option<(u8, u8)>,
    trinexx_front_part: Option<(u8, u8)>,
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
    moblin_collision_started: bool,
    moblin_collision_geometry: Option<u8>,
    moblin_attribute_loaded: Option<u8>,
    vitreous_player_damage_pending: Option<u8>,
    vitreous_ai_pending: Option<u8>,
    mini_moldorm_ai_pending: Option<u8>,
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
    /// A state-8 prep (`SpritePrep_Spike` / `SpritePrep_RockStal`) stopped
    /// inside its `Sprite_MoveY` (or at the velocity clear after it) with the
    /// named assignment completed.
    initialize_prep_move_y: Option<(u8, SpriteMoveXYCheckpoint)>,
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
    trinexx_death_spawn: Option<(u8, u8, SpriteDynamicSpawnProgress)>,
    /// `Sprite_Agahnim_ApplyMotionBlur`'s type-$C1 afterimage spawn in
    /// flight: parent slot, spawned slot, and the shared-helper progress.
    agahnim_motion_blur_spawn: Option<(u8, u8, SpriteDynamicSpawnProgress)>,
    #[serde(default)]
    antfairy_subtype2_increment_slot: Option<u8>,
    #[serde(default)]
    lanmola_subtype2_increment_slot: Option<u8>,
    lanmola_draw_prefix: Option<(u8, u8)>,
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
    /// An indoor Boulder's `Sprite_MoveXYZ`: the active slot, the count of
    /// published `Sprite_MoveXY` coordinate stores, and whether `Sprite_MoveZ`
    /// stored its result.
    boulder_movement: Option<(u8, u8, bool)>,
    /// A Zora fireball (`Sprite_Fireball`) in the active slot, its completed
    /// `Sprite_MoveXY` coordinate stores in source order, and whether the
    /// movement is known complete (the tile-collision entry followed it).
    zora_fireball: Option<(u8, u8, bool)>,
    /// A laser eye (`Sprite_95_LaserEyeLeft`) whose `LaserEye_Draw` prologue
    /// stored the object priority; its `Sprite_DrawMultiple` is in flight.
    laser_eye_draw_prologue: Option<u8>,
    #[serde(default)]
    master_sword_light_beam_spawn: Option<(u8, u8, SpriteDynamicSpawnProgress)>,
    #[serde(default)]
    cucco_animation_slot: Option<(u8, u8)>,
    #[serde(default)]
    big_key_drop_graphics_slot: Option<u8>,
    #[serde(default)]
    king_zora_flippers_graphics_slot: Option<u8>,
    happiness_pond_rupee_graphics_slot: Option<u8>,
    catfish_medallion_graphics_slot: Option<u8>,
    waterfall_gt_cutscene_graphics_slot: Option<u8>,
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

    fn observe_moblin_collision_geometry(&mut self, event: &RawTraceEvent) {
        // PHX preserves the slot above the vertical probe's JSR return.
        // The attribute is published; classification has no effects yet.
        if self.moblin_collision_started
            && event.pc == Some(0x06_e818)
            && event
                .return_address
                .is_some_and(|stack| stack >> 8 == 0xe5f0 && self.current_slot == Some(stack as u8))
        {
            self.moblin_attribute_loaded = self.current_slot;
        }
        // First downward tile probe: only local outdoor coordinates exist;
        // the caller has moved and cleared wallcoll, but no tile is read yet.
        let geometry = event
            .pc
            .is_some_and(|pc| (0x06_e773..=0x06_e795).contains(&pc))
            && event
                .return_address
                .is_some_and(|stack| stack & 0xffff == 0xe5f0)
            && self.current_slot.map(u16::from) == event.x;
        let direction_setup = event
            .pc
            .is_some_and(|pc| (0x06_e72f..0x06_e73c).contains(&pc))
            && event
                .return_address
                .is_some_and(|stack| stack & 0xffff == 0xe5f0)
            && self.current_slot.map(u16::from) == event.x
            && event.y == Some(1);
        // The outdoor attribute lookup has returned, but STA sprite_tiletype
        // is pending. PHY/PHX preserve direction and slot above JSR $E7A0.
        let attribute_pending = matches!(event.pc, Some(0x06_e8ce | 0x06_e8d0))
            && event.return_address.is_some_and(|stack| {
                stack & 0xff00ff == 0xa00002 && self.current_slot == Some((stack >> 8) as u8)
            });
        let attribute_lookup = event
            .pc
            .is_some_and(|pc| (0x00_882e..0x00_8888).contains(&pc))
            && event.return_address == Some(0x06_e8cd);
        if self.moblin_collision_started
            && (direction_setup
                || ((geometry || attribute_pending || attribute_lookup) && event.y == Some(2)))
        {
            self.moblin_collision_geometry = self.current_slot;
        }
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
        self.initialize_prep_move_y = None;
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
        self.initialize_prep_move_y = None;
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
        self.initialize_prep_move_y = None;
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
        self.initialize_prep_move_y = None;
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

    fn observe_trinexx_head_draw(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if event.event == "wram-write" {
            // STZ/INC $0FB5 publish the neck segment being computed or
            // drawn; boundaries after the angle calculation pulled the
            // cursor out of Y and X, so the counter is the only source.
            if event.address == Some(0x0fb5) {
                self.trinexx_segment_counter = event.value;
            }
            return Ok(());
        }
        if !matches!(event.event.as_str(), "nmi" | "frame") {
            return Ok(());
        }
        self.trinexx_head_draw = None;
        self.trinexx_front_part = None;
        self.trinexx_head_draw_setup = None;
        if matches!(event.pc, Some(0x06_84c0 | 0x1d_bb8c)) {
            // Sidenexx copied its base position into the sprite and
            // Sprite_Get16BitCoords published it; only the RTL (or the
            // pending JSR to the OAM coordinate prep) remains.
            let slot = self
                .current_slot
                .ok_or("Trinexx head-draw setup omitted its active slot")?;
            if event.x != Some(u16::from(slot)) {
                return Err("Trinexx head-draw setup has the wrong active slot".into());
            }
            if event.pc == Some(0x06_84c0)
                && (event.return_address.map(|r| r & 0x00ff_ffff) != Some(0x1d_bb8b)
                    || event.stack4 != Some(0xdc))
            {
                return Err("Trinexx head-draw setup has the wrong source stack".into());
            }
            if event.pc == Some(0x1d_bb8c)
                && event.return_address.map(|r| r & 0xffff) != Some(0xb8dc)
            {
                return Err("Trinexx head-draw setup has the wrong caller".into());
            }
            self.trinexx_head_draw_setup = Some(slot);
            return Ok(());
        }
        if matches!(
            event.pc,
            Some(0x1d_bce0 | 0x1d_bce1 | 0x1d_bce2 | 0x1d_bce3 | 0x1d_bce4 | 0x1d_bce6 | 0x1d_bce8)
        ) {
            let slot = self
                .current_slot
                .ok_or("Trinexx first-part drawing omitted its active slot")?;
            let entry = event
                .x
                .filter(|&x| x < 5)
                .ok_or("Trinexx first-part drawing has an invalid entry")?
                as u8;
            let y = entry * 4 + 3;
            // PHY, PHX and the local JSR return to $1D:BC39 retain the
            // OAM byte index and active sprite above the drawing caller.
            if event.return_address != Some(0x39_0000 | (u32::from(slot) << 8) | u32::from(y))
                || event.stack4 != Some(0xbc)
            {
                return Err("Trinexx first-part drawing has the wrong source stack".into());
            }
            self.trinexx_front_part = Some((
                slot,
                entry * 5 + if event.pc == Some(0x1d_bce8) { 5 } else { 4 },
            ));
            return Ok(());
        }

        // The first-part OAM loop body ($1D:BCAB..BCDF and the INY/INX after
        // PLY). X is the OAM entry being built; only PHX (the slot) and the
        // local JSR return ($1D:BC39) sit above the Sidenexx caller. Each
        // arm names how many of the entry's five stores have executed; the
        // cur_sprite scratch bytes it also writes are re-derived on resume.
        let loop_stores = match event.pc {
            Some(
                0x1d_bcab | 0x1d_bcae | 0x1d_bcaf | 0x1d_bcb1 | 0x1d_bcb4 | 0x1d_bcb5 | 0x1d_bcb8,
            ) => Some(0u8),
            Some(
                0x1d_bcba | 0x1d_bcbd | 0x1d_bcbe | 0x1d_bcc0 | 0x1d_bcc3 | 0x1d_bcc4 | 0x1d_bcc7
                | 0x1d_bcc9 | 0x1d_bccb | 0x1d_bccc | 0x1d_bcce | 0x1d_bccf,
            ) => Some(1),
            Some(0x1d_bcd1 | 0x1d_bcd4 | 0x1d_bcd5) => Some(2),
            Some(0x1d_bcd7 | 0x1d_bcd9 | 0x1d_bcdc | 0x1d_bcdd) => Some(3),
            Some(0x1d_bcdf) => Some(4),
            Some(0x1d_bce9 | 0x1d_bcea) => Some(5),
            _ => None,
        };
        if let Some(stores) = loop_stores {
            let slot = self
                .current_slot
                .ok_or("Trinexx first-part loop omitted its active slot")?;
            let entry = event
                .x
                .filter(|&x| x < 5)
                .ok_or("Trinexx first-part loop has an invalid entry")?
                as u8;
            if event.return_address.map(|r| r & 0x00ff_ffff) != Some(0xbc_3900 | u32::from(slot))
                || event.stack4 != Some(0xdc)
            {
                return Err("Trinexx first-part loop has the wrong source stack".into());
            }
            self.trinexx_front_part = Some((slot, entry * 5 + stores));
            return Ok(());
        }
        // After INX the loop counter names the completed entries; the CPX,
        // the loop branch and the closing PLX store nothing.
        if matches!(event.pc, Some(0x1d_bceb | 0x1d_bced | 0x1d_bcef)) {
            let slot = self
                .current_slot
                .ok_or("Trinexx first-part loop omitted its active slot")?;
            let entries = event
                .x
                .filter(|&x| (1..=5).contains(&x))
                .ok_or("Trinexx first-part loop exit has an invalid counter")?
                as u8;
            if event.return_address.map(|r| r & 0x00ff_ffff) != Some(0xbc_3900 | u32::from(slot))
                || event.stack4 != Some(0xdc)
            {
                return Err("Trinexx first-part loop exit has the wrong source stack".into());
            }
            self.trinexx_front_part = Some((slot, entries * 5));
            return Ok(());
        }
        // The prologue ($1D:BCA0..BCA8) has stored nothing yet: subtype
        // scratch, PHX and the entry/OAM cursor loads precede the loop.
        if matches!(
            event.pc,
            Some(0x1d_bca0 | 0x1d_bca3 | 0x1d_bca5 | 0x1d_bca6 | 0x1d_bca8)
        ) {
            let slot = self
                .current_slot
                .ok_or("Trinexx first-part prologue omitted its active slot")?;
            let pushed = matches!(event.pc, Some(0x1d_bca6 | 0x1d_bca8));
            let expected_x = if event.pc == Some(0x1d_bca8) {
                0
            } else {
                u16::from(slot)
            };
            let stack_ok = if pushed {
                event.return_address.map(|r| r & 0x00ff_ffff) == Some(0xbc_3900 | u32::from(slot))
                    && event.stack4 == Some(0xdc)
            } else {
                event.return_address.map(|r| r & 0x00ff_ffff) == Some(0xdc_bc39)
                    && event.stack4 == Some(0xb8)
            };
            if event.x != Some(expected_x) || !stack_ok {
                return Err("Trinexx first-part prologue has the wrong source stack".into());
            }
            self.trinexx_front_part = Some((slot, 0));
            return Ok(());
        }
        // The first-part tail after the OAM entries: the shared scratch
        // advance ($1D:BCF0..BCF6), then Sprite_SetX/SetY's four stores.
        // Only the local JSR return ($1D:BC39) and the Sidenexx caller
        // ($1D:B8DC) remain on the stack above the head-draw code.
        let tail_completed = match event.pc {
            Some(0x1d_bcf0 | 0x1d_bcf3 | 0x1d_bcf4) => Some(25u8),
            Some(
                0x1d_bcf9 | 0x1d_bcfb | 0x1d_bcfe | 0x1d_bd00 | 0x1d_bd01 | 0x1d_bd02 | 0x1d_bd05,
            ) => Some(26),
            Some(0x1d_bd08 | 0x1d_bd09 | 0x1d_bd0c) => Some(27),
            Some(
                0x1d_bd0f | 0x1d_bd11 | 0x1d_bd14 | 0x1d_bd16 | 0x1d_bd17 | 0x1d_bd18 | 0x1d_bd1b,
            ) => Some(28),
            Some(0x1d_bd1e | 0x1d_bd1f | 0x1d_bd22) => Some(29),
            Some(0x1d_bd25) => Some(30),
            _ => None,
        };
        if let Some(completed_stores) = tail_completed {
            let slot = self
                .current_slot
                .ok_or("Trinexx first-part position omitted its active slot")?;
            if event.x != Some(u16::from(slot))
                || event.return_address.map(|r| r & 0x00ff_ffff) != Some(0xdc_bc39)
                || event.stack4 != Some(0xb8)
            {
                return Err("Trinexx first-part position has the wrong source stack".into());
            }
            self.trinexx_front_part = Some((slot, completed_stores));
            return Ok(());
        }

        if event.pc == Some(0x1d_bc7c) {
            let slot = self
                .current_slot
                .ok_or("Trinexx neck loop omitted its active slot")?;
            if event.x != Some(u16::from(slot))
                || event.return_address.map(|r| r & 0xffff) != Some(0xb8dc)
            {
                return Err("Trinexx neck loop has the wrong active slot or caller".into());
            }
            // INC $0FB5, LDA $0FB5, CMP subtype2 and the loop branch have
            // completed. A retains the next segment at the pending JMP.
            let segment = event.a.ok_or("Trinexx neck loop omitted its counter")? as u8;
            if segment == 0 || segment >= 9 {
                return Err("Trinexx neck loop has an invalid counter".into());
            }
            self.trinexx_head_draw = Some((slot, segment));
            return Ok(());
        }
        // After PLX the two TrinexxHeadSin multiplications store only the
        // delta scratch ($0FA8/$0FA9) and hardware multiply registers, then
        // LDA $0FB5 / BNE select the first-part JSR or the segment draw at
        // $1D:BC3C. Nothing of segment `$0FB5` has been drawn yet.
        if event
            .pc
            .is_some_and(|pc| (0x1d_bbe2..=0x1d_bc3c).contains(&pc))
        {
            let slot = self
                .current_slot
                .ok_or("Trinexx neck delta calculation omitted its active slot")?;
            if event.x != Some(u16::from(slot))
                || event.return_address.map(|r| r & 0x00ff_ffff) != Some(0x1f_b8dc)
                || event.stack4 != Some(0xc2)
            {
                return Err("Trinexx neck delta calculation has the wrong source stack".into());
            }
            let segment = self
                .trinexx_segment_counter
                .ok_or("Trinexx neck delta calculation has no traced segment counter")?;
            if segment >= 9
                || (event.pc == Some(0x1d_bc37) && segment != 0)
                || (event.pc == Some(0x1d_bc3c) && segment == 0)
            {
                return Err("Trinexx neck delta calculation has an invalid segment".into());
            }
            self.trinexx_head_draw = Some((slot, segment));
            return Ok(());
        }
        // The straight-line neck angle calculation from the REP #$30 after
        // PHX through the PLX only loads sine-table words and stores
        // direct-page scratch ($08/$0A/$0C); every boundary in it leaves
        // the pending segment undrawn with the slot still pushed.
        if !matches!(
            event.pc,
            Some(
                0x1d_bbe1
                    | 0x1d_bbbf
                    | 0x1d_bbc1
                    | 0x1d_bbc4
                    | 0x1d_bbc5
                    | 0x1d_bbc6
                    | 0x1d_bbca
                    | 0x1d_bbcc
                    | 0x1d_bbce
                    | 0x1d_bbcf
                    | 0x1d_bbd2
                    | 0x1d_bbd4
                    | 0x1d_bbd7
                    | 0x1d_bbd8
                    | 0x1d_bbd9
                    | 0x1d_bbdd
                    | 0x1d_bbdf
            )
        ) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Trinexx neck calculation omitted its active slot")?;
        // PHX saved the active slot above TrinexxHead_Draw's local JSR
        // return. Y is the cached neck index while X holds a sine-table index.
        if event.return_address != Some(0xb8_dc00 | u32::from(slot)) {
            return Err("Trinexx neck calculation has the wrong saved slot or caller".into());
        }
        let cursor = event
            .y
            .ok_or("Trinexx neck calculation omitted its segment cursor")?;
        let segment = cursor
            .checked_sub(u16::from(slot) * 9)
            .filter(|&i| i < 9)
            .ok_or("Trinexx neck calculation has an invalid segment cursor")?
            as u8;
        self.trinexx_head_draw = Some((slot, segment));
        Ok(())
    }

    fn observe_trinexx_breath_tile_collision(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        if !matches!(event.event.as_str(), "nmi" | "frame") {
            return Ok(());
        }
        self.trinexx_breath_tile_collision = None;
        // Sprite_CC_CD_Common ($1D:BD44) ends with `JSR $8094` (a JSL to
        // Sprite_CheckTileCollision, $06:E496) followed by `BEQ`/`STZ
        // $0DD0,X`. Every boundary from the RTS closing
        // Sprite_CheckTileCollision2 through the pending BEQ has published
        // the wall-collision byte and still owes the state clear.
        let matched = match event.pc {
            Some(0x06_e5b7) => {
                event.return_address.map(|r| r & 0xffff) == Some(0xe49b)
                    && event.stack4 == Some(0x97)
            }
            Some(0x06_e49d | 0x06_e4a0) => {
                event.return_address.map(|r| r & 0x00ff_ffff) == Some(0x1d_8097)
            }
            Some(0x1d_8097) => event.return_address.map(|r| r & 0xffff) == Some(0xbd5e),
            Some(0x1d_bd5f) => true,
            _ => return Ok(()),
        };
        if !matched {
            // The collision helper and the bank-$1D `JSL` trampoline at
            // $1D:8094 serve every bank-$1D sprite; a foreign caller's stack
            // means this boundary is simply not the breath checkpoint (cold
            // route frame 378813 stopped there under another sprite).
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Trinexx breath tile-collision return omitted its active slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err("Trinexx breath tile-collision return has the wrong active slot".into());
        }
        self.trinexx_breath_tile_collision = Some(slot);
        Ok(())
    }

    fn observe_trinexx_final_phase_tile_collision(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        if event.event == "wram-write" {
            // `DEC $0D90,X` at $1D:AE74 is Sprite_Trinexx_FinalPhase state 0's
            // `--sprite_A`; the tile-collision call follows it.
            if event.pc == Some(0x1d_ae77)
                && event.address == Some(0x0d90 + event.x.unwrap_or(0xffff))
            {
                let slot = self
                    .current_slot
                    .ok_or("Trinexx final phase decremented A before a sprite slot")?;
                if event.x != Some(u16::from(slot)) {
                    return Err("Trinexx final phase A store has the wrong active slot".into());
                }
                self.trinexx_final_phase_case0 = Some(slot);
            }
            return Ok(());
        }
        if !matches!(event.event.as_str(), "nmi" | "frame") {
            return Ok(());
        }
        self.trinexx_final_phase_tile_collision = None;
        let Some(slot) = self.trinexx_final_phase_case0 else {
            return Ok(());
        };
        if self.current_slot != Some(slot) {
            return Ok(());
        }
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return Ok(());
        };
        let ret24 = event.return_address.map(|r| r & 0x00ff_ffff);
        let ret16 = event.return_address.map(|r| r & 0xffff);
        // Sprite_CheckTileCollision2 is JSR'd from the $06:E496 JSL body
        // under the $1D:8094 trampoline: [$E49B, DB, $97]. Its single-layer
        // helper JSRs the `$68` property probe at $06:E532 ([$E534, $E49B])
        // whose shared tail JSRs GetTileAttribute at $06:E79D ([$E7A0, $E534]).
        let probes_completed = match pc {
            0x06_e4ab => {
                if ret16 != Some(0xe49b) || event.stack4 != Some(0x97) {
                    return Err(
                        "Trinexx final-phase tile-collision entry has the wrong stack".into(),
                    );
                }
                false
            }
            0x06_e525 | 0x06_e528 | 0x06_e52a | 0x06_e52d | 0x06_e52f | 0x06_e530 | 0x06_e532 => {
                if ret16 != Some(0xe49b) || event.stack4 != Some(0x97) {
                    return Err("Trinexx final-phase property probe has the wrong stack".into());
                }
                true
            }
            0x06_e73c..=0x06_e882 => {
                if ret24 != Some(0x9b_e534) || event.stack4 != Some(0xe4) {
                    return Err(
                        "Trinexx final-phase property probe body has the wrong stack".into(),
                    );
                }
                true
            }
            0x06_e883..=0x06_e8fd => {
                if ret24 != Some(0x34_e7a0) || event.stack4 != Some(0xe5) {
                    return Err("Trinexx final-phase tile attribute has the wrong stack".into());
                }
                true
            }
            _ => return Ok(()),
        };
        if event.x != Some(u16::from(slot)) {
            return Err("Trinexx final-phase tile collision has the wrong active slot".into());
        }
        self.trinexx_final_phase_tile_collision = Some((slot, probes_completed));
        Ok(())
    }

    fn observe_trinexx_final_phase_draw(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if event.event == "wram-write" {
            if event.address == Some(0x0fb6) {
                self.trinexx_d_draw_counter = event.value;
                if event.pc == Some(0x1d_af94) && event.value == Some(0) {
                    self.trinexx_d_draw_active = self.current_slot;
                }
            }
            return Ok(());
        }
        if !matches!(event.event.as_str(), "nmi" | "frame") {
            return Ok(());
        }
        self.trinexx_final_phase_draw = None;
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return Ok(());
        };
        // SpriteDraw_SingleLarge ($06:DBF0 wrapper, $06:DC10 body) called
        // from the loop's $1D:B05C JSL for a body segment: its wrapper
        // pushes, the pending prep JSR, and Sprite_PrepOamCoordOrDoubleRet's
        // in-bounds body ($06:E41E..E485, only the pause clear and scratch)
        // all leave the segment's draw pending. The segment comes from the
        // traced `$0FB6` counter because the prep helper clobbers Y.
        if let Some(active) = self.trinexx_d_draw_active {
            if self.current_slot == Some(active) {
                let ret24 = event.return_address.map(|r| r & 0x00ff_ffff);
                let draw_pending = match pc {
                    0x06_dbf0 => ret24 == Some(0x1d_b05f) && event.stack4 == Some(0xd9),
                    0x06_dbf1 | 0x06_dbf3 => ret24 == Some(0xb0_5f1d) && event.stack4 == Some(0x1d),
                    0x06_dbf2 => ret24 == Some(0x5f_1d06) && event.stack4 == Some(0xb0),
                    0x06_dc10 | 0x06_dc13 | 0x06_dc15 => {
                        ret24 == Some(0x1d_dbf5) && event.stack4 == Some(0x5f)
                    }
                    0x06_e41e..=0x06_e485 => ret24 == Some(0xf5_dc12) && event.stack4 == Some(0xdb),
                    _ => false,
                };
                if draw_pending {
                    if event.x != Some(u16::from(active)) {
                        return Err("Trinexx segment draw prep has the wrong active slot".into());
                    }
                    let segment = self
                        .trinexx_d_draw_counter
                        .ok_or("Trinexx segment draw prep has no traced counter")?;
                    if segment >= 24 {
                        return Err("Trinexx segment draw prep has an invalid counter".into());
                    }
                    self.trinexx_final_phase_draw = Some((active, segment, 7));
                    return Ok(());
                }
            }
        }
        if !(0x1d_af94..=0x1d_b078).contains(&pc)
            && !(0x1d_b560..=0x1d_b586).contains(&pc)
            && !(0x1d_b079..=0x1d_b0c7).contains(&pc)
        {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Trinexx final-phase draw omitted its active slot")?;
        let ret24 = event.return_address.map(|r| r & 0x00ff_ffff);
        let ret16 = event.return_address.map(|r| r & 0xffff);
        let y = event
            .y
            .ok_or("Trinexx final-phase draw omitted its segment register")?;
        if (0x1d_b079..=0x1d_b0c7).contains(&pc) {
            // Sprite_Trinexx_CheckDamageToFlashingSegment, JSR'd at $1D:B043
            // for segment 4 with the segment PHY'd above the loop frame. The
            // four PHA'd coordinate bytes sit above its $B045 return until the
            // matching PLAs; Sprite_CheckDamageFromLink clobbers Y.
            let (stores, pushed) =
                match pc {
                    0x1d_b079 | 0x1d_b07c => (0u8, 0u8),
                    0x1d_b07d | 0x1d_b080 => (0, 1),
                    0x1d_b081 | 0x1d_b084 => (0, 2),
                    0x1d_b085 | 0x1d_b088 => (0, 3),
                    0x1d_b089 | 0x1d_b08c => (0, 4),
                    0x1d_b08f | 0x1d_b092 => (1, 4),
                    0x1d_b095 | 0x1d_b098 => (2, 4),
                    0x1d_b09b | 0x1d_b09e => (3, 4),
                    0x1d_b0a1 | 0x1d_b0a3 => (4, 4),
                    0x1d_b0a6 => (5, 4),
                    0x1d_b0a9 => (6, 4),
                    0x1d_b0ad | 0x1d_b0af => (7, 4),
                    0x1d_b0b2 | 0x1d_b0b4 => (8, 4),
                    0x1d_b0b7 | 0x1d_b0b8 => (9, 4),
                    0x1d_b0bb | 0x1d_b0bc => (10, 2),
                    0x1d_b0bf | 0x1d_b0c0 => (11, 1),
                    0x1d_b0c3 | 0x1d_b0c4 => (12, 0),
                    0x1d_b0c7 => (13, 0),
                    _ => return Err(
                        "Trinexx flashing-segment check stopped inside Sprite_CheckDamageFromLink"
                            .into(),
                    ),
                };
            if pc == 0x1d_b0b7 || pc == 0x1d_b0b8 {
                // One PLA has run: [y_low, x_high, x_low, $45, $B0].
                if event.stack4 != Some(0x45) {
                    return Err("Trinexx flashing-segment restore has the wrong stack".into());
                }
            } else {
                let stack_ok = match pushed {
                    0 => ret16 == Some(0xb045),
                    1 => event.return_address.map(|r| r & 0x00ff_ff00) == Some(0xb0_4500),
                    2 => {
                        event.return_address.map(|r| (r >> 16) & 0xff) == Some(0x45)
                            && event.stack4 == Some(0xb0)
                    }
                    3 => event.stack4 == Some(0x45),
                    _ => true,
                };
                if !stack_ok {
                    return Err("Trinexx flashing-segment check has the wrong source stack".into());
                }
            }
            if stores < 7 && y != 4 {
                return Err("Trinexx flashing-segment check is not on segment 4".into());
            }
            if event.x != Some(u16::from(slot)) {
                return Err("Trinexx flashing-segment check has the wrong active slot".into());
            }
            self.trinexx_final_phase_draw = Some((slot, 4, 16 + stores));
            return Ok(());
        }
        // Sprite_TrinexxD_Draw is JSR'd from Sprite_Trinexx_FinalPhase
        // ($1D:ADD9 under SpriteActive_Main's $1F:xxxx return). PHX (the
        // slot) covers the history lookup, PHY (the segment) the Link damage
        // check through the OAM pointer advance and the flashing-segment
        // call; the head draw helper adds its own $1D:B069 return.
        #[derive(PartialEq)]
        enum Stack {
            Plain,
            PushedSlot,
            PushedSegment,
            HeadHelper,
        }
        let (stack, segment, stage) = match pc {
            0x1d_af94 | 0x1d_af97 => {
                let counter = self
                    .trinexx_d_draw_counter
                    .ok_or("Trinexx final-phase draw loop head has no traced counter")?;
                (Stack::Plain, u16::from(counter), 0u8)
            }
            0x1d_af98..=0x1d_af9e | 0x1d_afc6 => (Stack::Plain, y, 0),
            0x1d_af9f..=0x1d_afc5 => (Stack::PushedSlot, y, 0),
            0x1d_afc7..=0x1d_afc9 => (Stack::PushedSegment, y, 0),
            0x1d_afca..=0x1d_afff => (Stack::PushedSegment, y / 2, 0),
            0x1d_b000..=0x1d_b01a => {
                return Err(
                    "Trinexx final-phase draw stopped inside the Link damage stores".into(),
                );
            }
            0x1d_b01b..=0x1d_b024 => (Stack::PushedSegment, y / 2, 1),
            0x1d_b025..=0x1d_b02e => (Stack::PushedSegment, y / 2, 2),
            0x1d_b02f..=0x1d_b031 => (Stack::PushedSegment, y / 2, 3),
            0x1d_b032..=0x1d_b036 => (Stack::Plain, y, 3),
            0x1d_b037..=0x1d_b042 => (Stack::Plain, y, 4),
            0x1d_b043 | 0x1d_b046..=0x1d_b04e => (Stack::PushedSegment, y, 4),
            0x1d_b051 => (Stack::PushedSegment, y, 5),
            0x1d_b052..=0x1d_b055 => (Stack::Plain, y, 5),
            0x1d_b058 | 0x1d_b05a => (Stack::Plain, y, 6),
            0x1d_b05c => (Stack::Plain, y, 7),
            0x1d_b060 => (Stack::Plain, y + 1, 0),
            0x1d_b062 | 0x1d_b064 => (Stack::Plain, y, 6),
            0x1d_b067 => (Stack::Plain, y, 7),
            0x1d_b560..=0x1d_b583 => (Stack::HeadHelper, y, 7),
            0x1d_b586 => (Stack::HeadHelper, y + 1, 0),
            0x1d_b06a | 0x1d_b06d | 0x1d_b070 | 0x1d_b073 | 0x1d_b075 | 0x1d_b078 => {
                (Stack::Plain, y + 1, 0)
            }
            _ => return Ok(()),
        };
        if (0x1d_afca..=0x1d_b031).contains(&pc) && y & 1 != 0 {
            return Err("Trinexx final-phase draw segment register is not doubled".into());
        }
        let segment =
            u8::try_from(segment).map_err(|_| "Trinexx final-phase draw segment overflowed")?;
        if segment >= 24 {
            return Err("Trinexx final-phase draw has an invalid segment".into());
        }
        let stack_ok = match stack {
            Stack::Plain => ret24 == Some(0x1f_add9) && event.stack4 == Some(0xc2),
            Stack::PushedSlot => {
                ret24 == Some(0xad_d900 | u32::from(slot)) && event.stack4 == Some(0x1f)
            }
            Stack::PushedSegment => {
                let pushed = if (0x1d_afca..=0x1d_b031).contains(&pc) {
                    u32::from(segment)
                } else {
                    u32::from(y)
                };
                ret24 == Some(0xad_d900 | pushed) && event.stack4 == Some(0x1f)
            }
            Stack::HeadHelper => ret24 == Some(0xd9_b069) && event.stack4 == Some(0xad),
        };
        if !stack_ok {
            return Err("Trinexx final-phase draw has the wrong source stack".into());
        }
        if stack != Stack::PushedSlot && event.x != Some(u16::from(slot)) {
            return Err("Trinexx final-phase draw has the wrong active slot".into());
        }
        self.trinexx_final_phase_draw = Some((slot, segment, stage));
        Ok(())
    }

    fn observe_helmasaur_hard_hat_tile_collision(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        if !matches!(event.event.as_str(), "nmi" | "frame") {
            return Ok(());
        }
        self.helmasaur_hard_hat_tile_collision = None;
        let Some(slot) = self.helmasaur_hard_hat_beetle_subtype2_increment_slot else {
            return Ok(());
        };
        if self.current_slot != Some(slot) {
            return Ok(());
        }
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return Ok(());
        };
        if !(0x06_e4a8..=0x06_e8fd).contains(&pc) {
            return Ok(());
        }
        let ret24 = event.return_address.map(|r| r & 0x00ff_ffff);
        // The tile-property tail PHXs around its table lookups ($06:E7A8,
        // $06:E812), so a boundary there sees one pushed byte above the
        // expected return bytes.
        let shape = |ret: u32, stack4: u8| {
            (ret24 == Some(ret) && event.stack4 == Some(stack4))
                || (ret24.map(|r| r >> 8) == Some(ret & 0xffff)
                    && event.stack4 == Some((ret >> 16) as u8))
        };
        // HelmasaurHardHatBeetleCommon JSRs Sprite_CheckTileCollision2 directly
        // from $06:A48E under Sprite_ExecuteSingle's return: [$A491, $83A6].
        // The single-layer helper JSRs the vertical probe at $06:E501 ([$E503])
        // and the horizontal probe at $06:E50E ([$E510]); each wrapper JSRs
        // Sprite_CheckTileInDirection ([$E5F0] / [$E5BA]) whose shared tail
        // JSRs GetTileAttribute at $06:E79D ([$E7A0]); the `$68` property
        // probe is JSR'd at $06:E532 ([$E534]).
        use SpriteTileCollisionStage as S;
        let base = shape(0xa6_a491, 0x83);
        let stage = match pc {
            0x06_e4ab if base => S::Entered,
            0x06_e4a8 | 0x06_e4ae..=0x06_e4b6 | 0x06_e4db..=0x06_e501 if base => S::Cleared,
            0x06_e5ee..=0x06_e5f9 if shape(0x91_e503, 0xa4) => S::Cleared,
            0x06_e72f..=0x06_e882 if shape(0x03_e5f0, 0xe5) => S::Cleared,
            0x06_e883..=0x06_e8fd if shape(0xf0_e7a0, 0xe5) => S::Cleared,
            0x06_e504..=0x06_e50e if base => S::VerticalProbeDone,
            0x06_e5b8..=0x06_e5c3 if shape(0x91_e510, 0xa4) => S::VerticalProbeDone,
            0x06_e72f..=0x06_e882 if shape(0x10_e5ba, 0xe5) => S::VerticalProbeDone,
            0x06_e883..=0x06_e8fd if shape(0xba_e7a0, 0xe5) => S::VerticalProbeDone,
            0x06_e511 | 0x06_e525..=0x06_e532 if base => S::ProbesCompleted,
            0x06_e73c..=0x06_e882 if shape(0x91_e534, 0xa4) => S::ProbesCompleted,
            0x06_e883..=0x06_e8fd if shape(0x34_e7a0, 0xe5) => S::ProbesCompleted,
            _ => {
                return Err(format!(
                    "Helmasaur/Hardhat tile collision stopped at an unmodeled boundary ${pc:06x} ret={ret24:?} stack4={:?}",
                    event.stack4
                ))
            }
        };
        if event.x != Some(u16::from(slot)) {
            return Err("Helmasaur/Hardhat tile collision has the wrong active slot".into());
        }
        self.helmasaur_hard_hat_tile_collision = Some((slot, stage));
        Ok(())
    }

    fn observe_sprite_handler_returned(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if !matches!(event.event.as_str(), "nmi" | "frame") {
            return Ok(());
        }
        self.handler_returned_slot = None;
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return Ok(());
        };
        // SpriteActive_Main dispatches bank-$1D handlers through the $06:BFF4
        // trampoline (JSL $1D:C21A: PHB/PHK/PLB, JSR C222, PLB, RTL). At the
        // wrapper's PLB/RTL the handler has returned; the trampoline's RTS
        // then returns to Sprite_ExecuteSingle.
        let returned = match pc {
            0x1d_c220 | 0x1d_c221 => {
                event.return_address.map(|r| r & 0x00ff_ffff) == Some(0x06_bff7)
                    && event.stack4 == Some(0xa6)
            }
            0x06_bff7 => event.return_address.map(|r| r & 0xffff) == Some(0x84a6),
            _ => false,
        };
        if !returned {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("sprite handler returned outside an active slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err("sprite handler return has the wrong active slot".into());
        }
        self.handler_returned_slot = Some(slot);
        Ok(())
    }

    fn observe_laser_eye_draw_prologue(&mut self, event: &RawTraceEvent) {
        if !matches!(event.event.as_str(), "nmi" | "frame") {
            return;
        }
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return;
        };
        if self.laser_eye_draw_prologue.is_some() && !laser_eye_draw_in_flight_pc(pc) {
            self.laser_eye_draw_prologue = None;
        }
    }

    fn observe_lanmola_draw_prefix(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if !matches!(event.event.as_str(), "nmi" | "frame") {
            return Ok(());
        }
        self.lanmola_draw_prefix = None;
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return Ok(());
        };
        if !(0x05_a64a..=0x05_a6a5).contains(&pc) {
            return Ok(());
        }
        let slot = self
            .current_slot
            .ok_or("Lanmola draw prologue omitted its active slot")?;
        let ret24 = event.return_address.map(|r| r & 0x00ff_ffff);
        let ret16 = event.return_address.map(|r| r & 0xffff);
        // Lanmola_Draw is JSR'd from Sprite_54_Lanmolas ($05:A3A5, return
        // $A3A8). Its prologue pushes X (the slot) and the x, y, z and
        // graphics bytes, then pops them into the trail arrays; the stack
        // shape names how many bytes sit above the return.
        let slot_pushed = ret24 == Some(0xa3_a800 | u32::from(slot));
        let plain = ret16 == Some(0xa3a8);
        let (completed_stores, stack_ok, x_is_slot) = match pc {
            0x05_a64a..=0x05_a66d => (0u8, plain, true),
            0x05_a670 | 0x05_a673 | 0x05_a675 => (1, plain, true),
            0x05_a676..=0x05_a679 => (1, slot_pushed, true),
            0x05_a67a..=0x05_a67d => (
                1,
                ret24.map(|r| r >> 8) == Some(0xa800 | u32::from(slot))
                    && event.stack4 == Some(0xa3),
                true,
            ),
            0x05_a67e..=0x05_a681 => (
                1,
                ret24.map(|r| r >> 16) == Some(u32::from(slot)) && event.stack4 == Some(0xa8),
                true,
            ),
            0x05_a682..=0x05_a685 => (1, event.stack4 == Some(slot), true),
            0x05_a686..=0x05_a68f => (1, true, true),
            0x05_a690..=0x05_a692 => (1, event.stack4 == Some(slot), false),
            0x05_a696 => (2, event.stack4 == Some(slot), false),
            0x05_a697 => (2, event.stack4 == Some(0xa8), false),
            0x05_a69b => (3, event.stack4 == Some(0xa8), false),
            0x05_a69c => (3, event.stack4 == Some(0xa3), false),
            0x05_a6a0 => (4, event.stack4 == Some(0xa3), false),
            0x05_a6a1 => (4, slot_pushed, false),
            0x05_a6a5 => (5, slot_pushed, false),
            _ => {
                return Err(format!(
                    "Lanmola draw prologue stopped at an unmodeled boundary ${pc:06x}"
                ))
            }
        };
        if !stack_ok {
            return Err("Lanmola draw prologue has the wrong source stack".into());
        }
        if x_is_slot && event.x != Some(u16::from(slot)) {
            return Err("Lanmola draw prologue has the wrong active slot".into());
        }
        self.lanmola_draw_prefix = Some((slot, completed_stores));
        Ok(())
    }

    fn observe_sidenexx_neck_target(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if !matches!(event.event.as_str(), "nmi" | "frame") {
            return Ok(());
        }
        self.sidenexx_neck_target = None;
        // Sprite_Sidenexx state 2 ($1D:B9F2): PHX saves the slot, X walks
        // the nine cached segments and each iteration runs two type passes
        // and one radius pass of compare / INC-or-DEC / INC $01. `counted`
        // names how many of the current segment's six steps have run;
        // `x_is_next` marks the INY/DEC/BPL tail where X already advanced.
        let (x_is_next, counted) = match event.pc {
            Some(0x1d_ba07 | 0x1d_ba0a | 0x1d_ba0d | 0x1d_ba0f | 0x1d_ba11 | 0x1d_ba18) => {
                (false, 0u8)
            }
            Some(0x1d_ba14 | 0x1d_ba1b) => (false, 1),
            Some(
                0x1d_ba16 | 0x1d_ba1d | 0x1d_ba20 | 0x1d_ba23 | 0x1d_ba25 | 0x1d_ba27 | 0x1d_ba2e,
            ) => (false, 2),
            Some(0x1d_ba2a | 0x1d_ba31) => (false, 3),
            Some(
                0x1d_ba2c | 0x1d_ba33 | 0x1d_ba35 | 0x1d_ba37 | 0x1d_ba39 | 0x1d_ba3c | 0x1d_ba3f
                | 0x1d_ba41 | 0x1d_ba43 | 0x1d_ba4a,
            ) => (false, 4),
            Some(0x1d_ba46 | 0x1d_ba4d) => (false, 5),
            Some(0x1d_ba48 | 0x1d_ba4f) => (false, 6),
            Some(0x1d_ba50 | 0x1d_ba51 | 0x1d_ba53 | 0x1d_ba55) => (true, 0),
            Some(0x1d_ba56 | 0x1d_ba58 | 0x1d_ba5a | 0x1d_ba5d) => {
                // PLX restored the slot; the change-count test, the idle
                // state store or its random delay are pending.
                let slot = self
                    .current_slot
                    .ok_or("Sidenexx neck-target completion omitted its active slot")?;
                if event.x != Some(u16::from(slot))
                    || event.return_address.map(|r| r & 0x00ff_ffff) != Some(0x06_c21f)
                {
                    return Err("Sidenexx neck-target completion has the wrong source stack".into());
                }
                let step = if event.pc == Some(0x1d_ba5d) { 55 } else { 54 };
                self.sidenexx_neck_target = Some((slot, step));
                return Ok(());
            }
            _ => return Ok(()),
        };
        let slot = self
            .current_slot
            .ok_or("Sidenexx neck-target loop omitted its active slot")?;
        if event.return_address.map(|r| r & 0x00ff_ffff) != Some(0xc2_1f00 | u32::from(slot))
            || event.stack4 != Some(0x06)
        {
            return Err("Sidenexx neck-target loop has the wrong source stack".into());
        }
        let x = event
            .x
            .ok_or("Sidenexx neck-target loop omitted its segment cursor")?;
        let segment = x
            .checked_sub(u16::from(slot) * 9)
            .ok_or("Sidenexx neck-target loop cursor precedes its head")?;
        let step = if x_is_next {
            if !(1..=9).contains(&segment) {
                return Err("Sidenexx neck-target loop tail has an invalid cursor".into());
            }
            segment as u8 * 6
        } else {
            if segment >= 9 {
                return Err("Sidenexx neck-target loop has an invalid cursor".into());
            }
            segment as u8 * 6 + counted
        };
        self.sidenexx_neck_target = Some((slot, step));
        Ok(())
    }

    fn observe_buzzblob_movement(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        // A WRAM event names the instruction performing the store; an
        // interrupt endpoint names the instruction still pending. Only the
        // latter proves the subpixel-only prefix at this shared axis helper.
        if event.event == "wram-write" {
            return Ok(());
        }
        if event.pc == Some(0x06_d8e2) {
            let slot = self
                .current_slot
                .ok_or("Buzzblob movement has no sprite slot")?;
            if event.x != Some(u16::from(slot)) {
                return Err("Buzzblob movement call disagreed with the active slot".into());
            }
            self.buzzblob_movement = Some((slot, false));
        } else if event.pc == Some(0x06_d8e5) {
            self.buzzblob_movement = None;
        } else if event.pc == Some(0x06_e94e) {
            if let Some((slot, published)) = self.buzzblob_movement.as_mut() {
                if event.x != Some(u16::from(*slot) + 16) {
                    return Err("Buzzblob X-subpixel publication has the wrong axis cursor".into());
                }
                *published = true;
            }
        }
        Ok(())
    }

    fn observe_guard_animation_checkpoint(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        if event.pc == Some(0x05_cbe0) {
            self.hog_spear_active_body = true;
        }
        if event.pc == Some(0x05_cab0)
            && event.x == Some(0)
            && self.hog_spear_active_body
            && self.timers_and_oam_dispatch_state == Some(8)
        {
            let slot = self
                .current_slot
                .ok_or("Hog Spear initializer has no slot")?;
            if !(1..=2).contains(&self.initialize_active_main_calls)
                || event.sub != Some(0)
                || event.return_address != Some(0xc68800 | u32::from(slot))
            {
                return Err("Hog Spear body return lacked its initializer caller proof".into());
            }
            self.guard_animation_checkpoint = Some((
                slot,
                zelda3::GuardAnimationCheckpoint::HogSpearInitializerBodyReturned {
                    active_call: self.initialize_active_main_calls,
                },
            ));
            return Ok(());
        }
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
        self.initialize_prep_move_y = None;
        self.initialize_reset_properties = Some((slot, phase, completed_stores));
        Ok(())
    }

    /// `SpritePrep_Spike` ($06:91D7) / `SpritePrep_RockStal` ($06:91DC) set
    /// their velocities, `JSR Sprite_MoveY` from $06:91E1 (pushed return
    /// $91E3), then clear y_vel at $06:91E4. The interrupted PC alone names
    /// which `Sprite_MoveY` assignments completed.
    fn observe_initialize_prep_move_y(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        // The trailing `frame` event of an interrupted host reports the NMI
        // handler's PC, so only the `nmi` event carries the boundary.
        if event.event != "nmi" {
            return Ok(());
        }
        self.initialize_prep_move_y = None;
        if self.timers_and_oam_dispatch_state != Some(8) {
            return Ok(());
        }
        let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) else {
            return Ok(());
        };
        let ret16 = event.return_address.map(|r| r & 0xffff);
        let ret16_over_php = event.return_address.map(|r| (r >> 8) & 0xffff);
        let checkpoint = match pc {
            0x06_e93e..=0x06_e94b if ret16 == Some(0x91e3) => {
                SpriteMoveXYCheckpoint::BeforeMovement
            }
            0x06_e94e..=0x06_e951 if ret16 == Some(0x91e3) => {
                SpriteMoveXYCheckpoint::AfterYSubpixel
            }
            // PHP at $06:E951 holds P above the return until PLP at $06:E958.
            0x06_e952..=0x06_e958 if ret16_over_php == Some(0x91e3) => {
                SpriteMoveXYCheckpoint::AfterYSubpixel
            }
            0x06_e959..=0x06_e961 if ret16 == Some(0x91e3) => {
                SpriteMoveXYCheckpoint::AfterYSubpixel
            }
            0x06_e964..=0x06_e968 if ret16 == Some(0x91e3) => SpriteMoveXYCheckpoint::AfterYLow,
            0x06_e96b if ret16 == Some(0x91e3) => SpriteMoveXYCheckpoint::AfterYHigh,
            // RTS done; STZ y_vel pending under Sprite_ExecuteSingle's frame.
            0x06_91e4 if event.return_address == Some(0x00_83a6) => {
                SpriteMoveXYCheckpoint::AfterYHigh
            }
            _ => return Ok(()),
        };
        let slot = self
            .current_slot
            .ok_or("state-8 prep Sprite_MoveY boundary has no active slot")?;
        if event.x != Some(u16::from(slot)) {
            return Err("state-8 prep Sprite_MoveY boundary disagrees with its active slot".into());
        }
        self.initialize_prep_pending = None;
        self.initialize_prep_move_y = None;
        self.initialize_reset_properties = None;
        self.initialize_load_properties = None;
        self.initialize_prep_move_y = Some((slot, checkpoint));
        Ok(())
    }

    fn observe_initialize_prep_pending(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        let pc = event.pc.map(|pc| pc & 0xff_ffff);
        // FireBar's private prep entry is still before its first INC store;
        // the initializer has already published properties and state 9.
        let initializer_dispatch = matches!(pc, Some(0x06_8654 | 0x06_8657 | 0x06_91b4))
            // The dispatched prep routine's first instruction, reached by
            // JumpTableLocal's JML with Sprite_ExecuteSingle's frame on top.
            || (pc.is_some_and(|pc| {
                pc >> 16 == 0x06 && SPRITE_PREP_ENTRY_PCS.contains(&((pc & 0xffff) as u16))
            }) && event.return_address == Some(0x00_83a6))
            || (pc == Some(0x00_8781)
                && event.return_address.map(|pc| pc & 0xff_ffff) == Some(0x06_865a))
            // The prep entry was a `JSL` into a `_bounce` trampoline whose
            // PHB/PHK/PLB prologue has not reached the prep body.
            || pc.is_some_and(|pc| {
                sprite_prep_bounce_pending(pc, event.return_address, event.stack4)
            })
            // JumpTableLocal has popped its inline-table return and loaded
            // the prep target. The remaining stack belongs to the state-8
            // dispatch in Sprite_ExecuteSingle; LDY $03 and JML [$00] remain.
            // Sprite_ExecuteSingle's own state dispatch reaches the same
            // instruction with the same stack: its loaded target is
            // SpriteModule_Initialize ($06:864D) and nothing of the
            // initializer has run, so that boundary stays after timers/OAM.
            || (pc == Some(0x00_8797)
                && event.return_address == Some(0x00_83a6)
                && event.a != Some(0x864d)
                && event.y.is_some_and(|index| index & 1 == 1 && index <= 0x1e5));
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
        self.initialize_prep_move_y = None;
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
        self.initialize_prep_move_y = None;
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
        self.initialize_prep_move_y = None;
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

    fn observe_trinexx_death_spawn_write(&mut self, event: &RawTraceEvent) -> Result<(), String> {
        let pc = event.pc.ok_or("Snes9x WRAM write omitted PC")? & 0x00ff_ffff;
        let address = event.address.ok_or("Snes9x WRAM write omitted address")?;
        // Sprite_MakeBossDeathExplosion_NoSound ($1D:DC30) calls the spawn
        // helper with type 0; the JSR return low byte $AA under its JSL
        // return names Sprite_CB_TrinexxRockHead's death branch caller.
        if pc == SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC
            && event.return_address.map(|r| r & 0x00ff_ffff) == Some(0x1d_dc35)
            && event.stack4 == Some(0xaa)
        {
            let slot = self
                .current_slot
                .ok_or("Snes9x spawned a Trinexx death explosion before a sprite slot")?;
            let spawned_slot = u8::try_from(
                event
                    .y
                    .ok_or("Snes9x Trinexx explosion spawn type write omitted slot Y")?,
            )
            .map_err(|_| "Snes9x Trinexx explosion spawned slot exceeded one byte")?;
            if event.x != Some(u16::from(slot))
                || spawned_slot >= 16
                || address != SPRITE_TYPE_BASE + u16::from(spawned_slot)
                || event.value != Some(0)
            {
                return Err(format!(
                    "Snes9x Trinexx explosion spawn type publication disagreed: parent={slot}, x={:?}, spawned={spawned_slot}, address=${address:04x}, value={:?}",
                    event.x, event.value,
                ));
            }
            self.trinexx_death_spawn = Some((
                slot,
                spawned_slot,
                SpriteDynamicSpawnProgress::TypePublished,
            ));
            return Ok(());
        }
        observe_dynamic_spawn_progress_write(
            &mut self.trinexx_death_spawn,
            event,
            "Trinexx explosion",
        )
    }

    fn observe_agahnim_motion_blur_spawn_write(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        let pc = event.pc.ok_or("Snes9x WRAM write omitted PC")? & 0x00ff_ffff;
        let address = event.address.ok_or("Snes9x WRAM write omitted address")?;
        // Sprite_Agahnim_ApplyMotionBlur ($1D:D392) JSLs the spawn helper
        // from $1D:D39B with type $C1; the JSL return names the caller.
        if pc == SPRITE_SPAWN_DYNAMICALLY_FIRST_TYPE_STORE_PC
            && event.return_address.map(|r| r & 0x00ff_ffff) == Some(0x1d_d39f)
        {
            let slot = self
                .current_slot
                .ok_or("Snes9x spawned an Agahnim afterimage before a sprite slot")?;
            let spawned_slot = u8::try_from(
                event
                    .y
                    .ok_or("Snes9x Agahnim afterimage type write omitted slot Y")?,
            )
            .map_err(|_| "Snes9x Agahnim afterimage slot exceeded one byte")?;
            if event.x != Some(u16::from(slot))
                || spawned_slot >= 16
                || address != SPRITE_TYPE_BASE + u16::from(spawned_slot)
                || event.value != Some(0xc1)
            {
                return Err(format!(
                    "Snes9x Agahnim afterimage type publication disagreed: parent={slot}, x={:?}, spawned={spawned_slot}, address=${address:04x}, value={:?}",
                    event.x, event.value,
                ));
            }
            self.agahnim_motion_blur_spawn = Some((
                slot,
                spawned_slot,
                SpriteDynamicSpawnProgress::TypePublished,
            ));
            return Ok(());
        }
        observe_dynamic_spawn_progress_write(
            &mut self.agahnim_motion_blur_spawn,
            event,
            "Agahnim afterimage",
        )
    }

    fn observe_agahnim_motion_blur_spawn_boundary(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        observe_dynamic_spawn_progress_boundary(&mut self.agahnim_motion_blur_spawn, event)
    }

    fn observe_trinexx_death_spawn_boundary(
        &mut self,
        event: &RawTraceEvent,
    ) -> Result<(), String> {
        observe_dynamic_spawn_progress_boundary(&mut self.trinexx_death_spawn, event)
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
        if let Some((slot, spawned_slot, progress)) = self.trinexx_death_spawn {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Trinexx explosion dynamic spawn outlived its active sprite slot",
            );
            return SpriteMainProgress::TrinexxDeathExplosionSpawn {
                slot,
                spawned_slot,
                progress,
            };
        }
        if let Some((slot, spawned_slot, progress)) = self.agahnim_motion_blur_spawn {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Agahnim afterimage spawn outlived its active sprite slot",
            );
            return SpriteMainProgress::AgahnimMotionBlurSpawn {
                slot,
                spawned_slot,
                progress,
            };
        }
        if let Some((slot, checkpoint)) = self.initialize_prep_move_y {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::InitializePrepMoveY { slot, checkpoint };
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
        if let Some((slot, completed_stores)) = self.lanmola_draw_prefix {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::LanmolaDrawPrefix {
                slot,
                completed_stores,
            };
        }
        if let Some(slot) = self.lanmola_subtype2_increment_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Lanmola subtype2 publication outlived its active sprite slot",
            );
            return SpriteMainProgress::AfterLanmolaSubtype2Increment(slot);
        }
        if let Some((slot, stage)) = self.helmasaur_hard_hat_tile_collision {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::HelmasaurHardHatTileCollision { slot, stage };
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
        if let Some(slot) = self.happiness_pond_rupee_graphics_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Happiness Pond rupee graphics publication outlived its active sprite slot",
            );
            assert_eq!(
                self.cucco_animation_slot, None,
                "one sprite slot published two incompatible partial checkpoints",
            );
            assert_eq!(
                self.cucco_subtype_increments, None,
                "one sprite slot published two incompatible partial checkpoints",
            );
            return SpriteMainProgress::HappinessPondRupeeGraphicsStarted(slot);
        }
        if let Some(slot) = self.catfish_medallion_graphics_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Catfish medallion rupee graphics publication outlived its active sprite slot",
            );
            assert_eq!(
                self.cucco_animation_slot, None,
                "one sprite slot published two incompatible partial checkpoints",
            );
            assert_eq!(
                self.cucco_subtype_increments, None,
                "one sprite slot published two incompatible partial checkpoints",
            );
            return SpriteMainProgress::CatfishMedallionGraphicsStarted(slot);
        }
        if let Some(slot) = self.waterfall_gt_cutscene_graphics_slot {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "GT cutscene graphics publication outlived its active sprite slot",
            );
            return SpriteMainProgress::WaterfallGtCutsceneGraphicsStarted(slot);
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
        if let Some(slot) = self.mini_moldorm_ai_pending {
            return SpriteMainProgress::MiniMoldormAiPending(slot);
        }
        if let Some(slot) = self.vitreous_player_damage_pending {
            return SpriteMainProgress::VitreousPlayerDamagePending(slot);
        }
        if let Some(slot) = self.moblin_attribute_loaded {
            return SpriteMainProgress::MoblinAttributeLoaded(slot);
        }
        if let Some(slot) = self.moblin_collision_geometry {
            return SpriteMainProgress::MoblinCollisionGeometry(slot);
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
        if let Some(slot) = self.trinexx_breath_tile_collision {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::TrinexxBreathTileCollisionReturned(slot);
        }
        if let Some((slot, step)) = self.sidenexx_neck_target {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::SidenexxNeckTargetLoop { slot, step };
        }
        if let Some((slot, probes_completed)) = self.trinexx_final_phase_tile_collision {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::TrinexxFinalPhaseTileCollision {
                slot,
                probes_completed,
            };
        }
        if let Some((slot, segment, stage)) = self.trinexx_final_phase_draw {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::TrinexxFinalPhaseDraw {
                slot,
                segment,
                stage,
            };
        }
        if let Some(slot) = self.handler_returned_slot {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::AfterSlot(slot);
        }
        if let Some(slot) = self.trinexx_head_draw_setup {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::TrinexxHeadDrawSetup(slot);
        }
        if let Some((slot, completed_stores)) = self.trinexx_front_part {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::TrinexxHeadFrontPart {
                slot,
                completed_stores,
            };
        }
        if let Some((slot, segment)) = self.trinexx_head_draw {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::TrinexxHeadDraw { slot, segment };
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
        if let Some(slot) = self.laser_eye_draw_prologue {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "laser eye draw prologue outlived its active sprite slot",
            );
            return SpriteMainProgress::LaserEyeDrawPrologue(slot);
        }
        if let Some((slot, checkpoint_ordinal, moved)) = self.zora_fireball {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Zora fireball movement outlived its active sprite slot",
            );
            let checkpoint = match (checkpoint_ordinal, moved) {
                (_, true) | (6, false) => SpriteMoveXYCheckpoint::AfterYHigh,
                (0, false) => SpriteMoveXYCheckpoint::BeforeMovement,
                (1, false) => SpriteMoveXYCheckpoint::AfterXSubpixel,
                (2, false) => SpriteMoveXYCheckpoint::AfterXLow,
                (3, false) => SpriteMoveXYCheckpoint::AfterXHigh,
                (4, false) => SpriteMoveXYCheckpoint::AfterYSubpixel,
                (5, false) => SpriteMoveXYCheckpoint::AfterYLow,
                (count, _) => panic!("invalid Zora fireball movement store count {count}"),
            };
            return SpriteMainProgress::ZoraFireballMovement { slot, checkpoint };
        }
        if let Some((slot, checkpoint_ordinal, z_done)) = self.boulder_movement {
            assert_eq!(
                self.current_slot,
                Some(slot),
                "Boulder movement outlived its active sprite slot",
            );
            assert!(
                z_done,
                "Boulder movement boundary inside Sprite_MoveZ is not modeled",
            );
            let checkpoint = match checkpoint_ordinal {
                0 => SpriteMoveXYCheckpoint::BeforeMovement,
                1 => SpriteMoveXYCheckpoint::AfterXSubpixel,
                2 => SpriteMoveXYCheckpoint::AfterXLow,
                3 => SpriteMoveXYCheckpoint::AfterXHigh,
                4 => SpriteMoveXYCheckpoint::AfterYSubpixel,
                5 => SpriteMoveXYCheckpoint::AfterYLow,
                6 => SpriteMoveXYCheckpoint::AfterYHigh,
                count => panic!("invalid Boulder movement store count {count}"),
            };
            return SpriteMainProgress::BoulderMovement { slot, checkpoint };
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
        if let Some((slot, true)) = self.buzzblob_movement {
            assert_eq!(self.current_slot, Some(slot));
            return SpriteMainProgress::BuzzblobAfterXSubpixel(slot);
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
            SpriteMainProgress::BoulderMovement { slot, checkpoint } => {
                MainLoopInterruption::SpriteMainBoulderMovement { slot, checkpoint }
            }
            SpriteMainProgress::InitializePrepMoveY { slot, checkpoint } => {
                MainLoopInterruption::SpriteMainInitializePrepMoveY { slot, checkpoint }
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
            SpriteMainProgress::HappinessPondRupeeGraphicsStarted(slot) => {
                MainLoopInterruption::SpriteMainHappinessPondRupeeGraphicsStarted(slot)
            }
            SpriteMainProgress::CatfishMedallionGraphicsStarted(slot) => {
                MainLoopInterruption::SpriteMainCatfishMedallionGraphicsStarted(slot)
            }
            SpriteMainProgress::WaterfallGtCutsceneGraphicsStarted(slot) => {
                MainLoopInterruption::SpriteMainWaterfallGtCutsceneGraphicsStarted(slot)
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
            SpriteMainProgress::TrinexxDeathExplosionSpawn {
                slot,
                spawned_slot,
                progress,
            } => MainLoopInterruption::SpriteMainTrinexxDeathExplosionSpawn {
                slot,
                spawned_slot,
                progress,
            },
            SpriteMainProgress::AgahnimMotionBlurSpawn {
                slot,
                spawned_slot,
                progress,
            } => MainLoopInterruption::SpriteMainAgahnimMotionBlurSpawn {
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
            SpriteMainProgress::LanmolaDrawPrefix {
                slot,
                completed_stores,
            } => MainLoopInterruption::SpriteMainLanmolaDrawPrefix {
                slot,
                completed_stores,
            },
            SpriteMainProgress::InitializePrepPending(slot) => {
                MainLoopInterruption::SpriteMainInitializePrepPending(slot)
            }
            SpriteMainProgress::HogSpearBodyGraphicsPending(slot) => {
                MainLoopInterruption::SpriteMainHogSpearBodyGraphicsPending(slot)
            }
            SpriteMainProgress::BuzzblobAfterXSubpixel(slot) => {
                MainLoopInterruption::SpriteMainBuzzblobAfterXSubpixel(slot)
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
            SpriteMainProgress::MoblinAttributeLoaded(slot) => {
                MainLoopInterruption::SpriteMainMoblinAttributeLoaded(slot)
            }
            SpriteMainProgress::MoblinCollisionGeometry(slot) => {
                MainLoopInterruption::SpriteMainMoblinCollisionGeometry(slot)
            }
            SpriteMainProgress::VitreousDamagePending(slot) => {
                MainLoopInterruption::SpriteMainVitreousDamagePending(slot)
            }
            SpriteMainProgress::VitreousAiPending(slot) => {
                MainLoopInterruption::SpriteMainVitreousAiPending(slot)
            }
            SpriteMainProgress::MiniMoldormAiPending(slot) => {
                MainLoopInterruption::SpriteMainMiniMoldormAiPending(slot)
            }
            SpriteMainProgress::VitreousPlayerDamagePending(slot) => {
                MainLoopInterruption::SpriteMainVitreousPlayerDamagePending(slot)
            }
            SpriteMainProgress::SwamolaSegmentDraw { slot, segment } => {
                MainLoopInterruption::SpriteMainSwamolaSegmentDraw { slot, segment }
            }
            SpriteMainProgress::TrinexxHeadDrawSetup(slot) => {
                MainLoopInterruption::SpriteMainTrinexxHeadDrawSetup(slot)
            }
            SpriteMainProgress::TrinexxBreathTileCollisionReturned(slot) => {
                MainLoopInterruption::SpriteMainTrinexxBreathTileCollisionReturned(slot)
            }
            SpriteMainProgress::LaserEyeDrawPrologue(slot) => {
                MainLoopInterruption::SpriteMainLaserEyeDrawPrologue(slot)
            }
            SpriteMainProgress::ZoraFireballMovement { slot, checkpoint } => {
                MainLoopInterruption::SpriteMainZoraFireballMovement { slot, checkpoint }
            }
            SpriteMainProgress::TrinexxHeadDraw { slot, segment } => {
                MainLoopInterruption::SpriteMainTrinexxHeadDraw { slot, segment }
            }
            SpriteMainProgress::SidenexxNeckTargetLoop { slot, step } => {
                MainLoopInterruption::SpriteMainSidenexxNeckTargetLoop { slot, step }
            }
            SpriteMainProgress::TrinexxFinalPhaseTileCollision {
                slot,
                probes_completed,
            } => MainLoopInterruption::SpriteMainTrinexxFinalPhaseTileCollision {
                slot,
                probes_completed,
            },
            SpriteMainProgress::TrinexxFinalPhaseDraw {
                slot,
                segment,
                stage,
            } => MainLoopInterruption::SpriteMainTrinexxFinalPhaseDraw {
                slot,
                segment,
                stage,
            },
            SpriteMainProgress::TrinexxHeadFrontPart {
                slot,
                completed_stores,
            } => MainLoopInterruption::SpriteMainTrinexxHeadFrontPart {
                slot,
                completed_stores,
            },
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
            SpriteMainProgress::HelmasaurHardHatTileCollision { slot, stage } => {
                MainLoopInterruption::SpriteMainHelmasaurHardHatTileCollision { slot, stage }
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
        return offset.is_multiple_of(3).then_some((offset / 3) as u8);
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
        return offset.is_multiple_of(4).then_some(35 + (offset / 4) as u8);
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
    stack4: Option<u8>,
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
    /// Project one decoded Z3TRACE1 record onto the adapter's event view.
    fn from_record(record: &parity::trace_format::TraceRecord) -> Self {
        RawTraceEvent {
            event: record.event().to_string(),
            stage: record.stage_name().map(str::to_string),
            run: Some(record.run),
            pc: Some(record.pc),
            s: Some(record.s),
            return_address: Some(record.return_address),
            stack1: Some(record.stack[0]),
            stack4: Some(record.stack[3]),
            a: Some(record.a),
            main: Some(record.main),
            sub: Some(record.sub),
            subsub: Some(record.subsub),
            room: Some(record.room),
            frame_counter: Some(record.frame_counter),
            nmi_latch: Some(record.nmi_latch),
            link_y: Some(record.link_y),
            bg2_v: Some(record.bg2_v),
            bg2_h: Some(record.bg2_h),
            spotlight_radius: Some(record.spotlight_radius),
            spotlight_var4_low: Some(record.spotlight_var4_low),
            palette_countdown: Some(record.palette_countdown),
            spotlight_lower_cursor: Some(record.spotlight_lower_cursor),
            joypad_high: Some(record.joypad_high),
            joypad_low: Some(record.joypad_low),
            joypad_high_filtered: Some(record.joypad_high_filtered),
            joypad_low_filtered: Some(record.joypad_low_filtered),
            x: Some(record.x),
            y: Some(record.y),
            address: record.address().and_then(|value| u16::try_from(value).ok()),
            value: record.value().and_then(|value| u8::try_from(value).ok()),
            nmi_ppu_register_operands: Some(record.nmi_ppu_register_operands),
        }
    }

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

/// Zelda's poly (intro / Triforce-room polyhedral) thread runs on stack page
/// `$1F`; the main thread's stack lives in page `$01`.
fn is_poly_thread_stack(stack: u16) -> bool {
    (0x1f00..=0x1fff).contains(&stack)
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
    mirror_portal_load_progress: Option<u8>,
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
        if event.event == "nmi-resume" {
            self.mirror_portal_load_progress = None;
        }
        if event.event == "nmi"
            || event.event == "frame" && event.stage.as_deref() == Some("return")
        {
            if let Some(slot) = self.mirror_portal_spawn_slot {
                if event.x == Some(u16::from(slot)) && event.return_address == Some(0xa60f09) {
                    if let Some(completed) = event
                        .pc
                        .and_then(sprite_prep_load_properties_completed_stores)
                    {
                        self.mirror_portal_load_progress = Some(completed);
                    }
                }
            }
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
        if returned.pc == 0x02_a4aa {
            if !matches!(returned.main, 9 | 0x0b) {
                return Err("Module09 scroll prefix escaped its source caller".into());
            }
            receipts.push(OriginalTimingSemanticReceipt::Module09FinalScrollPairPending);
        }
        // A portal reset belongs to the spawn proven by its private caller.
        if let Some(completed_stores) = self.mirror_portal_load_progress {
            let slot = self
                .mirror_portal_spawn_slot
                .expect("portal property load has a spawn owner");
            let receipt = receipts
                .iter_mut()
                .find(|receipt| {
                    **receipt
                        == OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                            OverworldSpriteReloadProgress::GenerationReturned,
                        )
                })
                .ok_or("mirror portal property load omitted its enclosing generation return")?;
            *receipt = OriginalTimingSemanticReceipt::OverworldSpriteReloadProgress(
                OverworldSpriteReloadProgress::GenerationReturnedAtPortalLoadProperties {
                    slot,
                    completed_stores,
                },
            );
        }
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
        .or(link_velocity_running_test_interruption(
            returned.pc,
            Some(returned.main),
            Some(returned.sub),
            returned.a,
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
                        "06d8e2", "06d8e5", "05cbe0", "05cbcd", "05eb1d", "05eb21", "068328",
                        "0683a7", "0684e2", "0684aa", "058af3", "0684eb", "069271", "06a628",
                        "06a724", "06b9cc", "06b9d0", "0799ad", "079a0b", "008225", "0082c7",
                        "00d4ed", "09c499", "09c4aa", "09c173", "09f63f", "09f825", "0ffdc3",
                        "00d423", "00e75c", "00e766", "00d44c", "069853", "069860", "06988d",
                        "0698b2", "06e4ab", "0ecfe2", "0ed088", "0ed0c2", "06d051", "02824d",
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
        // The pinned core writes Z3TRACE1 binary records; re-open at the
        // byte offset of the first record this host has not consumed.
        let mut reader =
            parity::trace_format::open_trace(&self.path, self.offset).map_err(|error| {
                format!(
                    "open Snes9x semantic trace {}: {error}",
                    self.path.display()
                )
            })?;
        let mut receipts = Vec::new();
        let mut host_frame = HostFrameWindow::default();
        let mut dialogue_scroll = DialogueScrollHostWindow::default();
        host_frame.overworld_load_overlays_sprite_reload_active =
            self.overworld_load_overlays_sprite_reload_active;
        let zelda_run_game_loop_call_active_at_entry = self.zelda_run_game_loop_call_active;
        let mut returned_event = None;
        loop {
            let record = match reader.next_record() {
                Ok(Some(record)) => record,
                Ok(None) => break,
                Err(error) => {
                    return Err(format!(
                        "read Snes9x semantic trace {} at byte {}: {error}",
                        self.path.display(),
                        reader.offset()
                    ))
                }
            };
            self.offset = reader.offset();
            let event = RawTraceEvent::from_record(&record);
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
                execution.observe_buzzblob_movement(returned_event)?;
                // A host ending inside NMI has not advanced the interrupted
                // drawing caller. Retain its precise checkpoint until resume.
                if self.nmi_resume_targets.is_empty() {
                    execution.observe_trinexx_head_draw(returned_event)?;
                    execution.observe_trinexx_breath_tile_collision(returned_event)?;
                    execution.observe_lanmola_draw_prefix(returned_event)?;
                    execution.observe_laser_eye_draw_prologue(returned_event);
                    execution.observe_sprite_handler_returned(returned_event)?;
                    execution.observe_trinexx_final_phase_tile_collision(returned_event)?;
                    execution.observe_helmasaur_hard_hat_tile_collision(returned_event)?;
                    execution.observe_trinexx_final_phase_draw(returned_event)?;
                    execution.observe_sidenexx_neck_target(returned_event)?;
                }
                execution.observe_guard_animation_checkpoint(returned_event)?;
                execution.observe_hog_spear_body_graphics_pending(returned_event)?;
                execution.observe_absorbable_tile_lookup(returned_event)?;
                execution.observe_swamola_segment_draw(returned_event)?;
                execution.observe_vitreous_damage_pending(returned_event);
                execution.observe_moblin_collision_geometry(returned_event);
                execution.observe_dispatch_trampoline_return(returned_event)?;
                execution.observe_pengator_slide_pending(returned_event)?;
                execution.observe_antifairy_bounce_pending(returned_event)?;
                execution.observe_kholdstare_damage_pending(returned_event)?;
                execution.observe_guard_prep_parry_hitbox(returned_event)?;
                execution.observe_guard_prep_patrol_delay(returned_event)?;
                execution.observe_guard_prep_tile_collision_return(returned_event)?;
                execution.observe_fire_debirando_spawn_boundary(returned_event)?;
                execution.observe_trinexx_death_spawn_boundary(returned_event)?;
                execution.observe_agahnim_motion_blur_spawn_boundary(returned_event)?;
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
                execution.observe_initialize_prep_move_y(returned_event)?;
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
            if let Some(next_index) = dungeon_push_blocks_in_progress(returned_event) {
                receipts.push(OriginalTimingSemanticReceipt::DungeonPushBlocksInProgress {
                    next_index,
                });
            }
            if dungeon_push_blocks_handled(returned_event) {
                receipts.push(OriginalTimingSemanticReceipt::DungeonPushBlocksHandled);
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
                            | MainLoopInterruption::LinkVelocityClearProgress { .. }
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
                self.consume_pc_event(&event, receipts)?;
            }
            "wram-write" => {
                self.consume_wram_write_event(&event, receipts)?;
            }
            "nmi" => {
                self.consume_nmi_event(&event, receipts)?;
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
    // Sprite_ResetAll's JSL return identifies the shared disable loop.
    if event.main == Some(6)
        && return_address == Some(0x09_c451)
        // The next stack byte belongs to Sprite_ResetAll's outer JSL.
        // Module_PreDungeon returns to $02:834B; the spotlight goal caller
        // also publishes module 6 before resetting but has return low byte $22.
        && event.stack4 == Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC as u8)
        && matches!(pc, Some(0x09_c288 | 0x09_c28c | 0x09_c28d))
    {
        let x = event.x.ok_or("Sprite_ResetAll garnish loop omitted X")? as u8;
        let slot = if pc == Some(0x09_c28c) {
            x
        } else {
            x.wrapping_add(1)
        };
        if slot > 30 {
            return Err("Sprite_ResetAll garnish loop has invalid clear cursor".into());
        }
        receipts.retain(|r| {
            !matches!(
                r,
                OriginalTimingSemanticReceipt::PreDungeonSpriteDisableThrough { .. }
                    | OriginalTimingSemanticReceipt::PreDungeonGarnishDisableThrough { .. }
            )
        });
        receipts.push(
            OriginalTimingSemanticReceipt::PreDungeonGarnishDisableThrough { slot, boundary },
        );
        return Ok(true);
    }
    // At DEX the current slot's STZ has completed; BPL follows that DEX.
    if event.main == Some(6)
        && return_address == Some(0x09_c451)
        && matches!(pc, Some(0x09_c234 | 0x09_c235))
    {
        let x = event.x.ok_or("Sprite_ResetAll disable loop omitted X")?;
        let slot = if pc == Some(0x09_c234) {
            x
        } else {
            (x as u8).wrapping_add(1) as u16
        };
        if slot >= 16 {
            return Err("Sprite_ResetAll disable loop has invalid slot".into());
        }
        receipts.retain(|r| {
            !matches!(
                r,
                OriginalTimingSemanticReceipt::PreDungeonSpriteDisableThrough { .. }
            )
        });
        receipts.push(
            OriginalTimingSemanticReceipt::PreDungeonSpriteDisableThrough {
                slot: slot as u8,
                boundary,
            },
        );
        return Ok(true);
    }
    let pre_dungeon_caller = return_address == Some(MODULE_PRE_DUNGEON_AFTER_SPRITE_RESET_PC);
    // Module18_GanonEmerges' pyramid-area load (state 3) runs the same
    // FluteMenu_LoadSelectedScreen stack under main module $18 (route host
    // 1547878).
    let bird_travel_caller = matches!(
        (event.main, event.sub),
        (Some(0x0e), Some(0x0a)) | (Some(0x18), Some(0))
    ) && matches!(
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
                Some((((u32::from(pending_circle_input) << 8) / u32::from(radius)) >> 1) as u16)
            } else {
                None
            };
            if retained_x == Some(observed_x)
                && matched
                    .replace((completed_iterations, upper_cursor, lower_cursor))
                    .is_some()
            {
                return Err(format!(
                        "Snes9x spotlight loop-test X {observed_x} maps to multiple source iterations without a unique source cursor",
                    ));
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
                x.div_ceil(2) * 8
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
    if main == Some(0x0f)
        && sub == Some(1)
        && matches!(x, Some(0 | 1))
        && (0x07_e359..=0x07_e361).contains(&pc)
    {
        // STA $27,x has returned. Only scratch-mask shifts precede DEX:
        // X=1 proves the horizontal component, X=0 proves both components.
        return Some(if x == Some(1) {
            MainLoopInterruption::LinkActualVelocity {
                horizontal_resolved: Some(true),
            }
        } else {
            MainLoopInterruption::LinkActualVelocityCompleted
        });
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
    } else if main == Some(0x0f) && sub == Some(1) && (0x07_e2cc..0x07_e2d2).contains(&pc) {
        Some(MainLoopInterruption::LinkVelocityClearProgress {
            completed: ((pc - 0x07_e2ca) / 2) as u8,
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
            // The moving/running branch tests after LDA $0372: the BEQ at
            // $07:E282, the moving-and-running scratch speed (LDA #$18 /
            // STA $00 / BRA) and, on the non-moving path, the LDA $0372 /
            // BEQ pair plus the still-pending STZ $57. None of these has
            // published gameplay state.
            || matches!(
                pc,
                0x07_e282 | 0x07_e284 | 0x07_e286 | 0x07_e288 | 0x07_e28d | 0x07_e290 | 0x07_e292
            )
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
        if !offset.is_multiple_of(3) {
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
    if let Some(interruption) =
        link_velocity_running_test_interruption(pc, event.main, event.sub, event.a)?
    {
        return Ok(Some(interruption));
    }
    Ok(main_loop_interruption_for_source_state(
        pc, event.main, event.sub, event.x,
    ))
}

/// Link_HandleVelocity's `LDA $0316` at $07:E29E is reached either by the
/// BEQ on link_is_running ($0372) or after `STZ $57` and the dash counter
/// test. A zero accumulator proves the BEQ path, so the speed-modifier
/// clear has not executed and no gameplay state is stored yet.
fn link_velocity_running_test_interruption(
    pc: u32,
    main: Option<u8>,
    sub: Option<u8>,
    a: Option<u16>,
) -> Result<Option<MainLoopInterruption>, String> {
    if main != Some(0x0f) || sub != Some(1) || pc & 0x00ff_ffff != 0x07_e29e {
        return Ok(None);
    }
    if a.map(|a| a & 0xff) == Some(0) {
        return Ok(Some(MainLoopInterruption::LinkPositionBeforeCoordinates));
    }
    Err("Link_HandleVelocity boundary after the running speed-modifier clear is unmodeled".into())
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
                MainLoopInterruption::SpriteMainTrinexxDeathExplosionSpawn {
                    slot,
                    spawned_slot,
                    progress,
                } => Some(SpriteMainProgress::TrinexxDeathExplosionSpawn {
                    slot,
                    spawned_slot,
                    progress,
                }),
                MainLoopInterruption::SpriteMainAgahnimMotionBlurSpawn {
                    slot,
                    spawned_slot,
                    progress,
                } => Some(SpriteMainProgress::AgahnimMotionBlurSpawn {
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
                MainLoopInterruption::SpriteMainBuzzblobAfterXSubpixel(slot) => {
                    Some(SpriteMainProgress::BuzzblobAfterXSubpixel(slot))
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
                MainLoopInterruption::SpriteMainMoblinAttributeLoaded(slot) => {
                    Some(SpriteMainProgress::MoblinAttributeLoaded(slot))
                }
                MainLoopInterruption::SpriteMainMoblinCollisionGeometry(slot) => {
                    Some(SpriteMainProgress::MoblinCollisionGeometry(slot))
                }
                MainLoopInterruption::SpriteMainVitreousDamagePending(slot) => {
                    Some(SpriteMainProgress::VitreousDamagePending(slot))
                }
                MainLoopInterruption::SpriteMainVitreousAiPending(slot) => {
                    Some(SpriteMainProgress::VitreousAiPending(slot))
                }
                MainLoopInterruption::SpriteMainMiniMoldormAiPending(slot) => {
                    Some(SpriteMainProgress::MiniMoldormAiPending(slot))
                }
                MainLoopInterruption::SpriteMainVitreousPlayerDamagePending(slot) => {
                    Some(SpriteMainProgress::VitreousPlayerDamagePending(slot))
                }
                MainLoopInterruption::SpriteMainSwamolaSegmentDraw { slot, segment } => {
                    Some(SpriteMainProgress::SwamolaSegmentDraw { slot, segment })
                }
                MainLoopInterruption::SpriteMainTrinexxHeadDrawSetup(slot) => {
                    Some(SpriteMainProgress::TrinexxHeadDrawSetup(slot))
                }
                MainLoopInterruption::SpriteMainTrinexxBreathTileCollisionReturned(slot) => {
                    Some(SpriteMainProgress::TrinexxBreathTileCollisionReturned(slot))
                }
                MainLoopInterruption::SpriteMainLaserEyeDrawPrologue(slot) => {
                    Some(SpriteMainProgress::LaserEyeDrawPrologue(slot))
                }
                MainLoopInterruption::SpriteMainZoraFireballMovement { slot, checkpoint } => {
                    Some(SpriteMainProgress::ZoraFireballMovement { slot, checkpoint })
                }
                MainLoopInterruption::SpriteMainInitializePrepMoveY { slot, checkpoint } => {
                    Some(SpriteMainProgress::InitializePrepMoveY { slot, checkpoint })
                }
                MainLoopInterruption::SpriteMainTrinexxHeadDraw { slot, segment } => {
                    Some(SpriteMainProgress::TrinexxHeadDraw { slot, segment })
                }
                MainLoopInterruption::SpriteMainSidenexxNeckTargetLoop { slot, step } => {
                    Some(SpriteMainProgress::SidenexxNeckTargetLoop { slot, step })
                }
                MainLoopInterruption::SpriteMainTrinexxFinalPhaseTileCollision {
                    slot,
                    probes_completed,
                } => Some(SpriteMainProgress::TrinexxFinalPhaseTileCollision {
                    slot,
                    probes_completed,
                }),
                MainLoopInterruption::SpriteMainTrinexxFinalPhaseDraw {
                    slot,
                    segment,
                    stage,
                } => Some(SpriteMainProgress::TrinexxFinalPhaseDraw {
                    slot,
                    segment,
                    stage,
                }),
                MainLoopInterruption::SpriteMainTrinexxHeadFrontPart {
                    slot,
                    completed_stores,
                } => Some(SpriteMainProgress::TrinexxHeadFrontPart {
                    slot,
                    completed_stores,
                }),
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

/// A boundary inside Dungeon_PushBlock_Handler's loop ($01:D7C8..D823), JSL'd
/// from Module07_Dungeon ($02:87B2). Y holds the misc object word offset of
/// the entry being examined; at the two INCs and the reload it has been
/// processed, at the loop test Y already holds the advanced offset. Only the
/// loop's own level is modeled: entries whose handling was interrupted
/// inside RoomDraw/PushBlock helpers keep the fail-closed default.
fn dungeon_push_blocks_in_progress(event: &RawTraceEvent) -> Option<u16> {
    if event.main != Some(7) || event.return_address.map(|pc| pc & 0xff_ffff) != Some(0x02_87b5) {
        return None;
    }
    let y = event.y?;
    match event.pc.map(|pc| pc & 0xff_ffff)? {
        0x01_d7c8 | 0x01_d7cb => Some(y),
        0x01_d815 | 0x01_d818 | 0x01_d81b | 0x01_d81d => Some(y.wrapping_add(2)),
        0x01_d820 | 0x01_d823 => Some(y),
        _ => None,
    }
}

/// A boundary in OrientLampLightCone's guard ($00:F566..F570, JSL'd from
/// Module07_Dungeon at $02:87E1 right after Dungeon_PushBlock_Handler): the
/// handler has returned and nothing of the lamp cone or scroll copies ran.
fn dungeon_push_blocks_handled(event: &RawTraceEvent) -> bool {
    event.main == Some(7)
        && event.return_address.map(|pc| pc & 0xff_ffff) == Some(0x02_87e4)
        && matches!(
            event.pc.map(|pc| pc & 0xff_ffff),
            Some(0x00_f566 | 0x00_f567 | 0x00_f56a | 0x00_f56c | 0x00_f56e | 0x00_f570)
        )
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
mod tests;
