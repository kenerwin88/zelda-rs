# HD Art via ML Super-Resolution — Design

**Date:** 2026-07-02
**Status:** Approved (brainstorming)

## Goal

Produce real HD art by ML super-resolution and make it visible in-game through
the existing modern HD-override pipeline (Phase 1 + Phase 2), at integer scale N,
palette-responsive within a scene, and byte-identical to today when disabled.
First cut proves the whole chain end-to-end on one scene (Kakariko) at 4×.

## Non-goals / out of scope

- Full-atlas coverage (all ~thousands of cells). First cut is one scene.
- Per-scene palette manifests / a canonical global palette across all scenes.
- Luma-detail-only chroma preservation (a future palette-robustness refinement).
- Any change to the renderer or game logic. This is offline authoring only.
- Committing the HD assets or model weights (all gitignored).

## Background: what already exists (do not rebuild)

- **Override store & kernel** (`crates/renderer/src/modern_hd_overrides.rs`):
  - Manifest schema:
    ```json
    { "reference_palette": "reference_palette.png",
      "overrides": [ { "key": "0x<sourcekey>", "rgba": "cells/<key>.png" } ] }
    ```
  - `HdCell { width, height (multiples of 8), rgba: RGBA8 }`.
  - `detail_modulate(live, hd, reference) = clamp(live * (hd / max(reference,1)))`
    per channel; alpha from `live`. Art authored as `reference[idx]` → detail 1 →
    exact parity. Loaded via `ZELDA3_MODERN_HD_OVERRIDES=<manifest>`; unset/invalid
    → disabled (byte-identical). Bad individual entries are skipped, not fatal.
- **N× rendering** (Phase 2): `render_modern_frame_full_scaled(..., scale)` samples
  `HdCell` sub-pixel at N× via `sample_scaled`. Live wiring runs the sources+overrides
  path when `ZELDA3_RENDERER=assets-anim`.
- **Source keys**: `modern_source_key(kind, pack, tile_off)`; the atlas
  (`assets_by_source.json`) maps keys → kind/pack/tile_off.
- **Reference palette dump**: `--dump-reference-palette <frame> [out.png]` writes the
  live CGRAM as a 256×1 RGBA PNG (the authoring palette).
- **Placement data**: the modern renderer's tile instances already carry
  `source_key` + screen position (`screen_x/screen_y`) — the basis for the
  per-frame placement map, so no per-pixel winning-layer analysis is needed at
  BG-tile granularity.

## Architecture

Four offline stages producing gitignored assets under `hd_art/`, then a manual
in-game proof. The Rust game/render path is unchanged; new Rust code is
authoring-only subcommands.

```
[capture] frames + placement maps + reference palette
   → [super-resolve] torch Real-ESRGAN x4 (MPS)
   → [slice] per-key HD cells from SR'd frames
   → [assemble] manifest.json
   → [prove] run game with the manifest, eyeball HD
```

### Component 1 — Capture (Rust: `--dump-hd-capture`)

New `zelda3-bin` subcommand. Given a list of target frames (a scene), for each it
emits into `hd_art/capture/`:
- `frame_<n>.png` — native 256×224 modern-composited RGBA, colorized by live
  CGRAM (render via the sources path so positions match the placement map).
- `frame_<n>.map.json` — placement map: `[{ "key": "0x..", "x", "y", "w", "h" }]`
  in native screen pixels, one entry per drawn tile instance with a real source
  key (`key != NO_SOURCE_KEY`). Sourced from the modern renderer's tile
  instances. Off-screen-clipped instances are clamped or dropped (documented).
- `reference_palette.png` — 256×1 CGRAM at capture (reuse `--dump-reference-palette`
  logic). All captured frames in one run share ONE palette state (scene invariant).

Frame selection: the scene's frames come from the combined replay route; the
command takes explicit frame numbers (or a small named-scene table). First cut:
Kakariko, ~3–8 frames covering its visible cells.

### Component 2 — Super-resolve (Python: `scripts/hd_super_resolve.py`)

- torch 2.12 on MPS (already installed). Real-ESRGAN x4 (RRDBNet). Default to the
  illustration/anime-tuned weights (`RealESRGAN_x4plus_anime_6B`) which suit
  flat tile/sprite art better than the photo model; allow selecting the photo
  model via flag. Weights fetched once to a gitignored cache dir
  (`hd_art/models/`).
- Input: `hd_art/capture/frame_*.png` → output `hd_art/sr/frame_*.x4.png`
  (1024×896 at 4×). Scale factor is a CLI arg matching `ZELDA3_HD_SCALE`.
- Deterministic per run (no random seed in inference). Fails loudly if weights or
  torch/MPS are unavailable.

### Component 3 — Slice (Rust: `--slice-hd-cells`, or a small Python slicer)

For each `frame_<n>.map.json` entry, crop the matching `frame_<n>.x4.png` at
`(x·N, y·N, w·N, h·N)` → an N·w × N·h RGBA cell. Write `hd_art/cells/<key>.png`.
Dedup **keep-first** per key across frames in a deterministic frame/entry order
(mirrors the atlas dump's keep-first). Skip keys already written. Chosen home:
Rust subcommand (keeps PNG I/O and key formatting consistent with the existing
dumps); a Python slicer is an acceptable alternative if simpler.

### Component 4 — Assemble manifest (part of slice stage or a tiny step)

Emit `hd_art/manifest.json` with `reference_palette: "capture/reference_palette.png"`
(or a copy alongside the manifest) and one `overrides` entry per written cell:
`{ "key": "0x..", "rgba": "cells/<key>.png" }`. Paths are relative to the
manifest (as `load_manifest` expects).

### Component 5 — Prove (manual)

```
ZELDA3_RENDERER=assets-anim ZELDA3_HD_SCALE=4 \
  ZELDA3_MODERN_HD_OVERRIDES=hd_art/manifest.json \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav <kakariko-frame>
```
Expect: HD detail where cells have overrides, blocky nearest-upscale elsewhere;
without the env vars, display unchanged.

## Data flow & palette coherence

SR cells are colorized with the capture CGRAM, and the manifest's
`reference_palette` IS that CGRAM. So `detail = SR / reference` is structural
variation around 1, and `final = live · detail` re-lights that structure by the
live palette — coherent within one palette family. This is why the first cut is
scoped to a single scene/palette. Large palette swaps can hue-shift SR output;
the luma-detail-only refinement (preserve reference chroma, take SR luma) is the
documented future fix, out of scope here.

## Error handling

- Disabled path unchanged: no manifest → `ModernHdOverrides::from_env()` is
  `None` → renderer byte-identical.
- Bad/missing cell PNG → `load_manifest` skips it (already implemented).
- Missing model/torch/MPS → the Python script exits non-zero with guidance.
- Missing ROM/replay for capture → the Rust command exits with the expected path.
- All `hd_art/` outputs and model weights are gitignored.

## Testing

- **Placement map (Rust unit test):** a synthetic `ModernFrame` with a known tile
  instance at a known `(screen_x, screen_y)` and source key produces the expected
  `{key,x,y,w,h}` entry.
- **Slicer (Rust unit test):** a synthetic SR frame with a spatially-encoded
  pattern, sliced at a known `(x·N,y·N,w·N,h·N)`, yields exactly that region;
  keep-first dedup keeps the first occurrence.
- **Manifest load (Rust):** a generated proof manifest loads via
  `ModernHdOverrides::load_manifest` and resolves a known key to an `HdCell` of
  the expected dimensions.
- **Parity:** disabled-path byte-identity covered by the existing renderer suite
  (no renderer change here).
- **End-to-end:** the in-game proof is a documented manual visual check (no
  display in headless CI), not automated.

## Deliverables

- `zelda3-bin` subcommands: `--dump-hd-capture`, `--slice-hd-cells` (+ manifest).
- `scripts/hd_super_resolve.py`.
- `.gitignore`: `hd_art/`.
- A generated `hd_art/manifest.json` + cells for the Kakariko proof (local, gitignored).
- Docs: a short `hd_art/README.md` describing the 4-stage pipeline and the proof command.
