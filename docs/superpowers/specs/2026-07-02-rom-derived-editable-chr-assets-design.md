# ROM-Derived Editable CHR Assets (Design)

**Status:** Approved design direction; awaiting review of written spec.
**Date:** 2026-07-02

## Goal

Replace the current monolithic `developer_tilesets/assets_by_source.png` workflow with a
maintainable two-layer CHR asset pipeline:

- **Editable source sheets:** normal-color PNGs organized like the original source art.
- **Generated runtime atlas:** a compact canonical pattern atlas plus manifests consumed by
  the GPU renderer.

The shipped pipeline must be able to extract the editable sheets **from a supported ROM**.
The leaked/source `NES_Ver2/us_char/*.CGX` files are useful for understanding the original
organization and validating layout choices, but they are not required input for users.

## Key Findings

- The current route/source-key atlas has `28,691` cells. That is not the authored art count;
  it is inflated by runtime source keys, especially streamed/content-hashed BG tiles.
- Direct ROM extraction of the graphics tables matches the generated bundle byte-for-byte:
  `kSprGfx`, `kBgGfx`, and `kLinkGraphics` are not being inflated by the pack-array reader.
- Runtime-decoded ROM graphics contain `15,616` tile slots and `11,751` exact unique 8x8
  patterns.
- The original source CGX payloads contain `17,920` tile slots and about `11,669` exact unique
  patterns. This closely matches the ROM count, confirming the scale is real.
- `NES_Ver2/us_char/chr-cnv1.TBL` preserves the original CGX sheet order:
  `2m-2q`, `a-h`, `i-p`, `q-x`, `y-1f`, through `4t-4x`.
- `chr-cnv4.TBL` preserves finer-grained conversion/export group names such as
  `a-h.DAT1N`, `2r-2w.DAT7`, etc. These are useful provenance labels even if the ROM-only
  extractor cannot recover every original editor-side file boundary exactly.

## Chosen Approach

Use **ROM-first extraction with CGX-informed organization**.

The extractor will read the ROM's CHR-relevant regions and graphics pointer tables, then
write editable PNG sheets that follow the CGX grouping/layout as closely as possible:

```text
developer_tilesets/source/a-h.png
developer_tilesets/source/i-p.png
developer_tilesets/source/q-x.png
developer_tilesets/source/2m-2q.png
developer_tilesets/source/4t-4x.png
developer_tilesets/generated/patterns.png
developer_tilesets/generated/patterns.json
developer_tilesets/generated/source_map.json
```

The CGX files are used during development to calibrate:

- sheet names and order,
- expected tile-grid dimensions,
- likely source grouping,
- palette-preview choices,
- and provenance labels.

After calibration, the normal user path is:

```text
zelda3.sfc -> extractor -> editable PNG sheets -> canonical runtime atlas -> renderer
```

No user should need `~/Documents/NES_Ver2` to regenerate editable sheets from their ROM.

## Rejected Approaches

### Use CGX as the required source

Rejected because it would make the modding pipeline depend on files that normal users do not
have. CGX remains a reference corpus, not a dependency.

### Keep one giant source atlas

Rejected because it hides the original organization, makes edits hard to review, and repeats
many source-key variants that should be metadata entries pointing at shared patterns.

### Load CGX-style sheets directly at runtime

Rejected because runtime wants compact GPU-friendly pattern data and stable source-key lookup,
not editor-oriented sheet organization. The editable layer and runtime layer should be
separate.

## Asset Layers

### 1. Editable Source Sheets

Editable sheets are normal-color PNGs with stable layout. They should be pleasant to inspect
and edit in common image tools.

Each sheet has a sidecar manifest:

```json
{
  "sheet": "a-h",
  "source": {
    "kind": "rom",
    "rom_sha1": "6d4f10a8b10e10dbe624cb23cf03b88bb8252973",
    "rom_ranges": ["0x108000..0x110000"]
  },
  "layout": {
    "tile_width": 8,
    "tile_height": 8,
    "columns": 16,
    "rows": 64
  },
  "palette": {
    "preview": "color-rom-a",
    "mode": "indexed-with-sidecar"
  },
  "tiles": [
    {
      "x": 0,
      "y": 0,
      "rom_tile_id": "chr:0x108000:0",
      "pattern_id": 0
    }
  ]
}
```

The PNG colors are preview colors, not the authoritative runtime palette. The authoritative
tile data is the palette-index identity recovered from the PNG via the sidecar's color table.
This allows "normal colors" while preserving SNES index semantics.

### 2. Canonical Pattern Atlas

The generated runtime atlas deduplicates 8x8 patterns. A source tile maps to:

- `pattern_id`,
- transform (`none`, `hflip`, `vflip`, `hvflip`),
- `index_remap` when the same shape uses different palette-index labels,
- provenance back to the editable sheet and ROM source.

The expected target count is around:

- `~11.7k` exact unique patterns,
- `~11.0k` if using flip + palette-index-remap canonicalization.

The canonical atlas is generated output. It is not hand-edited.

### 3. Runtime Source Map

The runtime source map bridges old renderer/source keys to the canonical atlas:

```json
{
  "source_key": {
    "kind": "BG",
    "pack": 42,
    "tile": 17
  },
  "pattern_id": 1234,
  "transform": "hflip",
  "index_remap": [0, 2, 1, 3, 4, 5, 6, 7],
  "provenance": {
    "sheet": "a-h",
    "tile_x": 3,
    "tile_y": 12,
    "rom_range": "0x108000..0x108020"
  }
}
```

For `BG_STREAM` / content-hashed dynamic tiles, keep a separate generated stream map if those
tiles cannot be traced to stable ROM sheet coordinates. They should not bloat the editable
authored-art sheets.

## Data Flow

### Extraction

```text
ROM
  -> read CHR region and graphics pointer tables
  -> reconstruct CGX-informed editable sheets
  -> write source PNGs + sidecar JSON
```

Validation during development:

```text
NES_Ver2/us_char/*.CGX
  -> compare recovered sheet layout and tile payloads
  -> record mapping decisions in tests/fixtures
```

### Runtime Build

```text
source PNGs + sidecars
  -> recover palette-index tiles
  -> canonicalize patterns
  -> write generated/patterns.png + manifests
  -> renderer loads generated runtime atlas
```

### Rendering

```text
source key from extracted ModernFrame
  -> runtime source map
  -> pattern atlas lookup
  -> apply transform + index_remap
  -> CGRAM lookup
  -> final GPU pixel
```

## Error Handling

- Missing or unsupported ROM: fail with the ROM SHA1 and list supported expectations.
- ROM extraction cannot match a CGX-informed layout: emit a precise manifest error naming the
  sheet, ROM range, and tile coordinate.
- PNG edit uses a color not present in the sidecar palette table: fail the asset build with
  the pixel coordinate and expected palette name.
- Duplicate canonicalization collision with incompatible index semantics: fail generation;
  do not silently merge.
- Runtime missing source key: fail closed in compare/test paths and visibly mark missing art
  in developer paths.

## Testing

Tests should be written before implementation.

Required coverage:

- ROM `kSprGfx`, `kBgGfx`, and Link CHR extraction matches current direct ROM bytes.
- ROM-derived editable sheets are deterministic for the known USA ROM.
- CGX calibration fixtures confirm sheet names, order, dimensions, and representative tile
  mappings.
- PNG sidecar round-trip preserves palette indices despite normal preview colors.
- Canonicalization correctly handles exact duplicates, h/v flips, and palette-index remaps.
- Generated runtime atlas renders byte-identical to the current source-atlas path on targeted
  frames before broad parity runs.

## Rollout

1. Add ROM/CGX analysis fixtures and tests.
2. Add a developer command to extract editable source sheets from ROM.
3. Add sidecar-aware PNG round-trip.
4. Add canonical runtime atlas generation.
5. Update the renderer loader to accept the new manifest while keeping the old atlas as a
   fallback during migration.
6. Regenerate assets and run targeted `--modern-index-compare`.
7. Remove the old monolithic atlas only after parity is proven.

## Open Constraints

- The CGX files should guide organization, but the checked-in/generated user-facing pipeline
  must not require them.
- The asset format must preserve final-pixel parity at `256x224`.
- Runtime output remains native-resolution indexed composition; HD/upscale work is separate.
- Generated assets should include provenance because future debugging needs to answer "which
  ROM tile/sheet did this visible pixel come from?"
