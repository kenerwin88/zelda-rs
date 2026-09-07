//! Offline authoring only. The game does not depend on this crate.
//! Unedited maps reproduce the entire input pack, including opaque bytes.
pub mod cli;
#[cfg(feature = "preview")]
pub mod preview;
pub mod project;
mod project_codec;
pub mod room;
pub mod tiled;

use serde_json::Value;
use std::{io::Write, path::Path};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn ensure(condition: bool, message: impl Into<String>) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into().into())
    }
}

pub fn integer(value: &Value, low: i64, high: i64, label: &str) -> Result<i64> {
    value
        .as_i64()
        .filter(|n| (low..=high).contains(n))
        .ok_or_else(|| format!("{label}: expected integer {low}..{high}, got {value}").into())
}

pub fn array<'a>(value: &'a Value, label: &str) -> Result<&'a Vec<Value>> {
    value
        .as_array()
        .ok_or_else(|| format!("{label}: expected array").into())
}

pub fn keys(value: &Value, fields: &[&str], label: &str) -> Result<()> {
    ensure(
        value
            .as_object()
            .is_some_and(|o| o.len() == fields.len() && fields.iter().all(|k| o.contains_key(*k))),
        format!("{label}: expected exactly these fields: {fields:?}"),
    )
}

pub fn unchanged(a: &Value, b: &Value, label: &str) -> Result<()> {
    ensure(
        a == b,
        format!("{label}: unsupported edit; only positions are editable"),
    )
}

pub fn without(value: &Value, fields: &[&str]) -> Result<Value> {
    let mut object = value.as_object().ok_or("expected object")?.clone();
    for field in fields {
        object.remove(*field);
    }
    Ok(Value::Object(object))
}

pub fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

pub fn json_bytes(value: &Value) -> Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn write_new(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(data)?;
    Ok(())
}
