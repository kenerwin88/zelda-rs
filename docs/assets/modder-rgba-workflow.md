# Modder RGBA Workflow

This is the preferred workflow for replacing stable visual art without editing
CHR bytes, CGRAM rows, or CGX-era source sheets directly. Start with the
canonical art sheet, not the brute-force runtime variant atlas: palette rows,
flashes, fades, and flipped duplicates should stay metadata unless there is a
specific parity/debug reason to inspect a fully materialized variant directly.

## Extract

Generate the ignored asset workspace from a ROM:

```bash
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets
```

This creates the compact runtime/art atlases under
`generated/zelda3_assets/atlas/` and the readable source tree under
`generated/zelda3_assets/assets_src/`. It does not write the giant
`tile_variants.*` diagnostic atlas unless you ask for it:

```bash
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets --write-diagnostic-variants
```

## Edit Canonical Art First

The compact editable sheet is:

```text
generated/zelda3_assets/atlas/art_tiles.png
generated/zelda3_assets/atlas/art_tiles.json
```

`art_tiles.png` stores one canonical RGBA preview per raw index tile, with
hflip/vflip-equivalent tiles collapsed into a single editable tile. The JSON
sidecar records `source_refs`, so many ROM/source draw identities can point at
the same art. The renderer expands those refs and preserves their stored
hflip/vflip transforms, so this compact sheet can serve both editing and the
default variant GPU runtime. This is the right place to make broad visual
replacements before clean source-family naming exists.

## Edit

Open `art_tiles.png` in any RGBA-capable editor and change pixels directly.
Use `art_tiles.json` to see every source tile that points at the edited art.
Do not edit `tile_variants.png` as an authoring sheet; it is an opt-in derived
oracle/cache for parity debugging.

## Rebuild Runtime Art

The canonical art sheet is the authoring source and the default runtime input.
The renderer loads `art_tiles.*` directly when present, expands `source_refs`
into runtime draw entries, and keeps palette/effect behavior in metadata.

Run the manifest gate after extractor changes or art-sheet edits:

```bash
python3 scripts/variant_atlas_summary.py --require-full-stable generated/zelda3_assets/atlas
```

## Run

The default renderer now uses the variant atlas GPU path and presents it through
the live window GPU renderer without headless readback:

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
rows should remain metadata/effects until replay evidence proves a stable
final-pixel policy.

The parity rule is final framebuffer equality at `256x224`, not whether a PNG
looks plausible in isolation.
