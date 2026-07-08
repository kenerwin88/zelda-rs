//! Editable CHR sheet authority: planar codec, pack container, sidecar-v2
//! decode, sha1 parity lock, and pack compilation shared by `build.rs` and
//! the `--bless-chr` CLI.
//!
//! Tracked editable CHR PNG sheets are the source of truth for the packed binary
//! CHR assets `kSprGfx` (asset 064) and `kBgGfx` (asset 065). An unedited sheet
//! tree compiles to byte-identical packs (donor items pass through verbatim); a
//! parity lock (sha1 per sheet block) surfaces deliberate edits, matching the
//! dialogue `messages.toml` / `messages.sha1` pattern.

mod compile;
mod compress;
mod container;
mod lock;
mod planar;
mod sidecar;

pub use compile::compile_chr_packs;
pub use compress::{compress_literal, decompress_asset};
pub use container::{pack_arrays, unpack_packed_arrays};
pub use lock::{
    build_lock, generate_sha_lock, parse_lock, serialize_lock, verify_against_lock, ChrShaEntry,
    ChrShaLock, FORMAT_CHR_SHA_LOCK,
};
pub use planar::{decode_planar_tile_indices, encode_planar_tiles};
pub use sidecar::{
    decode_sheet, parse_manifest, read_sheets_dir, DecodedSheet, SidecarBlock, SidecarLayout,
    SidecarManifest, SidecarPaletteRow, SHEET_NAMES,
};
