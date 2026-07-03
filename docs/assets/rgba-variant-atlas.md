# Canonical Art and Effect Atlas Contract

The canonical art/effect atlas is the bridge between SNES-style source graphics
and the modder-facing RGBA workflow. It keeps each editable source tile as
human-readable art once, then represents recolors, flashes, fades, and
palette-style looks as reusable runtime effects instead of baking every recolor
into a separate PNG tile.

## Files

`scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets`
emits the compact default atlas set:

```text
generated/zelda3_assets/atlas/tile_effects.json
generated/zelda3_assets/atlas/art_tiles.png
generated/zelda3_assets/atlas/art_tiles.json
```

`art_tiles.png` is the modder-facing art sheet and the default compact runtime
atlas. It deduplicates source tiles by raw index art, collapses
hflip/vflip-equivalent tiles into one editable tile, and records every source
identity that maps to that art under `source_refs`. The renderer expands those
source refs into lookup entries and composes the stored source flip with the
live draw flip. Palette rows, flashes, fades, and other recolors stay metadata
or effects instead of becoming more editable PNG tiles.

`tile_effects.json` describes reusable shader/material effects that can produce
stable palette looks without duplicating the base art. Extraction can still
write the older pre-colored base preview atlas for debugging:

```bash
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets --write-base-effect-atlas
```

That optional legacy output creates:

```text
generated/zelda3_assets/atlas/base_tiles.png
generated/zelda3_assets/atlas/base_tiles.json
```

For oracle/debug work, extraction can also write the diagnostic brute-force
files:

```text
generated/zelda3_assets/atlas/tile_variants.png
generated/zelda3_assets/atlas/tile_variants.json
```

Enable them explicitly:

```bash
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets --write-diagnostic-variants
```

Those files are useful as an oracle and for size/coverage analysis. They are not
the hand-authored source format and are no longer part of default extraction.
The live variant GPU path requires `art_tiles.*`; `tile_variants.*` is retained
as a diagnostic compatibility format.

## Entry Identity

Each base entry represents one source tile rendered with one preview palette:

```json
{
  "id": "sprite:kSprGfx:pack12:tile37:3bpp",
  "source_kind": "sprite",
  "asset": "kSprGfx",
  "pack": 12,
  "tile": 37,
  "bpp": 3,
  "preview_palette": "palette_main_spr",
  "preview_palette_row": 3,
  "preview_source": "palette_usage",
  "palette_usage_evidence_count": 42,
  "rect": [128, 64, 8, 8],
  "sha1": "pixel-content-sha1",
  "duplicate_of": null
}
```

`duplicate_of` points at the first entry with identical RGBA pixels. Duplicate
entries keep their own source identity but reuse the original atlas rectangle.

## Canonical Art Identity

Each canonical art entry represents one editable 8x8 tile:

```json
{
  "art_id": "art:raw-index-sha1",
  "bpp": 3,
  "rect": [128, 64, 8, 8],
  "sha1_indices": "raw-index-sha1",
  "preview_palette": "palette_main_spr",
  "preview_palette_row": 3,
  "preview_source": "palette_usage",
  "source_refs": [
    {
      "source_kind": "sprite",
      "asset": "kSprGfx",
      "pack": 12,
      "tile": 37,
      "bpp": 3,
      "hflip": false,
      "vflip": false,
      "preview_palette": "palette_main_spr",
      "preview_palette_row": 3,
      "preview_source": "palette_usage"
    }
  ]
}
```

If two source tiles are identical after an hflip/vflip transform, they share the
same `art_id`. The transform is recorded on the source reference so a later
runtime or compiler layer can reconstruct the original draw identity.

## Palette Usage

Raw CHR tile bytes do not encode their real colors. SNES BG tilemap attributes,
OAM attributes, CGRAM state, and runtime palette effects decide the final color.
That means `art_tiles.png` can only use the "right" preview palette when
extraction has usage evidence for that source tile.

The generator reads the first existing usage file from:

```text
generated/zelda3_assets/atlas/palette_usage.json
generated/zelda3_assets/assets_src/palette_usage.json
```

Refresh the usage file from the combined-route replay before regenerating the
atlas when you want the editable PNG preview to use the most human-expected row:

```bash
cargo run --profile parity -p zelda3-bin -- --dump-assets-by-source 2000000
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets
```

The source walk stops at replay end; the current combined route reaches frame
`1073092`. Its usage map covers observed raw BG and sprite pack tiles. Tiles not
drawn during the route, Link-specific sources, BG3/HUD cells, and content-hashed
streamed BG sources may still use source-kind default preview colors in the
editable PNG. Runtime drawing can still use stable `tile_effects.json` LUTs for
those defaults when the live draw key names a modeled stable palette row; unknown
or runtime-derived palettes remain on the live indexed fallback.

Usage entries are keyed by the same ROM-derived source identity recorded in
`art_tiles.json` source refs:

```json
{
  "format": "zelda3_palette_usage_v1",
  "entries": [
    {
      "source_kind": "sprite",
      "asset": "kSprGfx",
      "pack": 12,
      "tile": 37,
      "bpp": 3,
      "preview_palette": "palette_main_spr",
      "preview_palette_row": 3,
      "evidence_count": 42
    }
  ]
}
```

If an entry references a missing palette or a row outside the palette's color
range, the canonical art atlas falls back to the source-kind default for that
tile.

`preview_source` records where the preview color came from:

- `palette_usage`: `palette_usage.json` supplied a valid real-use palette row.
- `source_kind_default`: no valid usage entry was available.

## Default Preview Fallback

The fallback preview palette is chosen for human editing, not as a claim that
this is the only palette the game can use:

- sprite art defaults to `palette_main_spr` row `0`
- BG art defaults to `palette_overworld_bg_main` row `0`
- if the preferred palette is absent, extraction falls back to another extracted
  palette instead of failing the whole source build

All extracted palette JSON files are loaded by default, with the common sprite
and BG palettes ordered first. This lets usage evidence point at auxiliary
sprite/BG palettes without requiring a manual palette list.

## Effect Table

`tile_effects.json` stores palette-like transformations as reusable effects:

```json
{
  "id": "palette_main_spr:8color:row3",
  "type": "palette_lut",
  "palette": "palette_main_spr",
  "palette_row": 3,
  "colors_per_row": 8,
  "index_to_rgb": [[255, 255, 255], [206, 49, 16]],
  "dynamic_policy": "stable",
  "runtime": "shader_effect"
}
```

The effect ID includes the row width because 2bpp, 3bpp, and 4bpp tiles interpret
palette rows differently:

- 2bpp: 4-color LUTs
- 3bpp: 8-color LUTs
- 4bpp: 16-color LUTs

Effects should be used for palette swaps, flashes, fades, lighting/darkening,
and other temporary visual treatments. Separate RGBA art should be authored only
for true semantic variants: different materials, different costumes, different
objects, or intentionally changed art.

The Rust atlas loader reads `tile_effects.json` alongside `art_tiles.*`. The
software variant renderer uses a stable matching
`palette_lut` effect by sampling the live source tile index and looking up the
final RGB in `index_to_rgb`; index zero remains transparent. If no matching
stable effect exists, it keeps the older preview-RGBA atlas sampling behavior.
The GPU variant path now has a LUT-backed shader/material route for stable
effect-backed BG and sprite draws, including source/draw flips and OBJ row
masks. Mixed frames keep live indexed fallback for missing/dynamic cells while
overlaying effect-backed stable cells through the LUT shader; stable entries
without an effect still use the preview-RGBA atlas overlay until those material
cases are modeled.

`dynamic_policy` remains conservative:

- `stable`: extracted static palette effect can be represented by the effect
  table.
- `requires_live_palette`: unknown/runtime-derived effect must stay on the live
  palette path until replay evidence proves a stable policy.

## Parity Rule

The base/effect path is not correct just because `base_tiles.png` looks right.
Correctness is final `256x224` framebuffer parity against the current
renderer/oracle. Verification must report:

- `stable_preview_draws`: source art drawn directly from `art_tiles.png`
  because the live draw material matches the entry preview material.
- `stable_effect_draws`: source art drawn through a stable `tile_effects.json`
  palette/effect LUT.
- `dynamic_material_draws`: source art exists, but the live material has no
  modeled stable effect, so the renderer uses the live indexed fallback.
- `missing_art_draws`: the live draw has a source key but no canonical art
  entry.
- `unkeyed_fallback_draws`: the live draw has no ROM/source key and must stay
  on the live indexed path.
- `mixed_overlay_bg_effect_draws`: stable BG effect packets actually overlaid
  on top of a mixed fallback frame by the conservative safe-packet selector.
- `mixed_overlay_bg_effect_candidates`: stable BG effect packets seen in mixed
  fallback frames before the conservative overlay guards are applied.
- `mixed_overlay_bg_effect_reject_complex_frame`: candidates blocked because
  the frame still uses composition features the overlay path does not model yet
  such as color math, windows, mosaic, or non-simple layer state.
- `mixed_overlay_bg_effect_reject_cgram_mismatch`: candidates blocked because
  the extracted stable effect LUT does not exactly match the live CGRAM colors
  used by the source tile indices.
- `mixed_overlay_bg_effect_reject_overlap`: candidates blocked because their
  screen footprint overlaps another BG or OBJ packet, so a late overlay could
  disturb final priority/composition.
- final mismatched pixels

Legacy log names remain available for compatibility: `variant_draws` is the
sum of stable preview and stable effect draws, `fallback_draws` is the sum of
dynamic material, missing art, and unkeyed fallback draws,
`dynamic_palette_draws` mirrors `dynamic_material_draws`, and
`missing_variant_draws` mirrors `missing_art_draws`.

The CHR/palette-index path remains the oracle until representative replay and
oracle-window comparisons prove the base/effect path.

For a focused route-window proof that avoids a broad scan while still requiring
nonzero stable source-art/effect coverage, run:

```bash
python3 scripts/gpu_render_compare_oracle_windows.py \
  --renderer assets-variant-gpu \
  --windows docs/porting/oracle_windows.tsv \
  --only opening-uncle-dismiss-and-move \
  --fast \
  --frames 1000 \
  --stride 60 \
  --require-stable-draws \
  --progress-every 0 \
  --release
```

Expected output includes `mismatched_pixels=0` and nonzero
`stable_preview_draws` or `stable_effect_draws`. The current representative
proof reports `stable_effect_draws=21038`,
`mixed_overlay_bg_effect_candidates=20674`,
`mixed_overlay_bg_effect_draws=0`,
`mixed_overlay_bg_effect_reject_complex_frame=20674`,
`mixed_overlay_bg_effect_reject_cgram_mismatch=0`, and
`mixed_overlay_bg_effect_reject_overlap=0` over 17 sampled compares from the
checkpointed opening route tail. That means this route has stable effect
opportunities and no palette/overlap blocker in the sampled window; the next
modernization target is the frame-composition guard.

Mixed frames that still need dynamic, missing, or unkeyed fallback cells start
from the fully composited fallback pixels for parity. The GPU path may overlay
a stable BG effect packet only when the frame has no color-math/window/mosaic
composition features, the effect LUT matches the live CGRAM for every nonzero
source index in the tile, and the packet footprint is disjoint from every other
BG or OBJ packet. Other stable opportunities stay counted but are not drawn
over the mixed fallback image until packet visibility and final composition
state are modeled more completely.

## Modder Workflow Target

Use `art_tiles.png` as the human-editable sheet. It is intentionally smaller
than both `base_tiles.png` and the brute-force `tile_variants.png` cache because
palette rows and hflip/vflip duplicates are represented in JSON metadata.

The current art-inspection loop is:

```bash
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets
# inspect or edit generated/zelda3_assets/atlas/art_tiles.png
cargo run --profile parity -p zelda3-bin
```

The default runtime loads `art_tiles.*` directly, so edits to the canonical
sheet are the preferred runtime input. Do not use
`tile_variants.png` as an authoring sheet; generate it only with
`--write-diagnostic-variants` when parity/oracle debugging needs the brute-force
cache. Dynamic-palette fallback remains a runtime parity concern: an art tile
can be useful for editing while the renderer still uses live palette fallback
for rows that are not stable final pixels.

Unset `ZELDA3_RENDERER` now chooses the variant atlas GPU path. Live play
presents that path on the window renderer's GPU device without headless
readback or CPU RGBA upload. Use `ZELDA3_VARIANT_ATLAS=off` to keep the older
indexed GPU atlas path, or `ZELDA3_RENDERER=assets-anim` for the CPU atlas
compositor oracle.

If `art_tiles.*` fails to load, the selected default mode reports a canonical
art atlas error instead of silently switching to legacy base previews. Use
`ZELDA3_VARIANT_ATLAS=off` for the older indexed GPU atlas path, or
`ZELDA3_RENDERER=assets-anim` for the CPU atlas compositor oracle/debug mode.

For a cheap live coverage check without a replay scan, set
`ZELDA3_VARIANT_LIVE_STATS=1`. The play loop prints aggregate
`variant_live_summary` lines every 300 presented variant frames by default.
Those lines include both legacy counters and the source/material counters named
above. Use `ZELDA3_VARIANT_LIVE_STATS_EVERY=<frames>` to change the interval.

For an even cheaper generated-asset check that does not start the emulator, run:

```bash
python3 scripts/variant_atlas_summary.py --require-full-stable \
  --manifest generated/zelda3_assets/manifest.json \
  generated/zelda3_assets/atlas
```

That command reads `art_tiles.json` and `tile_effects.json` and reports how many
canonical source refs are covered by the same stable preview/effect rule used by
the variant atlas loader, while also checking that `art_tiles.png` exists,
matches the manifest dimensions, and contains every declared art rect.
`--require-full-stable` makes it fail if the PNG/manifest size drifts, the art
or source-ref counts drift, any art rect is malformed/out of bounds, or any
canonical source ref lacks stable coverage.

`scripts/extract_assets.py` runs this gate for the default compact atlas output
and writes the resulting counts to `manifest.json` as
`canonical_art_atlas_summary`. Passing `--manifest` verifies that the stored
summary still matches the current atlas files.
