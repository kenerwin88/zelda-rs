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
`1073092`. Its usage map covers observed raw BG and sprite pack tiles.
Content-hashed Link, BG3/HUD, and streamed BG cells may still use source-kind
default preview colors in the editable PNG. Runtime drawing uses stable
`tile_effects.json` LUTs for those defaults when the live draw key names a
modeled stable palette row; unknown or runtime-derived palettes remain on the
live indexed fallback.

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

Runtime BG/OBJ drawing still samples live SNES CGRAM as `palette * 16 + index`.
When both source-width and 16-color rows exist for the same palette row, the
renderer prefers the 16-color row for runtime material matching and keeps the
source-width rows available for preview/diagnostic compatibility.

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
overlaying safe stable BG cells through the LUT shader. If the static
`tile_effects.json` LUT does not match live CGRAM but the packet is otherwise
composition-safe, the mixed overlay path binds a per-frame live-CGRAM LUT
instead of rejecting the packet. Stable entries without an effect still use the
preview-RGBA atlas overlay until those material cases are modeled.

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
- `dynamic_material_draws`: compatibility aggregate for source-art draws that
  need a runtime material decision. This includes both modeled effect draws and
  true dynamic fallback draws.
- `effect_material_draws`: source art drawn through a modeled stable effect
  material. This is already on the modern material path.
- `live_index_draws`: GPU live-index material draws. These are first-class
  indexed draws sourced from the current frame's tile data; they still preserve
  dynamic SNES VRAM behavior, but they are not CPU fallback.
- `live_index_bg_draws`: live-index draws from BG tile packets.
- `live_index_bg12_draws`: live-index draws from BG1/BG2 tile packets.
- `live_index_bg3_draws`: live-index draws from BG3/HUD/message-layer tile
  packets.
- `live_index_sprite_draws`: live-index draws from OBJ packets.
- `gpu_prefinal_base_frames`: frames where the live-index/prefinal base stayed
  on the GPU. This replaces the misleading internal name
  `direct_gpu_fallback_frames`.
- `dynamic_material_fallback_draws`: source art exists, but the live material
  has no modeled stable effect, so the renderer uses the live indexed fallback.
- `dynamic_material_fallback_instance_source_draws`: fallback draws forced by
  instance source-key identity. These still use source art, but the draw needs
  live indexed pixels to preserve the exact source instance.
- `dynamic_material_fallback_brightness_draws`: regression counter for fallback
  draws forced by non-full master brightness. Stable material/effect draws are
  expected to stay on the material path and let the final compositor apply
  frame brightness; this counter should remain zero.
- `dynamic_material_fallback_policy_draws`: fallback draws whose atlas entry is
  explicitly marked `requires_live_palette`.
- `dynamic_material_fallback_missing_effect_draws`: fallback draws where the
  source art exists but no stable `tile_effects.json` effect matches the live
  material key yet.
- `dynamic_material_fallback_unsupported_draws`: fallback draws caused by a
  runtime material class that the renderer does not model yet.
- `unsupported_material_draws`: the fallback subset caused by an explicit
  runtime material the renderer does not model yet.
- `missing_art_draws`: the live draw has a source key but no canonical art
  entry.
- `unkeyed_fallback_draws`: the live draw has no ROM/source key and must stay
  on the live indexed path. This is a legacy reason bucket; these draws also
  count as `live_index_draws`, not `fallback_draws`.
- `unkeyed_bg_fallback_draws`: unkeyed fallback draws that came from BG tile
  packets. Legacy alias for `live_index_bg_draws`.
- `unkeyed_bg12_fallback_draws`: BG1/BG2 unkeyed fallback draws. These are
  live-indexed because their current source identity is not injective enough for
  canonical art selection. Legacy alias for `live_index_bg12_draws`.
- `unkeyed_bg3_fallback_draws`: BG3/HUD/message-layer unkeyed fallback draws.
  These are live-indexed because the layer is procedurally composed in VRAM.
  Legacy alias for `live_index_bg3_draws`.
- `unkeyed_sprite_fallback_draws`: unkeyed fallback draws that came from OBJ
  packets. Legacy alias for `live_index_sprite_draws`.
- `mixed_overlay_bg_effect_draws`: stable BG effect packets actually overlaid
  on top of a mixed fallback frame by the conservative safe-packet selector.
- `mixed_overlay_bg_effect_candidates`: stable BG effect packets seen in mixed
  fallback frames before the conservative overlay guards are applied.
- `mixed_overlay_bg_effect_culled_invisible_main`: candidates that have no
  visible main-screen pixels after raw/per-scanline main-screen masking. These
  packets need no GPU draw and are not CPU fallback blockers.
- `mixed_overlay_bg_effect_reject_complex_frame`: candidates blocked because
  the frame still uses composition features the overlay path does not model yet
  such as color math, windows, mosaic, or non-simple layer state.
- `mixed_overlay_bg_effect_reject_complex_brightness`: complex-frame rejects
  blocked by non-full master brightness.
- `mixed_overlay_bg_effect_reject_complex_invalid_layer`: complex-frame rejects
  whose source layer cannot be mapped to a supported BG overlay layer.
- `mixed_overlay_bg_effect_reject_complex_mosaic`: complex-frame rejects
  blocked by active mosaic on the candidate's BG layer.
- `mixed_overlay_bg_effect_reject_complex_sub_window`: complex-frame rejects
  blocked by sub-screen windowing state.
- `mixed_overlay_bg_effect_reject_complex_effect_bounds`: complex-frame rejects
  whose source indices exceed the static effect row.
- `mixed_overlay_bg_effect_reject_complex_scanline_main`: complex-frame rejects
  blocked by main-screen scanline state while still having visible main-screen
  contribution. Fully invisible packets are counted as culls instead.
- `mixed_overlay_bg_effect_reject_complex_layer_window`: complex-frame rejects
  blocked by layer window masking at one of the candidate's nontransparent
  pixels.
- `mixed_overlay_bg_effect_reject_complex_color_math`: complex-frame rejects
  blocked because color math would change at least one nontransparent candidate
  pixel and the current overlay shader writes final RGB directly.
- `mixed_overlay_bg_effect_reject_complex_color_math_clip`: color-math rejects
  caused by color-window clipping that would black out at least one
  nontransparent candidate pixel.
- `mixed_overlay_bg_effect_reject_complex_color_math_subscreen`: color-math
  rejects caused by sub-screen addition/subtraction, where the final candidate
  RGB depends on another composited screen pixel.
- `mixed_overlay_bg_effect_reject_complex_color_math_fixed_color`: color-math
  rejects caused by fixed-color addition/subtraction or half-color math.
- `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch`:
  packets accepted by the color-math pre-final policy but still blocked because
  neither the static effect LUT nor the live-CGRAM LUT can represent the source
  indices.
- `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap`: packets
  accepted by the color-math pre-final policy but still blocked because their
  nontransparent pixels overlap another BG or OBJ packet before finalization.
- `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg`:
  pre-final overlap rejects caused by a front or same-order BG packet at one of
  the candidate's nontransparent pixels.
- `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj`:
  pre-final overlap rejects caused by an OBJ pixel at one of the candidate's
  nontransparent pixels.
- `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_deeper_chain`:
  BG overlap rejects where the front/same-order BG packet has another front BG
  packet at the same pixel, so a two-packet terminal group is not enough.
- `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front`:
  BG overlap rejects where the front/same-order BG packet is not currently
  representable by the pre-final static-effect or live-CGRAM paths.
- `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_mixed_static_live_order`:
  BG overlap rejects where both packets are representable, but the current
  static-then-live pre-final overlay batches would not preserve original packet
  order.
- `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect`:
  unrepresentable-front BG rejects where the front/same-order BG packet is not
  a stable effect packet.
- `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex`:
  unrepresentable-front BG rejects where the front/same-order BG packet is a
  stable effect packet but is blocked by the same complex-frame guards as a
  primary candidate.
- `mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch`:
  unrepresentable-front BG rejects where the front/same-order BG packet is a
  stable effect packet but neither the static effect LUT nor the live-CGRAM LUT
  can represent its source indices.
- `mixed_overlay_bg_effect_reject_cgram_mismatch`: candidates blocked because
  neither the extracted stable effect LUT nor the live-CGRAM LUT can represent
  the packet's source indices.
- `mixed_overlay_bg_effect_reject_overlap`: candidates blocked because their
  nontransparent screen pixels overlap another BG or OBJ packet, so a late
  overlay could disturb final priority/composition.
- final mismatched pixels

Legacy log names remain available for compatibility: `variant_draws` is the
sum of stable preview and stable effect draws, `fallback_draws` is the sum of
dynamic material fallback and missing art draws, while `live_index_draws`
tracks unkeyed/live-index material draws separately.
`live_index_bg_draws`, `live_index_bg12_draws`, `live_index_bg3_draws`, and
`live_index_sprite_draws` are the preferred reason buckets for live-index
material.
`dynamic_palette_draws` mirrors `dynamic_material_fallback_draws`,
`dynamic_material_draws` remains the aggregate of `effect_material_draws` and
`dynamic_material_fallback_draws`, `dynamic_material_fallback_draws` is the sum
of the five `dynamic_material_fallback_*` reason buckets above,
`unsupported_material_draws` is a subset of
`dynamic_material_fallback_unsupported_draws`, and `missing_variant_draws`
mirrors `missing_art_draws`. Logs still print `direct_gpu_fallback_frames` as
a legacy alias for `gpu_prefinal_base_frames`.

The CHR/palette-index path remains the oracle until representative replay and
oracle-window comparisons prove the base/effect path.

In live play, the default `assets-variant-gpu` path requires full GPU rendering
and full source-art/material coverage. It exits with
`gpu_path_unsupported_live` if variant stats report CPU-prefinal composition,
missing source art, unsupported or unmodeled material fallback, unkeyed
live-index fallback, or any other aggregate source-art fallback draw. Set
`ZELDA3_REQUIRE_FULL_GPU_PATH=0` only for explicit debugging of unsupported
escape hatches.

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

Expected output includes `mismatched_pixels=0` and nonzero material-backed
source-art coverage (`effect_material_draws`). The current representative proof
reports `effect_material_draws=154150`,
`live_index_draws=0`,
`live_index_bg_draws=0`,
`live_index_bg12_draws=0`,
`live_index_bg3_draws=0`,
`live_index_sprite_draws=0`,
`gpu_prefinal_base_frames=<GPU base/prefinal frames>`,
`unkeyed_fallback_draws=0`,
`unkeyed_bg_fallback_draws=0`,
`unkeyed_bg12_fallback_draws=0`,
`unkeyed_bg3_fallback_draws=0`,
`unkeyed_sprite_fallback_draws=0`,
`mixed_overlay_bg_effect_candidates=0`,
`mixed_overlay_bg_effect_draws=0`,
`mixed_overlay_bg_effect_culled_invisible_main=0`,
`mixed_overlay_bg_effect_reject_complex_frame=0`,
`mixed_overlay_bg_effect_reject_complex_effect_bounds=0`,
`mixed_overlay_bg_effect_reject_complex_scanline_main=0`,
`mixed_overlay_bg_effect_reject_complex_color_math=0`,
`mixed_overlay_bg_effect_reject_complex_color_math_clip=0`,
`mixed_overlay_bg_effect_reject_complex_color_math_subscreen=0`,
`mixed_overlay_bg_effect_reject_complex_color_math_fixed_color=0`,
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_cgram_mismatch=0`,
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap=0`,
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg=0`,
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_obj=0`,
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_no_effect=0`,
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_complex=0`,
`mixed_overlay_bg_effect_reject_complex_color_math_prefinal_overlap_bg_unrepresentable_front_cgram_mismatch=0`,
`mixed_overlay_bg_effect_reject_cgram_mismatch=0`, and
`mixed_overlay_bg_effect_reject_overlap=0` over 17 sampled compares from the
checkpointed opening route tail. That means this focused window now executes
through stable source-art/material draws with exact final-pixel parity and no
live-index fallback draws.

A broader fast oracle pass across the 12 default windows plus the SRAM-sidecar
TAS window (`--include-sram-windows --only tas-us-full-completion-smv`) reports
`mismatched_pixels=0`, `live_index_draws=0`, `missing_art_draws=0`, and
`unsupported_material_draws=0` for every sampled window.

The first pre-final sub-screen implementation supports static variant-effect BG
packets: those pixels can be written into the packed main-screen buffer before
the GPU finalizer runs, so final color math uses the real sub-screen operand.
The native live-CGRAM pre-final lane uses the fallback compositor's direct
indexed sampling rules (`cell.indices[y*8+x]`, `palette*16+index`) and writes
packed 5-bit RGB plus the layer math bit before the finalizer runs. This reduces
the representative sub-screen reject bucket from 1,861 to 910 with
`mismatched_pixels=0`. Behind-only BG overlap is now allowed for pre-final
packets: if every overlapping BG pixel is behind the variant packet in Mode-1
painter order and no OBJ pixel overlaps it, the packed main-screen overlay can
replace the fallback pixel before final color math. The representative route
still reports the same 910 pre-final overlap rejects, split as 703
front/same-order BG overlaps and 207 OBJ overlaps. The next useful renderer lane
is therefore an ordered pre-final BG group path before OBJ-aware composition.
The first ordered BG group slice admits terminal two-packet BG groups, where the
front overlapping packet is itself representable and has no further front BG at
that pixel. Splitting the remaining BG overlap rejects shows the opening route's
703 BG blockers are all `unrepresentable_front`, with `deeper_chain=0` and
`mixed_static_live_order=0`. Splitting that bucket again shows 39 front packets
have no stable effect and 664 are stable-effect packets still blocked by complex
frame state, with no cgram-mismatch blockers. The next useful lane is therefore
pre-final support for complex front BG packets, especially the existing
scanline-main and sub-screen color-math guards, before broadening group
ordering. The front-BG visibility slice now ignores front/same-order BG pixels
that are disabled by the per-scanline main-screen mask at the overlapped pixel.
That removes the representative BG-overlap blocker (`prefinal_overlap_bg=0`)
with `mismatched_pixels=0`; the next useful lanes are the primary
`scanline_main=1541` bucket and the remaining OBJ pre-final overlap bucket
(`prefinal_overlap_obj=350`). The OBJ-aware pre-final slice then tracks the BG
overlay rank for each replaced pixel and repaints front OBJ pixels from live
CGRAM before the GPU finalizer runs. That removes the representative OBJ
overlap blocker (`prefinal_overlap_obj=0`) with `mismatched_pixels=0`; the next
useful lane is the remaining primary `scanline_main=1541` bucket.

Mixed frames that still need dynamic, missing, or unkeyed fallback cells start
from the fully composited fallback pixels for parity. The GPU path may overlay
a stable BG effect packet only when the modeled per-pixel visibility and
composition state can preserve the fallback winner: invisible main pixels are
skipped, eligible pre-final color-math packets write before finalization, and
front OBJ pixels are restored over BG replacements by Mode-1 rank. Other stable
opportunities stay counted but are not drawn over the mixed fallback image until
their remaining composition state is modeled more completely.

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

Current generated output reports `art_count=43656`, `source_refs=62025`, and
`stable_by_kind bg=29081 bg3=24702 link=1010 sprite=7232`.

`scripts/extract_assets.py` runs this gate for the default compact atlas output
and writes the resulting counts to `manifest.json` as
`canonical_art_atlas_summary`. Passing `--manifest` verifies that the stored
summary still matches the current atlas files.
