# Modder RGBA Workflow

This is the preferred workflow for replacing stable visual art without editing
CHR bytes, CGRAM rows, or CGX-era source sheets directly. Start with the
canonical art sheet, not the brute-force runtime variant atlas: palette rows,
flashes, fades, and flipped duplicates should stay metadata unless there is a
specific reason to edit a runtime variant directly.

## Extract

Generate the ignored asset workspace from a ROM:

```bash
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets
```

This creates the runtime atlases under `generated/zelda3_assets/atlas/` and the
readable source tree under `generated/zelda3_assets/assets_src/`.

## Edit Canonical Art First

The compact editable sheet is:

```text
generated/zelda3_assets/atlas/art_tiles.png
generated/zelda3_assets/atlas/art_tiles.json
```

`art_tiles.png` stores one canonical RGBA preview per raw index tile, with
hflip/vflip-equivalent tiles collapsed into a single editable tile. The JSON
sidecar records `source_refs`, so many ROM/source draw identities can point at
the same art. This is the right place to make broad visual replacements before
semantic naming exists.

## Export Runtime Variant Sheets

Only after the canonical art layer is clean, create semantic RGBA sheets from
the current runtime atlas:

```bash
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets
```

Editable sheets are written under:

```text
generated/zelda3_assets/assets_src/semantic/backgrounds/
generated/zelda3_assets/assets_src/semantic/sprites/
```

Each PNG has a matching JSON sidecar. The PNG is the editable art. The JSON
defines frame rectangles and `emits` links back to exact runtime variant IDs.
Keep those links intact unless you are deliberately changing atlas coverage.

## Edit

Open the PNGs in any RGBA-capable editor and change pixels directly. The current
sheet names are mechanical, for example:

```text
sprites/sprite_kSprGfx_pack12.png
backgrounds/bg_kBgGfx_pack42.png
```

Do not rename sheets or frame IDs as object names yet. Semantic names should be
introduced only after coverage validation can prove the mapping.

## Validate

Check that every stable runtime variant is covered exactly once:

```bash
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --validate
```

The validator writes:

```text
generated/zelda3_assets/assets_src/semantic/coverage_report.json
```

Validation fails if a stable variant ID is missing, emitted more than once, or
referenced by a rectangle outside its PNG. Variants marked as requiring live
palette behavior may be absent only when listed under `dynamic_fallback`.

## Compile

Compile edited semantic sheets back into the runtime atlas:

```bash
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --compile
```

This rewrites:

```text
generated/zelda3_assets/atlas/tile_variants.png
generated/zelda3_assets/atlas/tile_variants.json
```

The compile step preserves the atlas schema consumed by the renderer, so mods
can replace art without changing runtime lookup code.

## Run

The default renderer now uses the variant atlas GPU path:

```bash
cargo run --profile parity -p zelda3-bin
```

Use these opt-outs when comparing behavior:

```bash
ZELDA3_VARIANT_ATLAS=off cargo run --profile parity -p zelda3-bin
ZELDA3_RENDERER=assets-anim cargo run --profile parity -p zelda3-bin
```

`ZELDA3_VARIANT_ATLAS=off` keeps the older indexed GPU atlas path.
`ZELDA3_RENDERER=assets-anim` uses the CPU atlas compositor oracle.

## Dynamic Palette Fallback

Some variants are not stable final pixels. The game can recolor a source tile
through live palette updates, flashes, fades, lighting, or other effects. Those
rows may still appear in semantic PNGs for editing and inspection, but the
renderer must use live palette fallback until replay evidence proves a stable
final-pixel policy.

The parity rule is final framebuffer equality at `256x224`, not whether a PNG
looks plausible in isolation.
