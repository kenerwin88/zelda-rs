# GPU PNG-Atlas Renderer — Sub-project 1: Foundation (Design)

**Status:** Approved (design). Next: implementation plan.
**Date:** 2026-07-02

## Parent project

Render the game from the PNG source atlas (`developer_tilesets/assets_by_source.png`)
**on the GPU**, byte-identical to the classic renderer, with **no CPU-compositor
fallback** in the end state (Mode-1 *and* Mode-7 on the GPU). Today the byte-identical
PNG-atlas path (`ZELDA3_RENDERER=assets-anim`) runs on a **CPU** software compositor
(`render_modern_frame_full_scaled`). This project moves that drawing to the GPU for
performance while preserving exact parity with the classic renderer.

Agreed scope decisions:
- **Fidelity:** byte-identical to classic (same bar as the CPU `assets-anim` path — `mismatch_px=0`).
- **Coverage:** everything on GPU (Mode-1 + Mode-7), no CPU fallback in the end state.
- **HD overrides:** out of scope (base drawing at native 1× only; GPU HD-override sampling is a separate later project).
- **Rollout:** opt-in `ZELDA3_RENDERER=assets-anim-gpu` now; flip the interactive default to GPU once proven `mismatch_px=0` route-wide.

Because the CPU compositor being replicated does per-layer priority passes, per-scanline
HDMA scroll, per-scanline main-screen TM gates, window masks, mosaic, main/sub SNES color
math (add/sub/half + fixed color), master brightness, the OBJ per-scanline budget, and a
separate Mode-7 affine model, the parent is **decomposed into sub-projects**, each its own
spec → plan → build cycle:

1. **Foundation (this spec)** — wire the existing GPU index+sprite renderers into the live
   present as opt-in `assets-anim-gpu`, plus a GPU-vs-classic verification harness.
2. Mode-1 BG↔OBJ priority interleave + per-scanline main-screen TM + window masks.
3. SNES color math (main/sub combine, add/sub/half, fixed color, per-layer enable, windows) + master brightness.
4. Per-scanline HDMA BG scroll + mosaic.
5. OBJ per-scanline range/time-over budget.
6. Mode-7 affine GPU pipeline; remove the CPU fallback.
7. Flip interactive default to GPU once `mismatch_px=0` route-wide.

Architecture approach (chosen): **A — extend the existing instance renderers.**
`ModernGpuIndexRenderer` / `ModernGpuSpriteRenderer` (in `crates/renderer/src/modern_gpu.rs`)
already draw from the index atlas + live CGRAM on the GPU and already have GPU==CPU tests.
We grow this feature-by-feature, mirroring how the classic wgpu renderer already solved
priority + sprites. Rejected: (B) a monolithic compute shader — discards the existing
renderers + tests; (C) two-target hardware model — intermediate buffers must also carry
per-pixel "which layer won" state, making byte-identity fiddlier.

## Foundation — goal

`ZELDA3_RENDERER=assets-anim-gpu` renders the live interactive frame from the PNG atlas on
the GPU, end-to-end. It is **byte-identical to classic on effect-free frames** (simple
z-order matches). A GPU arm of `--modern-index-compare` reports `mismatch_px` per frame,
quantifying exactly which frames still differ — that measurement is the worklist for
sub-projects 2–6, not a Foundation failure.

## Existing building blocks (already in the repo, verified)

- `ModernGpuIndexRenderer::new(device, queue, cells: &[ModernIndexTile], format)` builds a
  static `R8Uint` index atlas and renders BG `index_tiles` colored via a `256×1` CGRAM
  texture. `ModernGpuSpriteRenderer` does the same for OAM `index_sprites` (flip + OBJ
  CGRAM half). Both consume a `&ModernFrame` and have tests asserting
  `gpu_rgba == render_modern_frame_software*` output.
- `extract_modern_frame_from_sources(gpu_frame, src_table, atlas) -> (ModernFrame, Vec<ModernIndexTile>)`
  and `extract_modern_sprites_from_sources(...) -> (Vec<ModernIndexTile>, sprites)` produce
  exactly the `(ModernFrame, bg_cells, sprite_cells)` triple the CPU path already feeds to
  `render_modern_frame_full`. **The GPU renderers take the same `&[ModernIndexTile]` cells.**
- `NativeFrontend::present_modern_rgba(rgba, w, h)` is the existing "caller composited the
  modern frame, blit it" entry; `FrameRenderer` (renderer crate) owns the wgpu
  device/queue/surface and the surface-error handling we reuse.

The one structural gap: the GPU renderers build their index atlas at `new()`, but the
extracted cells are **per-frame**. Foundation resolves this with a persistent-pipeline
wrapper that re-uploads per frame.

## Components

### 1. `ModernGpuCompositor` (new; `crates/renderer/src/modern_gpu.rs`)
Wraps the two existing GPU renderers but **splits persistent from per-frame state**:
- **Persistent:** render pipelines, bind-group layouts, sampler (built once).
- **Per-frame (re-uploaded each `render` call):** the `R8Uint` index atlas texture (from
  that frame's `bg_cells` / `sprite_cells`), the `256×1` CGRAM texture, and the instance
  buffers, with the bind group rebuilt to reference the fresh textures.
- One method:
  `render(&self, device, queue, frame: &ModernFrame, bg_cells: &[ModernIndexTile], sprite_cells: &[ModernIndexTile], output_view: &wgpu::TextureView)`.
- **Draw order (SP1):** the existing *simple* z-order — clear to backdrop, draw each enabled
  BG layer's `index_tiles` in layer order, then draw `index_sprites` on top. (Full Mode-1
  priority interleave is SP2.)

This is a **refactor** of the existing renderers (extract the atlas/CGRAM/instance upload out
of `new`/`render` into per-frame paths); the existing GPU==CPU unit tests must keep passing.

### 2. `FrameRenderer::present_modern_gpu(&ModernFrame, bg_cells, sprite_cells)` (renderer crate)
Owns an optionally-constructed `ModernGpuCompositor` (built lazily on first use, from the
renderer's own `device`/`queue`/surface `format`). Acquires the surface texture, renders the
compositor to its view, presents. Reuses `present_modern_rgba`'s surface-error handling
(`SurfaceReconfigureNeeded` → resize; `SurfaceSkipped` → skip; `Fatal` → log).

### 3. `NativeFrontend::present_modern_gpu(&ModernFrame, bg_cells, sprite_cells)` (platform crate)
Thin passthrough to `FrameRenderer::present_modern_gpu`, mirroring `present_modern_rgba`
(same `if let Some(renderer)` guard + error match).

### 4. `GpuPlayRenderer` wiring (zelda3-bin)
Add a mode flag (e.g. `atlas_gpu: bool`) resolved from `effective_play_renderer() == "assets-anim-gpu"`.
When set and `gpu_frame.mode != 7` and the atlas loaded, build `(ModernFrame, bg_cells,
sprite_cells)` exactly as the CPU branch does today, then call
`frontend.present_modern_gpu(&modern, &bg_cells, &sprite_cells)` **instead of**
`render_modern_frame_full_scaled` + `present_modern_rgba`. Mode-7 and atlas-miss keep the
existing fallback **in SP1** (SP6 removes it). The CPU `assets-anim` branch is unchanged.

### 5. Selection (`effective_play_renderer` / renderer-mode plumbing, zelda3-bin)
- `assets-anim-gpu` is recognized as a valid interactive renderer string.
- It maps to `RendererMode::Modern` (for the frontend's Mode-7 / fallback handling) and sets
  the `atlas_gpu` flag on `GpuPlayRenderer`.
- The interactive **default stays `assets-anim` (CPU)**; `classic` still opts into the wgpu PPU.
- The **replay/parity harness is untouched** — it reads `ZELDA3_RENDERER` through its own
  `assets_anim_mode` gate in `run_replay_save` and still defaults to classic; render-hash /
  fingerprint gates are unaffected (as verified for the CPU-default change).

## Data flow (per live frame)
```
GameState
  → gpu_frame_from_ppu(ppu, hdma_cgram, scanlines)                       (existing)
  → extract_modern_frame_from_sources / _sprites (needs vram_chr_source) (existing)
     ⇒ (ModernFrame{ index_tiles, index_sprites, cgram_rgba, … }, bg_cells, sprite_cells)
  → frontend.present_modern_gpu(&modern, &bg_cells, &sprite_cells)       (NEW passthrough)
  → FrameRenderer::present_modern_gpu                                    (NEW)
  → ModernGpuCompositor::render:                                         (NEW)
       re-upload index atlas (from cells) + CGRAM(256×1) + instances,
       clear→backdrop, draw BG layers (order), draw OBJ on top,
       present to surface.  (No readback on the live path.)
```

## Verification harness
- **Renderer-crate helper** `render_modern_frame_gpu_readback(device, queue, frame, bg_cells, sprite_cells) -> Vec<u8>`:
  renders `ModernGpuCompositor` to an **offscreen** `Rgba8Unorm` texture (256×224) and reads
  it back to a `Vec<u8>`, reusing the exact readback code the existing
  `modern_gpu_indexed_matches_software` test uses (create buffer, copy texture→buffer,
  `map_async`, poll, copy out, respecting `bytes_per_row` 256-alignment).
- **`--modern-index-compare` GPU arm** (zelda3-bin): when `assets-anim-gpu` is selected, the
  compare block creates **one** headless wgpu device up front, and per compared frame renders
  the classic RGBA (`offscreen.render_gpu_frame`) and the GPU-atlas RGBA
  (`render_modern_frame_gpu_readback`), diffs them the same way the CPU arm does, and prints
  `modern_index_compare frame=… mode=… ppumode=… mismatch_px=… via=gpu`.
- **Expected SP1 numbers:** `mismatch_px=0` on effect-free frames; `>0` on frames using
  priority-interleave / color-math / window / HDMA / mosaic. Recorded as the SP2–6 worklist.

## Error handling
- `ModernGpuCompositor` construction failure (or headless-device creation failure in the
  harness) is logged and falls back to the CPU path — never a hard crash (same posture as the
  existing atlas-load fallback).
- Live-surface errors reuse the existing `present_modern_rgba` error match.
- `assets-anim-gpu` with the atlas missing → same graceful message + CPU/classic fallback as
  `assets-anim`.

## Testing
- **Unit (headless wgpu, renderer crate):** drive `ModernGpuCompositor` on a synthetic frame
  with ≥2 BG cells, ≥2 OAM sprites (incl. an h/v-flipped sprite), and a non-trivial CGRAM;
  assert the compositor RGBA equals the **simple-z-order CPU reference** the existing GPU
  renderers already match — i.e. BG from `render_modern_frame_software_indexed(&frame, &bg_cells)`
  composed with sprites from `draw_modern_sprites_indexed(...)` in the same BG-then-OBJ order
  (model this on the existing combined BG+sprite readback test in `modern_gpu.rs`). This
  proves the persistent-wrapper refactor preserved byte-identity and that BG+OBJ compose into
  one target correctly. **Note:** this reference is the *simple* z-order path, **not**
  `render_modern_frame_full` (whose priority-interleave / color-math is SP2–SP3) — SP1 does
  not change what "correct" means for that simple path.
- **Unit:** the pre-existing `modern_gpu_*_matches_software` tests still pass (regression
  guard on the refactor).
- **Integration:** a short `--modern-index-compare` run (with `assets-anim-gpu`) over a small
  set of known effect-free frames asserts `via=gpu` and `mismatch_px=0`.
- **No-regression:** default `assets-anim` (CPU) and `classic` paths untouched; the full
  existing test suite stays green; `--modern-index-compare` with `ZELDA3_RENDERER` unset still
  reports `via=vram` (replay/parity harness unaffected).

## Explicit non-goals (deferred)
Mode-1 BG↔OBJ priority interleave (SP2); per-scanline main-screen TM + window masks (SP2);
SNES color math + master brightness (SP3); per-scanline HDMA BG scroll + mosaic (SP4); OBJ
per-scanline range/time-over budget (SP5); Mode-7 GPU pipeline + removing the CPU fallback
(SP6); flipping the interactive default to GPU (SP7); HD-override (detail-modulate / N×)
sampling on the GPU (separate project).

## Success criteria (Foundation)
1. `ZELDA3_RENDERER=assets-anim-gpu <rom>` runs interactively, drawing from the PNG atlas on
   the GPU, without crashing; Mode-7 / atlas-miss still render correctly via the SP1 fallback.
2. `ModernGpuCompositor` output is byte-identical to the simple-z-order CPU reference
   (`render_modern_frame_software_indexed` + `draw_modern_sprites_indexed`) on the unit
   fixture, and the pre-existing GPU==CPU tests still pass.
3. `--modern-index-compare` with `assets-anim-gpu` prints `via=gpu` and reports
   `mismatch_px=0` on effect-free frames; the non-zero frames are recorded as the SP2–6
   worklist.
4. Default (`assets-anim` CPU), `classic`, and the replay/parity harness are all unchanged.
