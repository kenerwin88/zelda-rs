//! Retained source-ordered CPU timeline alongside translated native execution.
//!
//! This owner is opt-in while native scheduling still consumes the aggregate
//! ROM timing plans. It does not publish source WRAM or presentation state.

use super::{
    sprite::OverworldSpriteReloadScanWork, ExtendedOamPackingProgress, GameState,
    GameWorkContinuation, ItemReceiptGraphicsContinuation, ItemReceiptReturn,
    OverworldSpriteReloadWorkload, SpriteSlotsState, ZeldaState,
};
use crate::game_state::constants::{
    ANCILLA_STEP, DIALOGUE_MSG_SRC_OFFS, OVERWORLD_DECOMP_BUFFER, OVERWORLD_MAP16_DECODE_SRC,
    TEXT_DIALOGUE_POINTERS,
};
use crate::rom_cpu_timing::ROM_DIALOGUE_MESSAGE_COUNT;
use crate::{
    CachedSpriteCacheField, DungeonResetSpritesCpuProgress, DungeonResetSpritesProgressReceipt,
    OriginalTimingBoundary,
};
use snes::{
    Snes9xColdCpuExecutor, Snes9xCpuQuiescentCheckpoint, Snes9xCpuQuiescentCheckpointError,
    SourceCpuAcceptedInterrupt, SourceCpuBusAccessKind, SourceCpuError, SourceCpuStepReceipt,
};
use std::collections::HashMap;

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
pub(super) enum NativeExactCpuSpotlightEvent {
    WindowYInitialized { value: u16, at: u64 },
    CircleInputRead { value: u16, at: u64 },
    RowAdvanced { at: u64 },
}

/// Source statement boundaries for the original-ROM overworld proximity scan.
/// The translated scan may consume these events without importing source WRAM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub(super) fn advance_native_exact_cpu_bg_chars_gate(&mut self) {
        if crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_BG_CHARS_GATE_LIVE_TRIAL").is_none() {
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

    /// A landing-wipe build left inside its row loop at the CPU host return.
    /// The ordinal comes from completed `$F39C` cursor updates, not from a
    /// frame estimate or source WRAM copy. A host may stop mid-call without
    /// accepting an NMI, so the retained CPU's return PC is authoritative in
    /// that case.
    pub(super) fn native_exact_cpu_landing_spotlight_checkpoint(
        &self,
    ) -> Option<crate::SpotlightTableBuildProgress> {
        if crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_SPOTLIGHT_LIVE_TRIAL").is_none()
            || self.game_state.frame.main_module != 7
            || self.game_state.frame.submodule != 0x0f
            || self.game_state.frame.subsubmodule != 1
        {
            return None;
        }
        let trace = self.native_exact_cpu_host_trace.as_ref()?;
        let progress = trace.spotlight_build_progress(|completed_iterations| {
            self.spotlight_lower_cursor_after_completed_rows(completed_iterations)
        })?;
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
        if crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_ROOM_HEADER_LIVE_TRIAL").is_none() {
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
            && crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_OBJECT_DRAW_LIVE_TRIAL").is_some()
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
        if trace.object_completions.is_empty() {
            return;
        }
        let host = trace.host;
        let completions = trace.object_completions.clone();
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
        eprintln!(
            "native-exact-cpu-live-objects host={host} completed_objects={}",
            work.completed_objects()
        );
        self.active_early_dungeon_object_draw = Some(work);
    }

    pub(super) fn advance_native_exact_cpu_room_upload(&mut self) {
        if crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_ROOM_UPLOAD_LIVE_TRIAL").is_none() {
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
                    let Some(mut objects) = self.active_early_dungeon_object_draw.take() else {
                        continue;
                    };
                    assert!(
                        !self.advance_dungeon_object_draw_one(&mut objects),
                        "source room upload began before translated objects completed"
                    );
                    assert!(objects.finished());
                    assert!(self.active_early_dungeon_room_upload.is_none());
                    self.active_early_dungeon_room_upload =
                        Some(self.begin_dungeon_room_upload_after_early_objects());
                    eprintln!(
                        "native-exact-cpu-live-upload host={host} began after_objects={}",
                        objects.completed_objects()
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
        if crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_ITEM_RECEIPT_LIVE_TRIAL").is_none() {
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
        if let Some(trial) = self.native_exact_cpu_overworld_scan_trial.as_mut() {
            trial.expected_final_ram = Some(self.ram.clone());
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

#[derive(Clone, Copy, Debug, Default, serde::Serialize, serde::Deserialize)]
struct NativeExactCpuFloorDrawTracker {
    active: bool,
    completed_words: u16,
    floor_complete: bool,
    object_in_progress: bool,
    completed_objects: u16,
}

impl NativeExactCpuFloorDrawTracker {
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
    pub main_wait_returns: Vec<NativeExactCpuMainWaitReturn>,
    /// The room-loader JSR has returned to Module07's next instruction.
    pub room_load_returned: bool,
    /// The auxiliary sprite graphics JSR has returned to Module07.
    pub auxiliary_sprite_graphics_returned: bool,
    /// Source-owned cached sprite statement at this host return, if the
    /// retained CPU stopped inside a resumable translated handler.
    pub cached_sprite_progress: Option<crate::CachedSpriteExecutionProgressReceipt>,
    pub dungeon_reset_progress: Option<DungeonResetSpritesProgressReceipt>,
    pub main_loop_entries: Vec<NativeExactCpuMainLoopEntry>,
    pub sprite_preparation_interruptions: Vec<NativeExactCpuSpritePreparationInterruption>,
    pub dungeon_header_index_writes: Vec<NativeExactCpuDungeonHeaderIndexWrite>,
    pub floor_tile_writes: Vec<NativeExactCpuFloorTileWrite>,
    pub object_completions: Vec<NativeExactCpuObjectCompletion>,
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
    fn spotlight_build_progress(
        &self,
        lower_cursor_after_rows: impl FnOnce(u16) -> u16,
    ) -> Option<crate::SpotlightTableBuildProgress> {
        let stop_pc = self
            .pending_nmis_at_return
            .iter()
            .find(|nmi| {
                self.nmis.contains(nmi)
                    && ((0x00_f4cc..=0x00_f53d).contains(&nmi.interrupted_pc)
                        || (0x00_f361..=0x00_f3a0).contains(&nmi.interrupted_pc))
            })
            .map(|nmi| nmi.interrupted_pc)
            .unwrap_or(self.return_pc);
        if !(0x00_f361..=0x00_f3a0).contains(&stop_pc)
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
        let completed_iterations = u16::try_from(
            active
                .iter()
                .filter(|event| matches!(event, NativeExactCpuSpotlightEvent::RowAdvanced { .. }))
                .count(),
        )
        .expect("spotlight builder exceeded its 224 scanline rows");
        assert!(completed_iterations <= 224);
        let checkpoint = match (stop_pc, active.last().copied()) {
            (
                0x00_f4cc..=0x00_f53d,
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
                    lower_cursor: lower_cursor_after_rows(completed_iterations),
                }
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
struct NativeExactCpuRead {
    pc: u32,
    address: u32,
    value: u16,
    width: u8,
    at: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct NativeExactCpuWrite {
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
        .find(|write| write.at < read.at && native_exact_cpu_wram_key(write.address) == Some(key))
        .copied()
        .map(|write| NativeExactCpuWriteWitness { host, write })
        .or_else(|| prior.get(&key).copied())
}

fn native_exact_cpu_record_writers(
    writers: &mut HashMap<u32, NativeExactCpuWriteWitness>,
    writes: &[NativeExactCpuWrite],
    host: u32,
) {
    for &write in writes {
        if let Some(key) = native_exact_cpu_wram_key(write.address) {
            writers.insert(key, NativeExactCpuWriteWitness { host, write });
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    fn progress_at_return(
        self,
        pc: u32,
        cpu: &Snes9xColdCpuExecutor,
    ) -> Option<crate::CachedSpriteExecutionProgress> {
        let slot = self.slot?;
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
    floor_draw: NativeExactCpuFloorDrawTracker,
    cached_sprite: NativeExactCpuCachedSpriteTracker,
    dungeon_cache: NativeExactCpuDungeonCacheTracker,
    diagnostic_writers: HashMap<u32, NativeExactCpuWriteWitness>,
}

const NATIVE_EXACT_CPU_CHECKPOINT_VERSION: u8 = 1;

#[derive(serde::Serialize, serde::Deserialize)]
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

impl NativeExactCpuOwner {
    fn capture_checkpoint(&self) -> Result<NativeExactCpuOwnerCheckpoint, String> {
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
            floor_draw: checkpoint.floor_draw,
            cached_sprite: checkpoint.cached_sprite,
            dungeon_cache: checkpoint.dungeon_cache,
            diagnostic_writers: checkpoint.diagnostic_writers,
        })
    }
}

impl ZeldaState {
    /// Separate sidecar because ordinary gameplay save states deliberately
    /// omit the retained source CPU and its interrupted call stack.
    pub fn native_exact_cpu_checkpoint_available(&self) -> bool {
        self.native_exact_cpu_owner
            .as_ref()
            .is_some_and(|owner| owner.next_host == self.frame_ctr_dbg)
    }

    pub fn capture_native_exact_cpu_checkpoint(&self) -> Result<Option<Vec<u8>>, String> {
        let Some(owner) = self.native_exact_cpu_owner.as_ref() else {
            return Ok(None);
        };
        if owner.next_host != self.frame_ctr_dbg {
            return Err("native exact CPU checkpoint is not at the game host boundary".to_string());
        }
        bincode::serialize(&owner.capture_checkpoint()?)
            .map(Some)
            .map_err(|error| error.to_string())
    }

    pub fn restore_native_exact_cpu_checkpoint(&mut self, bytes: &[u8]) -> Result<(), String> {
        let checkpoint: NativeExactCpuOwnerCheckpoint =
            bincode::deserialize(bytes).map_err(|error| error.to_string())?;
        let owner = NativeExactCpuOwner::from_checkpoint(checkpoint)?;
        if owner.next_host != self.frame_ctr_dbg {
            return Err(format!(
                "native exact CPU checkpoint host {} differs from game host {}",
                owner.next_host, self.frame_ctr_dbg
            ));
        }
        self.native_exact_cpu_owner = Some(owner);
        self.native_exact_cpu_host_trace = None;
        Ok(())
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
            floor_draw: self.floor_draw,
            cached_sprite: self.cached_sprite,
            dungeon_cache: self.dungeon_cache,
            diagnostic_writers: self.diagnostic_writers.clone(),
        }
    }
}

impl NativeExactCpuOwner {
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
            floor_draw: NativeExactCpuFloorDrawTracker::default(),
            cached_sprite: NativeExactCpuCachedSpriteTracker::default(),
            dungeon_cache: NativeExactCpuDungeonCacheTracker::default(),
            diagnostic_writers: HashMap::new(),
        })
    }

    fn record_step(
        interrupt_stack: &mut Vec<NativeExactCpuInterrupt>,
        overworld_scan: &mut NativeExactCpuOverworldScanTracker,
        sprite_preparation: &mut NativeExactCpuSpritePreparationTracker,
        floor_draw: &mut NativeExactCpuFloorDrawTracker,
        trace: &mut NativeExactCpuHostTrace,
        step: &SourceCpuStepReceipt,
        reads: Option<&mut Vec<NativeExactCpuRead>>,
        writes: Option<&mut Vec<NativeExactCpuWrite>>,
        instructions: Option<&mut Vec<NativeExactCpuInstruction>>,
    ) -> Option<NativeExactCpuNmiCompletion> {
        overworld_scan.observe(step, &mut trace.overworld_scan_events);
        sprite_preparation.observe(step);
        floor_draw.observe(step, trace);
        if step.origin_pc == 0x02_8a5f {
            trace.room_load_returned = true;
        }
        if step.origin_pc == 0x02_8a67 {
            trace.auxiliary_sprite_graphics_returned = true;
            if crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_TRACE_AUX_RETURN").is_some() {
                eprintln!(
                    "native-exact-cpu-aux-return host={} at={}",
                    trace.host,
                    step.started_at.master_cycles()
                );
            }
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
            let wram_bank = bank == 0x7e
                || (address < 0x2000 && (bank <= 0x3f || (0x80..=0xbf).contains(&bank)));
            if trace.wram_watch_address == Some(address) && wram_bank {
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

    fn advance_host_with_hook<E>(
        &mut self,
        host: u32,
        raw_input: u16,
        mut reads: Option<&mut Vec<NativeExactCpuRead>>,
        mut writes: Option<&mut Vec<NativeExactCpuWrite>>,
        mut instructions: Option<&mut Vec<NativeExactCpuInstruction>>,
        mut hook: impl FnMut(
            &mut Snes9xColdCpuExecutor,
            Option<NativeExactCpuNmiCompletion>,
        ) -> Result<(), E>,
    ) -> Result<NativeExactCpuHostTrace, E>
    where
        E: From<SourceCpuError>,
    {
        assert_eq!(
            host, self.next_host,
            "exact CPU host sequence must be contiguous"
        );
        self.cpu.set_libretro_joypad_words(raw_input, 0);
        let mut trace = NativeExactCpuHostTrace {
            host,
            track_floor_tile_writes: crate::debug_env::var_os(
                "ZELDA3_NATIVE_EXACT_CPU_FLOOR_DRAW_LIVE_TRIAL",
            )
            .is_some(),
            wram_watch_address: crate::debug_env::var("ZELDA3_NATIVE_EXACT_CPU_WATCH_WRAM_ADDR")
                .ok()
                .and_then(|value| {
                    let value = value.trim_start_matches("0x");
                    u16::from_str_radix(value, 16).ok()
                }),
            ..Default::default()
        };
        self.cpu
            .run_until_main_loop_return_with_quiescent_step_hook(|step, cpu| {
                let completed = Self::record_step(
                    &mut self.interrupt_stack,
                    &mut self.overworld_scan,
                    &mut self.sprite_preparation,
                    &mut self.floor_draw,
                    &mut trace,
                    step,
                    reads.as_deref_mut(),
                    writes.as_deref_mut(),
                    instructions.as_deref_mut(),
                );
                self.cached_sprite.observe(step, cpu);
                self.dungeon_cache.observe(step, cpu);
                hook(cpu, completed)
            })?;
        self.source_nmis_accepted += trace.nmis.len() as u64;
        self.source_nmis_completed += trace.nmi_completions.len() as u64;
        trace.pending_nmis_at_return = self
            .interrupt_stack
            .iter()
            .filter_map(|interrupt| match interrupt {
                NativeExactCpuInterrupt::Nmi(accepted) => Some(*accepted),
                NativeExactCpuInterrupt::Irq => None,
            })
            .collect();
        let registers = &self.cpu.machine().snes().cpu;
        trace.return_pc = (u32::from(registers.k) << 16) | u32::from(registers.pc);
        let cached_boundary_nmi = trace
            .pending_nmis_at_return
            .iter()
            .rev()
            .find(|nmi| trace.nmis.contains(nmi));
        let cached_stop_pc = cached_boundary_nmi.map_or(trace.return_pc, |nmi| nmi.interrupted_pc);
        trace.cached_sprite_progress = self
            .cached_sprite
            .progress_at_return(cached_stop_pc, &self.cpu)
            .map(|progress| crate::CachedSpriteExecutionProgressReceipt {
                progress,
                boundary: if cached_boundary_nmi.is_some() {
                    crate::OriginalTimingBoundary::NmiAccepted
                } else {
                    crate::OriginalTimingBoundary::HostReturn
                },
            });
        trace.dungeon_reset_progress = self.dungeon_cache.progress_at_return(
            cached_stop_pc,
            if cached_boundary_nmi.is_some() {
                OriginalTimingBoundary::NmiAccepted
            } else {
                OriginalTimingBoundary::HostReturn
            },
        );
        drop(self.cpu.take_dsp_samples());
        self.next_host += 1;
        Ok(trace)
    }

    fn advance_host(
        &mut self,
        host: u32,
        raw_input: u16,
        reads: Option<&mut Vec<NativeExactCpuRead>>,
        writes: Option<&mut Vec<NativeExactCpuWrite>>,
        instructions: Option<&mut Vec<NativeExactCpuInstruction>>,
    ) -> Result<NativeExactCpuHostTrace, SourceCpuError> {
        self.advance_host_with_hook(host, raw_input, reads, writes, instructions, |_, _| Ok(()))
    }
}

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

    pub(super) fn advance_native_exact_cpu_host(&mut self) {
        if !self.rom_startup_timing
            || self.original_timing_semantic_receipts.is_some()
            || crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_OWNER").is_none()
        {
            self.native_exact_cpu_host_trace = None;
            return;
        }
        let host = self
            .frame_ctr_dbg
            .checked_sub(1)
            .expect("native exact CPU host starts after frame entry");
        let trace_host = crate::debug_env::var("ZELDA3_NATIVE_EXACT_CPU_TRACE_HOST")
            .ok()
            .and_then(|value| {
                let (first, last) = value.split_once('-').unwrap_or((&value, &value));
                Some((first.parse::<u32>().ok()?, last.parse::<u32>().ok()?))
            })
            .is_some_and(|(first, last)| (first..=last).contains(&host));
        let independent_trial_host =
            crate::debug_env::var("ZELDA3_NATIVE_EXACT_CPU_REBASE_TRIAL_HOST")
                .ok()
                .and_then(|value| {
                    let (first, last) = value.split_once('-').unwrap_or((&value, &value));
                    Some((first.parse::<u32>().ok()?, last.parse::<u32>().ok()?))
                })
                .is_some_and(|(first, last)| (first..=last).contains(&host));
        let contiguous_trial_start =
            crate::debug_env::var("ZELDA3_NATIVE_EXACT_CPU_CONTIGUOUS_TRIAL_START")
                .ok()
                .and_then(|value| value.parse::<u32>().ok());
        let diagnose_ownership =
            crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_DIAGNOSE_OWNERSHIP").is_some();
        let owner = if let Some(owner) = self.native_exact_cpu_owner.as_mut() {
            owner
        } else {
            self.native_exact_cpu_owner = Some(
                NativeExactCpuOwner::new(&self.rom, &self.sram)
                    .expect("native exact CPU requires a cold Zelda ROM/SRAM seed"),
            );
            self.native_exact_cpu_owner.as_mut().unwrap()
        };
        if crate::debug_env::var("ZELDA3_NATIVE_EXACT_CPU_NMI_PHASE_HOST")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            == Some(host)
        {
            eprintln!("native-exact-cpu-entry host={host} source_nmis_accepted={} source_nmis_completed={} translated_nmi_handlers_completed={} carried_nmis={:?} scheduler={:?}",
                owner.source_nmis_accepted, owner.source_nmis_completed,
                owner.translated_nmi_handlers_completed,
                owner.interrupt_stack.iter().filter_map(|interrupt| match interrupt {
                    NativeExactCpuInterrupt::Nmi(accepted) => Some(*accepted),
                    NativeExactCpuInterrupt::Irq => None,
                }).collect::<Vec<_>>(), self.game_execution_scheduler);
        }
        let contiguous_trial = contiguous_trial_start.is_some_and(|start| host >= start)
            && !owner.translated_trial_stopped;
        let trial_host = independent_trial_host || contiguous_trial;
        let preserve_stack_segment = trial_host
            && crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_REBASE_TRIAL_STACK").is_some();
        let preserve_direct_page = preserve_stack_segment
            && crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_REBASE_TRIAL_DP").is_some();
        let preserve_dialogue_asset_state = preserve_direct_page
            && crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_REBASE_TRIAL_DIALOGUE").is_some();
        let preserve_overworld_decode_scratch = preserve_stack_segment
            && crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_REBASE_TRIAL_OVERWORLD_SCRATCH")
                .is_some();
        // A trial never changes the retained owner or translated gameplay.
        // It tests whether the exact CPU can read translated memory at this
        // instruction boundary while retaining its physical PPU/APU/DMA bus.
        let mut trial_reads = Vec::new();
        let mut trial_writes = Vec::new();
        let mut trial_instructions = Vec::new();
        let trial = if trial_host {
            let mut candidate = if contiguous_trial {
                owner
                    .translated_trial
                    .take()
                    .map(|trial| *trial)
                    .unwrap_or_else(|| {
                        let mut trial = owner.clone();
                        trial.translated_trial = None;
                        trial
                    })
            } else {
                owner.clone()
            };
            let pre_rebase_wram =
                diagnose_ownership.then(|| candidate.cpu.machine().snes().ram.clone());
            let carried_nmi = candidate
                .interrupt_stack
                .iter()
                .rev()
                .find_map(|interrupt| match interrupt {
                    NativeExactCpuInterrupt::Nmi(accepted) => Some(*accepted),
                    NativeExactCpuInterrupt::Irq => None,
                });
            let phase_aligned_rebase =
                crate::debug_env::var_os("ZELDA3_NATIVE_EXACT_CPU_REBASE_AFTER_CARRIED_NMI")
                    .is_some()
                    && self
                        .game_execution_scheduler
                        .resumed_call_stack_is_before_nmi()
                    && carried_nmi.is_some();
            let mut entry_sp = candidate.cpu.machine().snes().cpu.sp;
            let mut rebased_after_handler = false;
            let result: Result<NativeExactCpuHostTrace, Box<dyn std::error::Error>> =
                if phase_aligned_rebase {
                    let carried = carried_nmi.unwrap();
                    candidate.advance_host_with_hook(
                        host,
                        self.previous_host_controller_input,
                        Some(&mut trial_reads),
                        Some(&mut trial_writes),
                        Some(&mut trial_instructions),
                        |cpu, completed| {
                            if !rebased_after_handler
                                && completed
                                    .is_some_and(|nmi| nmi.accepted_at == carried.accepted_at)
                            {
                                entry_sp = rebase_translated_timing_state(
                                    cpu,
                                    &self.ram,
                                    &self.sram,
                                    preserve_stack_segment,
                                    preserve_direct_page,
                                    preserve_dialogue_asset_state,
                                    preserve_overworld_decode_scratch,
                                )?;
                                rebased_after_handler = true;
                            }
                            Ok(())
                        },
                    )
                } else {
                    match rebase_translated_timing_state(
                        &mut candidate.cpu,
                        &self.ram,
                        &self.sram,
                        preserve_stack_segment,
                        preserve_direct_page,
                        preserve_dialogue_asset_state,
                        preserve_overworld_decode_scratch,
                    ) {
                        Ok(sp) => {
                            entry_sp = sp;
                            candidate
                                .advance_host(
                                    host,
                                    self.previous_host_controller_input,
                                    Some(&mut trial_reads),
                                    Some(&mut trial_writes),
                                    Some(&mut trial_instructions),
                                )
                                .map_err(Into::into)
                        }
                        Err(error) => Err(error.into()),
                    }
                };
            let cpu = &candidate.cpu.machine().snes().cpu;
            Some((
                result,
                (u32::from(cpu.k) << 16) | u32::from(cpu.pc),
                candidate.cpu.machine().timestamp().master_cycles(),
                entry_sp,
                phase_aligned_rebase,
                rebased_after_handler,
                candidate,
                pre_rebase_wram,
            ))
        } else {
            None
        };
        let mut baseline_reads = Vec::new();
        let mut baseline_writes = Vec::new();
        let mut baseline_instructions = Vec::new();
        let trace = owner
            .advance_host(
                host,
                self.previous_host_controller_input,
                trial_host.then_some(&mut baseline_reads),
                trial_host.then_some(&mut baseline_writes),
                trial_host.then_some(&mut baseline_instructions),
            )
            .expect("retained native exact CPU failed before host return");
        if let Some((
            trial_result,
            candidate_pc,
            candidate_clock,
            entry_sp,
            phase_aligned_rebase,
            rebased_after_handler,
            mut candidate,
            pre_rebase_wram,
        )) = trial
        {
            let first_read_difference = baseline_reads
                .iter()
                .zip(&trial_reads)
                .find(|(baseline, candidate)| baseline != candidate)
                .map(|(baseline, candidate)| (*baseline, *candidate));
            let first_write_difference = baseline_writes
                .iter()
                .zip(&trial_writes)
                .find(|(baseline, candidate)| baseline != candidate)
                .map(|(baseline, candidate)| (*baseline, *candidate));
            let trial_status = trial_result
                .as_ref()
                .map(|result| format!("nmis={:?}", result.nmis))
                .unwrap_or_else(|error| format!("stopped={error}"));
            let cpu = &owner.cpu.machine().snes().cpu;
            let baseline_pc = (u32::from(cpu.k) << 16) | u32::from(cpu.pc);
            let baseline_clock = owner.cpu.machine().timestamp().master_cycles();
            let same_schedule = trial_result.as_ref().is_ok_and(|candidate| {
                candidate.nmis == trace.nmis
                    && baseline_instructions == trial_instructions
                    && candidate_pc == baseline_pc
                    && candidate_clock == baseline_clock
            });
            let same_path =
                same_schedule && baseline_reads == trial_reads && baseline_writes == trial_writes;
            if diagnose_ownership {
                if let Some((source_read, trial_read)) = first_read_difference {
                    let source_writer = native_exact_cpu_last_writer(
                        source_read,
                        &baseline_writes,
                        &owner.diagnostic_writers,
                        host,
                    );
                    let trial_writer = native_exact_cpu_last_writer(
                        trial_read,
                        &trial_writes,
                        &candidate.diagnostic_writers,
                        host,
                    );
                    let rebase = native_exact_cpu_wram_key(source_read.address).and_then(|key| {
                        let index = usize::try_from(key).ok()?;
                        let before = pre_rebase_wram.as_ref()?.get(index).copied()?;
                        let native = self.ram.get(index).copied()?;
                        Some((before, native))
                    });
                    eprintln!("native-exact-cpu-ownership host={host} first_read=({source_read:?}, {trial_read:?}) source_last_writer={source_writer:?} trial_last_writer={trial_writer:?} rebase_before_native={rebase:?} native_continuation={:?} cached_checkpoint={:?} dungeon_reset_checkpoint={:?}",
                        self.game_execution_scheduler.current_work(),
                        trace.cached_sprite_progress, trace.dungeon_reset_progress);
                }
                native_exact_cpu_record_writers(
                    &mut owner.diagnostic_writers,
                    &baseline_writes,
                    host,
                );
                native_exact_cpu_record_writers(
                    &mut candidate.diagnostic_writers,
                    &trial_writes,
                    host,
                );
            }
            let first_instruction_difference = baseline_instructions
                .iter()
                .zip(&trial_instructions)
                .find(|(baseline, candidate)| baseline != candidate)
                .map(|(baseline, candidate)| (*baseline, *candidate));
            eprintln!("native-exact-cpu-rebase-trial host={host} contiguous={contiguous_trial} entry_sp={entry_sp:04x} phase_aligned_rebase={phase_aligned_rebase} rebased_after_handler={rebased_after_handler} stack_segment_preserved={preserve_stack_segment} direct_page_preserved={preserve_direct_page} dialogue_asset_state_preserved={preserve_dialogue_asset_state} overworld_decode_scratch_preserved={preserve_overworld_decode_scratch} same_path={same_path} same_schedule={same_schedule} baseline_instructions={} trial_instructions={} first_instruction_difference={first_instruction_difference:?} baseline_reads={} trial_reads={} first_read_difference={first_read_difference:?} baseline_writes={} trial_writes={} first_write_difference={first_write_difference:?} baseline_nmis={:?} trial_status={trial_status} baseline_pc={baseline_pc:06x} trial_pc={candidate_pc:06x} baseline_clock={baseline_clock} trial_clock={candidate_clock}",
                baseline_instructions.len(), trial_instructions.len(),
                baseline_reads.len(), trial_reads.len(), baseline_writes.len(), trial_writes.len(), trace.nmis);
            if trace.wram_watch_address.is_some() && (trace_host || first_read_difference.is_some())
            {
                eprintln!("native-exact-cpu-wram-watch host={host} address={:?} source={:?} translated={:?}",
                    trace.wram_watch_address, trace.wram_watch_accesses,
                    trial_result.as_ref().map(|candidate| &candidate.wram_watch_accesses));
            }
            if contiguous_trial {
                // A data read can differ without changing the executed CPU
                // instruction timeline. Retain that distinction rather than
                // treating rendered content as a timing failure.
                if same_schedule {
                    owner.translated_trial = Some(Box::new(candidate));
                } else {
                    owner.translated_trial_stopped = true;
                }
            }
        }
        if trace_host {
            eprintln!("native-exact-cpu host={} return_pc={:06x} room_load_returned={} auxiliary_sprite_graphics_returned={} nmis={:?} nmi_completions={:?} pending_nmis_at_return={:?} nmi_update_decisions={:?} main_wait_returns={:?} main_loop_entries={:?} sprite_preparation_interruptions={:?} dungeon_header_index_writes={:?} nmi_gate_accesses={:?} ppu_reads={:?} overworld_scan_events={:?}", trace.host, trace.return_pc, trace.room_load_returned, trace.auxiliary_sprite_graphics_returned,
                trace.nmis, trace.nmi_completions, trace.pending_nmis_at_return,
                trace.nmi_update_decisions, trace.main_wait_returns, trace.main_loop_entries, trace.sprite_preparation_interruptions, trace.dungeon_header_index_writes, trace.nmi_gate_accesses, trace.ppu_reads, trace.overworld_scan_events);
            if trace.wram_watch_address.is_some() && !trial_host {
                eprintln!(
                    "native-exact-cpu-wram-watch host={host} address={:?} source={:?}",
                    trace.wram_watch_address, trace.wram_watch_accesses
                );
            }
            if trace.track_floor_tile_writes && !trace.floor_tile_writes.is_empty() {
                eprintln!(
                    "native-exact-cpu-floor-writes host={host} count={} first={:?} last={:?}",
                    trace.floor_tile_writes.len(),
                    trace.floor_tile_writes.first(),
                    trace.floor_tile_writes.last()
                );
            }
            if !trace.object_completions.is_empty() {
                eprintln!(
                    "native-exact-cpu-object-completions host={host} count={} first={:?} last={:?}",
                    trace.object_completions.len(),
                    trace.object_completions.first(),
                    trace.object_completions.last()
                );
            }
            if !trace.room_upload_events.is_empty() {
                eprintln!(
                    "native-exact-cpu-room-upload-events host={host} events={:?}",
                    trace.room_upload_events
                );
            }
            if !trace.item_receipt_step_writes.is_empty() {
                eprintln!(
                    "native-exact-cpu-item-receipt-steps host={host} writes={:?}",
                    trace.item_receipt_step_writes
                );
            }
            if !trace.spotlight_events.is_empty() {
                let starts = trace
                    .spotlight_events
                    .iter()
                    .filter(|event| {
                        matches!(
                            event,
                            NativeExactCpuSpotlightEvent::WindowYInitialized { .. }
                        )
                    })
                    .count();
                let reads = trace
                    .spotlight_events
                    .iter()
                    .filter(|event| {
                        matches!(event, NativeExactCpuSpotlightEvent::CircleInputRead { .. })
                    })
                    .count();
                let rows = trace
                    .spotlight_events
                    .iter()
                    .filter(|event| {
                        matches!(event, NativeExactCpuSpotlightEvent::RowAdvanced { .. })
                    })
                    .count();
                eprintln!("native-exact-cpu-spotlight-events host={host} starts={starts} reads={reads} rows={rows} last={:?}",
                    trace.spotlight_events.last());
            }
            if !trace.spotlight_reset_stores.is_empty() {
                eprintln!(
                    "native-exact-cpu-spotlight-reset host={host} stores={} first={:?} last={:?}",
                    trace.spotlight_reset_stores.len(),
                    trace.spotlight_reset_stores.first(),
                    trace.spotlight_reset_stores.last()
                );
            }
            if !trace.music_port_accesses.is_empty() {
                eprintln!(
                    "native-exact-cpu-music-port host={host} accesses={:?}",
                    trace.music_port_accesses
                );
            }
        }
        self.reconcile_native_exact_cpu_landing_music_nmi(&trace);
        self.native_exact_cpu_host_trace = Some(trace);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn module07_call_returns_are_cpu_statements_not_nmi_counts() {
        let mut stack = Vec::new();
        let mut scan = NativeExactCpuOverworldScanTracker::default();
        let mut sprite = NativeExactCpuSpritePreparationTracker::default();
        let mut floor = NativeExactCpuFloorDrawTracker::default();
        let mut trace = NativeExactCpuHostTrace::default();
        for pc in [0x01_88e4, 0x02_8a5f, 0x02_8a67] {
            NativeExactCpuOwner::record_step(
                &mut stack,
                &mut scan,
                &mut sprite,
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
            trace.spotlight_build_progress(|rows| 224 - rows),
            Some(crate::SpotlightTableBuildProgress {
                completed_iterations: 1,
                checkpoint: crate::SpotlightTableBuildCheckpoint::AfterUpperTableWrite {
                    lower_cursor: 223,
                },
            },)
        );
        trace.return_pc = 0x00_8036;
        assert_eq!(trace.spotlight_build_progress(|_| unreachable!()), None);
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
        let mut floor = NativeExactCpuFloorDrawTracker::default();
        let mut trace = NativeExactCpuHostTrace::default();
        let caller_write = scan_step(0x0e_c9f9, Some((0x0e_0710, 2)), 10);
        NativeExactCpuOwner::record_step(
            &mut stack,
            &mut scan,
            &mut sprite,
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
        let mut floor = NativeExactCpuFloorDrawTracker::default();
        let mut trace = NativeExactCpuHostTrace::default();
        NativeExactCpuOwner::record_step(
            &mut stack,
            &mut scan,
            &mut sprite,
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
