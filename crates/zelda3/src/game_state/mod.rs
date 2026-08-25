//! Game state access layers.

pub(crate) mod constants;
mod native;
mod view;

pub(crate) use native::*;
pub use native::{
    CachedSpriteCacheField, OverworldMap16LoadState, SmallOverworldMap16ScrollBackupState,
};
pub(crate) use view::*;
