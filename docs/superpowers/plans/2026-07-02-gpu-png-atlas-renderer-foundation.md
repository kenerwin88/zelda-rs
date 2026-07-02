# GPU PNG-Atlas Renderer — Sub-project 1 (Foundation) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Draw the live interactive frame from the PNG source atlas on the GPU, opt-in as `ZELDA3_RENDERER=assets-anim-gpu`, plus a GPU arm of `--modern-index-compare` that measures GPU-vs-classic per-frame pixel mismatch.

**Architecture:** Reuse the existing `ModernGpuIndexRenderer` / `ModernGpuSpriteRenderer` (index-atlas + live-CGRAM GPU renderers, already GPU==CPU tested). Refactor them to accept per-frame cells (persistent pipeline, per-frame atlas upload), wrap them in a `ModernGpuCompositor` that renders BG-then-OBJ (simple z-order) into an offscreen `Rgba8Unorm` texture, and wire that into `FrameRenderer` → `NativeFrontend` → `GpuPlayRenderer`. The compositor renders into the renderer's existing 256×224 `game_texture` (via GPU texture-copy, no readback) and reuses the standard blit/presentation path.

**Tech Stack:** Rust, wgpu (headless + surface), the `renderer` / `platform` / `zelda3-bin` crates. Existing modern-frame data model (`ModernFrame`, `ModernIndexTile`, `ModernIndexTileInstance`).

## Global Constraints

- **Byte-identity is the bar** (this sub-project only reaches it on effect-free frames): the GPU compositor's output MUST equal the *simple z-order* CPU reference `render_modern_frame_software_indexed(&frame, &bg_cells)` composed with `draw_modern_sprites_indexed(&mut out, &frame, &sprite_cells)`. It is NOT expected to equal `render_modern_frame_full` (priority-interleave / color-math / window / HDMA / mosaic are sub-projects 2–5).
- **Do not change the default renderer.** The interactive default stays `assets-anim` (CPU). `assets-anim-gpu` is opt-in. `classic` still selects the wgpu PPU.
- **Do not touch the replay/parity harness default.** `run_replay_save` reads `ZELDA3_RENDERER` through its own `assets_anim_mode` gate and must still default to classic (render-hash/fingerprint gates unaffected). Verify: `--modern-index-compare` with `ZELDA3_RENDERER` unset still reports `via=vram`.
- **The existing GPU==CPU tests in `crates/renderer/src/modern_gpu.rs` must keep passing** after the Task 1 refactor (they are the regression guard).
- **Never `git checkout <file>`** (nukes unstaged WIP) and **never `git add -A`/`.`** — stage only the files each task names. The user works the repo concurrently; commit with `--no-verify` (the heavy pre-commit hook races their commits).
- Commit messages end with `Claude-Session: https://claude.ai/code/session_01R7ZVWfMDiByS4aBvrXNFTk`.
- Build/test profile: `cargo test -p renderer` for renderer-crate tests; `cargo build --profile parity -p zelda3-bin` for the binary. Replay/interactive runs need the 7 timing-hack env vars (see the repo `CLAUDE.md`).
- Branch: do this work on a dedicated branch off `main` (e.g. `gpu-atlas-foundation`); the current working tree has an unrelated in-progress `renderer-default-assets-anim` branch — do not disturb it.

---

## File Structure

- `crates/renderer/src/modern_gpu.rs` — **modify.** Refactor `ModernGpuIndexRenderer` / `ModernGpuSpriteRenderer` to per-frame cells (Task 1). Add `ModernGpuCompositor` + `ModernGpuHeadless` (Task 2).
- `crates/renderer/src/lib.rs` — **modify.** Re-export the new public types; add `FrameRenderer::present_modern_gpu` + a lazily-built compositor field (Task 3).
- `crates/platform/src/lib.rs` — **modify.** Add `NativeFrontend::present_modern_gpu` passthrough (Task 4).
- `zelda3-bin/src/main.rs` — **modify.** `assets-anim-gpu` selection + `GpuPlayRenderer` wiring (Task 5); GPU arm of `--modern-index-compare` (Task 6).

All tests live inline in the crate they exercise (`#[cfg(test)] mod tests` in `modern_gpu.rs`), matching the existing pattern.

---

## Task 1: Refactor GPU index + sprite renderers to per-frame cells

The renderers currently build their index atlas at `new()` from a fixed `cells` slice. The live path needs per-frame cells, so move the atlas build into `render()` while keeping the pipeline persistent.

**Files:**
- Modify: `crates/renderer/src/modern_gpu.rs` (`ModernGpuIndexRenderer::new`/`render` ~ lines 350–650; `ModernGpuSpriteRenderer::new`/`render` ~ lines 686–990; tests `modern_gpu_indexed_matches_software` ~1262, `gpu_bg_then_sprites` ~1373 and its callers ~1485+)
- Test: same file, existing tests updated.

**Interfaces:**
- Produces:
  - `ModernGpuIndexRenderer::new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self` (no `cells`).
  - `ModernGpuIndexRenderer::render(&self, device: &wgpu::Device, queue: &wgpu::Queue, cells: &[ModernIndexTile], frame: &ModernFrame, output_view: &wgpu::TextureView)`.
  - `ModernGpuSpriteRenderer::new(device, queue, format) -> Self` and `ModernGpuSpriteRenderer::render(&self, device, queue, cells: &[ModernIndexTile], frame: &ModernFrame, output_view: &wgpu::TextureView)`.
- The `cell_count` field is removed; the `inst.cell_id >= cell_count` guard becomes `inst.cell_id as usize >= cells.len()`.

- [ ] **Step 1: Update the existing index test to the new signatures (make it the failing test)**

In `modern_gpu_indexed_matches_software` (~line 1295 and ~1320), change construction + render:

```rust
let renderer = ModernGpuIndexRenderer::new(&device, &queue, wgpu::TextureFormat::Rgba8Unorm);
// ...
renderer.render(&device, &queue, &cells, &frame, &view);
```

In `gpu_bg_then_sprites` (~lines 1382–1408), change to:

```rust
let bg = ModernGpuIndexRenderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm);
let spr = ModernGpuSpriteRenderer::new(device, queue, wgpu::TextureFormat::Rgba8Unorm);
// ... target/view unchanged ...
bg.render(device, queue, bg_cells, frame, &view);
spr.render(device, queue, sprite_cells, frame, &view);
```

- [ ] **Step 2: Run tests to verify they fail to compile**

Run: `cargo test -p renderer modern_gpu 2>&1 | tail -20`
Expected: compile error — `new` takes 4 args / `render` takes 4 args (signature mismatch).

- [ ] **Step 3: Move the atlas build from `new` into `render` (index renderer)**

In `ModernGpuIndexRenderer::new`: remove the `cells: &[ModernIndexTile]` parameter and the entire index-atlas texture creation/upload block (the `grid_rows`/`tex_width`/`tex_height`, the `data` layout loop, `create_texture`, `write_texture`, and the stored `index_atlas_texture`/`index_atlas_view`/`cell_count` fields). Keep the pipeline + bind-group layout creation. Store only the pipeline + `bind_group_layout` (the layout is needed to rebuild the bind group per frame).

Add a private helper on the impl:

```rust
/// Build + upload the R8Uint index atlas for this frame's cells. Returns the
/// texture (kept alive by the caller for the duration of the render) and its view.
fn upload_index_atlas(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cells: &[ModernIndexTile],
) -> (wgpu::Texture, wgpu::TextureView) {
    let cell_count = cells.len() as u32;
    let grid_rows = cell_count.div_ceil(INDEX_GRID_COLS).max(1);
    let tex_width = INDEX_GRID_COLS * 8;
    let tex_height = grid_rows * 8;
    let mut data = vec![0u8; (tex_width * tex_height) as usize];
    for cell in cells {
        let col = cell.id % INDEX_GRID_COLS;
        let row = cell.id / INDEX_GRID_COLS;
        let ox = col * 8;
        let oy = row * 8;
        for ly in 0..8u32 {
            for lx in 0..8u32 {
                let px = (oy + ly) * tex_width + (ox + lx);
                data[px as usize] = cell.indices[(ly * 8 + lx) as usize];
            }
        }
    }
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("modern_index_atlas"),
        size: wgpu::Extent3d { width: tex_width, height: tex_height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::R8Uint,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(tex_width), rows_per_image: Some(tex_height) },
        wgpu::Extent3d { width: tex_width, height: tex_height, depth_or_array_layers: 1 },
    );
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}
```

In `render`: add `cells: &[ModernIndexTile]` as the 3rd parameter; at the top call `let (_atlas_tex, atlas_view) = Self::upload_index_atlas(device, queue, cells);` and bind `&atlas_view` where the code currently binds `&self.index_atlas_view`. Replace the `inst.cell_id >= self.cell_count` guard with `inst.cell_id as usize >= cells.len()`. Keep `_atlas_tex` in scope until after `queue.submit(...)` so the texture isn't dropped mid-encode.

- [ ] **Step 4: Apply the identical refactor to `ModernGpuSpriteRenderer`**

Same change: drop `cells`/`cell_count` from `new`; add a `upload_index_atlas`-equivalent (identical body — factor a shared free function `fn build_index_atlas(device, queue, cells) -> (wgpu::Texture, wgpu::TextureView)` at module scope and call it from both renderers to stay DRY); `render` gains `cells: &[ModernIndexTile]` and uses `cells.len()` for the guard at ~line 867.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p renderer modern_gpu 2>&1 | tail -20`
Expected: PASS — `modern_gpu_indexed_matches_software`, `gpu_bg_then_sprites`-backed sprite tests, and the untouched `ModernGpuRenderer` atlas tests all green.

- [ ] **Step 6: Commit**

```bash
git add crates/renderer/src/modern_gpu.rs
git commit --no-verify -m "refactor(renderer): GPU index+sprite renderers take per-frame cells

Move the R8Uint index-atlas build out of new() into render() (persistent
pipeline, per-frame atlas upload) so the live path can feed per-frame cells.
Existing GPU==CPU tests updated to the new signatures.

Claude-Session: https://claude.ai/code/session_01R7ZVWfMDiByS4aBvrXNFTk"
```

---

## Task 2: `ModernGpuCompositor` + `ModernGpuHeadless` + parity unit test

Wrap the two renderers into one compositor (BG-then-OBJ into a caller view) and a headless helper that owns a device + offscreen target and returns readback RGBA (for the unit test and, later, the compare harness).

**Files:**
- Modify: `crates/renderer/src/modern_gpu.rs` (add types near the bottom, before `#[cfg(test)]`; add one test inside `mod tests`).
- Modify: `crates/renderer/src/lib.rs` (re-export `ModernGpuCompositor`, `ModernGpuHeadless`).

**Interfaces:**
- Consumes (Task 1): `ModernGpuIndexRenderer::new(device, queue, format)`, `.render(device, queue, cells, frame, view)`; same for sprite renderer.
- Produces:
  - `pub struct ModernGpuCompositor` with `pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self` and `pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, frame: &ModernFrame, bg_cells: &[ModernIndexTile], sprite_cells: &[ModernIndexTile], output_view: &wgpu::TextureView)`.
  - `pub struct ModernGpuHeadless` with `pub fn new() -> Self` and `pub fn render_rgba(&self, frame: &ModernFrame, bg_cells: &[ModernIndexTile], sprite_cells: &[ModernIndexTile]) -> Vec<u8>` (256×224×4 RGBA, `Rgba8Unorm` byte order).

- [ ] **Step 1: Write the failing parity test**

Add inside `mod tests` in `modern_gpu.rs`:

```rust
#[test]
fn modern_gpu_compositor_matches_simple_zorder_software() {
    use crate::modern_frame::{ModernBgLayer, ModernIndexTileInstance};
    use crate::modern_index_atlas::ModernIndexTile;
    use crate::modern_hd_overrides::NO_SOURCE_KEY;
    use crate::modern_software::{draw_modern_sprites_indexed, render_modern_frame_software_indexed};

    // Two BG cells + two sprite cells (one h/v-flipped sprite instance).
    let mut a = [0u8; 64]; a[0] = 1; a[9] = 2;              // (0,0)=1, (1,1)=2
    let mut b = [0u8; 64]; b[63] = 3;                        // (7,7)=3
    let bg_cells = vec![
        ModernIndexTile { id: 0, indices: a, source_key: NO_SOURCE_KEY, hflip: false, vflip: false },
        ModernIndexTile { id: 1, indices: b, source_key: NO_SOURCE_KEY, hflip: false, vflip: false },
    ];
    let mut s0 = [0u8; 64]; s0[0] = 4;
    let mut s1 = [0u8; 64]; s1[7] = 5;                       // top-right pixel, to exercise hflip
    let sprite_cells = vec![
        ModernIndexTile { id: 0, indices: s0, source_key: NO_SOURCE_KEY, hflip: false, vflip: false },
        ModernIndexTile { id: 1, indices: s1, source_key: NO_SOURCE_KEY, hflip: false, vflip: false },
    ];

    let mut frame = ModernFrame::empty();
    frame.backdrop_color_rgba = [0, 0, 0, 0xff];
    // BG palette 3, OBJ palette 1 (OBJ CGRAM half starts at 0x80).
    frame.cgram_rgba[3 * 16 + 1] = [10, 20, 30, 0xff];
    frame.cgram_rgba[3 * 16 + 2] = [40, 50, 60, 0xff];
    frame.cgram_rgba[3 * 16 + 3] = [70, 80, 90, 0xff];
    frame.cgram_rgba[0x80 + 1 * 16 + 4] = [100, 110, 120, 0xff];
    frame.cgram_rgba[0x80 + 1 * 16 + 5] = [130, 140, 150, 0xff];

    let mut layer = ModernBgLayer::new(0);
    layer.enabled_main = true;
    layer.index_tiles.push(ModernIndexTileInstance { cell_id: 0, screen_x: 0, screen_y: 0, palette: 3, hflip: false, vflip: false, priority: false });
    layer.index_tiles.push(ModernIndexTileInstance { cell_id: 1, screen_x: 16, screen_y: 8, palette: 3, hflip: false, vflip: false, priority: false });
    frame.bg_layers[0] = layer;

    frame.index_sprites.push(crate::modern_frame::ModernIndexSpriteInstance { cell_id: 0, screen_x: 40, screen_y: 40, palette: 1, priority: 0, hflip: false, vflip: false, row_mask: 0xff });
    frame.index_sprites.push(crate::modern_frame::ModernIndexSpriteInstance { cell_id: 1, screen_x: 48, screen_y: 40, palette: 1, priority: 0, hflip: true, vflip: true, row_mask: 0xff });

    let gpu = ModernGpuHeadless::new().render_rgba(&frame, &bg_cells, &sprite_cells);

    let mut sw = render_modern_frame_software_indexed(&frame, &bg_cells);
    draw_modern_sprites_indexed(&mut sw, &frame, &sprite_cells);

    assert_eq!(gpu.len(), sw.len());
    assert_eq!(gpu, sw, "GPU compositor must match simple z-order CPU reference");
}
```

The sprite instance type is `ModernIndexSpriteInstance` with fields `cell_id: u32, screen_x: i16, screen_y: i16, palette: u8, priority: u8, hflip: bool, vflip: bool, row_mask: u8` (confirmed in `crates/renderer/src/modern_frame.rs:170`). `row_mask = 0xff` = all rows visible.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p renderer modern_gpu_compositor_matches 2>&1 | tail -20`
Expected: FAIL — `ModernGpuCompositor`/`ModernGpuHeadless` not found.

- [ ] **Step 3: Implement `ModernGpuCompositor`**

Add near the bottom of `modern_gpu.rs` (before `#[cfg(test)]`):

```rust
/// Simple-z-order GPU compositor over the PNG index atlas: draws each enabled BG
/// layer's index tiles (in layer order) then the OBJ sprites on top, into a caller
/// `Rgba8Unorm` view. Persistent pipelines; per-frame atlas/CGRAM/instance uploads.
/// NOTE: simple z-order only — Mode-1 BG/OBJ priority interleave, color math,
/// windows, HDMA scroll, mosaic, and the OBJ budget are later sub-projects.
pub struct ModernGpuCompositor {
    bg: ModernGpuIndexRenderer,
    sprites: ModernGpuSpriteRenderer,
}

impl ModernGpuCompositor {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        Self {
            bg: ModernGpuIndexRenderer::new(device, queue, format),
            sprites: ModernGpuSpriteRenderer::new(device, queue, format),
        }
    }

    /// Draw BG (clears the target to backdrop) then OBJ (loads on top) into `output_view`.
    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
        output_view: &wgpu::TextureView,
    ) {
        self.bg.render(device, queue, bg_cells, frame, output_view);
        self.sprites.render(device, queue, sprite_cells, frame, output_view);
    }
}
```

Confirm the BG renderer's render pass uses `LoadOp::Clear(backdrop)` and the sprite renderer uses `LoadOp::Load` (from Task 1 they are unchanged from the originals, which already do this — the sprite renderer early-returns when `instance_count == 0` "leave the BG render untouched"). If the BG renderer does not clear to `frame.backdrop_color_rgba`, that is a pre-existing behavior the CPU reference also encodes — do not change it here; the parity test will catch a mismatch.

- [ ] **Step 4: Implement `ModernGpuHeadless`**

Add below the compositor:

```rust
/// Owns a headless wgpu device + a 256x224 offscreen Rgba8Unorm target + the
/// compositor. `render_rgba` renders one frame and reads it back. Construct ONCE
/// and reuse (device creation is expensive).
pub struct ModernGpuHeadless {
    device: wgpu::Device,
    queue: wgpu::Queue,
    compositor: ModernGpuCompositor,
    target: wgpu::Texture,
    view: wgpu::TextureView,
}

impl ModernGpuHeadless {
    pub fn new() -> Self {
        let instance = crate::create_wgpu_instance();
        let (_adapter, device, queue) =
            pollster::block_on(crate::create_device_queue(&instance, None));
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let compositor = ModernGpuCompositor::new(&device, &queue, format);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("modern_gpu_headless_target"),
            size: wgpu::Extent3d { width: 256, height: 224, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&wgpu::TextureViewDescriptor::default());
        Self { device, queue, compositor, target, view }
    }

    pub fn render_rgba(
        &self,
        frame: &ModernFrame,
        bg_cells: &[ModernIndexTile],
        sprite_cells: &[ModernIndexTile],
    ) -> Vec<u8> {
        self.compositor.render(&self.device, &self.queue, frame, bg_cells, sprite_cells, &self.view);
        let (w, h) = (256u32, 224u32);
        let bytes_per_row = w * 4;
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("modern_gpu_headless_readback"),
            size: (bytes_per_row * h) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &self.target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo { buffer: &readback, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bytes_per_row), rows_per_image: None } },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        self.queue.submit([encoder.finish()]);
        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::wait_indefinitely()).expect("GPU poll failed during readback");
        let mapped = slice.get_mapped_range();
        let out = mapped.to_vec();
        drop(mapped);
        readback.unmap();
        out
    }
}
```

Because `create_wgpu_instance`/`create_device_queue` are private free functions in `lib.rs`, and `modern_gpu.rs` is a module of the same crate, reference them as `crate::create_wgpu_instance` / `crate::create_device_queue`. Confirm `pollster` is already a dependency (`rg -n "^pollster" crates/renderer/Cargo.toml`; the existing tests use `pollster::block_on`, so it is).

- [ ] **Step 5: Re-export the new types**

In `crates/renderer/src/lib.rs`, find the existing `pub use ... modern_gpu ...` (or add one) and export:

```rust
pub use modern_gpu::{ModernGpuCompositor, ModernGpuHeadless};
```

Run `rg -n "pub use.*modern_gpu|mod modern_gpu" crates/renderer/src/lib.rs` to place it beside the existing module declaration/exports.

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test -p renderer modern_gpu 2>&1 | tail -20`
Expected: PASS — `modern_gpu_compositor_matches_simple_zorder_software` green, all prior `modern_gpu` tests still green.

- [ ] **Step 7: Commit**

```bash
git add crates/renderer/src/modern_gpu.rs crates/renderer/src/lib.rs
git commit --no-verify -m "feat(renderer): ModernGpuCompositor + headless readback (simple z-order)

BG-then-OBJ GPU compositor over the PNG index atlas, byte-identical to the
simple z-order CPU reference (render_modern_frame_software_indexed +
draw_modern_sprites_indexed). ModernGpuHeadless owns a device + offscreen
target for readback (unit test + future compare harness).

Claude-Session: https://claude.ai/code/session_01R7ZVWfMDiByS4aBvrXNFTk"
```

---

## Task 3: `FrameRenderer::present_modern_gpu` (live GPU-only present)

Render the compositor into the renderer's existing 256×224 `game_texture` via a GPU texture-copy (no readback), then blit with the standard presentation path.

**Files:**
- Modify: `crates/renderer/src/lib.rs` (`FrameRenderer` struct + impl; `present_modern_rgba`/`render` are the models, ~lines 1958–2456).

**Interfaces:**
- Consumes (Task 2): `ModernGpuCompositor::new`, `.render`.
- Produces: `FrameRenderer::present_modern_gpu(&mut self, frame: &ModernFrame, bg_cells: &[ModernIndexTile], sprite_cells: &[ModernIndexTile]) -> Result<(), RenderError>`.

- [ ] **Step 1: Add compositor + offscreen fields to `FrameRenderer`**

Add fields to the struct (~line 1987, after `hd_scale`):

```rust
    /// Lazily built on first `present_modern_gpu` call (assets-anim-gpu mode).
    modern_gpu: Option<ModernGpuCompositor>,
    /// Offscreen Rgba8Unorm 256x224 target the compositor renders into before
    /// it is GPU-copied into `game_texture` and blit by `render()`.
    modern_gpu_target: Option<(wgpu::Texture, wgpu::TextureView)>,
```

Initialize both to `None` in `FrameRenderer::new` (find the struct-literal return near the end of `new`, add `modern_gpu: None, modern_gpu_target: None,`).

- [ ] **Step 2: Implement `present_modern_gpu`**

Add to the `impl FrameRenderer` block (near `present_modern_rgba`, ~line 2380):

```rust
    /// Live GPU present of the PNG-atlas path (ZELDA3_RENDERER=assets-anim-gpu).
    /// Renders the compositor into an offscreen 256x224 target, GPU-copies it into
    /// `game_texture`, then blits via the standard presentation `render()`. No CPU
    /// readback. Byte-identical to the simple z-order CPU path (Foundation scope).
    pub fn present_modern_gpu(
        &mut self,
        frame: &modern_frame::ModernFrame,
        bg_cells: &[modern_index_atlas::ModernIndexTile],
        sprite_cells: &[modern_index_atlas::ModernIndexTile],
    ) -> Result<(), RenderError> {
        let format = wgpu::TextureFormat::Rgba8Unorm;
        if self.modern_gpu.is_none() {
            self.modern_gpu = Some(ModernGpuCompositor::new(&self.device, &self.queue, format));
        }
        if self.modern_gpu_target.is_none() {
            let target = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("modern_gpu_live_target"),
                size: wgpu::Extent3d { width: 256, height: 224, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());
            self.modern_gpu_target = Some((target, view));
        }

        // Ensure the blit source texture is native 256x224 (classic size).
        self.game_texture.ensure_size(
            &self.device,
            &self.bind_group_layout,
            &self.presentation_buf,
            self.presentation_params.presentation,
            256,
            224,
        );

        let compositor = self.modern_gpu.as_ref().expect("compositor built above");
        let (target_tex, target_view) = self.modern_gpu_target.as_ref().expect("target built above");
        compositor.render(&self.device, &self.queue, frame, bg_cells, sprite_cells, target_view);

        // GPU copy offscreen -> game_texture (both Rgba8Unorm 256x224).
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("modern_gpu_copy_to_game_texture"),
        });
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo { texture: target_tex, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyTextureInfo { texture: &self.game_texture.texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::Extent3d { width: 256, height: 224, depth_or_array_layers: 1 },
        );
        self.queue.submit([encoder.finish()]);

        self.render()
    }
```

Note: `game_texture` is `COPY_DST` already, so `copy_texture_to_texture` into it is valid. Use `modern_frame` / `modern_index_atlas` module paths that match the crate (run `rg -n "pub mod modern_frame|pub mod modern_index_atlas|mod modern_frame|mod modern_index_atlas" crates/renderer/src/lib.rs` and match the existing path; if they're re-exported at crate root, use `crate::ModernFrame` / `crate::modern_index_atlas::ModernIndexTile` consistently with how `present_modern_rgba`'s neighbors refer to them).

- [ ] **Step 3: Build to verify it compiles**

Run: `cargo build -p renderer 2>&1 | tail -20`
Expected: compiles clean (no test yet — this path needs a surface; it's exercised by the Task 5 smoke + Task 6 compare).

- [ ] **Step 4: Confirm no existing tests broke**

Run: `cargo test -p renderer 2>&1 | tail -15`
Expected: all green (the new fields default to `None`, no behavior change to existing paths).

- [ ] **Step 5: Commit**

```bash
git add crates/renderer/src/lib.rs
git commit --no-verify -m "feat(renderer): FrameRenderer::present_modern_gpu (GPU-only live present)

Renders ModernGpuCompositor into an offscreen 256x224 target, GPU-copies it
into game_texture, then blits via the standard presentation path. No readback.

Claude-Session: https://claude.ai/code/session_01R7ZVWfMDiByS4aBvrXNFTk"
```

---

## Task 4: `NativeFrontend::present_modern_gpu` passthrough

**Files:**
- Modify: `crates/platform/src/lib.rs` (near `present_modern_rgba`, ~line 269).

**Interfaces:**
- Consumes (Task 3): `FrameRenderer::present_modern_gpu(frame, bg_cells, sprite_cells) -> Result<(), RenderError>`.
- Produces: `NativeFrontend::present_modern_gpu(&mut self, frame: &renderer::ModernFrame, bg_cells: &[renderer::modern_index_atlas::ModernIndexTile], sprite_cells: &[renderer::modern_index_atlas::ModernIndexTile])`.

- [ ] **Step 1: Add the passthrough method**

Mirror `present_modern_rgba` (lines 269–284):

```rust
    /// Live GPU present of the PNG-atlas path (assets-anim-gpu). The caller builds
    /// `(ModernFrame, bg_cells, sprite_cells)` via the sources extractor (it needs
    /// the CHR-source table this crate can't reach), same as `present_modern_rgba`.
    pub fn present_modern_gpu(
        &mut self,
        frame: &renderer::ModernFrame,
        bg_cells: &[renderer::modern_index_atlas::ModernIndexTile],
        sprite_cells: &[renderer::modern_index_atlas::ModernIndexTile],
    ) {
        if let Some(renderer) = &mut self.handler.renderer {
            let result = renderer.present_modern_gpu(frame, bg_cells, sprite_cells);
            match result {
                Ok(()) => {}
                Err(RenderError::SurfaceReconfigureNeeded) => {
                    if let Some(window) = &self.handler.window {
                        renderer.resize(window.inner_size());
                    }
                }
                Err(RenderError::SurfaceSkipped) => {}
                Err(RenderError::Fatal(e)) => eprintln!("render error: {e}"),
            }
        }
        self.sleep_after_present();
    }
```

Confirm the exact public paths: run `rg -n "pub use|ModernFrame|modern_index_atlas" crates/renderer/src/lib.rs | rg "pub"` and match how `platform` already names renderer types (e.g. the existing `GpuFrame`/`RendererMode` imports at the top of `platform/src/lib.rs`). Add any missing `use renderer::...` there. Ensure `renderer::ModernFrame` and `renderer::modern_index_atlas::ModernIndexTile` are re-exported from the renderer crate root; if not, add `pub use modern_frame::ModernFrame;` and `pub use modern_index_atlas;` (or `pub mod modern_index_atlas;`) in `crates/renderer/src/lib.rs` in this step.

- [ ] **Step 2: Build to verify it compiles**

Run: `cargo build -p platform 2>&1 | tail -20`
Expected: compiles clean.

- [ ] **Step 3: Commit**

```bash
git add crates/platform/src/lib.rs crates/renderer/src/lib.rs
git commit --no-verify -m "feat(platform): NativeFrontend::present_modern_gpu passthrough

Claude-Session: https://claude.ai/code/session_01R7ZVWfMDiByS4aBvrXNFTk"
```

---

## Task 5: `assets-anim-gpu` selection + `GpuPlayRenderer` wiring

Recognize `ZELDA3_RENDERER=assets-anim-gpu`, and route the live present through `present_modern_gpu` instead of the CPU compositor when selected.

**Files:**
- Modify: `zelda3-bin/src/main.rs` (`GpuPlayRenderer` struct + `new` ~1405–1433; `present_frame` ~1441–1495; `run_play_with_state` renderer-mode selection ~1543–1549; `effective_play_renderer` ~1406).

**Interfaces:**
- Consumes (Task 4): `frontend.present_modern_gpu(&modern, &bg_cells, &sprite_cells)`.
- Produces: a working `ZELDA3_RENDERER=assets-anim-gpu` interactive mode.

- [ ] **Step 1: Add an `atlas_gpu` flag to `GpuPlayRenderer`**

In the struct (line ~1405) add `atlas_gpu: bool,`. In `GpuPlayRenderer::new` (line ~1411), compute it and keep the existing atlas-load gate loading for BOTH `assets-anim` and `assets-anim-gpu`:

```rust
        let mode = effective_play_renderer();
        let assets_anim_mode = mode == "assets-anim" || mode == "assets-anim-gpu";
        let atlas_gpu = mode == "assets-anim-gpu";
        // ... existing source_atlas load using assets_anim_mode ...
        Self { source_atlas, hd_overrides, atlas_gpu }
```

(Adjust the returned struct literal at ~line 1429 to include `atlas_gpu`.)

- [ ] **Step 2: Route present through the GPU path when `atlas_gpu`**

In `present_frame` (the `if let Some(atlas) = self.source_atlas.as_ref().filter(|_| gpu_frame.mode != 7)` block, ~line 1457), after building `modern`, `bg_cells`, `sprite_cells` (the code that produces `(mut modern, bg_cells)` and `(sprite_cells, sprites)` then sets `modern.index_sprites = sprites`), branch before the CPU render:

```rust
            if self.atlas_gpu {
                frontend.present_modern_gpu(&modern, &bg_cells, &sprite_cells);
                return;
            }
            // ... existing CPU path: HdOverrideCtx + render_modern_frame_full_scaled + present_modern_rgba ...
```

Leave the Mode-7 / no-atlas fallthrough (`present_gpu_frame_with_context`) unchanged — that is the SP1 fallback.

- [ ] **Step 3: Recognize `assets-anim-gpu` in renderer-mode selection**

In `run_play_with_state` (~line 1543), the mapping already treats non-`assets-anim` strings via `RendererMode::parse`. Update so `assets-anim-gpu` also maps to `Modern` (needed for the Mode-7 fallback path):

```rust
    let renderer_env = effective_play_renderer();
    let renderer_mode = if renderer_env == "assets-anim" || renderer_env == "assets-anim-gpu" {
        renderer::RendererMode::Modern
    } else {
        renderer::RendererMode::parse(Some(renderer_env.as_str()))
    };
```

- [ ] **Step 4: Build**

Run: `cargo build --profile parity -p zelda3-bin 2>&1 | tail -8`
Expected: `Finished` clean.

- [ ] **Step 5: Headless smoke — GPU path boots and stays up**

Run:
```bash
env SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy SDL_RENDER_DRIVER=software \
  ZELDA3_RENDERER=assets-anim-gpu ZELDA3_SKIP_HOST_MENU=1 \
  target/parity/zelda3 saves/zelda3.sfc > /tmp/gpu_boot.log 2>&1 &
PID=$!; sleep 5; kill $PID 2>/dev/null; wait $PID 2>/dev/null
rg -i "panic|error|atlas.*failed|assertion" /tmp/gpu_boot.log || echo "OK: assets-anim-gpu booted with no error"
```
Expected: `OK: assets-anim-gpu booted with no error` (no panic, no atlas-load failure). If the dummy SDL backend has no usable GPU adapter in this environment, note that and rely on Task 6's headless-wgpu compare for functional proof instead.

- [ ] **Step 6: Regression — default and classic unchanged**

Run:
```bash
env SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy SDL_RENDER_DRIVER=software \
  ZELDA3_SKIP_HOST_MENU=1 target/parity/zelda3 saves/zelda3.sfc > /tmp/def_boot.log 2>&1 &
PID=$!; sleep 4; kill $PID 2>/dev/null; wait $PID 2>/dev/null
rg -i "panic|error" /tmp/def_boot.log || echo "OK: default (assets-anim CPU) unchanged"
```
Expected: `OK: default (assets-anim CPU) unchanged`.

- [ ] **Step 7: Commit**

```bash
git add zelda3-bin/src/main.rs
git commit --no-verify -m "feat(zelda3-bin): assets-anim-gpu opt-in routes live present to GPU

ZELDA3_RENDERER=assets-anim-gpu builds (ModernFrame, bg_cells, sprite_cells)
exactly as the CPU path and presents via frontend.present_modern_gpu. Mode-7
and atlas-miss keep the existing fallback (SP1). Default assets-anim (CPU) and
classic unchanged.

Claude-Session: https://claude.ai/code/session_01R7ZVWfMDiByS4aBvrXNFTk"
```

---

## Task 6: GPU arm of `--modern-index-compare` (the verification gate)

Add a GPU comparison arm so the existing compare harness can print `mismatch_px … via=gpu` — the per-frame GPU-vs-classic gate for this and every later sub-project.

**Files:**
- Modify: `zelda3-bin/src/main.rs` (`run_replay_save` compare block ~5414–5527, plus the setup region ~3641–3660 where `assets_anim_mode`/`source_atlas` are resolved).

**Interfaces:**
- Consumes (Task 2): `renderer::ModernGpuHeadless::new()`, `.render_rgba(&modern, &bg_cells, &sprite_cells)`.
- Consumes existing: `extract_modern_frame_from_sources`, `extract_modern_sprites_from_sources`, `offscreen.render_gpu_frame(&gpu_frame)`.

- [ ] **Step 1: Detect the GPU compare arm + build the headless renderer once**

In `run_replay_save`, near where `assets_anim_mode` is resolved for the compare harness (~line 3645), add:

```rust
    let atlas_gpu_compare = std::env::var("ZELDA3_RENDERER").map(|v| v == "assets-anim-gpu").unwrap_or(false);
    // Build the headless GPU renderer once (device creation is expensive) only when needed.
    let mut modern_gpu_headless: Option<renderer::ModernGpuHeadless> =
        if modern_index_compare != 0 && atlas_gpu_compare {
            Some(renderer::ModernGpuHeadless::new())
        } else {
            None
        };
```

Also make the `source_atlas` load gate include the GPU mode (~line 3648): change the condition to
`modern_index_compare != 0 && (assets_anim_mode || atlas_gpu_compare)` so the atlas loads for the GPU compare too (the extractor needs it).

- [ ] **Step 2: Add the GPU branch in the compare block**

In the compare block (~line 5450), the code chooses `(modern_rgba, via)` among mode7 / sources / vram. Add a GPU arm as the **first** choice when `modern_gpu_headless` is `Some` and the frame is not Mode 7:

```rust
                let (modern_rgba, via) = if let (Some(headless), false) =
                    (modern_gpu_headless.as_ref(), gpu_frame.mode == 7)
                {
                    let atlas = source_atlas.as_ref().expect("atlas loaded for gpu compare");
                    let src_slice: Vec<(u8, u16, u16)> = game
                        .vram_chr_source().as_slice().iter()
                        .map(|s| (s.kind, s.pack, s.tile_off)).collect();
                    let (mut modern, bg_cells) =
                        renderer::modern_extract::extract_modern_frame_from_sources(&gpu_frame, &src_slice[..], atlas);
                    let (sprite_cells, sprites) =
                        renderer::modern_extract::extract_modern_sprites_from_sources(&gpu_frame, &src_slice[..], atlas);
                    modern.index_sprites = sprites;
                    (headless.render_rgba(&modern, &bg_cells, &sprite_cells), "gpu")
                } else if gpu_frame.mode == 7 {
                    // ... existing mode7 arm ...
```

Keep the rest of the `else if … source_atlas … else …` chain exactly as-is (it handles the non-GPU modes). `let _ = &mut modern_gpu_headless;` if the borrow checker needs the binding kept alive.

- [ ] **Step 3: Build**

Run: `cargo build --profile parity -p zelda3-bin 2>&1 | tail -8`
Expected: `Finished` clean.

- [ ] **Step 4: Run the GPU compare over a chunk of the route**

Run (7 timing hacks as `HACKS`, see `CLAUDE.md`):
```bash
env "${HACKS[@]}" ZELDA3_RENDERER=assets-anim-gpu ZELDA3_COMPARE_ALL_MODULES=1 \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 12000 \
  --modern-index-compare 2000 2>&1 | rg "modern_index_compare"
```
Expected: lines with `via=gpu`. `mismatch_px=0` on effect-free frames; some frames `>0` (priority-interleave / color-math / window / HDMA — the SP2–6 worklist). Record which frames/modes are non-zero into the commit message or a scratch note.

- [ ] **Step 5: Verify the replay/parity default is untouched**

Run (note: `ZELDA3_RENDERER` UNSET):
```bash
env "${HACKS[@]}" ZELDA3_COMPARE_ALL_MODULES=1 \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 6000 \
  --modern-index-compare 3000 2>&1 | rg "modern_index_compare"
```
Expected: `via=vram` (NOT `via=gpu`) — the harness still defaults to classic/vram; parity gates unaffected.

- [ ] **Step 6: Commit**

```bash
git add zelda3-bin/src/main.rs
git commit --no-verify -m "feat(zelda3-bin): GPU arm of --modern-index-compare (assets-anim-gpu)

Renders classic + GPU-atlas (ModernGpuHeadless readback) from identical state
and prints mismatch_px via=gpu. mismatch_px=0 on effect-free frames; non-zero
frames are the SP2-6 worklist. Replay default (unset) still via=vram.

Claude-Session: https://claude.ai/code/session_01R7ZVWfMDiByS4aBvrXNFTk"
```

---

## Self-Review notes (for the executor)

- **Spec coverage:** Task 1–2 = `ModernGpuCompositor` + persistent-pipeline refactor + readback helper + parity unit test; Task 3 = `FrameRenderer::present_modern_gpu`; Task 4 = frontend passthrough; Task 5 = `GpuPlayRenderer` wiring + `assets-anim-gpu` selection; Task 6 = GPU `--modern-index-compare` arm + verification (incl. the "default untouched" check). Error handling reuses the existing `present_modern_rgba` match (Task 3/4). Non-goals (priority/color-math/HDMA/mosaic/OBJ-budget/Mode-7/HD overrides) are explicitly NOT implemented.
- **Type consistency:** `ModernGpuIndexRenderer::new(device, queue, format)` + `.render(device, queue, cells, frame, view)` are used identically in Tasks 1, 2. `ModernGpuHeadless::render_rgba(frame, bg_cells, sprite_cells) -> Vec<u8>` is used in Task 2 (test) and Task 6 (harness). `present_modern_gpu(frame, bg_cells, sprite_cells)` signature matches across Tasks 3, 4, 5.
- **Verify-before-code anchors:** several steps say "run `rg` to confirm the exact field/module name" (e.g. `ModernIndexSprite` fields, `modern_frame`/`modern_index_atlas` module paths, `pollster` dependency). These are confirmations of names that already exist in the tree, not open design choices — the surrounding code shows the shape.
