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
    DungeonLoadSpritesCpuProgress, DungeonResetSpritesCpuProgress,
    DungeonResetSpritesProgressReceipt, DungeonSpriteDisableCpuProgress,
    DungeonSpriteLoadCheckpoint, MainLoopInterruption, OriginalTimingBoundary,
    OriginalTimingSemanticReceipt,
};

const TRACE_PATH_ENV: &str = "ZELDA3_SNES9X_TRACE";
const TRACE_EVENTS_ENV: &str = "ZELDA3_SNES9X_TRACE_EVENTS";
const TRACE_WRAM_ENV: &str = "ZELDA3_SNES9X_TRACE_WRAM";

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
const NMI_PREPARE_SPRITES_START_PC: u32 = 0x0085fc;
const NMI_PREPARE_SPRITES_END_PC: u32 = 0x008901;
const LINK_OAM_START_PC: u32 = 0x0da18e;
const LINK_OAM_END_PC: u32 = 0x0dadb6;
const UNCACHE_SPRITE_START_PC: u32 = 0x1dea00;
const UNCACHE_SPRITE_RESTORE_START_PC: u32 = 0x1deb06;
const UNCACHE_SPRITE_END_PC: u32 = 0x1deb68;
const SPRITE_STATE_BASE: u16 = 0x0dd0;
const SPRITE_Y_HIGH_BASE: u16 = 0x0d20;

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
}

#[derive(Deserialize)]
struct RawTraceEvent {
    event: String,
    #[serde(default)]
    pc: Option<u32>,
    #[serde(default)]
    x: Option<u16>,
    #[serde(default)]
    address: Option<u16>,
    #[serde(default)]
    value: Option<u8>,
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
                append_csv(env::var(TRACE_EVENTS_ENV).ok().as_deref(), &["nmi", "wram"]),
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
                        "02ec",
                        "0b00-0b1d",
                        "0b6a",
                        "0b89-0b98",
                        "0ba0-0baf",
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
        })
    }

    pub(crate) fn read_after_host_call(
        &mut self,
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
            self.consume_event(event, &mut receipts)?;
        }
        // `retro_run` may return at the SCAN_KEYS boundary without accepting
        // an NMI.  A synchronous Zelda call can therefore remain suspended at
        // a source-visible write even though no `nmi` trace row closed the
        // interval.  Publish that same semantic progress at every host return;
        // the following host reconstructs continuation order from its next
        // observed write, so no CPU address or call-stack state escapes this
        // adapter.
        self.flush_host_boundary_progress(&mut receipts, OriginalTimingBoundary::HostReturn);
        Ok(receipts)
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
                self.flush_host_boundary_progress(receipts, OriginalTimingBoundary::NmiAccepted);
                receipts.push(OriginalTimingSemanticReceipt::NmiAccepted);
                if let Some(pc) = event.pc.map(|pc| pc & 0x00ff_ffff) {
                    let phase = if (LINK_OAM_START_PC..LINK_OAM_END_PC).contains(&pc) {
                        Some(MainLoopInterruption::LinkOam)
                    } else if (NMI_PREPARE_SPRITES_START_PC..NMI_PREPARE_SPRITES_END_PC)
                        .contains(&pc)
                    {
                        Some(MainLoopInterruption::SpritePreparation)
                    } else {
                        None
                    };
                    if let Some(phase) = phase {
                        receipts.push(OriginalTimingSemanticReceipt::MainLoopInterrupted(phase));
                    }
                }
            }
            _ => {}
        }
        Ok(())
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
            pc,
            x,
            address,
            value: address.map(|_| 0),
        }
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
    fn cached_sprite_load_and_restore_writes_become_semantic_progress() {
        let mut tracker = Snes9xOracleSemanticTrace {
            path: PathBuf::new(),
            offset: 0,
            cache_write_progress: None,
            normal_load_ordinal: None,
            pending_reset_progress: None,
            cached_sprite_execution: None,
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
