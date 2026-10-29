//! Sprite subsystem modules, shared types, property tables, and regression tests.
// Methods ported from zelda3/src/sprite.c and included inside ZeldaState.

use super::*;
use crate::game_state::constants::{SPR_RANGED_BASED_TOGGLER};
use crate::tile_definition::{EntityCollision, NativeTile};
use crate::types::{
    project_speed_from_differences, sign16, sign8, PairU8, Point16U, PointU8, ProjectSpeedRet,
    SpriteHitBox,
};

// Explicit ROM call paths. Unpriced skips added wrapper markers; existing
// scalar and instruction-stepped child costs retain their original ownership.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteDistressEntry {
    Unpriced,
    Rom06A733,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteMoveEntry {
    Unpriced,
    Rom06E93E,
    Rom05FA00,
    Rom1EFF34,
    Rom1DE95D,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteMoveZEntry {
    Unpriced,
    Rom06E96C,
    Rom1EFF62,
    Rom1DE98B,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteDamageFromEntry {
    Unpriced,
    Rom06F2AA,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteDamageToEntry {
    Unpriced,
    Rom06F121,
    Rom1EC26B,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteDamageBothEntry {
    Unpriced,
    Rom05AB93,
    Rom06D88B,
    Rom1DC211,
    Rom1EC267,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteTileCollisionEntry {
    Unpriced,
    Rom06E496,
    Rom1E8B2E,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteHitBoxEntry {
    Unpriced,
    Rom0683EA,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteHitBox00Entry {
    Unpriced,
    Rom06F1F6,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteSingleSmallEntry {
    Unpriced,
    Rom06DCEF,
    Rom06DBF8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteSingleLargeEntry {
    Unpriced,
    Rom06DBF0,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteShadowEntry {
    Unpriced,
    Rom06DC5C,
    Rom06DC54,
    Rom06DC54Default,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteInactiveEntry {
    Unpriced,
    Rom06D9EC,
    Rom05F94E,
    Rom1AF954,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpritePausedEntry {
    Unpriced,
    Rom06D9F3,
    Rom05F955,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SpriteRecoilEntry {
    Unpriced,
    Rom1EFE9B,
    Rom1DE8C5,
}

mod sprite_shared;
use sprite_shared::*;

/// Retained shadow work and its caller-owned coordinate workspace. Sprite and
/// OAM inputs are read from their native owners when each instruction retires.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct ShadowOamTail {
    work: crate::cycle_models::sprite_shadow::ShadowWork,
    slot: u8,
    x: u16,
    y: u16,
    flags: u8,
}

/// A large draw owns the table-read temporaries and the shadow it tail-calls.
/// Neither the caller nor a resumed shadow may repeat the ordinary OAM stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct SingleLargeDrawContinuation {
    prep: Option<CoordinatePrepContinuation>,
    skipped: bool,
    work: crate::cycle_models::sprite_single::SingleLargeWork,
    shadow: Option<ShadowOamTail>,
    slot: u8,
    x: u16,
    y: u16,
    flags: u8,
}

impl SingleLargeDrawContinuation {
    fn new(k: usize, info: &PrepOamCoordsRet) -> Self {
        Self {
            prep: None,
            skipped: false,
            work: crate::cycle_models::sprite_single::SingleLargeWork::new(),
            shadow: None,
            slot: u8::try_from(k).expect("sprite slot fits a byte"),
            x: info.x,
            y: info.y,
            flags: info.flags,
        }
    }

    #[cfg(test)]
    pub(super) fn coordinates_prepared(&self) -> bool { self.prep.is_none() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct CoordinatePrepContinuation {
    call_pending: bool,
    work: crate::cycle_models::sprite_prep::PrepWork,
    kill: Option<crate::cycle_models::sprite_kill::KillWork>,
}

impl CoordinatePrepContinuation {
    fn new(call_pending: bool) -> Self {
        Self { call_pending, work: crate::cycle_models::sprite_prep::PrepWork::new(), kill: None }
    }
}

// `Sprite_DrawMultiple` consumes a table of these draws (sprite.h:38-42).
#[derive(Copy, Clone)]
pub(super) struct DrawMultipleData {
    pub x: i8,
    pub y: i8,
    pub char_flags: u16,
    pub ext: u8,
}

/// A compact draw-table entry: signed pixel offsets, the OAM character and flags word,
/// and the size/extended byte.
pub(super) const fn dmd(x: i8, y: i8, char_flags: u16, ext: u8) -> DrawMultipleData {
    DrawMultipleData {
        x,
        y,
        char_flags,
        ext,
    }
}

#[derive(Copy, Clone)]
struct DrawMultipleWordData {
    x: u16,
    y: u16,
    char_flags: u16,
    ext: u8,
}

// `Sprite_SetSpawnedCoordinates` consumes this struct (sprite.h:28-34).
#[derive(Copy, Clone, Default)]
pub(super) struct SpriteSpawnInfo {
    pub r0_x: u16,
    pub r2_y: u16,
    pub r4_z: u8,
    pub r5_overlord_x: u16,
    pub r7_overlord_y: u16,
}

/// Translated state of the ROM's interruptible $09:C55E proximity scan.
/// A cell is one return from the source $09:C6F5 inner call. Keeping the
/// cursor and caller locals explicit lets the CPU owner stop the translated
/// scan at that same boundary without rerunning completed cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldProximityScanWork {
    saved_bg2_h: u16,
    saved_scroll_delta: u8,
    x_extension: u16,
    x: u16,
    y: u16,
    y_start: u16,
    rows_in_column: u8,
    columns_remaining: u8,
    cells_completed: u16,
    total_cells: u16,
    in_bounds_proximity_checks: u16,
}

impl OverworldProximityScanWork {
    pub(crate) fn total_cells(&self) -> u16 {
        self.total_cells
    }

    pub(crate) fn cells_completed(&self) -> u16 {
        self.cells_completed
    }
}

/// The caller state between the overworld presence load and the return from
/// its proximity scan. Reset and presence publication run exactly once; only
/// the per-cell scan can be resumed after an interrupt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OverworldSpriteReloadScanWork {
    sprite_records: usize,
    scan: OverworldProximityScanWork,
}

impl OverworldSpriteReloadScanWork {
    pub(crate) fn cells_completed(&self) -> u16 {
        self.scan.cells_completed()
    }

    pub(crate) fn total_cells(&self) -> u16 {
        self.scan.total_cells()
    }
}

#[derive(Copy, Clone, Default, serde::Serialize, serde::Deserialize, Debug)]
pub(super) struct PrepOamCoordsRet {
    pub x: u16,
    pub y: u16,
    pub r4: u8,
    pub flags: u8,
}

/// The ROM entry a translated caller uses to reach
/// `Sprite_PrepOamCoordOrDoubleRet` ($06:E41E). Cycle ledger only: every
/// entry computes the same result, but each wrapper is its own JSR/JSL
/// target with its own cost (see `sprite_prep_oam_coord_or_double_ret_raw_from`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrepOamCoordEntry {
    /// `JSR $06:E41E` from bank 6 (the double return skips the caller's
    /// own return).
    Direct,
    /// `JSR $06:E41A` Sprite_PrepOamCoordSafeWrapper from bank 6.
    Safe,
    /// `JSL $06:E416` Sprite_PrepOamCoord from another bank.
    Long,
    /// `JSR $05:FA50` Sprite_PrepOamCoordOrDoubleRet__ (bank 5, and every
    /// caller of Sprite_DrawMultiple $05:DF6C, whose prologue uses it).
    Bank5DoubleRet,
    /// `JSR $1D:E9AD` Sprite_PrepOamCoordOrDoubleRet___ (bank $1D).
    Bank1dDoubleRet,
    /// `JSR $1E:FF84` Sprite_PrepOamCoordOrDoubleRet____ (bank $1E).
    Bank1eDoubleRet,
    /// `JSR $05:9257` Sprite_PrepOamCoord_wrapper (bank 5), the void form
    /// of `Bank5DoubleRet`.
    Bank5Safe,
}

/// The ROM entry a caller of `Sprite_DrawMultiple` ($05:DF6C) uses. Cycle
/// ledger only: every entry draws the same table, but each is its own
/// JSL/JSR target with its own scope and entry cost (see
/// `sprite_draw_multiple_from`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DrawMultipleEntry {
    /// `JSL $05:DF6C` with the entry count in A (STA $06 : STZ $07 first).
    Direct,
    /// `JSL $05:DF70` Sprite_DrawMultiple_R6 with the count already in $06
    /// (Uncle_Draw).
    R6,
    /// `JSR $1D:8549` Sprite_DrawMultiple__ (bank $1D): JSL $05:DF6C : RTS.
    Bank1d,
}

impl PrepOamCoordsRet {
    pub(super) fn from_tuple(t: (u16, u16, u8)) -> Self {
        Self {
            x: t.0,
            y: t.1,
            r4: 0,
            flags: t.2,
        }
    }
}

/// A C statement boundary inside `Dungeon_LoadSingleSprite` (`sprite.c:3649-3660`).
///
/// The room loader can cross vblank between these writes.  Keeping the
/// checkpoint semantic lets the translated loader execute the same prefix and
/// resume the same record without replaying already-visible sprite fields.
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum DungeonSpriteLoadCheckpoint {
    State,
    TempY,
    Floor,
    YLow,
    YHigh,
    SharedX,
    XLow,
    XHigh,
    Type,
    SubtypeClear,
    TempSubtype,
    SubtypeFinal,
    SpawnIndex,
    Complete,
}

impl DungeonSpriteLoadCheckpoint {
    /// Last published statement at an instruction boundary in the normal
    /// record body of Dungeon_LoadSingleSprite. The PC is the next instruction,
    /// so a pending store still belongs to the continuation.
    pub(crate) fn from_next_source_pc(pc: u32) -> Option<Self> {
        Some(match pc {
            0x09_c38c..=0x09_c38e => Self::State,
            0x09_c391..=0x09_c395 => Self::TempY,
            0x09_c398..=0x09_c39e => Self::Floor,
            0x09_c3a1..=0x09_c3a6 => Self::YLow,
            0x09_c3a9..=0x09_c3ac => Self::YHigh,
            0x09_c3af..=0x09_c3b3 => Self::SharedX,
            0x09_c3b6..=0x09_c3bb => Self::XLow,
            0x09_c3be..=0x09_c3c1 => Self::XHigh,
            0x09_c3c4 => Self::Type,
            0x09_c3c7..=0x09_c3ce => Self::SubtypeClear,
            0x09_c3d1..=0x09_c3dc => Self::TempSubtype,
            0x09_c3df..=0x09_c3e1 => Self::SubtypeFinal,
            0x09_c3e4 => Self::SpawnIndex,
            0x09_c3e7 => Self::Complete,
            _ => return None,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DungeonLoadSpritesCpuProgress {
    /// Zero-based ordinal among normal sprite-slot publications. Room-list
    /// marker, overlord, and already-loaded records do not publish
    /// `sprite_state` and therefore do not consume an ordinal.
    pub normal_load_ordinal: u16,
    pub slot: u8,
    pub checkpoint: DungeonSpriteLoadCheckpoint,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DungeonSpriteDisableCpuProgress {
    /// The descending live-sprite loop has visited slots 15 through `slot`.
    SpriteStatesThrough { slot: u8 },
    /// The live-sprite loop is complete and the descending ancilla loop has
    /// cleared slots 9 through `slot`.
    AncillasThrough { slot: u8 },
    /// Both descending loops are complete and the held-ancilla flag is clear.
    AncillaPickupFlagCleared,
    /// `sprite_limit_instance = 0` has published after the two loops.
    SpriteLimitInstanceCleared,
}

impl DungeonSpriteDisableCpuProgress {
    const fn source_ordinal(self) -> u8 {
        match self {
            Self::SpriteStatesThrough { slot } => 15 - slot,
            Self::AncillasThrough { slot } => 16 + (9 - slot),
            Self::AncillaPickupFlagCleared => 26,
            Self::SpriteLimitInstanceCleared => 27,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DungeonResetSpritesCpuProgress {
    Cache {
        slot: u8,
        field: CachedSpriteCacheField,
    },
    /// Source-level progress within `Sprite_DisableAll`. This deliberately
    /// describes Zelda statements rather than the backend PC that observed
    /// them, so a native timing owner can later publish the same receipt.
    Disable(DungeonSpriteDisableCpuProgress),
    /// Descending garnish-type clear: this slot and all higher slots have
    /// published; lower slots still belong to the interrupted reset body.
    GarnishTypesThrough { slot: u8 },
    /// `Sprite_DisableAll` has completed, including its final garnish-slot
    /// clear, but the remaining room-history bookkeeping and new room sprite
    /// load have not yet published.
    SpritesDisabled,
    /// `Sprite_DisableAll` has returned and the caller has published
    /// `sprcoll_x_size = 0xffff`; the paired Y-size store is still pending.
    CollisionXSizeSet,
    /// Both collision-size stores have published. The following four-entry
    /// room-history search is read-only, so this is also the resumable source
    /// checkpoint anywhere inside that search before its first mutation.
    RoomHistorySearchStarted,
    Load(DungeonLoadSpritesCpuProgress),
    /// Room origin and sorting setting are published, but the first room
    /// record has not yet mutated a sprite or an overlord.
    LoadStarted,
    /// Reset and room history have completed; the loader is reading its
    /// pointer and room number before the first room-origin store.
    LoadBeforeOrigin,
}

fn hex_nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 0,
    }
}

fn sprite_init_value(table: usize, ty: u8) -> u8 {
    let idx = (table * SPRITE_INIT_TABLE_LEN + ty as usize) * 2;
    let bytes = SPRITE_INIT_TABLES_C_HEX.as_bytes();
    (hex_nibble(bytes[idx]) << 4) | hex_nibble(bytes[idx + 1])
}

fn empty_sprite_hit_box() -> SpriteHitBox {
    SpriteHitBox {
        r0_xlo: 0,
        r8_xhi: 0,
        r1_ylo: 0,
        r9_yhi: 0,
        r2: 0,
        r3: 0,
        r4_spr_xlo: 0,
        r10_spr_xhi: 0,
        r5_spr_ylo: 0,
        r11_spr_yhi: 0,
        r6_spr_xsize: 0,
        r7_spr_ysize: 0,
    }
}

#[path = "sprite/loading.rs"]
mod loading;
#[path = "sprite/runtime.rs"]
mod runtime;
#[path = "sprite/continuation.rs"]
mod continuation;
#[path = "sprite/garnish.rs"]
mod garnish;
#[path = "sprite/lifecycle.rs"]
mod lifecycle;
#[path = "sprite/collision.rs"]
mod collision;
#[path = "sprite/drawing.rs"]
mod drawing;
#[path = "sprite/movement_spawn.rs"]
mod movement_spawn;

#[cfg(test)]
#[path = "sprite_tests.rs"]
mod tests;
