use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::runner::{self, Paths};

pub const SCHEMA: u32 = 1;
pub const CHECKPOINT_FORMAT_VERSION: u32 = 3;
pub const SEED_COMMAND_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointCacheManifest {
    pub schema: u32,
    pub checkpoint_format_version: u32,
    pub seed_command_version: u32,
    pub rust_bin_sha256: String,
    pub rom_sha256: String,
    pub save_sha256: String,
    pub timing_hacks: Vec<String>,
}

impl CheckpointCacheManifest {
    pub fn current(paths: &Paths) -> std::io::Result<Self> {
        Ok(Self {
            schema: SCHEMA,
            checkpoint_format_version: CHECKPOINT_FORMAT_VERSION,
            seed_command_version: SEED_COMMAND_VERSION,
            rust_bin_sha256: runner::sha256_file(&paths.rust_bin)?,
            rom_sha256: runner::sha256_file(&paths.rom)?,
            save_sha256: runner::sha256_file(&paths.save)?,
            timing_hacks: runner::HACK_KEYS
                .iter()
                .map(|key| key.to_string())
                .collect(),
        })
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_vec_pretty(self).unwrap())
    }

    pub fn is_compatible_with(&self, expected: &Self) -> bool {
        self.incompatibility_reason(expected).is_none()
    }

    pub fn incompatibility_reason(&self, expected: &Self) -> Option<&'static str> {
        if self.schema != expected.schema {
            return Some("manifest schema changed");
        }
        if self.checkpoint_format_version != expected.checkpoint_format_version {
            return Some("checkpoint format changed");
        }
        if self.seed_command_version != expected.seed_command_version {
            return Some("seed command changed");
        }
        if self.rust_bin_sha256 != expected.rust_bin_sha256 {
            return Some("rust binary changed");
        }
        if self.rom_sha256 != expected.rom_sha256 {
            return Some("ROM changed");
        }
        if self.save_sha256 != expected.save_sha256 {
            return Some("replay save changed");
        }
        if self.timing_hacks != expected.timing_hacks {
            return Some("timing hacks changed");
        }
        None
    }
}
