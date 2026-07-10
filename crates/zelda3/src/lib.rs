//! Zelda 3 game logic. Port of the C sources under `zelda3/src/`.
//!
//! Holds the reverse-engineered game code (sprites, dungeons, overworld,
//! HUD, etc.). Designed to run in lockstep with `snes::` so each frame's
//! WRAM/SRAM/VRAM can be byte-compared against the original ROM.

#![allow(dead_code)]

pub mod chr_source;
pub mod config;
pub mod game_output;
pub(crate) mod game_state;
pub mod modern_audio;
pub mod modern_audio_sequence;
pub mod modern_sfx_catalog;
pub mod oracle;
pub mod spc_player;
pub mod types;
pub mod util;
pub mod zelda_cpu_infra;
#[path = "main.rs"]
pub mod zelda_main;
pub mod zelda_rtl;

pub use chr_source::{
    chr_content_hash32, LogicalChrSrc, VramChrSourceTable, CHR_KIND_BG, CHR_KIND_BG3,
    CHR_KIND_BG_ANIM, CHR_KIND_BG_STREAM, CHR_KIND_LINK, CHR_KIND_LINK_CONTENT, CHR_KIND_NONE,
    CHR_KIND_SPRITE, CHR_LINK_SRC_RAM_FLAG, VRAM_CHR_SLOTS,
};
pub use game_state::{OverworldMap16LoadState, SmallOverworldMap16ScrollBackupState};
pub use zelda3_dialogue as dialogue_ir;
pub use zelda_cpu_infra::{
    ComparisonReport, Difference, LockstepOracle, OracleError, Region, SemanticAncillaSlot,
    SemanticComparisonReport, SemanticDifference, SemanticFrame, SemanticPlayer, SemanticPpu,
    SemanticSnapshot, SemanticSpriteSlot, SemanticWorld, RUN_MAIN, RUN_POLY,
};
pub use zelda_rtl::{Bg3VwfGlyphRun, ZeldaState, SRAM_SIZE, VRAM_WORDS};
