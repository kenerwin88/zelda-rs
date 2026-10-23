//! Debug switches snapshot each environment variable on its first read.
//!
//! All three accessors share one process-wide cache, including absent and
//! non-Unicode values. Later environment mutations are deliberately ignored.
//! User configuration and behavior-affecting options must not use this cache.

use std::env::VarError;
use std::ffi::OsString;

pub const ENABLED: bool = cfg!(feature = "parity-debug");

/// Literal-name hot path: after initialization, only a `OnceLock` read and
/// (for present values) a clone. The function cache still owns the snapshot
/// shared by different accessors and call sites.
#[macro_export]
macro_rules! var_os {
    ($name:literal) => { $crate::var_os!($name, $crate::ENABLED) };
    ($name:literal, $enabled:expr) => {{
        if $enabled {
            static VALUE: std::sync::OnceLock<Option<std::ffi::OsString>> = std::sync::OnceLock::new();
            VALUE.get_or_init(|| $crate::var_os($name)).clone()
        } else {
            None::<std::ffi::OsString>
        }
    }};
}

#[macro_export]
macro_rules! var {
    ($name:literal) => { $crate::var!($name, $crate::ENABLED) };
    ($name:literal, $enabled:expr) => {{
        if $enabled {
            static VALUE: std::sync::OnceLock<Result<String, std::env::VarError>> = std::sync::OnceLock::new();
            VALUE.get_or_init(|| $crate::var($name)).clone()
        } else {
            Err::<String, _>(std::env::VarError::NotPresent)
        }
    }};
}

#[cfg(feature = "parity-debug")]
fn with_value<T>(name: &str, read: impl FnOnce(&Option<OsString>) -> T) -> T {
    use std::collections::HashMap;
    use std::sync::{OnceLock, RwLock};

    static CACHE: OnceLock<RwLock<HashMap<String, Option<OsString>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(RwLock::default);
    let entries = cache.read().unwrap_or_else(|error| error.into_inner());
    if let Some(value) = entries.get(name) {
        return read(value);
    }
    drop(entries);
    let mut entries = cache.write().unwrap_or_else(|error| error.into_inner());
    read(entries.entry(name.to_owned()).or_insert_with(|| std::env::var_os(name)))
}

#[inline]
pub fn var_os(name: &str) -> Option<OsString> {
    #[cfg(feature = "parity-debug")]
    { with_value(name, Clone::clone) }
    #[cfg(not(feature = "parity-debug"))]
    { let _ = name; None }
}

#[inline]
pub fn var(name: &str) -> Result<String, VarError> {
    var_os(name).ok_or(VarError::NotPresent)?
        .into_string().map_err(VarError::NotUnicode)
}

/// Whether the switch is present (including an empty or non-Unicode value).
#[inline]
pub fn is_set(name: &str) -> bool {
    #[cfg(feature = "parity-debug")]
    { with_value(name, Option::is_some) }
    #[cfg(not(feature = "parity-debug"))]
    { let _ = name; false }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unique names keep these tests independent of each other and game switches.
    #[test]
    fn snapshots_first_read_across_accessors() {
        let name = "ZELDA3_DEBUG_ENV_CACHE_TEST_VALUE";
        unsafe { std::env::set_var(name, "first"); }
        let initial = var(name);
        unsafe { std::env::set_var(name, "second"); }
        assert_eq!(initial, if ENABLED { Ok("first".into()) } else { Err(VarError::NotPresent) });
        assert_eq!(var(name), initial);
        assert_eq!(crate::var!("ZELDA3_DEBUG_ENV_CACHE_TEST_VALUE"), initial);
        assert_eq!(crate::var_os!("ZELDA3_DEBUG_ENV_CACHE_TEST_VALUE"),
            ENABLED.then(|| OsString::from("first")));
        assert_eq!(crate::var_os!("ZELDA3_DEBUG_ENV_CACHE_TEST_VALUE", false), None);
        assert_eq!(crate::var!("ZELDA3_DEBUG_ENV_CACHE_TEST_VALUE", false), Err(VarError::NotPresent));
        assert_eq!(is_set(name), ENABLED);
        unsafe { std::env::remove_var(name); }
        assert_eq!(var(name), initial);
    }

    #[test]
    fn snapshots_absence() {
        let name = "ZELDA3_DEBUG_ENV_CACHE_TEST_ABSENT";
        unsafe { std::env::remove_var(name); }
        assert_eq!(var_os(name), None);
        unsafe { std::env::set_var(name, "late"); }
        assert!(!is_set(name));
        assert_eq!(var(name), Err(VarError::NotPresent));
        unsafe { std::env::remove_var(name); }
    }

    #[cfg(unix)]
    #[test]
    fn preserves_non_unicode_and_empty_values() {
        use std::os::unix::ffi::OsStringExt;
        let name = "ZELDA3_DEBUG_ENV_CACHE_TEST_NON_UNICODE";
        let value = OsString::from_vec(vec![0xff]);
        unsafe { std::env::set_var(name, &value); }
        assert_eq!(var_os(name), ENABLED.then(|| value.clone()));
        assert_eq!(var(name), Err(if ENABLED { VarError::NotUnicode(value) } else { VarError::NotPresent }));
        assert_eq!(is_set(name), ENABLED);
        unsafe { std::env::remove_var(name); }

        let empty = "ZELDA3_DEBUG_ENV_CACHE_TEST_EMPTY";
        unsafe { std::env::set_var(empty, ""); }
        assert_eq!(var(empty), if ENABLED { Ok(String::new()) } else { Err(VarError::NotPresent) });
        assert_eq!(is_set(empty), ENABLED);
        unsafe { std::env::remove_var(empty); }
    }
}
