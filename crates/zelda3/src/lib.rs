//! Zelda 3 game logic. Port of the C sources under `zelda3/src/`.
//!
//! Holds the reverse-engineered game code (sprites, dungeons, overworld,
//! HUD, etc.). Designed to run in lockstep with `snes::` so each frame's
//! WRAM/SRAM/VRAM can be byte-compared against the original ROM.

#![allow(dead_code)]

pub mod config;
pub(crate) mod game_state;
pub mod oracle;
pub mod spc_player;
pub mod types;
pub mod util;
pub mod zelda_cpu_infra;
#[path = "main.rs"]
pub mod zelda_main;
pub mod zelda_rtl;

pub use zelda_cpu_infra::{
    ComparisonReport, Difference, LockstepOracle, OracleError, Region, SemanticAncillaSlot,
    SemanticComparisonReport, SemanticDifference, SemanticFrame, SemanticPlayer, SemanticPpu,
    SemanticSnapshot, SemanticSpriteSlot, SemanticWorld, RUN_MAIN, RUN_POLY,
};
pub use zelda_rtl::{OverworldMap16LoadState, ZeldaState, SRAM_SIZE, VRAM_WORDS};
