//! Debug and trace environment switches.
//!
//! Every `ZELDA3_DEBUG_*`, `ZELDA3_TRACE_*`, `ZELDA3_REPLAY_*`,
//! `ZELDA3_ASSERT_*`, `ZELDA3_WW_*`, `ZELDA3_AUDIT_*`, `ZELDA3_CAPTURE_*` and
//! `ZELDA3_SCRATCH_CONFLICT_*` read in this crate goes through here. With the
//! `parity-debug` cargo feature (on by default, used by the parity profile and
//! the tests) these snapshot each variable on first read; without it they are
//! compile-time `None`, so release binaries carry none of the tracing
//! branches. Behavior-affecting knobs (`ZELDA3_SAVE_DIR`, the
//! `ZELDA3_SNES9X_*`/`ZELDA3_ROM_*` timing knobs) deliberately do NOT use
//! this module.

use std::env::VarError;
use std::ffi::OsString;

/// Whether this crate's debug switches are compiled in.
pub const ENABLED: bool = cfg!(feature = "parity-debug");

// Pass this crate's feature gate explicitly: another workspace crate enabling
// the shared cache must not enable Zelda diagnostics in a feature-off build.
#[macro_export]
macro_rules! zelda3_debug_var {
    ($name:literal) => { $crate::debug_env::shared::var!($name, $crate::debug_env::ENABLED) };
}

#[macro_export]
macro_rules! zelda3_debug_var_os {
    ($name:literal) => { $crate::debug_env::shared::var_os!($name, $crate::debug_env::ENABLED) };
}

#[doc(hidden)]
pub use zelda3_debug_env as shared;
pub use crate::{zelda3_debug_var as var, zelda3_debug_var_os as var_os};

#[inline]
pub fn var_os(name: &str) -> Option<OsString> {
    if ENABLED { zelda3_debug_env::var_os(name) } else { None }
}

#[inline]
pub fn var(name: &str) -> Result<String, VarError> {
    if ENABLED { zelda3_debug_env::var(name) } else { Err(VarError::NotPresent) }
}

#[inline]
pub fn is_set(name: &str) -> bool {
    ENABLED && zelda3_debug_env::is_set(name)
}
