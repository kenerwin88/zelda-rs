pub mod fingerprint;
pub use fingerprint::{
    fnv1a, fnv1a_u32s, FrameFingerprint, FINGERPRINT_MASK, PAGE, RECORD_LEN, VRAM_PAGES, WRAM_PAGES,
};
