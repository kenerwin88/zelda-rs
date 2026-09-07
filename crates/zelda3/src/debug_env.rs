//! Debug and trace environment switches.
//!
//! Every `ZELDA3_DEBUG_*`, `ZELDA3_TRACE_*`, `ZELDA3_REPLAY_*`,
//! `ZELDA3_ASSERT_*`, `ZELDA3_WW_*`, `ZELDA3_AUDIT_*`, `ZELDA3_CAPTURE_*` and
//! `ZELDA3_SCRATCH_CONFLICT_*` read in this crate goes through here. With the
//! `parity-debug` cargo feature (on by default, used by the parity profile and
//! the tests) these are plain environment lookups; without it they are
//! compile-time `None`, so release binaries carry none of the tracing
//! branches. Behavior-affecting knobs (`ZELDA3_SAVE_DIR`,
//! `ZELDA3_PARITY_RUNTIME_CONFIG`, the `ZELDA3_SNES9X_*`/`ZELDA3_ROM_*` timing
//! knobs) deliberately do NOT use this module.

use std::env::VarError;
use std::ffi::OsString;

/// Whether the debug switches are compiled in.
pub const ENABLED: bool = cfg!(feature = "parity-debug");

#[inline]
pub fn var_os(name: &str) -> Option<OsString> {
    if ENABLED {
        std::env::var_os(name)
    } else {
        None
    }
}

#[inline]
pub fn var(name: &str) -> Result<String, VarError> {
    if ENABLED {
        std::env::var(name)
    } else {
        Err(VarError::NotPresent)
    }
}

/// `true` when the switch is present in the environment (any value).
#[inline]
pub fn is_set(name: &str) -> bool {
    var_os(name).is_some()
}
