//! Retained source-ordered CPU timeline alongside translated native execution.
//!
//! This owner is opt-in while native scheduling still consumes the aggregate
//! ROM timing plans. It does not publish source WRAM or presentation state.

#[cfg(feature = "rom-timing")]
use super::{decompress_asset, find_index_in_memblk, MemBlk, SPOTLIGHT_VISIBLE_SCANLINES};
use super::{
    spotlight_table_row_pairs, spotlight_vertical_center,
    player_oam, sprite::OverworldSpriteReloadScanWork,
    DungeonSupertileTransitionWork, ExtendedOamPackingProgress, GameState, OriginalTimingOwnerState,
    GameWorkContinuation, ItemReceiptGraphicsContinuation, ItemReceiptReturn,
    OverworldSpriteReloadWorkload, PreMainNmiResume, SpriteMainCpuBoundary, SpriteSlotsState, ZeldaState,
};
#[cfg(feature = "rom-timing")]
use crate::game_state::constants::{
    HDMA_TABLE_DYNAMIC, RESERVED_HDMA_TABLE,
    ANCILLA_STEP, DIALOGUE_MSG_SRC_OFFS, OVERWORLD_DECOMP_BUFFER, OVERWORLD_MAP16_DECODE_SRC,
    TEXT_DIALOGUE_POINTERS,
};
#[cfg(feature = "rom-timing")]
use crate::rom_cpu_timing::ROM_DIALOGUE_MESSAGE_COUNT;
#[cfg(feature = "rom-timing")]
use crate::DungeonResetSpritesCpuProgress;
use crate::{
    CachedSpriteCacheField, DungeonResetSpritesProgressReceipt,
    OriginalTimingBoundary,
};
use snes::Snes9xColdCpuExecutor;
#[cfg(feature = "rom-timing")]
use snes::{Snes9xCpuQuiescentCheckpoint, Snes9xCpuQuiescentCheckpointError,
    SourceCpuAcceptedInterrupt, SourceCpuBusAccessKind, SourceCpuError, SourceCpuStepReceipt};
use std::collections::HashMap;
use super::load_gfx::{AnimatedOverworldGraphicsProgress, TilesetGraphicsPublication};
use super::load_gfx::{AnimatedDungeonGraphicsProgress, AnimatedDungeonGraphicsContinuation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct NativeExactCpuNmi {
    pub interrupted_pc: u32,
    pub accepted_at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuNmiCompletion {
    pub accepted_at: u64,
    pub returned_at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum NativeExactCpuInterrupt {
    Nmi(NativeExactCpuNmi),
    Irq,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuPpuAccess {
    pub pc: u32,
    pub address: u32,
    pub value: u8,
    pub at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Shared timing vocabulary is inspected by ungated gameplay accessors.
#[cfg_attr(not(feature = "rom-timing"), allow(dead_code))]
pub(super) enum NativeExactCpuNmiGateKind {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuNmiGateAccess {
    pub pc: u32,
    pub address: u16,
    pub kind: NativeExactCpuNmiGateKind,
    pub value: u16,
    pub width: u8,
    pub at: u64,
    pub nmi_accepted_at: Option<u64>,
}

/// Bus-ordered access to one opt-in WRAM address in the retained CPU owner.
/// This is diagnostic provenance, never a translated-memory publication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuWramWatchAccess {
    pub pc: u32,
    pub kind: NativeExactCpuNmiGateKind,
    pub value: u16,
    pub width: u8,
    pub at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuMusicAccess {
    pub pc: u32,
    pub address: u16,
    pub kind: NativeExactCpuNmiGateKind,
    pub value: u16,
    pub width: u8,
    pub at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuNmiUpdateDecision {
    pub accepted_at: u64,
    pub decided_at: u64,
    pub latch: u8,
    pub runs_updates: bool,
}

/// The shared main-loop suffix finished `NMI_PrepareSprites` and released
/// its $12 latch. A caller interrupted inside sprite preparation cannot
/// publish this return until after the carried NMI handler completes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuMainWaitReturn {
    pub at: u64,
}

/// ZeldaRunGameLoop began a fresh iteration and incremented its own frame
/// counter after a suspended caller returned within the same host interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuMainLoopEntry {
    pub frame_counter: u8,
    pub at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuSpritePreparationInterruption {
    pub accepted_at: u64,
    pub interrupted_pc: u32,
    pub progress: ExtendedOamPackingProgress,
}

/// `Dungeon_LoadHeader` published the room's three-byte table index before
/// the long room draw crossed its first NMI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuDungeonHeaderIndexWrite {
    pub value: u16,
    pub at: u64,
}

/// A source floor-chunk store. The address and value are witnesses for the
/// translated cursor; neither is copied into native tilemap memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuFloorTileWrite {
    pub ordinal: u16,
    pub address: u32,
    pub value: u16,
    pub at: u64,
}

/// A room object has returned to the stream parser. Native drawing can
/// advance one complete object without borrowing source tilemap contents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuObjectCompletion {
    pub ordinal: u16,
    pub at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Shared timing vocabulary is inspected by ungated gameplay accessors.
#[cfg_attr(not(feature = "rom-timing"), allow(dead_code))]
pub(super) enum NativeExactCpuRoomUploadEvent {
    Began { at: u64 },
    QuadrantPrepared { count: u8, at: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NativeExactCpuItemReceiptStepWrite {
    pub slot: u8,
    pub value: u8,
    pub at: u64,
}

/// Statement boundaries in the source spotlight row builder. These identify
/// when a translated builder may begin or advance; the bus value is a witness,
/// never a replacement for the native spotlight state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Shared timing vocabulary is inspected by ungated gameplay accessors.
#[cfg_attr(not(feature = "rom-timing"), allow(dead_code))]
pub(super) enum NativeExactCpuSpotlightEvent {
    WindowYInitialized { value: u16, at: u64 },
    CircleInputRead { value: u16, at: u64 },
    RowAdvanced { at: u64 },
    ProjectionWordStored { at: u64 },
}

/// Source statement boundaries for the original-ROM overworld proximity scan.
/// The translated scan may consume these events without importing source WRAM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
// Shared timing vocabulary is inspected by ungated gameplay accessors.
#[cfg_attr(not(feature = "rom-timing"), allow(dead_code))]
pub(super) enum NativeExactCpuOverworldScanEvent {
    Began { at: u64 },
    CellReturned { at: u64, ordinal: u16 },
    Finished { at: u64, cells: u16 },
}

/// A diagnostic translated caller driven solely by source CPU statement
/// boundaries. It never publishes into the live native state. Once the
/// shadow agrees with the eager generation, the live Module09 caller can be
/// migrated without using source WRAM as an answer key.
#[derive(Clone, Debug)]
pub(super) struct NativeExactCpuOverworldScanTrial {
    shadow: Box<ZeldaState>,
    work: OverworldSpriteReloadScanWork,
    began: bool,
    expected_final_ram: Option<Vec<u8>>,
    expected_final_game_state: Option<Box<GameState>>,
}

/// Opt-in live caller experiment. The cursor belongs to translated gameplay;
/// only its progress boundary comes from the retained CPU.
#[derive(Clone, Debug)]
pub(super) struct NativeExactCpuOverworldLiveScan {
    sprite_records: usize,
    work: Option<OverworldSpriteReloadScanWork>,
    began: bool,
    expected_final_slots: SpriteSlotsState,
}

impl ZeldaState {
    pub(super) fn native_exact_cpu_link_oam_equipment_is_pending(&self) -> bool {
        crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_LINK_OAM_LIVE_TRIAL").is_some()
            && self.native_exact_cpu_host_trace.as_ref()
                .is_some_and(|trace| trace.link_oam_equipment_selection)
    }

    pub(super) fn advance_native_exact_cpu_link_oam_prefix(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_LINK_OAM_LIVE_TRIAL").is_some()
            && self.native_exact_cpu_host_trace.as_ref()
                .is_some_and(|trace| trace.link_oam_body_selection) {
            self.refine_dungeon_link_oam_to_body_selection();
        }
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_LINK_OAM_LIVE_TRIAL").is_some() {
            let arrow_progress = self.native_exact_cpu_host_trace.as_ref().and_then(|trace| {
                trace.module07_caller_passed_link_without_nmi(self.game_state.frame.frame_counter)
                    .then(|| Self::hud_arrow_digit_progress_at_pc(trace.return_pc)).flatten()
            });
            if let Some(master_cycles) = arrow_progress {
                self.refine_dungeon_subtile_caller_to_hud_arrow_digits(master_cycles);
            }
            if let Some(progress) = self.native_exact_cpu_host_trace.as_ref()
                .and_then(|trace| trace.link_oam_lower_body_progress) {
                self.refine_dungeon_link_oam_body_progress(Some(progress));
            }
        }
    }

    pub(super) fn advance_native_exact_cpu_sprite_timer_prefix(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_SPRITE_TIMERS_LIVE_TRIAL").is_none() {
            return;
        }
        if let Some((boundary, _)) = self.take_native_exact_cpu_follower_body_draw_boundary() {
            self.refine_dungeon_spiral_caller_to_sprite_prefix(boundary);
        }
        if let Some((slot, next_instruction, _)) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.sprite_coordinate_prefix) {
            self.refine_sprite_main_to_coordinate_moves(slot, next_instruction);
        }
        if let Some(slot) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.sprite_timer_return_slot) {
            self.refine_sprite_main_to_timer_return(slot);
        }
        if let Some(slot) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.guard_offscreen_kill_self_slot) {
            self.refine_sprite_main_to_guard_offscreen_kill(slot);
        }
        if let Some(slot) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.guard_active_state_checked_slot) {
            self.refine_sprite_main_to_guard_checkpoint(slot,
                crate::GuardAnimationCheckpoint::ActiveStateChecked);
        }
        if let Some(slot) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.rat_oam_flags_pending_slot) {
            self.refine_sprite_main_to_rat_oam_flags_store(slot);
        }
        if let Some(slot) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.keese_inactive_check_slot) {
            self.refine_sprite_main_to_keese_inactive_check(slot);
        }
    }

    pub(super) fn advance_native_exact_cpu_story_decompression(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_STORY_DECOMP_LIVE_TRIAL")
            .is_none()
        {
            return;
        }
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else {
            return;
        };
        if trace.story_decompression_writes.is_empty() {
            return;
        }
        if !matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDialogueInitializationPrefix { .. })
        ) {
            return;
        }
        let first = self.decompressed_sprite_graphics_data(0x67)
            .expect("story sprite sheet 0x67 is missing");
        let second = self.decompressed_sprite_graphics_data(0x68)
            .expect("story sprite sheet 0x68 is missing");
        for write in &trace.story_decompression_writes {
            for byte_index in 0..usize::from(write.width) {
                let address = write.address as usize + byte_index;
                let (sheet, offset) = if address < 0x7f_4800 {
                    (&first, address - 0x7f_4000)
                } else {
                    (&second, address - 0x7f_4800)
                };
                let native_byte = *sheet.get(offset)
                    .expect("source story decompression exceeded the native sheet");
                let source_byte = (write.value >> (byte_index * 8)) as u8;
                assert_eq!(native_byte, source_byte,
                    "native story sheet disagrees with the source decompressor");
                self.ram[address - 0x7e_0000] = native_byte;
            }
        }
    }

    pub(super) fn arm_native_exact_cpu_leading_spiral_sprite_nmi(&mut self) {
        if crate::debug_env::var_os(
            "ZELDA3_NATIVE_EXACT_CPU_SPIRAL_SPRITE_LEADING_NMI_LIVE_TRIAL",
        )
        .is_none()
            || self.game_state.frame.main_module != 7
            || self.game_state.frame.submodule != 0x0e
            || self.game_state.frame.subsubmodule != 6
            || !self.game_execution_scheduler.is_idle()
        {
            return;
        }
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else {
            return;
        };
        let Some(main_entry) = trace.main_loop_entries.first() else {
            return;
        };
        let leading_open_nmi = trace.nmi_update_decisions.iter().any(|decision| {
            decision.runs_updates
                && trace.nmi_completions.iter().any(|completion| {
                    completion.accepted_at == decision.accepted_at
                        && completion.returned_at < main_entry.at
                })
        });
        if leading_open_nmi {
            // The prior state published BG character request 10 after its
            // trailing handler. Hardware consumes it before the next fresh
            // spiral SpriteGFX iteration. The native schedule would enter
            // the long graphics call first and delay the NMI until it returns.
            self.game_execution_scheduler.schedule_pre_main_nmi_resume(
                PreMainNmiResume::DungeonSupertileNextIterationAfterLeadingNmi,
            );
        }
    }

    pub(super) fn advance_native_exact_cpu_bg_chars_gate(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_BG_CHARS_GATE_LIVE_TRIAL").is_none() {
            return;
        }
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else {
            return;
        };
        // The two BG-character DMA calls publish their request only after
        // their CPU-owned conversion/transition statement completes.
        let published: Vec<u8> = trace
            .nmi_gate_accesses
            .iter()
            .filter_map(
                |access| match (access.pc, access.address, access.kind, access.value) {
                    (0x02_8e17, 0x0710, NativeExactCpuNmiGateKind::Write, 9) => Some(9),
                    (0x02_8e21, 0x0710, NativeExactCpuNmiGateKind::Write, 10) => Some(10),
                    _ => None,
                },
            )
            .collect();
        for subroutine in published {
            self.set_pending_nmi_subroutine(subroutine);
            self.set_core_update_disable_flag(subroutine);
        }
    }

    /// A shared BG conversion can finish after its carried held handler,
    /// reaching Main's wait without accepting another NMI. Its native caller
    /// must retire after that handler so the caller's latch clear survives.
    pub(super) fn native_exact_cpu_bg_conversion_returns_after_held_nmi(&self) -> bool {
        if matches!(self.original_timing_owner, OriginalTimingOwnerState::Live)
            || !matches!(self.game_execution_scheduler.current_work(), Some(
                GameWorkContinuation::FinishDungeonSupertileTransition { work:
                    DungeonSupertileTransitionWork::SpiralBgCharacters34
                    | DungeonSupertileTransitionWork::FallingBgCharacters34
                    | DungeonSupertileTransitionWork::StraightInterroomBgCharacters34 }
            )) {
            return false;
        }
        self.native_exact_cpu_host_trace.as_ref()
            .is_some_and(NativeExactCpuHostTrace::caller_returns_after_held_nmi)
    }

    /// A landing-wipe build left inside its row loop at the CPU host return.
    /// The ordinal comes from completed `$F39C` cursor updates, not from a
    /// frame estimate or source WRAM copy. A host may stop mid-call without
    /// accepting an NMI, so the retained CPU's return PC is authoritative in
    /// that case.
    pub(super) fn native_exact_cpu_landing_spotlight_checkpoint(
        &self,
    ) -> Option<crate::SpotlightTableBuildProgress> {
        if self.game_state.frame.main_module != 7
            || self.game_state.frame.submodule != 0x0f
            || self.game_state.frame.subsubmodule != 1
        {
            return None;
        }
        self.native_exact_cpu_spotlight_build_checkpoint()
    }

    /// Shared table-builder progress. Call after the owning spotlight entry
    /// has initialized its native controls, so source values remain witnesses.
    pub(super) fn native_exact_cpu_spotlight_build_checkpoint(
        &self,
    ) -> Option<crate::SpotlightTableBuildProgress> {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_SPOTLIGHT_LIVE_TRIAL").is_none() {
            return None;
        }
        let trace = self.native_exact_cpu_host_trace.as_ref()?;
        let center = spotlight_vertical_center(
            self.game_state.player.follower_link.y(),
            self.game_state.display.ppu_scroll_copy.bg2_v_copy2(),
        );
        let progress = trace.spotlight_build_progress(|completed_iterations| {
            let lower = self.spotlight_lower_cursor_after_completed_rows(completed_iterations);
            (center.wrapping_mul(2).wrapping_sub(lower), lower)
        })?;
        assert!(
            if matches!(progress.checkpoint, crate::SpotlightTableBuildCheckpoint::ProjectionCopy { .. }) {
                progress.completed_iterations == spotlight_table_row_pairs(center)
            } else {
                progress.completed_iterations < spotlight_table_row_pairs(center)
            },
            "source spotlight progress exceeds the native row-pair loop",
        );
        if let Some(NativeExactCpuSpotlightEvent::WindowYInitialized { value, .. }) =
            trace.spotlight_events.iter().rev().find(|event| {
                matches!(
                    event,
                    NativeExactCpuSpotlightEvent::WindowYInitialized { .. }
                )
            })
        {
            assert_eq!(
                *value,
                self.game_state.display.spotlight_hdma.window_radius(),
                "source spotlight prologue disagrees with the native radius",
            );
        }
        Some(progress)
    }

    pub(super) fn advance_native_exact_cpu_pre_dungeon_room_header(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_ROOM_HEADER_LIVE_TRIAL").is_none() {
            return;
        }
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else {
            return;
        };
        if trace.dungeon_header_index_writes.is_empty()
            || !matches!(
                self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishPreDungeonEntranceLoad { .. })
            )
        {
            return;
        }
        let host = trace.host;
        self.begin_early_dungeon_room_header_if_source_returned(
            super::dungeon::EarlyDungeonRoomOwner::PreDungeon,
        );
        eprintln!("native-exact-cpu-live-pre-dungeon-room-header host={host}");
    }

    pub(super) fn advance_native_exact_cpu_floor_live_draw(&mut self) {
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else {
            return;
        };
        if trace.floor_tile_writes.is_empty() {
            return;
        }
        let host = trace.host;
        let writes = trace.floor_tile_writes.clone();
        let Some(mut work) = self.active_early_dungeon_floor_draw.take() else {
            return;
        };
        for write in writes {
            self.advance_dungeon_floor_draw_one(
                &mut work,
                write.ordinal,
                write.address,
                write.value,
            );
        }
        eprintln!(
            "native-exact-cpu-live-floor host={host} completed_words={}",
            work.completed_words()
        );
        if work.completed_words() == super::dungeon::DungeonFloorDrawWork::TOTAL_WORDS
            && crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_OBJECT_DRAW_LIVE_TRIAL").is_some()
        {
            assert!(self.active_early_dungeon_object_draw.is_none());
            self.active_early_dungeon_object_draw = self.begin_dungeon_object_draw_current_room();
            assert!(
                self.active_early_dungeon_object_draw.is_some(),
                "source object draw entered without translated room layout"
            );
        }
        self.active_early_dungeon_floor_draw = Some(work);
    }

    pub(super) fn advance_native_exact_cpu_object_live_draw(&mut self) {
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else {
            return;
        };
        if trace.object_completions.is_empty() && trace.object_4x2_boundary.is_none() {
            return;
        }
        let host = trace.host;
        let completions = trace.object_completions.clone();
        let boundary = trace.object_4x2_boundary;
        let Some(mut work) = self.active_early_dungeon_object_draw.take() else {
            return;
        };
        for completion in completions {
            assert!(
                self.advance_dungeon_object_draw_one(&mut work),
                "source completed more objects than translated room stream"
            );
            assert_eq!(
                work.completed_objects(),
                completion.ordinal,
                "source and translated object streams lost position"
            );
        }
        if let Some((ordinal, boundary)) = boundary {
            self.advance_dungeon_object_4x2_prefix(&mut work, ordinal, boundary);
        }
        eprintln!(
            "native-exact-cpu-live-objects host={host} completed_objects={}",
            work.completed_objects()
        );
        self.active_early_dungeon_object_draw = Some(work);
    }

    pub(super) fn advance_native_exact_cpu_post_object_prefix(&mut self) {
        let Some(source_value) = self.native_exact_cpu_host_trace.as_ref().and_then(
            |trace| trace.room_post_object_prefix_value,
        ) else {
            return;
        };
        if self.active_early_dungeon_floor_draw.is_none() {
            return;
        }
        let mut objects = self.active_early_dungeon_object_draw.take()
            .expect("source post-object store lost its translated object cursor");
        assert!(
            !self.advance_dungeon_object_draw_one(&mut objects),
            "source published the torch range before native objects completed",
        );
        assert!(objects.finished());
        self.dungeon_load_room_post_object_prefix();
        assert_eq!(
            self.game_state.dungeon.torch.torches_start_index(),
            source_value,
            "native post-object torch range disagrees with the source store",
        );
    }

    pub(super) fn take_native_exact_cpu_follower_body_draw_boundary(
        &mut self,
    ) -> Option<(SpriteMainCpuBoundary, OriginalTimingBoundary)> {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_SPRITE_TIMERS_LIVE_TRIAL").is_none() {
            return None;
        }
        self.native_exact_cpu_host_trace.as_mut()?.follower_body_draw_prefix.take()
    }

    pub(super) fn take_native_exact_cpu_quadrant_copy_boundary(
        &mut self,
    ) -> Option<(u16, OriginalTimingBoundary)> {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_ROOM_UPLOAD_LIVE_TRIAL").is_none() {
            return None;
        }
        self.native_exact_cpu_host_trace.as_mut()?.quadrant_copy_boundary.take()
    }

    pub(super) fn advance_native_exact_cpu_zelda_follower_graphics(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_SPRITE_TIMERS_LIVE_TRIAL").is_none() {
            return;
        }
        let Some((slot, stage)) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.zelda_follower_graphics_progress)
        else {
            return;
        };
        let Some(GameWorkContinuation::FinishSpriteMain { boundary, caller }) =
            self.game_execution_scheduler.current_work()
        else {
            panic!("source Zelda follower graphics lost its native Sprite_Main caller");
        };
        self.lane_live_sprite_main_refined_boundary(
            Some(super::SpriteMainCpuBoundary::FollowerGraphics {
                slot,
                caller: crate::SpriteFollowerGraphicsCaller::Zelda,
                prefix_completed: false,
                saved_follower_indicator: None,
                stage,
            }),
            Some((boundary, caller)),
        );
        assert_eq!(self.game_state.sprites.follower_runtime.indicator(), 1);
    }

    pub(super) fn advance_native_exact_cpu_big_key_graphics(&mut self) {
        let Some((first_sheet, completed, source_byte)) = self.native_exact_cpu_host_trace
            .as_ref()
            .and_then(|trace| trace.sprite_decompression_progress)
        else {
            return;
        };
        let Some(GameWorkContinuation::FinishBigKeyDropGraphics {
            sprite_slot,
            dungeon,
            progress,
        }) = self.game_execution_scheduler.current_work() else {
            return;
        };
        use super::AnimatedSpriteTileDecompressionProgress::{FirstSheet, SecondSheet};
        let next = if first_sheet { FirstSheet(completed) } else { SecondSheet(completed) };
        match (progress, next) {
            (None, FirstSheet(end)) => {
                self.apply_animated_sprite_tile_sheet_slice(0x22, true, 0, end);
            }
            (None, SecondSheet(end)) => {
                self.apply_animated_sprite_tile_sheet_slice(0x22, true, 0, 0x0600);
                self.apply_animated_sprite_tile_sheet_slice(0x22, false, 0, end);
            }
            (Some(FirstSheet(start)), FirstSheet(end)) => {
                assert!(start <= end, "source animated first sheet moved backward");
                self.apply_animated_sprite_tile_sheet_slice(0x22, true, start, end);
            }
            (Some(FirstSheet(start)), SecondSheet(end)) => {
                self.apply_animated_sprite_tile_sheet_slice(0x22, true, start, 0x0600);
                self.apply_animated_sprite_tile_sheet_slice(0x22, false, 0, end);
            }
            (Some(SecondSheet(start)), SecondSheet(end)) => {
                assert!(start <= end, "source animated second sheet moved backward");
                self.apply_animated_sprite_tile_sheet_slice(0x22, false, start, end);
            }
            (Some(SecondSheet(_)), FirstSheet(_)) => {
                panic!("source animated sprite loader returned to its first sheet");
            }
        }
        let base = if first_sheet { 0x14600 } else { 0x14000 };
        assert_eq!(
            self.ram[base + usize::from(completed) - 1],
            source_byte,
            "native decoded animated sheet disagrees with the source store",
        );
        self.game_execution_scheduler.refine_scheduled_work(
            GameWorkContinuation::FinishBigKeyDropGraphics { sprite_slot, dungeon, progress },
            GameWorkContinuation::FinishBigKeyDropGraphics {
                sprite_slot,
                dungeon,
                progress: Some(next),
            },
        );
    }

    /// Publish the translated `LoadTransAuxGFX` output through the source
    /// decompressor's last store. Both decoders emit each sheet in address
    /// order, so the store address alone is the cursor; the source value is
    /// only compared.
    pub(super) fn advance_native_exact_cpu_spiral_aux_graphics(&mut self) {
        let Some((address, source_value)) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.aux_graphics_decompression_store)
        else {
            return;
        };
        if !matches!(
            self.game_execution_scheduler.current_work(),
            Some(GameWorkContinuation::FinishDungeonSupertileTransition {
                work: super::DungeonSupertileTransitionWork::SpiralRoomInitialization,
            })
        ) {
            return;
        }
        if self.active_spiral_aux_graphics.is_none() {
            if self.active_early_dungeon_floor_draw.is_none() {
                return;
            }
            self.finish_live_dungeon_supertile_room_load();
            self.active_spiral_aux_graphics = Some(self.begin_spiral_aux_graphics_publication());
        }
        let mut publication = self.active_spiral_aux_graphics.take().unwrap();
        publication.publish_through(address, &mut self.ram);
        assert_eq!(
            self.ram[address], source_value,
            "native LoadTransAuxGFX byte at ${address:04x} disagrees with the source store",
        );
        self.active_spiral_aux_graphics = Some(publication);
    }

    pub(super) fn advance_native_exact_cpu_pre_dungeon_animated_graphics(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_PRE_DUNGEON_GRAPHICS_LIVE_TRIAL").is_none()
            || !matches!(self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishPreDungeonEntranceLoad { .. })) {
            return;
        }
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else { return };
        let target = trace.animated_dungeon_graphics_progress;
        let tileset_events = trace.pre_dungeon_tileset_publications.clone();
        let palette_cache = trace.pre_dungeon_palette_cache;
        if let Some(target) = target {
            let mut work = self.native_pre_dungeon_animated_graphics.take().unwrap_or_else(|| {
                // This call is after the native room's construction and custom
                // attributes. Retire that prefix before publishing its graphics.
                self.complete_module_pre_dungeon_before_animated_tiles();
                AnimatedDungeonGraphicsContinuation::default()
            });
            let pack = super::misc::DUNG_ANIMATED_TILES[
                self.game_state.world.palette_theme.main_tile_theme_index() as usize];
            self.advance_animated_dungeon_graphics(usize::from(pack), &mut work, target);
            self.native_pre_dungeon_animated_graphics = Some(work);
        }
        self.advance_pre_dungeon_tileset_publication(&tileset_events);
        if let Some(target) = palette_cache {
            self.advance_pre_dungeon_palette_cache(target);
        }
    }

    pub(super) fn advance_native_exact_cpu_pre_overworld_animated_graphics(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_PRE_OVERWORLD_GRAPHICS_LIVE_TRIAL").is_none() {
            return;
        }
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else { return };
        let target = trace.animated_overworld_graphics_progress;
        let tileset_events = trace.pre_overworld_tileset_publications.clone();
        let Some(GameWorkContinuation::PreOverworldPropertiesSpriteReset { animated_tiles, .. }) =
            self.game_execution_scheduler.current_work() else { return };
        if let Some(target) = target {
            let mut progress = self.native_pre_overworld_animated_graphics
                .unwrap_or(AnimatedOverworldGraphicsProgress::FirstSheet(0));
            self.advance_animated_overworld_graphics(animated_tiles, &mut progress, target);
            self.native_pre_overworld_animated_graphics = Some(progress);
        }
        self.advance_pre_overworld_tileset_publication(&tileset_events);
    }

    pub(super) fn advance_native_exact_cpu_overworld_map8(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_MAP8_LIVE_TRIAL").is_none() {
            return;
        }
        let Some(target) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.overworld_map8_progress) else { return };
        let (page, upload_base) = match self.game_execution_scheduler.current_work() {
            Some(GameWorkContinuation::FinishPreOverworldOverlays
                | GameWorkContinuation::FinishWorldMapOverlayReload) =>
                (super::OverworldMap16SourcePage::Overlay, 0x1000),
            Some(GameWorkContinuation::FinishPreOverworldScreenBuild) =>
                (super::OverworldMap16SourcePage::Main, 0),
            _ => return,
        };
        let mut cursor = self.native_overworld_map8.take()
            .unwrap_or_else(|| self.begin_overworld_map8_cursor(page, upload_base));
        self.advance_overworld_map8_cursor(&mut cursor, target);
        self.native_overworld_map8 = Some(cursor);
    }

    pub(super) fn advance_native_exact_cpu_overworld_quadrants(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_QUADRANTS_LIVE_TRIAL").is_none()
            || !matches!(self.game_execution_scheduler.current_work(),
                Some(GameWorkContinuation::FinishOverworldMapQuadrants { .. })) {
            return;
        }
        let Some(target) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.overworld_quadrant_stores) else { return };
        let mut cursor = self.native_overworld_quadrants.take()
            .unwrap_or_else(|| self.begin_overworld_quadrant_cursor());
        self.advance_overworld_quadrant_cursor(&mut cursor, target);
        self.native_overworld_quadrants = Some(cursor);
    }

    pub(super) fn advance_native_exact_cpu_initial_scroll(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_SCROLL_MAP_LIVE_TRIAL").is_none()
            || self.game_execution_scheduler.current_work()
                != Some(GameWorkContinuation::FinishOverworldScreenMapAndSpriteGraphicsTail) {
            return;
        }
        let Some(target) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.initial_scroll_steps) else { return };
        let mut cursor = self.native_initial_scroll.take()
            .unwrap_or_else(|| self.begin_initial_scroll_cursor());
        self.advance_initial_scroll_cursor(&mut cursor, target);
        self.native_initial_scroll = Some(cursor);
    }



    pub(super) fn advance_native_exact_cpu_transition_aux_graphics(&mut self) {
        let Some(target) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(|trace| trace.transition_aux_graphics_cursor)
        else { return; };
        if self.game_execution_scheduler.current_work()
            == Some(GameWorkContinuation::FinishOverworldAuxGraphics) {
            self.advance_transition_aux_graphics(self.native_transition_aux_graphics_progress, target);
            self.native_transition_aux_graphics_progress = Some(target);
            return;
        }
        let Some(GameWorkContinuation::FinishDungeonSupertileTransition { work }) =
            self.game_execution_scheduler.current_work() else { return };
        let straight = match work {
            super::DungeonSupertileTransitionWork::SpiralRoomInitialization => false,
            super::DungeonSupertileTransitionWork::StraightInterroomRoomInitialization => true,
            _ => return,
        };
        let completed = self.native_transition_aux_graphics_progress;
        if completed.is_none() {
            // Entering this shared graphics call proves Dungeon_LoadRoom
            // returned. Retire its native tail before any sheet is published.
            if self.active_early_dungeon_floor_draw.is_none() { return; }
            self.finish_live_dungeon_supertile_room_load();
            if straight {
                self.dungeon_restore_star_tile_chr();
            } else {
                self.ResetStarTileGraphics();
            }
        }
        self.advance_transition_aux_graphics(completed, target);
        self.native_transition_aux_graphics_progress = Some(target);
    }

    pub(super) fn advance_native_exact_cpu_room_upload(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_ROOM_UPLOAD_LIVE_TRIAL").is_none() {
            return;
        }
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else {
            return;
        };
        let host = trace.host;
        let events = trace.room_upload_events.clone();
        for event in events {
            match event {
                NativeExactCpuRoomUploadEvent::Began { .. } => {
                    let post_object_prefix_complete = self.active_early_dungeon_object_draw.is_none();
                    if let Some(mut objects) = self.active_early_dungeon_object_draw.take() {
                        assert!(
                            !self.advance_dungeon_object_draw_one(&mut objects),
                            "source room upload began before translated objects completed"
                        );
                        assert!(objects.finished());
                    } else if self.active_early_dungeon_floor_draw.is_none() {
                        continue;
                    }
                    assert!(self.active_early_dungeon_room_upload.is_none());
                    self.active_early_dungeon_room_upload = Some(
                        if post_object_prefix_complete {
                            self.begin_dungeon_room_upload_after_early_object_prefix()
                        } else {
                            self.begin_dungeon_room_upload_after_early_objects()
                        }
                    );
                    eprintln!(
                        "native-exact-cpu-live-upload host={host} began after_objects={}",
                        self.game_state.dungeon.object_tracking.misc_object_index() >> 1
                    );
                }
                NativeExactCpuRoomUploadEvent::QuadrantPrepared { count, .. } => {
                    let Some(mut work) = self.active_early_dungeon_room_upload.take() else {
                        continue;
                    };
                    self.advance_dungeon_room_upload_one(&mut work);
                    assert_eq!(
                        work.completed_quadrants(),
                        count,
                        "source and translated quadrant upload lost position"
                    );
                    eprintln!(
                        "native-exact-cpu-live-upload host={host} completed_quadrants={count}"
                    );
                    self.active_early_dungeon_room_upload = Some(work);
                }
            }
        }
    }

    pub(super) fn advance_native_exact_cpu_ground_item_receipt_tail(&mut self) {
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_ITEM_RECEIPT_LIVE_TRIAL").is_none() {
            return;
        }
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else {
            return;
        };
        let host = trace.host;
        let writes = trace.item_receipt_step_writes.clone();
        for write in writes {
            let Some(GameWorkContinuation::FinishItemReceiptGraphics {
                continuation:
                    ItemReceiptGraphicsContinuation::CallerAlreadyCompleted {
                        ground_apress_tail: Some(receipt),
                        ..
                    },
            }) = self.game_execution_scheduler.current_work()
            else {
                continue;
            };
            assert_eq!(receipt.ancilla_slot, write.slot);
            assert!(self.early_ground_item_receipt_tail.is_none());
            self.complete_ancilla_add_item_receipt(receipt);
            assert_eq!(
                self.ancilla_slot_view(usize::from(write.slot)).step(),
                write.value,
                "translated receipt tail disagrees with source step write"
            );
            self.early_ground_item_receipt_tail = Some(receipt);
            eprintln!(
                "native-exact-cpu-live-item-receipt host={host} slot={} item={:02x}",
                write.slot, receipt.item
            );
        }
    }

    pub(super) fn complete_ground_item_receipt_tail_once(&mut self, receipt: ItemReceiptReturn) {
        if let Some(early) = self.early_ground_item_receipt_tail.take() {
            assert_eq!(
                early, receipt,
                "early ground-item receipt belongs to another caller"
            );
        } else {
            self.complete_ancilla_add_item_receipt(receipt);
        }
    }

    pub(super) fn begin_native_exact_cpu_overworld_live_scan(
        &mut self,
        sprite_records: usize,
    ) -> (OverworldSpriteReloadWorkload, SpriteSlotsState) {
        assert!(self.native_exact_cpu_overworld_live_scan.is_none());
        let owner = self.native_exact_cpu_owner.take();
        let mut preview = self.clone();
        self.native_exact_cpu_owner = owner;
        let work = preview.begin_overworld_sprite_reload_scan_from_presence(sprite_records);
        let workload = preview.complete_overworld_sprite_reload_scan(work);
        let final_slots = preview.game_state.sprites.sprite_slots.clone();
        self.native_exact_cpu_overworld_live_scan = Some(NativeExactCpuOverworldLiveScan {
            sprite_records,
            work: None,
            began: false,
            expected_final_slots: final_slots.clone(),
        });
        (workload, final_slots)
    }

    pub(super) fn advance_native_exact_cpu_overworld_live_scan(&mut self) {
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else {
            return;
        };
        if trace.overworld_scan_events.is_empty() {
            return;
        }
        let host = trace.host;
        let events = trace.overworld_scan_events.clone();
        let Some(mut live) = self.native_exact_cpu_overworld_live_scan.take() else {
            return;
        };
        let mut finished = false;
        let mut completed_cells = live.work.as_ref().map_or(0, |work| work.cells_completed());
        for event in &events {
            match *event {
                NativeExactCpuOverworldScanEvent::Began { .. } => {
                    assert!(!live.began, "live overworld scan began twice");
                    live.began = true;
                    live.work = Some(
                        self.begin_overworld_sprite_reload_scan_from_presence(live.sprite_records),
                    );
                }
                NativeExactCpuOverworldScanEvent::CellReturned { ordinal, .. } => {
                    assert!(live.began, "live overworld scan cell returned before begin");
                    self.advance_overworld_sprite_reload_scan_through_cell(
                        live.work.as_mut().expect("begun scan has a cursor"),
                        ordinal,
                    );
                    completed_cells = ordinal;
                }
                NativeExactCpuOverworldScanEvent::Finished { cells, .. } => {
                    assert!(live.began, "live overworld scan finished before begin");
                    let work = live.work.take().expect("returning scan lost its cursor");
                    assert_eq!(cells, work.total_cells());
                    assert_eq!(cells, work.cells_completed());
                    self.finish_overworld_sprite_reload_scan(work);
                    completed_cells = cells;
                    assert_eq!(
                        self.game_state.sprites.sprite_slots, live.expected_final_slots,
                        "CPU-driven live scan built different sprite slots"
                    );
                    finished = true;
                }
            }
        }
        eprintln!(
            "native-exact-cpu-live-scan host={} cells={} finished={} scroll_delta={:02x}",
            host,
            completed_cells,
            finished,
            self.overworld_horizontal_scroll_delta_low(),
        );
        if !finished {
            self.native_exact_cpu_overworld_live_scan = Some(live);
        } else if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_SPRITE_TIMERS_LIVE_TRIAL").is_some() {
            // A completed scan alone proves only its inner return. The
            // TimersAndOam coordinate call additionally proves that Module09
            // returned through its reload suffix and entered Sprite_Main.
            if let Some((slot, next_instruction, boundary)) = self.native_exact_cpu_host_trace.as_ref()
                .and_then(|trace| trace.sprite_coordinate_prefix) {
                self.refine_module09_reload_to_sprite_main(boundary);
                self.refine_sprite_main_to_coordinate_moves(slot, next_instruction);
            }
        }
    }

    pub(super) fn begin_native_exact_cpu_overworld_scan_trial(
        &mut self,
        work: OverworldSpriteReloadScanWork,
    ) {
        assert!(self.native_exact_cpu_overworld_scan_trial.is_none());
        // The retained CPU owns a second machine and cartridge image. The
        // translated shadow only needs game state, so move that owner around
        // this one-time clone rather than duplicating it.
        let owner = self.native_exact_cpu_owner.take();
        let mut shadow = Box::new(self.clone());
        self.native_exact_cpu_owner = owner;
        shadow.native_exact_cpu_host_trace = None;
        self.native_exact_cpu_overworld_scan_trial =
            Some(Box::new(NativeExactCpuOverworldScanTrial {
                shadow,
                work,
                began: false,
                expected_final_ram: None,
                expected_final_game_state: None,
            }));
    }

    pub(super) fn record_native_exact_cpu_overworld_scan_expected_state(&mut self) {
        let image = self.native_exact_cpu_overworld_scan_trial.as_ref()
            .map(|_| self.wram_image().into_owned());
        if let Some(trial) = self.native_exact_cpu_overworld_scan_trial.as_mut() {
            trial.expected_final_ram = image;
            trial.expected_final_game_state = Some(Box::new(self.game_state.clone()));
        }
    }

    pub(super) fn advance_native_exact_cpu_overworld_scan_trial(&mut self) {
        let Some(trace) = self.native_exact_cpu_host_trace.as_ref() else {
            return;
        };
        if trace.overworld_scan_events.is_empty() {
            return;
        }
        let Some(mut trial) = self.native_exact_cpu_overworld_scan_trial.take() else {
            return;
        };
        let mut finished = false;
        for event in &trace.overworld_scan_events {
            match *event {
                NativeExactCpuOverworldScanEvent::Began { .. } => {
                    assert!(!trial.began, "translated scan began twice");
                    trial.began = true;
                }
                NativeExactCpuOverworldScanEvent::CellReturned { ordinal, .. } => {
                    assert!(trial.began, "translated scan cell returned before begin");
                    trial
                        .shadow
                        .advance_overworld_sprite_reload_scan_through_cell(
                            &mut trial.work,
                            ordinal,
                        );
                }
                NativeExactCpuOverworldScanEvent::Finished { cells, .. } => {
                    assert!(trial.began, "translated scan finished before begin");
                    assert_eq!(cells, trial.work.total_cells());
                    assert_eq!(cells, trial.work.cells_completed());
                    trial.shadow.finish_overworld_sprite_reload_scan(trial.work);
                    let expected_ram = trial
                        .expected_final_ram
                        .as_ref()
                        .expect("eager RAM missing");
                    let expected_game_state = trial
                        .expected_final_game_state
                        .as_ref()
                        .expect("eager game state missing");
                    assert_eq!(
                        &trial.shadow.ram, expected_ram,
                        "source-driven translated scan built different RAM"
                    );
                    assert_eq!(
                        &trial.shadow.game_state,
                        expected_game_state.as_ref(),
                        "source-driven translated scan built different game state"
                    );
                    finished = true;
                }
            }
        }
        let outstanding_ram = trial.expected_final_ram.as_ref().map(|expected| {
            trial
                .shadow
                .ram
                .iter()
                .zip(expected)
                .enumerate()
                .filter_map(|(offset, (partial, final_value))| {
                    (partial != final_value).then_some((offset, *partial, *final_value))
                })
                .collect::<Vec<_>>()
        });
        eprintln!(
            "native-exact-cpu-scan-trial host={} cells={} finished={} shadow_scroll_delta={:02x} outstanding_ram_count={} outstanding_ram_prefix={:?}",
            trace.host,
            trial.work.cells_completed(),
            finished,
            trial.shadow.overworld_horizontal_scroll_delta_low(),
            outstanding_ram.as_ref().map_or(0, Vec::len),
            outstanding_ram.as_ref().map(|diff| &diff[..diff.len().min(32)]),
        );
        if !finished {
            self.native_exact_cpu_overworld_scan_trial = Some(trial);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
struct NativeExactCpuOverworldScanTracker {
    active: bool,
    cells: u16,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
struct NativeExactCpuSpritePreparationTracker {
    group_start: Option<u8>,
    progress: Option<ExtendedOamPackingProgress>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct NativeExactCpuScrollCopyProgress {
    pub completed_passes: u8,
    pub stores_in_pass: u16,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
struct NativeExactCpuScrollCopyTracker {
    active: bool,
    progress: NativeExactCpuScrollCopyProgress,
}

impl NativeExactCpuScrollCopyTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt, trace: &mut NativeExactCpuHostTrace) {
        if step.origin_pc == 0x0e_cfe2 {
            assert!(!self.active, "source scroll copy entered twice without returning");
            self.active = true;
            self.progress = NativeExactCpuScrollCopyProgress::default();
        }
        if !self.active {
            return;
        }
        for access in &step.accesses {
            if (0x0e_cffd..=0x0e_d079).contains(&step.origin_pc)
                && (0x7f_0000..0x7f_07e0).contains(&access.address)
                && matches!(access.kind, SourceCpuBusAccessKind::Write { width: 2, .. })
            {
                self.progress.stores_in_pass += 1;
                assert!(
                    self.progress.stores_in_pass <= 126 * 8 + 21,
                    "source scroll copy exceeded one pass of word stores",
                );
            }
            if step.origin_pc == 0x0e_d084
                && native_exact_cpu_wram_key(access.address) == Some(0x1cdf)
                && matches!(access.kind, SourceCpuBusAccessKind::Write { width: 1, .. })
            {
                assert_eq!(
                    self.progress.stores_in_pass,
                    126 * 8 + 21,
                    "source scroll pass completed before every word store",
                );
                self.progress.completed_passes += 1;
                self.progress.stores_in_pass = 0;
            }
        }
        if step.origin_pc == 0x0e_d0c2 {
            assert_eq!(step.opcode, 0x60, "source scroll return is no longer RTS");
            trace.scroll_copy_returned = true;
            self.active = false;
        }
    }
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
struct NativeExactCpuFloorDrawTracker {
    active: bool,
    completed_words: u16,
    floor_complete: bool,
    object_in_progress: bool,
    completed_objects: u16,
}

impl NativeExactCpuFloorDrawTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt, trace: &mut NativeExactCpuHostTrace) {
        if !trace.track_floor_tile_writes {
            return;
        }
        if step.origin_pc == 0x01_89dc {
            assert!(
                !self.active,
                "source floor draw began before its prior pass returned"
            );
            self.active = true;
            self.completed_words = 0;
            self.floor_complete = false;
            self.object_in_progress = false;
            self.completed_objects = 0;
        }
        if self.floor_complete {
            if step.origin_pc == 0x01_893c || step.origin_pc == 0x01_8916 {
                self.object_in_progress = true;
            } else if self.object_in_progress
                && (step.origin_pc == 0x01_88e4 || step.origin_pc == 0x01_8910)
            {
                self.completed_objects += 1;
                self.object_in_progress = false;
                trace
                    .object_completions
                    .push(NativeExactCpuObjectCompletion {
                        ordinal: self.completed_objects,
                        at: step.started_at.master_cycles(),
                    });
            }
        }
        if !self.active || !(0x01_8a44..=0x01_8a7d).contains(&step.origin_pc) {
            return;
        }
        for access in &step.accesses {
            if !(0x7e_2000..0x7e_6000).contains(&access.address) {
                continue;
            }
            if let SourceCpuBusAccessKind::Write { value, width: 2 } = access.kind {
                self.completed_words += 1;
                trace.floor_tile_writes.push(NativeExactCpuFloorTileWrite {
                    ordinal: self.completed_words,
                    address: access.address,
                    value,
                    at: access.timestamp.master_cycles(),
                });
                if self.completed_words == 8192 {
                    self.active = false;
                    self.floor_complete = true;
                    break;
                }
            }
        }
    }
}

impl NativeExactCpuSpritePreparationTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt) {
        if step.origin_pc == 0x00_85fc {
            self.group_start = Some(32);
            self.progress = None;
        }
        if step.origin_pc == 0x00_85fe {
            let group_start = self
                .group_start
                .expect("sprite preparation packing entered without its caller")
                .checked_sub(4)
                .expect("sprite preparation exceeded eight packing groups");
            self.group_start = Some(group_start);
            self.progress = Some(ExtendedOamPackingProgress::before_group(group_start));
        }
        if (0x00_85fe..=0x00_865a).contains(&step.origin_pc) {
            let progress = self
                .progress
                .as_mut()
                .expect("packing instruction lost its group");
            let instruction_end = match step.accepted_interrupt {
                Some(SourceCpuAcceptedInterrupt::Nmi { started_at, .. })
                | Some(SourceCpuAcceptedInterrupt::Irq { started_at, .. }) => {
                    started_at.master_cycles()
                }
                None => step.ended_at.master_cycles(),
            };
            // Physical elapsed time may include WRAM refresh while the
            // translated packing ledger counts the instruction's charged
            // master cycles. An interrupt's entry transactions belong to
            // the handler, not to this interrupted packing group.
            let instruction_cycles: u16 = step
                .transactions
                .iter()
                .filter(|transaction| transaction.started_at.master_cycles() < instruction_end)
                .map(|transaction| u16::from(transaction.duration_master_cycles))
                .sum();
            progress.group_master_cycles += instruction_cycles;
            progress.completed_bytes += step
                .accesses
                .iter()
                .filter(|access| {
                    (0x0a00..0x0a20).contains(&(access.address as u16))
                        && matches!(access.kind, SourceCpuBusAccessKind::Write { width: 1, .. })
                })
                .count() as u8;
        }
        if step.origin_pc == 0x00_865c {
            self.group_start = None;
            self.progress = None;
        }
    }
}

impl NativeExactCpuOverworldScanTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(
        &mut self,
        step: &SourceCpuStepReceipt,
        events: &mut Vec<NativeExactCpuOverworldScanEvent>,
    ) {
        // $09:C56A installs the scan's temporary horizontal delta. The JSR
        // $C6F5 in its inner loop returns at $09:C5E6 once per candidate
        // cell. $09:C585 restores the caller's delta after the outer loop.
        // These are instruction boundaries, independent of host/frame counts.
        if self.active && step.origin_pc == 0x09_c5e6 {
            self.cells = self
                .cells
                .checked_add(1)
                .expect("overworld scan cell count fits u16");
            events.push(NativeExactCpuOverworldScanEvent::CellReturned {
                at: step.started_at.master_cycles(),
                ordinal: self.cells,
            });
        }
        if !matches!(step.origin_pc, 0x09_c56a | 0x09_c585) {
            return;
        }
        let write = step.accesses.iter().find_map(|access| {
            if access.address & 0xffff != 0x069f {
                return None;
            }
            match access.kind {
                SourceCpuBusAccessKind::Write { value, width: 1 } => {
                    Some((value as u8, access.timestamp.master_cycles()))
                }
                _ => None,
            }
        });
        match (step.origin_pc, write) {
            (0x09_c56a, Some((0xff, at))) => {
                assert!(
                    !self.active,
                    "source overworld scan began twice without returning"
                );
                self.active = true;
                self.cells = 0;
                events.push(NativeExactCpuOverworldScanEvent::Began { at });
            }
            (0x09_c585, Some((_, at))) => {
                assert!(
                    self.active,
                    "source overworld scan returned without beginning"
                );
                events.push(NativeExactCpuOverworldScanEvent::Finished {
                    at,
                    cells: self.cells,
                });
                self.active = false;
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Default)]
// Shared timing vocabulary is inspected by ungated gameplay accessors.
#[cfg_attr(not(feature = "rom-timing"), allow(dead_code))]
pub(super) struct NativeExactCpuHostTrace {
    pub host: u32,
    /// Retained CPU PC at the host boundary, even when no interrupt occurred.
    pub return_pc: u32,
    pub nmis: Vec<NativeExactCpuNmi>,
    pub nmi_completions: Vec<NativeExactCpuNmiCompletion>,
    /// NMIs accepted by this CPU owner whose handler has not returned at the
    /// host boundary. The same acceptance appears in a later completion.
    pub pending_nmis_at_return: Vec<NativeExactCpuNmi>,
    pub ppu_reads: Vec<NativeExactCpuPpuAccess>,
    pub nmi_gate_accesses: Vec<NativeExactCpuNmiGateAccess>,
    pub nmi_update_decisions: Vec<NativeExactCpuNmiUpdateDecision>,
    pub story_decompression_writes: Vec<NativeExactCpuWrite>,
    pub dialogue_decode_cursor_writes: Vec<NativeExactCpuWrite>,
    pub track_story_decompression_writes: bool,
    pub track_dialogue_decode_cursor_writes: bool,
    pub track_vwf_cpu_work: bool,
    pub vwf_cpu_work_master_cycles: u64,
    pub vwf_work_since_nmi: u64,
    pub vwf_first_step_at: Option<u64>,
    pub vwf_last_step_at: Option<u64>,
    pub vwf_cursor_work: Vec<(u16, u64)>,
    pub vwf_draw_entry_work: Vec<u64>,
    pub nmi_cpu_work_by_block: [u64; 4],
    pub main_wait_returns: Vec<NativeExactCpuMainWaitReturn>,
    /// The room-loader JSR has returned to Module07's next instruction.
    pub room_load_returned: bool,
    /// The auxiliary sprite graphics JSR has returned to Module07.
    pub auxiliary_sprite_graphics_returned: bool,
    /// Module 7 returned from LinkOam_Main into Hud_RefillLogic.
    pub module07_link_oam_returned: bool,
    /// Return timestamps associate caller progress with its main-loop entry,
    /// rather than letting an earlier iteration retire a later caller.
    pub module07_link_oam_returns: Vec<u64>,
    /// Source-owned cached sprite statement at this host return, if the
    /// retained CPU stopped inside a resumable translated handler.
    pub cached_sprite_progress: Option<crate::CachedSpriteExecutionProgressReceipt>,
    /// The cached loop slot sampled when an NMI interrupted its DEX.
    pub cached_slot_returned_at_nmi: Option<u8>,
    pub dungeon_reset_progress: Option<DungeonResetSpritesProgressReceipt>,
    pub dungeon_load_boundary: Option<(u8, crate::DungeonSpriteLoadCheckpoint, OriginalTimingBoundary)>,
    pub main_loop_entries: Vec<NativeExactCpuMainLoopEntry>,
    pub sprite_preparation_interruptions: Vec<NativeExactCpuSpritePreparationInterruption>,
    /// Source CPU stopped inside extended-OAM packing at the host return.
    /// The translated owner uses this cursor to run its own packing prefix.
    pub sprite_preparation_progress_at_return: Option<ExtendedOamPackingProgress>,
    /// Stores completed by the resident RenderText_Draw_Scroll call. This is
    /// a source cursor, never source-authored render-buffer data.
    pub scroll_copy_progress: Option<NativeExactCpuScrollCopyProgress>,
    pub scroll_copy_returned: bool,
    pub dungeon_header_index_writes: Vec<NativeExactCpuDungeonHeaderIndexWrite>,
    pub floor_tile_writes: Vec<NativeExactCpuFloorTileWrite>,
    pub object_completions: Vec<NativeExactCpuObjectCompletion>,
    pub object_4x2_boundary: Option<(u16, super::dungeon::Dungeon4x2DrawBoundary)>,
    /// The source reached an active handler's register-only entry prologue;
    /// its native timer/OAM prefix has completed, but its handler has not.
    pub sprite_timer_return_slot: Option<u8>,
    /// Cursor in the timer helper's coordinate moves; values remain native.
    pub sprite_coordinate_prefix: Option<(u8, u8, OriginalTimingBoundary)>,
    /// Shared coordinate helper called KillSelf for an active guard and has
    /// stopped before the callee's first possible persistent mutation.
    pub guard_offscreen_kill_self_slot: Option<u8>,
    /// Guard_Main returned from drawing and passed ReturnIfInactive's state
    /// test. Its native pose restore precedes the still-pending pause test.
    pub guard_active_state_checked_slot: Option<u8>,
    /// Rat graphics are stored; its flags store and drawing remain pending.
    pub rat_oam_flags_pending_slot: Option<u8>,
    /// Keese's draw returned into its own call to the shared inactive check.
    pub keese_inactive_check_slot: Option<u8>,
    /// LinkOam published its native graphics index; body OAM is still pending.
    pub link_oam_body_selection: bool,
    pub link_oam_equipment_selection: bool,
    pub link_oam_lower_body_progress: Option<player_oam::LinkOamLowerBodyProgress>,
    /// Dungeon_LoadRoom stored the completed object count as the torch-range
    /// start before entering its separate torch stream.
    pub room_post_object_prefix_value: Option<u16>,
    /// Source CPU cursor inside Zelda's shared follower decompressor. The
    /// translated Sprite_Main prefix and sheet bytes remain native-owned.
    pub zelda_follower_graphics_progress: Option<(u8, crate::RescuedMaidenInitializationStage)>,
    pub follower_body_draw_prefix: Option<(SpriteMainCpuBoundary, OriginalTimingBoundary)>,
    pub quadrant_copy_boundary: Option<(u16, OriginalTimingBoundary)>,
    /// Last byte committed by the shared sprite decompressor in this host.
    /// The destination selects a scratch sheet; only native asset data is
    /// written when a translated caller advances to this cursor.
    pub sprite_decompression_progress: Option<(bool, u16, u8)>,
    /// Last `LoadTransAuxGFX` decompressor store into $7E:6000..8FFF in this
    /// host: its WRAM offset and the source value, a witness only.
    pub aux_graphics_decompression_store: Option<(usize, u8)>,
    pub animated_overworld_graphics_progress: Option<AnimatedOverworldGraphicsProgress>,
    pub animated_dungeon_graphics_progress: Option<AnimatedDungeonGraphicsProgress>,
    pub pre_overworld_tileset_publications: Vec<TilesetGraphicsPublication>,
    pub pre_dungeon_tileset_publications: Vec<TilesetGraphicsPublication>,
    pub pre_dungeon_palette_cache: Option<u16>,
    pub overworld_map8_progress: Option<u16>,
    pub overworld_quadrant_stores: Option<u16>,
    pub initial_scroll_steps: Option<u16>,
    pub transition_aux_graphics_cursor: Option<super::load_gfx::TransitionAuxGraphicsProgress>,
    pub room_upload_events: Vec<NativeExactCpuRoomUploadEvent>,
    pub item_receipt_step_writes: Vec<NativeExactCpuItemReceiptStepWrite>,
    pub spotlight_events: Vec<NativeExactCpuSpotlightEvent>,
    /// Source-ordered reset-table store positions; values are witnesses and
    /// never copied into translated table memory.
    pub spotlight_reset_stores: Vec<u16>,
    /// Bus-order witnesses for the NMI music command and its last-command
    /// mirror. The translated audio owner must produce its own command.
    pub music_port_accesses: Vec<NativeExactCpuMusicAccess>,
    pub track_floor_tile_writes: bool,
    pub overworld_scan_events: Vec<NativeExactCpuOverworldScanEvent>,
    pub wram_watch_address: Option<u16>,
    pub wram_watch_accesses: Vec<NativeExactCpuWramWatchAccess>,
}

impl NativeExactCpuHostTrace {
    #[cfg(feature = "rom-timing")]
    fn observe_uncle_priest_dispatch(&mut self, step: &SourceCpuStepReceipt, slot: u8) {
        match step.origin_pc {
            // The concrete type-$73 selector calls JumpTableLocal after the
            // shared timers/OAM prefix. Its body has no persistent writes.
            0x05_db91 => {
                assert_eq!(step.opcode, 0x22);
                assert!(slot < 16);
                self.sprite_timer_return_slot = Some(slot);
            }
            // JMP [$0000] enters the selected body; from here timer return
            // alone no longer describes the complete native prefix.
            0x00_8799 => self.sprite_timer_return_slot = None,
            _ => {}
        }
    }

    pub(crate) fn dialogue_decode_cursor_at_return(&self) -> Option<u16> {
        self.dialogue_decode_cursor_writes.last()
            .filter(|write| write.width == 2 && write.value != 0)
            .map(|write| write.value)
    }

    pub(super) fn sprite_preparation_progress_at_host_boundary(&self) -> Option<ExtendedOamPackingProgress> {
        self.sprite_preparation_progress_at_return.or_else(|| {
            self.sprite_preparation_interruptions.iter().rev().find(|interruption| {
                self.pending_nmis_at_return.iter().any(|nmi| {
                    nmi.accepted_at == interruption.accepted_at
                        && nmi.interrupted_pc == interruption.interrupted_pc
                })
            }).map(|interruption| interruption.progress)
        })
    }

    /// The source accepted this NMI in this host but left its handler on the
    /// CPU stack at the host boundary.
    pub(super) fn accepted_nmi_carries_out(&self) -> bool {
        self.pending_nmis_at_return
            .iter()
            .any(|accepted| self.nmis.contains(accepted))
    }

    /// The only handler completes while the saved caller is still held,
    /// before its main-wait return, with no subsequent acceptance or fresh
    /// iteration. Only the opt-in CPU owner supplies this lifecycle witness.
    pub(super) fn caller_returns_after_held_nmi(&self) -> bool {
        let [completion] = self.nmi_completions.as_slice() else { return false };
        self.main_loop_entries.is_empty()
            && self.main_wait_returns.first().is_some_and(|wait| completion.returned_at < wait.at)
            && self.nmis.iter().all(|nmi| nmi.accepted_at < completion.returned_at)
            && self.nmi_update_decisions.iter().any(|decision|
                decision.accepted_at == completion.accepted_at && !decision.runs_updates)
    }

    /// A handler returns before this host enters its main-loop iteration.
    /// Its publication precedes native main work whether its acceptance was
    /// carried in or occurred at the beginning of this host interval.
    pub(super) fn leading_nmi_completes_before_main(&self) -> bool {
        let Some(main_entry) = self.main_loop_entries.first() else {
            return false;
        };
        self.nmi_completions.iter().any(|completion| {
            completion.returned_at < main_entry.at
        })
    }

    pub(super) fn module07_caller_passed_link_without_nmi(&self, frame_counter: u8) -> bool {
        self.main_loop_entries.iter().enumerate().any(|(index, entry)| {
            let in_iteration = |at: u64| at > entry.at
                && self.main_loop_entries.get(index + 1).is_none_or(|next| at < next.at);
            entry.frame_counter == frame_counter
                && self.module07_link_oam_returns.iter().copied().any(in_iteration)
                && !self.nmis.iter().any(|nmi| in_iteration(nmi.accepted_at))
        })
    }

}

impl NativeExactCpuHostTrace {
    fn spotlight_build_progress(
        &self,
        cursors_after_rows: impl FnOnce(u16) -> (u16, u16),
    ) -> Option<crate::SpotlightTableBuildProgress> {
        let stop_pc = self
            .pending_nmis_at_return
            .iter()
            .find(|nmi| {
                self.nmis.contains(nmi)
                    && ((0x00_f4cc..=0x00_f53d).contains(&nmi.interrupted_pc)
                        || (0x00_f361..=0x00_f3c5).contains(&nmi.interrupted_pc))
            })
            .map(|nmi| nmi.interrupted_pc)
            .unwrap_or(self.return_pc);
        if !(0x00_f361..=0x00_f3c5).contains(&stop_pc)
            && !(0x00_f4cc..=0x00_f53d).contains(&stop_pc)
        {
            return None;
        }
        let start = self.spotlight_events.iter().rposition(|event| {
            matches!(
                event,
                NativeExactCpuSpotlightEvent::WindowYInitialized { .. }
            )
        })?;
        let active = &self.spotlight_events[start + 1..];
        let mut completed_iterations = u16::try_from(
            active
                .iter()
                .filter(|event| matches!(event, NativeExactCpuSpotlightEvent::RowAdvanced { .. }))
                .count(),
        )
        .expect("spotlight builder row count exceeded its native cursor width");
        let checkpoint = match (stop_pc, active.last().copied()) {
            (
                0x00_f375 | 0x00_f4cc..=0x00_f53d,
                Some(NativeExactCpuSpotlightEvent::CircleInputRead { value, .. }),
            ) => crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                pending_circle_input: u8::try_from(value)
                    .expect("spotlight circle input exceeded its byte-sized native operand"),
            },
            (0x00_f361..=0x00_f36c, Some(NativeExactCpuSpotlightEvent::RowAdvanced { .. })) => {
                crate::SpotlightTableBuildCheckpoint::BeforeIterationInitialization
            }
            (0x00_f387..=0x00_f38e, _) => {
                crate::SpotlightTableBuildCheckpoint::AfterUpperTableWrite {
                    lower_cursor: cursors_after_rows(completed_iterations).1,
                }
            }
            (0x00_f392, _) => crate::SpotlightTableBuildCheckpoint::BeforeLowerTableStore {
                lower_cursor: cursors_after_rows(completed_iterations).1,
            },
            (0x00_f39e, Some(NativeExactCpuSpotlightEvent::RowAdvanced { .. })) => {
                // INC $04 retired, but DEC $06 has not. Reconstruct the
                // previous row, then retain the half-completed cursor update.
                completed_iterations = completed_iterations.checked_sub(1)?;
                let (upper, lower) = cursors_after_rows(completed_iterations);
                crate::SpotlightTableBuildCheckpoint::BeforeLowerCursorDecrement {
                    upper_cursor: upper.wrapping_add(1), lower_cursor: lower,
                }
            }
            (0x00_f3a3..=0x00_f3c5, _) => {
                // The final row exits without INC $04. Its words are complete
                // before the V-counter wait and the separate HDMA projection.
                completed_iterations += 1;
                let copied_words = active.iter().filter(|event|
                    matches!(event, NativeExactCpuSpotlightEvent::ProjectionWordStored { .. })
                ).count();
                assert!(copied_words <= 224);
                crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words: copied_words as u16 }
            }
            _ => return None,
        };
        Some(crate::SpotlightTableBuildProgress {
            completed_iterations,
            checkpoint,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(feature = "rom-timing")]
struct NativeExactCpuRead {
    pc: u32,
    address: u32,
    value: u16,
    width: u8,
    at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct NativeExactCpuWrite {
    pc: u32,
    address: u32,
    value: u16,
    width: u8,
    at: u64,
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
struct NativeExactCpuWriteWitness {
    host: u32,
    write: NativeExactCpuWrite,
}

#[cfg(feature = "rom-timing")]
fn native_exact_cpu_wram_key(address: u32) -> Option<u32> {
    let bank = address >> 16;
    let offset = address & 0xffff;
    match bank {
        0x7e => Some(offset),
        0x7f => Some(0x10000 | offset),
        0x00..=0x3f | 0x80..=0xbf if offset < 0x2000 => Some(offset),
        _ => None,
    }
}

#[cfg(feature = "rom-timing")]
fn native_exact_cpu_last_writer(
    read: NativeExactCpuRead,
    current: &[NativeExactCpuWrite],
    prior: &HashMap<u32, NativeExactCpuWriteWitness>,
    host: u32,
) -> Option<NativeExactCpuWriteWitness> {
    let key = native_exact_cpu_wram_key(read.address)?;
    current
        .iter()
        .rev()
        .find(|write| write.at < read.at && (0..write.width).any(|byte| {
            native_exact_cpu_wram_key(write.address + u32::from(byte)) == Some(key)
        }))
        .copied()
        .map(|write| NativeExactCpuWriteWitness { host, write })
        .or_else(|| prior.get(&key).copied())
}

/// A word read can differ only in its high byte. Attribute ownership to the
/// byte that actually differs, rather than an unrelated low-byte writer.
#[cfg(feature = "rom-timing")]
fn native_exact_cpu_differing_read_bytes(
    source: NativeExactCpuRead,
    trial: NativeExactCpuRead,
) -> (NativeExactCpuRead, NativeExactCpuRead) {
    if source.address != trial.address || source.width != trial.width {
        return (source, trial);
    }
    let Some(byte) = (0..source.width).find(|byte|
        (source.value >> (byte * 8)) as u8 != (trial.value >> (byte * 8)) as u8
    ) else { return (source, trial) };
    let select = |read: NativeExactCpuRead| NativeExactCpuRead {
        address: read.address + u32::from(byte),
        value: (read.value >> (byte * 8)) & 0xff,
        width: 1,
        ..read
    };
    (select(source), select(trial))
}

#[cfg(feature = "rom-timing")]
fn native_exact_cpu_record_writers(
    writers: &mut HashMap<u32, NativeExactCpuWriteWitness>,
    writes: &[NativeExactCpuWrite],
    host: u32,
) {
    for &write in writes {
        for byte in 0..write.width {
            if let Some(key) = native_exact_cpu_wram_key(write.address + u32::from(byte)) {
                writers.insert(key, NativeExactCpuWriteWitness { host, write });
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(feature = "rom-timing")]
struct NativeExactCpuInstruction {
    pc: u32,
    opcode: u8,
    started_at: u64,
    ended_at: u64,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
struct NativeExactCpuCachedSpriteTracker {
    slot: Option<u8>,
    recruit_draw_entered: bool,
    timer_oam_active: bool,
}

/// Source CPU cache stores identify statements for the translated reset;
/// their values never enter native state.
#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
struct NativeExactCpuDungeonCacheTracker {
    last: Option<(u8, CachedSpriteCacheField)>,
    next_field: usize,
}

impl NativeExactCpuDungeonCacheTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt, cpu: &Snes9xColdCpuExecutor) {
        if step.origin_pc == 0x09_c244 {
            *self = Self::default();
            return;
        }
        if !(0x09_c176..0x09_c244).contains(&step.origin_pc) {
            return;
        }
        for access in &step.accesses {
            let SourceCpuBusAccessKind::Write { width: 1, .. } = access.kind else {
                continue;
            };
            let slot = cpu.machine().snes().cpu.x as u8;
            if slot >= 16 {
                continue;
            }
            self.observe_store(slot, access.address as u16);
        }
    }

    #[cfg(feature = "rom-timing")]
    fn observe_store(&mut self, slot: u8, address: u16) {
        let is_cache_field = CachedSpriteCacheField::C_SOURCE_ORDER
            .iter()
            .any(|field| address == field.alt_address() as u16 + u16::from(slot));
        if !is_cache_field {
            return;
        }
        if self.last.is_none_or(|(previous, _)| previous != slot) {
            if let Some((previous, _)) = self.last {
                assert!(
                    slot < previous || slot == 15,
                    "cache loop changed slot order"
                );
            }
            self.next_field = 0;
        }
        let field = CachedSpriteCacheField::C_SOURCE_ORDER
            .get(self.next_field)
            .copied()
            .expect("cache wrote past the last source field");
        assert_eq!(
            address,
            field.alt_address() as u16 + u16::from(slot),
            "cache store skipped or reordered a source field",
        );
        self.last = Some((slot, field));
        self.next_field += 1;
    }

    #[cfg(feature = "rom-timing")]
    fn progress_at_return(
        self,
        pc: u32,
        boundary: OriginalTimingBoundary,
    ) -> Option<DungeonResetSpritesProgressReceipt> {
        if !(0x09_c176..0x09_c244).contains(&pc) {
            return None;
        }
        let (slot, field) = self.last?;
        Some(DungeonResetSpritesProgressReceipt {
            progress: DungeonResetSpritesCpuProgress::Cache { slot, field },
            boundary,
        })
    }
}

impl NativeExactCpuCachedSpriteTracker {
    /// The straight-line PLA/STA tail restores the native live-slot backup in
    /// reverse field order. Count only stores before the interrupted opcode;
    /// a PLA alone does not publish its field. This cursor needs no saved CPU
    /// values or extra checkpoint payload, including across a carried NMI.
    #[cfg(feature = "rom-timing")]
    fn restore_live_fields_at_pc(pc: u32) -> Option<u8> {
        const STORES: [u32; 24] = [
            0x1d_eb07, 0x1d_eb0b, 0x1d_eb10, 0x1d_eb14,
            0x1d_eb18, 0x1d_eb1c, 0x1d_eb20, 0x1d_eb24,
            0x1d_eb28, 0x1d_eb2c, 0x1d_eb30, 0x1d_eb34,
            0x1d_eb38, 0x1d_eb3c, 0x1d_eb40, 0x1d_eb44,
            0x1d_eb48, 0x1d_eb4c, 0x1d_eb50, 0x1d_eb54,
            0x1d_eb58, 0x1d_eb5c, 0x1d_eb60, 0x1d_eb64,
        ];
        (0x1d_eb06..=0x1d_eb67).contains(&pc)
            .then(|| (STORES.len() - STORES.partition_point(|store| *store < pc)) as u8)
    }

    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt, cpu: &Snes9xColdCpuExecutor) {
        match step.origin_pc {
            // UncacheAndExecuteSprite retains X as its live slot while it
            // swaps 24 fields and calls Sprite_ExecuteSingle_.
            0x1d_ea00 => {
                let slot = cpu.machine().snes().cpu.x as u8;
                assert!(slot < 16, "cached sprite entered with an invalid slot");
                self.slot = Some(slot);
                self.recruit_draw_entered = false;
                self.timer_oam_active = false;
            }
            0x06_83f2 if self.slot.is_some() => self.timer_oam_active = true,
            0x06_84eb => self.timer_oam_active = false,
            0x05_bd7e if self.slot.is_some() => {
                self.recruit_draw_entered = true;
            }
            0x1d_eb67 | 0x1d_e9f8 => *self = Self::default(),
            _ => {}
        }
    }

    #[cfg(feature = "rom-timing")]
    fn progress_at_return(
        self,
        pc: u32,
        cpu: &Snes9xColdCpuExecutor,
    ) -> Option<crate::CachedSpriteExecutionProgress> {
        let slot = self.slot?;
        if let Some(live_fields) = Self::restore_live_fields_at_pc(pc) {
            return Some(crate::CachedSpriteExecutionProgress::Restoring { slot, live_fields });
        }
        if pc == 0x06_dc7b {
            let ram = &cpu.machine().snes().ram;
            // The cached guard has finished head/body/weapon drawing and is
            // in the shadow helper's register-only tests, before shadow OAM.
            if ram[0xdd0 + usize::from(slot)] == 9
                && (0x41..=0x44).contains(&ram[0xe20 + usize::from(slot)]) {
                return Some(crate::CachedSpriteExecutionProgress::Executing {
                    slot, progress: crate::CachedSpriteExecutionBodyProgress::GuardShadowOamPending,
                });
            }
        }
        if self.timer_oam_active && matches!(pc, 0x06_83f5 | 0x06_83f8
            | 0x06_83fa | 0x06_83fb | 0x06_83fc | 0x06_83fd | 0x06_8400
            | 0x06_8402 | 0x06_8405 | 0x06_8407 | 0x06_840d | 0x06_8413) {
            return Some(crate::CachedSpriteExecutionProgress::Executing {
                slot, progress: crate::CachedSpriteExecutionBodyProgress::AfterCoordinates,
            });
        }
        if self.timer_oam_active && (0x0d_bb56..=0x0d_bb5a).contains(&pc) {
            return Some(crate::CachedSpriteExecutionProgress::Executing {
                slot,
                progress: crate::CachedSpriteExecutionBodyProgress::AfterOamAllocation,
            });
        }
        if !self.recruit_draw_entered || pc != 0x06_e41e {
            return None;
        }
        let ram = &cpu.machine().snes().ram;
        assert_eq!(ram[0xe20 + usize::from(slot)], 0x4b);
        assert_eq!(ram[0xdd0 + usize::from(slot)], 9);
        Some(crate::CachedSpriteExecutionProgress::Executing {
            slot,
            progress: crate::CachedSpriteExecutionBodyProgress::BeforeGreenKnifeGuardRecruitOamPrep,
        })
    }
}

/// Zelda's concrete LoadFollowerGraphics call survives host/NMI boundaries.
/// Only caller identity and completed stores are retained; bytes stay native.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(not(feature = "rom-timing"), allow(dead_code))]
struct NativeExactCpuFollowerGraphicsTracker {
    active: Option<(u8, crate::RescuedMaidenInitializationStage)>,
}

impl NativeExactCpuFollowerGraphicsTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt, slot: u8) {
        use crate::RescuedMaidenInitializationStage::{FirstFollowerSheet, SecondFollowerSheet, Conversion};
        if step.origin_pc == 0x05_ebf2 {
            assert_eq!(step.opcode, 0x22, "Zelda graphics entry must be JSL");
            assert!(slot < 16 && self.active.is_none(), "nested Zelda follower graphics call");
            self.active = Some((slot, FirstFollowerSheet { completed_bytes: 0 }));
        }
        let Some((_, stage)) = self.active.as_mut() else { return };
        match step.origin_pc {
            0x00_d445 => {
                assert_eq!(*stage, FirstFollowerSheet { completed_bytes: 1536 });
                *stage = SecondFollowerSheet { completed_bytes: 0 };
            }
            0x00_d44a => {
                assert_eq!(*stage, SecondFollowerSheet { completed_bytes: 1536 });
                *stage = Conversion { completed_stores: 0 };
            }
            0x05_ebf6 => {
                assert_eq!(*stage, Conversion { completed_stores: 512 });
                self.active = None;
                return;
            }
            _ => {}
        }
        let sheet_base = match *stage {
            FirstFollowerSheet { .. } => Some(0x7f_4600),
            SecondFollowerSheet { .. } => Some(0x7f_4000),
            Conversion { .. } => None,
        };
        for access in &step.accesses {
            let SourceCpuBusAccessKind::Write { width, .. } = access.kind else { continue };
            match stage {
                FirstFollowerSheet { completed_bytes } | SecondFollowerSheet { completed_bytes }
                    if (0x00_e79e..=0x00_e851).contains(&step.origin_pc)
                        && (0x7f_4000..0x7f_4c00).contains(&access.address) => {
                    assert_eq!(width, 1);
                    let base = sheet_base.expect("decompressor requires its active sheet");
                    assert_eq!(access.address, base + u32::from(*completed_bytes),
                        "follower decompression stores must be contiguous");
                    *completed_bytes += 1;
                    assert!(*completed_bytes <= 1536);
                }
                Conversion { completed_stores }
                    if matches!(step.origin_pc, 0x00_d5dd | 0x00_d5ea | 0x00_d5f4 | 0x00_d601) => {
                    assert_eq!(width, 2);
                    let row = u32::from(*completed_stores / 2);
                    let dst = 0x7e_b940 + (row / 8) * 32 + (row % 8) * 2
                        + u32::from(*completed_stores % 2) * 16;
                    assert_eq!(access.address, dst, "follower plane stores changed order");
                    *completed_stores += 1;
                    assert!(*completed_stores <= 512);
                }
                _ => {}
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct NativeExactCpuAnimatedDungeonGraphicsTracker {
    active: Option<AnimatedDungeonGraphicsProgress>,
}

impl NativeExactCpuAnimatedDungeonGraphicsTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt) -> Option<AnimatedDungeonGraphicsProgress> {
        use AnimatedDungeonGraphicsProgress::*;
        if step.origin_pc == 0x00_d337 {
            assert!(self.active.is_none(), "nested animated dungeon graphics");
            self.active = Some(FirstSheet(0));
        }
        let stage = self.active.as_mut()?;
        match step.origin_pc {
            0x00_d33d => { assert_eq!(*stage, FirstSheet(1536)); *stage = FirstConversion(0); }
            0x00_d34a => { assert_eq!(*stage, FirstConversion(768)); *stage = SecondSheet(0); }
            0x00_d351 => { assert_eq!(*stage, SecondSheet(1536)); *stage = SecondConversion(0); }
            0x00_d35e => { assert_eq!(*stage, SecondConversion(768)); *stage = Rotation(0); }
            0x00_d38d => { assert_eq!(*stage, Rotation(1024)); *stage = DestinationPublished; }
            0x00_d393 => {
                assert_eq!(*stage, DestinationPublished);
                self.active = None;
                return Some(Returned);
            }
            _ => {}
        }
        let first_conversion = matches!(*stage, FirstConversion(_));
        for access in &step.accesses {
            let SourceCpuBusAccessKind::Write { width, .. } = access.kind else { continue };
            match stage {
                FirstSheet(count) | SecondSheet(count)
                    if (0x00_e79e..=0x00_e851).contains(&step.origin_pc)
                        && (0x7f_4000..0x7f_4600).contains(&access.address) => {
                    assert_eq!(width, 1);
                    assert_eq!(access.address, 0x7f_4000 + u32::from(*count));
                    *count += 1;
                    assert!(*count <= 1536);
                }
                FirstConversion(count) | SecondConversion(count)
                    if matches!(step.origin_pc, 0x00_d5dd | 0x00_d5ea | 0x00_d5f4 | 0x00_d601)
                        && (0x7e_a680..0x7e_b280).contains(&access.address) => {
                    assert_eq!(width, 2);
                    let row = u32::from(*count / 2);
                    let base = if first_conversion { 0x7e_a680 } else { 0x7e_ac80 };
                    assert_eq!(access.address, base + (row / 8) * 32 + (row % 8) * 2 + u32::from(*count % 2) * 16);
                    *count += 1;
                    assert!(*count <= 768);
                }
                Rotation(count) if matches!(step.origin_pc, 0x00_d36a | 0x00_d372 | 0x00_d37a | 0x00_d37f)
                    && (0x7e_a880..0x7e_b080).contains(&access.address) => {
                    let lane = usize::from(*count % 4);
                    assert_eq!(step.origin_pc, [0x00_d36a, 0x00_d372, 0x00_d37a, 0x00_d37f][lane]);
                    assert_eq!(width, 2);
                    assert_eq!(access.address, [0x7e_a880, 0x7e_ac80, 0x7e_ae80, 0x7e_aa80][lane] + u32::from(*count / 4) * 2);
                    *count += 1;
                    assert!(*count <= 1024);
                }
                _ => {}
            }
        }
        self.active
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct NativeExactCpuAnimatedOverworldGraphicsTracker {
    active: Option<AnimatedOverworldGraphicsProgress>,
}

impl NativeExactCpuAnimatedOverworldGraphicsTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt) -> Option<AnimatedOverworldGraphicsProgress> {
        use AnimatedOverworldGraphicsProgress::*;
        if step.origin_pc == 0x00_d394 {
            assert!(self.active.is_none(), "nested animated overworld graphics");
            self.active = Some(FirstSheet(0));
        }
        let stage = self.active.as_mut()?;
        match step.origin_pc {
            0x00_d39b => { assert_eq!(*stage, FirstSheet(1536)); *stage = FirstConversion(0); }
            0x00_d3a8 => { assert_eq!(*stage, FirstConversion(1024)); *stage = SecondSheet(0); }
            0x00_d3af => { assert_eq!(*stage, SecondSheet(1536)); *stage = SecondConversion(0); }
            0x00_d3c5 => {
                assert_eq!(*stage, SecondConversion(512));
                self.active = None;
                return Some(Returned);
            }
            _ => {}
        }
        let first_conversion = matches!(*stage, FirstConversion(_));
        for access in &step.accesses {
            let SourceCpuBusAccessKind::Write { width, .. } = access.kind else { continue };
            match stage {
                FirstSheet(count) | SecondSheet(count)
                    if (0x00_e79e..=0x00_e851).contains(&step.origin_pc)
                        && (0x7f_4000..0x7f_4600).contains(&access.address) => {
                    assert_eq!(width, 1);
                    assert_eq!(access.address, 0x7f_4000 + u32::from(*count),
                        "animated overworld decompression stores must be contiguous");
                    *count += 1;
                    assert!(*count <= 1536);
                }
                FirstConversion(count) | SecondConversion(count)
                    if matches!(step.origin_pc, 0x00_d5dd | 0x00_d5ea | 0x00_d5f4 | 0x00_d601) => {
                    assert_eq!(width, 2);
                    let row = u32::from(*count / 2);
                    let base = if first_conversion { 0x7e_a680 } else { 0x7e_ae80 };
                    let dst = base + (row / 8) * 32 + (row % 8) * 2 + u32::from(*count % 2) * 16;
                    assert_eq!(access.address, dst, "animated overworld plane stores changed order");
                    *count += 1;
                    assert!(*count <= if first_conversion { 1024 } else { 512 });
                }
                _ => {}
            }
        }
        self.active
    }
}

#[derive(Clone, Copy, Debug, Default)]
// Shared timing vocabulary is inspected by ungated gameplay accessors.
#[cfg_attr(not(feature = "rom-timing"), allow(dead_code))]
struct NativeExactCpuTilesetTracker {
    active: bool,
    current_sheet: Option<(usize, u32, u16, u16)>,
}

#[derive(Clone, Copy, Debug, Default)]
// Shared timing vocabulary is inspected by ungated gameplay accessors.
#[cfg_attr(not(feature = "rom-timing"), allow(dead_code))]
struct NativeExactCpuPreDungeonPaletteTracker {
    active: bool,
    cache_words: Option<u16>,
}

impl NativeExactCpuPreDungeonPaletteTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt) -> Option<u16> {
        if step.origin_pc == 0x02_828f {
            assert!(!self.active, "nested pre-dungeon palette load");
            self.active = true;
            self.cache_words = None;
        }
        if !self.active { return None; }
        if step.origin_pc == 0x02_c769 {
            assert!(self.cache_words.is_none());
            self.cache_words = Some(0);
        }
        if step.origin_pc == 0x02_8292 {
            assert_eq!(self.cache_words, Some(257));
            self.active = false;
            return Some(257);
        }
        let Some(count) = self.cache_words.as_mut() else { return None };
        for access in &step.accesses {
            let SourceCpuBusAccessKind::Write { width, .. } = access.kind else { continue };
            if matches!(step.origin_pc, 0x02_c771 | 0x02_c779 | 0x02_c781 | 0x02_c789
                | 0x02_c791 | 0x02_c799 | 0x02_c7a1 | 0x02_c7a9)
                && (0x7e_c500..0x7e_c700).contains(&access.address) {
                assert!(*count < 256);
                assert_eq!(width, 2);
                assert_eq!(step.origin_pc, 0x02_c771 + u32::from(*count % 8) * 8);
                assert_eq!(access.address, 0x7e_c500
                    + u32::from(*count % 8) * 64 + u32::from(*count / 8) * 2);
                *count += 1;
            } else if step.origin_pc == 0x02_c7b5
                && native_exact_cpu_wram_key(access.address) == Some(0x15) {
                assert_eq!((*count, width), (256, 1));
                *count = 257;
            }
        }
        Some(*count)
    }
}

impl NativeExactCpuTilesetTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt, sheet: u16, events: &mut Vec<TilesetGraphicsPublication>) {
        self.observe_call(step, sheet, events, 0x02_8492, 0x02_8496);
    }

    #[cfg(feature = "rom-timing")]
    fn observe_call(&mut self, step: &SourceCpuStepReceipt, sheet: u16,
        events: &mut Vec<TilesetGraphicsPublication>, entered_pc: u32, returned_pc: u32) {
        if step.origin_pc == entered_pc {
            assert!(!self.active, "nested tileset initialization");
            self.active = true;
            self.current_sheet = None;
            events.push(TilesetGraphicsPublication::Entered);
        }
        if !self.active { return; }
        if step.origin_pc == returned_pc {
            assert!(matches!(self.current_sheet, Some((11, _, _, 1536))),
                "pre-overworld tileset return omitted a sheet");
            events.push(TilesetGraphicsPublication::Returned);
            self.active = false;
            return;
        }
        if matches!(step.origin_pc, 0x00_e772 | 0x00_e78f) {
            let index = self.current_sheet.map_or(0, |(index, _, _, completed)| {
                assert_eq!(completed, 1536, "tileset advanced before completing a sheet");
                index + 1
            });
            assert!(index < 12);
            self.current_sheet = Some((index, step.origin_pc, sheet, 0));
            events.push(TilesetGraphicsPublication::Sheet { index, entry_pc: step.origin_pc, sheet, completed_bytes: 0 });
        }
        for access in &step.accesses {
            let SourceCpuBusAccessKind::Write { value, width: 1 } = access.kind else { continue };
            if (0x7e_c2fc..0x7e_c300).contains(&access.address) {
                events.push(TilesetGraphicsPublication::SpriteSelection {
                    slot: (access.address - 0x7e_c2fc) as usize, value: value as u8,
                });
            }
            if !(0x00_e79e..=0x00_e851).contains(&step.origin_pc) { continue; }
            let Some((index, entry_pc, sheet, completed)) = self.current_sheet.as_mut() else { continue };
            let base = match *index {
                0..=3 => 0x7e_7800 + *index as u32 * 0x600,
                7..=10 => 0x7e_6000 + (*index as u32 - 7) * 0x600,
                _ => 0x7f_4000,
            };
            if !(base..base + 1536).contains(&access.address) { continue; }
            assert_eq!(access.address, base + u32::from(*completed), "tileset decompression cursor skipped a store");
            *completed += 1;
            match events.last_mut() {
                Some(TilesetGraphicsPublication::Sheet { index: last, completed_bytes, .. }) if last == index => {
                    *completed_bytes = *completed;
                }
                _ => events.push(TilesetGraphicsPublication::Sheet {
                    index: *index, entry_pc: *entry_pc, sheet: *sheet, completed_bytes: *completed,
                }),
            }
        }
    }
}

/// Count the shared Map16ToMap8 stores and row tails. No source payload or
/// register value is retained; the native caller owns page, offsets and data.
#[derive(Clone, Copy, Debug, Default)]
// Shared timing vocabulary is inspected by ungated gameplay accessors.
#[cfg_attr(not(feature = "rom-timing"), allow(dead_code))]
struct NativeExactCpuOverworldMap8Tracker {
    active: bool,
    progress: Option<u16>,
}

impl NativeExactCpuOverworldMap8Tracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt) -> Option<u16> {
        use super::overworld::{OVERWORLD_MAP8_ROW_STEPS, OVERWORLD_MAP8_TOTAL_STEPS};
        if step.origin_pc == 0x02_fd46 {
            assert!(!self.active, "nested Map16ToMap8 call");
            self.active = true;
            self.progress = None;
        }
        if !self.active { return None; }
        if step.origin_pc == 0x02_fd4e {
            assert!(self.progress.is_none());
            self.progress = Some(0);
        }
        if step.origin_pc == 0x02_fd86 {
            assert_eq!(self.progress, Some(OVERWORLD_MAP8_TOTAL_STEPS));
            self.active = false;
            return self.progress;
        }
        let progress = self.progress.as_mut()?;
        let row_step = *progress % OVERWORLD_MAP8_ROW_STEPS;
        let retired = match step.origin_pc {
            0x02_fd9c | 0x02_fdaf => {
                assert!(row_step < 32);
                assert_eq!(step.origin_pc, if row_step % 2 == 0 { 0x02_fd9c } else { 0x02_fdaf });
                true
            }
            0x02_fdff => {
                assert!(matches!(row_step, 32 | 97));
                true
            }
            0x02_fe1e | 0x02_fe25 | 0x02_fe2e | 0x02_fe35 => {
                assert!((33..162).contains(&row_step) && row_step != 97);
                let quarter = ((row_step - 32) % 65 - 1) % 4;
                assert_eq!(step.origin_pc, [0x02_fe1e, 0x02_fe25, 0x02_fe2e, 0x02_fe35][usize::from(quarter)]);
                true
            }
            0x02_fd62 | 0x02_fd75 => { assert_eq!(row_step, 162); true }
            0x02_fd6a | 0x02_fd7d => { assert_eq!(row_step, 163); true }
            _ => false,
        };
        if retired {
            assert!(step.accesses.iter().any(|access|
                matches!(access.kind, SourceCpuBusAccessKind::Write { width: 2, .. })),
                "Map16ToMap8 progress requires its retired word store");
            *progress += 1;
            assert!(*progress <= OVERWORLD_MAP8_TOTAL_STEPS);
        }
        self.progress
    }
}

/// Publication progress of the shared four-quadrant caller. Source values
/// and addresses never author native map words.
#[derive(Clone, Copy, Debug, Default)]
// Shared timing vocabulary is inspected by ungated gameplay accessors.
#[cfg_attr(not(feature = "rom-timing"), allow(dead_code))]
struct NativeExactCpuOverworldQuadrantTracker {
    active: bool,
    completed_stores: u16,
}

impl NativeExactCpuOverworldQuadrantTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt) -> Option<u16> {
        if step.origin_pc == 0x02_f54a {
            assert!(!self.active, "nested overworld quadrant call");
            self.active = true;
            self.completed_stores = 0;
        }
        if !self.active { return None; }
        if matches!(step.origin_pc, 0x02_f7b5 | 0x02_f7ba | 0x02_f7c1 | 0x02_f7c6) {
            assert_eq!(step.origin_pc,
                [0x02_f7b5, 0x02_f7ba, 0x02_f7c1, 0x02_f7c6][usize::from(self.completed_stores % 4)]);
            assert!(step.accesses.iter().any(|access|
                matches!(access.kind, SourceCpuBusAccessKind::Write { width: 2, .. })));
            self.completed_stores += 1;
            assert!(self.completed_stores <= 4096);
        }
        if step.origin_pc == 0x02_f637 && self.completed_stores == 4096 {
            self.active = false;
        }
        Some(self.completed_stores)
    }
}

/// Counts retired initial-scroll stores, including each stripe's coordinate
/// tail and the enclosing South/East correction. No source payload is copied.
#[derive(Clone, Copy, Debug, Default)]
// Shared timing vocabulary is inspected by ungated gameplay accessors.
#[cfg_attr(not(feature = "rom-timing"), allow(dead_code))]
struct NativeExactCpuInitialScrollTracker {
    active: bool,
    progress: Option<u16>,
}

impl NativeExactCpuInitialScrollTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, step: &SourceCpuStepReceipt) -> Option<u16> {
        let pc = step.origin_pc;
        if pc == 0x02_f031 {
            assert!(!self.active, "nested initial scroll build");
            self.active = true;
            self.progress = None;
        }
        if !self.active { return None; }
        // The first upload header proves all direction-specific prefix
        // stores returned. Native derives that prefix from its own state.
        if matches!(pc, 0x02_ef8c | 0x02_efc5) {
            assert!(self.progress.is_none());
            self.progress = Some(0);
        } else if matches!(pc,
            0x02_f3dd | 0x02_f3ef | 0x02_f43b | 0x02_f43f
            | 0x02_f45a | 0x02_f461 | 0x02_f46a | 0x02_f471
            | 0x02_f4a7 | 0x02_f4b5 | 0x02_f4f1 | 0x02_f501
            | 0x02_f522 | 0x02_f529 | 0x02_f532 | 0x02_f539
            | 0x02_efcf | 0x02_efd1 | 0x02_efd9 | 0x02_ef9c | 0x02_efa4
            | 0x02_efe4 | 0x02_efaf
            | 0x02_f102 | 0x02_f10a | 0x02_f0b3 | 0x02_f0bb
            | 0x02_f1ef | 0x02_f1f7 | 0x02_f178 | 0x02_f180) {
            assert!(step.accesses.iter().any(|access|
                matches!(access.kind, SourceCpuBusAccessKind::Write { width: 2, .. })));
            *self.progress.as_mut().expect("initial scroll store before prefix") += 1;
        }
        if matches!(pc, 0x02_f086 | 0x02_f0bf | 0x02_f0db | 0x02_f10e
            | 0x02_f140 | 0x02_f184 | 0x02_f1b6 | 0x02_f1fb) {
            assert!(matches!(self.progress, Some(1149 | 1170 | 1315 | 1339)),
                "incomplete initial scroll return: {:?}", self.progress);
            self.active = false;
        }
        self.progress
    }
}

/// Only TimersAndOam's JSR owns this coordinate continuation. Keep that
/// provenance across NMI, where the handler has its own stack and registers.
#[derive(Clone, Copy, Debug, Default)]
struct NativeExactCpuSpriteCoordinateTracker {
    active_slot: Option<u8>,
}

impl NativeExactCpuSpriteCoordinateTracker {
    #[cfg(feature = "rom-timing")]
    fn observe(&mut self, pc: u32, slot: u8) {
        if pc == 0x06_83f2 {
            assert!(slot < 16);
            self.active_slot = Some(slot);
        } else if pc == 0x06_84d9 {
            self.active_slot = None;
        }
    }

    #[cfg(feature = "rom-timing")]
    fn prefix(&self, pc: u32) -> Option<(u8, u8)> {
        if (0x06_84c1..=0x06_84d9).contains(&pc) && (pc - 0x06_84c1) % 3 == 0 {
            self.active_slot.map(|slot| (slot, ((pc - 0x06_84c1) / 3) as u8))
        } else {
            None
        }
    }
}

pub(super) struct NativeExactCpuOwner {
    cpu: Snes9xColdCpuExecutor,
    next_host: u32,
    source_nmis_accepted: u64,
    source_nmis_completed: u64,
    translated_nmi_handlers_completed: u64,
    translated_trial: Option<Box<NativeExactCpuOwner>>,
    translated_trial_stopped: bool,
    interrupt_stack: Vec<NativeExactCpuInterrupt>,
    overworld_scan: NativeExactCpuOverworldScanTracker,
    sprite_preparation: NativeExactCpuSpritePreparationTracker,
    scroll_copy: NativeExactCpuScrollCopyTracker,
    floor_draw: NativeExactCpuFloorDrawTracker,
    cached_sprite: NativeExactCpuCachedSpriteTracker,
    dungeon_cache: NativeExactCpuDungeonCacheTracker,
    follower_graphics: NativeExactCpuFollowerGraphicsTracker,
    animated_overworld_graphics: NativeExactCpuAnimatedOverworldGraphicsTracker,
    animated_dungeon_graphics: NativeExactCpuAnimatedDungeonGraphicsTracker,
    pre_overworld_tilesets: NativeExactCpuTilesetTracker,
    pre_dungeon_tilesets: NativeExactCpuTilesetTracker,
    pre_dungeon_palettes: NativeExactCpuPreDungeonPaletteTracker,
    overworld_map8: NativeExactCpuOverworldMap8Tracker,
    overworld_quadrants: NativeExactCpuOverworldQuadrantTracker,
    initial_scroll: NativeExactCpuInitialScrollTracker,
    sprite_coordinates: NativeExactCpuSpriteCoordinateTracker,
    diagnostic_writers: HashMap<u32, NativeExactCpuWriteWitness>,
}

#[cfg(feature = "rom-timing")]
const NATIVE_EXACT_CPU_CHECKPOINT_VERSION: u8 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
#[cfg(feature = "rom-timing")]
struct NativeExactCpuOwnerCheckpoint {
    version: u8,
    cpu: Snes9xCpuQuiescentCheckpoint,
    next_host: u32,
    source_nmis_accepted: u64,
    source_nmis_completed: u64,
    translated_nmi_handlers_completed: u64,
    translated_trial: Option<Box<NativeExactCpuOwnerCheckpoint>>,
    translated_trial_stopped: bool,
    interrupt_stack: Vec<NativeExactCpuInterrupt>,
    overworld_scan: NativeExactCpuOverworldScanTracker,
    sprite_preparation: NativeExactCpuSpritePreparationTracker,
    floor_draw: NativeExactCpuFloorDrawTracker,
    cached_sprite: NativeExactCpuCachedSpriteTracker,
    dungeon_cache: NativeExactCpuDungeonCacheTracker,
    diagnostic_writers: HashMap<u32, NativeExactCpuWriteWitness>,
}

/// Version 2 retains source-copy positions alongside the unchanged version-1
/// owner shape, so existing paired checkpoints remain readable.
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg(feature = "rom-timing")]
struct NativeExactCpuOwnerCheckpointV2 {
    version: u8,
    owner: NativeExactCpuOwnerCheckpoint,
    scroll_cursors: Vec<NativeExactCpuScrollCopyTracker>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[cfg(feature = "rom-timing")]
struct NativeExactCpuDialogueContinuationCheckpoint {
    fast_forward_hold_pending: bool,
    fast_forward_hold_active: bool,
    completion_stops_before_scroll: bool,
    handler_completed_at_endpoint: bool,
    live_message_read_position_target: Option<u16>,
    handler_entry_phase: crate::zelda_rtl::messaging::VwfHandlerEntryPhase,
    glyph_cpu_phase: crate::zelda_rtl::messaging::VwfGlyphCpuPhase,
    pending_glyph_read_pos: Option<u16>,
    dispatch_cursor: crate::cycle_models::vwf::DispatchCursor,
    character_load_progress: Option<crate::zelda_rtl::messaging::DialogueCharacterLoadProgress>,
    deferred_handler_exits: u32,
    scroll_copy_cursor: Option<crate::zelda_rtl::messaging::NativeScrollCopyCursor>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[cfg(feature = "rom-timing")]
struct NativeExactCpuOwnerCheckpointV3 {
    version: u8,
    owner: NativeExactCpuOwnerCheckpoint,
    scroll_cursors: Vec<NativeExactCpuScrollCopyTracker>,
    dialogue: NativeExactCpuDialogueContinuationCheckpoint,
}

/// Version 4 retains the CPU work between completed glyphs when NMI stops
/// inside `VWF_RenderSingle`'s return path.
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg(feature = "rom-timing")]
struct NativeExactCpuOwnerCheckpointV4 {
    version: u8,
    owner: NativeExactCpuOwnerCheckpoint,
    scroll_cursors: Vec<NativeExactCpuScrollCopyTracker>,
    dialogue: NativeExactCpuDialogueContinuationCheckpoint,
    transition_remaining_master_cycles: u32,
}

impl NativeExactCpuOwner {
    fn checkpoint_hosts_are_aligned(&self) -> bool {
        self.translated_trial.as_ref().is_none_or(|trial| {
            trial.next_host == self.next_host && trial.checkpoint_hosts_are_aligned()
        })
    }

    fn follower_graphics_is_quiescent(&self) -> bool {
        self.follower_graphics.active.is_none()
            && self.animated_overworld_graphics.active.is_none()
            && self.animated_dungeon_graphics.active.is_none()
            && !self.pre_overworld_tilesets.active
            && !self.pre_dungeon_tilesets.active
            && !self.pre_dungeon_palettes.active
            && !self.overworld_map8.active
            && !self.overworld_quadrants.active
            && !self.initial_scroll.active
            && self.sprite_coordinates.active_slot.is_none()
            && self.translated_trial.as_ref().is_none_or(|trial| trial.follower_graphics_is_quiescent())
    }

    #[cfg(feature = "rom-timing")]
    fn capture_checkpoint(&self) -> Result<NativeExactCpuOwnerCheckpoint, String> {
        if !self.checkpoint_hosts_are_aligned() {
            return Err("native exact CPU trial checkpoint has a different next host".to_string());
        }
        if !self.follower_graphics_is_quiescent() {
            return Err("native exact CPU checkpoint cannot capture active follower graphics or overworld graphics".to_string());
        }
        Ok(NativeExactCpuOwnerCheckpoint {
            version: NATIVE_EXACT_CPU_CHECKPOINT_VERSION,
            cpu: self
                .cpu
                .capture_quiescent_checkpoint()
                .map_err(|error| error.to_string())?,
            next_host: self.next_host,
            source_nmis_accepted: self.source_nmis_accepted,
            source_nmis_completed: self.source_nmis_completed,
            translated_nmi_handlers_completed: self.translated_nmi_handlers_completed,
            translated_trial: self
                .translated_trial
                .as_ref()
                .map(|trial| trial.capture_checkpoint().map(Box::new))
                .transpose()?,
            translated_trial_stopped: self.translated_trial_stopped,
            interrupt_stack: self.interrupt_stack.clone(),
            overworld_scan: self.overworld_scan,
            sprite_preparation: self.sprite_preparation,
            floor_draw: self.floor_draw,
            cached_sprite: self.cached_sprite,
            dungeon_cache: self.dungeon_cache,
            diagnostic_writers: self.diagnostic_writers.clone(),
        })
    }

    #[cfg(feature = "rom-timing")]
    fn from_checkpoint(checkpoint: NativeExactCpuOwnerCheckpoint) -> Result<Self, String> {
        if checkpoint.version != NATIVE_EXACT_CPU_CHECKPOINT_VERSION {
            return Err(format!(
                "unsupported native exact CPU checkpoint version {}",
                checkpoint.version
            ));
        }
        let translated_trial = checkpoint
            .translated_trial
            .map(|trial| Self::from_checkpoint(*trial).map(Box::new))
            .transpose()?;
        if translated_trial
            .as_ref()
            .is_some_and(|trial| trial.next_host != checkpoint.next_host)
        {
            return Err("native exact CPU trial checkpoint has a different next host".to_string());
        }
        Ok(Self {
            cpu: Snes9xColdCpuExecutor::from_quiescent_checkpoint(checkpoint.cpu)
                .map_err(|error| error.to_string())?,
            next_host: checkpoint.next_host,
            source_nmis_accepted: checkpoint.source_nmis_accepted,
            source_nmis_completed: checkpoint.source_nmis_completed,
            translated_nmi_handlers_completed: checkpoint.translated_nmi_handlers_completed,
            translated_trial,
            translated_trial_stopped: checkpoint.translated_trial_stopped,
            interrupt_stack: checkpoint.interrupt_stack,
            overworld_scan: checkpoint.overworld_scan,
            sprite_preparation: checkpoint.sprite_preparation,
            scroll_copy: NativeExactCpuScrollCopyTracker::default(),
            floor_draw: checkpoint.floor_draw,
            cached_sprite: checkpoint.cached_sprite,
            dungeon_cache: checkpoint.dungeon_cache,
            follower_graphics: NativeExactCpuFollowerGraphicsTracker::default(),
            animated_overworld_graphics: NativeExactCpuAnimatedOverworldGraphicsTracker::default(),
            animated_dungeon_graphics: NativeExactCpuAnimatedDungeonGraphicsTracker::default(),
            pre_overworld_tilesets: NativeExactCpuTilesetTracker::default(),
            pre_dungeon_tilesets: NativeExactCpuTilesetTracker::default(),
            pre_dungeon_palettes: NativeExactCpuPreDungeonPaletteTracker::default(),
            overworld_map8: NativeExactCpuOverworldMap8Tracker::default(),
            overworld_quadrants: NativeExactCpuOverworldQuadrantTracker::default(),
            initial_scroll: NativeExactCpuInitialScrollTracker::default(),
            sprite_coordinates: NativeExactCpuSpriteCoordinateTracker::default(),
            diagnostic_writers: checkpoint.diagnostic_writers,
        })
    }

    #[cfg(feature = "rom-timing")]
    fn collect_scroll_cursors(&self, cursors: &mut Vec<NativeExactCpuScrollCopyTracker>) {
        cursors.push(self.scroll_copy);
        if let Some(trial) = self.translated_trial.as_ref() {
            trial.collect_scroll_cursors(cursors);
        }
    }

    #[cfg(feature = "rom-timing")]
    fn restore_scroll_cursors(
        &mut self,
        cursors: &mut impl Iterator<Item = NativeExactCpuScrollCopyTracker>,
    ) -> Result<(), String> {
        self.scroll_copy = cursors.next()
            .ok_or("native exact CPU checkpoint omitted a scroll cursor")?;
        if let Some(trial) = self.translated_trial.as_mut() {
            trial.restore_scroll_cursors(cursors)?;
        }
        Ok(())
    }
}

impl ZeldaState {
    pub(super) fn advance_native_exact_cpu_dialogue_loader(&mut self) {
        let Some(source_cursor) = self.native_exact_cpu_host_trace.as_ref()
            .and_then(NativeExactCpuHostTrace::dialogue_decode_cursor_at_return)
        else {
            return;
        };
        if crate::debug_env::var_os!("ZELDA3_DEBUG_NATIVE_DIALOGUE_LOADER").is_some() {
            eprintln!("native-dialogue-loader host={} source_cursor={} native_cursor={} work={:?} message={}",
                self.frame_ctr_dbg - 1, source_cursor,
                self.game_state.messaging.runtime.dialogue_msg_read_pos(),
                self.game_execution_scheduler.current_work(),
                self.game_state.messaging.dialogue_message_index.value());
        }
        if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_DIALOGUE_DECODE_LIVE_TRIAL").is_none()
            || self.game_execution_scheduler.current_work()
                != Some(GameWorkContinuation::FinishDialogueInitializationCallerReturn)
        {
            return;
        }
        if source_cursor > self.game_state.messaging.runtime.dialogue_msg_read_pos() {
            // The source caller is still inside Text_LoadCharacterBuffer.
            // Resume the retained native decoder once through the source's
            // committed input cursor; no decoded bytes come from the trace.
            self.prepare_text_character_buffer_for_carry();
        }
    }


    /// Separate sidecar because ordinary gameplay save states deliberately
    /// omit the retained source CPU and its interrupted call stack.
    fn native_exact_cpu_dialogue_checkpoint_is_quiescent(&self) -> bool {
        // The sidecar retains semantic VWF progress, but not the native
        // cycle-ledger anchor used to price a held caller's next slice.
        // Capture after that caller returns, when NMI establishes a new anchor.
        !self.dialogue_fast_forward_hold_pending
            && !self.dialogue_fast_forward_hold_active
            && self.dialogue_vwf_glyph_cpu_phase.is_ready()
            && self.dialogue_vwf_transition_remaining_master_cycles == 0
    }

    pub fn native_exact_cpu_checkpoint_available(&self) -> bool {
        self.native_exact_cpu_owner
            .as_ref()
            .is_some_and(|owner| owner.next_host == self.frame_ctr_dbg
                && owner.checkpoint_hosts_are_aligned() && owner.follower_graphics_is_quiescent())
            && self.native_exact_cpu_dialogue_checkpoint_is_quiescent()
            && self.native_pre_overworld_animated_graphics.is_none()
            && self.native_pre_dungeon_animated_graphics.is_none()
            && self.native_pre_overworld_tileset_publication.is_none()
            && self.native_pre_dungeon_tileset_publication.is_none()
            && self.native_pre_dungeon_palette_cache.is_none()
            && self.native_overworld_map8.is_none()
            && self.native_overworld_quadrants.is_none()
            && self.native_initial_scroll.is_none()
            && self.native_overworld_spotlight_link_oam.is_none()
            && self.native_exact_cpu_overworld_live_scan.is_none()
            && self.native_exact_cpu_overworld_scan_trial.is_none()
            && !matches!(self.dialogue_scroll_phase(),
                crate::zelda_rtl::DialogueScrollPhase::CompletionStagedAfterFrozenScanout
                | crate::zelda_rtl::DialogueScrollPhase::CompletionStagedAfterSnapshot)
            && self.active_spiral_aux_graphics.is_none()
    }




}

impl std::fmt::Debug for NativeExactCpuOwner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeExactCpuOwner")
            .field("next_host", &self.next_host)
            .field("timestamp", &self.cpu.machine().timestamp())
            .finish_non_exhaustive()
    }
}

impl Clone for NativeExactCpuOwner {
    fn clone(&self) -> Self {
        let checkpoint = self
            .cpu
            .capture_quiescent_checkpoint()
            .expect("native exact CPU is cloned only at a host return");
        Self {
            cpu: Snes9xColdCpuExecutor::from_quiescent_checkpoint(checkpoint)
                .expect("a captured exact CPU checkpoint restores"),
            next_host: self.next_host,
            source_nmis_accepted: self.source_nmis_accepted,
            source_nmis_completed: self.source_nmis_completed,
            translated_nmi_handlers_completed: self.translated_nmi_handlers_completed,
            translated_trial: self
                .translated_trial
                .as_ref()
                .map(|trial| Box::new((**trial).clone())),
            translated_trial_stopped: self.translated_trial_stopped,
            interrupt_stack: self.interrupt_stack.clone(),
            overworld_scan: self.overworld_scan,
            sprite_preparation: self.sprite_preparation,
            scroll_copy: self.scroll_copy,
            floor_draw: self.floor_draw,
            cached_sprite: self.cached_sprite,
            dungeon_cache: self.dungeon_cache,
            follower_graphics: self.follower_graphics,
            animated_overworld_graphics: self.animated_overworld_graphics,
            animated_dungeon_graphics: self.animated_dungeon_graphics,
            pre_overworld_tilesets: self.pre_overworld_tilesets,
            pre_dungeon_tilesets: self.pre_dungeon_tilesets,
            pre_dungeon_palettes: self.pre_dungeon_palettes,
            overworld_map8: self.overworld_map8,
            overworld_quadrants: self.overworld_quadrants,
            initial_scroll: self.initial_scroll,
            sprite_coordinates: self.sprite_coordinates,
            diagnostic_writers: self.diagnostic_writers.clone(),
        }
    }
}

impl NativeExactCpuOwner {
    #[cfg(feature = "rom-timing")]
    fn new(rom: &[u8], sram: &[u8]) -> Result<Self, SourceCpuError> {
        Ok(Self {
            cpu: Snes9xColdCpuExecutor::from_lorom_reset_with_sram(rom, Some(sram))?,
            next_host: 0,
            source_nmis_accepted: 0,
            source_nmis_completed: 0,
            translated_nmi_handlers_completed: 0,
            translated_trial: None,
            translated_trial_stopped: false,
            interrupt_stack: Vec::new(),
            overworld_scan: NativeExactCpuOverworldScanTracker::default(),
            sprite_preparation: NativeExactCpuSpritePreparationTracker::default(),
            scroll_copy: NativeExactCpuScrollCopyTracker::default(),
            floor_draw: NativeExactCpuFloorDrawTracker::default(),
            cached_sprite: NativeExactCpuCachedSpriteTracker::default(),
            dungeon_cache: NativeExactCpuDungeonCacheTracker::default(),
            follower_graphics: NativeExactCpuFollowerGraphicsTracker::default(),
            animated_overworld_graphics: NativeExactCpuAnimatedOverworldGraphicsTracker::default(),
            animated_dungeon_graphics: NativeExactCpuAnimatedDungeonGraphicsTracker::default(),
            pre_overworld_tilesets: NativeExactCpuTilesetTracker::default(),
            pre_dungeon_tilesets: NativeExactCpuTilesetTracker::default(),
            pre_dungeon_palettes: NativeExactCpuPreDungeonPaletteTracker::default(),
            overworld_map8: NativeExactCpuOverworldMap8Tracker::default(),
            overworld_quadrants: NativeExactCpuOverworldQuadrantTracker::default(),
            initial_scroll: NativeExactCpuInitialScrollTracker::default(),
            sprite_coordinates: NativeExactCpuSpriteCoordinateTracker::default(),
            diagnostic_writers: HashMap::new(),
        })
    }

    #[cfg(feature = "rom-timing")]
    fn record_step(
        interrupt_stack: &mut Vec<NativeExactCpuInterrupt>,
        overworld_scan: &mut NativeExactCpuOverworldScanTracker,
        sprite_preparation: &mut NativeExactCpuSpritePreparationTracker,
        scroll_copy: &mut NativeExactCpuScrollCopyTracker,
        floor_draw: &mut NativeExactCpuFloorDrawTracker,
        trace: &mut NativeExactCpuHostTrace,
        step: &SourceCpuStepReceipt,
        reads: Option<&mut Vec<NativeExactCpuRead>>,
        writes: Option<&mut Vec<NativeExactCpuWrite>>,
        instructions: Option<&mut Vec<NativeExactCpuInstruction>>,
    ) -> Option<NativeExactCpuNmiCompletion> {
        if trace.track_vwf_cpu_work && interrupt_stack.iter().any(|interrupt| matches!(interrupt, NativeExactCpuInterrupt::Nmi(_))) {
            let block = match step.origin_pc {
                0x00_80c9..=0x00_811b => 0,
                0x00_811c..=0x00_8143 => 1,
                0x00_8144..=0x00_822c => 2,
                _ => 3,
            };
            trace.nmi_cpu_work_by_block[block] += step.transactions.iter()
                .map(|transaction| u64::from(transaction.duration_master_cycles))
                .sum::<u64>();
        }
        if trace.track_vwf_cpu_work
            && ((0x0e_c984..=0x0e_ccf7).contains(&step.origin_pc)
                || (0x00_8781..=0x00_87b5).contains(&step.origin_pc))
        {
            trace.vwf_first_step_at.get_or_insert(step.started_at.master_cycles());
            trace.vwf_last_step_at = Some(step.ended_at.master_cycles());
            let interrupt_started_at = step.accepted_interrupt.map(|interrupt| match interrupt {
                SourceCpuAcceptedInterrupt::Nmi { started_at, .. }
                | SourceCpuAcceptedInterrupt::Irq { started_at, .. } => started_at.master_cycles(),
            });
            let work = step.transactions.iter()
                .filter(|transaction| interrupt_started_at.is_none_or(|at| transaction.started_at.master_cycles() < at))
                .map(|transaction| u64::from(transaction.duration_master_cycles))
                .sum::<u64>();
            trace.vwf_cpu_work_master_cycles += work;
            trace.vwf_work_since_nmi += work;
            if step.origin_pc == 0x0e_cbd1 {
                trace.vwf_draw_entry_work.push(trace.vwf_work_since_nmi - work);
            }
        }
        overworld_scan.observe(step, &mut trace.overworld_scan_events);
        sprite_preparation.observe(step);
        scroll_copy.observe(step, trace);
        floor_draw.observe(step, trace);
        if (0x00_e79e..=0x00_e851).contains(&step.origin_pc) {
            for access in &step.accesses {
                let SourceCpuBusAccessKind::Write { value, width: 1 } = access.kind else {
                    continue;
                };
                if (0x7e_6000..0x7e_9000).contains(&access.address) {
                    let offset = access.address - 0x7e_6000;
                    trace.transition_aux_graphics_cursor = Some(super::load_gfx::TransitionAuxGraphicsProgress {
                        sheet: (offset / 0x600) as u8,
                        completed_bytes: (offset % 0x600 + 1) as u16,
                    });
                }
                let (first_sheet, base) = match access.address {
                    0x7f_4600..=0x7f_4bff => (true, 0x7f_4600),
                    0x7f_4000..=0x7f_45ff => (false, 0x7f_4000),
                    _ => continue,
                };
                let completed_bytes = (access.address - base + 1) as u16;
                trace.sprite_decompression_progress = Some((first_sheet, completed_bytes, value as u8));
            }
            for access in &step.accesses {
                let SourceCpuBusAccessKind::Write { value, width: 1 } = access.kind else {
                    continue;
                };
                if (0x7e_6000..0x7e_9000).contains(&access.address) {
                    trace.aux_graphics_decompression_store =
                        Some(((access.address & 0xffff) as usize, value as u8));
                }
            }
        }
        if step.origin_pc == 0x02_8a5f {
            trace.room_load_returned = true;
        }
        if step.origin_pc == 0x02_8a67 {
            trace.auxiliary_sprite_graphics_returned = true;
            if crate::debug_env::var_os!("ZELDA3_NATIVE_EXACT_CPU_TRACE_AUX_RETURN").is_some() {
                eprintln!(
                    "native-exact-cpu-aux-return host={} at={}",
                    trace.host,
                    step.started_at.master_cycles()
                );
            }
        }
        if step.origin_pc == 0x02_8856 {
            trace.module07_link_oam_returned = true;
            trace.module07_link_oam_returns.push(step.started_at.master_cycles());
        }
        if step.origin_pc == 0x01_88a2 {
            trace.room_post_object_prefix_value = step.accesses.iter().find_map(|access| {
                match access.kind {
                    SourceCpuBusAccessKind::Write { value, width: 2 }
                        if access.address as u16 == 0x0478 => Some(value),
                    _ => None,
                }
            });
        }
        if let Some(instructions) = instructions {
            instructions.push(NativeExactCpuInstruction {
                pc: step.origin_pc,
                opcode: step.opcode,
                started_at: step.started_at.master_cycles(),
                ended_at: step.ended_at.master_cycles(),
            });
        }
        if let Some(reads) = reads {
            for access in &step.accesses {
                if let SourceCpuBusAccessKind::Read { value, width } = access.kind {
                    reads.push(NativeExactCpuRead {
                        pc: step.origin_pc,
                        address: access.address,
                        value,
                        width,
                        at: access.timestamp.master_cycles(),
                    });
                }
            }
        }
        if let Some(writes) = writes {
            for access in &step.accesses {
                if let SourceCpuBusAccessKind::Write { value, width } = access.kind {
                    writes.push(NativeExactCpuWrite {
                        pc: step.origin_pc,
                        address: access.address,
                        value,
                        width,
                        at: access.timestamp.master_cycles(),
                    });
                }
            }
        }
        if trace.track_story_decompression_writes
            && (0x00_e790..=0x00_e842).contains(&step.origin_pc)
        {
            for access in &step.accesses {
                if (0x7f_4000..0x7f_5000).contains(&access.address) {
                    if let SourceCpuBusAccessKind::Write { value, width } = access.kind {
                        trace.story_decompression_writes.push(NativeExactCpuWrite {
                            pc: step.origin_pc,
                            address: access.address,
                            value,
                            width,
                            at: access.timestamp.master_cycles(),
                        });
                    }
                }
            }
        }
        if trace.track_dialogue_decode_cursor_writes
            && (0x0e_c000..=0x0e_dfff).contains(&step.origin_pc)
        {
            for access in &step.accesses {
                if access.address as u16 == 0x1cd9
                    && (access.address >> 16) <= 0x3f
                {
                    if let SourceCpuBusAccessKind::Write { value, width } = access.kind {
                        trace.dialogue_decode_cursor_writes.push(NativeExactCpuWrite {
                            pc: step.origin_pc,
                            address: access.address,
                            value,
                            width,
                            at: access.timestamp.master_cycles(),
                        });
                        if trace.track_vwf_cpu_work {
                            trace.vwf_cursor_work.push((value, trace.vwf_work_since_nmi));
                        }
                    }
                }
            }
        }
        for access in &step.accesses {
            // Keep the CPU's actual NMI branch operands in bus order. A
            // dialogue caller can set $0710 while the $12 latch makes one
            // accepted handler skip NMI_DoUpdates entirely.
            let bank = access.address >> 16;
            let address = access.address as u16;
            if bank <= 0x3f && matches!(address, 0x012c | 0x0133) {
                let music_access = match access.kind {
                    SourceCpuBusAccessKind::Read { value, width } => {
                        Some((NativeExactCpuNmiGateKind::Read, value, width))
                    }
                    SourceCpuBusAccessKind::Write { value, width } => {
                        Some((NativeExactCpuNmiGateKind::Write, value, width))
                    }
                    _ => None,
                };
                if let Some((kind, value, width)) = music_access {
                    trace.music_port_accesses.push(NativeExactCpuMusicAccess {
                        pc: step.origin_pc,
                        address,
                        kind,
                        value,
                        width,
                        at: access.timestamp.master_cycles(),
                    });
                }
            }
            if address == 0x067a && bank <= 0x3f {
                match (step.origin_pc, access.kind) {
                    (0x00_f341, SourceCpuBusAccessKind::Write { value, width: 2 }) => {
                        trace.spotlight_events.push(
                            NativeExactCpuSpotlightEvent::WindowYInitialized {
                                value,
                                at: access.timestamp.master_cycles(),
                            },
                        );
                    }
                    (0x00_f36d, SourceCpuBusAccessKind::Read { value, width: 2 }) => {
                        trace.spotlight_events.push(
                            NativeExactCpuSpotlightEvent::CircleInputRead {
                                value,
                                at: access.timestamp.master_cycles(),
                            },
                        );
                    }
                    _ => {}
                }
            }
            if bank <= 0x3f
                && (0x1b00..0x1cc0).contains(&address)
                && (0x00_f42f..=0x00_f441).contains(&step.origin_pc)
            {
                if let SourceCpuBusAccessKind::Write { value, width: 2 } = access.kind {
                    assert_eq!(value, 0xff00, "spotlight reset table source store changed");
                    assert_eq!(address & 1, 0, "spotlight reset table wrote an odd byte");
                    trace.spotlight_reset_stores.push((address - 0x1b00) / 2);
                }
            }
            if step.origin_pc == 0x09_8801
                && (ANCILLA_STEP..ANCILLA_STEP + 10).contains(&usize::from(address))
            {
                if let SourceCpuBusAccessKind::Write { value, width: 1 } = access.kind {
                    trace
                        .item_receipt_step_writes
                        .push(NativeExactCpuItemReceiptStepWrite {
                            slot: (usize::from(address) - ANCILLA_STEP) as u8,
                            value: value as u8,
                            at: access.timestamp.master_cycles(),
                        });
                }
            }
            if trace.track_floor_tile_writes && address == 0x045c {
                if let SourceCpuBusAccessKind::Write { value, width: 1 } = access.kind {
                    let at = access.timestamp.master_cycles();
                    if step.origin_pc == 0x02_c58a && value == 0 {
                        trace
                            .room_upload_events
                            .push(NativeExactCpuRoomUploadEvent::Began { at });
                    } else if step.origin_pc == 0x00_91b3
                        && value != 0
                        && value <= 16
                        && value % 4 == 0
                    {
                        trace.room_upload_events.push(
                            NativeExactCpuRoomUploadEvent::QuadrantPrepared {
                                count: (value / 4) as u8,
                                at,
                            },
                        );
                    }
                }
            }
            if step.origin_pc == 0x01_b6c1 && address == 0x0110 {
                if let SourceCpuBusAccessKind::Write { value, width: 2 } = access.kind {
                    trace
                        .dungeon_header_index_writes
                        .push(NativeExactCpuDungeonHeaderIndexWrite {
                            value,
                            at: access.timestamp.master_cycles(),
                        });
                }
            }
            let wram_bank = matches!(bank, 0x7e | 0x7f)
                || (address < 0x2000 && (bank <= 0x3f || (0x80..=0xbf).contains(&bank)));
            if trace.wram_watch_address == Some(address) && wram_bank {
                if trace.track_vwf_cpu_work && matches!(access.kind, SourceCpuBusAccessKind::Write { .. }) {
                    eprintln!("native-exact-vwf-store-work host={} pc={:06x} address={:06x} work={} at={}",
                        trace.host, step.origin_pc, access.address, trace.vwf_work_since_nmi,
                        access.timestamp.master_cycles());
                }
                let watched = match access.kind {
                    SourceCpuBusAccessKind::Read { value, width } => {
                        Some((NativeExactCpuNmiGateKind::Read, value, width))
                    }
                    SourceCpuBusAccessKind::Write { value, width } => {
                        Some((NativeExactCpuNmiGateKind::Write, value, width))
                    }
                    SourceCpuBusAccessKind::OpcodeFetch { .. }
                    | SourceCpuBusAccessKind::ReadLong { .. } => None,
                };
                if let Some((kind, value, width)) = watched {
                    trace
                        .wram_watch_accesses
                        .push(NativeExactCpuWramWatchAccess {
                            pc: step.origin_pc,
                            kind,
                            value,
                            width,
                            at: access.timestamp.master_cycles(),
                        });
                }
            }
            let nmi_accepted_at =
                interrupt_stack
                    .iter()
                    .rev()
                    .find_map(|interrupt| match interrupt {
                        NativeExactCpuInterrupt::Nmi(nmi) => Some(nmi.accepted_at),
                        NativeExactCpuInterrupt::Irq => None,
                    });
            if step.origin_pc == 0x00_805d && address == 0x0012 {
                if let SourceCpuBusAccessKind::Write { value: 0, width: 1 } = access.kind {
                    trace.main_wait_returns.push(NativeExactCpuMainWaitReturn {
                        at: access.timestamp.master_cycles(),
                    });
                }
            }
            if step.origin_pc == 0x00_8051 && address == 0x001a {
                if let SourceCpuBusAccessKind::Write { value, width: 1 } = access.kind {
                    trace.main_loop_entries.push(NativeExactCpuMainLoopEntry {
                        frame_counter: value as u8,
                        at: access.timestamp.master_cycles(),
                    });
                }
            }
            if matches!(address, 0x0012 | 0x0710)
                && (bank <= 0x3f || bank == 0x7e || (0x80..=0xbf).contains(&bank))
                && (address != 0x0012 || nmi_accepted_at.is_some())
            {
                let (kind, value, width) = match access.kind {
                    SourceCpuBusAccessKind::Read { value, width } => {
                        (NativeExactCpuNmiGateKind::Read, value, width)
                    }
                    SourceCpuBusAccessKind::Write { value, width } => {
                        (NativeExactCpuNmiGateKind::Write, value, width)
                    }
                    SourceCpuBusAccessKind::OpcodeFetch { .. }
                    | SourceCpuBusAccessKind::ReadLong { .. } => continue,
                };
                trace.nmi_gate_accesses.push(NativeExactCpuNmiGateAccess {
                    pc: step.origin_pc,
                    address,
                    kind,
                    value,
                    width,
                    at: access.timestamp.master_cycles(),
                    nmi_accepted_at,
                });
                if step.origin_pc == 0x00_8138 && address == 0x0012 {
                    if let (Some(accepted_at), NativeExactCpuNmiGateKind::Read) =
                        (nmi_accepted_at, kind)
                    {
                        trace
                            .nmi_update_decisions
                            .push(NativeExactCpuNmiUpdateDecision {
                                accepted_at,
                                decided_at: access.timestamp.master_cycles(),
                                latch: value as u8,
                                runs_updates: value == 0,
                            });
                    }
                }
            }
            if matches!(access.address as u16, 0x2137 | 0x213c | 0x213d | 0x213f) {
                if let SourceCpuBusAccessKind::Read { value, width: 1 } = access.kind {
                    trace.ppu_reads.push(NativeExactCpuPpuAccess {
                        pc: step.origin_pc,
                        address: access.address,
                        value: value as u8,
                        at: access.timestamp.master_cycles(),
                    });
                }
            }
        }
        if step.origin_pc == 0x00_f39c {
            trace
                .spotlight_events
                .push(NativeExactCpuSpotlightEvent::RowAdvanced {
                    at: step.ended_at.master_cycles(),
                });
        }
        if step.origin_pc == 0x00_f3bb {
            for access in &step.accesses {
                if matches!(access.kind, SourceCpuBusAccessKind::Write { width: 2, .. })
                    && native_exact_cpu_wram_key(access.address)
                        .is_some_and(|offset| (0x1b00..0x1cc0).contains(&offset)) {
                    trace.spotlight_events.push(NativeExactCpuSpotlightEvent::ProjectionWordStored {
                        at: access.timestamp.master_cycles(),
                    });
                }
            }
        }
        // The instruction's bus accesses precede both its RTI return and an
        // interrupt accepted at its end. Update ancestry only afterward.
        let completed = if step.opcode == 0x40 {
            match interrupt_stack.pop() {
                Some(NativeExactCpuInterrupt::Nmi(accepted)) => Some(NativeExactCpuNmiCompletion {
                    accepted_at: accepted.accepted_at,
                    returned_at: step.ended_at.master_cycles(),
                }),
                Some(NativeExactCpuInterrupt::Irq) | None => None,
            }
        } else {
            None
        };
        if let Some(completed) = completed {
            trace.nmi_completions.push(completed);
            trace.vwf_work_since_nmi = 0;
        }
        match step.accepted_interrupt {
            Some(SourceCpuAcceptedInterrupt::Nmi {
                started_at,
                interrupted_pc,
            }) => {
                let accepted = NativeExactCpuNmi {
                    interrupted_pc,
                    accepted_at: started_at.master_cycles(),
                };
                trace.nmis.push(accepted);
                if (0x00_85fe..=0x00_865a).contains(&interrupted_pc) {
                    let progress = sprite_preparation
                        .progress
                        .expect("interrupted sprite preparation lost its packing cursor");
                    progress.validate();
                    trace.sprite_preparation_interruptions.push(
                        NativeExactCpuSpritePreparationInterruption {
                            accepted_at: accepted.accepted_at,
                            interrupted_pc,
                            progress,
                        },
                    );
                }
                interrupt_stack.push(NativeExactCpuInterrupt::Nmi(accepted));
            }
            Some(SourceCpuAcceptedInterrupt::Irq { .. }) => {
                interrupt_stack.push(NativeExactCpuInterrupt::Irq);
            }
            None => {}
        }
        completed
    }




}

#[cfg(feature = "rom-timing")]
fn rebase_translated_timing_state(
    cpu: &mut Snes9xColdCpuExecutor,
    ram: &[u8],
    sram: &[u8],
    preserve_stack_segment: bool,
    preserve_direct_page: bool,
    preserve_dialogue_asset_state: bool,
    preserve_overworld_decode_scratch: bool,
) -> Result<u16, Snes9xCpuQuiescentCheckpointError> {
    let registers = cpu.machine().snes().cpu.clone();
    let at = cpu.machine().timestamp();
    if preserve_stack_segment {
        // CPU-private stack, direct-page scratch and the original-ROM text
        // decoder cannot be replaced by translated game representations.
        let stack_start = usize::from(registers.sp) + 1;
        let stack_end = (usize::from(registers.sp) | 0xff) + 1;
        let mut private = vec![stack_start..stack_end];
        if preserve_direct_page {
            let start = usize::from(registers.dp);
            let end = start + 0x100;
            private.push(start..end.min(ram.len()));
            if end > 0x10000 {
                private.push(0..end - 0x10000);
            }
        }
        if preserve_dialogue_asset_state {
            private.push(DIALOGUE_MSG_SRC_OFFS..DIALOGUE_MSG_SRC_OFFS + 3);
            private.push(
                TEXT_DIALOGUE_POINTERS..TEXT_DIALOGUE_POINTERS + ROM_DIALOGUE_MESSAGE_COUNT * 3,
            );
        }
        if preserve_overworld_decode_scratch {
            // The ROM's in-flight bank-02 decode owns both staging buffers.
            // Translated C may hold another phase of the decompression/copy.
            private.push(OVERWORLD_MAP16_DECODE_SRC..OVERWORLD_MAP16_DECODE_SRC + 512);
            private.push(OVERWORLD_DECOMP_BUFFER..OVERWORLD_DECOMP_BUFFER + 256);
        }
        private.sort_by_key(|range| range.start);
        let mut regions = Vec::new();
        let mut copied_until = 0;
        for range in private {
            if range.start > copied_until {
                regions.push(copied_until..range.start);
            }
            copied_until = copied_until.max(range.end);
        }
        if copied_until < ram.len() {
            regions.push(copied_until..ram.len());
        }
        cpu.rebase_private_timing_regions_at(
            at,
            &registers,
            ram,
            sram,
            &regions,
            &[0..sram.len()],
        )?;
    } else {
        cpu.rebase_private_timing_state_at(at, &registers, ram, sram)?;
    }
    Ok(registers.sp)
}

impl ZeldaState {
    #[cfg(feature = "rom-timing")]
    fn native_exact_cpu_trial_ram(
        native_ram: &[u8],
        work: Option<GameWorkContinuation>,
        landing_spotlight_suspended: bool,
    ) -> std::borrow::Cow<'_, [u8]> {
        if !matches!(
            work,
            Some(GameWorkContinuation::FinishDungeonExitSpotlightEntry { .. }
                | GameWorkContinuation::FinishDungeonExitSpotlightBuild { .. }
                | GameWorkContinuation::FinishOverworldSpotlightBuild { .. })
        ) && !(landing_spotlight_suspended
            && work == Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn)) {
            return std::borrow::Cow::Borrowed(native_ram);
        }
        // The native builder owns two distinct generations: its working
        // table at $7F:DBA0 and its published table at $7F:7000. The ROM
        // addresses those at $7F:7000 and $7E:1B00 respectively. Relocate
        // native bytes only in the trial view; never replace either with a
        // source-CPU payload or collapse an unfinished copy into one table.
        let bytes = SPOTLIGHT_VISIBLE_SCANLINES * 2;
        let mut ram = native_ram.to_vec();
        ram[0x1b00..0x1b00 + bytes]
            .copy_from_slice(&native_ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + bytes]);
        ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + bytes]
            .copy_from_slice(&native_ram[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + bytes]);
        std::borrow::Cow::Owned(ram)
    }

    #[cfg(feature = "rom-timing")]
    fn reconcile_native_exact_cpu_landing_music_nmi(&mut self, trace: &NativeExactCpuHostTrace) {
        if !self.native_exact_cpu_landing_music_nmi_pending {
            return;
        }
        let Some(read) = trace.music_port_accesses.iter().find(|access| {
            access.pc == 0x00_80dc
                && access.address == 0x012c
                && access.kind == NativeExactCpuNmiGateKind::Read
                && access.width == 1
        }) else {
            return;
        };
        assert_eq!(
            read.value as u8,
            self.game_state.system_signals.music_control(),
            "the next source NMI music read disagrees with the native caller command",
        );
        // The carried handler's pre-main audio sample may not pass through a
        // second translated display interrupt to retire this flag. A distinct
        // source NMI reading the new command begins a new audio sample.
        self.audio_nmi_processed_before_main = false;
        self.native_exact_cpu_landing_music_nmi_pending = false;
    }

    pub(crate) fn note_native_exact_cpu_translated_nmi_completion(&mut self) {
        if let Some(owner) = self.native_exact_cpu_owner.as_mut() {
            owner.translated_nmi_handlers_completed += 1;
        }
    }


}

#[cfg(all(test, feature = "rom-timing"))]
mod tests {
    use super::*;

    #[test]
    fn link_oam_return_belongs_to_its_main_loop_iteration() {
        let mut trace = NativeExactCpuHostTrace {
            main_loop_entries: vec![
                NativeExactCpuMainLoopEntry { frame_counter: 255, at: 100 },
                NativeExactCpuMainLoopEntry { frame_counter: 0, at: 300 },
            ],
            // The first return belongs to a carried caller, before either entry.
            module07_link_oam_returns: vec![50, 200],
            ..Default::default()
        };
        assert!(trace.module07_caller_passed_link_without_nmi(255));
        assert!(!trace.module07_caller_passed_link_without_nmi(0));
        assert!(!trace.module07_caller_passed_link_without_nmi(254));
        trace.module07_link_oam_returns.push(400);
        assert!(trace.module07_caller_passed_link_without_nmi(0));
        // An actual trailing NMI still owns display publication and cannot
        // be discarded just because the caller reached LinkOam earlier.
        trace.nmis.push(NativeExactCpuNmi { interrupted_pc: 0x0d_fcd1, accepted_at: 450 });
        assert!(!trace.module07_caller_passed_link_without_nmi(0));
        assert!(trace.module07_caller_passed_link_without_nmi(255));
        trace.nmis[0].accepted_at = 350;
        assert!(!trace.module07_caller_passed_link_without_nmi(0));
        trace.nmis[0].accepted_at = 75;
        assert!(trace.module07_caller_passed_link_without_nmi(0));
        trace.main_loop_entries.clear();
        assert!(!trace.module07_caller_passed_link_without_nmi(0));
    }

    #[test]
    fn accepted_nmi_carries_its_handler_to_the_next_main_iteration() {
        let accepted = NativeExactCpuNmi {
            interrupted_pc: 0x00_8036,
            accepted_at: 100,
        };
        let mut first = NativeExactCpuHostTrace::default();
        first.nmis.push(accepted);
        first.pending_nmis_at_return.push(accepted);
        assert!(first.accepted_nmi_carries_out());
        assert!(!first.leading_nmi_completes_before_main());

        let mut next = NativeExactCpuHostTrace::default();
        next.nmi_completions.push(NativeExactCpuNmiCompletion {
            accepted_at: accepted.accepted_at,
            returned_at: 130,
        });
        next.main_loop_entries.push(NativeExactCpuMainLoopEntry {
            frame_counter: 1,
            at: 150,
        });
        assert!(next.leading_nmi_completes_before_main());
        assert!(!next.accepted_nmi_carries_out());
        next.nmis.push(accepted);
        assert!(next.leading_nmi_completes_before_main());
    }

    #[test]
    fn fresh_leading_nmi_is_distinct_from_the_next_pending_acceptance() {
        let leading = NativeExactCpuNmi { interrupted_pc: 0x00_8034, accepted_at: 100 };
        let trailing = NativeExactCpuNmi { interrupted_pc: 0x00_8036, accepted_at: 1000 };
        let mut trace = NativeExactCpuHostTrace {
            nmis: vec![leading, trailing],
            nmi_completions: vec![NativeExactCpuNmiCompletion { accepted_at: 100, returned_at: 200 }],
            pending_nmis_at_return: vec![trailing],
            main_loop_entries: vec![NativeExactCpuMainLoopEntry { frame_counter: 7, at: 250 }],
            ..Default::default()
        };
        assert!(trace.leading_nmi_completes_before_main());
        assert!(trace.accepted_nmi_carries_out());
        // A later interrupt must not be moved before an already-started main.
        trace.main_loop_entries[0].at = 50;
        assert!(!trace.leading_nmi_completes_before_main());
        trace.main_loop_entries.clear();
        assert!(!trace.leading_nmi_completes_before_main());
    }

    #[test]
    fn cached_restore_cursor_matches_rom_stores_at_every_instruction() {
        let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../saves/zelda3.sfc"));
        let Ok(mut rom) = std::fs::read(path) else {
            eprintln!("cached restore reference skipped: no development ROM");
            return;
        };
        if rom.len() % 0x400 == 0x200 { rom.drain(..0x200); }
        let mut pc = 0x1d_eb06u32;
        let mut fields = super::super::CACHED_SPRITE_LIVE_FIELDS.iter().rev();
        let mut remaining = 24;
        loop {
            assert_eq!(NativeExactCpuCachedSpriteTracker::restore_live_fields_at_pc(pc),
                Some(remaining), "next instruction {pc:06x}");
            let offset = ((pc >> 16) * 0x8000 + (pc & 0x7fff)) as usize;
            match rom[offset] {
                0x68 => pc += 1, // PLA: the live field is still pending.
                opcode @ (0x9d | 0x9f) => {
                    let address = u16::from_le_bytes([rom[offset + 1], rom[offset + 2]]) as usize;
                    let address = if opcode == 0x9f {
                        address + ((usize::from(rom[offset + 3]) & 1) << 16)
                    } else { address };
                    assert_eq!(fields.next().copied(), Some(address));
                    remaining -= 1;
                    pc += if opcode == 0x9f { 4 } else { 3 };
                }
                0x60 => { assert_eq!(remaining, 0); break; }
                opcode => panic!("unexpected restore opcode {opcode:02x} at {pc:06x}"),
            }
        }
        assert!(fields.next().is_none());
        assert_eq!(NativeExactCpuCachedSpriteTracker::restore_live_fields_at_pc(pc + 1), None);
    }

    #[test]
    fn dungeon_cache_cursor_keeps_source_field_order_across_slots() {
        let mut tracker = NativeExactCpuDungeonCacheTracker::default();
        tracker.observe_store(
            15,
            CachedSpriteCacheField::StateClear.alt_address() as u16 + 15,
        );
        for field in CachedSpriteCacheField::C_SOURCE_ORDER.iter().take(14) {
            tracker.observe_store(3, field.alt_address() as u16 + 3);
        }
        assert_eq!(
            tracker.progress_at_return(0x09_c1dd, OriginalTimingBoundary::NmiAccepted),
            Some(DungeonResetSpritesProgressReceipt {
                progress: DungeonResetSpritesCpuProgress::Cache {
                    slot: 3,
                    field: CachedSpriteCacheField::Flags2,
                },
                boundary: OriginalTimingBoundary::NmiAccepted,
            }),
        );
        assert_eq!(
            tracker.progress_at_return(0x00_8034, OriginalTimingBoundary::HostReturn),
            None,
        );
    }

    #[test]
    #[should_panic(expected = "cache store skipped or reordered a source field")]
    fn dungeon_cache_cursor_rejects_missing_source_stores() {
        let mut tracker = NativeExactCpuDungeonCacheTracker::default();
        tracker.observe_store(3, CachedSpriteCacheField::Type.alt_address() as u16 + 3);
    }

    #[test]
    fn word_read_ownership_uses_the_differing_byte_writer() {
        let source = NativeExactCpuRead {
            pc: 0x00_e1b0, address: 0x00_0aa3, value: 0x0a45, width: 2, at: 300,
        };
        let trial = NativeExactCpuRead { value: 0x0145, ..source };
        let low = NativeExactCpuWrite {
            pc: 0x01_b64d, address: 0x01_0aa3, value: 0x45, width: 1, at: 100,
        };
        let high = NativeExactCpuWrite {
            pc: 0x02_8283, address: 0x02_0aa4, value: 0x0a, width: 1, at: 200,
        };
        let (source_byte, trial_byte) = native_exact_cpu_differing_read_bytes(source, trial);
        assert_eq!((source_byte.address, source_byte.value, source_byte.width), (0xaa4, 10, 1));
        assert_eq!((trial_byte.address, trial_byte.value, trial_byte.width), (0xaa4, 1, 1));
        let mut writers = HashMap::new();
        native_exact_cpu_record_writers(&mut writers, &[low], 7);
        native_exact_cpu_record_writers(&mut writers, &[high], 8);
        assert_eq!(native_exact_cpu_last_writer(source_byte, &[], &writers, 9)
            .map(|w| (w.host, w.write)), Some((8, high)));
        assert_eq!(native_exact_cpu_differing_read_bytes(source, source), (source, source));
    }

    #[test]
    fn cpu_writer_lookup_resolves_wram_mirrors_before_rebase() {
        let write = NativeExactCpuWrite {
            pc: 0x06_84c4,
            address: 0x00_0fd8,
            value: 0x30,
            width: 1,
            at: 100,
        };
        let read = NativeExactCpuRead {
            pc: 0x06_e423,
            address: 0x05_0fd8,
            value: 0x0130,
            width: 2,
            at: 200,
        };
        let mut writers = HashMap::new();
        native_exact_cpu_record_writers(&mut writers, &[write], 12435);
        assert_eq!(
            native_exact_cpu_last_writer(read, &[], &writers, 12436).map(|w| w.host),
            Some(12435),
        );
        assert_eq!(
            native_exact_cpu_last_writer(read, &[write], &writers, 12436).map(|w| w.host),
            Some(12436),
        );
    }

    #[test]
    fn cpu_writer_lookup_finds_high_byte_of_a_word_store() {
        let write = NativeExactCpuWrite {
            pc: 0x02_8813, address: 0x02_0124, value: 0x0a0b, width: 2, at: 100,
        };
        let read = NativeExactCpuRead {
            pc: 0x00_819a, address: 0x00_0125, value: 0x0a, width: 1, at: 200,
        };
        let mut writers = HashMap::new();
        native_exact_cpu_record_writers(&mut writers, &[write], 7);
        assert_eq!(native_exact_cpu_last_writer(read, &[], &writers, 8)
            .map(|w| (w.host, w.write)), Some((7, write)));
        assert_eq!(native_exact_cpu_last_writer(read, &[write], &writers, 8)
            .map(|w| (w.host, w.write)), Some((8, write)));
        let byte_write = NativeExactCpuWrite { address: 0x7e_0125, value: 3,
            width: 1, at: 150, ..write };
        native_exact_cpu_record_writers(&mut writers, &[byte_write], 8);
        assert_eq!(native_exact_cpu_last_writer(read, &[], &writers, 9)
            .map(|w| w.write), Some(byte_write));
    }
    use snes::{
        CpuMasterTimestamp, SourceCpuBusAccess, SourceCpuTransaction, SourceCpuTransactionKind,
    };

    fn scan_step(pc: u32, write: Option<(u32, u16)>, at: u64) -> SourceCpuStepReceipt {
        SourceCpuStepReceipt {
            origin_pc: pc,
            opcode: 0xea,
            started_at: CpuMasterTimestamp::new(at),
            ended_at: CpuMasterTimestamp::new(at + 6),
            accesses: write
                .map(|(address, value)| SourceCpuBusAccess {
                    address,
                    timestamp: CpuMasterTimestamp::new(at + 4),
                    charged_master_cycles: 6,
                    kind: SourceCpuBusAccessKind::Write { value, width: 1 },
                })
                .into_iter()
                .collect(),
            transactions: Vec::new(),
            accepted_interrupt: None,
        }
    }

    #[test]
    fn uncle_priest_dispatch_witness_ends_at_the_indirect_jump() {
        let mut trace = NativeExactCpuHostTrace::default();
        trace.observe_uncle_priest_dispatch(&scan_step(0x00_8781, None, 0), 3);
        assert_eq!(trace.sprite_timer_return_slot, None);
        let mut call = scan_step(0x05_db91, None, 10);
        call.opcode = 0x22;
        trace.observe_uncle_priest_dispatch(&call, 3);
        for pc in [0x00_8781, 0x00_8797, 0x00_80c9, 0x00_82dc] {
            trace.observe_uncle_priest_dispatch(&scan_step(pc, None, 20), 12);
            assert_eq!(trace.sprite_timer_return_slot, Some(3));
        }
        trace.observe_uncle_priest_dispatch(&scan_step(0x00_8799, None, 30), 3);
        assert_eq!(trace.sprite_timer_return_slot, None);
    }

    #[test]
    fn follower_graphics_cursor_survives_interrupts_and_counts_both_sheets_and_planes() {
        use crate::RescuedMaidenInitializationStage::{Conversion, FirstFollowerSheet, SecondFollowerSheet};
        let mut tracker = NativeExactCpuFollowerGraphicsTracker::default();
        tracker.observe(&scan_step(0x00_e7f2, Some((0x7f_4600, 0xc6)), 0), 9);
        assert!(tracker.active.is_none());
        let mut entry = scan_step(0x05_ebf2, None, 0);
        entry.opcode = 0x22;
        tracker.observe(&entry, 1);
        for (base, transition) in [(0x7f_4600, 0x00_d445), (0x7f_4000, 0x00_d44a)] {
            for byte in 0..1536 {
                tracker.observe(&scan_step(0x00_e7f2, Some((base + byte, 0xc6)), 10), 9);
                // Host returns inside NMI and changing CPU scratch registers
                // cannot erase or reassign the suspended caller's progress.
                tracker.observe(&scan_step(0x00_80c9, None, 20), 15);
                tracker.observe(&scan_step(0x00_82dc, None, 30), 15);
                let stage = if base == 0x7f_4600 {
                    FirstFollowerSheet { completed_bytes: (byte + 1) as u16 }
                } else {
                    SecondFollowerSheet { completed_bytes: (byte + 1) as u16 }
                };
                assert_eq!(tracker.active, Some((1, stage)));
            }
            tracker.observe(&scan_step(transition, None, 40), 9);
        }
        // Enumerate tiles and rows independently of the tracker's ordinal formula.
        for tile in 0..32 {
            for row in 0..8 {
                for plane in 0..2 {
                    let address = 0x7e_b940 + tile * 32 + row * 2 + plane * 16;
                    let pc = match (row % 2, plane) {
                        (0, 0) => 0x00_d5dd,
                        (0, 1) => 0x00_d5ea,
                        (1, 0) => 0x00_d5f4,
                        _ => 0x00_d601,
                    };
                    let mut step = scan_step(pc, Some((address, 0xabcd)), 50);
                    step.accesses[0].kind = SourceCpuBusAccessKind::Write { value: 0xabcd, width: 2 };
                    tracker.observe(&step, 9);
                }
            }
        }
        assert_eq!(tracker.active, Some((1, Conversion { completed_stores: 512 })));
        tracker.observe(&scan_step(0x05_ebf6, None, 60), 9);
        assert!(tracker.active.is_none());
    }

    #[test]
    #[should_panic(expected = "follower decompression stores must be contiguous")]
    fn follower_graphics_cursor_rejects_skipped_writes() {
        let mut tracker = NativeExactCpuFollowerGraphicsTracker {
            active: Some((1, crate::RescuedMaidenInitializationStage::SecondFollowerSheet {
                completed_bytes: 32,
            })),
        };
        tracker.observe(&scan_step(0x00_e7f2, Some((0x7f_4021, 0)), 0), 1);
    }

    #[test]
    fn vwf_checkpoint_waits_for_the_native_timing_anchor_to_be_released() {
        use crate::zelda_rtl::messaging::VwfGlyphCpuPhase;
        let mut state = ZeldaState::new();
        assert!(state.native_exact_cpu_dialogue_checkpoint_is_quiescent());
        state.dialogue_vwf_glyph_cpu_phase = VwfGlyphCpuPhase::PreparingDrawingPublished {
            remaining_master_cycles: 150, drawing_master_cycles: 52398, cursor_advanced: true,
        };
        assert!(!state.native_exact_cpu_dialogue_checkpoint_is_quiescent());
        state.dialogue_vwf_glyph_cpu_phase = VwfGlyphCpuPhase::Ready;
        state.dialogue_vwf_transition_remaining_master_cycles = 128;
        assert!(!state.native_exact_cpu_dialogue_checkpoint_is_quiescent());
        state.dialogue_vwf_transition_remaining_master_cycles = 0;
        state.dialogue_fast_forward_hold_pending = true;
        assert!(!state.native_exact_cpu_dialogue_checkpoint_is_quiescent());
        state.dialogue_fast_forward_hold_pending = false;
        state.dialogue_fast_forward_hold_active = true;
        assert!(!state.native_exact_cpu_dialogue_checkpoint_is_quiescent());
        state.dialogue_fast_forward_hold_active = false;
        assert!(state.native_exact_cpu_dialogue_checkpoint_is_quiescent());
    }

    #[test]
    fn animated_dungeon_tracker_retains_sheet_plane_and_rotation_boundaries() {
        use AnimatedDungeonGraphicsProgress::*;
        let mut tracker = NativeExactCpuAnimatedDungeonGraphicsTracker::default();
        assert_eq!(tracker.observe(&scan_step(0x00_e7f2, Some((0x7f_4000, 3)), 0)), None);
        assert_eq!(tracker.observe(&scan_step(0x00_d337, None, 0)), Some(FirstSheet(0)));
        assert_eq!(tracker.observe(&scan_step(0x00_e79e, Some((0xcd, 3)), 0)), Some(FirstSheet(0)),
            "decompression scratch stores must not publish graphics bytes");
        for (first, convert, next, base) in [(true, 0x00_d33d, 0x00_d34a, 0x7e_a680),
            (false, 0x00_d351, 0x00_d35e, 0x7e_ac80)] {
            for byte in 0..1536u16 {
                tracker.observe(&scan_step(0x00_e7f2, Some((0x7f_4000 + u32::from(byte), 0)), 0));
            }
            assert_eq!(tracker.active, Some(if first { FirstSheet(1536) } else { SecondSheet(1536) }));
            tracker.observe(&scan_step(convert, None, 0));
            for store in 0..768u16 {
                let row = u32::from(store / 2);
                let pc = [0x00_d5dd, 0x00_d5ea, 0x00_d5f4, 0x00_d601][usize::from(store % 4)];
                let address = base + (row / 8) * 32 + (row % 8) * 2 + u32::from(store % 2) * 16;
                let mut step = scan_step(pc, Some((address, 0)), 0);
                step.accesses[0].kind = SourceCpuBusAccessKind::Write { value: 0xffff, width: 2 };
                tracker.observe(&step);
                let expected = Some(if first { FirstConversion(store + 1) } else { SecondConversion(store + 1) });
                assert_eq!(tracker.observe(&scan_step(0x00_80c9, None, 0)), expected);
            }
            tracker.observe(&scan_step(next, None, 0));
        }
        for store in 0..1024u16 {
            let lane = usize::from(store % 4);
            let pc = [0x00_d36a, 0x00_d372, 0x00_d37a, 0x00_d37f][lane];
            let address = [0x7e_a880, 0x7e_ac80, 0x7e_ae80, 0x7e_aa80][lane] + u32::from(store / 4) * 2;
            let mut step = scan_step(pc, Some((address, 0)), 0);
            step.accesses[0].kind = SourceCpuBusAccessKind::Write { value: 0xabcd, width: 2 };
            step.accesses.extend(scan_step(pc, Some((0x01ff, 0x7e)), 0).accesses);
            assert_eq!(tracker.observe(&step), Some(Rotation(store + 1)));
            assert_eq!(tracker.observe(&scan_step(0x00_80c9, None, 0)), Some(Rotation(store + 1)));
        }
        tracker.observe(&scan_step(0x00_d38d, None, 0));
        assert_eq!(tracker.active, Some(DestinationPublished));
        assert_eq!(tracker.observe(&scan_step(0x00_d393, None, 0)), Some(Returned));
        assert!(tracker.active.is_none());
    }

    #[test]
    fn animated_overworld_tracker_preserves_both_sheets_and_conversion_stores_across_nmi() {
        use AnimatedOverworldGraphicsProgress::*;
        let mut tracker = NativeExactCpuAnimatedOverworldGraphicsTracker::default();
        assert_eq!(tracker.observe(&scan_step(0x00_d394, None, 0)), Some(FirstSheet(0)));
        for (first, convert, next, tiles, base) in [
            (true, 0x00_d39b, 0x00_d3a8, 64, 0x7e_a680),
            (false, 0x00_d3af, 0x00_d3c5, 32, 0x7e_ae80),
        ] {
            for byte in 0..1536 {
                tracker.observe(&scan_step(0x00_e81c, Some((0x7f_4000 + byte, 0xa5)), 0));
                tracker.observe(&scan_step(0x00_80c9, None, 1));
            }
            assert_eq!(tracker.active, Some(if first { FirstSheet(1536) } else { SecondSheet(1536) }));
            tracker.observe(&scan_step(convert, None, 2));
            for tile in 0..tiles {
                for row in 0..8 {
                    for plane in 0..2 {
                        let mut step = scan_step(0x00_d5dd, Some((base + tile * 32 + row * 2 + plane * 16, 0x1234)), 3);
                        step.accesses[0].kind = SourceCpuBusAccessKind::Write { value: 0x1234, width: 2 };
                        tracker.observe(&step);
                        tracker.observe(&scan_step(0x00_82dc, None, 4));
                    }
                }
            }
            let result = tracker.observe(&scan_step(next, None, 5));
            assert_eq!(result, Some(if first { SecondSheet(0) } else { Returned }));
        }
        assert!(tracker.active.is_none());
    }

    #[test]
    fn pre_dungeon_palette_tracker_preserves_cache_columns_and_update_tail() {
        let mut tracker = NativeExactCpuPreDungeonPaletteTracker::default();
        assert_eq!(tracker.observe(&scan_step(0x02_828f, None, 0)), None);
        assert!(tracker.active);
        assert_eq!(tracker.observe(&scan_step(0x1b_ef3d, Some((0x7e_c4ce, 0xad)), 0)), None);
        assert_eq!(tracker.observe(&scan_step(0x02_c769, None, 0)), Some(0));
        for ordinal in 0..256u16 {
            let pc = 0x02_c771 + u32::from(ordinal % 8) * 8;
            let address = 0x7e_c500 + u32::from(ordinal % 8) * 64 + u32::from(ordinal / 8) * 2;
            let mut step = scan_step(pc, Some((address, 0)), 0);
            step.accesses[0].kind = SourceCpuBusAccessKind::Write { value: 0x1234, width: 2 };
            step.accesses.extend(scan_step(pc, Some((0x01ff, 0)), 0).accesses);
            assert_eq!(tracker.observe(&step), Some(ordinal + 1));
            assert_eq!(tracker.observe(&scan_step(0x00_80c9, None, 0)), Some(ordinal + 1));
        }
        assert_eq!(tracker.observe(&scan_step(0x02_c7b5, Some((0x15, 1)), 0)), Some(257));
        assert_eq!(tracker.observe(&scan_step(0x02_8292, None, 0)), Some(257));
        assert!(!tracker.active);
    }

    #[test]
    fn pre_overworld_tileset_tracker_keeps_each_decompression_cursor_across_hosts() {
        for (entered_pc, returned_pc) in [(0x02_8492, 0x02_8496), (0x02_8286, 0x02_828a)] {
            let mut tracker = NativeExactCpuTilesetTracker::default();
            let mut events = Vec::new();
            tracker.observe_call(&scan_step(entered_pc, None, 0), 0, &mut events, entered_pc, returned_pc);
            assert!(matches!(events.as_slice(), [TilesetGraphicsPublication::Entered]));
            for index in 0..12 {
                let entry = if index < 4 { 0x00_e772 } else { 0x00_e78f };
                let base = match index { 0..=3 => 0x7e_7800 + index * 1536,
                    7..=10 => 0x7e_6000 + (index - 7) * 1536, _ => 0x7f_4000 };
                events.clear();
                tracker.observe_call(&scan_step(entry, None, 0), 70 + index as u16, &mut events, entered_pc, returned_pc);
                for byte in 0..1536 {
                    tracker.observe_call(&scan_step(0x00_e7f2, Some((base + byte, 0x55)), 0), 0, &mut events, entered_pc, returned_pc);
                    if byte == 644 {
                        assert!(matches!(events.last(), Some(TilesetGraphicsPublication::Sheet { completed_bytes: 645, .. })));
                        events.clear(); // host boundary does not reset the source call cursor
                        tracker.observe_call(&scan_step(0x00_80c9, None, 0), 0, &mut events, entered_pc, returned_pc);
                    }
                }
                assert!(matches!(events.as_slice(), [TilesetGraphicsPublication::Sheet { completed_bytes: 1536, .. }]));
            }
            tracker.observe_call(&scan_step(returned_pc, None, 0), 0, &mut events, entered_pc, returned_pc);
            assert!(!tracker.active);
            assert!(matches!(events.last(), Some(TilesetGraphicsPublication::Returned)));
        }
    }

    #[test]
    fn map8_tracker_retains_each_store_and_row_tail_across_nmi() {
        let mut tracker = NativeExactCpuOverworldMap8Tracker::default();
        assert_eq!(tracker.observe(&scan_step(0x02_fd46, None, 0)), None);
        assert_eq!(tracker.observe(&scan_step(0x02_fd4e, None, 1)), Some(0));
        for row in 0..32u16 {
            for step in 0..164u16 {
                let (pc, address) = match step {
                    0..32 => (if step % 2 == 0 { 0x02_fd9c } else { 0x02_fdaf }, 0x0f_0500),
                    32 | 97 => (0x02_fdff, 0x7f_4000),
                    162 => (if row % 2 == 0 { 0x02_fd62 } else { 0x02_fd75 }, 0x00_0084),
                    163 => (if row % 2 == 0 { 0x02_fd6a } else { 0x02_fd7d }, 0x00_0088),
                    _ => ([0x02_fe1e, 0x02_fe25, 0x02_fe2e, 0x02_fe35][usize::from(((step - 32) % 65 - 1) % 4)], 0x7f_2000),
                };
                let mut receipt = scan_step(pc, Some((address, 0x55)), 10);
                for access in &mut receipt.accesses {
                    if let SourceCpuBusAccessKind::Write { width, .. } = &mut access.kind { *width = 2; }
                }
                let expected = Some(row * 164 + step + 1);
                assert_eq!(tracker.observe(&receipt), expected);
                if step % 17 == 0 {
                    assert_eq!(tracker.observe(&scan_step(0x00_80c9, None, 20)), expected);
                }
            }
        }
        assert_eq!(tracker.observe(&scan_step(0x02_fd86, None, 30)), Some(5248));
        assert!(!tracker.active);
    }

    #[test]
    fn quadrant_tracker_retains_store_progress_across_all_four_returns_and_nmi() {
        let mut tracker = NativeExactCpuOverworldQuadrantTracker::default();
        assert_eq!(tracker.observe(&scan_step(0x02_f54a, None, 0)), Some(0));
        for store in 0..4096u16 {
            let pc = [0x02_f7b5, 0x02_f7ba, 0x02_f7c1, 0x02_f7c6][usize::from(store % 4)];
            let mut receipt = scan_step(pc, Some((0x7e_2000, 0x55)), 1);
            for access in &mut receipt.accesses {
                if let SourceCpuBusAccessKind::Write { width, .. } = &mut access.kind { *width = 2; }
            }
            assert_eq!(tracker.observe(&receipt), Some(store + 1));
            assert_eq!(tracker.observe(&scan_step(0x00_80c9, None, 2)), Some(store + 1));
            if store % 1024 == 1023 {
                assert_eq!(tracker.observe(&scan_step(0x02_f637, None, 3)), Some(store + 1));
                assert_eq!(tracker.active, store != 4095);
            }
        }
        assert_eq!(tracker.observe(&scan_step(0x00_80c9, None, 4)), None);
    }

    #[test]
    fn initial_scroll_tracker_counts_both_axes_and_caller_coordinate_corrections() {
        for horizontal in [true, false] {
            for stripes in [7, 8] {
                let mut tracker = NativeExactCpuInitialScrollTracker::default();
                assert_eq!(tracker.observe(&scan_step(0x02_f031, None, 0)), None);
                let mut pcs = vec![if horizontal { 0x02_efc5 } else { 0x02_ef8c }];
                for _ in 0..stripes {
                    for _ in 0..16 {
                        pcs.extend(if horizontal { [0x02_f3dd, 0x02_f3ef] } else { [0x02_f4a7, 0x02_f4b5] });
                    }
                    for half in 0..2 {
                        if horizontal { pcs.extend([0x02_f43b, 0x02_f43f]); }
                        else { pcs.push(if half == 0 { 0x02_f4f1 } else { 0x02_f501 }); }
                        for _ in 0..16 {
                            pcs.extend(if horizontal { [0x02_f45a, 0x02_f461, 0x02_f46a, 0x02_f471] }
                                else { [0x02_f522, 0x02_f529, 0x02_f532, 0x02_f539] });
                        }
                    }
                    if horizontal { pcs.extend([0x02_efcf, 0x02_efd1, 0x02_efd9]); }
                    else { pcs.extend([0x02_ef9c, 0x02_efa4]); }
                }
                pcs.push(if horizontal { 0x02_efe4 } else { 0x02_efaf });
                if stripes == 8 {
                    pcs.extend(if horizontal { [0x02_f102, 0x02_f10a] } else { [0x02_f0b3, 0x02_f0bb] });
                }
                for (i, pc) in pcs.iter().copied().enumerate() {
                    let mut receipt = scan_step(pc, Some((0x001100, 0x55)), 1);
                    for access in &mut receipt.accesses {
                        if let SourceCpuBusAccessKind::Write { width, .. } = &mut access.kind { *width = 2; }
                    }
                    assert_eq!(tracker.observe(&receipt), Some(i as u16));
                    assert_eq!(tracker.observe(&scan_step(0x00_80c9, None, 2)), Some(i as u16));
                }
                let pc = match (horizontal, stripes) {
                    (true, 7) => 0x02_f0db, (true, 8) => 0x02_f10e,
                    (false, 7) => 0x02_f086, _ => 0x02_f0bf,
                };
                assert_eq!(tracker.observe(&scan_step(pc, None, 3)), Some(pcs.len() as u16 - 1));
                assert!(!tracker.active);
            }
        }
    }

    #[test]
    fn checkpoint_capture_refuses_a_retained_trial_from_an_earlier_host() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../saves/zelda3.sfc");
        let Ok(rom) = std::fs::read(path) else { return };
        let mut owner = NativeExactCpuOwner::new(&rom, &[0; 8192]).unwrap();
        owner.translated_trial = Some(Box::new(owner.clone()));
        // Resuming without the contiguous trial enabled advances only the
        // baseline. Such an owner must never emit a sidecar restore refuses.
        owner.next_host += 1;
        let mut game = ZeldaState::new();
        game.frame_ctr_dbg = owner.next_host;
        game.native_exact_cpu_owner = Some(owner);
        assert!(!game.native_exact_cpu_checkpoint_available());
        assert!(game.capture_native_exact_cpu_checkpoint().unwrap_err()
            .contains("different next host"));
        let owner = game.native_exact_cpu_owner.as_mut().unwrap();
        owner.translated_trial.as_mut().unwrap().next_host = owner.next_host;
        assert!(game.native_exact_cpu_checkpoint_available());
        let bytes = game.capture_native_exact_cpu_checkpoint().unwrap().unwrap();
        game.restore_native_exact_cpu_checkpoint(&bytes).unwrap();
    }

    #[test]
    fn follower_graphics_checkpoint_requires_both_cpu_owners_to_be_idle() {
        let path = std::env::var_os("ZELDA3_ROM").map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../saves/zelda3.sfc"));
        let Ok(rom) = std::fs::read(path) else { return };
        let mut owner = NativeExactCpuOwner::new(&rom, &[0; 8192]).unwrap();
        let progress = Some((1, crate::RescuedMaidenInitializationStage::SecondFollowerSheet {
            completed_bytes: 863,
        }));
        owner.follower_graphics.active = progress;
        assert!(!owner.follower_graphics_is_quiescent());
        assert!(owner.capture_checkpoint().err().unwrap().contains("active follower graphics"));
        let trial = owner.clone();
        assert_eq!(trial.follower_graphics.active, progress);
        owner.follower_graphics.active = None;
        owner.translated_trial = Some(Box::new(trial));
        assert!(!owner.follower_graphics_is_quiescent());
        assert!(owner.capture_checkpoint().err().unwrap().contains("active follower graphics"));
        owner.translated_trial.as_mut().unwrap().follower_graphics.active = None;
        owner.animated_overworld_graphics.active = Some(AnimatedOverworldGraphicsProgress::FirstSheet(645));
        assert!(!owner.follower_graphics_is_quiescent());
        assert!(owner.capture_checkpoint().is_err());
        owner.animated_overworld_graphics.active = None;
        owner.translated_trial.as_mut().unwrap().pre_overworld_tilesets.active = true;
        assert!(!owner.follower_graphics_is_quiescent());
        assert!(owner.capture_checkpoint().is_err());
        owner.translated_trial.as_mut().unwrap().pre_overworld_tilesets.active = false;
        owner.pre_dungeon_tilesets.active = true;
        assert!(!owner.follower_graphics_is_quiescent());
        assert!(owner.capture_checkpoint().is_err());
        owner.pre_dungeon_tilesets.active = false;
        owner.pre_dungeon_palettes.active = true;
        assert!(!owner.follower_graphics_is_quiescent());
        assert!(owner.capture_checkpoint().is_err());
        owner.pre_dungeon_palettes.active = false;
        owner.overworld_map8.active = true;
        assert!(!owner.follower_graphics_is_quiescent());
        assert!(owner.capture_checkpoint().is_err());
        owner.overworld_map8.active = false;
        owner.overworld_quadrants.active = true;
        assert!(!owner.follower_graphics_is_quiescent());
        assert!(owner.capture_checkpoint().is_err());
        owner.overworld_quadrants.active = false;
        owner.initial_scroll.active = true;
        assert!(!owner.follower_graphics_is_quiescent());
        assert!(owner.capture_checkpoint().is_err());
        owner.initial_scroll.active = false;
        owner.sprite_coordinates.observe(0x06_83f2, 13);
        assert!(!owner.follower_graphics_is_quiescent());
        assert!(owner.capture_checkpoint().is_err());
        owner.sprite_coordinates.observe(0x06_84d9, 13);
        owner.animated_dungeon_graphics.active = Some(AnimatedDungeonGraphicsProgress::Rotation(1));
        assert!(!owner.follower_graphics_is_quiescent());
        assert!(owner.capture_checkpoint().is_err());
        owner.animated_dungeon_graphics.active = None;
        assert!(owner.follower_graphics_is_quiescent());
        let restored = NativeExactCpuOwner::from_checkpoint(owner.capture_checkpoint().unwrap()).unwrap();
        assert!(restored.follower_graphics_is_quiescent());
        assert!(restored.translated_trial.is_some());
        let mut state = ZeldaState::new();
        state.frame_ctr_dbg = restored.next_host;
        state.native_exact_cpu_owner = Some(restored);
        assert!(state.native_exact_cpu_checkpoint_available());
        state.native_pre_overworld_animated_graphics = Some(AnimatedOverworldGraphicsProgress::Returned);
        assert!(!state.native_exact_cpu_checkpoint_available());
        assert!(state.capture_native_exact_cpu_checkpoint().is_err());
        state.native_pre_overworld_animated_graphics = None;
        state.native_pre_dungeon_animated_graphics = Some(AnimatedDungeonGraphicsContinuation::default());
        assert!(!state.native_exact_cpu_checkpoint_available());
        assert!(state.capture_native_exact_cpu_checkpoint().is_err());
        state.native_pre_dungeon_animated_graphics = None;
        state.begin_native_exact_cpu_overworld_live_scan(0);
        assert!(!state.native_exact_cpu_checkpoint_available());
        assert!(state.capture_native_exact_cpu_checkpoint().err().unwrap().contains("active native overworld scan"));
        state.native_exact_cpu_overworld_live_scan = None;
        assert!(state.native_exact_cpu_checkpoint_available());
        state.native_overworld_map8 = Some(state.begin_overworld_map8_cursor(
            super::super::OverworldMap16SourcePage::Overlay, 0x1000,
        ));
        assert!(!state.native_exact_cpu_checkpoint_available());
        assert!(state.capture_native_exact_cpu_checkpoint().is_err());
        state.native_overworld_map8 = None;
        state.native_overworld_quadrants = Some(state.begin_overworld_quadrant_cursor());
        assert!(!state.native_exact_cpu_checkpoint_available());
        assert!(state.capture_native_exact_cpu_checkpoint().is_err());
        state.native_overworld_quadrants = None;
        state.native_initial_scroll = Some(super::super::overworld::NativeInitialScrollCursor::checkpoint_test_cursor());
        assert!(!state.native_exact_cpu_checkpoint_available());
        assert!(state.capture_native_exact_cpu_checkpoint().is_err());
        state.native_initial_scroll = None;
        state.native_overworld_spotlight_link_oam = Some(state.link_oam_before_equipment());
        assert!(!state.native_exact_cpu_checkpoint_available());
        assert!(state.capture_native_exact_cpu_checkpoint().is_err());
    }

    #[test]
    fn coordinate_call_provenance_survives_interrupts_and_excludes_other_callers() {
        let mut tracker = NativeExactCpuSpriteCoordinateTracker::default();
        for instruction in 0..=8 {
            assert_eq!(tracker.prefix(0x06_84c1 + instruction * 3), None);
        }
        for slot in [15, 13, 0] {
            tracker.observe(0x06_83f2, slot);
            for instruction in 0..=8 {
                let pc = 0x06_84c1 + instruction * 3;
                assert_eq!(tracker.prefix(pc), Some((slot, instruction as u8)));
                // The interrupt's handler registers need not retain X/slot.
                for handler_pc in [0x00_80c9, 0x00_811c, 0x00_822c] {
                    tracker.observe(handler_pc, 0xff);
                    assert_eq!(tracker.prefix(handler_pc), None);
                }
                assert_eq!(tracker.prefix(pc), Some((slot, instruction as u8)));
                assert_eq!(tracker.prefix(pc + 1), None);
            }
            tracker.observe(0x06_84d9, slot);
            assert_eq!(tracker.prefix(0x06_84c1), None);
        }
    }

    #[test]
    fn module07_call_returns_are_cpu_statements_not_nmi_counts() {
        let mut stack = Vec::new();
        let mut scan = NativeExactCpuOverworldScanTracker::default();
        let mut sprite = NativeExactCpuSpritePreparationTracker::default();
        let mut scroll = NativeExactCpuScrollCopyTracker::default();
        let mut floor = NativeExactCpuFloorDrawTracker::default();
        let mut trace = NativeExactCpuHostTrace::default();
        for pc in [0x01_88e4, 0x02_8a5f, 0x02_8a67] {
            NativeExactCpuOwner::record_step(
                &mut stack,
                &mut scan,
                &mut sprite,
                &mut scroll,
                &mut floor,
                &mut trace,
                &scan_step(pc, None, 10),
                None,
                None,
                None,
            );
            assert_eq!(trace.room_load_returned, pc >= 0x02_8a5f);
            assert_eq!(trace.auxiliary_sprite_graphics_returned, pc == 0x02_8a67);
        }
        assert!(trace.nmis.is_empty());
    }

    #[test]
    fn spotlight_builder_events_follow_source_statement_order() {
        let mut stack = Vec::new();
        let mut scan = NativeExactCpuOverworldScanTracker::default();
        let mut sprite = NativeExactCpuSpritePreparationTracker::default();
        let mut scroll = NativeExactCpuScrollCopyTracker::default();
        let mut floor = NativeExactCpuFloorDrawTracker::default();
        let mut trace = NativeExactCpuHostTrace::default();
        let mut prologue = scan_step(0x00_f341, None, 10);
        prologue.accesses.push(SourceCpuBusAccess {
            address: 0x00_067a,
            timestamp: CpuMasterTimestamp::new(14),
            charged_master_cycles: 6,
            kind: SourceCpuBusAccessKind::Write {
                value: 98,
                width: 2,
            },
        });
        let mut circle_input = scan_step(0x00_f36d, None, 20);
        circle_input.accesses.push(SourceCpuBusAccess {
            address: 0x00_067a,
            timestamp: CpuMasterTimestamp::new(24),
            charged_master_cycles: 6,
            kind: SourceCpuBusAccessKind::Read {
                value: 98,
                width: 2,
            },
        });
        for step in [prologue, circle_input, scan_step(0x00_f39c, None, 30)] {
            NativeExactCpuOwner::record_step(
                &mut stack,
                &mut scan,
                &mut sprite,
                &mut scroll,
                &mut floor,
                &mut trace,
                &step,
                None,
                None,
                None,
            );
        }
        assert_eq!(
            trace.spotlight_events,
            [
                NativeExactCpuSpotlightEvent::WindowYInitialized { value: 98, at: 14 },
                NativeExactCpuSpotlightEvent::CircleInputRead { value: 98, at: 24 },
                NativeExactCpuSpotlightEvent::RowAdvanced { at: 36 },
            ]
        );
    }

    #[test]
    fn spotlight_reset_store_positions_follow_the_source_bus() {
        let mut stack = Vec::new();
        let mut scan = NativeExactCpuOverworldScanTracker::default();
        let mut sprite = NativeExactCpuSpritePreparationTracker::default();
        let mut scroll = NativeExactCpuScrollCopyTracker::default();
        let mut floor = NativeExactCpuFloorDrawTracker::default();
        let mut trace = NativeExactCpuHostTrace::default();
        for (at, pc, address) in [(10, 0x00_f42f, 0x00_1b3e), (20, 0x00_f432, 0x00_1b7e)] {
            let mut step = scan_step(pc, None, at);
            step.accesses.push(SourceCpuBusAccess {
                address,
                timestamp: CpuMasterTimestamp::new(at + 4),
                charged_master_cycles: 6,
                kind: SourceCpuBusAccessKind::Write {
                    value: 0xff00,
                    width: 2,
                },
            });
            NativeExactCpuOwner::record_step(
                &mut stack,
                &mut scan,
                &mut sprite,
                &mut scroll,
                &mut floor,
                &mut trace,
                &step,
                None,
                None,
                None,
            );
        }
        assert_eq!(trace.spotlight_reset_stores, [31, 63]);
    }

    #[test]
    fn landing_music_command_waits_for_its_distinct_source_nmi() {
        let mut state = ZeldaState::new();
        state.set_music_control(0x10);
        state.audio_nmi_processed_before_main = true;
        state.native_exact_cpu_landing_music_nmi_pending = true;
        let carried = NativeExactCpuHostTrace {
            music_port_accesses: vec![NativeExactCpuMusicAccess {
                pc: 0x00_80dc,
                address: 0x012c,
                kind: NativeExactCpuNmiGateKind::Read,
                value: 0,
                width: 1,
                at: 10,
            }],
            ..Default::default()
        };
        let unrelated = NativeExactCpuHostTrace::default();
        state.reconcile_native_exact_cpu_landing_music_nmi(&unrelated);
        assert!(state.native_exact_cpu_landing_music_nmi_pending);
        let next = NativeExactCpuHostTrace {
            music_port_accesses: vec![NativeExactCpuMusicAccess {
                value: 0x10,
                ..carried.music_port_accesses[0]
            }],
            ..Default::default()
        };
        state.reconcile_native_exact_cpu_landing_music_nmi(&next);
        assert!(!state.audio_nmi_processed_before_main);
        assert!(!state.native_exact_cpu_landing_music_nmi_pending);
        assert_eq!(state.game_state.system_signals.music_control(), 0x10);
    }

    #[test]
    fn spotlight_trial_relocation_keeps_working_and_published_generations_separate() {
        let mut state = ZeldaState::new();
        state.follower_link_state_mut().set_y(226);
        state.set_spotlight_window_radius(126);
        let table_build = state.begin_iris_spotlight_configure_table(209);
        state.ram[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + 448].fill(0xa5);
        let original = state.ram.clone();
        assert!(matches!(ZeldaState::native_exact_cpu_trial_ram(&state.ram, None, false), std::borrow::Cow::Borrowed(_)));
        state.game_execution_scheduler.schedule_work(
            GameWorkContinuation::FinishDungeonExitSpotlightEntry {
                table_build,
                iteration: super::super::SpotlightIteration::closing(
                    super::super::SpotlightIterationPhase::CloseEntryBeforeTablePublication,
                ),
            },
            1,
        );
        let trial = ZeldaState::native_exact_cpu_trial_ram(&state.ram, state.game_execution_scheduler.current_work(), false);
        assert_eq!(&trial[0x1b00..0x1cc0], &original[RESERVED_HDMA_TABLE..RESERVED_HDMA_TABLE + 448]);
        assert_eq!(&trial[0x17000..0x171c0], &original[HDMA_TABLE_DYNAMIC..HDMA_TABLE_DYNAMIC + 448]);
        assert_eq!(&trial[0x1708e..0x17090], &[0xff, 0]);
        let opening = GameWorkContinuation::FinishOverworldSpotlightBuild {
            table_build,
            phase: super::super::OverworldSpotlightBuildPhase::Recurring,
            projection_completed: false,
            iteration: super::super::SpotlightIteration::opening_from_rom_cpu_plan(None),
        };
        assert_eq!(ZeldaState::native_exact_cpu_trial_ram(&state.ram, Some(opening), false), trial);
        let landing = Some(GameWorkContinuation::FinishDungeonAfterSubmoduleCallerReturn);
        assert!(matches!(ZeldaState::native_exact_cpu_trial_ram(&state.ram, landing, false), std::borrow::Cow::Borrowed(_)));
        assert_eq!(ZeldaState::native_exact_cpu_trial_ram(&state.ram, landing, true), trial);
        for address in 0..original.len() {
            if !(0x1b00..0x1cc0).contains(&address) && !(0x17000..0x171c0).contains(&address) {
                assert_eq!(trial[address], original[address], "unowned ${address:05x}");
            }
        }
        assert_eq!(state.ram, original);
    }

    #[test]
    fn spotlight_row_progress_can_exceed_the_visible_table_height() {
        let mut trace = NativeExactCpuHostTrace {
            return_pc: 0x00_f51a,
            spotlight_events: vec![NativeExactCpuSpotlightEvent::WindowYInitialized { value: 126, at: 0 }],
            ..Default::default()
        };
        trace.spotlight_events.extend((0..230).map(|at| NativeExactCpuSpotlightEvent::RowAdvanced { at: at + 1 }));
        trace.spotlight_events.push(NativeExactCpuSpotlightEvent::CircleInputRead { value: 9, at: 231 });
        assert_eq!(trace.spotlight_build_progress(|rows| (rows, 476 - rows)).unwrap().completed_iterations, 230);
    }

    #[test]
    fn spotlight_projection_wait_and_copy_retain_the_unstepped_radius() {
        for (pc, words) in [(0x00_f3a3, 0), (0x00_f3b7, 0), (0x00_f3be, 168), (0x00_f3c5, 224)] {
            for interrupted in [false, true] {
                let nmi = NativeExactCpuNmi { interrupted_pc: pc, accepted_at: 1000 };
                let mut trace = NativeExactCpuHostTrace {
                    return_pc: if interrupted { 0x00_80c9 } else { pc },
                    nmis: if interrupted { vec![nmi] } else { Vec::new() },
                    pending_nmis_at_return: if interrupted { vec![nmi] } else { Vec::new() },
                    ..Default::default()
                };
                trace.spotlight_events.push(NativeExactCpuSpotlightEvent::WindowYInitialized { value: 77, at: 0 });
                trace.spotlight_events.extend((0..212).map(|at| NativeExactCpuSpotlightEvent::RowAdvanced { at: at + 1 }));
                trace.spotlight_events.extend((0..words).map(|at| NativeExactCpuSpotlightEvent::ProjectionWordStored { at: at + 300 }));
                assert_eq!(trace.spotlight_build_progress(|_| unreachable!()), Some(
                    crate::SpotlightTableBuildProgress {
                        completed_iterations: 213,
                        checkpoint: crate::SpotlightTableBuildCheckpoint::ProjectionCopy { copied_words: words as u16 },
                    }));
            }
        }
    }

    #[test]
    fn spotlight_upper_cursor_store_does_not_count_the_pending_lower_decrement() {
        let trace = NativeExactCpuHostTrace {
            return_pc: 0x00_f39e,
            spotlight_events: vec![
                NativeExactCpuSpotlightEvent::WindowYInitialized { value: 20, at: 0 },
                NativeExactCpuSpotlightEvent::RowAdvanced { at: 1 },
            ],
            ..Default::default()
        };
        assert_eq!(trace.spotlight_build_progress(|rows| (rows, 224 - rows)), Some(
            crate::SpotlightTableBuildProgress {
                completed_iterations: 0,
                checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeLowerCursorDecrement {
                    upper_cursor: 1, lower_cursor: 224,
                },
            }));
    }

    #[test]
    fn spotlight_circle_call_boundary_includes_the_retired_native_decrement() {
        let events = vec![
            NativeExactCpuSpotlightEvent::WindowYInitialized { value: 20, at: 10 },
            NativeExactCpuSpotlightEvent::RowAdvanced { at: 20 },
            NativeExactCpuSpotlightEvent::CircleInputRead { value: 19, at: 30 },
        ];
        for pc in [0x00_f370, 0x00_f372, 0x00_f375, 0x00_f4cc] {
            for interrupted in [false, true] {
                let nmi = NativeExactCpuNmi { interrupted_pc: pc, accepted_at: 40 };
                let trace = NativeExactCpuHostTrace {
                    return_pc: if interrupted { 0x00_80c9 } else { pc },
                    spotlight_events: events.clone(),
                    nmis: if interrupted { vec![nmi] } else { Vec::new() },
                    pending_nmis_at_return: if interrupted { vec![nmi] } else { Vec::new() },
                    ..Default::default()
                };
                let expected = matches!(pc, 0x00_f375 | 0x00_f4cc).then_some(
                    crate::SpotlightTableBuildProgress {
                        completed_iterations: 1,
                        checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeCircleCalculation {
                            pending_circle_input: 19,
                        },
                    });
                assert_eq!(trace.spotlight_build_progress(|rows| (rows, 224 - rows)), expected);
            }
        }
    }

    #[test]
    fn spotlight_builder_uses_cpu_host_return_without_an_nmi() {
        let mut trace = NativeExactCpuHostTrace {
            return_pc: 0x00_f38d,
            spotlight_events: vec![
                NativeExactCpuSpotlightEvent::WindowYInitialized { value: 20, at: 10 },
                NativeExactCpuSpotlightEvent::RowAdvanced { at: 20 },
                NativeExactCpuSpotlightEvent::CircleInputRead { value: 19, at: 30 },
            ],
            ..Default::default()
        };
        assert_eq!(
            trace.spotlight_build_progress(|rows| (rows, 224 - rows)),
            Some(crate::SpotlightTableBuildProgress {
                completed_iterations: 1,
                checkpoint: crate::SpotlightTableBuildCheckpoint::AfterUpperTableWrite {
                    lower_cursor: 223,
                },
            },)
        );
        trace.return_pc = 0x00_f392;
        assert_eq!(trace.spotlight_build_progress(|rows| (rows, 224 - rows)), Some(
            crate::SpotlightTableBuildProgress {
                completed_iterations: 1,
                checkpoint: crate::SpotlightTableBuildCheckpoint::BeforeLowerTableStore { lower_cursor: 223 },
            }));
        trace.return_pc = 0x00_8036;
        assert_eq!(trace.spotlight_build_progress(|_| unreachable!()), None);
    }

    #[test]
    fn sprite_packing_host_boundary_retains_only_an_unreturned_interrupt_cursor() {
        let cursor = ExtendedOamPackingProgress {
            group_start: 20, completed_bytes: 1, group_master_cycles: 412,
        };
        let nmi = NativeExactCpuNmi { interrupted_pc: 0x00_8620, accepted_at: 1000 };
        let mut trace = NativeExactCpuHostTrace {
            sprite_preparation_interruptions: vec![NativeExactCpuSpritePreparationInterruption {
                accepted_at: nmi.accepted_at, interrupted_pc: nmi.interrupted_pc, progress: cursor,
            }],
            ..Default::default()
        };
        assert_eq!(trace.sprite_preparation_progress_at_host_boundary(), None,
            "a completed interrupt cannot suspend a returned packing call");
        trace.pending_nmis_at_return.push(NativeExactCpuNmi { accepted_at: 999, ..nmi });
        assert_eq!(trace.sprite_preparation_progress_at_host_boundary(), None);
        trace.pending_nmis_at_return[0] = nmi;
        assert_eq!(trace.sprite_preparation_progress_at_host_boundary(), Some(cursor));
        trace.pending_nmis_at_return.clear();
        trace.sprite_preparation_progress_at_return = Some(cursor);
        assert_eq!(trace.sprite_preparation_progress_at_host_boundary(), Some(cursor));
    }

    #[test]
    fn sprite_packing_cursor_excludes_refresh_and_interrupt_entry_clocks() {
        let mut tracker = NativeExactCpuSpritePreparationTracker::default();
        tracker.observe(&scan_step(0x00_85fc, None, 100));
        let mut interrupted = scan_step(0x00_85fe, None, 110);
        interrupted.transactions = vec![
            SourceCpuTransaction {
                kind: SourceCpuTransactionKind::CpuOpsAddCyclesDraining,
                duration_master_cycles: 6,
                origin_pc: 0x00_85fe,
                opcode: interrupted.opcode,
                started_at: CpuMasterTimestamp::new(110),
                ended_at: CpuMasterTimestamp::new(156),
                cpu_model_identity: 1,
                cpu_model_5a22: 2,
                start_wram_refresh_position: 0,
                end_wram_refresh_position: 0,
            },
            SourceCpuTransaction {
                kind: SourceCpuTransactionKind::CpuOpsAddCyclesDraining,
                duration_master_cycles: 30,
                origin_pc: 0x00_85fe,
                opcode: interrupted.opcode,
                started_at: CpuMasterTimestamp::new(156),
                ended_at: CpuMasterTimestamp::new(186),
                cpu_model_identity: 1,
                cpu_model_5a22: 2,
                start_wram_refresh_position: 0,
                end_wram_refresh_position: 0,
            },
        ];
        interrupted.accepted_interrupt = Some(SourceCpuAcceptedInterrupt::Nmi {
            started_at: CpuMasterTimestamp::new(156),
            interrupted_pc: 0x00_85ff,
        });
        interrupted.ended_at = CpuMasterTimestamp::new(186);
        tracker.observe(&interrupted);
        assert_eq!(
            tracker.progress,
            Some(ExtendedOamPackingProgress {
                group_start: 28,
                completed_bytes: 0,
                group_master_cycles: 6,
            })
        );
    }

    #[test]
    fn nmi_gate_bus_accesses_keep_caller_and_handler_order() {
        let mut stack = Vec::new();
        let mut scan = NativeExactCpuOverworldScanTracker::default();
        let mut sprite = NativeExactCpuSpritePreparationTracker::default();
        let mut scroll = NativeExactCpuScrollCopyTracker::default();
        let mut floor = NativeExactCpuFloorDrawTracker::default();
        let mut trace = NativeExactCpuHostTrace::default();
        let caller_write = scan_step(0x0e_c9f9, Some((0x0e_0710, 2)), 10);
        NativeExactCpuOwner::record_step(
            &mut stack,
            &mut scan,
            &mut sprite,
                &mut scroll,
            &mut floor,
            &mut trace,
            &caller_write,
            None,
            None,
            None,
        );
        stack.push(NativeExactCpuInterrupt::Nmi(NativeExactCpuNmi {
            interrupted_pc: 0x00_8034,
            accepted_at: 20,
        }));
        let mut handler_read = scan_step(0x00_89e7, None, 30);
        handler_read.accesses.push(SourceCpuBusAccess {
            address: 0x80_0710,
            timestamp: CpuMasterTimestamp::new(34),
            charged_master_cycles: 6,
            kind: SourceCpuBusAccessKind::Read { value: 2, width: 1 },
        });
        NativeExactCpuOwner::record_step(
            &mut stack,
            &mut scan,
            &mut sprite,
                &mut scroll,
            &mut floor,
            &mut trace,
            &handler_read,
            None,
            None,
            None,
        );
        assert_eq!(
            trace.nmi_gate_accesses,
            [
                NativeExactCpuNmiGateAccess {
                    pc: 0x0e_c9f9,
                    address: 0x0710,
                    kind: NativeExactCpuNmiGateKind::Write,
                    value: 2,
                    width: 1,
                    at: 14,
                    nmi_accepted_at: None,
                },
                NativeExactCpuNmiGateAccess {
                    pc: 0x00_89e7,
                    address: 0x0710,
                    kind: NativeExactCpuNmiGateKind::Read,
                    value: 2,
                    width: 1,
                    at: 34,
                    nmi_accepted_at: Some(20),
                },
            ]
        );
    }

    #[test]
    fn nmi_update_branch_is_owned_by_the_source_latch_read() {
        let mut stack = vec![NativeExactCpuInterrupt::Nmi(NativeExactCpuNmi {
            interrupted_pc: 0x00_8034,
            accepted_at: 20,
        })];
        let mut scan = NativeExactCpuOverworldScanTracker::default();
        let mut sprite = NativeExactCpuSpritePreparationTracker::default();
        let mut scroll = NativeExactCpuScrollCopyTracker::default();
        let mut floor = NativeExactCpuFloorDrawTracker::default();
        let mut trace = NativeExactCpuHostTrace::default();
        let mut branch = scan_step(0x00_8138, None, 30);
        branch.accesses.push(SourceCpuBusAccess {
            address: 0x00_0012,
            timestamp: CpuMasterTimestamp::new(34),
            charged_master_cycles: 6,
            kind: SourceCpuBusAccessKind::Read { value: 1, width: 1 },
        });
        NativeExactCpuOwner::record_step(
            &mut stack,
            &mut scan,
            &mut sprite,
                &mut scroll,
            &mut floor,
            &mut trace,
            &branch,
            None,
            None,
            None,
        );
        assert_eq!(
            trace.nmi_update_decisions,
            [NativeExactCpuNmiUpdateDecision {
                accepted_at: 20,
                decided_at: 34,
                latch: 1,
                runs_updates: false,
            }]
        );
    }

    #[test]
    fn access_before_nmi_acceptance_keeps_caller_ancestry() {
        let mut stack = Vec::new();
        let mut scan = NativeExactCpuOverworldScanTracker::default();
        let mut sprite = NativeExactCpuSpritePreparationTracker::default();
        let mut scroll = NativeExactCpuScrollCopyTracker::default();
        let mut floor = NativeExactCpuFloorDrawTracker::default();
        let mut trace = NativeExactCpuHostTrace::default();
        let mut step = scan_step(0x00_8034, None, 30);
        step.accesses.push(SourceCpuBusAccess {
            address: 0x00_0710,
            timestamp: CpuMasterTimestamp::new(34),
            charged_master_cycles: 6,
            kind: SourceCpuBusAccessKind::Read { value: 2, width: 1 },
        });
        step.accepted_interrupt = Some(SourceCpuAcceptedInterrupt::Nmi {
            started_at: CpuMasterTimestamp::new(40),
            interrupted_pc: 0x00_8036,
        });
        NativeExactCpuOwner::record_step(
            &mut stack,
            &mut scan,
            &mut sprite,
                &mut scroll,
            &mut floor,
            &mut trace,
            &step,
            None,
            None,
            None,
        );
        assert_eq!(trace.nmi_gate_accesses[0].nmi_accepted_at, None);
        assert_eq!(trace.nmis[0].accepted_at, 40);
    }

    #[test]
    fn main_wait_return_requires_the_actual_latch_clear_store() {
        let mut stack = Vec::new();
        let mut scan = NativeExactCpuOverworldScanTracker::default();
        let mut sprite = NativeExactCpuSpritePreparationTracker::default();
        let mut scroll = NativeExactCpuScrollCopyTracker::default();
        let mut floor = NativeExactCpuFloorDrawTracker::default();
        let mut trace = NativeExactCpuHostTrace::default();
        NativeExactCpuOwner::record_step(
            &mut stack,
            &mut scan,
            &mut sprite,
                &mut scroll,
            &mut floor,
            &mut trace,
            &scan_step(0x00_805d, Some((0x00_0012, 0)), 30),
            None,
            None,
            None,
        );
        assert_eq!(
            trace.main_wait_returns,
            [NativeExactCpuMainWaitReturn { at: 34 }]
        );
        NativeExactCpuOwner::record_step(
            &mut stack,
            &mut scan,
            &mut sprite,
                &mut scroll,
            &mut floor,
            &mut trace,
            &scan_step(0x00_805d, Some((0x00_0012, 1)), 40),
            None,
            None,
            None,
        );
        assert_eq!(trace.main_wait_returns.len(), 1);
    }

    #[test]
    fn module09_reload_restores_scan_locals_before_publishing_caller_scroll() {
        let mut state = ZeldaState::new();
        let mut ranges = vec![(0, 0); 161];
        ranges[159] = (0, 2);
        ranges[160] = (2, 6);
        state.assets = Some(super::super::AssetPack::from_data_ranges(
            vec![0, 0, 0x05, 0x51, 0x41, 0xff], ranges,
        ));
        state.set_main_module(9);
        state.set_submodule(4);
        state.set_bg2_x(0x0120);
        state.set_bg2_y(0x0180);
        state.set_bg1_x_offset(7);
        state.set_bg1_y_offset(9);
        state.set_overworld_horizontal_scroll_delta_low(2);
        let records = state.begin_overworld_sprite_reload_presence_just_load();
        let mut atomic = state.clone();
        let scan = atomic.begin_overworld_sprite_reload_scan_from_presence(records);
        atomic.complete_overworld_sprite_reload_scan(scan);
        atomic.complete_module09_load_new_sprites_after_reload();
        let expected_scroll = atomic.begin_module09_sprite_main();

        let (_, slots) = state.begin_native_exact_cpu_overworld_live_scan(records);
        state.pending_overworld_sprite_reload_slots = Some(slots);
        state.native_exact_cpu_host_trace = Some(NativeExactCpuHostTrace {
            overworld_scan_events: vec![
                NativeExactCpuOverworldScanEvent::Began { at: 10 },
                NativeExactCpuOverworldScanEvent::CellReturned { at: 20, ordinal: 440 },
            ],
            ..Default::default()
        });
        state.advance_native_exact_cpu_overworld_live_scan();
        assert_eq!(state.game_state.display.ppu_scroll_copy.bg2_h_copy2(), 0x0120 + 20 * 16);
        state.native_exact_cpu_host_trace.as_mut().unwrap().overworld_scan_events = vec![
            NativeExactCpuOverworldScanEvent::CellReturned { at: 30, ordinal: 484 },
            NativeExactCpuOverworldScanEvent::Finished { at: 40, cells: 484 },
        ];
        // This return must consume the final native scan before composing
        // Module09's offset scroll registers from the restored camera locals.
        state.publish_deferred_module09_sprite_slots_at_reload_return();
        assert!(state.native_exact_cpu_overworld_live_scan.is_none());
        assert_eq!(state.game_state.display.ppu_scroll_copy.bg2_h_copy2(), 0x0120);
        state.complete_module09_load_new_sprites_after_reload();
        assert_eq!(state.begin_module09_sprite_main(), expected_scroll);
        assert_eq!(*state.wram_image(), *atomic.wram_image());
        assert_eq!(state.game_state, atomic.game_state);
        let once = state.ram.clone();
        state.advance_native_exact_cpu_overworld_live_scan();
        assert_eq!(state.ram, once, "host tail must not replay a scan consumed by its caller");
    }

    #[test]
    fn pre_overworld_scan_publishes_activations_before_its_caller_returns() {
        let mut state = ZeldaState::new();
        let mut ranges = vec![(0, 0); 161];
        ranges[159] = (0, 2);
        ranges[160] = (2, 6);
        state.assets = Some(super::super::AssetPack::from_data_ranges(
            vec![0, 0, 0x05, 0x51, 0x41, 0xff], ranges,
        ));
        state.set_bg2_x(0x0120);
        state.set_bg2_y(0x0180);
        state.set_overworld_horizontal_scroll_delta_low(2);
        state.sprite_begin_reload_all_overworld();
        let records = state.overworld_load_sprites();
        state.begin_overworld_proximity_scan_continuation();
        let mut atomic = state.clone();
        atomic.sprite_activate_all_proxima();
        atomic.complete_overworld_proximity_scan_continuation();

        state.begin_native_exact_cpu_overworld_live_scan(records);
        state.native_exact_cpu_host_trace = Some(NativeExactCpuHostTrace {
            overworld_scan_events: vec![
                NativeExactCpuOverworldScanEvent::Began { at: 10 },
                NativeExactCpuOverworldScanEvent::CellReturned { at: 20, ordinal: 1 },
            ],
            ..Default::default()
        });
        state.advance_native_exact_cpu_overworld_live_scan();
        assert!((0..16).any(|slot| {
            let sprite = state.sprite_slot_view(slot);
            sprite.state() == 8 && sprite.sprite_type() == 0x41
        }));
        assert!(state.native_exact_cpu_overworld_live_scan.is_some());
        state.native_exact_cpu_host_trace.as_mut().unwrap().overworld_scan_events = vec![
            NativeExactCpuOverworldScanEvent::CellReturned { at: 30, ordinal: 484 },
            NativeExactCpuOverworldScanEvent::Finished { at: 40, cells: 484 },
        ];
        state.advance_native_exact_cpu_overworld_live_scan();
        state.complete_overworld_proximity_scan_continuation();
        assert!(state.native_exact_cpu_overworld_live_scan.is_none());
        assert_eq!(*state.wram_image(), *atomic.wram_image());
        assert_eq!(state.game_state, atomic.game_state);
    }

    #[test]
    fn source_overworld_scan_events_cross_host_boundaries_without_copying_memory() {
        let mut tracker = NativeExactCpuOverworldScanTracker::default();
        let mut first_host = Vec::new();
        tracker.observe(
            &scan_step(0x09_c56a, Some((0x09_069f, 0xff)), 10),
            &mut first_host,
        );
        tracker.observe(&scan_step(0x09_c5e6, None, 20), &mut first_host);
        tracker.observe(
            &scan_step(0x06_83ae, Some((0x06_069f, 0)), 30),
            &mut first_host,
        );
        assert_eq!(
            first_host,
            [
                NativeExactCpuOverworldScanEvent::Began { at: 14 },
                NativeExactCpuOverworldScanEvent::CellReturned { at: 20, ordinal: 1 },
            ]
        );

        let mut next_host = Vec::new();
        tracker.observe(&scan_step(0x09_c5e6, None, 40), &mut next_host);
        tracker.observe(
            &scan_step(0x09_c585, Some((0x09_069f, 0)), 50),
            &mut next_host,
        );
        tracker.observe(&scan_step(0x09_c5e6, None, 60), &mut next_host);
        assert_eq!(
            next_host,
            [
                NativeExactCpuOverworldScanEvent::CellReturned { at: 40, ordinal: 2 },
                NativeExactCpuOverworldScanEvent::Finished { at: 54, cells: 2 },
            ]
        );
        assert!(!tracker.active);
    }
}

#[cfg(feature = "rom-timing")]
#[path = "rom_timing_shadow_execution.rs"]
mod rom_timing_shadow_execution;
