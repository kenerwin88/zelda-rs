//! Byte-backed typed views over game state.

use crate::types::{read_le_u16, write_le_u16};

use crate::game_state::constants::*;

mod compatibility;
mod frame;
mod player;
mod sprites;

pub(crate) use compatibility::*;
#[allow(unused_imports)]
pub(crate) use frame::*;
pub(crate) use player::*;
#[allow(unused_imports)]
pub(crate) use sprites::*;

fn byte(ram: &[u8], offset: usize) -> u8 {
    ram.get(offset).copied().unwrap_or(0)
}
