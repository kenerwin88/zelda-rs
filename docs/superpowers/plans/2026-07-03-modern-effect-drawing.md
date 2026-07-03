# Modern Effect Drawing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace baked/precolored variant drawing with compact source art plus reusable palette/effect materials while preserving final framebuffer parity.

**Architecture:** `art_tiles.*` provides source identity and atlas rects for the default runtime path. `tile_effects.json` provides stable palette LUT effects keyed by palette, row, and color depth. `base_tiles.*` is legacy/debug output only. The software renderer is the oracle for effect application first; the GPU renderer then moves the same LUT application into shader/material data instead of relying on preview RGBA pixels.

**Tech Stack:** Rust renderer crate, serde JSON loaders, existing `ModernFrame`/`ModernIndexTile` draw data, wgpu, Python extractor-generated `tile_effects.json`, existing replay/oracle parity scripts.

## Current State After Source-Key Material Split

The default `assets-variant-gpu` path now loads `art_tiles.*` as the runtime
source atlas and resolves atlas art by ROM/source identity rather than by
preview palette. Live palette/material selection is resolved separately through
`tile_effects.json`. This means a source tile drawn with a different live
palette row should no longer be classified as missing art.

The next modernization step is a shared draw resolver: one small layer in
`modern_variant_atlas.rs` should classify each live draw as stable preview art,
stable effect-backed art, dynamic/live-indexed fallback, missing art, or unkeyed
fallback. The software oracle, GPU overlay builder, and GPU effect instance
builder should consume that resolver instead of duplicating the classification
rules.

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

Run: `cargo test -p renderer modern_canonical_art_atlas_loads_source_refs`

Expected: fail because `effect_for_entry` does not exist.

- [x] **Step 3: Implement loader and lookup**

Parse `zelda3_tile_effect_table_v1`, accept `palette_lut`, convert `index_to_rgb` to opaque RGBA, and return an empty effect list when the file is absent.

- [x] **Step 4: Verify green**

Run: `cargo test -p renderer modern_canonical_art_atlas_loads_source_refs`

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

- [x] **Step 1: Add a failing GPU/software effect comparison**

Construct a `ModernVariantAtlas` with wrong preview RGBA and a correct LUT, then assert the GPU variant renderer matches the software variant renderer.

- [x] **Step 2: Add effect material upload**

Upload compact LUT data and per-instance effect selection to the GPU variant renderer. Keep fallback behavior for missing/dynamic effects.

- [x] **Step 3: Apply LUT in shader**

Use source tile index plus effect LUT color instead of preview RGBA for effect-backed stable draws.

- [x] **Step 4: Verify GPU/software parity**

Run: `cargo test -p renderer modern_gpu_variant_atlas_bg_tile_matches_software_variant`

Expected: pass with effect-backed draw cases.

### Task 4: Broaden Runtime Proof

**Files:**
- Modify docs and compare scripts only if new counters are exposed publicly.
- Test: `cargo test -p renderer`
- Test: `cargo build --profile parity -p zelda3-bin`

- [x] **Step 1: Run renderer tests**

Run: `cargo test -p renderer`

Expected: all renderer tests pass.

Current evidence: `cargo test -p renderer --lib -- --skip perf_render_modern_frame_scaled`
passes the renderer functional suite. The unfiltered perf threshold test remains
separate from functional parity.

- [x] **Step 2: Run parity build**

Run: `cargo build --profile parity -p zelda3-bin`

Expected: build succeeds.

- [x] **Step 3: Run representative oracle windows**

Run: `python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv --cold`

Expected: no new mismatches from effect-backed draws; remaining fallbacks are documented dynamic/missing cases. The wrapper now uses the existing oracle checkpoint ledger by default for routine tail validation; pass `--cold` when the proof must replay every selected window from frame 0.

Historical evidence before the default-art/effect-loader fixes: `title-start`
and `file-select-new-game` passed with
`mismatched_pixels=0` under `assets-variant-gpu`. The progress-friendly wrapper
also proved the longer `file-select-enter-game` window:
`compared=107000`, `mismatched_pixels=0`, `variant_draws=0`,
`fallback_draws=1194079978`, `dynamic_palette_draws=0`,
`missing_variant_draws=524171`.

The extended saved-slot gameplay route also passes under `assets-variant-gpu`:
`file-select-message-dismiss-wander` completed `compared=122000`,
`mismatched_pixels=0`, `variant_draws=0`, `fallback_draws=1363234978`,
`dynamic_palette_draws=0`, `missing_variant_draws=554171`.

The diagonal indoor movement route also passes under `assets-variant-gpu`:
`file-select-diagonal-sweeps` completed `compared=116000`,
`mismatched_pixels=0`, `variant_draws=0`, `fallback_draws=1295572978`,
`dynamic_palette_draws=0`, `missing_variant_draws=542171`.

The isolated button-tap route also passes under `assets-variant-gpu`:
`file-select-button-taps` completed `compared=112000`,
`mismatched_pixels=0`, `variant_draws=0`, `fallback_draws=1250464978`,
`dynamic_palette_draws=0`, `missing_variant_draws=534171`.

The button-direction probe route also passes under `assets-variant-gpu`:
`file-select-button-direction-probes` completed `compared=114000`,
`mismatched_pixels=0`, `variant_draws=0`, `fallback_draws=1273018978`,
`dynamic_palette_draws=0`, `missing_variant_draws=538171`.

The remaining non-SRAM TAS/opening windows also pass under
`assets-variant-gpu`:
`tas-us-rta-ace` completed `compared=32613`, `mismatched_pixels=0`,
`variant_draws=0`, `fallback_draws=288354177`,
`dynamic_palette_draws=0`, `missing_variant_draws=1258620`;
`opening-uncle-dismiss-and-move` completed `compared=36810`,
`mismatched_pixels=0`, `variant_draws=0`, `fallback_draws=402547348`,
`dynamic_palette_draws=0`, `missing_variant_draws=383791`;
`opening-uncle-extended-move` completed `compared=45610`,
`mismatched_pixels=0`, `variant_draws=0`, `fallback_draws=501784948`,
`dynamic_palette_draws=0`, `missing_variant_draws=401391`;
`opening-uncle-diagonal-sweeps` completed `compared=45610`,
`mismatched_pixels=0`, `variant_draws=0`, `fallback_draws=501784948`,
`dynamic_palette_draws=0`, `missing_variant_draws=401391`.

`no-input-intro` is now included in dry-runs after fixing the empty-input SRAM
sidecar filter. That exposed a real modern compositor gap: the first mismatch
was frame `1954`, `ppumode=1`, `via=vram`/`variant-gpu`, with a far-left
scroll-wrap strip (`classic_rgb=(66,24,0)`, `modern_rgb=(24,16,0)`). The root
cause was uniform non-zero BG scroll using the fast screen-space compositor path
instead of the existing torus wrap sampler. New evidence after the fix:
`ZELDA3_RENDERER=classic ... --play-gpu-render-compare ... 30000
--modern-index-compare 1` completed with `bad_count=0`, `bad_pixels=0`, and
`cpu_count=30000`. The default `assets-variant-gpu` path now passes the full
`no-input-intro` window too: `compared=30000`, `mismatched_pixels=0`,
`variant_draws=0`, `fallback_draws=188749315`, `dynamic_palette_draws=0`,
`missing_variant_draws=2070467`.

The non-SRAM representative oracle matrix has now passed under
`assets-variant-gpu`. For faster iteration, the GPU oracle wrapper resumes from
the newest recorded checkpoint by default. A real checkpointed rerun of
`file-select-button-taps` compared only the `5000`-frame tail from
`start_frame=107000` and passed with `mismatched_pixels=0`,
`fallback_draws=34650495`, `dynamic_palette_draws=0`, and
`missing_variant_draws=67564`; the equivalent cold run remains available with
`--cold`. The wrapper also accepts `--jobs N` for controlled parallel window
execution. A real `--jobs 2` rerun of `file-select-button-taps` and
`opening-uncle-extended-move` passed with `compared=22000`,
`mismatched_pixels=0`, `fallback_draws=216038244`,
`dynamic_palette_draws=0`, and `missing_variant_draws=2088816`.

Current implementation evidence after the default-art/effect-loader and live
present work: `art_tiles.*` source-kind-default refs can resolve as stable when
`tile_effects.json` has a stable LUT; mixed fallback/effect frames overlay
effect-backed stable cells through the LUT shader; and live
`assets-variant-gpu` presents on the window renderer's GPU device without
headless readback or CPU RGBA upload. Use `ZELDA3_VARIANT_LIVE_STATS=1` for a
cheap live draw-mix check; representative oracle windows should be rerun when a
fresh route-wide proof is required.

### Task 5: Share Variant Draw Classification Across Render Backends

**Files:**
- Modify: `crates/renderer/src/modern_variant_atlas.rs`
- Modify: `crates/renderer/src/modern_software.rs`
- Modify: `crates/renderer/src/modern_gpu.rs`
- Test: `cargo test -p renderer modern_variant_atlas`
- Test: `cargo test -p renderer variant_atlas_software`
- Test: `cargo test -p renderer modern_gpu`

**Interfaces:**
- Produces: `VariantAtlasDraw<'a>` with these cases:
  `Stable { entry, effect }`, `DynamicPalette { entry }`, `MissingArt`,
  and `Unkeyed`.
- Produces: `ModernVariantAtlas::resolve_draw(Option<&VariantAtlasKey>)`.
- Produces: `ModernVariantAtlas::effect_row_for_effect(&TileEffect)`.
- Consumes: existing `entry_for_source_key`, `effect_for_key`, and preview
  material matching.

- [x] **Step 1: Write failing resolver tests**

Add tests in `crates/renderer/src/modern_variant_atlas.rs`:

```rust
#[test]
fn resolve_draw_returns_live_effect_for_source_art() {
    let atlas = bg_test_atlas(0, vec![bg_test_effect_with_palette_row(3)]);
    let live_key = bg_test_key_with_palette_row(3);

    match atlas.resolve_draw(Some(&live_key)) {
        VariantAtlasDraw::Stable { entry, effect: Some(effect) } => {
            assert_eq!(entry.id, "bg:kBgGfx:pack0:tile0:3bpp");
            assert_eq!(effect.id, "palette_dung_bg_main:8color:row3");
        }
        other => panic!("expected stable effect-backed draw, got {other:?}"),
    }
}

#[test]
fn resolve_draw_keeps_unmodeled_material_on_dynamic_fallback() {
    let atlas = bg_test_atlas(0, Vec::new());
    let live_key = bg_test_key_with_palette_row(3);

    match atlas.resolve_draw(Some(&live_key)) {
        VariantAtlasDraw::DynamicPalette { entry } => {
            assert_eq!(entry.id, "bg:kBgGfx:pack0:tile0:3bpp");
        }
        other => panic!("expected dynamic fallback, got {other:?}"),
    }
}
```

Run:

```bash
cargo test -p renderer modern_variant_atlas -- --nocapture
```

Expected: fail because `VariantAtlasDraw` and `resolve_draw` do not exist.

- [x] **Step 2: Implement `VariantAtlasDraw` and resolver**

Add this API in `crates/renderer/src/modern_variant_atlas.rs`:

```rust
#[derive(Clone, Copy, Debug)]
pub enum VariantAtlasDraw<'a> {
    Stable {
        entry: &'a VariantAtlasEntry,
        effect: Option<&'a TileEffect>,
    },
    DynamicPalette {
        entry: &'a VariantAtlasEntry,
    },
    MissingArt,
    Unkeyed,
}
```

`resolve_draw(None)` returns `Unkeyed`. `resolve_draw(Some(key))` resolves the
entry by source key only. If no entry exists, it returns `MissingArt`. If the
entry is stable and a stable effect exists for the live key, it returns
`Stable { effect: Some(effect) }`. If the entry is stable and its preview
material exactly matches the live key, it returns `Stable { effect: None }`.
Otherwise it returns `DynamicPalette { entry }`.

Run:

```bash
cargo test -p renderer modern_variant_atlas -- --nocapture
```

Expected: pass.

- [x] **Step 3: Route software variant draws through the resolver**

Replace the local `entry_can_render_stable`, `entry_matches_material`, and
`key_has_stable_effect` helpers in `crates/renderer/src/modern_software.rs`
with one `match atlas.resolve_draw(key.as_ref())`. Pass the returned `effect`
directly into the BG and sprite variant draw helpers.

Run:

```bash
cargo test -p renderer variant_atlas_software -- --nocapture
```

Expected: pass, including
`variant_atlas_software_resolves_art_by_source_and_effect_by_live_palette`.

- [x] **Step 4: Route GPU variant draws through the resolver**

Replace the duplicated classification in `ModernGpuVariantRenderer::build_variant_frame`
and `ModernGpuVariantEffectRenderer::{render_bg, render_sprites}` with
`atlas.resolve_draw(Some(&key))`. The GPU effect renderer should only emit
effect instances for `VariantAtlasDraw::Stable { effect: Some(effect), .. }`.
Use `ModernVariantAtlas::effect_row_for_effect(effect)` to encode the LUT row.

Run:

```bash
cargo test -p renderer modern_gpu -- --nocapture
```

Expected: pass, including mixed fallback/effect overlay tests.

- [x] **Step 5: Commit**

```bash
git add crates/renderer/src/modern_variant_atlas.rs \
  crates/renderer/src/modern_software.rs \
  crates/renderer/src/modern_gpu.rs \
  docs/superpowers/plans/2026-07-03-modern-effect-drawing.md
git commit -m "refactor(renderer): share variant draw resolution"
```

### Task 6: Expose Draw-Mix Stats Around the Shared Resolver

**Files:**
- Modify: `crates/renderer/src/modern_software.rs`
- Modify: `crates/renderer/src/modern_gpu.rs`
- Modify: `zelda3-bin/src/main.rs`
- Modify: `scripts/gpu_render_compare_oracle_windows.py`
- Modify: `scripts/gpu_render_compare_windows.py`
- Modify: `scripts/test_gpu_render_compare_oracle_windows.py`
- Modify: `docs/assets/rgba-variant-atlas.md`
- Test: `cargo test -p renderer variant_atlas_software modern_gpu_variant_headless_mixed_fallback_uses_effect_overlay`
- Test: `python3 scripts/test_gpu_render_compare_oracle_windows.py`

**Goal:** Make live and oracle logs report the modern draw mix in source terms:
stable preview draws, stable effect draws, dynamic material fallback draws,
missing source-art draws, and unkeyed live draws. This replaces ambiguous
legacy wording where all non-preview draws could appear as generic fallback.

- [x] **Step 1: Add failing counter tests**

Add assertions for `stable_preview_draws`, `stable_effect_draws`,
`dynamic_material_draws`, `missing_art_draws`, and
`unkeyed_fallback_draws` in the software variant atlas and GPU variant
headless tests. Add a software regression test where source art exists but the
live material has no stable effect, proving it counts as
`dynamic_material_draws`.

Run:

```bash
cargo test -p renderer variant_atlas_software -- --nocapture
cargo test -p renderer modern_gpu_variant_headless -- --nocapture
```

Expected red result: compile failure because the new stats fields do not exist.

- [x] **Step 2: Implement resolver-driven stats**

Extend `VariantAtlasRenderStats` with the five source/material counters and a
single `record_draw(&VariantAtlasDraw)` helper. Route both the software oracle
and GPU variant build path through that helper so legacy and modern counters
are updated together.

- [x] **Step 3: Expose counters in live and oracle logs**

Append `stable_preview_draws`, `stable_effect_draws`,
`dynamic_material_draws`, `missing_art_draws`, and
`unkeyed_fallback_draws` to `variant_live_summary`,
`modern_index_compare`, and `modern_index_compare_summary` lines while keeping
the existing legacy field names.

- [x] **Step 4: Update compare wrappers and docs**

Update the oracle/window wrapper parsers to accept both old and new summary
formats, aggregate the new counters when present, and print them in wrapper
summaries. Document the counter meanings in `docs/assets/rgba-variant-atlas.md`.

### Task 7: Compile Variant Draws Into Backend-Neutral Draw Packets

**Files:**
- Create: `crates/renderer/src/modern_variant_draw.rs`
- Modify: `crates/renderer/src/lib.rs`
- Modify: `crates/renderer/src/modern_software.rs`
- Modify: `crates/renderer/src/modern_gpu.rs`
- Test: `cargo test -p renderer modern_variant_draw`

**Goal:** Build one backend-neutral list of source-art/effect/indexed-fallback
draw packets per `ModernFrame`. Software and GPU renderers consume the same
packet list, which is the next clean break from the old CPU composition shape.

### Task 8: Add A Route-Window Proof For Nonzero Variant Draws

**Files:**
- Modify: `scripts/gpu_render_compare_oracle_windows.py` only if stats parsing
  needs the Task 6 names.
- Modify: `docs/assets/rgba-variant-atlas.md`
- Test: `python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv --cold --limit 1`

**Goal:** Prove at least one representative oracle window has nonzero stable
source-art/effect draws and zero mismatched pixels. Keep this as a focused
window proof, not a full route scan.
