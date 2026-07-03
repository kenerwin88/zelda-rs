# Modern Effect Drawing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace baked/precolored variant drawing with compact source art plus reusable palette/effect materials while preserving final framebuffer parity.

**Architecture:** `art_tiles.*` and `base_tiles.*` provide source identity and atlas rects. `tile_effects.json` provides stable palette LUT effects keyed by palette, row, and color depth. The software renderer is the oracle for effect application first; the GPU renderer then moves the same LUT application into shader/material data instead of relying on preview RGBA pixels.

**Tech Stack:** Rust renderer crate, serde JSON loaders, existing `ModernFrame`/`ModernIndexTile` draw data, wgpu, Python extractor-generated `tile_effects.json`, existing replay/oracle parity scripts.

## Global Constraints

- Do not reintroduce default `tile_variants.*` generation.
- Keep `tile_variants.*` diagnostic-only behind `--write-diagnostic-variants`.
- Preserve fallback correctness: missing or dynamic effects must use the existing live indexed path until the effect is modeled.
- Final proof is `256x224` framebuffer parity, not atlas visual plausibility.
- Keep log formats compatible with existing compare scripts unless those scripts are updated in the same commit.

---

### Task 1: Load Effect Tables Into the Runtime Atlas

**Files:**
- Modify: `crates/renderer/src/modern_variant_atlas.rs`
- Test: `cargo test -p renderer modern_variant_atlas`

**Interfaces:**
- Produces: `TileEffect { id, palette, palette_row, colors_per_row, index_to_rgba, dynamic_policy }`
- Produces: `ModernVariantAtlas.effects: Vec<TileEffect>`
- Produces: `ModernVariantAtlas::effect_for_entry(&VariantAtlasEntry) -> Option<&TileEffect>`

- [x] **Step 1: Write failing loader test**

Add a `tile_effects.json` fixture beside a base/art atlas fixture and assert that `effect_for_entry` resolves `palette_dung_bg_main:8color:row2`.

- [x] **Step 2: Verify red**

Run: `cargo test -p renderer modern_base_art_atlas_loads_preview_keys_from_base_tiles_manifest`

Expected: fail because `effect_for_entry` does not exist.

- [x] **Step 3: Implement loader and lookup**

Parse `zelda3_tile_effects_v1`, accept `palette_lut`, convert `index_to_rgb` to opaque RGBA, and return an empty effect list when the file is absent.

- [x] **Step 4: Verify green**

Run: `cargo test -p renderer modern_base_art_atlas_loads_preview_keys_from_base_tiles_manifest`

Expected: pass.

### Task 2: Use Stable Palette LUTs In The Software Oracle

**Files:**
- Modify: `crates/renderer/src/modern_software.rs`
- Test: `cargo test -p renderer variant_atlas_software`

**Interfaces:**
- Consumes: `ModernVariantAtlas::effect_for_entry`
- Produces: `VariantAtlasRenderStats.effect_draws`
- Produces: BG and sprite software draws that prefer stable LUT effects and fall back to preview-RGBA atlas sampling when no stable effect exists.

- [x] **Step 1: Write failing BG color test**

Use a wrong preview RGBA pixel and a correct LUT color. Expected output must match the LUT color.

- [x] **Step 2: Verify red**

Run: `cargo test -p renderer variant_atlas_software_uses_palette_effect_for_bg_color`

Expected: fail with preview pixel output.

- [x] **Step 3: Implement BG LUT sampling**

For stable effects, sample `ModernIndexTile.indices` using the composed source/draw flip and use `effect.index_to_rgba[index]` for nonzero indices.

- [x] **Step 4: Add sprite coverage**

Repeat the same LUT proof for `ModernIndexSpriteInstance` and source-keyed sprite cells.

- [x] **Step 5: Verify green**

Run: `cargo test -p renderer variant_atlas_software`

Expected: pass.

### Task 3: Move LUT Application Into The GPU Variant Path

**Files:**
- Modify: `crates/renderer/src/modern_gpu.rs`
- Add or modify shader source used by the variant path
- Test: `cargo test -p renderer modern_gpu_variant_atlas_bg_tile_matches_software_variant`

**Interfaces:**
- Consumes: `ModernVariantAtlas.effects`
- Produces: GPU draw path that resolves the same LUT color as `render_modern_frame_software_variant_atlas`.

- [ ] **Step 1: Add a failing GPU/software effect comparison**

Construct a `ModernVariantAtlas` with wrong preview RGBA and a correct LUT, then assert the GPU variant renderer matches the software variant renderer.

- [ ] **Step 2: Add effect material upload**

Upload compact LUT data and per-instance effect selection to the GPU variant renderer. Keep fallback behavior for missing/dynamic effects.

- [ ] **Step 3: Apply LUT in shader**

Use source tile index plus effect LUT color instead of preview RGBA for effect-backed stable draws.

- [ ] **Step 4: Verify GPU/software parity**

Run: `cargo test -p renderer modern_gpu_variant_atlas_bg_tile_matches_software_variant`

Expected: pass with effect-backed draw cases.

### Task 4: Broaden Runtime Proof

**Files:**
- Modify docs and compare scripts only if new counters are exposed publicly.
- Test: `cargo test -p renderer`
- Test: `cargo build --profile parity -p zelda3-bin`

- [ ] **Step 1: Run renderer tests**

Run: `cargo test -p renderer`

Expected: all renderer tests pass.

- [ ] **Step 2: Run parity build**

Run: `cargo build --profile parity -p zelda3-bin`

Expected: build succeeds.

- [ ] **Step 3: Run representative oracle windows**

Run: `python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv`

Expected: no new mismatches from effect-backed draws; remaining fallbacks are documented dynamic/missing cases.
