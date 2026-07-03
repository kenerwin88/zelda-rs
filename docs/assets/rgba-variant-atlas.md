# Base Art and Effect Atlas Contract

The base/effect atlas is the bridge between SNES-style source graphics and the
future modder-facing RGBA workflow. It keeps each source tile as human-readable
base art once, then represents recolors, flashes, fades, and palette-style looks
as reusable runtime effects instead of baking every recolor into a separate PNG
tile.

## Files

`scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets`
emits:

```text
generated/zelda3_assets/atlas/base_tiles.png
generated/zelda3_assets/atlas/base_tiles.json
generated/zelda3_assets/atlas/tile_effects.json
```

The PNG is an RGBA atlas of pre-colored 8x8 base tiles. When
`atlas/palette_usage.json` exists, each tile is previewed with the palette row
seen in real draw usage. Without that evidence, extraction falls back to a broad
source-kind default. `base_tiles.json` is the source identity contract.
`tile_effects.json` describes reusable shader/material effects that can produce
the alternate looks without duplicating the base art.

The extraction also still writes the diagnostic brute-force files:

```text
generated/zelda3_assets/atlas/tile_variants.png
generated/zelda3_assets/atlas/tile_variants.json
```

Those files are useful as an oracle and for size/coverage analysis. They are not
the intended long-term hand-authored source format, but they remain the runtime
contract for pre-colored RGBA variants. Semantic modder sheets compile back into
this same `tile_variants.*` schema.

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
atlas:

```bash
cargo run --profile parity -p zelda3-bin -- --dump-assets-by-source 2000000
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets
```

The source walk stops at replay end; the current combined route reaches frame
`1073092`. Its usage map covers observed raw BG and sprite pack tiles. Tiles not
drawn during the route, Link-specific sources, BG3/HUD cells, and content-hashed
streamed BG sources remain on fallback preview colors until they get their own
stable semantic source IDs or additional evidence.

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

Semantic RGBA sheets compile into the same `tile_variants.*` runtime contract.
The runtime should not care whether RGBA variant art came directly from ROM CHR
or from a modder-edited semantic PNG; both paths must emit the same source IDs
and coverage checks.

## Semantic RGBA Authoring

The semantic sheet exporter creates normal RGBA PNGs grouped by source kind,
asset, and pack. These names are mechanical on purpose; object names such as
`guards` or `pots` should wait until coverage tooling can prove the mapping.

Export sheets from the current atlas:

```bash
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets
```

This writes files such as:

```text
generated/zelda3_assets/assets_src/semantic/sprites/sprite_kSprGfx_pack12.png
generated/zelda3_assets/assets_src/semantic/sprites/sprite_kSprGfx_pack12.json
generated/zelda3_assets/assets_src/semantic/backgrounds/bg_kBgGfx_pack42.png
generated/zelda3_assets/assets_src/semantic/backgrounds/bg_kBgGfx_pack42.json
```

The sidecar JSON lists frames and exact runtime variant IDs:

```json
{
  "format": "zelda3_semantic_rgba_sheet_v1",
  "source_kind": "sprite",
  "asset": "kSprGfx",
  "pack": 12,
  "image_file": "sprite_kSprGfx_pack12.png",
  "frames": [
    {
      "id": "sprite_kSprGfx_pack12_tile3_palette_main_spr_row0",
      "source_rect": [0, 0, 8, 8],
      "emits": ["sprite:kSprGfx:pack12:tile3:3bpp:palette_main_spr:row0"]
    }
  ]
}
```

Edit the PNGs, keep frame rectangles and `emits` links intact, then compile
back into the runtime atlas:

```bash
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --compile
```

The compile step rewrites:

```text
generated/zelda3_assets/atlas/tile_variants.png
generated/zelda3_assets/atlas/tile_variants.json
```

The current modder loop is:

```bash
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets
# edit generated/zelda3_assets/assets_src/semantic/**/*.png
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --compile
ZELDA3_RENDERER=assets-variant-gpu cargo run --profile parity -p zelda3-bin
```

Compilation fails if a required variant ID is missing, emitted more than once,
or referenced by a frame rectangle outside its PNG bounds. Dynamic-palette
fallback remains a runtime parity concern: a PNG can be useful for editing while
the renderer still uses live palette fallback for rows that are not stable final
pixels.
