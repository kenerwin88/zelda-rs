//! Source-statement progress through `Dungeon_ResetSprites` ($09:C114).
//!
//! The reset caches the old room's sprites, runs `Sprite_DisableAll`, rotates
//! the room history and loads the new room's sprite list. NMI can interrupt
//! any of these statements. This tracker names the last completed source
//! statement from the CPU's WRAM stores and its interrupted PC, so every
//! execution backend (the pinned Snes9x trace adapter and the native ROM
//! timing shadow) publishes the same typed `DungeonResetSpritesCpuProgress`.
//!
//! Write PCs are the address of the instruction *after* the store, as both
//! backends report them.

use serde::{Deserialize, Serialize};

use crate::{
    CachedSpriteCacheField, DungeonLoadSpritesCpuProgress, DungeonResetSpritesCpuProgress,
    DungeonSpriteDisableCpuProgress, DungeonSpriteLoadCheckpoint,
};

pub const DUNGEON_CACHE_TRANS_SPRITES_START_PC: u32 = 0x09c176;
pub const DUNGEON_CACHE_TRANS_SPRITES_END_PC: u32 = 0x09c244;
pub const DUNGEON_RESET_SPRITES_CLEAR_PC: u32 = 0x09c244;
// After Sprite_DisableAll returns, Dungeon_ResetSprites publishes the paired
// collision sizes and performs a read-only four-entry room-history search.
// An interrupt anywhere in the search can resume from its source start: no
// history mutation has occurred yet. These PCs distinguish that caller
// checkpoint from the completed callee boundary at $09:C244..C290.
pub const DUNGEON_RESET_SPRITES_AFTER_DISABLE_PC: u32 = 0x09c124;
pub const DUNGEON_RESET_SPRITES_COLLISION_Y_STORE_PC: u32 = 0x09c12c;
pub const DUNGEON_RESET_SPRITES_HISTORY_SEARCH_START_PC: u32 = 0x09c12f;
pub const DUNGEON_RESET_SPRITES_HISTORY_FIRST_MUTATION_PC: u32 = 0x09c148;
pub const DUNGEON_RESET_SPRITES_HISTORY_FOUND_PC: u32 = 0x09c16e;
pub const DUNGEON_RESET_SPRITES_LOAD_CALL_PC: u32 = 0x09c170;
/// First instruction after Dungeon_LoadSprites returns to Dungeon_ResetSprites.
/// Any partial cache/disable/load cursor is retired here even when the room had
/// no new sprite-record writes to supersede it.
pub const DUNGEON_RESET_SPRITES_RETURN_PC: u32 = 0x09c173;
pub const SPRITE_DISABLE_ALL_END_PC: u32 = 0x09c290;
pub const SPRITE_DISABLE_ALL_FINAL_GARNISH_PC: u32 = 0x09c281;
pub const GARNISH_TYPE_SLOT_ZERO: u16 = 0x0b00;
pub const ANCILLA_TYPE_BASE: u16 = 0x0c4a;
pub const ANCILLA_PICKUP_FLAG: u16 = 0x02ec;
pub const SPRITE_LIMIT_INSTANCE: u16 = 0x0b6a;
pub const DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC: u32 = 0x09c38c;
pub const DUNGEON_LOAD_SINGLE_SPRITE_TEMP_Y_PC: u32 = 0x09c391;
pub const DUNGEON_LOAD_SINGLE_SPRITE_FLOOR_PC: u32 = 0x09c398;
pub const DUNGEON_LOAD_SINGLE_SPRITE_Y_LOW_PC: u32 = 0x09c3a1;
pub const DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC: u32 = 0x09c3a9;
pub const DUNGEON_LOAD_SINGLE_SPRITE_SHARED_X_PC: u32 = 0x09c3af;
pub const DUNGEON_LOAD_SINGLE_SPRITE_X_LOW_PC: u32 = 0x09c3b6;
pub const DUNGEON_LOAD_SINGLE_SPRITE_X_HIGH_PC: u32 = 0x09c3be;
pub const DUNGEON_LOAD_SINGLE_SPRITE_TYPE_PC: u32 = 0x09c3c4;
pub const DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_CLEAR_PC: u32 = 0x09c3c7;
pub const DUNGEON_LOAD_SINGLE_SPRITE_TEMP_SUBTYPE_PC: u32 = 0x09c3d1;
pub const DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_FINAL_PC: u32 = 0x09c3df;
pub const DUNGEON_LOAD_SINGLE_SPRITE_SPAWN_INDEX_PC: u32 = 0x09c3e4;
pub const DUNGEON_LOAD_SINGLE_SPRITE_COMPLETE_PC: u32 = 0x09c3e7;
pub const DUNGEON_LOAD_SINGLE_SPRITE_END_PC: u32 = 0x09c3e8;
pub const SPRITE_STATE_BASE: u16 = 0x0dd0;
pub const SPRITE_Y_LOW_BASE: u16 = 0x0d00;
pub const SPRITE_Y_HIGH_BASE: u16 = 0x0d20;
pub const SPRITE_X_LOW_BASE: u16 = 0x0d10;
pub const SPRITE_X_HIGH_BASE: u16 = 0x0d30;
pub const SPRITE_N_WORD_BASE: u16 = 0x0bc0;
pub const SPRITE_TYPE_BASE: u16 = 0x0e20;
pub const SPRITE_SUBTYPE_BASE: u16 = 0x0e30;
pub const SPRITE_FLOOR_BASE: u16 = 0x0f20;
pub const SPRITE_DIE_ACTION_BASE: u16 = 0x0cba;
pub const DUNGEON_LOAD_TEMP_Y: u16 = 0x0fb5;
pub const DUNGEON_LOAD_SHARED_X: u16 = 0x0fb6;

/// `Dungeon_CacheTransSprites` store order for one slot.
pub const CACHE_FIELD_WRITES: [(CachedSpriteCacheField, u16); 25] = [
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
pub struct CacheWriteProgress {
    pub slot: u8,
    pub next_field_index: usize,
}

/// Write-driven cursor through one `Dungeon_ResetSprites` call.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DungeonResetSpritesProgressTracker {
    pub cache_write_progress: Option<CacheWriteProgress>,
    pub normal_load_ordinal: Option<u16>,
    pub pending_reset_progress: Option<DungeonResetSpritesCpuProgress>,
}

impl DungeonResetSpritesProgressTracker {
    /// The reset returned to its caller; no partial statement remains.
    pub fn retire_at_return(&mut self) {
        self.pending_reset_progress = None;
        self.cache_write_progress = None;
        self.normal_load_ordinal = None;
    }

    /// Observe one completed CPU WRAM store. `pc` is the PC after the store
    /// and `x` the index register it used. `foreign_write` is true when a
    /// different source domain (a cached-sprite swap) owns this store; such a
    /// store can still supersede a reset candidate but never advances it.
    pub fn observe_write(
        &mut self,
        pc: u32,
        address: u16,
        value: Option<u8>,
        x: Option<u16>,
        foreign_write: bool,
    ) -> Result<(), String> {
        let disable_progress = sprite_disable_progress(pc, address, value)?;
        // `SpritesDisabled` is a candidate for a host boundary at the final
        // Sprite_DisableAll write, not a durable description of the rest of
        // Dungeon_ResetSprites. Once any later source write is observed,
        // execution has advanced beyond that exact statement. Drop the
        // candidate fail-closed; a more precise cache/load receipt below may
        // replace it.
        let final_garnish_clear = pc == SPRITE_DISABLE_ALL_FINAL_GARNISH_PC
            && x == Some(0)
            && address == GARNISH_TYPE_SLOT_ZERO;
        if matches!(
            self.pending_reset_progress,
            Some(DungeonResetSpritesCpuProgress::SpritesDisabled)
        ) && !final_garnish_clear
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
        if foreign_write {
            return Ok(());
        }
        if let Some(progress) = disable_progress {
            self.cache_write_progress = None;
            self.normal_load_ordinal = None;
            self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::Disable(progress));
        } else if final_garnish_clear {
            self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::SpritesDisabled);
        } else if (DUNGEON_CACHE_TRANS_SPRITES_START_PC..DUNGEON_CACHE_TRANS_SPRITES_END_PC)
            .contains(&pc)
            && CACHE_FIELD_WRITES
                .iter()
                .any(|&(_, base)| (base..base + 16).contains(&address))
        {
            self.observe_cache_write(address, x)?;
        } else if pc == DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC
            && (SPRITE_STATE_BASE..SPRITE_STATE_BASE + 16).contains(&address)
        {
            let slot = (address - SPRITE_STATE_BASE) as u8;
            if x != Some(u16::from(slot)) {
                return Err(format!(
                    "Dungeon_LoadSingleSprite state write disagrees on slot: x={x:?}, address=${address:04x}",
                ));
            }
            let ordinal = self
                .normal_load_ordinal
                .map_or(0, |ordinal| ordinal.saturating_add(1));
            self.normal_load_ordinal = Some(ordinal);
            self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::Load(
                DungeonLoadSpritesCpuProgress {
                    normal_load_ordinal: ordinal,
                    slot,
                    checkpoint: DungeonSpriteLoadCheckpoint::State,
                },
            ));
        } else if (DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC..DUNGEON_LOAD_SINGLE_SPRITE_END_PC)
            .contains(&pc)
        {
            let Some((slot, checkpoint)) =
                dungeon_load_single_sprite_write_progress(pc, address, x)?
            else {
                return Err(format!(
                    "Dungeon_LoadSingleSprite wrote unsupported source field ${address:04x} at ${pc:06x}",
                ));
            };
            let normal_load_ordinal = self
                .normal_load_ordinal
                .ok_or("Dungeon_LoadSingleSprite field observed before record state")?;
            self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::Load(
                DungeonLoadSpritesCpuProgress {
                    normal_load_ordinal,
                    slot,
                    checkpoint,
                },
            ));
        }
        Ok(())
    }

    fn observe_cache_write(&mut self, address: u16, x: Option<u16>) -> Result<(), String> {
        let slot = u8::try_from(x.ok_or("Dungeon_CacheTransSprites write omitted X")?)
            .map_err(|_| "Dungeon_CacheTransSprites X exceeded one byte")?;
        if slot >= 16 {
            return Err(format!(
                "Dungeon_CacheTransSprites slot {slot} is outside 0..16"
            ));
        }
        let progress = match self.cache_write_progress {
            Some(progress) if progress.slot == slot => progress,
            Some(progress) if slot < progress.slot => CacheWriteProgress {
                slot,
                next_field_index: 0,
            },
            // A completed call may have no later traced reset write
            // (Sprite_DisableAll stores only active slots). The next source
            // call is nevertheless unambiguous: its descending C loop begins
            // with slot 15's StateClear publication.
            Some(_) if slot == 15 && address == 0x1d0f => CacheWriteProgress {
                slot,
                next_field_index: 0,
            },
            Some(progress) => {
                return Err(format!(
                    "Dungeon_CacheTransSprites slot order advanced from {} to {slot}",
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
            .ok_or("Dungeon_CacheTransSprites wrote past the final field")?;
        let expected_address = base + u16::from(slot);
        if address != expected_address {
            return Err(format!(
                "Dungeon_CacheTransSprites field {field:?} for slot {slot} expected ${expected_address:04x}, observed ${address:04x}"
            ));
        }
        self.cache_write_progress = Some(CacheWriteProgress {
            slot,
            next_field_index: progress.next_field_index + 1,
        });
        self.pending_reset_progress = Some(DungeonResetSpritesCpuProgress::Cache { slot, field });
        Ok(())
    }

    /// Refine the pending candidate at an NMI acceptance or host return whose
    /// interrupted PC lies in a non-mutating stretch of the reset caller.
    pub fn observe_boundary(&mut self, pc: u32, a: Option<u16>, y: Option<u16>) {
        if let Some(progress) = dungeon_reset_sprites_caller_progress(pc, a, y) {
            self.pending_reset_progress = Some(progress);
        }
    }
}

pub fn sprite_disable_progress(
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
            "Sprite_DisableAll progress wrote nonzero value {value:?} to ${address:04x}",
        ));
    }
    Ok(progress)
}

pub fn dungeon_load_single_sprite_write_progress(
    pc: u32,
    address: u16,
    x: Option<u16>,
) -> Result<Option<(u8, DungeonSpriteLoadCheckpoint)>, String> {
    let slot = u8::try_from(x.ok_or("Dungeon_LoadSingleSprite field write omitted source slot X")?)
        .map_err(|_| "Dungeon_LoadSingleSprite source slot exceeded one byte")?;
    if slot >= 16 {
        return Err(format!(
            "Dungeon_LoadSingleSprite used invalid source slot {slot}",
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

/// Caller-level progress at an interrupted PC with no store of its own.
pub fn dungeon_reset_sprites_caller_progress(
    pc: u32,
    a: Option<u16>,
    y: Option<u16>,
) -> Option<DungeonResetSpritesCpuProgress> {
    let pc = pc & 0x00ff_ffff;
    // The history rotation has finished. CMP #$ffff / BEQ skips the
    // evicted-room death-mask clear for an empty history entry, leaving no
    // gameplay writes before Dungeon_LoadSprites starts reading its pointer.
    if matches!(pc, 0x09_c163 | 0x09_c166) && a == Some(0xffff) {
        return Some(DungeonResetSpritesCpuProgress::LoadBeforeOrigin);
    }
    // First Dungeon_LoadSingleSprite call, after its two INYs and type
    // read, before either the marker branch or normal-slot publication.
    if (0x09_c32b..0x09_c330).contains(&pc) && y == Some(3) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Store every Dungeon_LoadSingleSprite field of one normal record through
    /// `last`, with the post-store PCs and addresses the ROM uses at $09:C38C..
    fn load_record_through(
        tracker: &mut DungeonResetSpritesProgressTracker,
        slot: u8,
        y: u8,
        last: DungeonSpriteLoadCheckpoint,
    ) {
        let x = Some(u16::from(slot));
        let s = u16::from(slot);
        let stores = [
            (DUNGEON_LOAD_SINGLE_SPRITE_STATE_PC, SPRITE_STATE_BASE + s, 8, DungeonSpriteLoadCheckpoint::State),
            (DUNGEON_LOAD_SINGLE_SPRITE_TEMP_Y_PC, DUNGEON_LOAD_TEMP_Y, y, DungeonSpriteLoadCheckpoint::TempY),
            (DUNGEON_LOAD_SINGLE_SPRITE_FLOOR_PC, SPRITE_FLOOR_BASE + s, y >> 7, DungeonSpriteLoadCheckpoint::Floor),
            (DUNGEON_LOAD_SINGLE_SPRITE_Y_LOW_PC, SPRITE_Y_LOW_BASE + s, y << 4, DungeonSpriteLoadCheckpoint::YLow),
            (DUNGEON_LOAD_SINGLE_SPRITE_Y_HIGH_PC, SPRITE_Y_HIGH_BASE + s, 0, DungeonSpriteLoadCheckpoint::YHigh),
            (DUNGEON_LOAD_SINGLE_SPRITE_SHARED_X_PC, DUNGEON_LOAD_SHARED_X, 0, DungeonSpriteLoadCheckpoint::SharedX),
            (DUNGEON_LOAD_SINGLE_SPRITE_X_LOW_PC, SPRITE_X_LOW_BASE + s, 0, DungeonSpriteLoadCheckpoint::XLow),
            (DUNGEON_LOAD_SINGLE_SPRITE_X_HIGH_PC, SPRITE_X_HIGH_BASE + s, 0, DungeonSpriteLoadCheckpoint::XHigh),
            (DUNGEON_LOAD_SINGLE_SPRITE_TYPE_PC, SPRITE_TYPE_BASE + s, 0, DungeonSpriteLoadCheckpoint::Type),
            (DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_CLEAR_PC, SPRITE_SUBTYPE_BASE + s, 0, DungeonSpriteLoadCheckpoint::SubtypeClear),
            (DUNGEON_LOAD_SINGLE_SPRITE_TEMP_SUBTYPE_PC, DUNGEON_LOAD_TEMP_Y, 0, DungeonSpriteLoadCheckpoint::TempSubtype),
            (DUNGEON_LOAD_SINGLE_SPRITE_SUBTYPE_FINAL_PC, SPRITE_SUBTYPE_BASE + s, 0, DungeonSpriteLoadCheckpoint::SubtypeFinal),
            (DUNGEON_LOAD_SINGLE_SPRITE_SPAWN_INDEX_PC, SPRITE_N_WORD_BASE + s, slot, DungeonSpriteLoadCheckpoint::SpawnIndex),
            (DUNGEON_LOAD_SINGLE_SPRITE_COMPLETE_PC, SPRITE_DIE_ACTION_BASE + s, 0, DungeonSpriteLoadCheckpoint::Complete),
        ];
        for (pc, address, value, checkpoint) in stores {
            tracker.observe_write(pc, address, Some(value), x, false).unwrap();
            if checkpoint == last {
                return;
            }
        }
    }

    #[test]
    fn caller_nmi_inside_second_record_names_its_last_completed_store() {
        // Route host 23194: the source NMI interrupted $09:C3AC, after slot 1
        // published `$0FB5 = 0x99` (at $09:C38E) and its Y high byte, before
        // the shared X store. Both the Snes9x adapter and the native room-load
        // shadow must resume that record after YHigh, not re-run it whole.
        let mut tracker = DungeonResetSpritesProgressTracker::default();
        load_record_through(&mut tracker, 0, 0x28, DungeonSpriteLoadCheckpoint::Complete);
        load_record_through(&mut tracker, 1, 0x99, DungeonSpriteLoadCheckpoint::YHigh);
        tracker.observe_boundary(0x09_c3ac, Some(0), Some(6));
        assert_eq!(
            tracker.pending_reset_progress,
            Some(DungeonResetSpritesCpuProgress::Load(DungeonLoadSpritesCpuProgress {
                normal_load_ordinal: 1,
                slot: 1,
                checkpoint: DungeonSpriteLoadCheckpoint::YHigh,
            })),
        );
        tracker.retire_at_return();
        assert_eq!(tracker, DungeonResetSpritesProgressTracker::default());
    }

    #[test]
    fn foreign_store_supersedes_but_never_advances_a_disable_candidate() {
        let mut tracker = DungeonResetSpritesProgressTracker::default();
        tracker
            .observe_write(DUNGEON_RESET_SPRITES_CLEAR_PC, SPRITE_STATE_BASE + 15, Some(0), Some(15), false)
            .unwrap();
        assert!(matches!(
            tracker.pending_reset_progress,
            Some(DungeonResetSpritesCpuProgress::Disable(_))
        ));
        tracker.observe_write(0x1d_ea10, 0x0dd3, Some(9), Some(3), true).unwrap();
        assert_eq!(tracker.pending_reset_progress, None);
    }
}
