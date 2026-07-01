# Modern off-VRAM HD source-key overrides (Phase 1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let HD art override off-VRAM ("modern"/assets-anim) BG and sprite cells, keyed by logical source key, recolored through the live CGRAM ("detail-modulate") at native 256×224 — parity-safe.

**Architecture:** A new `modern_hd_overrides` module holds a manifest-loaded store (`source_key → HdCell` + a reference palette) and a pure recolor kernel. `ModernIndexTile` carries its `source_key` (set in extract). A new `render_modern_frame_full_with_overrides(...)` threads an `HdOverrideCtx` through the four full-path composite resolve sites; each resolves an override once per instance and calls the kernel per pixel. The existing `render_modern_frame_full` becomes a thin wrapper with a disabled ctx, so all current callers stay byte-identical.

**Tech Stack:** Rust, `png` crate (already a renderer dependency), `serde`/`serde_json` (already used in the crate), `wgpu`-independent (pure CPU path).

## Global Constraints

- **Parity is sacred:** with no override manifest loaded, modern-render output MUST be byte-identical to today. Every existing renderer test (132) stays green; `zparity` / `scripts/validate_all_parity.py` unaffected (parity runs set no override env var).
- **Build/test profile:** `cargo build --profile parity -p renderer` and `cargo test --profile parity -p renderer`.
- **One shared kernel:** BG and sprite paths both call the same `resolve_pixel_color`; the kernel takes an already-computed `cgram_idx` (BG `palette*16+index`, OBJ `0x80+palette*16+index`).
- **Per-instance override resolve, not per-pixel:** resolve `Option<&HdCell>` once per tile instance (before the pixel loop).
- **Reference palette is required for overrides:** if a manifest's reference palette is missing or not 256 px, disable overrides entirely (log) rather than mis-recolor.
- **Commit after each task** with `--no-verify` (the repo's pre-commit gate is heavy and races unrelated in-tree WIP; these changes are renderer-only and covered by `cargo test -p renderer`).
- **Do not** `git checkout` any file (nukes unstaged WIP). Surgically revert your own edits only.

---

### Task 1: Recolor kernel + `HdCell` (`modern_hd_overrides` module)

**Files:**
- Create: `crates/renderer/src/modern_hd_overrides.rs`
- Modify: `crates/renderer/src/lib.rs` (register `pub mod modern_hd_overrides;` in the module list, alphabetically near the other `modern_*` mods around line 20-29)

**Interfaces:**
- Produces:
  - `pub const NO_SOURCE_KEY: u64 = 0;`
  - `pub struct HdCell { pub width: u32, pub height: u32, pub rgba: Vec<u8> }`
  - `impl HdCell { pub fn sample_native(&self, lx: u32, ly: u32) -> [u8; 3] }`
  - `pub fn detail_modulate(live: [u8; 4], hd: [u8; 3], reference: [u8; 3]) -> [u8; 4]`
  - `pub fn resolve_pixel_color(base_index: u8, cgram_idx: usize, live_rgba: [u8; 4], override_cell: Option<&HdCell>, reference: &[[u8; 4]; 256], lx: u32, ly: u32) -> Option<[u8; 4]>`

- [ ] **Step 1: Register the module.** In `crates/renderer/src/lib.rs`, add to the `pub mod modern_*` block (keep it grouped with the others):

```rust
pub mod modern_hd_overrides;
```

- [ ] **Step 2: Write the failing tests.** Create `crates/renderer/src/modern_hd_overrides.rs` with just the tests first:

```rust
//! Source-keyed HD overrides for the modern (off-VRAM) software renderer.
//!
//! HD art overrides a cell by its logical source key and is recolored through the
//! LIVE CGRAM every frame ("detail-modulate"): `final = live * (override / reference)`.
//! Art authored as `reference[cgram_idx]` gives `detail == 1` → exact parity; a
//! different HD color recolors while still tracking the runtime palette. Phase 1
//! samples at native 8×8 (nearest block top-left); Phase 2 will sample at N×.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detail_one_identity_returns_live() {
        // hd == reference → detail 1 → final == live (exact, CPU math).
        let out = detail_modulate([200, 100, 50, 0xff], [128, 128, 128], [128, 128, 128]);
        assert_eq!(out, [200, 100, 50, 0xff]);
    }

    #[test]
    fn detail_modulate_recolors_by_ratio() {
        // live 100 * (hd 64 / ref 128) = 50.
        let out = detail_modulate([100, 100, 100, 0xff], [64, 64, 64], [128, 128, 128]);
        assert_eq!(out, [50, 50, 50, 0xff]);
    }

    #[test]
    fn detail_modulate_clamps_and_guards_zero_reference() {
        // ref 0 → guarded to 1; huge detail clamps to 255. alpha preserved from live.
        let out = detail_modulate([200, 0, 0, 0xff], [255, 0, 0], [0, 0, 0]);
        assert_eq!(out, [255, 0, 0, 0xff]);
    }

    #[test]
    fn sample_native_nearest_block_top_left() {
        // 16×16 HD cell (scale 2). Native (1,0) → HD (2,0).
        let mut rgba = vec![0u8; 16 * 16 * 4];
        let idx = ((0 * 16 + 2) * 4) as usize;
        rgba[idx..idx + 4].copy_from_slice(&[9, 8, 7, 0xff]);
        let cell = HdCell { width: 16, height: 16, rgba };
        assert_eq!(cell.sample_native(1, 0), [9, 8, 7]);
    }

    #[test]
    fn resolve_transparent_when_base_index_zero() {
        let reference = [[0u8; 4]; 256];
        assert_eq!(
            resolve_pixel_color(0, 5, [1, 2, 3, 0xff], None, &reference, 0, 0),
            None
        );
    }

    #[test]
    fn resolve_returns_live_without_override() {
        let reference = [[0u8; 4]; 256];
        assert_eq!(
            resolve_pixel_color(1, 5, [1, 2, 3, 0xff], None, &reference, 0, 0),
            Some([1, 2, 3, 0xff])
        );
    }

    #[test]
    fn resolve_detail_modulates_with_override() {
        let mut reference = [[0u8; 4]; 256];
        reference[5] = [128, 128, 128, 0xff];
        let cell = HdCell { width: 8, height: 8, rgba: vec![64u8; 8 * 8 * 4] };
        // live 100 * (hd 64 / ref 128) = 50.
        assert_eq!(
            resolve_pixel_color(1, 5, [100, 100, 100, 0xff], Some(&cell), &reference, 0, 0),
            Some([50, 50, 50, 0xff])
        );
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail (compile error).**

Run: `cargo test --profile parity -p renderer modern_hd_overrides 2>&1 | tail -20`
Expected: FAIL — `cannot find function/type` (kernel not defined yet).

- [ ] **Step 4: Implement the kernel.** Add above the `#[cfg(test)] mod tests` block:

```rust
/// Sentinel for a cell with no atlas source key (live-VRAM-decoded animation cells,
/// test cells): never has an override.
pub const NO_SOURCE_KEY: u64 = 0;

/// A decoded HD override image for one 8×8 logical cell. `width`/`height` are multiples
/// of 8 (an N× upscale; 8×8 == 1×). `rgba` is row-major RGBA8, `width*height*4` bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HdCell {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl HdCell {
    /// Sample the color for native 8×8 pixel `(lx, ly)` (each 0..8) as the top-left of
    /// its N× block — nearest downsample, matching the atlas convention. Alpha is
    /// dropped: transparency is decided by the base cell's slot index, not HD alpha.
    pub fn sample_native(&self, lx: u32, ly: u32) -> [u8; 3] {
        let scale_x = (self.width / 8).max(1);
        let scale_y = (self.height / 8).max(1);
        let px = (lx * scale_x).min(self.width.saturating_sub(1));
        let py = (ly * scale_y).min(self.height.saturating_sub(1));
        let idx = ((py * self.width + px) * 4) as usize;
        [self.rgba[idx], self.rgba[idx + 1], self.rgba[idx + 2]]
    }
}

/// `final = clamp(live * (hd / max(reference, 1)))` per RGB channel; alpha from `live`.
/// Reference is guarded away from 0 to avoid divide-by-zero on dark slots. Authoring HD
/// as `reference[idx]` yields detail 1 → `final == live` (exact parity).
pub fn detail_modulate(live: [u8; 4], hd: [u8; 3], reference: [u8; 3]) -> [u8; 4] {
    let mut out = [0u8, 0, 0, live[3]];
    for c in 0..3 {
        let detail = hd[c] as f32 / (reference[c] as f32).max(1.0);
        out[c] = (live[c] as f32 * detail).round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// Resolve the final RGBA for one cell pixel. `None` = transparent (base slot index 0;
/// HD art never changes tile shape). With an override, detail-modulate; without,
/// return the live color unchanged (byte-identical to the direct CGRAM lookup).
#[allow(clippy::too_many_arguments)]
pub fn resolve_pixel_color(
    base_index: u8,
    cgram_idx: usize,
    live_rgba: [u8; 4],
    override_cell: Option<&HdCell>,
    reference: &[[u8; 4]; 256],
    lx: u32,
    ly: u32,
) -> Option<[u8; 4]> {
    if base_index == 0 {
        return None;
    }
    match override_cell {
        Some(hd) => {
            let hd_rgb = hd.sample_native(lx, ly);
            let r = reference[cgram_idx];
            Some(detail_modulate(live_rgba, hd_rgb, [r[0], r[1], r[2]]))
        }
        None => Some(live_rgba),
    }
}
```

- [ ] **Step 5: Run the tests to verify they pass.**

Run: `cargo test --profile parity -p renderer modern_hd_overrides 2>&1 | tail -20`
Expected: PASS — 7 tests.

- [ ] **Step 6: Commit.**

```bash
git add crates/renderer/src/modern_hd_overrides.rs crates/renderer/src/lib.rs
git commit --no-verify -m "feat(renderer): HD override recolor kernel (modern_hd_overrides)"
```

---

### Task 2: Manifest, store, and `HdOverrideCtx`

**Files:**
- Modify: `crates/renderer/src/modern_hd_overrides.rs`

**Interfaces:**
- Consumes: `HdCell`, `NO_SOURCE_KEY` (Task 1).
- Produces:
  - `pub struct ModernHdOverrides` with:
    - `pub fn from_parts(by_key: std::collections::HashMap<u64, HdCell>, reference: [[u8; 4]; 256]) -> Self` (test/programmatic constructor)
    - `pub fn from_env() -> Option<Self>` (reads `ZELDA3_MODERN_HD_OVERRIDES`)
    - `pub fn load_manifest(path: &std::path::Path) -> Option<Self>`
    - `pub fn get(&self, key: u64) -> Option<&HdCell>`
    - `pub fn reference(&self) -> &[[u8; 4]; 256]`
    - `pub fn is_enabled(&self) -> bool`
  - `pub struct HdOverrideCtx<'a>` with:
    - `pub fn disabled() -> Self`
    - `pub fn new(store: &'a ModernHdOverrides) -> Self`
    - `pub fn resolve(&self, source_key: u64) -> Option<&'a HdCell>`
    - `pub fn reference(&self) -> &[[u8; 4]; 256]`

- [ ] **Step 1: Write the failing tests.** Add inside the existing `mod tests` in `modern_hd_overrides.rs`:

```rust
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn write_png(path: &std::path::Path, width: u32, height: u32, rgba: &[u8]) {
        let file = std::fs::File::create(path).unwrap();
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), width, height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(rgba).unwrap();
    }

    fn unique_dir(tag: &str) -> PathBuf {
        // No Date/random needed: process id + tag is unique enough per test run.
        let dir = std::env::temp_dir().join(format!("zelda3_hd_ovr_{}_{}", std::process::id(), tag));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn ctx_disabled_resolves_nothing() {
        let ctx = HdOverrideCtx::disabled();
        assert!(ctx.resolve(0x0600_0000_1234_0000).is_none());
        assert_eq!(ctx.reference(), &[[0u8, 0, 0, 0xff]; 256]);
    }

    #[test]
    fn store_get_ignores_no_source_key() {
        let mut by_key = HashMap::new();
        by_key.insert(NO_SOURCE_KEY, HdCell { width: 8, height: 8, rgba: vec![0; 256] });
        let store = ModernHdOverrides::from_parts(by_key, [[0u8; 4]; 256]);
        assert!(store.get(NO_SOURCE_KEY).is_none());
    }

    #[test]
    fn load_manifest_decodes_overrides_and_reference() {
        let dir = unique_dir("load_ok");
        write_png(&dir.join("ref.png"), 256, 1, &vec![0x80u8; 256 * 4]);
        write_png(&dir.join("grass.png"), 8, 8, &vec![0x40u8; 8 * 8 * 4]);
        let manifest = dir.join("m.json");
        std::fs::write(
            &manifest,
            r#"{"reference_palette":"ref.png","overrides":[{"key":"0x0000000100000000","rgba":"grass.png"}]}"#,
        )
        .unwrap();

        let store = ModernHdOverrides::load_manifest(&manifest).unwrap();
        assert!(store.is_enabled());
        assert_eq!(store.reference()[5], [0x80, 0x80, 0x80, 0xff]);
        let cell = store.get(0x0000_0001_0000_0000).unwrap();
        assert_eq!((cell.width, cell.height), (8, 8));
    }

    #[test]
    fn load_manifest_disables_when_reference_missing() {
        let dir = unique_dir("no_ref");
        write_png(&dir.join("grass.png"), 8, 8, &vec![0x40u8; 8 * 8 * 4]);
        let manifest = dir.join("m.json");
        std::fs::write(
            &manifest,
            r#"{"reference_palette":"missing.png","overrides":[{"key":"0x1","rgba":"grass.png"}]}"#,
        )
        .unwrap();

        // Reference missing/unreadable → overrides disabled entirely (returns None).
        assert!(ModernHdOverrides::load_manifest(&manifest).is_none());
    }
```

- [ ] **Step 2: Run the tests to verify they fail.**

Run: `cargo test --profile parity -p renderer modern_hd_overrides 2>&1 | tail -20`
Expected: FAIL — `ModernHdOverrides`/`HdOverrideCtx` not found.

- [ ] **Step 3: Implement the store, manifest, and ctx.** Add to `modern_hd_overrides.rs` (above the tests). Put the imports at the top of the file:

```rust
use std::collections::HashMap;
use std::io::BufReader;
use std::path::Path;

use serde::Deserialize;
```

Then the types:

```rust
#[derive(Debug, Deserialize)]
struct ManifestJson {
    reference_palette: String,
    #[serde(default)]
    overrides: Vec<OverrideJson>,
}

#[derive(Debug, Deserialize)]
struct OverrideJson {
    /// Logical source key, hex string (`0x…`) as emitted by `--dump-assets-by-source`.
    key: String,
    /// RGBA PNG path relative to the manifest; dims are multiples of 8.
    rgba: String,
}

/// Source-keyed HD override store: `source_key → HdCell` plus the reference palette the
/// HD art was authored against. Loaded once; empty/absent → the modern renderer is
/// byte-identical to today.
#[derive(Debug, Clone)]
pub struct ModernHdOverrides {
    by_key: HashMap<u64, HdCell>,
    reference: [[u8; 4]; 256],
}

impl ModernHdOverrides {
    pub fn from_parts(by_key: HashMap<u64, HdCell>, reference: [[u8; 4]; 256]) -> Self {
        Self { by_key, reference }
    }

    /// Load from `ZELDA3_MODERN_HD_OVERRIDES=<manifest path>`. Unset → `None` (disabled).
    pub fn from_env() -> Option<Self> {
        let path = std::env::var_os("ZELDA3_MODERN_HD_OVERRIDES")?;
        Self::load_manifest(Path::new(&path))
    }

    /// Parse a manifest and decode its art. Returns `None` (overrides disabled) if the
    /// manifest is unreadable/invalid or the reference palette is missing / not 256 px —
    /// never mis-recolor against a bad reference. Individual bad `rgba` entries are
    /// skipped (logged); other overrides still load.
    pub fn load_manifest(path: &Path) -> Option<Self> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| eprintln!("ZELDA3_MODERN_HD_OVERRIDES read {}: {e}", path.display()))
            .ok()?;
        let manifest: ManifestJson = serde_json::from_str(&json)
            .map_err(|e| eprintln!("ZELDA3_MODERN_HD_OVERRIDES parse {}: {e}", path.display()))
            .ok()?;
        let base = path.parent().unwrap_or_else(|| Path::new("."));

        let reference = decode_reference(&base.join(&manifest.reference_palette))?;

        let mut by_key = HashMap::new();
        for ovr in &manifest.overrides {
            let Some(key) = parse_key(&ovr.key) else {
                eprintln!("ZELDA3_MODERN_HD_OVERRIDES bad key {:?}; skipping", ovr.key);
                continue;
            };
            match decode_rgba_cell(&base.join(&ovr.rgba)) {
                Some(cell) => {
                    by_key.insert(key, cell);
                }
                None => eprintln!(
                    "ZELDA3_MODERN_HD_OVERRIDES bad rgba {}; skipping",
                    base.join(&ovr.rgba).display()
                ),
            }
        }
        Some(Self { by_key, reference })
    }

    pub fn get(&self, key: u64) -> Option<&HdCell> {
        if key == NO_SOURCE_KEY {
            return None;
        }
        self.by_key.get(&key)
    }

    pub fn reference(&self) -> &[[u8; 4]; 256] {
        &self.reference
    }

    pub fn is_enabled(&self) -> bool {
        !self.by_key.is_empty()
    }
}

fn parse_key(s: &str) -> Option<u64> {
    let t = s.trim();
    match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        Some(hex) => u64::from_str_radix(hex, 16).ok(),
        None => t.parse::<u64>().ok(),
    }
}

/// Decode a `width×height` RGBA8 PNG (dims multiples of 8) into an `HdCell`.
fn decode_rgba_cell(path: &Path) -> Option<HdCell> {
    let (width, height, rgba) = decode_rgba_png(path)?;
    if width == 0 || height == 0 || width % 8 != 0 || height % 8 != 0 {
        eprintln!(
            "ZELDA3_MODERN_HD_OVERRIDES rgba {} dims {width}×{height} not multiples of 8",
            path.display()
        );
        return None;
    }
    Some(HdCell { width, height, rgba })
}

/// Decode a 256×1 RGBA PNG into a `[[u8;4];256]` reference palette. `None` if not 256 px.
fn decode_reference(path: &Path) -> Option<[[u8; 4]; 256]> {
    let (width, height, rgba) = decode_rgba_png(path)
        .or_else(|| {
            eprintln!("ZELDA3_MODERN_HD_OVERRIDES reference {} unreadable", path.display());
            None
        })?;
    if (width * height) as usize != 256 || rgba.len() != 256 * 4 {
        eprintln!(
            "ZELDA3_MODERN_HD_OVERRIDES reference {} must be 256 RGBA px (got {width}×{height})",
            path.display()
        );
        return None;
    }
    let mut out = [[0u8; 4]; 256];
    for (i, px) in out.iter_mut().enumerate() {
        px.copy_from_slice(&rgba[i * 4..i * 4 + 4]);
    }
    Some(out)
}

/// Decode any RGBA8 PNG to `(width, height, rgba)`.
fn decode_rgba_png(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let file = std::fs::File::open(path).ok()?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    buf.truncate(info.buffer_size());
    Some((info.width, info.height, buf))
}

/// Render-time override context threaded through the compositor. `disabled()` (no store)
/// makes every resolve a no-op → byte-identical to today.
#[derive(Clone, Copy)]
pub struct HdOverrideCtx<'a> {
    store: Option<&'a ModernHdOverrides>,
}

static ZERO_REFERENCE: [[u8; 4]; 256] = [[0, 0, 0, 0xff]; 256];

impl<'a> HdOverrideCtx<'a> {
    pub fn disabled() -> Self {
        Self { store: None }
    }

    pub fn new(store: &'a ModernHdOverrides) -> Self {
        Self { store: Some(store) }
    }

    pub fn resolve(&self, source_key: u64) -> Option<&'a HdCell> {
        self.store.and_then(|s| s.get(source_key))
    }

    pub fn reference(&self) -> &[[u8; 4]; 256] {
        match self.store {
            Some(s) => s.reference(),
            None => &ZERO_REFERENCE,
        }
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass.**

Run: `cargo test --profile parity -p renderer modern_hd_overrides 2>&1 | tail -20`
Expected: PASS — 11 tests total (7 from Task 1 + 4 here).

- [ ] **Step 5: Commit.**

```bash
git add crates/renderer/src/modern_hd_overrides.rs
git commit --no-verify -m "feat(renderer): HD override manifest/store + HdOverrideCtx"
```

---

### Task 3: Carry `source_key` on `ModernIndexTile`

**Files:**
- Modify: `crates/renderer/src/modern_index_atlas.rs` (struct + constructors)
- Modify: `crates/renderer/src/modern_extract.rs` (set the real key at atlas-backed cell pushes)
- Modify (compile-fix only): every remaining `ModernIndexTile { .. }` literal across the crate

**Interfaces:**
- Consumes: `NO_SOURCE_KEY` (Task 1).
- Produces: `ModernIndexTile` gains `pub source_key: u64`.

- [ ] **Step 1: Add the field.** In `crates/renderer/src/modern_index_atlas.rs`, extend the struct (keep the existing doc comment):

```rust
pub struct ModernIndexTile {
    pub id: u32,
    pub indices: [u8; 64],
    /// Logical source key (`modern_source_key(kind, pack, tile_off)`) for HD override
    /// lookup, or `crate::modern_hd_overrides::NO_SOURCE_KEY` (0) for cells with no
    /// atlas source (live-VRAM-decoded animation cells, test cells).
    pub source_key: u64,
}
```

- [ ] **Step 2: Set the real key at the atlas-backed extract sites.** In `crates/renderer/src/modern_extract.rs`, the two `_from_sources` functions push atlas-resolved cells. At each push that comes from `source_cell(atlas, kind, pack, tile_off)` (BG main path ~line 1189-1193; sprite path — see Step 4), set the real key. The BG atlas push becomes:

```rust
let id = *cell_ids.entry((src.id, hflip, vflip)).or_insert_with(|| {
    let indices = flip_index_pattern(&src.indices, hflip, vflip);
    let id = cells.len() as u32;
    cells.push(ModernIndexTile {
        id,
        indices,
        source_key: crate::modern_source_atlas::modern_source_key(kind, pack, tile_off),
    });
    id
});
```

(`modern_source_key` is the same function `ModernSourceAtlas::from_keyed_cells_for_test` uses; confirm the exact path with `rg -n "pub fn modern_source_key" crates/renderer/src/modern_source_atlas.rs` and use that path.)

- [ ] **Step 3: Default every other literal to `NO_SOURCE_KEY`.** Build to get the compiler's exhaustive list of missing-field errors:

Run: `cargo build --profile parity -p renderer 2>&1 | rg "missing field .source_key|ModernIndexTile" | head -40`

For each reported `ModernIndexTile { .. }` literal that is NOT an atlas-backed `_from_sources` push (i.e. the non-injective live-decode push at ~line 1137, the BG3-baked push at ~line 1088, all `modern_dungeon_atlas.rs` / `modern_sprite_atlas.rs` / `modern_source_atlas.rs` / `modern_gpu.rs` / `modern_software.rs` test literals, and any `let cell = ModernIndexTile { .. }` in tests), add the field:

```rust
// before: ModernIndexTile { id, indices }
// after:
ModernIndexTile { id, indices, source_key: crate::modern_hd_overrides::NO_SOURCE_KEY }
```

For sprite `_from_sources` cells that DO resolve via the atlas (`source_cell`), set the real key exactly as in Step 2 (`modern_source_key(kind, pack, tile_off)`); for the special-cased Link/live sprite cells that do NOT come from `source_cell`, use `NO_SOURCE_KEY`.

- [ ] **Step 4: Verify build is clean and existing tests unchanged.**

Run: `cargo build --profile parity -p renderer 2>&1 | tail -5`
Expected: builds (only the pre-existing `cell_count` dead-code warning).

Run: `cargo test --profile parity -p renderer 2>&1 | grep "test result"`
Expected: all pass, same count as before Task 3 + the 11 hd_overrides tests. (The new field is not yet read by any renderer, so output is byte-identical.)

- [ ] **Step 5: Commit.**

```bash
git add crates/renderer/src/modern_index_atlas.rs crates/renderer/src/modern_extract.rs crates/renderer/src/modern_dungeon_atlas.rs crates/renderer/src/modern_sprite_atlas.rs crates/renderer/src/modern_source_atlas.rs crates/renderer/src/modern_gpu.rs crates/renderer/src/modern_software.rs
git commit --no-verify -m "feat(renderer): carry source_key on ModernIndexTile (set in extract)"
```

---

### Task 4: Thread `HdOverrideCtx` through the compositor

**Files:**
- Modify: `crates/renderer/src/modern_software.rs`

**Interfaces:**
- Consumes: `HdOverrideCtx`, `resolve_pixel_color` (Tasks 1-2); `ModernIndexTile.source_key` (Task 3).
- Produces:
  - `pub fn render_modern_frame_full_with_overrides(frame: &ModernFrame, bg_cells: &[ModernIndexTile], sprite_cells: &[ModernIndexTile], ctx: &crate::modern_hd_overrides::HdOverrideCtx) -> Vec<u8>`
  - `render_modern_frame_full(...)` retained as a wrapper calling the above with `HdOverrideCtx::disabled()`.

- [ ] **Step 1: Add a byte-identical guard test.** In the `mod tests` of `modern_software.rs`, add a test asserting the wrapper and the disabled-ctx entry agree on an existing fixture. Reuse the BG fixture pattern already in the file (a `ModernFrame` + `cells`); the minimal version:

```rust
    #[test]
    fn disabled_overrides_match_plain_full_render() {
        use crate::modern_hd_overrides::HdOverrideCtx;
        // Build the same tiny frame the other full-path tests use (index-tile BG).
        let (frame, cells) = tiny_bg_frame_for_test(); // see note below
        let plain = render_modern_frame_full(&frame, &cells, &[]);
        let disabled =
            render_modern_frame_full_with_overrides(&frame, &cells, &[], &HdOverrideCtx::disabled());
        assert_eq!(plain, disabled);
    }
```

Note: if no shared `tiny_bg_frame_for_test` helper exists, inline the smallest existing full-path BG fixture in this file (search the test module for an existing `render_modern_frame_full(&frame, &cells, &[])` call and copy its setup). Do not invent new frame semantics — copy an existing passing fixture.

- [ ] **Step 2: Run it to verify it fails to compile.**

Run: `cargo test --profile parity -p renderer disabled_overrides_match_plain_full_render 2>&1 | tail -10`
Expected: FAIL — `render_modern_frame_full_with_overrides` not found.

- [ ] **Step 3: Add the new entry + wrapper.** Replace the existing `render_modern_frame_full` body (line ~965-1014) so the wrapper delegates:

```rust
pub fn render_modern_frame_full(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
) -> Vec<u8> {
    render_modern_frame_full_with_overrides(
        frame,
        bg_cells,
        sprite_cells,
        &crate::modern_hd_overrides::HdOverrideCtx::disabled(),
    )
}

/// As `render_modern_frame_full`, but applies source-keyed HD overrides via `ctx`
/// (detail-modulated recolor). `HdOverrideCtx::disabled()` → byte-identical to the
/// plain entry.
pub fn render_modern_frame_full_with_overrides(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
) -> Vec<u8> {
    // ... the original body, but pass `ctx` into both composite_mode1 calls ...
}
```

Move the original body (forced-blank, backdrop, both `composite_mode1(...)` calls, `finalize_frame`) into `_with_overrides`, adding `ctx` as the final argument to each `composite_mode1(...)` call.

- [ ] **Step 4: Thread `ctx` through the composite chain.** Add `ctx: &crate::modern_hd_overrides::HdOverrideCtx` as the final parameter to each of these fns, and forward it at every call site within `modern_software.rs`:
  - `composite_mode1` → forwards to `composite_mode1_mosaic`, `composite_mode1_scanline_scroll`, `composite_index_tiles_c5`, and `resolve_obj_layer`.
  - `composite_mode1_mosaic` → forwards to `render_bg_layer_buf` and `resolve_obj_layer`.
  - `composite_mode1_scanline_scroll` → forwards to `render_bg_layer_torus` and `resolve_obj_layer`.
  - `render_bg_layer_buf`, `render_bg_layer_torus`, `composite_index_tiles_c5`, `resolve_obj_layer` → consume `ctx` (Step 5).

- [ ] **Step 5: Replace the four resolve sites with the kernel.** In each of `render_bg_layer_buf` (BG), `render_bg_layer_torus` (BG), `composite_index_tiles_c5` (BG), and `resolve_obj_layer` (OBJ):

  (a) Once per instance, before the pixel loop, resolve the override from the cell:

```rust
let ov = ctx.resolve(cell.source_key);
```

  (b) At the pixel resolve, replace the direct CGRAM lookup. For the **BG** sites, the current line is:

```rust
let color = frame.cgram_rgba[inst.palette as usize * 16 + index as usize];
```

  becomes (using the same local `(col, row)` this loop uses to index `cell.indices` as `(lx, ly)`):

```rust
let cgram_idx = inst.palette as usize * 16 + index as usize;
let color = match crate::modern_hd_overrides::resolve_pixel_color(
    index, cgram_idx, frame.cgram_rgba[cgram_idx], ov, ctx.reference(), col as u32, row as u32,
) {
    Some(c) => c,
    None => continue, // matches the pre-existing `if index == 0 { continue }` behavior
};
```

  For the **OBJ** site (`resolve_obj_layer`, line ~739) the current line is:

```rust
let color = frame.cgram_rgba[0x80 + inst.palette as usize * 16 + index as usize];
```

  becomes:

```rust
let cgram_idx = 0x80 + inst.palette as usize * 16 + index as usize;
let color = match crate::modern_hd_overrides::resolve_pixel_color(
    index, cgram_idx, frame.cgram_rgba[cgram_idx], ov, ctx.reference(), col as u32, row as u32,
) {
    Some(c) => c,
    None => continue,
};
```

  IMPORTANT: use whatever local variables the existing loop already uses for the in-cell column/row (the ones indexing `cell.indices[row * 8 + col]`) as `(lx, ly)` — do not introduce new coordinate math. If the existing code already skips on `index == 0` earlier, keep the kernel's `None => continue` as the single transparency gate and remove the now-redundant earlier `if index == 0 { continue }` in that block (they are equivalent; the kernel returns `None` for index 0).

- [ ] **Step 6: Build and run the full renderer suite.**

Run: `cargo test --profile parity -p renderer 2>&1 | grep "test result"`
Expected: all pass. The `disabled_overrides_match_plain_full_render` test passes, and every pre-existing full-path test is byte-identical (disabled ctx → kernel returns live).

- [ ] **Step 7: Commit.**

```bash
git add crates/renderer/src/modern_software.rs
git commit --no-verify -m "feat(renderer): thread HdOverrideCtx through modern compositor (disabled=parity)"
```

---

### Task 5: Integration test — BG + sprite override recolor and parity

**Files:**
- Modify: `crates/renderer/src/modern_software.rs` (test module)

**Interfaces:**
- Consumes: `ModernHdOverrides::from_parts`, `HdOverrideCtx::new`, `render_modern_frame_full_with_overrides`.

- [ ] **Step 1: Write the failing integration test.** Add to the `mod tests` of `modern_software.rs`. It builds one BG index-tile instance and one sprite instance whose cells carry a real `source_key`, then renders with (a) a reference == live palette store (expect byte-identical to disabled) and (b) a recoloring store.

```rust
    #[test]
    fn source_keyed_overrides_recolor_bg_and_sprite() {
        use crate::modern_hd_overrides::{HdCell, HdOverrideCtx, ModernHdOverrides};
        use std::collections::HashMap;

        // Reuse an existing minimal full-path fixture that draws ONE BG index tile and
        // ONE index sprite at known screen positions with known palettes/indices.
        // (Copy the smallest such setup already present in this test module.)
        let (frame, bg_cells, sprite_cells, bg_probe, spr_probe) = tiny_bg_and_sprite_fixture();
        // bg_probe / spr_probe = (pixel_byte_offset, cgram_idx, base_index) for a known
        // opaque pixel of each, taken from the fixture.

        let bg_key = bg_cells[0].source_key;
        let spr_key = sprite_cells[0].source_key;
        assert_ne!(bg_key, crate::modern_hd_overrides::NO_SOURCE_KEY);
        assert_ne!(spr_key, crate::modern_hd_overrides::NO_SOURCE_KEY);

        let plain = render_modern_frame_full(&frame, &bg_cells, &sprite_cells);

        // (a) reference == the frame's live palette, HD art == reference → detail 1 →
        // byte-identical to the plain (no-override) render.
        let mut ref_pal = [[0u8; 4]; 256];
        for (i, e) in ref_pal.iter_mut().enumerate() {
            *e = frame.cgram_rgba[i];
        }
        let hd_identity = |cgram_idx: usize| {
            let c = ref_pal[cgram_idx];
            HdCell { width: 8, height: 8, rgba: vec![c[0], c[1], c[2], 0xff].repeat(64) }
        };
        let mut by_key = HashMap::new();
        by_key.insert(bg_key, hd_identity(bg_probe.1));
        by_key.insert(spr_key, hd_identity(spr_probe.1));
        let store_identity = ModernHdOverrides::from_parts(by_key, ref_pal);
        let identity = render_modern_frame_full_with_overrides(
            &frame, &bg_cells, &sprite_cells, &HdOverrideCtx::new(&store_identity),
        );
        assert_eq!(identity, plain, "detail=1 override must match no-override render");

        // (b) HD art = half the reference → live halved at the probed pixels.
        let hd_half = |cgram_idx: usize| {
            let c = ref_pal[cgram_idx];
            HdCell { width: 8, height: 8, rgba: vec![c[0] / 2, c[1] / 2, c[2] / 2, 0xff].repeat(64) }
        };
        let mut by_key2 = HashMap::new();
        by_key2.insert(bg_key, hd_half(bg_probe.1));
        by_key2.insert(spr_key, hd_half(spr_probe.1));
        let store_half = ModernHdOverrides::from_parts(by_key2, ref_pal);
        let recolored = render_modern_frame_full_with_overrides(
            &frame, &bg_cells, &sprite_cells, &HdOverrideCtx::new(&store_half),
        );

        for probe in [bg_probe, spr_probe] {
            let (off, cgram_idx, _base) = probe;
            let live = ref_pal[cgram_idx];
            let expected = [live[0] / 2, live[1] / 2, live[2] / 2];
            assert_eq!(&recolored[off..off + 3], &expected, "recolor at {off}");
            assert_ne!(&recolored[off..off + 3], &plain[off..off + 3], "must differ from plain");
        }
    }
```

Note: `tiny_bg_and_sprite_fixture()` — build from the existing full-path fixtures already in this module. There are existing tests that place a single BG index tile (see the `cgram_rgba[3 * 16 + 1]` / index-tile tests near the end of the file) and a single index sprite (see the `cgram_rgba[0x80 + 3 * 16 + ...]` sprite tests). Compose one frame with both, set `bg_cells[0].source_key` / `sprite_cells[0].source_key` to distinct real keys (any nonzero u64, e.g. `0x0000_0001_0000_0000` and `0x0000_0002_0000_0000`), and record the on-screen byte offset of one opaque pixel of each (screen_x/screen_y → `(y*256 + x)*4`) plus its `cgram_idx` and base index. Keep halving exact by choosing live palette channels that are even (e.g. set `cgram_rgba[idx] = [200, 100, 60, 0xff]`).

- [ ] **Step 2: Run it to verify it fails.**

Run: `cargo test --profile parity -p renderer source_keyed_overrides_recolor_bg_and_sprite 2>&1 | tail -20`
Expected: FAIL first on a compile/fixture issue; iterate on the fixture until it compiles, then it should exercise identity (a) and recolor (b).

- [ ] **Step 3: Make it pass.** Fix the fixture offsets/indices until both assertions hold. No production code changes should be needed (the kernel + threading already exist); if identity (a) fails, the bug is in coordinate mapping (`lx,ly`) or `cgram_idx` — verify the probed pixel's `(col,row)` matches what the compositor uses.

- [ ] **Step 4: Run the full suite.**

Run: `cargo test --profile parity -p renderer 2>&1 | grep "test result"`
Expected: all pass.

- [ ] **Step 5: Commit.**

```bash
git add crates/renderer/src/modern_software.rs
git commit --no-verify -m "test(renderer): source-keyed HD override recolor + detail=1 parity (BG+sprite)"
```

---

### Task 6: End-to-end manifest load + docs/memory

**Files:**
- Modify: `crates/renderer/src/modern_hd_overrides.rs` (one end-to-end test)
- Modify: `docs/superpowers/plans/2026-07-01-modern-hd-source-key-overrides.md` (mark Phase 1 done; note Phase 2 hookup)

**Interfaces:**
- Consumes: everything from Tasks 1-4.

- [ ] **Step 1: End-to-end test — manifest file → store → render.** In `modern_hd_overrides.rs` tests, write a manifest + PNGs to a temp dir, load via `load_manifest`, and assert `get(key)` returns a cell whose `sample_native` matches the PNG. (Rendering is already covered by Task 5's `from_parts` path; this proves the file → store leg end-to-end.)

```rust
    #[test]
    fn end_to_end_manifest_load_resolves_override_cell() {
        let dir = unique_dir("e2e");
        write_png(&dir.join("ref.png"), 256, 1, &vec![0x80u8; 256 * 4]);
        // 16×16 (2×) HD cell, solid color 0x40.
        write_png(&dir.join("hd.png"), 16, 16, &vec![0x40u8; 16 * 16 * 4]);
        let manifest = dir.join("m.json");
        std::fs::write(
            &manifest,
            r#"{"reference_palette":"ref.png","overrides":[{"key":"0x00000002abcd0000","rgba":"hd.png"}]}"#,
        )
        .unwrap();

        let store = ModernHdOverrides::load_manifest(&manifest).unwrap();
        let ctx = HdOverrideCtx::new(&store);
        let cell = ctx.resolve(0x0000_0002_abcd_0000).unwrap();
        assert_eq!((cell.width, cell.height), (16, 16));
        assert_eq!(cell.sample_native(3, 3), [0x40, 0x40, 0x40]);
        assert!(ctx.resolve(NO_SOURCE_KEY).is_none());
    }
```

- [ ] **Step 2: Run it.**

Run: `cargo test --profile parity -p renderer modern_hd_overrides 2>&1 | tail -20`
Expected: PASS (12 tests total in the module).

- [ ] **Step 3: Full suite + confirm parity untouched.**

Run: `cargo test --profile parity -p renderer 2>&1 | grep "test result"`
Expected: all pass.

Run: `cargo build --profile parity -p zelda3-bin 2>&1 | tail -3`
Expected: builds (the bin's existing `render_modern_frame_full` call still compiles — the wrapper signature is unchanged).

- [ ] **Step 4: Record the deferred hookup.** Append a short "Phase 1 status / Phase 2 hookup" note to this plan file: the live on-screen display still calls `render_modern_frame_full` (disabled ctx); wiring `ZELDA3_MODERN_HD_OVERRIDES` into the live `sources` render path (main.rs ~5382, building `HdOverrideCtx::new(&ModernHdOverrides::from_env())`) is a Phase 2 concern, done together with the N× buffer so there is something visible to show. Update the project memory `assets-by-source-png-migration.md` accordingly.

- [ ] **Step 5: Commit.**

```bash
git add crates/renderer/src/modern_hd_overrides.rs docs/superpowers/plans/2026-07-01-modern-hd-source-key-overrides.md
git commit --no-verify -m "test(renderer): end-to-end HD override manifest load; Phase 1 complete"
```

---

## Self-review notes

- **Spec coverage:** Manifest+store (Task 2) ↔ spec §Components 1-2; `source_key` on cell (Task 3) ↔ spec §Component 3 (corrected); kernel (Task 1) ↔ §Component 4; threading + wrapper (Task 4) ↔ §Data flow; error handling (Task 2: disable-on-bad-reference, skip-bad-rgba) ↔ §Error handling; tests (Tasks 1,2,5,6) ↔ §Testing (kernel identity/recolor/transparency, BG+sprite integration, parity via disabled ctx, e2e load). Parity gate untouched ↔ §Parity gate.
- **Not covered (by design):** simple non-parity path (`render_modern_frame_software_indexed` / `draw_modern_sprites_indexed`); live on-screen display wiring; N× output — all explicitly Phase 2.
- **Type consistency:** `resolve_pixel_color` signature identical in Task 1 (def) and Task 4 (call). `HdOverrideCtx::{disabled,new,resolve,reference}` consistent Tasks 2/4/5. `ModernIndexTile.source_key: u64` consistent Tasks 3/4/5. `ModernHdOverrides::{from_parts,load_manifest,get,reference}` consistent Tasks 2/5/6.

---

## Phase 1 status / Phase 2 hookup

**Phase 1 COMPLETE** (all 6 tasks):
- Kernel + detail-modulate recolor (`modern_hd_overrides` module, Task 1)
- Manifest parser + store (`ModernHdOverrides`, `HdOverrideCtx`, Task 2)
- Source-key carry on `ModernIndexTile` (Task 3)
- Compositor threading with `HdOverrideCtx` (Task 4)
- Integration + parity tests (Tasks 5–6)

**Current state:** The live on-screen display still calls `render_modern_frame_full` (internally with `HdOverrideCtx::disabled()`), so no overrides are applied on screen yet. The full end-to-end infrastructure (manifest → store → kernel recolor → render) is proven by tests; it is ready for live wiring.

**Phase 2 (future):** Wiring `ZELDA3_MODERN_HD_OVERRIDES` into the live `sources` render path (zelda3-bin/src/main.rs ~5382, building `HdOverrideCtx::new(&ModernHdOverrides::from_env())`) will be done together with the N× buffer upscaling so there is something visible to show on screen.
