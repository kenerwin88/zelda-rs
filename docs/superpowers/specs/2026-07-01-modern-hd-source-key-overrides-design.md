# Modern off-VRAM HD source-key overrides — Phase 1 (native-res recolor)

**Date:** 2026-07-01
**Status:** Design approved; ready for implementation planning.

## Goal

Let HD art override off-VRAM ("modern"/assets-anim) tiles and sprites, keyed by the
logical **source key** (`kind, pack, tile_off`), and recolor it through the **live
CGRAM** every frame ("detail-modulated") so HD art stays palette-responsive
(day/night, area swaps, flashes, color-math) — exactly like the palette-indexed base
path.

This is **Phase 1**: the source-key → HD-art plumbing plus the detail-modulate recolor
kernel, integrated at the **native 256×224** output resolution (no added resolution
yet). It is small and parity-safe. **Phase 2** (a follow-on spec) adds the N× render
pass that produces visibly higher-resolution output; it reuses Phase 1's store and
kernel verbatim, changing only the sampler (nearest-native → N×).

### Non-goals (Phase 1)
- N× / higher-resolution output buffers or display plumbing (that is Phase 2).
- Overrides on the classic GPU renderer path (that path is keyed by VRAM `tile_num`
  and already has its own detail-modulate override via `bg_layer.wgsl`; see the
  committed Phase 2 GPU work). This spec is the **modern software** path only.
- Any change to behavior when no override manifest is loaded — that MUST be byte-exact
  with today's output (the parity guarantee).

## Background / current state

The modern software compositor (`crates/renderer/src/modern_software.rs`) renders a
`ModernFrame` to a fixed **256×224** RGBA buffer. Every composite path resolves a pixel
the same way:

```
index = cell[local_y*8 + local_x]        // palette-SLOT index 0..15 (or baked BG3)
if index == 0 { transparent }
else { color = frame.cgram_rgba[cgram_idx] }   // cgram_idx = palette*16 + index
```

- BG paths: `cgram_idx = palette*16 + index`
  (`composite_index_tiles`, `composite_index_tiles_c5`, `composite_mode1_mosaic`,
  `composite_mode1_scanline_scroll`).
- Sprite path: `cgram_idx = 0x80 + palette*16 + index` (`resolve_obj_layer`).

Cells are already keyed by logical source key via `ModernSourceAtlas.key_to_cell`
(`crates/renderer/src/modern_source_atlas.rs`); the `--dump-assets-by-source` JSON
emits `{ id, key, kind, pack, tile_off }` per cell. BG and sprite use **separate** cell
atlases (each with its own `cell_id` space and its own source keys).

A companion reference-palette tool already exists: `--dump-reference-palette <frame>`
writes the live CGRAM as a 256×1 RGBA PNG using `renderer::tile_atlas::
expand_cgram_to_rgba8` (same channel expansion as the live `cgram_palette`). Phase 1
reuses this as the HD authoring palette.

## Architecture

A source-keyed HD override layer sits **between** the loaded atlas and the compositor.
It is loaded once from a manifest, resolved into a per-`cell_id` lookup at atlas-load,
and consulted by a single shared recolor kernel inside each composite path. With no
manifest, all tables are empty and output is byte-identical to today.

```
manifest.json ─┬─> ModernHdOverrides (store: source_key -> HdCell, + reference palette)
               │
   atlas load ─┴─> per-atlas override table  Vec<Option<HdCell>> indexed by cell_id
                     (one for BG atlas, one for sprite atlas)
                        │
   render ──────────────┴─> composite_* / resolve_obj_layer
                              per instance: table[cell_id] -> Option<&HdCell>
                              per pixel:    resolve_pixel_color(...) kernel
```

Four units, each independently testable:

1. **Manifest + loader** — reads/decodes the override art and reference palette.
2. **Override store** (`ModernHdOverrides`) — `source_key -> &HdCell`, plus one global
   reference palette `[[u8;4];256]`.
3. **Per-atlas override table** — `Vec<Option<HdCell>>` indexed by `cell_id`, resolved
   once per atlas from the store via the atlas's `cell_id -> source_key` map.
4. **Recolor kernel** — pure per-pixel detail-modulate.

## Components

### 1. `ModernHdOverrideManifest` (JSON)

```json
{
  "reference_palette": "ref.png",
  "overrides": [
    { "key": "0x0600000012340000", "rgba": "grass_hd.png" }
  ]
}
```

- `key` — the logical source key (u64) as emitted by `--dump-assets-by-source`
  (hex string, `0x`-prefixed).
- `rgba` — path (relative to the manifest) to an RGBA PNG whose dimensions are
  multiples of 8 (an N× upscale of the 8×8 cell; 8×8 == 1× reference).
- `reference_palette` — path to a 256×1 RGBA PNG (from `--dump-reference-palette`), the
  CGRAM the HD art was authored against.

### 2. `ModernHdOverrides` (store)

- Env-gated: `ZELDA3_MODERN_HD_OVERRIDES=<manifest path>`; unset/absent → empty store.
- Decodes each `rgba` PNG into `HdCell { width, height, rgba: Vec<u8> }`.
- Decodes `reference_palette` into `reference: [[u8;4];256]`.
- API: `get(source_key: u64) -> Option<&HdCell>`, `reference() -> &[[u8;4];256]`,
  `is_enabled() -> bool`.

### 3. Per-atlas override table

- Requires `cell_id -> source_key` on the atlas. Add a parallel `cell_keys: Vec<u64>`
  to `ModernSourceAtlas`, populated wherever `key_to_cell` is built (both the real
  loader and `from_keyed_cells_for_test`). The sprite atlas exposes the same.
- At atlas construction, build `Vec<Option<HdCell>>` (clone-or-Rc the `HdCell`) indexed
  by `cell_id`: `table[cell_id] = store.get(cell_keys[cell_id]).cloned()`. Direct index
  at composite time — no per-pixel hashing.

### 4. Recolor kernel (`modern_software`)

```rust
fn resolve_pixel_color(
    base_index: u8,          // slot index, for transparency (shape unchanged by HD art)
    cgram_idx: usize,        // palette*16+index (BG) or 0x80+palette*16+index (OBJ)
    live_rgba: [u8; 4],      // == frame.cgram_rgba[cgram_idx] today
    override_cell: Option<&HdCell>,
    reference: &[[u8; 4]; 256],
    lx: u32, ly: u32,        // local pixel within the 8x8 cell (post-flip)
) -> Option<[u8; 4]> {       // None = transparent
    if base_index == 0 { return None; }
    match override_cell {
        Some(hd) => {
            let hd_rgb = hd.sample_native(lx, ly); // Nx -> 8 nearest (block top-left)
            let refc = reference[cgram_idx];
            // detail = override / reference; final = live * detail (per channel)
            Some(detail_modulate(live_rgba, hd_rgb, refc))
        }
        None => Some(live_rgba),
    }
}
```

- `sample_native`: for an `N*8 × N*8` HD cell, native pixel `(lx, ly)` maps to HD
  block top-left `(lx*N, ly*N)` — nearest, matching the atlas downsample convention.
  At 1× (8×8) this is the identity.
- `detail_modulate`: per channel `clamp(live * (hd / max(reference, 1/255)), 0, 255)`.
  Reference clamped away from zero to avoid divide-by-zero on dark slots.

## Data flow & threading

1. **Load once** → `ModernHdOverrides` store.
2. **At atlas construction** (BG atlas, sprite atlas) → two `Vec<Option<HdCell>>`
   override tables.
3. **Per frame**: `render_modern_frame_full` threads `(&bg_table, &sprite_table,
   reference)` into `composite_index_tiles`, `composite_index_tiles_c5`,
   `composite_mode1_mosaic`, `composite_mode1_scanline_scroll`, and
   `resolve_obj_layer`. Each fn resolves `Option<&HdCell>` **once per instance** by
   `cell_id`, then calls `resolve_pixel_color` per pixel. Because the kernel takes the
   already-computed `cgram_idx`, BG and sprite paths use the identical kernel.

The exact function-signature threading is an implementation detail for the plan; the
constraint is: one shared kernel, per-instance override resolve (not per-pixel), and a
`None`/empty-table fast path that leaves no-override pixels byte-identical to today.

## Error handling

- **No manifest / env unset** → empty store → empty tables → **exact parity**.
- **An `rgba` PNG fails to load, or dims not a multiple of 8** → skip that one override
  (log a warning); other overrides unaffected.
- **Reference palette missing / not 256 px** → **disable overrides entirely** (log)
  rather than mis-recolor; overrides require a reference.

## Testing & acceptance

### Unit (kernel)
- **detail=1 identity**: HD authored == reference → kernel returns `live` exactly.
- **recolor**: HD ≠ reference → `clamp(live * hd/reference)` (with ±1 tolerance only if
  a GPU path is involved; this is CPU, so exact).
- **transparency**: `base_index == 0` → `None` regardless of override.
- **sample_native**: N× cell samples the block top-left; 1× is identity.

### Integration (modern_software)
- A small `ModernFrame` with **a BG override cell and a sprite override cell**:
  - reference == live palette → output **byte-identical** to the no-override render
    (parity via detail=1).
  - reference ≠ live → recolored per the kernel.

### Parity gate
- `zparity` / `scripts/validate_all_parity.py` unchanged: parity runs load no manifest,
  so the override branch never fires. Existing renderer tests (132) stay green.

### Acceptance criteria
- Both BG and sprite source-keyed overrides recolor at native res.
- Parity preserved with no manifest (byte-exact; all tests + gate green).
- Kernel + threading covered by unit and integration tests.
- Store and kernel are structured so the Phase 2 N× pass reuses them unchanged (only
  the sampler switches from nearest-native to N×).

## Phase 2 (out of scope here, for context)

Render the whole modern compositor at N× (buffers `N*256 × N*224`); HD-override cells
sample at full HD detail, non-override cells nearest-upscale; display plumbing consumes
the larger frame. Reuses the Phase 1 manifest/store/kernel; the only kernel change is
`sample_native` → an N× sampler.
