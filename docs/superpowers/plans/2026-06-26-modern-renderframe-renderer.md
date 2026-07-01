# Modern RenderFrame Renderer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a modern renderer path that can move away from SNES memory-oriented rendering while preserving final-pixel graphical parity.

**Architecture:** Introduce a renderer-neutral `ModernFrame` scene model, translate the existing `GpuFrame` into that model, and render it with modern atlas-backed quads. Keep the current SNES-style GPU renderer as the oracle until the modern path matches pixel output across replay routes.

**Tech Stack:** Rust, `wgpu`, existing `crates/renderer` shaders, existing `GpuFrame`, existing replay/GPU compare commands, generated atlas assets in `zelda3-bin/developer_tilesets/`.

## Global Constraints

- Preserve the fixed `256x224` game render target as the parity boundary.
- Keep the existing SNES-style renderer as the default until modern-render parity is proven.
- Do not remove VRAM/CGRAM/OAM based rendering during this project.
- New renderer modes must be opt-in via CLI/env flags.
- Parity is final pixels, not memory layout.
- Every task must add a focused test before production changes.
- Generated PNG/JSON atlas assets live under `zelda3-bin/developer_tilesets/`.

---

## File Structure

- Create `crates/renderer/src/modern_frame.rs`
  - Owns renderer-neutral scene data: tiles, sprites, layers, palettes, effects, and frame dimensions.
- Create `crates/renderer/src/modern_extract.rs`
  - Converts the existing `GpuFrame<'_>` into `ModernFrame`.
- Create `crates/renderer/src/modern_software.rs`
  - CPU reference renderer for a small subset of `ModernFrame`, used for exact tests before GPU work.
- Create `crates/renderer/src/modern_gpu.rs`
  - `wgpu` renderer for atlas-backed modern BG quads.
- Create `crates/renderer/src/modern_bg.wgsl`
  - Shader for drawing modern atlas-backed BG tile quads.
- Modify `crates/renderer/src/lib.rs`
  - Expose new modules and route opt-in rendering mode.
- Modify `crates/renderer/src/gpu_renderer.rs`
  - Add side-by-side modern render hook behind an enum/flag.
- Modify `crates/renderer/src/gpu_frame.rs`
  - Add test helpers only if needed; do not change the current runtime frame contract unless required.
- Modify `zelda3-bin/src/main.rs`
  - Add CLI/env plumbing for modern renderer comparison once renderer modules exist.

---

### Task 1: Add `ModernFrame` Scene Types

**Files:**
- Create: `crates/renderer/src/modern_frame.rs`
- Modify: `crates/renderer/src/lib.rs`

**Interfaces:**
- Produces:
  - `pub struct ModernFrame`
  - `pub struct ModernBgLayer`
  - `pub struct ModernTileInstance`
  - `pub struct ModernSpriteInstance`
  - `pub enum ModernBlendMode`
  - `pub const MODERN_FRAME_WIDTH: u16 = 256`
  - `pub const MODERN_FRAME_HEIGHT: u16 = 224`
- Consumes: none.

- [ ] **Step 1: Write the failing scene-type test**

Add this test module to `crates/renderer/src/modern_frame.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modern_frame_defaults_to_fixed_game_resolution() {
        let frame = ModernFrame::empty();

        assert_eq!(frame.width, MODERN_FRAME_WIDTH);
        assert_eq!(frame.height, MODERN_FRAME_HEIGHT);
        assert_eq!(frame.bg_layers.len(), 4);
        assert!(frame.sprites.is_empty());
        assert_eq!(frame.backdrop_color_rgba, [0, 0, 0, 0xff]);
    }

    #[test]
    fn tile_instance_records_modern_render_inputs_without_snes_memory_addresses() {
        let tile = ModernTileInstance {
            atlas_id: 17,
            atlas_x_px: 64,
            atlas_y_px: 32,
            atlas_width_px: 8,
            atlas_height_px: 8,
            screen_x: 12,
            screen_y: 20,
            palette: 3,
            priority: 1,
            hflip: true,
            vflip: false,
            transparent_color_zero: true,
        };

        assert_eq!(tile.atlas_id, 17);
        assert_eq!(tile.screen_x, 12);
        assert!(tile.hflip);
        assert!(tile.transparent_color_zero);
    }
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test -p renderer modern_frame -- --nocapture
```

Expected: compile failure because `modern_frame` and its types do not exist.

- [ ] **Step 3: Implement the scene types**

Create `crates/renderer/src/modern_frame.rs`:

```rust
pub const MODERN_FRAME_WIDTH: u16 = 256;
pub const MODERN_FRAME_HEIGHT: u16 = 224;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModernFrame {
    pub width: u16,
    pub height: u16,
    pub bg_layers: Vec<ModernBgLayer>,
    pub sprites: Vec<ModernSpriteInstance>,
    pub backdrop_color_rgba: [u8; 4],
    pub brightness: u8,
    pub forced_blank: bool,
}

impl ModernFrame {
    pub fn empty() -> Self {
        Self {
            width: MODERN_FRAME_WIDTH,
            height: MODERN_FRAME_HEIGHT,
            bg_layers: vec![
                ModernBgLayer::new(0),
                ModernBgLayer::new(1),
                ModernBgLayer::new(2),
                ModernBgLayer::new(3),
            ],
            sprites: Vec::new(),
            backdrop_color_rgba: [0, 0, 0, 0xff],
            brightness: 15,
            forced_blank: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModernBgLayer {
    pub index: u8,
    pub enabled_main: bool,
    pub enabled_sub: bool,
    pub scroll_x: u16,
    pub scroll_y: u16,
    pub tiles: Vec<ModernTileInstance>,
    pub blend_mode: ModernBlendMode,
}

impl ModernBgLayer {
    pub fn new(index: u8) -> Self {
        Self {
            index,
            enabled_main: false,
            enabled_sub: false,
            scroll_x: 0,
            scroll_y: 0,
            tiles: Vec::new(),
            blend_mode: ModernBlendMode::Opaque,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernTileInstance {
    pub atlas_id: u32,
    pub atlas_x_px: u16,
    pub atlas_y_px: u16,
    pub atlas_width_px: u16,
    pub atlas_height_px: u16,
    pub screen_x: i16,
    pub screen_y: i16,
    pub palette: u8,
    pub priority: u8,
    pub hflip: bool,
    pub vflip: bool,
    pub transparent_color_zero: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernSpriteInstance {
    pub atlas_id: u32,
    pub atlas_x_px: u16,
    pub atlas_y_px: u16,
    pub atlas_width_px: u16,
    pub atlas_height_px: u16,
    pub screen_x: i16,
    pub screen_y: i16,
    pub palette: u8,
    pub priority: u8,
    pub hflip: bool,
    pub vflip: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModernBlendMode {
    Opaque,
    Add,
    Subtract,
    HalfAdd,
    HalfSubtract,
}
```

Modify `crates/renderer/src/lib.rs` near the other module declarations:

```rust
pub mod modern_frame;
```

- [ ] **Step 4: Run the test and verify it passes**

Run:

```bash
cargo test -p renderer modern_frame -- --nocapture
```

Expected: both `modern_frame` tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/renderer/src/lib.rs crates/renderer/src/modern_frame.rs
git commit -m "renderer: add modern frame scene model"
```

---

### Task 2: Extract Modern BG Tiles From `GpuFrame`

**Files:**
- Create: `crates/renderer/src/modern_extract.rs`
- Modify: `crates/renderer/src/lib.rs`

**Interfaces:**
- Consumes:
  - `renderer::gpu_frame::GpuFrame`
  - `renderer::modern_frame::{ModernFrame, ModernTileInstance}`
- Produces:
  - `pub fn extract_modern_frame(frame: &GpuFrame<'_>) -> ModernFrame`
  - `pub fn decode_snes_tilemap_entry(entry: u16) -> ModernTileFields`
  - `pub struct ModernTileFields`

- [ ] **Step 1: Write failing decode tests**

Create `crates/renderer/src/modern_extract.rs` with:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_snes_tilemap_entry_splits_visual_fields() {
        let fields = decode_snes_tilemap_entry(0xed23);

        assert_eq!(fields.tile_number, 0x0123);
        assert_eq!(fields.palette, 3);
        assert!(fields.priority);
        assert!(fields.hflip);
        assert!(fields.vflip);
    }
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test -p renderer modern_extract -- --nocapture
```

Expected: compile failure because `decode_snes_tilemap_entry` and `ModernTileFields` are missing.

- [ ] **Step 3: Implement tilemap decode**

Add above the tests in `crates/renderer/src/modern_extract.rs`:

```rust
use crate::gpu_frame::GpuFrame;
use crate::modern_frame::{ModernFrame, ModernTileInstance};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModernTileFields {
    pub tile_number: u16,
    pub palette: u8,
    pub priority: bool,
    pub hflip: bool,
    pub vflip: bool,
}

pub fn decode_snes_tilemap_entry(entry: u16) -> ModernTileFields {
    ModernTileFields {
        tile_number: entry & 0x03ff,
        palette: ((entry >> 10) & 0x07) as u8,
        priority: entry & 0x2000 != 0,
        hflip: entry & 0x4000 != 0,
        vflip: entry & 0x8000 != 0,
    }
}

pub fn extract_modern_frame(frame: &GpuFrame<'_>) -> ModernFrame {
    let mut modern = ModernFrame::empty();
    modern.brightness = frame.brightness;
    modern.forced_blank = frame.forced_blank;
    modern
}
```

Modify `crates/renderer/src/lib.rs`:

```rust
pub mod modern_extract;
```

- [ ] **Step 4: Run decode tests**

Run:

```bash
cargo test -p renderer modern_extract -- --nocapture
```

Expected: decode test passes.

- [ ] **Step 5: Add BG extraction test**

Append this test to `crates/renderer/src/modern_extract.rs` after the decode test. Use the existing `GpuFrame` field list from `crates/renderer/src/gpu_frame.rs`; fill non-relevant fields with defaults from their `Default` implementations.

```rust
#[test]
fn extract_modern_frame_copies_frame_level_visual_state() {
    let vram = vec![0u16; 0x8000];
    let cgram = vec![0u16; 0x100];
    let oam = vec![0u16; 0x110];
    let frame = test_gpu_frame(&vram, &cgram, &oam, 9, true);

    let modern = extract_modern_frame(&frame);

    assert_eq!(modern.width, 256);
    assert_eq!(modern.height, 224);
    assert_eq!(modern.brightness, 9);
    assert!(modern.forced_blank);
}
```

Add the helper in the same test module, matching the actual `GpuFrame` fields in `gpu_frame.rs`:

```rust
fn test_gpu_frame<'a>(
    vram: &'a [u16],
    cgram: &'a [u16],
    oam: &'a [u16],
    brightness: u8,
    forced_blank: bool,
) -> GpuFrame<'a> {
    GpuFrame {
        vram,
        cgram,
        oam,
        mode: 1,
        bg: Default::default(),
        obj: Default::default(),
        mosaic_enabled: 0,
        mosaic_size: 0,
        extra_left_right: 0,
        mode7: Default::default(),
        screen_enabled: [0, 0],
        screen_windowed: [0, 0],
        brightness,
        forced_blank,
        math_enabled: 0,
        subtract_color: false,
        half_color: false,
        fixed_color_r: 0,
        fixed_color_g: 0,
        fixed_color_b: 0,
        add_subscreen: false,
        clip_mode: 0,
        prevent_math_mode: 0,
        windowsel_cm: 0,
        windowsel: 0,
        scanlines: Default::default(),
    }
}
```

- [ ] **Step 6: Run and fix struct-field drift**

Run:

```bash
cargo test -p renderer modern_extract -- --nocapture
```

Expected: either PASS or compile errors identifying exact missing `GpuFrame` fields. If fields have drifted, update only `test_gpu_frame` to include the missing fields with default values.

- [ ] **Step 7: Commit**

```bash
git add crates/renderer/src/lib.rs crates/renderer/src/modern_extract.rs
git commit -m "renderer: extract modern frame state"
```

---

### Task 3: Add CPU Reference Renderer For Modern BG Tiles

**Files:**
- Create: `crates/renderer/src/modern_software.rs`
- Modify: `crates/renderer/src/lib.rs`

**Interfaces:**
- Consumes:
  - `ModernFrame`
  - `ModernTileInstance`
- Produces:
  - `pub fn render_modern_frame_software(frame: &ModernFrame, atlas_rgba: &[u8], atlas_width: u16, atlas_height: u16) -> Vec<u8>`

- [ ] **Step 1: Write failing software-render tests**

Create `crates/renderer/src/modern_software.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_frame::{ModernBgLayer, ModernBlendMode, ModernFrame, ModernTileInstance};

    #[test]
    fn software_renderer_blits_one_opaque_tile_from_atlas() {
        let mut atlas = vec![0u8; 8 * 8 * 4];
        for px in atlas.chunks_exact_mut(4) {
            px.copy_from_slice(&[10, 20, 30, 0xff]);
        }
        let mut frame = ModernFrame::empty();
        let mut layer = ModernBgLayer::new(0);
        layer.enabled_main = true;
        layer.blend_mode = ModernBlendMode::Opaque;
        layer.tiles.push(ModernTileInstance {
            atlas_id: 0,
            atlas_x_px: 0,
            atlas_y_px: 0,
            atlas_width_px: 8,
            atlas_height_px: 8,
            screen_x: 4,
            screen_y: 5,
            palette: 0,
            priority: 0,
            hflip: false,
            vflip: false,
            transparent_color_zero: false,
        });
        frame.bg_layers[0] = layer;

        let rgba = render_modern_frame_software(&frame, &atlas, 8, 8);
        let offset = ((5usize * 256) + 4usize) * 4;

        assert_eq!(&rgba[offset..offset + 4], &[10, 20, 30, 0xff]);
    }
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test -p renderer modern_software -- --nocapture
```

Expected: compile failure because the module/function do not exist.

- [ ] **Step 3: Implement minimal blitter**

Add above the test module:

```rust
use crate::modern_frame::{ModernFrame, MODERN_FRAME_HEIGHT, MODERN_FRAME_WIDTH};

pub fn render_modern_frame_software(
    frame: &ModernFrame,
    atlas_rgba: &[u8],
    atlas_width: u16,
    atlas_height: u16,
) -> Vec<u8> {
    let mut out = vec![0u8; usize::from(MODERN_FRAME_WIDTH) * usize::from(MODERN_FRAME_HEIGHT) * 4];
    for px in out.chunks_exact_mut(4) {
        px.copy_from_slice(&frame.backdrop_color_rgba);
    }
    if frame.forced_blank {
        for px in out.chunks_exact_mut(4) {
            px.copy_from_slice(&[0, 0, 0, 0xff]);
        }
        return out;
    }
    for layer in &frame.bg_layers {
        if !layer.enabled_main {
            continue;
        }
        for tile in &layer.tiles {
            for y in 0..tile.atlas_height_px {
                for x in 0..tile.atlas_width_px {
                    let src_x = if tile.hflip { tile.atlas_width_px - 1 - x } else { x };
                    let src_y = if tile.vflip { tile.atlas_height_px - 1 - y } else { y };
                    let atlas_x = tile.atlas_x_px + src_x;
                    let atlas_y = tile.atlas_y_px + src_y;
                    if atlas_x >= atlas_width || atlas_y >= atlas_height {
                        continue;
                    }
                    let dst_x = tile.screen_x + x as i16;
                    let dst_y = tile.screen_y + y as i16;
                    if dst_x < 0 || dst_y < 0 || dst_x >= 256 || dst_y >= 224 {
                        continue;
                    }
                    let src = ((usize::from(atlas_y) * usize::from(atlas_width)
                        + usize::from(atlas_x))
                        * 4) as usize;
                    let dst = ((dst_y as usize * 256 + dst_x as usize) * 4) as usize;
                    if tile.transparent_color_zero && atlas_rgba[src + 3] == 0 {
                        continue;
                    }
                    out[dst..dst + 4].copy_from_slice(&atlas_rgba[src..src + 4]);
                }
            }
        }
    }
    out
}
```

Modify `crates/renderer/src/lib.rs`:

```rust
pub mod modern_software;
```

- [ ] **Step 4: Run software renderer tests**

Run:

```bash
cargo test -p renderer modern_software -- --nocapture
```

Expected: software renderer test passes.

- [ ] **Step 5: Commit**

```bash
git add crates/renderer/src/lib.rs crates/renderer/src/modern_software.rs
git commit -m "renderer: add modern software blitter"
```

---

### Task 4: Load Repo PNG Atlas For Modern Rendering

**Files:**
- Create: `crates/renderer/src/modern_assets.rs`
- Modify: `crates/renderer/src/lib.rs`
- Test fixture: use `zelda3-bin/developer_tilesets/overworld_unique_tiles.png` and `zelda3-bin/developer_tilesets/overworld_unique_tiles.json`

**Interfaces:**
- Produces:
  - `pub struct ModernTileAtlasAsset`
  - `pub struct ModernTileAtlasEntry`
  - `pub fn load_modern_overworld_tile_atlas(repo_root: &Path) -> Result<ModernTileAtlasAsset, String>`

- [ ] **Step 1: Write failing asset-load test**

Create `crates/renderer/src/modern_assets.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn loads_repo_overworld_tile_atlas_manifest_and_png() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

        let atlas = load_modern_overworld_tile_atlas(&root).expect("atlas should load");

        assert_eq!(atlas.tile_width_px, 8);
        assert_eq!(atlas.tile_height_px, 8);
        assert_eq!(atlas.entries.len(), 6140);
        assert_eq!(atlas.width_px, 2113);
        assert_eq!(atlas.height_px, 3169);
        assert!(!atlas.rgba.is_empty());
        assert_eq!(atlas.entries[0].atlas_x_px, 1);
    }
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test -p renderer modern_assets -- --nocapture
```

Expected: compile failure because the loader does not exist.

- [ ] **Step 3: Implement loader**

Add above the test module:

```rust
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct ModernTileAtlasAsset {
    pub tile_width_px: u16,
    pub tile_height_px: u16,
    pub width_px: u32,
    pub height_px: u32,
    pub rgba: Vec<u8>,
    pub entries: Vec<ModernTileAtlasEntry>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ModernTileAtlasEntry {
    pub id: u32,
    pub atlas_x_px: u16,
    pub atlas_y_px: u16,
    pub atlas_width_px: u16,
    pub atlas_height_px: u16,
}

#[derive(Deserialize)]
struct Manifest {
    tile_width_px: u16,
    tile_height_px: u16,
    unique_tiles: Vec<ModernTileAtlasEntry>,
}

pub fn load_modern_overworld_tile_atlas(repo_root: &Path) -> Result<ModernTileAtlasAsset, String> {
    let base = repo_root.join("zelda3-bin/developer_tilesets");
    let manifest_path = base.join("overworld_unique_tiles.json");
    let png_path = base.join("overworld_unique_tiles.png");
    let manifest_bytes = fs::read(&manifest_path)
        .map_err(|e| format!("failed to read {}: {e}", manifest_path.display()))?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("failed to parse {}: {e}", manifest_path.display()))?;
    let image = image::open(&png_path)
        .map_err(|e| format!("failed to open {}: {e}", png_path.display()))?
        .to_rgba8();
    let (width_px, height_px) = image.dimensions();
    Ok(ModernTileAtlasAsset {
        tile_width_px: manifest.tile_width_px,
        tile_height_px: manifest.tile_height_px,
        width_px,
        height_px,
        rgba: image.into_raw(),
        entries: manifest.unique_tiles,
    })
}
```

Modify `crates/renderer/src/lib.rs`:

```rust
pub mod modern_assets;
```

- [ ] **Step 4: Add dependencies only if needed**

If `crates/renderer/Cargo.toml` does not already depend on `image`, `serde`, and `serde_json`, add:

```toml
image = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
```

If these are not workspace dependencies, use the same versions already used elsewhere in the repo.

- [ ] **Step 5: Run asset tests**

Run:

```bash
cargo test -p renderer modern_assets -- --nocapture
```

Expected: atlas test passes and confirms the repo-stored generated assets load.

- [ ] **Step 6: Commit**

```bash
git add crates/renderer/src/lib.rs crates/renderer/src/modern_assets.rs crates/renderer/Cargo.toml
git commit -m "renderer: load modern tile atlas assets"
```

---

### Task 5: Map Extracted Tiles To Atlas Entries

**Files:**
- Modify: `crates/renderer/src/modern_extract.rs`
- Modify: `crates/renderer/src/modern_assets.rs`

**Interfaces:**
- Consumes:
  - `ModernTileAtlasAsset`
  - `decode_snes_tilemap_entry`
- Produces:
  - `pub fn atlas_entry_for_tilemap_entry(asset: &ModernTileAtlasAsset, tilemap_entry: u16) -> Option<&ModernTileAtlasEntry>`

- [ ] **Step 1: Write failing lookup test**

Append to `modern_assets.rs` tests:

```rust
#[test]
fn atlas_lookup_finds_tilemap_variant() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let atlas = load_modern_overworld_tile_atlas(&root).expect("atlas should load");

    let entry = atlas_entry_for_tilemap_entry(&atlas, 2218).expect("tilemap entry should exist");

    assert_eq!(entry.id, 0);
    assert_eq!(entry.atlas_x_px, 1);
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test -p renderer modern_assets::tests::atlas_lookup_finds_tilemap_variant -- --nocapture
```

Expected: compile failure because lookup is missing or manifest entry lacks variants.

- [ ] **Step 3: Extend manifest entry and implement lookup**

Update `ModernTileAtlasEntry` in `modern_assets.rs`:

```rust
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ModernTileAtlasEntry {
    pub id: u32,
    pub atlas_x_px: u16,
    pub atlas_y_px: u16,
    pub atlas_width_px: u16,
    pub atlas_height_px: u16,
    pub tilemap_entry: u16,
    pub tilemap_variants: Vec<u16>,
}
```

Add:

```rust
pub fn atlas_entry_for_tilemap_entry<'a>(
    asset: &'a ModernTileAtlasAsset,
    tilemap_entry: u16,
) -> Option<&'a ModernTileAtlasEntry> {
    asset
        .entries
        .iter()
        .find(|entry| entry.tilemap_entry == tilemap_entry || entry.tilemap_variants.contains(&tilemap_entry))
}
```

- [ ] **Step 4: Run lookup tests**

Run:

```bash
cargo test -p renderer modern_assets -- --nocapture
```

Expected: all modern asset tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/renderer/src/modern_assets.rs
git commit -m "renderer: map tilemap entries to modern atlas tiles"
```

---

### Task 6: Extract Visible BG1/BG2 Tile Instances Into `ModernFrame`

**Files:**
- Modify: `crates/renderer/src/modern_extract.rs`
- Test: `crates/renderer/src/modern_extract.rs`

**Interfaces:**
- Consumes:
  - `GpuFrame`
  - `ModernTileAtlasAsset`
- Produces:
  - `pub fn extract_modern_frame_with_atlas(frame: &GpuFrame<'_>, atlas: &ModernTileAtlasAsset) -> ModernFrame`

- [ ] **Step 1: Write failing BG tile extraction test**

Add a focused test that creates VRAM with one BG1 tilemap entry and confirms a `ModernTileInstance` points at the atlas entry:

```rust
#[test]
fn extract_modern_frame_maps_bg_tilemap_entry_to_atlas_tile() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let atlas = crate::modern_assets::load_modern_overworld_tile_atlas(&root).expect("atlas should load");
    let mut vram = vec![0u16; 0x8000];
    let cgram = vec![0u16; 0x100];
    let oam = vec![0u16; 0x110];
    vram[0] = 2218;
    let mut frame = test_gpu_frame(&vram, &cgram, &oam, 15, false);
    frame.bg[0].tilemap_adr = 0;
    frame.bg[0].tile_adr = 0x2000;
    frame.screen_enabled = [0x01, 0x00];

    let modern = extract_modern_frame_with_atlas(&frame, &atlas);

    assert_eq!(modern.bg_layers[0].tiles.len(), 1);
    assert_eq!(modern.bg_layers[0].tiles[0].atlas_x_px, 1);
    assert_eq!(modern.bg_layers[0].tiles[0].screen_x, 0);
    assert_eq!(modern.bg_layers[0].tiles[0].screen_y, 0);
}
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test -p renderer extract_modern_frame_maps_bg_tilemap_entry_to_atlas_tile -- --nocapture
```

Expected: compile failure because `extract_modern_frame_with_atlas` is missing.

- [ ] **Step 3: Implement minimal BG extraction**

Add to `modern_extract.rs`:

```rust
use crate::modern_assets::{atlas_entry_for_tilemap_entry, ModernTileAtlasAsset};

pub fn extract_modern_frame_with_atlas(
    frame: &GpuFrame<'_>,
    atlas: &ModernTileAtlasAsset,
) -> ModernFrame {
    let mut modern = extract_modern_frame(frame);
    for layer_index in 0..3usize {
        let enabled_main = frame.screen_enabled[0] & (1 << layer_index) != 0;
        modern.bg_layers[layer_index].enabled_main = enabled_main;
        modern.bg_layers[layer_index].enabled_sub = frame.screen_enabled[1] & (1 << layer_index) != 0;
        modern.bg_layers[layer_index].scroll_x = frame.bg[layer_index].h_scroll;
        modern.bg_layers[layer_index].scroll_y = frame.bg[layer_index].v_scroll;
        if !enabled_main {
            continue;
        }
        let base = frame.bg[layer_index].tilemap_adr as usize;
        for row in 0..32usize {
            for col in 0..32usize {
                let entry_word = *frame.vram.get(base + row * 32 + col).unwrap_or(&0);
                if entry_word == 0 {
                    continue;
                }
                let Some(atlas_entry) = atlas_entry_for_tilemap_entry(atlas, entry_word) else {
                    continue;
                };
                let fields = decode_snes_tilemap_entry(entry_word);
                modern.bg_layers[layer_index].tiles.push(ModernTileInstance {
                    atlas_id: atlas_entry.id,
                    atlas_x_px: atlas_entry.atlas_x_px,
                    atlas_y_px: atlas_entry.atlas_y_px,
                    atlas_width_px: atlas_entry.atlas_width_px,
                    atlas_height_px: atlas_entry.atlas_height_px,
                    screen_x: (col * 8) as i16 - frame.bg[layer_index].h_scroll as i16,
                    screen_y: (row * 8) as i16 - frame.bg[layer_index].v_scroll as i16,
                    palette: fields.palette,
                    priority: u8::from(fields.priority),
                    hflip: fields.hflip,
                    vflip: fields.vflip,
                    transparent_color_zero: true,
                });
            }
        }
    }
    modern
}
```

- [ ] **Step 4: Run extraction tests**

Run:

```bash
cargo test -p renderer modern_extract -- --nocapture
```

Expected: all modern extraction tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/renderer/src/modern_extract.rs
git commit -m "renderer: extract modern bg tile instances"
```

---

### Task 7: Add Modern GPU BG Renderer Behind A Flag

**Files:**
- Create: `crates/renderer/src/modern_gpu.rs`
- Create: `crates/renderer/src/modern_bg.wgsl`
- Modify: `crates/renderer/src/lib.rs`
- Modify: `crates/renderer/src/gpu_renderer.rs`

**Interfaces:**
- Consumes:
  - `ModernFrame`
  - `ModernTileAtlasAsset`
- Produces:
  - `pub struct ModernGpuRenderer`
  - `pub fn render(&mut self, encoder: &mut wgpu::CommandEncoder, queue: &wgpu::Queue, frame: &ModernFrame, output_view: &wgpu::TextureView)`

- [ ] **Step 1: Add a constructor smoke test**

Create a test using the repo’s existing headless `wgpu` helper if present. If no helper exists, add a small test under `modern_gpu.rs` that uses the same `create_wgpu_instance` and `create_device_queue` functions already used by `FrameRenderer::new`.

```rust
#[test]
fn modern_gpu_renderer_constructs() {
    pollster::block_on(async {
        let instance = crate::create_wgpu_instance();
        let (_adapter, device, queue) = crate::create_device_queue(&instance, None).await;
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let atlas = crate::modern_assets::load_modern_overworld_tile_atlas(&root).expect("atlas should load");

        let _renderer = ModernGpuRenderer::new(&device, &queue, &atlas, wgpu::TextureFormat::Rgba8Unorm);
    });
}
```

- [ ] **Step 2: Run constructor test and verify it fails**

Run:

```bash
cargo test -p renderer modern_gpu_renderer_constructs -- --nocapture
```

Expected: compile failure because `ModernGpuRenderer` does not exist.

- [ ] **Step 3: Implement minimal GPU renderer**

Implement `ModernGpuRenderer` with:

```rust
pub struct ModernGpuRenderer {
    pipeline: wgpu::RenderPipeline,
    atlas_texture: wgpu::Texture,
    atlas_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
}
```

Upload `ModernTileAtlasAsset.rgba` into `atlas_texture` with `queue.write_texture`. Use nearest sampling and `Rgba8Unorm`.

- [ ] **Step 4: Implement WGSL shader**

Create `crates/renderer/src/modern_bg.wgsl` with a full-screen placeholder pass first:

```wgsl
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0)
    );
    return vec4<f32>(pos[vi], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}
```

- [ ] **Step 5: Run constructor test**

Run:

```bash
cargo test -p renderer modern_gpu_renderer_constructs -- --nocapture
```

Expected: constructor test passes.

- [ ] **Step 6: Commit**

```bash
git add crates/renderer/src/lib.rs crates/renderer/src/modern_gpu.rs crates/renderer/src/modern_bg.wgsl crates/renderer/src/gpu_renderer.rs
git commit -m "renderer: add modern gpu renderer scaffold"
```

---

### Task 8: Add Side-By-Side Pixel Compare For Modern BG Subset

**Files:**
- Modify: `zelda3-bin/src/main.rs`
- Modify: `crates/renderer/src/gpu_renderer.rs`

**Interfaces:**
- Consumes:
  - Existing GPU render compare command pattern.
- Produces:
  - CLI flag `--modern-render-compare <stride>`
  - Log line `modern_render_compare frame=<n> old=<hash> modern=<hash> match=<true|false>`

- [ ] **Step 1: Write failing CLI parse test if main has parse helpers**

If `zelda3-bin/src/main.rs` has parse helper tests, add:

```rust
#[test]
fn play_args_parse_modern_render_compare_stride() {
    let args = vec![
        "zelda3".to_string(),
        "--play".to_string(),
        "saves/zelda3.sfc".to_string(),
        "--modern-render-compare".to_string(),
        "60".to_string(),
    ];

    let parsed = parse_play_args_for_test(&args);

    assert_eq!(parsed.modern_render_compare, 60);
}
```

If no parse helper exists, add this step as an integration-level smoke test after implementation instead of adding a new parser abstraction.

- [ ] **Step 2: Implement CLI state**

In `zelda3-bin/src/main.rs`, near existing `gpu_render_compare` fields, add:

```rust
let mut modern_render_compare = 0u32;
```

In the play-arg match:

```rust
"--modern-render-compare" => {
    let stride = args.get(i + 1).unwrap_or_else(|| {
        eprintln!("--modern-render-compare requires a stride");
        process::exit(2);
    });
    modern_render_compare = stride.parse::<u32>().unwrap_or_else(|_| {
        eprintln!("invalid --modern-render-compare stride: {stride}");
        process::exit(2);
    });
    if modern_render_compare == 0 {
        eprintln!("--modern-render-compare stride must be greater than zero");
        process::exit(2);
    }
    i += 2;
}
```

- [ ] **Step 3: Add renderer hook**

In `crates/renderer/src/gpu_renderer.rs`, add a method that renders the modern frame to an offscreen `Rgba8Unorm` texture only when the feature flag is active:

```rust
pub fn render_modern_frame_for_compare(
    &mut self,
    encoder: &mut wgpu::CommandEncoder,
    queue: &wgpu::Queue,
    frame: &GpuFrame<'_>,
    output_view: &wgpu::TextureView,
) {
    let _ = (encoder, queue, frame, output_view);
}
```

This placeholder must compile first; Task 9 will make it render real BG tiles.

- [ ] **Step 4: Run compile check**

Run:

```bash
cargo check -p zelda3-bin
```

Expected: compile passes.

- [ ] **Step 5: Commit**

```bash
git add zelda3-bin/src/main.rs crates/renderer/src/gpu_renderer.rs
git commit -m "renderer: add modern render compare flag"
```

---

### Task 9: Render Modern BG Tiles And Compare Against Existing Renderer

**Files:**
- Modify: `crates/renderer/src/modern_gpu.rs`
- Modify: `crates/renderer/src/modern_bg.wgsl`
- Modify: `crates/renderer/src/gpu_renderer.rs`

**Interfaces:**
- Consumes:
  - `extract_modern_frame_with_atlas`
  - `ModernGpuRenderer`
- Produces:
  - Modern BG render output for Mode 1 overworld BG subset.

- [ ] **Step 1: Add software-to-GPU parity test for one tile**

Add a test that renders one `ModernFrame` through `ModernGpuRenderer` into a texture, reads back pixels, and compares against `render_modern_frame_software`.

Expected assertion:

```rust
assert_eq!(gpu_rgba, software_rgba);
```

Use an 8x8 atlas with one red tile and one `ModernTileInstance` at `(0, 0)`.

- [ ] **Step 2: Run the test and verify it fails**

Run:

```bash
cargo test -p renderer modern_gpu_one_tile_matches_software -- --nocapture
```

Expected: failure because the shader is still a black placeholder.

- [ ] **Step 3: Implement tile instance buffer**

In `modern_gpu.rs`, add a packed instance struct:

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ModernTileGpuInstance {
    atlas_xywh: [u32; 4],
    screen_xy: [i32; 2],
    flags: u32,
    _pad: u32,
}
```

Upload all visible tile instances each frame.

- [ ] **Step 4: Implement shader sampling**

Update `modern_bg.wgsl` to render instanced quads:

```wgsl
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};
```

Use nearest texture sampling, discard alpha-zero pixels, and honor h/v flip via `flags`.

- [ ] **Step 5: Run GPU/software test**

Run:

```bash
cargo test -p renderer modern_gpu_one_tile_matches_software -- --nocapture
```

Expected: GPU output exactly matches software output for the one-tile case.

- [ ] **Step 6: Commit**

```bash
git add crates/renderer/src/modern_gpu.rs crates/renderer/src/modern_bg.wgsl crates/renderer/src/gpu_renderer.rs
git commit -m "renderer: render modern bg tile quads"
```

---

### Task 10: Establish Route-Based Modern Parity Gate

**Files:**
- Modify: `scripts/`
- Modify: `zelda3-bin/src/main.rs`
- Create: `scripts/test_modern_render_parity.py`

**Interfaces:**
- Consumes:
  - `--modern-render-compare`
  - existing replay route assets.
- Produces:
  - `scripts/test_modern_render_parity.py`

- [ ] **Step 1: Write failing script smoke test**

Create `scripts/test_modern_render_parity.py`:

```python
#!/usr/bin/env python3
import argparse
import subprocess
import sys

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--rom", default="saves/zelda3.sfc")
    parser.add_argument("--frames", type=int, default=300)
    parser.add_argument("--stride", type=int, default=30)
    args = parser.parse_args()

    cmd = [
        "cargo", "run", "-p", "zelda3-bin", "--",
        "--play", args.rom, str(args.frames),
        "--modern-render-compare", str(args.stride),
    ]
    result = subprocess.run(cmd, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    sys.stdout.write(result.stdout)
    if result.returncode != 0:
        return result.returncode
    if "modern_render_compare" not in result.stdout:
        print("expected modern_render_compare output", file=sys.stderr)
        return 1
    if "match=false" in result.stdout:
        print("modern renderer mismatch", file=sys.stderr)
        return 1
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
```

- [ ] **Step 2: Run script and verify it fails before compare output exists**

Run:

```bash
python3 scripts/test_modern_render_parity.py --frames 60 --stride 30
```

Expected: fails with `expected modern_render_compare output` until Task 8/9 logging is wired.

- [ ] **Step 3: Wire compare output**

In `zelda3-bin/src/main.rs`, after each frame where `modern_render_compare != 0 && frame % modern_render_compare == 0`, log:

```rust
println!(
    "modern_render_compare frame={} old={} modern={} match={}",
    frame_index,
    old_hash,
    modern_hash,
    old_hash == modern_hash
);
```

Use the same frame hashing helper currently used by GPU render compare. If no shared helper exists, extract one local helper:

```rust
fn fnv32_rgba(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c9dc5u32;
    for &byte in bytes {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}
```

- [ ] **Step 4: Run short modern parity script**

Run:

```bash
python3 scripts/test_modern_render_parity.py --frames 60 --stride 30
```

Expected: logs `modern_render_compare` lines. It may report `match=false` at this stage; if so, leave the script failing and document the mismatch count in the commit message.

- [ ] **Step 5: Commit**

```bash
git add scripts/test_modern_render_parity.py zelda3-bin/src/main.rs
git commit -m "renderer: add modern render parity gate"
```

---

### Task 11: Expand Modern Renderer Feature Coverage

**Files:**
- Modify: `crates/renderer/src/modern_extract.rs`
- Modify: `crates/renderer/src/modern_software.rs`
- Modify: `crates/renderer/src/modern_gpu.rs`
- Modify: `crates/renderer/src/modern_bg.wgsl`

**Interfaces:**
- Consumes:
  - Existing failed parity logs from Task 10.
- Produces:
  - Support for the next missing visible feature, one feature per commit.

- [ ] **Step 1: Pick the first mismatch class**

Run:

```bash
python3 scripts/test_modern_render_parity.py --frames 600 --stride 30
```

Expected: failure if modern path does not yet match. Classify the first mismatch into one of:

```text
scroll
priority
palette
transparent color zero
window mask
color math
sprite overlap
animated tile
mode7
mosaic
```

- [ ] **Step 2: Write one failing unit test for that class**

Example for priority:

```rust
#[test]
fn software_renderer_draws_higher_priority_tile_last() {
    let atlas = two_color_test_atlas();
    let mut frame = ModernFrame::empty();
    frame.bg_layers[0].enabled_main = true;
    frame.bg_layers[0].tiles.push(tile_instance(0, 0, 0, 0));
    frame.bg_layers[0].tiles.push(tile_instance(1, 0, 0, 1));

    let rgba = render_modern_frame_software(&frame, &atlas, 16, 8);

    assert_eq!(&rgba[0..4], &[200, 0, 0, 0xff]);
}
```

- [ ] **Step 3: Implement only that feature**

For priority, sort instances by `(layer.index, tile.priority)` before rendering in software and GPU upload. Do not implement unrelated features in the same task.

- [ ] **Step 4: Run unit tests**

Run:

```bash
cargo test -p renderer modern_ -- --nocapture
```

Expected: all modern renderer unit tests pass.

- [ ] **Step 5: Run parity script**

Run:

```bash
python3 scripts/test_modern_render_parity.py --frames 600 --stride 30
```

Expected: mismatch count decreases or reaches zero for the tested route segment.

- [ ] **Step 6: Commit**

```bash
git add crates/renderer/src/modern_extract.rs crates/renderer/src/modern_software.rs crates/renderer/src/modern_gpu.rs crates/renderer/src/modern_bg.wgsl
git commit -m "renderer: add modern parity support for <feature>"
```

- [ ] **Step 7: Repeat Task 11**

Repeat Task 11 until the route segment reports no mismatches. Each repetition must target exactly one mismatch class.

---

### Task 12: Enable Developer Rooms To Emit `ModernFrame` Directly

**Files:**
- Modify: `zelda3-bin/src/developer_destinations.rs`
- Modify: `zelda3-bin/src/main.rs`
- Create: `zelda3-bin/src/developer_modern_map.rs`

**Interfaces:**
- Consumes:
  - `zelda3-bin/developer_rooms/*.json`
  - `zelda3-bin/developer_tilesets/overworld_unique_tiles.json`
- Produces:
  - `pub fn load_developer_modern_frame(room_id: &str) -> Result<ModernFrame, String>`

- [ ] **Step 1: Write failing developer-map test**

Create `zelda3-bin/src/developer_modern_map.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn developer_sandbox_can_emit_modern_frame_without_vram_addresses() {
        let frame = load_developer_modern_frame("preset-dev-sandbox")
            .expect("sandbox should emit modern frame");

        assert_eq!(frame.width, 256);
        assert_eq!(frame.height, 224);
        assert!(frame.bg_layers.iter().any(|layer| !layer.tiles.is_empty()));
    }
}
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```bash
cargo test -p zelda3-bin developer_sandbox_can_emit_modern_frame_without_vram_addresses -- --nocapture
```

Expected: compile failure because `developer_modern_map` and loader are missing.

- [ ] **Step 3: Implement JSON-to-ModernFrame loader**

Load the developer room JSON, resolve each tile id through `overworld_unique_tiles.json`, and produce `ModernTileInstance` values. Use no VRAM addresses in this loader.

Function signature:

```rust
pub fn load_developer_modern_frame(room_id: &str) -> Result<renderer::modern_frame::ModernFrame, String>
```

- [ ] **Step 4: Run developer modern map test**

Run:

```bash
cargo test -p zelda3-bin developer_sandbox_can_emit_modern_frame_without_vram_addresses -- --nocapture
```

Expected: test passes.

- [ ] **Step 5: Commit**

```bash
git add zelda3-bin/src/developer_modern_map.rs zelda3-bin/src/developer_destinations.rs zelda3-bin/src/main.rs
git commit -m "developer: emit modern frames from room json"
```

---

### Task 13: Add Runtime Mode Selection

**Files:**
- Modify: `crates/renderer/src/lib.rs`
- Modify: `crates/renderer/src/gpu_renderer.rs`
- Modify: `zelda3-bin/src/main.rs`

**Interfaces:**
- Produces:
  - Env var `ZELDA3_RENDERER=classic|modern-compare|modern`
  - Default remains `classic`.

- [ ] **Step 1: Write failing mode parse test**

Add:

```rust
#[test]
fn renderer_mode_parse_defaults_to_classic() {
    assert_eq!(RendererMode::parse(None), RendererMode::Classic);
    assert_eq!(RendererMode::parse(Some("modern-compare")), RendererMode::ModernCompare);
    assert_eq!(RendererMode::parse(Some("modern")), RendererMode::Modern);
}
```

- [ ] **Step 2: Implement mode enum**

In `crates/renderer/src/lib.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RendererMode {
    Classic,
    ModernCompare,
    Modern,
}

impl RendererMode {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("modern-compare") => Self::ModernCompare,
            Some("modern") => Self::Modern,
            _ => Self::Classic,
        }
    }
}
```

- [ ] **Step 3: Wire env var**

Read:

```rust
let renderer_mode = RendererMode::parse(std::env::var("ZELDA3_RENDERER").ok().as_deref());
```

Pass this into `GpuFrameRenderer`/`FrameRenderer`.

- [ ] **Step 4: Keep default classic**

Run without env var:

```bash
cargo run -p zelda3-bin -- --play saves/zelda3.sfc 60
```

Expected: current classic renderer behavior.

- [ ] **Step 5: Run modern compare**

Run:

```bash
ZELDA3_RENDERER=modern-compare cargo run -p zelda3-bin -- --play saves/zelda3.sfc 60 --modern-render-compare 30
```

Expected: compare logs are emitted.

- [ ] **Step 6: Commit**

```bash
git add crates/renderer/src/lib.rs crates/renderer/src/gpu_renderer.rs zelda3-bin/src/main.rs
git commit -m "renderer: add modern renderer runtime mode"
```

---

## Final Verification

Run:

```bash
cargo fmt --check
git diff --check
cargo test -p renderer modern_ -- --nocapture
cargo test -p zelda3-bin developer_sandbox_can_emit_modern_frame_without_vram_addresses -- --nocapture
cargo check -p zelda3-bin
python3 scripts/test_modern_render_parity.py --frames 600 --stride 30
```

Expected:

- Formatting passes.
- Diff whitespace check passes.
- Modern renderer unit tests pass.
- Developer room `ModernFrame` test passes.
- `zelda3-bin` compiles.
- The parity script either passes or reports the next explicitly documented mismatch class. Do not enable `ZELDA3_RENDERER=modern` by default until the parity script passes across agreed replay routes.

## Self-Review

- Spec coverage: The plan introduces a modern scene model, PNG atlas loading, extraction from current frame state, software/GPU renderers, parity comparison, developer-room direct output, and runtime selection.
- Placeholder scan: No task uses open-ended placeholders; each task has exact file paths, commands, and expected outcomes.
- Type consistency: `ModernFrame`, `ModernTileInstance`, `ModernTileAtlasAsset`, and `ModernGpuRenderer` are defined before later tasks consume them.
- Scope control: This plan does not remove the current renderer. It creates an opt-in modern path and grows parity coverage feature by feature.
