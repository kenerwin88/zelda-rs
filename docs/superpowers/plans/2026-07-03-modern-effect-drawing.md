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
`tile_effects.json` has a stable LUT; frames with no fallback draws use
effect-backed stable cells through the LUT shader; mixed fallback/effect frames
now keep the full compositor result until packet visibility is modeled; and live
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

Expected: pass, including mixed fallback/effect tests.

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
- Test: `cargo test -p renderer variant_atlas_software modern_gpu_variant_headless_mixed_fallback_keeps_compositor_pixels`
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
- [x] Create: `crates/renderer/src/modern_variant_draw.rs`
- [x] Modify: `crates/renderer/src/lib.rs`
- [x] Modify: `crates/renderer/src/modern_software.rs`
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Test: `cargo test -p renderer modern_variant_draw -- --nocapture`
- [x] Test: `cargo test -p renderer variant_atlas_software -- --nocapture`
- [x] Test: `cargo test -p renderer modern_gpu_variant_headless -- --nocapture`

**Goal:** Build one backend-neutral list of source-art/effect/indexed-fallback
draw packets per `ModernFrame`. Software and GPU renderers consume the same
packet list, which is the next clean break from the old CPU composition shape.

**Done:** `modern_variant_draw::compile_variant_draws` now emits BG and sprite
draw packets plus shared `VariantAtlasRenderStats`. The software variant path,
live GPU variant path, GPU stable-overlay frame builder, and GPU effect overlay
all consume that plan instead of independently resolving atlas keys and draw
policies.

### Task 8: Add A Route-Window Proof For Nonzero Variant Draws

**Files:**
- [x] Modify: `scripts/gpu_render_compare_oracle_windows.py`
- [x] Modify: `scripts/test_gpu_render_compare_oracle_windows.py`
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Modify: `docs/assets/rgba-variant-atlas.md`
- [x] Test: `python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv --only opening-uncle-dismiss-and-move --fast --frames 1000 --stride 60 --require-stable-draws --progress-every 0 --release`

**Goal:** Prove at least one representative oracle window has nonzero stable
source-art/effect draws and zero mismatched pixels. Keep this as a focused
window proof, not a full route scan.

**Done:** The oracle-window wrapper now supports `--limit`, per-window
`--frames`, and `--require-stable-draws`. The focused proof command above
resumes from `opening-uncle-dismiss-and-move` frame `28610`, samples 17
comparison frames over a 1000-frame tail at stride 60, and reports
`mismatched_pixels=0`, `variant_draws=21038`,
`stable_effect_draws=21038`, and `unkeyed_fallback_draws=133112`.

### Task 9: Re-enable Provably Safe Mixed Fallback BG Effect Packets

**Files:**
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Modify: `docs/assets/rgba-variant-atlas.md`
- [x] Test: `cargo test -p renderer mixed -- --nocapture`
- [x] Test: `python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv --only opening-uncle-dismiss-and-move --fast --frames 1000 --stride 60 --require-stable-draws --progress-every 0 --release`

**Goal:** Move beyond the all-or-nothing mixed fallback guard by drawing only
the stable BG effect packets that are provably safe: simple frame composition,
effect LUT equals live CGRAM for all nonzero source indices, and the packet
footprint is disjoint from every other BG/OBJ packet.

**Done:** Mixed fallback frames now call `mixed_variant_overlay_bg_packets`.
The selector returns only CGRAM-matching, footprint-disjoint BG effect packets
for simple frames. Overlapping packets, palette/effect mismatches, sprites, and
frames with color math/window/mosaic state continue to use the fallback pixels.

### Task 10: Expose Actual Mixed Overlay Counts

**Files:**
- [x] Modify: `crates/renderer/src/modern_software.rs`
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Modify: `zelda3-bin/src/main.rs`
- [x] Modify: `scripts/gpu_render_compare_oracle_windows.py`
- [x] Modify: `scripts/gpu_render_compare_windows.py`
- [x] Modify: `scripts/test_gpu_render_compare_oracle_windows.py`
- [x] Modify: `docs/assets/rgba-variant-atlas.md`
- [x] Test: `python3 scripts/test_gpu_render_compare_oracle_windows.py`
- [x] Test: `cargo test -p renderer mixed_variant_overlay_selects_only_cgram_matching_disjoint_effect_bg_packets -- --nocapture`
- [x] Test: `python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv --only opening-uncle-dismiss-and-move --fast --frames 1000 --stride 60 --require-stable-draws --progress-every 0 --release`

**Goal:** Separate stable effect opportunities from actual mixed-frame overlay
execution, so the next visibility work can prove it is drawing more packets
rather than only keeping parity.

**Done:** `VariantAtlasRenderStats` now carries
`mixed_overlay_bg_effect_draws`, live/replay logs print it, and both compare
wrappers parse and aggregate it. The representative route proof remains
`mismatched_pixels=0` and reports `mixed_overlay_bg_effect_draws=0`, which
shows the current safe selector is parity-clean but too conservative for that
route tail.

### Task 11: Report Why Mixed Overlay Packets Are Rejected

**Files:**
- [x] Modify: `crates/renderer/src/modern_software.rs`
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Modify: `zelda3-bin/src/main.rs`
- [x] Modify: `scripts/gpu_render_compare_oracle_windows.py`
- [x] Modify: `scripts/gpu_render_compare_windows.py`
- [x] Modify: `scripts/test_gpu_render_compare_oracle_windows.py`
- [x] Modify: `docs/assets/rgba-variant-atlas.md`
- [x] Test: `python3 scripts/test_gpu_render_compare_oracle_windows.py`
- [x] Test: `cargo test -p renderer mixed_variant_overlay_selects_only_cgram_matching_disjoint_effect_bg_packets -- --nocapture`
- [x] Test: `python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv --only opening-uncle-dismiss-and-move --fast --frames 1000 --stride 60 --require-stable-draws --progress-every 0 --release`

**Goal:** Turn `mixed_overlay_bg_effect_draws=0` from a dead end into an
actionable blocker report by counting stable BG effect candidates and the
specific guard that rejected them.

**Done:** Mixed overlay stats now include total candidates plus complex-frame,
CGRAM-mismatch, and overlap rejection buckets in live, replay, and wrapper
summaries. The focused proof remains `mismatched_pixels=0` and reports
`mixed_overlay_bg_effect_candidates=20674`,
`mixed_overlay_bg_effect_reject_complex_frame=20674`,
`mixed_overlay_bg_effect_reject_cgram_mismatch=0`, and
`mixed_overlay_bg_effect_reject_overlap=0`. The next best modernization step is
therefore to replace the broad complex-frame guard with explicit modeled
composition state for these sampled mixed frames.

### Task 12: Draw Composition-Safe Mixed BG Packets With Live CGRAM

**Files:**
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Modify: `crates/renderer/src/modern_variant_atlas.rs`
- [x] Modify: `docs/assets/rgba-variant-atlas.md`
- [x] Test: `cargo test -p renderer mixed -- --nocapture`
- [x] Test: `cargo test -p renderer modern_variant_atlas -- --nocapture`
- [x] Test: `python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv --only opening-uncle-dismiss-and-move --fast --frames 1000 --stride 60 --require-stable-draws --progress-every 0 --release`

**Goal:** Convert the proven stable BG opportunities from counted-only to
actually drawn by the GPU overlay path while preserving final-pixel parity.

**Done:** The mixed overlay selector now proves safety per packet pixel instead
of using the old whole-frame guard, prefers runtime 16-color CGRAM-stride
effects for BG/OBJ material lookup, falls back to a per-frame live-CGRAM LUT
when static effects do not match the current palette, and rejects overlap only
when opaque packet pixels collide. The focused proof remains
`mismatched_pixels=0` and now reports
`mixed_overlay_bg_effect_draws=17272`,
`mixed_overlay_bg_effect_candidates=20674`,
`mixed_overlay_bg_effect_reject_complex_frame=3402`,
`mixed_overlay_bg_effect_reject_cgram_mismatch=0`, and
`mixed_overlay_bg_effect_reject_overlap=0`. The next modernization target is
the remaining explicit composition-state rejects.

### Task 13: Split Remaining Complex Rejects Into Actionable Subreasons

**Files:**
- [x] Modify: `crates/renderer/src/modern_software.rs`
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Modify: `zelda3-bin/src/main.rs`
- [x] Modify: `scripts/gpu_render_compare_oracle_windows.py`
- [x] Modify: `scripts/gpu_render_compare_windows.py`
- [x] Modify: `scripts/test_gpu_render_compare_oracle_windows.py`
- [x] Modify: `docs/assets/rgba-variant-atlas.md`
- [x] Test: `python3 scripts/test_gpu_render_compare_oracle_windows.py`
- [x] Test: `cargo test -p renderer mixed -- --nocapture`
- [x] Test: `python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv --only opening-uncle-dismiss-and-move --fast --frames 1000 --stride 60 --require-stable-draws --progress-every 0 --release`

**Goal:** Make the last mixed-overlay blocker concrete enough to drive the next
renderer change without another broad scan.

**Done:** `reject_complex_frame` is now split into brightness, invalid-layer,
mosaic, sub-window, effect-bounds, per-scanline main-screen visibility,
layer-window, color-math, and color-math subtype counters across live/replay
logs and both compare wrappers. The focused proof remains
`mismatched_pixels=0` and keeps `mixed_overlay_bg_effect_draws=17272`. The
remaining `mixed_overlay_bg_effect_reject_complex_frame=3402` splits into
`mixed_overlay_bg_effect_reject_complex_scanline_main=1541` and
`mixed_overlay_bg_effect_reject_complex_color_math=1861`; the color-math
rejects are all
`mixed_overlay_bg_effect_reject_complex_color_math_subscreen=1861`, with
`mixed_overlay_bg_effect_reject_complex_color_math_clip=0` and
`mixed_overlay_bg_effect_reject_complex_color_math_fixed_color=0`. The next
implementation target is therefore sub-screen-aware variant composition, not a
fixed-color-only shader. Scanline-main rejects are not visible on the main
screen and should remain non-drawn unless packet visibility is represented more
precisely.

### Task 14: Move Sub-Screen Color Math Into the Variant GPU Path

**Files:**
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Test: focused renderer unit covering a static sub-screen math packet
- [x] Test: `python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv --only opening-uncle-dismiss-and-move --fast --frames 1000 --stride 60 --require-stable-draws --progress-every 0 --release`

**Goal:** Let stable variant BG effect packets participate in the same
main/sub/final color-math resolve used by the non-variant GPU renderer, so the
variant path can draw the 1,861 currently rejected sub-screen-dependent packets
without losing final-pixel parity.

**Approach:** Stop treating sub-screen color math as a late-RGBA overlay problem.
Instead, render accepted variant effect packets into a pre-final main-screen
intermediate with the same layer-bit alpha contract used by
`post_process.wgsl`, render or reuse a matching sub-screen intermediate, and run
the finalizer/post-process after those packets are present. Keep fixed-color and
clip subtype counters in place so future routes can prove whether additional
math cases remain after sub-screen support lands.

**Done:** The mixed headless variant path now has a pre-final lane for static
variant-effect BG packets that need color math. It patches those packet pixels
into the packed main-screen buffer with the correct layer math bit, then runs
the existing GPU finalizer so sub-screen/fixed-color/clip math happens in the
same pass as the parity renderer. The old final-RGBA overlay path remains in
place for packets that do not need color math. The focused unit
`modern_gpu_variant_headless_applies_subscreen_math_to_mixed_effect_bg` proves
a static effect packet finalizes with the sub-screen operand.

**Route result:** The opening tail remains `mismatched_pixels=0`, but the
representative route counters stay at
`mixed_overlay_bg_effect_reject_complex_color_math_subscreen=1861`. Debugging
showed those route packets are live-CGRAM fallback packets, not static
variant-effect packets. Re-overlaying live-CGRAM packets in the pre-final lane
can overwrite already-correct fallback pixels, so they intentionally remain
rejected/fallback until the renderer has a native live-CGRAM pre-final draw
path.

### Task 15: Add Native Live-CGRAM Pre-Final Drawing

**Files:**
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Test: focused renderer unit covering a live-CGRAM sub-screen packet
- [x] Test: `python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv --only opening-uncle-dismiss-and-move --fast --frames 1000 --stride 60 --require-stable-draws --progress-every 0 --release`

**Goal:** Reduce the 1,861 representative route sub-screen rejects by drawing
live-CGRAM BG packets into the same pre-final composition space without
disturbing fallback pixels or reusing the final-RGBA overlay assumptions.

**Approach:** Build a dedicated pre-final live-CGRAM lane instead of reusing the
current late overlay transform. It must match the fallback compositor's source
orientation and palette lookup exactly, write packed 5-bit RGB plus layer math
bit, and only claim draws after the focused route counters drop with zero
mismatched pixels.

**Done:** The packed pre-final overlay now has a native live-CGRAM lane. Unlike
the late overlay path, it samples live BG packet indices exactly like the
fallback compositor (`cell.indices[y*8+x]`) and resolves color from
`frame.cgram_rgba[palette*16+index]`, then writes packed 5-bit RGB plus the BG
layer math bit before the GPU finalizer runs. The focused unit
`modern_gpu_variant_headless_applies_subscreen_math_to_mixed_live_cgram_bg`
guards this behavior with a flipped source cell to prevent accidentally using
the static atlas/source-flip transform.

**Route result:** The opening tail remains `mismatched_pixels=0`. The focused
route now reports `mixed_overlay_bg_effect_draws=18223`,
`mixed_overlay_bg_effect_reject_complex_frame=2451`,
`mixed_overlay_bg_effect_reject_complex_scanline_main=1541`, and
`mixed_overlay_bg_effect_reject_complex_color_math_subscreen=910`. This moves
951 additional mixed BG effect packets onto the GPU path while keeping
final-pixel parity.

### Task 16: Classify the Remaining Sub-Screen Rejects

**Files:**
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Modify: `zelda3-bin/src/main.rs`
- [x] Modify: `scripts/gpu_render_compare_oracle_windows.py`
- [x] Modify: `scripts/gpu_render_compare_windows.py`
- [x] Test: focused renderer unit for the next remaining subtype
- [x] Test: focused opening-route oracle window

**Goal:** Explain and reduce the remaining 910 sub-screen color-math rejects
without guessing. These packets are no longer the obvious static-effect or
live-CGRAM cases covered by Tasks 14 and 15, so the next step is to split the
remaining reject path into actionable reasons before adding another renderer
lane.

**Done:** Added pre-final color-math reject counters for CGRAM mismatch and
packet overlap, threaded them through renderer stats, `variant_live_summary`,
`modern_index_compare` lines, both compare wrappers, and the parser tests. The
focused unit
`modern_gpu_variant_headless_counts_prefinal_overlap_color_math_reject` guards
that a sub-screen color-math packet which reaches the pre-final policy but
overlaps another packet is counted in
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap`.

**Route result:** The opening tail remains `mismatched_pixels=0`. The route now
reports `mixed_overlay_bg_effect_reject_complex_color_math_subscreen=910`,
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch=0`,
and `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap=910`.
That makes the next best modernization target explicit: handle overlapped
pre-final BG effect packets in the packed composition buffer before GPU final
color math, rather than building another palette or fixed-color lane.

### Task 17: Accept Behind-Only Pre-Final BG Overlap

**Files:**
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Test: focused renderer unit covering a pre-final BG packet over a behind
  BG overlap
- [x] Test: focused opening-route oracle window

**Goal:** Start reducing the pre-final overlap blocker without giving up the
fallback compositor as the oracle. The first safe case is a variant BG packet
whose overlapping BG pixels are all behind it in Mode-1 painter order; replacing
the packed main-screen pixel before final color math preserves the same winning
visible source. OBJ overlap remains rejected.

**Done:** The pre-final selector now uses Mode-1 BG order for color-math
pre-final packets. Behind-only BG overlap is allowed, while same-rank later
tiles, front BG layers, and OBJ pixels remain unsafe. The focused unit
`modern_gpu_variant_headless_applies_prefinal_bg_over_behind_overlap` guards the
new path by proving the fallback pixel is replaced before sub-screen addition.
`modern_gpu_variant_headless_counts_prefinal_overlap_color_math_reject` still
guards the front/same-order reject case.

**Route result:** The opening tail remains `mismatched_pixels=0`, but the route
still reports `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap=910`.
That means the representative 910 rejects are not simple behind-BG overlap; the
next useful step is to split the overlap bucket into front/same-order BG versus
OBJ overlap and then handle fully eligible overlapping BG groups in Mode-1 draw
order.

### Task 18: Split Pre-Final Overlap by BG vs OBJ

**Files:**
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Modify: `crates/renderer/src/modern_software.rs`
- [x] Modify: `zelda3-bin/src/main.rs`
- [x] Modify: `scripts/gpu_render_compare_oracle_windows.py`
- [x] Modify: `scripts/gpu_render_compare_windows.py`
- [x] Test: focused renderer units for BG-order and OBJ overlap reasons
- [x] Test: parser summary regex
- [x] Test: focused opening-route oracle window

**Goal:** Make the remaining pre-final overlap bucket actionable. The previous
route proof showed 910 overlapping sub-screen color-math packets, but not
whether the blocker was BG ordering or OBJ composition.

**Done:** The pre-final overlap classifier now records BG and OBJ subreasons:
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg` and
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj`.
Renderer stats, live summaries, compare summaries, oracle-window wrappers, and
parser tests all carry the split. The focused units guard both a front/same-order
BG reject and an OBJ reject while keeping the behind-only overlap test green.

**Route result:** The opening tail remains `mismatched_pixels=0`. The remaining
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap=910` splits
into `prefinal_overlap_bg=703` and `prefinal_overlap_obj=207`. The next build
target should be an ordered pre-final BG group path; it covers most remaining
overlap rejects and avoids the harder OBJ interaction until BG ordering is
modeled.

### Task 19: Admit Terminal Ordered Pre-Final BG Groups

**Files:**
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Test: focused renderer unit covering a terminal two-packet BG group
- [x] Test: focused opening-route oracle window

**Goal:** Start the ordered BG group path without guessing across deeper chains.
The first safe group case is a behind packet overlapped by a front/same-order BG
packet that is itself representable in the pre-final path and has no further
front BG at that pixel. Drawing both packets in existing packet order preserves
the winning visible pixel while allowing the hidden behind packet to move off
the fallback-only path.

**Done:** The pre-final BG overlap predicate now allows a front/same-order BG
overlap when the front packet can cover the pixel as a terminal pre-final group
member. The focused unit
`modern_gpu_variant_headless_applies_ordered_prefinal_bg_group` proves that both
packets are counted as GPU draws and that final sub-screen color math remains
pixel-exact. Existing tests still guard the simple behind-only path, the
front/same-order reject, and OBJ overlap rejection.

**Route result:** The opening tail remains `mismatched_pixels=0`, but the route
counters stay at `prefinal_overlap=910`, `prefinal_overlap_bg=703`, and
`prefinal_overlap_obj=207`. The representative BG overlaps are therefore not
terminal two-packet groups; the next useful step is to split `prefinal_overlap_bg`
into terminal, deeper-chain, unrepresentable-front, and mixed static/live order
subreasons before broadening the group renderer.

### Task 20: Split Remaining Pre-Final BG Overlap Reasons

**Files:**
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Modify: `crates/renderer/src/modern_software.rs`
- [x] Modify: `zelda3-bin/src/main.rs`
- [x] Modify: `scripts/gpu_render_compare_oracle_windows.py`
- [x] Modify: `scripts/gpu_render_compare_windows.py`
- [x] Test: focused renderer units for deeper-chain and mixed static/live order
- [x] Test: parser summary regex
- [x] Test: focused opening-route oracle window

**Goal:** Turn the 703 remaining BG overlap rejects into implementation targets
instead of running broader scans. The classifier separates front-BG blockers
into deeper chains, currently unrepresentable front packets, and mixed
static/live ordering cases that the current two-batch overlay would draw in the
wrong order.

**Done:** The pre-final BG overlap classifier now records
`prefinal_overlap_bg_deeper_chain`,
`prefinal_overlap_bg_unrepresentable_front`, and
`prefinal_overlap_bg_mixed_static_live_order` while preserving the existing BG,
OBJ, and total overlap counters. The renderer tests guard the new deeper-chain
and mixed static/live paths, and the compare wrappers parse and summarize the
new fields.

**Route result:** The opening tail remains `mismatched_pixels=0`. The BG split
is decisive: `prefinal_overlap_bg=703`,
`prefinal_overlap_bg_deeper_chain=0`,
`prefinal_overlap_bg_unrepresentable_front=703`, and
`prefinal_overlap_bg_mixed_static_live_order=0`. The next build target is the
front packet representation gap, not deeper group ordering or mixed static/live
batch ordering.

### Task 21: Split Unrepresentable Front BG Packets

**Files:**
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Modify: `crates/renderer/src/modern_software.rs`
- [x] Modify: `zelda3-bin/src/main.rs`
- [x] Modify: `scripts/gpu_render_compare_oracle_windows.py`
- [x] Modify: `scripts/gpu_render_compare_windows.py`
- [x] Test: focused renderer units for no-effect and cgram-mismatch
  representation reasons
- [x] Test: parser summary regex
- [x] Test: focused opening-route oracle window

**Goal:** Stop treating `prefinal_overlap_bg_unrepresentable_front` as one
opaque blocker. The next renderer work should be driven by whether the front BG
packet lacks a stable effect, is blocked by complex composition state, or cannot
map to static/live cgram.

**Done:** The pre-final BG material classifier now returns exact reject reasons
for front/same-order BG packets and the overlap stats expose
`prefinal_overlap_bg_unrepresentable_front_no_effect`,
`prefinal_overlap_bg_unrepresentable_front_complex`, and
`prefinal_overlap_bg_unrepresentable_front_cgram_mismatch`. The route parsers,
live summaries, and compare summaries carry those fields. Focused renderer tests
cover the route-level no-effect case and the direct cgram-mismatch classifier.

**Route result:** The opening tail remains `mismatched_pixels=0`. The 703
front-packet representation blockers split into
`prefinal_overlap_bg_unrepresentable_front_no_effect=39`,
`prefinal_overlap_bg_unrepresentable_front_complex=664`, and
`prefinal_overlap_bg_unrepresentable_front_cgram_mismatch=0`. The next useful
drawing modernization is pre-final support for front BG packets that are stable
effects but currently fail the complex-frame guards, not palette/cgram storage
or broader route scans.

### Task 22: Ignore Invisible Front BG Pixels In Pre-Final Groups

**Files:**
- [x] Modify: `crates/renderer/src/modern_gpu.rs`
- [x] Modify: `docs/assets/rgba-variant-atlas.md`
- [x] Test: focused renderer unit for a scanline-disabled front BG overlap
- [x] Test: focused opening-route oracle window

**Goal:** Convert the front-BG complex bucket into real GPU draws where the
front/same-order BG packet is complex only because it is not visible at the
overlapped main-screen pixel. Such a pixel should not block the behind packet's
pre-final overlay.

**Done:** The pre-final BG overlap check now skips front/same-order BG packets
whose pixel is disabled by raw main-screen enable, per-scanline main-screen
masking, or layer window masking. The new renderer unit proves a BG2 target can
still be promoted when an overlapping BG1 packet is masked off for that
scanline, while the masked front packet remains counted as its own
`scanline_main` complex reject.

**Route result:** The opening tail remains `mismatched_pixels=0`. GPU overlay
draws rise from `18223` to `18783`. The pre-final BG overlap blocker is gone:
`prefinal_overlap_bg=0`, `prefinal_overlap_bg_unrepresentable_front=0`, and
`prefinal_overlap_bg_unrepresentable_front_complex=0`. Remaining color-math
pre-final overlap is now all OBJ overlap:
`prefinal_overlap=350`, `prefinal_overlap_obj=350`. The next useful drawing
modernization lanes are primary candidate `scanline_main=1541` and
OBJ-aware pre-final composition.
