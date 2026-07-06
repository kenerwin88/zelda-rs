# Source Glyph PNG Route Proof

This note tracks evidence that BG3/message glyph pixels are rendered from source glyph PNGs instead of dynamic BG3 content-hash chunks.

## Current Gate

`ModernAssetGpuReadbackRenderer::validate_game_full_gpu_path` now rejects visible BG3 dynamic/unkeyed text chunks after source-glyph extraction. The gate intentionally ignores:

- BG3 wraparound tiles fully outside the 256x224 viewport.
- Bulk repeated BG3 fill tiles, which are not sparse glyph/text chunks.
- Forced-blank or brightness-zero frames, where BG3 tile data cannot contribute visible pixels.

The gate runs inside replay `--asset-gpu-smoke`, so route scans fail at the first visible BG3 text chunk that is not owned by `dialogue_glyph_tiles.png`, `dialogue_vwf_glyphs.png`, or `dialogue_font_tiles.png`.

## Evidence

Baseline command run from repo root:

```sh
env \
  ZELDA3_SMV_SUPPRESS_SLOW_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPURIOUS_SELECT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPURIOUS_LATE_SELECT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_FLASHING_FAIRY_HACK=1 \
  ZELDA3_SMV_SUPPRESS_POSITION_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPRITE_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_UNCLE_DRIFT_HACK=1 \
  target/debug/zelda3 \
  --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 50000 \
  --asset-gpu-smoke \
  --asset-gpu-progress 5000 \
  --missing-assets-out /tmp/zelda3-bg3-dynamic-report.jsonl \
  --asset-gpu-checkpoint-dir /tmp/zelda3-asset-gpu-checkpoints \
  --asset-gpu-checkpoint-interval 10000
```

Result:

```text
replay-save asset GPU smoke passed frames=50000 ... validation_cache_hits=8860; validation_cache_misses=41140; validation_cache_entries=41140
```

Checkpoint artifacts were written at 10k-frame intervals under `/tmp/zelda3-asset-gpu-checkpoints`.

The 50k checkpoint then exposed two source-glyph gate issues past the initial slice:

- Frame 58,724 failed on visible choice text from message `0x0002` (`What do you say? Interested?`). The message choice redraw reset VWF cursor state and incorrectly discarded glyph-run ownership while the old choice pixels were still visible. Full message initialization now clears ownership; choice redraws only reset the cursor state.
- Frame 72,931 failed on five residual BG3 chunks during a black transition frame (`textrs=0x04`). The gate now skips forced-blank and brightness-zero frames because their BG3 tile data cannot affect final visible pixels.

Continuation command:

```sh
env \
  ZELDA3_SMV_SUPPRESS_SLOW_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPURIOUS_SELECT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPURIOUS_LATE_SELECT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_FLASHING_FAIRY_HACK=1 \
  ZELDA3_SMV_SUPPRESS_POSITION_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPRITE_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_UNCLE_DRIFT_HACK=1 \
  cargo run -p zelda3-bin -- \
  --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 100000 \
  --load-state /tmp/zelda3-asset-gpu-checkpoints/asset-gpu-frame-000070000.sav \
  --asset-gpu-smoke \
  --asset-gpu-progress 10000 \
  --missing-assets-out /tmp/zelda3-bg3-dynamic-report-100k-fix2.jsonl \
  --asset-gpu-checkpoint-dir /tmp/zelda3-asset-gpu-checkpoints \
  --asset-gpu-checkpoint-interval 10000
```

Result:

```text
replay-save asset GPU smoke passed frames=100000 ... validation_cache_hits=6673; validation_cache_misses=23327; validation_cache_entries=23327
```

Further continuation from the 100k checkpoint to 200k used the same gate:

```sh
env \
  ZELDA3_SMV_SUPPRESS_SLOW_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPURIOUS_SELECT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPURIOUS_LATE_SELECT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_FLASHING_FAIRY_HACK=1 \
  ZELDA3_SMV_SUPPRESS_POSITION_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPRITE_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_UNCLE_DRIFT_HACK=1 \
  target/debug/zelda3 \
  --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 200000 \
  --load-state /tmp/zelda3-asset-gpu-checkpoints/asset-gpu-frame-000100000.sav \
  --asset-gpu-smoke \
  --asset-gpu-progress 10000 \
  --missing-assets-out /tmp/zelda3-bg3-dynamic-report-200k.jsonl \
  --asset-gpu-checkpoint-dir /tmp/zelda3-asset-gpu-checkpoints \
  --asset-gpu-checkpoint-interval 10000
```

Result:

```text
replay-save asset GPU smoke passed frames=200000 ... validation_cache_hits=16240; validation_cache_misses=83760; validation_cache_entries=83760
```

Further continuation from the 200k checkpoint to 300k also passed:

```text
replay-save asset GPU smoke passed frames=300000 ... validation_cache_hits=24429; validation_cache_misses=75571; validation_cache_entries=75571
```

Further continuation from the 300k checkpoint to 400k also passed:

```text
replay-save asset GPU smoke passed frames=400000 ... validation_cache_hits=17915; validation_cache_misses=82085; validation_cache_entries=82085
```

Further continuation from the 400k checkpoint to 500k also passed:

```text
replay-save asset GPU smoke passed frames=500000 ... validation_cache_hits=19898; validation_cache_misses=80102; validation_cache_entries=80102
```

Further continuation from the 500k checkpoint to 600k also passed:

```text
replay-save asset GPU smoke passed frames=600000 ... validation_cache_hits=26268; validation_cache_misses=73732; validation_cache_entries=73732
```

Further continuation from the 600k checkpoint to 700k also passed:

```text
replay-save asset GPU smoke passed frames=700000 ... validation_cache_hits=12132; validation_cache_misses=87868; validation_cache_entries=87868
```

The 700k checkpoint landed during active dialogue (`msgmod=1`, `textrs=0x03`). A resume from the original checkpoint failed at frame 700,001 because the C-style replay checkpoint did not preserve the transient BG3 VWF glyph-run ownership sidecar. Replay-save checkpoint trailers now include `bg3_vwf_glyph_runs`, and an explicit load of the regenerated 700k checkpoint passed frame 700,001:

```text
replay-save asset GPU smoke passed frames=700001 ... validation_cache_hits=0; validation_cache_misses=1; validation_cache_entries=1
```

Further continuation from the regenerated 700k checkpoint to 800k also passed:

```text
replay-save asset GPU smoke passed frames=800000 ... validation_cache_hits=18329; validation_cache_misses=81671; validation_cache_entries=81671
```

Further continuation from the 800k checkpoint to 900k also passed:

```text
replay-save asset GPU smoke passed frames=900000 ... validation_cache_hits=18590; validation_cache_misses=81410; validation_cache_entries=81410
```

Further continuation from the 900k checkpoint to 1M also passed:

```text
replay-save asset GPU smoke passed frames=1000000 ... validation_cache_hits=15023; validation_cache_misses=84977; validation_cache_entries=84977
```

Further continuation exposed ending credits text at frame 1,032,993 (`THE RETURN OF THE KING` / `HYRULE CASTLE`). That text is not VWF overlay data or the `1w-2d` glyph sheet; the ending code copies the raw `kDialogueFont` CHR payload to BG3 at VRAM `0x7000` and writes the credits tilemap directly. The asset pipeline now parses the nested `kDialogueFont` memblk payload and emits `dialogue_font_tiles.png/json` as the PNG source for those 256 raw 2bpp font tiles.

Focused proof from the 1,030,000 checkpoint through the failing frame now passes:

```sh
env \
  ZELDA3_SMV_SUPPRESS_SLOW_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPURIOUS_SELECT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPURIOUS_LATE_SELECT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_FLASHING_FAIRY_HACK=1 \
  ZELDA3_SMV_SUPPRESS_POSITION_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPRITE_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_UNCLE_DRIFT_HACK=1 \
  target/debug/zelda3 \
  --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 1032993 \
  --load-state /tmp/zelda3-asset-gpu-checkpoints/asset-gpu-frame-001030000.sav \
  --asset-gpu-smoke \
  --asset-gpu-progress 1000 \
  --missing-assets-out /tmp/zelda3-bg3-dynamic-report-1032993-font-payload.jsonl
```

Result:

```text
replay-save asset GPU smoke passed frames=1032993 ...
replay-save completed frames=1032993 active=true ending=1 ...
```

The full continuation from the 1,030,000 checkpoint used a deliberately higher frame cap. The replay ended naturally before that cap:

```sh
env \
  ZELDA3_SMV_SUPPRESS_SLOW_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPURIOUS_SELECT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPURIOUS_LATE_SELECT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_FLASHING_FAIRY_HACK=1 \
  ZELDA3_SMV_SUPPRESS_POSITION_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_SPRITE_DRIFT_HACK=1 \
  ZELDA3_SMV_SUPPRESS_UNCLE_DRIFT_HACK=1 \
  target/debug/zelda3 \
  --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 1100000 \
  --load-state /tmp/zelda3-asset-gpu-checkpoints/asset-gpu-frame-001030000.sav \
  --asset-gpu-smoke \
  --asset-gpu-progress 10000 \
  --missing-assets-out /tmp/zelda3-bg3-dynamic-report-1100000-font-payload.jsonl \
  --asset-gpu-checkpoint-dir /tmp/zelda3-asset-gpu-checkpoints \
  --asset-gpu-checkpoint-interval 10000
```

Result:

```text
replay-save asset GPU smoke passed frames=1073092 ... validation_cache_hits=15416; validation_cache_misses=27676; validation_cache_entries=27676
replay-save completed frames=1073092 active=false ending=1 ... main=26 sub=38 ...
```

This run wrote continuation checkpoints through `/tmp/zelda3-asset-gpu-checkpoints/asset-gpu-frame-001070000.sav`.

## Status

This proves the source-glyph PNG gate over the full `saves/zelda3-combined-route.sav` replay. The route ends at frame 1,073,092, so the 1,100,000-frame cap was intentionally beyond the actual path length.

Visible BG3 message, VWF, and ending-credit font chunks are now source-owned by PNG atlases rather than dynamic BG3 content-hash chunks.
