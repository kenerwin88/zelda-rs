# Modern renderer HD N× output (Phase 2) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render the modern (off-VRAM) software compositor at an integer scale N so Phase-1 source-key HD overrides show at sub-pixel detail in-game at 60fps, with `N=1` byte-identical to today.

**Architecture:** Parameterize the **common** composite path (`composite_index_tiles_c5`, `resolve_obj_layer`, `paint_obj_priority`, `finalize_frame`, and `composite_mode1`'s dispatch) with `(out_width, scale)`; `(256, 1)` reproduces current behavior exactly. A new entry `render_modern_frame_full_scaled(…, scale)` dispatches: `scale≤1` → the existing native entry (untouched); a frame that would take the **mosaic** or **per-scanline-scroll** path → native render nearest-upscaled to N×; otherwise the parameterized common path at N·256×N·224. The mosaic/scanline helpers stay untouched (only ever run at scale=1). Live display routes through this entry.

**Tech Stack:** Rust, `crates/renderer` (CPU software compositor), `crates/platform` (wgpu presentation), `zelda3-bin` (live loop).

## Global Constraints

- **Parity is sacred / N=1 identity:** `render_modern_frame_full_scaled(…, 1)` and every `(out_width, scale) = (256, 1)` call MUST be byte-identical to today. The existing 147 renderer tests + `zparity` are the gate; if any pre-existing test's expected bytes change, STOP — it's a parity break.
- **Scale:** `ZELDA3_HD_SCALE`, integer, **default 2**, clamp **1..=4**.
- **Common path only:** parameterize `composite_index_tiles_c5`, `resolve_obj_layer`, `paint_obj_priority`, `finalize_frame`, `composite_mode1`. Do NOT touch `composite_mode1_mosaic`, `render_bg_layer_buf`, `mosaic_snap_bg_buf`, `paint_bg_buf`, `composite_mode1_scanline_scroll`, `render_bg_layer_torus` — mosaic/scanline frames use the native-upscale fallback.
- **Base index nearest, HD sub-pixel:** for output pixel `(ox,oy)` in a tile's `8·scale` footprint, the base slot index / `cgram_idx` come from native texel `(ox/scale, oy/scale)`; the HD override color comes from `sample_scaled` at the un-flipped `(ox,oy)` against `footprint_px = 8·scale`.
- **Per-scanline data is native-length:** TM (`main_tm[]`), window (`window_scanlines[]`), color-math index by the **native** row `out_y/scale` and native column `out_x/scale`.
- **Color stays 5-bit** through `finalize_frame` (unchanged math; only the buffer size and indexing scale).
- **Commit** each task with targeted `git add` + `--no-verify`; never `git add -A`; never stage unrelated user WIP (`crates/zelda3/*`, other `zelda3-bin/*`); never `git checkout`.
- **Build/test:** `cargo build --profile parity -p renderer` / `cargo test --profile parity -p renderer`.

---

### Task 1: `HdScale` — env-parsed scale factor

**Files:**
- Modify: `crates/renderer/src/modern_hd_overrides.rs` (add the type + tests)

**Interfaces:**
- Produces: `pub struct HdScale(u32)` with `pub fn from_env() -> HdScale`, `pub fn get(&self) -> u32`, `pub const DEFAULT: u32 = 2`.

- [ ] **Step 1: Write failing tests** (inside the existing `#[cfg(test)] mod tests`):

```rust
    #[test]
    fn hd_scale_parses_and_clamps() {
        assert_eq!(HdScale::from_str_opt(None).get(), 2);        // default
        assert_eq!(HdScale::from_str_opt(Some("1")).get(), 1);
        assert_eq!(HdScale::from_str_opt(Some("4")).get(), 4);
        assert_eq!(HdScale::from_str_opt(Some("0")).get(), 1);   // clamp low
        assert_eq!(HdScale::from_str_opt(Some("9")).get(), 4);   // clamp high
        assert_eq!(HdScale::from_str_opt(Some("xyz")).get(), 2); // invalid → default
    }
```

- [ ] **Step 2: Run to verify it fails.** `cargo test --profile parity -p renderer hd_scale_parses_and_clamps 2>&1 | tail -5` → FAIL (no `HdScale`).

- [ ] **Step 3: Implement** (above the tests):

```rust
/// Integer HD scale factor for the modern N× compositor. `ZELDA3_HD_SCALE`,
/// default 2, clamped to 1..=4 (the CPU 60fps ceiling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HdScale(u32);

impl HdScale {
    pub const DEFAULT: u32 = 2;

    pub fn from_env() -> Self {
        Self::from_str_opt(std::env::var("ZELDA3_HD_SCALE").ok().as_deref())
    }

    /// Testable core: parse/clamp an optional string.
    pub fn from_str_opt(s: Option<&str>) -> Self {
        let v = match s {
            None => Self::DEFAULT,
            Some(t) => match t.trim().parse::<u32>() {
                Ok(n) => n.clamp(1, 4),
                Err(_) => Self::DEFAULT,
            },
        };
        HdScale(v)
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}
```

- [ ] **Step 4: Run to verify pass.** `cargo test --profile parity -p renderer hd_scale 2>&1 | tail -5` → PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/renderer/src/modern_hd_overrides.rs
git commit --no-verify -m "feat(renderer): HdScale env-parsed HD scale factor (1..=4, default 2)"
```

---

### Task 2: `sample_scaled` + `resolve_pixel_color` footprint parameter

**Files:**
- Modify: `crates/renderer/src/modern_hd_overrides.rs`

**Interfaces:**
- Consumes: `HdCell` (Phase 1).
- Produces:
  - `pub fn HdCell::sample_scaled(&self, out_local_x: u32, out_local_y: u32, footprint_px: u32) -> [u8; 3]`
  - `resolve_pixel_color(base_index, cgram_idx, live_rgba, ov, reference, lx, ly, footprint_px)` — a new trailing `footprint_px: u32` argument; at `footprint_px == 8` it is byte-identical to the Phase 1 behavior (samples the native 8×8 top-left).

- [ ] **Step 1: Write failing tests** (inside `mod tests`):

```rust
    #[test]
    fn sample_scaled_maps_output_pixel_to_hd_texel() {
        // 16×16 HD cell (M=16). footprint 16 (scale 2): 1:1 mapping.
        let mut rgba = vec![0u8; 16 * 16 * 4];
        let put = |r: &mut Vec<u8>, x: usize, y: usize, c: [u8; 4]| {
            let i = (y * 16 + x) * 4;
            r[i..i + 4].copy_from_slice(&c);
        };
        put(&mut rgba, 9, 3, [7, 8, 9, 0xff]);
        let cell = HdCell { width: 16, height: 16, rgba };
        assert_eq!(cell.sample_scaled(9, 3, 16), [7, 8, 9]); // 9*16/16=9, 3*16/16=3
        // footprint 8 (scale 1) == native top-left of the whole cell region.
        assert_eq!(cell.sample_scaled(0, 0, 8), cell.sample_native(0, 0));
    }

    #[test]
    fn resolve_pixel_color_footprint_8_matches_phase1() {
        let mut reference = [[0u8; 4]; 256];
        reference[5] = [128, 128, 128, 0xff];
        let cell = HdCell { width: 8, height: 8, rgba: vec![64u8; 8 * 8 * 4] };
        // Same as the Phase 1 test, now with explicit footprint 8.
        assert_eq!(
            resolve_pixel_color(1, 5, [100, 100, 100, 0xff], Some(&cell), &reference, 0, 0, 8),
            Some([50, 50, 50, 0xff])
        );
    }
```

- [ ] **Step 2: Run to verify fail.** `cargo test --profile parity -p renderer sample_scaled 2>&1 | tail -10` → FAIL (method / arg missing).

- [ ] **Step 3: Implement.** Add `sample_scaled` next to `sample_native` (keep `sample_native` — it can delegate):

```rust
    /// Sample the HD texel for output-local pixel `(out_local_x, out_local_y)` within a
    /// tile footprint of `footprint_px` output pixels (= 8·scale): `hd = out_local *
    /// width / footprint_px`. At `footprint_px == 8` this is the native 8×8 sampling
    /// (equals `sample_native`). Alpha dropped (transparency is the base slot index's).
    pub fn sample_scaled(&self, out_local_x: u32, out_local_y: u32, footprint_px: u32) -> [u8; 3] {
        let fp = footprint_px.max(1);
        let px = (out_local_x * self.width / fp).min(self.width.saturating_sub(1));
        let py = (out_local_y * self.height / fp).min(self.height.saturating_sub(1));
        let idx = ((py * self.width + px) * 4) as usize;
        [self.rgba[idx], self.rgba[idx + 1], self.rgba[idx + 2]]
    }
```

Add the trailing `footprint_px: u32` param to `resolve_pixel_color` and call `hd.sample_scaled(lx, ly, footprint_px)` instead of `hd.sample_native(lx, ly)`:

```rust
#[allow(clippy::too_many_arguments)]
pub fn resolve_pixel_color(
    base_index: u8,
    cgram_idx: usize,
    live_rgba: [u8; 4],
    override_cell: Option<&HdCell>,
    reference: &[[u8; 4]; 256],
    lx: u32,
    ly: u32,
    footprint_px: u32,
) -> Option<[u8; 4]> {
    if base_index == 0 {
        return None;
    }
    match override_cell {
        Some(hd) => {
            let hd_rgb = hd.sample_scaled(lx, ly, footprint_px);
            let r = reference[cgram_idx];
            Some(detail_modulate(live_rgba, hd_rgb, [r[0], r[1], r[2]]))
        }
        None => Some(live_rgba),
    }
}
```

- [ ] **Step 4: Fix the 4 native call sites.** In `crates/renderer/src/modern_software.rs`, the 4 existing `resolve_pixel_color(…, hx as u32, hy as u32)` calls (in `render_bg_layer_buf`, `render_bg_layer_torus`, `composite_index_tiles_c5`, `resolve_obj_layer`) get a trailing `, 8` (native footprint). Also update any Phase 1 kernel tests that call `resolve_pixel_color` to pass a trailing `8`.

- [ ] **Step 5: Run to verify pass + no regressions.** `cargo test --profile parity -p renderer 2>&1 | grep "test result"` → all pass, count = prior + 2 new. (The `, 8` calls keep every render byte-identical.)

- [ ] **Step 6: Commit.**

```bash
git add crates/renderer/src/modern_hd_overrides.rs crates/renderer/src/modern_software.rs
git commit --no-verify -m "feat(renderer): HdCell::sample_scaled + resolve_pixel_color footprint arg (fp=8 identity)"
```

---

### Task 3: `upscale_rgba_nearest` — block-replicate a native frame to N×

**Files:**
- Modify: `crates/renderer/src/modern_software.rs`

**Interfaces:**
- Produces: `pub fn upscale_rgba_nearest(rgba: &[u8], width: usize, height: usize, scale: usize) -> Vec<u8>` → a `(width·scale)·(height·scale)·4` RGBA buffer; each source pixel becomes a `scale×scale` block. `scale == 1` returns an exact copy.

- [ ] **Step 1: Write failing test** (in `mod tests`):

```rust
    #[test]
    fn upscale_nearest_block_replicates() {
        // 2×1 source, scale 2 → 4×2 output; each pixel a 2×2 block.
        let src = vec![10, 20, 30, 40, /*px1*/ 50, 60, 70, 80];
        let out = upscale_rgba_nearest(&src, 2, 1, 2);
        assert_eq!(out.len(), 4 * 2 * 4);
        // row 0: px0 px0 px1 px1
        assert_eq!(&out[0..4], &[10, 20, 30, 40]);
        assert_eq!(&out[4..8], &[10, 20, 30, 40]);
        assert_eq!(&out[8..12], &[50, 60, 70, 80]);
        // row 1 mirrors row 0
        let row1 = 4 * 4;
        assert_eq!(&out[row1..row1 + 4], &[10, 20, 30, 40]);
        // scale 1 is identity
        assert_eq!(upscale_rgba_nearest(&src, 2, 1, 1), src);
    }
```

- [ ] **Step 2: Run to verify fail.** `cargo test --profile parity -p renderer upscale_nearest 2>&1 | tail -5` → FAIL.

- [ ] **Step 3: Implement** (near `finalize_frame`):

```rust
/// Block-replicate an RGBA frame to `scale`× (nearest upscale): each source pixel
/// becomes a `scale×scale` block. Used for the mosaic/per-scanline-scroll fallback,
/// which renders natively then upscales to match the HD frame size.
pub fn upscale_rgba_nearest(rgba: &[u8], width: usize, height: usize, scale: usize) -> Vec<u8> {
    if scale <= 1 {
        return rgba.to_vec();
    }
    let out_w = width * scale;
    let mut out = vec![0u8; out_w * height * scale * 4];
    for sy in 0..height {
        for sx in 0..width {
            let src = (sy * width + sx) * 4;
            let px: [u8; 4] = [rgba[src], rgba[src + 1], rgba[src + 2], rgba[src + 3]];
            for dy in 0..scale {
                let oy = sy * scale + dy;
                for dx in 0..scale {
                    let ox = sx * scale + dx;
                    let o = (oy * out_w + ox) * 4;
                    out[o..o + 4].copy_from_slice(&px);
                }
            }
        }
    }
    out
}
```

- [ ] **Step 4: Run to verify pass.** `cargo test --profile parity -p renderer upscale_nearest 2>&1 | tail -5` → PASS.

- [ ] **Step 5: Commit.**

```bash
git add crates/renderer/src/modern_software.rs
git commit --no-verify -m "feat(renderer): upscale_rgba_nearest block-replicate helper"
```

---

### Task 4: Parameterize the common composite path with `(out_width, scale)`

**Files:**
- Modify: `crates/renderer/src/modern_software.rs`

**Interfaces:**
- Consumes: `sample_scaled`/`resolve_pixel_color` footprint arg (Task 2).
- Produces: `composite_index_tiles_c5`, `resolve_obj_layer`, `paint_obj_priority`, `finalize_frame`, and `composite_mode1` each gain trailing `out_width: usize, scale: usize` params (or one `scale: usize` where `out_width` is derived as `256*scale`). At `scale == 1` every one is byte-identical to today.

**The transform (apply uniformly; `scale=1` must collapse to the current arithmetic):**
1. Any `let width = usize::from(MODERN_FRAME_WIDTH);` used to index the OUTPUT buffer → `let width = 256 * scale;` (pass it or derive it). Output height where needed → `224 * scale`.
2. A per-tile pixel loop `for sy in 0..8 { for sx in 0..8 { … cell.indices[sy*8+sx] … dst = inst.screen_x + sx … } }` becomes:

```rust
let fp = 8 * scale as u32;
for oy in 0..(8 * scale) {
    for ox in 0..(8 * scale) {
        let nsx = ox / scale;            // native texel column 0..8
        let nsy = oy / scale;            // native texel row 0..8
        let index = cell.indices[nsy * 8 + nsx];
        if index == 0 { continue; }
        let dst_x = inst.screen_x as isize * scale as isize + ox as isize;
        let dst_y = inst.screen_y as isize * scale as isize + oy as isize;
        if dst_x < 0 || dst_y < 0 || dst_x >= (256 * scale) as isize || dst_y >= (224 * scale) as isize { continue; }
        let (dst_x, dst_y) = (dst_x as usize, dst_y as usize);
        // un-flip HD sample coords against the footprint (Phase-1 rule, scaled):
        let hx = if cell.hflip { fp as usize - 1 - ox } else { ox };
        let hy = if cell.vflip { fp as usize - 1 - oy } else { oy };
        let cgram_idx = inst.palette as usize * 16 + index as usize; // OBJ: 0x80 + …
        let color = match crate::modern_hd_overrides::resolve_pixel_color(
            index, cgram_idx, frame.cgram_rgba[cgram_idx], ov, ctx.reference(),
            hx as u32, hy as u32, fp,
        ) { Some(c) => c, None => continue };
        // per-scanline gates index the NATIVE row/col:
        let nrow = dst_y / scale;        // 0..224
        let ncol = dst_x / scale;        // 0..256
        // (TM: main_tm[nrow] & bit; window: layer_window_masks(…, ncol as u32, nrow, …))
        let i = dst_y * width + dst_x;
        screen.c5[i] = [color[0] >> 3, color[1] >> 3, color[2] >> 3];
        // …existing real/bit/hipri writes at i…
    }
}
```

   Apply this shape to the per-tile loop in `composite_index_tiles_c5` (base index + TM + window gates already there — just index them by `nrow`/`ncol`), and to the sprite draw loop in `resolve_obj_layer` (OBJ `cgram_idx = 0x80 + palette*16 + index`; sprites are NOT flip-baked, so keep the existing `src_x/src_y` source-coord logic and pass those, scaled, as the HD sample coords — mirror how Phase 1 already handled the sprite path, now over the `8*scale` footprint).
3. `paint_obj_priority` and any buffer-wide loop that indexes `i / width` / `i % width` for TM/window → use `width = 256*scale` for the split, then index the per-scanline table by `(i / width) / scale` (native row) and window col by `(i % width) / scale`.
4. `finalize_frame`: `let width = 256 * scale; let len = main.c5.len();` iterate `0..len`; `out_x = i % width`, `out_y = i / width`; index `window_scanlines` / color-math by native `out_y / scale` and native column `out_x / scale`. All color-math + `expand_brightness` math is unchanged (per output pixel on the scaled `c5`).
5. `composite_mode1`: pass `out_width, scale` through to its simple-branch calls (`composite_index_tiles_c5`, `resolve_obj_layer`, `paint_obj_priority`). Its mosaic/scanline branches keep calling their helpers with native dims — those branches are only reached at `scale == 1` (the entry in Task 5 routes complex frames elsewhere), so leave them native.

- [ ] **Step 1: Write the N=1 identity guard + an N=2 direct geometry test** (in `mod tests`). Reuse the existing single-BG-tile fixture (`frame_with_single_bg_pixel` or the Task-5-era fixture) that renders through `composite_mode1` at native. Add:

```rust
    #[test]
    fn composite_scale1_is_identity_and_scale2_block_upscales() {
        // Build a tiny frame with one opaque BG tile (no mosaic, no scanline scroll,
        // no overrides) — reuse an existing simple fixture in this module.
        let (frame, bg_cells) = tiny_simple_bg_fixture();
        // Native reference.
        let native = render_modern_frame_full(&frame, &bg_cells, &[]); // 256×224×4
        // scale=2 via the new entry (Task 5); until Task 5, call the scaled simple
        // path directly if the entry isn't in yet. Assert each native pixel == its 2×2
        // block in the N=2 output.
        let hd = render_modern_frame_full_scaled(
            &frame, &bg_cells, &[], &crate::modern_hd_overrides::HdOverrideCtx::disabled(), 2);
        assert_eq!(hd.len(), 512 * 448 * 4);
        for ny in 0..224usize { for nx in 0..256usize {
            let src = (ny * 256 + nx) * 4;
            for dy in 0..2 { for dx in 0..2 {
                let o = ((ny * 2 + dy) * 512 + (nx * 2 + dx)) * 4;
                assert_eq!(&hd[o..o + 3], &native[src..src + 3], "block ({nx},{ny})");
            }}
        }}
    }
```

   (This test depends on the Task-5 entry; if you implement Task 4 and Task 5 together, fine — they land in one cohesive compositor change. Otherwise stage the test to run after Task 5.)

- [ ] **Step 2: Run the FULL suite to verify the N=1 identity holds after each function you touch.** `cargo test --profile parity -p renderer 2>&1 | grep "test result"`. Every pre-existing test MUST stay green with unchanged expected bytes. If any changes, STOP and report a parity break.

- [ ] **Step 3: Apply the transform** to `composite_index_tiles_c5`, `resolve_obj_layer`, `paint_obj_priority`, `finalize_frame`, `composite_mode1` per the recipe. Update all callers to pass `(256, 1)` for the native path (the existing `render_modern_frame_full_with_overrides`, `composite_mode1_mosaic`, `composite_mode1_scanline_scroll`, and the mode7 finalize call site). Build clean.

- [ ] **Step 4: Run the full suite.** `cargo test --profile parity -p renderer 2>&1 | grep "test result"` → all pass (N=1 identity intact).

- [ ] **Step 5: Commit.**

```bash
git add crates/renderer/src/modern_software.rs
git commit --no-verify -m "feat(renderer): parameterize common composite path with (out_width, scale); N=1 identical"
```

---

### Task 5: Scaled entry + complex-frame fallback

**Files:**
- Modify: `crates/renderer/src/modern_software.rs`

**Interfaces:**
- Consumes: Task 4 (parameterized `composite_mode1`/`finalize_frame`), Task 3 (`upscale_rgba_nearest`), `bg_layer_scroll_varies` (existing).
- Produces:
  - `pub fn render_modern_frame_full_scaled(frame, bg_cells, sprite_cells, ctx, scale: u32) -> Vec<u8>`
  - `fn frame_uses_complex_bg_path(frame: &ModernFrame) -> bool`

- [ ] **Step 1: Write the entry + fallback tests** (in `mod tests`):

```rust
    #[test]
    fn scaled_entry_scale1_equals_native() {
        let (frame, bg_cells) = tiny_simple_bg_fixture();
        let native = render_modern_frame_full(&frame, &bg_cells, &[]);
        let via = render_modern_frame_full_scaled(
            &frame, &bg_cells, &[], &crate::modern_hd_overrides::HdOverrideCtx::disabled(), 1);
        assert_eq!(via, native);
    }

    #[test]
    fn complex_frame_falls_back_to_native_upscaled() {
        // A mosaic-active frame: scale=2 output must equal the native render block-upscaled.
        let (mut frame, bg_cells) = tiny_simple_bg_fixture();
        frame.mosaic_size = 2;
        frame.mosaic_enabled = 0x01;                 // BG1 mosaic
        frame.screen_enabled_main |= 0x01;           // BG1 enabled
        assert!(frame_uses_complex_bg_path(&frame));
        let native = render_modern_frame_full(&frame, &bg_cells, &[]);
        let expected = upscale_rgba_nearest(&native, 256, 224, 2);
        let hd = render_modern_frame_full_scaled(
            &frame, &bg_cells, &[], &crate::modern_hd_overrides::HdOverrideCtx::disabled(), 2);
        assert_eq!(hd, expected);
    }
```

- [ ] **Step 2: Run to verify fail.** `cargo test --profile parity -p renderer scaled_entry 2>&1 | tail -10` → FAIL.

- [ ] **Step 3: Implement.**

```rust
/// True if the frame would take the mosaic OR per-scanline-scroll composite path on
/// EITHER screen — those paths are not N×-parameterized (Phase 2), so the entry renders
/// them natively and nearest-upscales. Detection mirrors `composite_mode1` verbatim.
fn frame_uses_complex_bg_path(frame: &ModernFrame) -> bool {
    for enabled in [frame.screen_enabled_main, frame.screen_enabled_sub] {
        let mosaic = frame.mosaic_size > 1 && (frame.mosaic_enabled & enabled & 0x07) != 0;
        let bg_on = |i: usize| (enabled >> i) & 1 != 0;
        let scanline = (0..3).any(|i| bg_on(i) && bg_layer_scroll_varies(frame, i));
        if mosaic || scanline {
            return true;
        }
    }
    false
}

/// Modern render at integer `scale` (1..=4). `scale<=1` → the native entry unchanged.
/// A mosaic / per-scanline-scroll frame → native render nearest-upscaled to N×.
/// Otherwise the parameterized common path at N·256 × N·224.
pub fn render_modern_frame_full_scaled(
    frame: &ModernFrame,
    bg_cells: &[ModernIndexTile],
    sprite_cells: &[ModernIndexTile],
    ctx: &crate::modern_hd_overrides::HdOverrideCtx,
    scale: u32,
) -> Vec<u8> {
    let scale = scale.clamp(1, 4) as usize;
    if scale == 1 {
        return render_modern_frame_full_with_overrides(frame, bg_cells, sprite_cells, ctx);
    }
    if frame_uses_complex_bg_path(frame) {
        let native = render_modern_frame_full_with_overrides(frame, bg_cells, sprite_cells, ctx);
        return upscale_rgba_nearest(&native, 256, 224, scale);
    }
    let out_width = 256 * scale;
    let len = out_width * 224 * scale;
    if frame.forced_blank {
        let mut out = vec![0u8; len * 4];
        for px in out.chunks_exact_mut(4) { px.copy_from_slice(&[0, 0, 0, 0xff]); }
        return out;
    }
    let bd = &frame.backdrop_color_rgba;
    let backdrop_c5 = [bd[0] >> 3, bd[1] >> 3, bd[2] >> 3];
    let mut main = Screen::new(backdrop_c5, len);
    composite_mode1(&mut main, frame, bg_cells, sprite_cells,
        frame.screen_enabled_main, Some(&frame.main_tm_scanlines), frame.screen_windowed_main, ctx,
        out_width, scale);
    let mut sub = Screen::new(backdrop_c5, len);
    composite_mode1(&mut sub, frame, bg_cells, sprite_cells,
        frame.screen_enabled_sub, None, frame.screen_windowed_sub, ctx, out_width, scale);
    finalize_frame(&main, &sub, frame, out_width, scale)
}
```

(Adjust the exact `composite_mode1` / `finalize_frame` argument lists to match the signatures you chose in Task 4.)

- [ ] **Step 4: Add the HD sub-pixel + scanline-replication tests** (prove overrides actually sample sub-pixel at N×, and native scanline data is replicated). Reuse the Phase-1 discrimination pattern (a spatially-varying `HdCell`) with `scale=2`, and a fixture with a known TM/window row to assert the N output sub-rows share the native row's gate. Concretely:

```rust
    #[test]
    fn hd_override_samples_subpixel_at_scale2() {
        // One BG tile with a real source_key; a 16×16 spatially-varying HD cell whose
        // top-left 2×2 differs from its (2,0) block. At scale=2, output pixels (0,0)
        // and (1,0) of that tile must sample DIFFERENT HD texels (0,0) and (1,0) — not
        // the same native texel — proving sub-pixel sampling. Build the store with
        // ModernHdOverrides::from_parts, render via render_modern_frame_full_scaled(…,2),
        // and assert the two adjacent output pixels differ as the HD art dictates.
        // (Mirror the Phase-1 `source_keyed_overrides_recolor_bg_and_sprite` construction.)
    }
```

   Fill the test body using the Phase-1 fixture helpers (`from_parts`, a real `source_key`, spatially-varying `rgba`); assert two adjacent output pixels of the override tile take their color from adjacent HD texels (differ), and that reverting to `scale=1` collapses them. If you cannot make it discriminate, the `(ox,oy)` → `sample_scaled` wiring in Task 4 is wrong — fix Task 4.

- [ ] **Step 5: Run the full suite.** `cargo test --profile parity -p renderer 2>&1 | grep "test result"` → all pass.

- [ ] **Step 6: Commit.**

```bash
git add crates/renderer/src/modern_software.rs
git commit --no-verify -m "feat(renderer): render_modern_frame_full_scaled entry + complex-frame native-upscale fallback"
```

---

### Task 6: Perf guard at N=2 and N=4

**Files:**
- Modify: `crates/renderer/src/modern_extract.rs` (near `perf_render_modern_frame_full_from_vram`)

**Interfaces:**
- Consumes: `render_modern_frame_full_scaled`.

- [ ] **Step 1: Write the perf test** (mirror the existing `perf_render_modern_frame_full_from_vram`, but call the scaled entry). It builds the same VRAM-derived `ModernFrame` + cells the existing perf test uses, times 50 iterations at scale 2 and scale 4, prints ms/frame, and asserts each is `< 16.6` ms:

```rust
    #[test]
    fn perf_render_modern_frame_scaled() {
        // Reuse the same frame/cells construction as perf_render_modern_frame_full_from_vram.
        let (frame, bg_cells, sprite_cells) = perf_fixture(); // extract from the existing test's setup
        for scale in [2u32, 4] {
            let iters = 20;
            let start = std::time::Instant::now();
            let mut sink = 0usize;
            for _ in 0..iters {
                let out = crate::modern_software::render_modern_frame_full_scaled(
                    &frame, &bg_cells, &sprite_cells,
                    &crate::modern_hd_overrides::HdOverrideCtx::disabled(), scale);
                sink = sink.wrapping_add(out.len());
            }
            let ms = start.elapsed().as_secs_f64() * 1000.0 / iters as f64;
            eprintln!("perf render_modern_frame_full_scaled x{scale}: {ms:.3} ms/frame ({sink})");
            assert!(ms < 16.6, "scale {scale} too slow: {ms:.3} ms");
        }
    }
```

   Note: the existing perf test builds its frame inline; extract that construction (or copy it) so both tests share the fixture. If the frame happens to be a complex (mosaic/scanline) frame it will hit the fallback — that's fine; still measures a realistic upper bound.

- [ ] **Step 2: Run it.** `cargo test --profile parity -p renderer perf_render_modern_frame_scaled -- --nocapture 2>&1 | rg "ms/frame|test result"` → prints both scales, PASS (< 16.6 ms).

- [ ] **Step 3: Commit.**

```bash
git add crates/renderer/src/modern_extract.rs
git commit --no-verify -m "test(renderer): perf guard for N× modern render (scale 2 & 4 < 16.6ms)"
```

---

### Task 7: Live wiring + display plumbing

**Files:**
- Modify: `crates/renderer/src/lib.rs` (`FrameRenderer::render_modern_frame` and its texture sizing)
- Modify: `crates/platform/src/lib.rs` (route Modern present with scale) and/or `zelda3-bin/src/main.rs` (the live sources render path)

**Interfaces:**
- Consumes: `render_modern_frame_full_scaled`, `HdScale::from_env`, `ModernHdOverrides::from_env`, `extract_modern_frame_from_sources`.

**Context:** `FrameRenderer::render_modern_frame` currently does `let rgba = render_modern_frame_full_from_vram(frame);` then uploads a fixed 256×224 texture. For HD, live present must (a) use the **sources** path (source atlas + overrides) when available, (b) render at N×, and (c) upload an N·256 × N·224 texture, recreating the game texture when the frame dimensions change.

- [ ] **Step 1: Add a size-aware upload test** (in `crates/renderer/src/lib.rs` tests, following the existing `upload_frame_*` GPU test pattern): render a 2× frame (`render_modern_frame_full_scaled(…, 2)` → 512×448 RGBA) and assert the FrameRenderer accepts a frame whose dimensions are 512×448 and (re)creates its game texture at that size — assert the internal texture extent updates. Model it on the existing `upload_frame_swaps_bgr_to_rgb` / render-modern test scaffolding.

- [ ] **Step 2: Run to verify fail.** Focused test → FAIL (texture fixed at 256×224).

- [ ] **Step 3: Implement.**
  - In `FrameRenderer`, make the game texture size dynamic: track `(tex_w, tex_h)`; when an uploaded modern frame's dimensions differ, recreate the texture + bind group at the new size (guard so it only recreates on change, not every frame).
  - Change `render_modern_frame` to: read `HdScale::from_env()` once (cache on the renderer) and the `ModernHdOverrides` store; if the source atlas + store are available, build the `HdOverrideCtx` and render via the **sources** extraction (`extract_modern_frame_from_sources`) + `render_modern_frame_full_scaled(…, scale)`; else keep the current `_from_vram` path but still `render_modern_frame_full_scaled(…, scale)` on a VRAM-decoded frame (no overrides, just N× nearest — wait: `_from_vram` returns a finished Vec<u8>; instead call the scaled entry on the extracted `ModernFrame`). Upload the resulting `N·256 × N·224` RGBA.
  - Mode-7: `render_modern_mode7_frame` returns a native Vec<u8>; wrap it with `upscale_rgba_nearest(&mode7, 256, 224, scale)` so the presented frame size stays `N·256 × N·224` across module transitions.
  - Thread the source atlas handle into `FrameRenderer` (it already exists for the sources compare path in `zelda3-bin`; expose/pass it, or load it in the renderer via the same loader `load_modern_source_atlas`).

  (The exact wiring depends on where the atlas + `GpuFrame` are available in `FrameRenderer` vs `zelda3-bin`; follow the existing `--modern-index-compare` sources path in `zelda3-bin/src/main.rs:5357-5387` as the reference for building `src_slice` + `extract_modern_frame_from_sources`. If the atlas isn't reachable inside `FrameRenderer`, do the sources render in `zelda3-bin`'s present loop and pass the finished N× RGBA to a size-aware `upload_frame`.)

- [ ] **Step 4: Run the focused test + full renderer suite + platform build.**
`cargo test --profile parity -p renderer 2>&1 | grep "test result"` → pass.
`cargo build --profile parity -p zelda3-bin 2>&1 | tail -3` → clean.

- [ ] **Step 5: Manual smoke (document, don't automate).** Note in the report the command to eyeball HD live: run the game/replay with `ZELDA3_RENDERER=assets-anim ZELDA3_HD_SCALE=2 ZELDA3_MODERN_HD_OVERRIDES=<manifest>` and confirm the window shows a 2×-resolution frame (blocky where no override, crisp where an override exists), and that without the env vars the display is unchanged.

- [ ] **Step 6: Commit.**

```bash
git add crates/renderer/src/lib.rs crates/platform/src/lib.rs zelda3-bin/src/main.rs
git commit --no-verify -m "feat: live N× HD display via sources path + size-aware presentation"
```

---

## Self-review notes

- **Spec coverage:** HdScale (T1) ↔ §Component 1; sample_scaled (T2) ↔ §Component 4; upscale helper (T3) ↔ §Component 6 fallback; parameterized common path (T4) ↔ §Architecture + §Components 2/3/5; scaled entry + complex fallback (T5) ↔ §Component 3 + §Scope-refinement; perf (T6) ↔ §Testing perf; live wiring + display plumbing (T7) ↔ §Component 6. N=1 identity guard appears in T2/T4/T5.
- **Deferred (spec-declared):** mosaic/scanline HD (native-upscaled via T5 fallback), Mode-7 HD (upscaled in T7), >4×, GPU path, 8-bit depth.
- **Type consistency:** `resolve_pixel_color(… , footprint_px: u32)` defined T2, used in T4's recipe; `render_modern_frame_full_scaled(…, scale: u32)` defined T5, used in T4 test, T6, T7; `upscale_rgba_nearest(rgba,width,height,scale)` defined T3, used T5/T7; `HdScale::from_env`/`get` defined T1, used T7; `frame_uses_complex_bg_path` defined T5.
- **Risk note:** T4 is the intricate task (parameterizing coupled per-pixel code); its gate is the full 147-test N=1 identity after each function. T4 and T5 may be implemented together (one cohesive compositor change) if the N=2 tests need the entry — the plan lists them separately for review granularity but a combined T4+T5 commit is acceptable if the reviewer sees both.
