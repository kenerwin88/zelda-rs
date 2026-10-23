//! Ported SpriteDraw_* helpers from sprite_main.c.
//!
//! Each method preserves a 1:1 mapping to the C source. The original C body
//! is reproduced as a comment block immediately above each port so a
//! reviewer can verify behavior line-by-line.
//!
//! The cucco's `_for_draw` helpers at the bottom of the file are the split
//! module's own subtype continuations; shared OAM and overworld helpers are
//! called by their canonical names.
//!
//! `SpriteDraw_BombGuard_Arm` (sprite_main.c:4527) lives in
//! `sprite_main_guard.rs` already; this file does NOT redefine it.

use super::sprite::SingleLargeDrawContinuation;
#[cfg(test)]
use super::sprite::PrepOamCoordsRet;
use super::*;
use crate::rom_random::RomRandomResult;

// ---------------------------------------------------------------------------
mod sprite_main_draw_shared;
use sprite_main_draw_shared::*;

/// `Lanmola_Draw` prologue stores (see `lanmola_draw_prefix_stores`).
pub(super) const LANMOLA_DRAW_PREFIX_STORES: u8 = 5;
pub(super) use sprite_main_draw_shared::{arrgi_sin, trinexx_head_sin};

/// ROM $05:a8f0 calls the RNG, then executes `AND #$7f; ADC #$40`.
/// `AND` preserves the carry produced by the RNG routine, so the delay is
/// occasionally one greater than a carry-blind source translation suggests.
const fn rat_random_run_delay(random: RomRandomResult) -> u8 {
    random.masked_adc(0x7f, 0x40)
}

const fn rope_random_run_delay(random: RomRandomResult) -> u8 {
    random.masked_adc(0x7f, 0x40)
}

/// Cucco's call to the shared draw owns both the suspended callee and the
/// slot to which it must return. No state/type dispatch is repeated on resume.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct CuccoDrawContinuation {
    slot: u8,
    prefix: crate::cycle_models::cucco::DrawPrefixWork,
    draw: Option<SingleLargeDrawContinuation>,
}

impl CuccoDrawContinuation {
    pub(super) const fn slot(&self) -> u8 { self.slot }
}

#[path = "sprite_main_draw/objects.rs"]
mod objects;
#[path = "sprite_main_draw/hazards.rs"]
mod hazards;
#[path = "sprite_main_draw/aquatic.rs"]
mod aquatic;
#[path = "sprite_main_draw/soldiers.rs"]
mod soldiers;
#[path = "sprite_main_draw/overworld_enemies.rs"]
mod overworld_enemies;
#[path = "sprite_main_draw/bosses.rs"]
mod bosses;
#[path = "sprite_main_draw/dispatch_effects.rs"]
mod dispatch_effects;
#[path = "sprite_main_draw/dungeon_enemies.rs"]
mod dungeon_enemies;
#[path = "sprite_main_draw/wildlife.rs"]
mod wildlife;
#[path = "sprite_main_draw/npcs.rs"]
mod npcs;
#[path = "sprite_main_draw/shops_items.rs"]
mod shops_items;


#[cfg(test)]
#[path = "sprite_main_draw_tests.rs"]
mod tests;

/// The OAM y of a Moldorm eye. Both eye loops (`$1D:F856` Mini Moldorm,
/// `$1D:DBD3` Giant Moldorm) follow the eye offset's `ADC` with `ADC #$0010`
/// and no `CLC`, so that addition's carry joins the off-screen test; every
/// other sprite clip test clears carry first (C `ClampYForOam`).
fn moldorm_eye_oam_y(base_y: u16, offset: u16) -> u8 {
    let (y, carry) = base_y.overflowing_add(offset);
    if y.wrapping_add(0x10 + u16::from(carry)) < 0x100 { y as u8 } else { 0xf0 }
}
