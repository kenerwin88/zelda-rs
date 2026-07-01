# Modern renderer HD N× output (Phase 2) — design

**Date:** 2026-07-01
**Status:** Design approved; ready for implementation planning.
**Predecessor:** `2026-07-01-modern-hd-source-key-overrides-design.md` (Phase 1 — the
source-keyed HD override kernel/store, shipped on `main`).

## Goal

Make the Phase 1 HD overrides **visible in-game at 60fps**: render the modern
(off-VRAM) software compositor at an integer scale factor **N**, so HD-override cells
show at full sub-pixel detail while the rest of the frame nearest-upscales — the classic
"HD texture pack while you play" look — and wire it into the live display path.

Grounding measurement: the full modern CPU compositor renders a native frame in
**0.939 ms** (`perf_render_modern_frame_full_from_vram`). The per-pixel work scales with
N²; extraction (per-tile) does not. Worst-case (all-scales) estimates: ~3.8 ms at 2×,
~8.5 ms at 3×, ~15 ms at 4× — all under the 16.6 ms/frame budget. So **CPU N× is viable
for live 60fps through ~4×**; no GPU renderer is built.

### Non-goals (Phase 2)
- No GPU modern renderer (the existing `ModernGpuRenderer` stays as-is; the live path
  is the CPU software compositor).
- No Mode-7 HD. Mode-7 (affine map) frames are nearest-upscaled to N×; they have no
  source-key BG overrides.
- No scale beyond 4× (CPU ceiling for 60fps).
- No increase in color depth: the compositor stays 5-bit-per-channel through
  `finalize_frame` (HD art is palette-quantized to 15-bit — the win is spatial detail,
  and colors already track the SNES palette). Keeping 5-bit is what makes N=1 identical
  to today.
- No change to the parity/compare/dump tooling, which keeps rendering at N=1.

## Background / current state

- The modern software compositor (`crates/renderer/src/modern_software.rs`) builds a
  native **256×224** `Screen` (a `Vec<[u8;3]>` of 5-bit `c5` colors, plus `real`/`bit`
  layer masks), composites Mode-1 z-order into it, and `finalize_frame` applies
  color-math + master-brightness per pixel using per-native-scanline window/scroll data,
  emitting native RGBA. `MODERN_FRAME_WIDTH`/`_HEIGHT` are compile-time 256/224.
- Phase 1 added source-keyed HD overrides: `render_modern_frame_full_with_overrides(
  frame, bg_cells, sprite_cells, ctx)` threads an `HdOverrideCtx` through the 4 resolve
  sites (`composite_index_tiles_c5`, `render_bg_layer_buf`, `render_bg_layer_torus`,
  `resolve_obj_layer`); each calls `resolve_pixel_color(base_index, cgram_idx, live_rgba,
  ov, reference, lx, ly)`, sampling `HdCell::sample_native(lx, ly)` at native 8×8
  (nearest block top-left), un-flipped for baked-flip BG cells via `cell.hflip/vflip`.
  With a disabled ctx the output is byte-identical to before.
- The **live** Modern display currently routes through `FrameRenderer::render_modern_
  frame` → `render_modern_frame_full_from_vram` (a VRAM-decode path with **no source
  keys**, so no overrides can apply), uploaded as a 256×224 texture that wgpu scales to
  the window. HD requires switching live present to the **sources** path
  (`extract_modern_frame_from_sources` + the `assets_by_source` atlas), which earlier
  work already proved renders every module cleanly over the full route.

## Architecture

Parameterize the compositor with an integer scale **N** (1..4). Render into an
**N·256 × N·224** buffer:

- each tile instance draws an **N·8 × N·8** footprint,
- the **base index and `cgram_idx`** for an output pixel come from the *native* texel it
  covers (nearest: `native_local = out_local / N`),
- **HD-override color** comes from a scale-aware sub-pixel sampler,
- **non-override cells** nearest-upscale (each native texel → an N×N block),
- each **native scanline**'s scroll/window/TM/color-math is **replicated across its N
  output sub-rows** (`native_scanline = out_y / N`) — SNES scanline effects are
  native-granularity, so HD supersamples *within* a tile, not *across* scanlines.

**N=1 is arithmetically the current path** — every `*N` / `/N` collapses and every
`for sub in 0..N` loop runs once — so the existing behavior (and parity) is preserved by
construction. `render_modern_frame_full` becomes a call to the scaled entry with `N=1`
(single code path, no duplication); the existing 147 renderer tests + `zparity` pin the
N=1 output byte-for-byte.

```
env ZELDA3_HD_SCALE ── HdScale(1..4)
                          │
extract_from_sources ─────┤ (native tile instances — resolution-independent)
ModernHdOverrides (Ph.1) ─┤
                          ▼
render_modern_frame_full_scaled(frame, bg_cells, sprite_cells, ctx, N)
   Screen @ N·256×N·224 → composite (N·8 footprints, HD sub-pixel) → finalize @ N×
                          ▼
        N·256 × N·224 RGBA → upload as game texture → wgpu scales to window
```

## Components (each independently testable)

### 1. `HdScale`
Parse/clamp `ZELDA3_HD_SCALE` to `1..=4`. Unset → default **2**; invalid or out-of-range
→ clamp, log once. A tiny value type (e.g. `struct HdScale(u32)` with `from_env()` and
`get() -> u32`). Lives in `modern_software.rs` (or a small sibling module).

### 2. Scaled `Screen`
`Screen::new(backdrop_c5, len)` already takes a pixel count — construct it with
`len = (N·256)·(N·224)`. Composite functions currently hardcode `width =
MODERN_FRAME_WIDTH`; they take a runtime `out_width = N·256` (and `out_height = N·224`)
instead. No struct field changes beyond sizing.

### 3. Scaled compositor entry
`pub fn render_modern_frame_full_scaled(frame, bg_cells, sprite_cells, ctx, scale: u32)
-> Vec<u8>` returning an `N·256 × N·224` RGBA buffer. `render_modern_frame_full` and
`render_modern_frame_full_with_overrides` delegate to it AT `scale=1`, keeping their
current signatures: `…_with_overrides(…, ctx)` → `…_scaled(…, ctx, 1)` (so Phase 1 tests
are unchanged) and `render_modern_frame_full(…)` → `…_scaled(…, disabled, 1)`. The
**live HD display** calls `…_scaled(…, ctx, N)` directly. Thread `scale` +
`out_width`/`out_height` through `composite_mode1`, the mosaic/scanline variants, the 3
BG resolve fns, `resolve_obj_layer`, and `finalize_frame`.

At each of the 4 resolve sites, the per-instance loop changes from `for sy in 0..8 { for
sx in 0..8 }` (native) to a footprint loop over `0..N·8`; for output-local `(ox, oy)`:
- `nsx = ox / N`, `nsy = oy / N` → `index = cell.indices[nsy*8 + nsx]`, `cgram_idx` as
  today from `nsx/nsy`'s palette+index;
- HD sample coords are the **un-flipped output-local** position (apply the Phase 1
  `hflip/vflip` un-flip to `(ox, oy)` against the N·8 footprint);
- the destination pixel is `inst.screen_x*N + ox`, `inst.screen_y*N + oy` (tile origin
  scales by N).

### 4. Scaled HD sampler
Replace `HdCell::sample_native(lx, ly)` with `HdCell::sample_scaled(out_local_x,
out_local_y, footprint_px)` where `footprint_px = N·8`: map the output-local pixel to the
HD-art texel via `hd_x = out_local_x * self.width / footprint_px` (and y). For an HD cell
of size `M = k·8`: at `N==k` this is 1:1 (crisp HD); at `N<k` it downsamples; at `N>k` it
upsamples (block). **At N=1, `footprint_px = 8` and this equals the old `sample_native`**
(so Phase 1 tests still hold). The un-flip from Phase 1 is applied to `(out_local_x,
out_local_y)` before sampling.

### 5. `finalize_frame` at N×
Iterate `N²·len` output pixels; per pixel, `out_x = i % out_width`, `out_y = i /
out_width`; index window/color-math by the **native** scanline `out_y / N` (and native
column `out_x / N` where window membership is column-based). Everything else
(color-math, brightness expansion) is per-pixel on the scaled `c5` buffer.

### 6. Live wiring + display plumbing
- The live Modern present routes through the **sources** path
  (`extract_modern_frame_from_sources` with the loaded `assets_by_source` atlas) +
  `HdOverrideCtx::new(&ModernHdOverrides::from_env())`, calling
  `render_modern_frame_full_scaled(…, N)`. When the atlas or manifest is absent, HD art
  simply doesn't appear (cells resolve to `None`); N× still renders (nearest-upscaled).
- Mode-7 frames (`render_modern_mode7_frame`) are **nearest-upscaled** to N·256 × N·224
  after their native render (a small post-scale), so the presented frame size is
  consistent across module transitions.
- `FrameRenderer` uploads a frame of size `N·256 × N·224`; the game texture is
  (re)created whenever the frame dimensions change (guard on width/height change, not
  every frame). wgpu scales the larger texture to the window as before.

## Data flow

`env` → `HdScale` + `ModernHdOverrides` store (Phase 1). Per frame: extract native tile
instances from sources (unchanged) → `render_modern_frame_full_scaled(…, N)` → N× RGBA →
upload → present. The parity/compare/dump path keeps calling the N=1 entry
(`render_modern_frame_full`), so it never changes size or exercises new arithmetic in a
way that isn't already pinned by the N=1 identity.

## Error handling

- Bad/out-of-range `ZELDA3_HD_SCALE` → clamp to `1..=4`, log once.
- No override manifest / no source atlas → N× renders with everything nearest-upscaled
  (no HD art); harmless. HD detail appears only where an override *and* the atlas exist.
- Mode-7 → nearest-upscale native output to N×.
- Frame dimensions change (N toggled, or first HD frame) → the presentation layer
  recreates the game texture at the new size; never assume a fixed 256×224 texture.

## Testing

- **N=1 identity (parity anchor):** `render_modern_frame_full_scaled(…, 1)` is
  byte-identical to `render_modern_frame_full`. The existing 147 renderer tests run
  through the single path unchanged; a dedicated test asserts the identity on a non-
  trivial fixture.
- **N=2 geometry:** a non-override BG tile at N=2 yields exactly the 2× nearest block of
  the N=1 render (each native pixel → a 2×2 output block); tile origin scales by N.
- **HD sub-pixel sampling (N=2 and N=4):** a **spatially-varying** override cell renders
  its HD detail (distinct per output pixel), correctly un-flipped for a flipped BG
  instance — mirrors the Phase 1 discrimination test, at scale.
- **Per-scanline replication:** a frame with a mid-screen scroll/window change → the N
  output sub-rows of each native scanline share that scanline's effect; assert the
  boundary rows (row `N·k-1` vs `N·k`).
- **Perf:** a perf test renders at N=2 and N=4 and asserts < 16.6 ms/frame.
- **Display plumbing:** a unit test that the presentation texture is (re)created at
  N·256 × N·224 when the frame size changes.
- **Live wiring:** manual/integration — HD art visible in-game with `ZELDA3_HD_SCALE=2`
  and a loaded manifest; parity tooling (native) unaffected.

### Acceptance criteria
- HD-override BG and sprite art is visible in-game at N× (default 2×) at 60fps.
- `N=1` output is byte-identical to today; all existing renderer tests + `zparity` stay
  green; the parity/compare/dump path is unchanged.
- N× stays under 16.6 ms/frame through 4× (measured).
- Mode-7 and no-manifest cases render correctly (nearest-upscaled, no crash, consistent
  frame size).

## Scope boundaries (restated)

**In:** Mode-1 N× compositor with source-key HD overrides, the N=1-identity parity
guarantee, live-display routing through the sources path at N×, Mode-7 nearest-upscale,
and the variable-size display plumbing.
**Out:** GPU modern renderer, Mode-7 HD, >4× scale, 8-bit color depth, any change to the
Phase 1 kernel/store semantics (the sampler signature changes; the recolor math does
not).
