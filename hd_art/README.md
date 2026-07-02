# HD-art authoring pipeline (ML super-resolution)

This directory (`hd_art/`) is **gitignored** (see `.gitignore` — everything under it is
regenerable authoring output / downloaded model weights, never committed). Only this
README is tracked (force-added).

The pipeline turns the native 256×224 SNES-composited frame into hand-curated N×
super-resolved cell art, keyed by the same content-hash the renderer already uses to
identify each 8×8 BG/sprite source tile. The result is an override manifest the modern
renderer (`ZELDA3_RENDERER=assets-anim`) can load to swap in HD art for specific cells
while leaving every un-overridden cell rendering exactly as before (nearest-neighbor
upscale of the native pixels). None of this touches the render/parity path — with the
three env vars below unset, the display is byte-identical to today.

## Prerequisites

```bash
pip install realesrgan basicsr    # once; downloads Real-ESRGAN model weights on first run
cargo build --profile parity -p zelda3-bin
```

All replay runs below need the 7 timing-hack env vars (same as every other replay tool
in this repo):

```bash
HACKS=(
  ZELDA3_SMV_SELECT_FILE_TIMING_HACKS=1
  ZELDA3_SMV_LOADFILE_TIMING_HACKS=1
  ZELDA3_SMV_DUNGEON_TIMING_HACKS=1
  ZELDA3_SMV_OVERWORLD_TIMING_HACKS=1
  ZELDA3_SMV_MESSAGING_TIMING_HACKS=1
  ZELDA3_SMV_DEATH_INTRO_TIMING_HACKS=1
  ZELDA3_SMV_DEATH_RELOAD_TIMING_HACKS=1
)
```

## Constraint: one canonical palette per manifest

`--dump-hd-capture` writes `reference_palette.png` (a 256×1 RGBA CGRAM snapshot) from
the *first* frame it captures in that invocation, and every frame captured in the same
invocation is assumed to share that palette. The live override path
(`modern_hd_overrides.rs`) checks the *current* CGRAM against this reference at load
time and disables overrides on mismatch — so a manifest built from mixed scenes/palette
states (e.g. one frame from Kakariko, one from a dungeon) will not apply cleanly.

**Capture all frames for one manifest from a single scene/palette state.** First cut:
Kakariko village (overworld, one outdoor palette). Do a separate `--dump-hd-capture` +
`--slice-hd-cells` pass (into a separate output tree, or one at a time) per distinct
palette state you want to author.

## Finding representative frames (e.g. Kakariko)

Kakariko is an overworld scene; there's no frame index lookup, so locate candidate
frames by dumping a coarse sweep of the combined replay and eyeballing the PNGs:

```bash
for n in 15000 20000 25000 30000 35000 40000; do
  env "${HACKS[@]}" target/parity/zelda3 --dump-frame /tmp/f_$n.png $n
done
# open /tmp/f_*.png — look for the village (houses, well, dirt paths, torch/bush
# clusters). If none of the sweep lands in Kakariko, narrow/widen the range and
# re-sweep; the combined route does pass through it.
```

Record the frame numbers that land inside the village (ideally 2-3 frames spanning a
few seconds so the capture map covers more of the visible cell set) and use those as
`<k1> <k2> <k3>` below.

## Stage 1 — capture native frames + placement maps

```bash
env "${HACKS[@]}" target/parity/zelda3 --dump-hd-capture <k1> <k2> <k3>
```

Writes, for each requested frame `<n>`:
- `hd_art/capture/frame_<n>.png` — native 256×224 RGBA (scale 1, overrides disabled;
  this is the frame the super-resolution step ingests)
- `hd_art/capture/frame_<n>.map.json` — `Vec<HdPlacement>`, i.e. every on-screen source
  key with its screen rect (`x`, `y`, `w`, `h`)
- `hd_art/capture/reference_palette.png` — 256×1 RGBA CGRAM snapshot from the *first*
  captured frame (see the one-palette-per-manifest constraint above)

Mode-7 frames are skipped (the sources path doesn't cover Mode 7).

## Stage 2 — super-resolve

```bash
python3 scripts/hd_super_resolve.py --in hd_art/capture --out hd_art/sr \
  --scale 4 --model anime
```

Runs Real-ESRGAN (anime 6-block model by default; `--model photo` for the general
model) over every `hd_art/capture/frame_*.png` and writes
`hd_art/sr/frame_<n>.x<scale>.png`. Model weights download once to `hd_art/models/`
(gitignored — do not commit). Uses MPS/CUDA if available, else CPU.

### Optional: neural style transfer (make overrides OBVIOUSLY different art)

Add `--style <udnie|mosaic|candy|rain_princess>` to run a fast neural-style-transfer
pass on each super-resolved frame before it is written. The override cells then render
as a dramatically different art style in-game (the detail-modulate kernel treats the art
as detail vs the reference palette and re-lights it through live CGRAM, so the restyle
shows through while still tracking the live palette). Slice/manifest/proof stages are
unchanged.

```bash
# one-time: fetch the pretrained fast-neural-style weights into hd_art/models/
git clone --depth 1 https://github.com/pytorch/examples
python examples/fast_neural_style/download_saved_models.py
cp examples/fast_neural_style/saved_models/udnie.pth hd_art/models/

python3 scripts/hd_super_resolve.py --in hd_art/capture --out hd_art/sr \
  --scale 4 --model anime --style udnie
```

`--style-weights <path>` overrides the default `hd_art/models/<style>.pth` lookup.
Plumbing checks (no weights/model needed): `--self-test` (SR geometry, torch-free) and
`--self-test-style` (runs the TransformerNet forward pass on MPS/CPU to confirm the
style net loads and is size-preserving).

## Stage 3 — slice cells + build the manifest

```bash
target/parity/zelda3 --slice-hd-cells 4
```

For each `hd_art/capture/frame_<n>.map.json`, ascending by `<n>` (so the first frame a
source key appears in wins — deterministic keep-first if the same tile shows up in more
than one captured frame), crops the matching `hd_art/sr/frame_<n>.x<scale>.png` at each
placement's `(x, y, w, h)` rect scaled up by `<scale>`, and writes one PNG per unique
source key into `hd_art/cells/<key>.png`. Finishes by writing `hd_art/manifest.json`
referencing `capture/reference_palette.png` and every cell written.

The `<scale>` argument (default 4) **must match** the SR pass's `--scale` and the
`ZELDA3_HD_SCALE` used in stage 4. If `--slice-hd-cells` reports far fewer cells
written than placements captured, the SR frame dimensions don't match
`256*scale × 224*scale` — check that `--scale` and `ZELDA3_HD_SCALE` agree.

## Stage 4 — prove HD in-game (manual visual check)

```bash
env "${HACKS[@]}" ZELDA3_RENDERER=assets-anim ZELDA3_HD_SCALE=4 \
  ZELDA3_MODERN_HD_OVERRIDES=hd_art/manifest.json \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav <k1>
```

Confirm:
- Cells covered by the manifest render crisper (visible ML-upscaled detail), not the
  blocky nearest-neighbor baseline.
- Cells with no override still render via the nearest-neighbor N× fallback (no visual
  regression, just not HD yet).
- With **none** of `ZELDA3_RENDERER` / `ZELDA3_HD_SCALE` / `ZELDA3_MODERN_HD_OVERRIDES`
  set, the display is byte-identical to the default path (parity-safe — this pipeline
  is purely additive/opt-in).

Capture a before/after screenshot pair (with vs. without the three env vars, same
frame) as the proof artifact.

## Directory layout (all gitignored except this file)

```
hd_art/
  capture/                 stage 1 output (native frames + placement maps + reference palette)
  sr/                       stage 2 output (super-resolved frames)
  cells/                    stage 3 output (per-source-key HD PNGs)
  models/                   downloaded Real-ESRGAN weights (stage 2, cached)
  manifest.json             stage 3 output (consumed by stage 4 via ZELDA3_MODERN_HD_OVERRIDES)
  README.md                 this file (tracked)
```
