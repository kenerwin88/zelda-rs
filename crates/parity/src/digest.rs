//! Hardware-independent frame digest records used by logical replay.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct VideoDigest {
    pub width: u32,
    pub height: u32,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AudioDigest {
    pub sample_frames: u64,
    pub channels: u32,
    pub sha256: String,
}

