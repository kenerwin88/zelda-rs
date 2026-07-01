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

### 3. Source key on the per-frame cell (`ModernIndexTile.source_key`)

**Design correction (found during planning):** the compositor's `cell_id` is NOT an
atlas-stable id. `extract_modern_frame_from_sources` / `extract_modern_sprites_from_
sources` build a fresh, per-frame `Vec<ModernIndexTile>` (`bg_cells` / `sprite_cells`),
deduped via a per-frame `cell_ids` map, and `ModernIndexTileInstance.cell_id` indexes
into that per-frame Vec. So an atlas-load table keyed by cell_id cannot work.

Instead, carry the source key on the cell, set at the exact point extract already
resolves the atlas source:

- Add `source_key: u64` to `ModernIndexTile` (`crates/renderer/src/modern_index_atlas.rs`).
  A dedicated `NO_SOURCE_KEY` sentinel (`0`) marks cells with no atlas source (the
  live-VRAM-decoded animation cells, which never have overrides). Real keys are the
  nonzero `modern_source_key(kind, pack, tile_off)`.
- In extract, set `source_key` where each atlas-backed cell is pushed
  (`source_cell(...)` path) using `modern_source_key(kind, pack, tile_off)`; set
  `NO_SOURCE_KEY` for the non-injective live-decode path and any test constructor.
- At composite time the override is resolved **once per instance** (not per pixel):
  `let ov = ctx.and_then(|s| s.get(cell.source_key));`. `ModernSourceAtlas` is
  unchanged; no `cell_keys` needed.

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

1. **Load once** → `ModernHdOverrides` store (env-gated).
2. **Extract** sets `ModernIndexTile.source_key` per cell (atlas key or `NO_SOURCE_KEY`).
3. **Per frame**: a new `render_modern_frame_full_with_overrides(frame, bg_cells,
   sprite_cells, ctx: &HdOverrideCtx)` threads `ctx` into the four full-path resolve
   sites — `composite_index_tiles_c5` (main BG), `render_bg_layer_buf` (mosaic BG),
   `render_bg_layer_torus` (scanline-scroll BG), and `resolve_obj_layer` (sprites) —
   via `composite_mode1`. Each resolves `Option<&HdCell>` **once per instance** from the
   cell's `source_key`, then calls `resolve_pixel_color` per pixel. Because the kernel
   takes the already-computed `cgram_idx`, BG and sprite paths use the identical kernel.
   `HdOverrideCtx { store: Option<&ModernHdOverrides> }`; `store: None` (or a store with
   no matching key) → the kernel returns `live` → byte-identical to today.
4. `render_modern_frame_full(frame, bg_cells, sprite_cells)` stays as a thin wrapper
   that calls `_with_overrides` with a **disabled** ctx (`store: None`), so all existing
   callers/tests are byte-unchanged. The `render_modern_frame_full_from_vram` oracle
   path likewise stays disabled (its cells are live-VRAM-decoded, `NO_SOURCE_KEY`).

The exact function-signature threading is an implementation detail for the plan; the
constraint is: one shared kernel, per-instance override resolve (not per-pixel), and a
disabled-ctx fast path that leaves no-override pixels byte-identical to today.

The **simple** (non-parity) path — `render_modern_frame_software_indexed` /
`draw_modern_sprites_indexed` (resolve sites at lines ~53 and ~180) — is intentionally
NOT covered in Phase 1; the parity/compare path is `render_modern_frame_full`.

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
