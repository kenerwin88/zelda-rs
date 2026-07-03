# Base Art and Effect Atlas Contract

The base/effect atlas is the bridge between SNES-style source graphics and the
future modder-facing RGBA workflow. It keeps each source tile as human-readable
base art once, then represents recolors, flashes, fades, and palette-style looks
as reusable runtime effects instead of baking every recolor into a separate PNG
tile.

## Files

`scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets`
emits the compact default atlas set:

```text
generated/zelda3_assets/atlas/base_tiles.png
generated/zelda3_assets/atlas/base_tiles.json
generated/zelda3_assets/atlas/tile_effects.json
generated/zelda3_assets/atlas/art_tiles.png
generated/zelda3_assets/atlas/art_tiles.json
```

The PNG is an RGBA atlas of pre-colored 8x8 base tiles. When
`atlas/palette_usage.json` exists, each tile is previewed with the palette row
seen in real draw usage. Without that evidence, extraction falls back to a broad
source-kind default. `base_tiles.json` is the source identity contract.
`tile_effects.json` describes reusable shader/material effects that can produce
the alternate looks without duplicating the base art.

`art_tiles.png` is the cleaner modder-facing art sheet and the preferred compact
runtime atlas when present. It deduplicates source tiles by raw index art,
collapses hflip/vflip-equivalent tiles into one editable tile, and records every
source identity that maps to that art under `source_refs`. The renderer expands
those source refs into lookup entries and composes the stored source flip with
the live draw flip. Palette rows, flashes, fades, and other recolors stay
metadata or effects instead of becoming more editable PNG tiles.

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
The live variant GPU path loads `art_tiles.*` when available and falls back to
`base_tiles.*`; `tile_variants.*` is retained as a diagnostic compatibility
format.

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
That means `base_tiles.png` can only use the "right" palette when extraction has
usage evidence for that source tile.

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

Usage entries are keyed by the same ROM-derived source identity used by
`base_tiles.json`:

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
range, the base atlas falls back to the source-kind default for that tile.

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
and other temporary visual treatments. Separate RGBA base art should be authored
only for true semantic variants: different materials, different costumes,
different objects, or intentionally changed art.

The Rust atlas loader reads `tile_effects.json` alongside `art_tiles.*` or
`base_tiles.*`. The software variant renderer uses a stable matching
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

- base art draws
- effect-table draws
- live palette fallback draws
- missing base tile draws
- final mismatched pixels

The CHR/palette-index path remains the oracle until representative replay and
oracle-window comparisons prove the base/effect path.

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

The compiler that projects edited `art_tiles.*` back into runtime assets is a
later layer. Do not use `tile_variants.png` as an authoring sheet; generate it
only with `--write-diagnostic-variants` when parity/oracle debugging needs the
brute-force cache. Dynamic-palette fallback remains a runtime parity concern: an
art tile can be useful for editing while the renderer still uses live palette
fallback for rows that are not stable final pixels.

Unset `ZELDA3_RENDERER` now chooses the variant atlas GPU path. Live play
presents that path on the window renderer's GPU device without headless
readback or CPU RGBA upload. Use `ZELDA3_VARIANT_ATLAS=off` to keep the older
indexed GPU atlas path, or `ZELDA3_RENDERER=assets-anim` for the CPU atlas
compositor oracle.

For a cheap live coverage check without a replay scan, set
`ZELDA3_VARIANT_LIVE_STATS=1`. The play loop prints aggregate
`variant_live_summary` lines every 300 presented variant frames by default; use
`ZELDA3_VARIANT_LIVE_STATS_EVERY=<frames>` to change the interval.
