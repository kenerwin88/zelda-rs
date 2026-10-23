//! Player subsystem modules, shared tables, and regression tests.
// Methods ported from zelda3/src/player.c and included inside ZeldaState.

use super::*;

#[path = "player/runtime.rs"]
mod runtime;
#[path = "player/movement.rs"]
mod movement;
#[path = "player/collision.rs"]
mod collision;
#[path = "player/interaction.rs"]
mod interaction;
#[path = "player/items.rs"]
mod items;
#[path = "player/actions.rs"]
mod actions;

mod player_shared;
use player_shared::*;

const DASH_FOLLOWER_SLOWDOWN_INDICATORS: [u8; 15] =
    [0xff, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const DASH_FOLLOWER_RELEASE_INDICATORS: [u8; 15] = [0xff, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

#[cfg(test)]
#[path = "player_tests.rs"]
mod tests;
