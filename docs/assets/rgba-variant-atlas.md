# RGBA Variant Atlas Contract

The RGBA variant atlas is the bridge between SNES-style source graphics and the
future modder-facing RGBA workflow.

## Files

`scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets`
emits:

```text
generated/zelda3_assets/atlas/tile_variants.png
generated/zelda3_assets/atlas/tile_variants.json
```

The PNG is an RGBA atlas of pre-colored 8x8 tile variants. The JSON is the
runtime contract and must stay authoritative; the renderer should resolve draw
identity through JSON entries, not by guessing from atlas coordinates.

## Entry Identity

Each entry represents one source tile rendered with one palette row:

```json
{
  "id": "sprite:kSprGfx:pack12:tile37:3bpp:palette_main_spr:row3",
  "source_kind": "sprite",
  "asset": "kSprGfx",
  "pack": 12,
  "tile": 37,
  "bpp": 3,
  "palette": "palette_main_spr",
  "palette_row": 3,
  "rect": [128, 64, 8, 8],
  "sha1": "pixel-content-sha1",
  "duplicate_of": null,
  "dynamic_policy": "stable"
}
```

`duplicate_of` points at the first entry with identical RGBA pixels. Duplicate
entries keep their own source identity but reuse the original atlas rectangle.

## Color Rules

The generator starts from decoded CHR index pixels and extracted palette JSON.
It applies palette rows according to source bit depth:

- 2bpp: 4-color rows
- 3bpp: 8-color rows
- 4bpp: 16-color rows

Index `0` is transparent in the generated RGBA tile: alpha `0`. Nonzero indices
use alpha `255`.

## Dynamic Palette Policy

`dynamic_policy` is conservative:

- `stable`: palette is a ROM-extracted static palette currently accepted for
  pre-rendered atlas use.
- `requires_live_palette`: palette identity is unknown or runtime-derived and
  must stay on a live palette path until replay evidence proves a stable policy.

Stable atlas entries can be used by the variant renderer. Entries requiring live
palette handling must either fall back to the existing palette-index renderer or
be regenerated when CGRAM changes.

## Parity Rule

The atlas is not correct just because the PNG looks right. Correctness is final
`256x224` framebuffer parity against the current renderer/oracle. Verification
must report:

- stable atlas draws
- dynamic palette fallback draws
- missing variant draws
- final mismatched pixels

The CHR/palette-index path remains the oracle until representative replay and
oracle-window comparisons prove the atlas path.

## Modder Workflow Target

Semantic RGBA sheets should compile into this same `tile_variants.json` contract.
The runtime should not care whether an entry came directly from ROM CHR or from a
modder-edited semantic PNG; both paths must emit the same IDs and coverage
checks.
