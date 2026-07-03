# RGBA Variant Atlas Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace CHR-as-rendered-art with a ROM-traceable base-art atlas plus reusable modern effects first, then layer semantic modder-friendly PNG authoring on top without losing final-pixel parity proof.

**Architecture:** Build generated `base_tiles.png`, `base_tiles.json`, and `tile_effects.json` from ROM-derived CHR and palettes. The base atlas stores each source tile once using palette-usage evidence when available, with a broad source-kind preview palette only as a fallback. The effect table represents recolors, flashes, fades, lighting, and palette-like looks as reusable shader/material transforms. Keep the brute-force `tile_variants.*` artifact only as a diagnostic oracle while the renderer migrates to base art plus effects.

**Tech Stack:** Python 3 asset generation scripts, `unittest`, Pillow/PNG tooling, Rust renderer crate, wgpu, existing `ModernFrame`, `ModernIndexTile`, `ModernSourceAtlas`, `ModernGpuCompositor`, replay parity scripts.

## Global Constraints

- The ROM remains the extraction source of truth. NES_Ver2/CGX files may inform naming and grouping, but the generated atlas must be buildable from a supported ROM plus repo metadata.
- `base_tiles.json` plus `tile_effects.json` is the bridge contract. Runtime and semantic authoring both target that contract.
- Raw CHR bytes do not contain final colors. Any claim that `base_tiles.png` uses the right real palette must be backed by `palette_usage.json` or replay/source-map evidence, not by source-kind guesses.
- Do not remove the current CHR/palette-index path until the base/effect path has frame-level parity proof over representative windows.
- Dynamic palette effects must remain correct. If a palette can change at runtime, either keep it on the existing palette path for that phase or model it as an explicit effect with replay evidence.
- Generated assets under `generated/` stay ignored unless a later task intentionally promotes a small fixture into the repo.
- Keep current dirty work isolated. Do not stage unrelated `crates/renderer/src/modern_source_atlas.rs` changes unless that task explicitly owns them.
- Verification must compare final `256x224` framebuffer pixels, not just atlas entries.

---

## File Structure

- Create: `scripts/rgba_variant_atlas.py`  
  Builds compact base-art atlases, reusable effect tables, and a diagnostic brute-force variant atlas from decoded CHR packs and extracted palette JSON.
- Create: `scripts/test_rgba_variant_atlas.py`  
  Unit tests for palette application, variant keys, deduplication, atlas packing, manifest schema, and dynamic-palette classification.
- Modify: `scripts/extract_assets.py`  
  Adds optional `write_rgba_variant_atlas(out_dir)` integration after palette and CHR source assets exist.
- Modify: `scripts/test_extract_asset_sources.py`  
  Integration test that extraction emits `atlas/base_tiles.png`, `atlas/base_tiles.json`, `atlas/tile_effects.json`, and diagnostic `atlas/tile_variants.*` when graphics and palettes are present.
- Create: `crates/renderer/src/modern_variant_atlas.rs`  
  Rust loader for diagnostic `tile_variants.json`/PNG, later extended to load `base_tiles.json`/PNG and `tile_effects.json`.
- Modify: `crates/renderer/src/lib.rs`  
  Exports `modern_variant_atlas`.
- Modify: `crates/renderer/src/modern_gpu.rs`  
  Adds a variant-atlas compositor/render path after the loader exists.
- Modify: `crates/renderer/src/modern_frame.rs`  
  Adds optional variant identity fields only if Phase 2 proves that existing `cell_id + palette` cannot resolve atlas entries cleanly.
- Modify: `zelda3-bin/src/main.rs`  
  Adds opt-in renderer/compare modes during Phase 2; default changes only after parity gates pass.
- Create: `scripts/semantic_rgba_sheets.py`  
  Generates and compiles semantic modder sheets in Phase 3.
- Create: `scripts/test_semantic_rgba_sheets.py`  
  Tests semantic sheet mapping, coverage validation, and atlas recompilation.
- Create: `docs/assets/rgba-variant-atlas.md`  
  Documents the contract for atlas, sidecar keys, dynamic-palette policy, and modder workflow.

---

## Phase 1: ROM-Derived Base Art and Effect Atlas

Purpose: generate a normal-color base-art atlas while preserving exact source identity, prefer real palette usage evidence for previews, and represent alternate palette rows as reusable effects instead of baking them into hundreds of thousands of duplicate recolors. This phase does not change runtime rendering.

### Task 1: Define Variant Keys and Palette Application

**Files:**
- Create: `scripts/rgba_variant_atlas.py`
- Test: `scripts/test_rgba_variant_atlas.py`

**Interfaces:**
- Produces: `VariantKey(source_kind: str, asset: str, pack: int, tile: int, bpp: int, palette: str, palette_row: int)`.
- Produces: `rgba_tile_from_indices(indices: bytes, palette_colors: list[list[int]], palette_row: int, colors_per_row: int) -> bytes`.
- Produces: `variant_id(key: VariantKey) -> str`.

- [ ] **Step 1: Write failing tests for palette application**

Add `scripts/test_rgba_variant_atlas.py`:

```python
import unittest

from rgba_variant_atlas import VariantKey, rgba_tile_from_indices, variant_id


class RgbaVariantAtlasTests(unittest.TestCase):
    def test_rgba_tile_from_indices_applies_palette_row(self) -> None:
        indices = bytes([0, 1, 2, 3] + [0] * 60)
        colors = [[i, i + 1, i + 2] for i in range(32)]

        rgba = rgba_tile_from_indices(indices, colors, palette_row=1, colors_per_row=16)

        self.assertEqual(list(rgba[:16]), [
            16, 17, 18, 0,
            17, 18, 19, 255,
            18, 19, 20, 255,
            19, 20, 21, 255,
        ])

    def test_variant_id_is_stable_and_readable(self) -> None:
        key = VariantKey("sprite", "kSprGfx", 12, 37, 3, "palette_main_spr", 3)

        self.assertEqual(
            variant_id(key),
            "sprite:kSprGfx:pack12:tile37:3bpp:palette_main_spr:row3",
        )


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run tests and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_rgba_variant_atlas.py`

Expected: fail because `rgba_variant_atlas` does not exist.

- [ ] **Step 3: Implement minimal key and RGBA conversion**

Create `scripts/rgba_variant_atlas.py`:

```python
#!/usr/bin/env python3
from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class VariantKey:
    source_kind: str
    asset: str
    pack: int
    tile: int
    bpp: int
    palette: str
    palette_row: int


def variant_id(key: VariantKey) -> str:
    return (
        f"{key.source_kind}:{key.asset}:pack{key.pack}:tile{key.tile}:"
        f"{key.bpp}bpp:{key.palette}:row{key.palette_row}"
    )


def rgba_tile_from_indices(
    indices: bytes,
    palette_colors: list[list[int]],
    palette_row: int,
    colors_per_row: int,
) -> bytes:
    if len(indices) != 64:
        raise ValueError("RGBA variant tiles must be built from one 8x8 index tile")
    base = palette_row * colors_per_row
    out = bytearray()
    for index in indices:
        color_index = base + index
        if color_index >= len(palette_colors):
            raise ValueError(f"palette index {color_index} outside palette with {len(palette_colors)} colors")
        r, g, b = palette_colors[color_index]
        alpha = 0 if index == 0 else 255
        out.extend([r, g, b, alpha])
    return bytes(out)
```

- [ ] **Step 4: Run tests and verify GREEN**

Run: `PYTHONPATH=scripts python3 scripts/test_rgba_variant_atlas.py`

Expected: 2 tests pass.

- [ ] **Step 5: Commit**

```bash
git add scripts/rgba_variant_atlas.py scripts/test_rgba_variant_atlas.py
git commit -m "feat: define RGBA variant atlas keys"
```

### Task 2: Pack RGBA Variants and Deduplicate Identical Tiles

**Files:**
- Modify: `scripts/rgba_variant_atlas.py`
- Modify: `scripts/test_rgba_variant_atlas.py`

**Interfaces:**
- Produces: `RgbaVariant(key: VariantKey, pixels: bytes)`.
- Produces: `AtlasEntry(id: str, key: VariantKey, rect: tuple[int, int, int, int], sha1: str, duplicate_of: str | None)`.
- Produces: `pack_rgba_variants(variants: list[RgbaVariant], columns: int = 32) -> tuple[int, int, bytes, list[AtlasEntry]]`.

- [ ] **Step 1: Write failing tests for dedupe and packing**

Append tests:

```python
from rgba_variant_atlas import RgbaVariant, pack_rgba_variants


def solid_rgba(value: int) -> bytes:
    return bytes([value, value, value, 255] * 64)


class RgbaVariantAtlasPackTests(unittest.TestCase):
    def test_pack_rgba_variants_deduplicates_identical_pixels(self) -> None:
        key_a = VariantKey("sprite", "kSprGfx", 1, 2, 3, "palette_main_spr", 0)
        key_b = VariantKey("sprite", "kSprGfx", 1, 3, 3, "palette_main_spr", 0)
        width, height, pixels, entries = pack_rgba_variants(
            [RgbaVariant(key_a, solid_rgba(7)), RgbaVariant(key_b, solid_rgba(7))],
            columns=2,
        )

        self.assertEqual((width, height), (16, 8))
        self.assertEqual(len(pixels), 16 * 8 * 4)
        self.assertIsNone(entries[0].duplicate_of)
        self.assertEqual(entries[1].duplicate_of, entries[0].id)
        self.assertEqual(entries[1].rect, entries[0].rect)
```

- [ ] **Step 2: Run tests and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_rgba_variant_atlas.py`

Expected: fail because `RgbaVariant` and `pack_rgba_variants` are missing.

- [ ] **Step 3: Implement atlas packing**

Add dataclasses and packer:

```python
import hashlib


@dataclass(frozen=True)
class RgbaVariant:
    key: VariantKey
    pixels: bytes


@dataclass(frozen=True)
class AtlasEntry:
    id: str
    key: VariantKey
    rect: tuple[int, int, int, int]
    sha1: str
    duplicate_of: str | None


def pack_rgba_variants(
    variants: list[RgbaVariant],
    columns: int = 32,
) -> tuple[int, int, bytes, list[AtlasEntry]]:
    if columns <= 0:
        raise ValueError("columns must be positive")
    unique_pixels: list[bytes] = []
    sha_to_rect: dict[str, tuple[str, tuple[int, int, int, int]]] = {}
    entries: list[AtlasEntry] = []
    for variant in variants:
        if len(variant.pixels) != 8 * 8 * 4:
            raise ValueError("each RGBA variant must be one 8x8 RGBA tile")
        digest = hashlib.sha1(variant.pixels).hexdigest()
        entry_id = variant_id(variant.key)
        if digest in sha_to_rect:
            original_id, rect = sha_to_rect[digest]
            entries.append(AtlasEntry(entry_id, variant.key, rect, digest, original_id))
            continue
        unique_index = len(unique_pixels)
        x = (unique_index % columns) * 8
        y = (unique_index // columns) * 8
        rect = (x, y, 8, 8)
        unique_pixels.append(variant.pixels)
        sha_to_rect[digest] = (entry_id, rect)
        entries.append(AtlasEntry(entry_id, variant.key, rect, digest, None))

    rows = max(1, (len(unique_pixels) + columns - 1) // columns)
    width = columns * 8
    height = rows * 8
    atlas = bytearray(width * height * 4)
    for unique_index, tile in enumerate(unique_pixels):
        x = (unique_index % columns) * 8
        y = (unique_index // columns) * 8
        for row in range(8):
            dst = ((y + row) * width + x) * 4
            src = row * 8 * 4
            atlas[dst : dst + 8 * 4] = tile[src : src + 8 * 4]
    return width, height, bytes(atlas), entries
```

- [ ] **Step 4: Run tests and verify GREEN**

Run: `PYTHONPATH=scripts python3 scripts/test_rgba_variant_atlas.py`

Expected: tests pass.

- [ ] **Step 5: Commit**

```bash
git add scripts/rgba_variant_atlas.py scripts/test_rgba_variant_atlas.py
git commit -m "feat: pack RGBA tile variants into deduped atlas"
```

### Task 3: Generate Atlas From Extracted ROM Assets

**Files:**
- Modify: `scripts/rgba_variant_atlas.py`
- Modify: `scripts/test_rgba_variant_atlas.py`
- Modify: `scripts/extract_assets.py`
- Modify: `scripts/test_extract_asset_sources.py`

**Interfaces:**
- Consumes: `chr_editable_sheets.read_decoded_chr_packs(asset_dir: Path)`.
- Consumes: extracted palette JSON under `assets_src/palettes/*.json`.
- Produces: `build_rom_variant_atlas(asset_dir: Path, palette_names: list[str]) -> tuple[int, int, bytes, list[AtlasEntry]]`.
- Produces: `write_rom_variant_atlas(asset_dir: Path, out_dir: Path | None = None) -> list[Path]`.
- Produces manifest paths: `atlas/tile_variants.png`, `atlas/tile_variants.json`.

- [ ] **Step 1: Write failing unit test using temp extracted assets**

In `scripts/test_rgba_variant_atlas.py`, create minimal packed sprite/BG assets using helpers copied from `scripts/test_chr_editable_sheets.py`, write `palette_main_spr.json`, then assert `write_rom_variant_atlas` emits both files and entry IDs include `palette_main_spr`.

- [ ] **Step 2: Run tests and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_rgba_variant_atlas.py`

Expected: fail because ROM atlas writer is missing.

- [ ] **Step 3: Implement ROM atlas writer**

Implementation requirements:
- Load decoded CHR packs with `chr_editable_sheets.read_decoded_chr_packs`.
- Load palette JSON with the same `rgb888` parser used by `chr_editable_sheets`.
- For 3bpp tiles, generate rows `0..7` for sprite palettes by default.
- For 4bpp tiles, generate rows `0..15`.
- For 2bpp tiles, generate rows `0..3`.
- Record `source_kind`, `asset`, `pack`, `tile`, `bpp`, `palette`, `palette_row`, `rect`, `sha1`, and `duplicate_of` in `tile_variants.json`.
- Save `tile_variants.png` as RGBA, not indexed.

- [ ] **Step 4: Add extraction integration**

In `scripts/extract_assets.py`, add:

```python
def write_rgba_variant_atlas(out_dir: Path) -> list[dict[str, str]]:
    import rgba_variant_atlas

    try:
        written = rgba_variant_atlas.write_rom_variant_atlas(out_dir)
    except FileNotFoundError as exc:
        print(f"skipping RGBA variant atlas: missing {exc.filename}", file=sys.stderr)
        return []
    if not written:
        return []
    return [{
        "image_file": "atlas/tile_variants.png",
        "manifest_file": "atlas/tile_variants.json",
        "source_format": "zelda3_rgba_variant_atlas_v1",
    }]
```

Call it after `write_chr_source_sheets(out_dir)` and add the result to top-level `manifest.json` as `rgba_variant_atlas`.

- [ ] **Step 5: Run tests and real extraction**

Run:

```bash
PYTHONPATH=scripts python3 scripts/test_rgba_variant_atlas.py
PYTHONPATH=scripts python3 scripts/test_extract_asset_sources.py
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets
```

Expected:
- tests pass
- extraction prints that `atlas/tile_variants.png` and `atlas/tile_variants.json` were written
- `generated/zelda3_assets/atlas/tile_variants.png` is RGBA

- [ ] **Step 6: Commit**

```bash
git add scripts/rgba_variant_atlas.py scripts/test_rgba_variant_atlas.py scripts/extract_assets.py scripts/test_extract_asset_sources.py
git commit -m "feat: generate ROM-derived RGBA variant atlas"
```

### Task 4: Classify Dynamic Palette Risk

**Files:**
- Modify: `scripts/rgba_variant_atlas.py`
- Modify: `scripts/test_rgba_variant_atlas.py`
- Create: `docs/assets/rgba-variant-atlas.md`

**Interfaces:**
- Produces: `dynamic_policy` field in `tile_variants.json`: `"stable" | "requires_live_palette"`.
- Produces: `classify_palette_policy(palette_name: str) -> str`.

- [ ] **Step 1: Write failing dynamic classification tests**

Add tests:

```python
from rgba_variant_atlas import classify_palette_policy


class DynamicPalettePolicyTests(unittest.TestCase):
    def test_known_static_palettes_are_stable(self) -> None:
        self.assertEqual(classify_palette_policy("palette_main_spr"), "stable")
        self.assertEqual(classify_palette_policy("palette_dung_bg_main"), "stable")

    def test_unknown_palette_requires_live_palette_until_proven(self) -> None:
        self.assertEqual(classify_palette_policy("palette_runtime_flash"), "requires_live_palette")
```

- [ ] **Step 2: Run tests and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_rgba_variant_atlas.py`

Expected: fail because classifier is missing.

- [ ] **Step 3: Implement conservative classifier**

Only mark palettes stable when they are extracted static palette assets used by the atlas generator. Unknown names and runtime-captured palettes return `"requires_live_palette"`.

- [ ] **Step 4: Document the contract**

Create `docs/assets/rgba-variant-atlas.md` with:
- atlas file locations
- JSON schema
- alpha rule: source index `0` becomes alpha `0`; nonzero indices become alpha `255`
- dynamic palette policy
- parity rule: atlas path must match final framebuffer pixels, not just source tile colors

- [ ] **Step 5: Run tests**

Run:

```bash
PYTHONPATH=scripts python3 scripts/test_rgba_variant_atlas.py
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets
```

Expected: tests pass and atlas JSON entries include `dynamic_policy`.

- [ ] **Step 6: Commit**

```bash
git add scripts/rgba_variant_atlas.py scripts/test_rgba_variant_atlas.py docs/assets/rgba-variant-atlas.md
git commit -m "docs: define RGBA variant atlas contract"
```

---

## Phase 2: Renderer Consumes Variant Atlas

Purpose: render from pre-colored RGBA atlas variants instead of CHR indices for stable art, with the current renderer as oracle.

### Task 5: Add Rust Variant Atlas Loader

**Files:**
- Create: `crates/renderer/src/modern_variant_atlas.rs`
- Modify: `crates/renderer/src/lib.rs`

**Interfaces:**
- Produces: `VariantAtlasKey { source_kind: String, asset: String, pack: u16, tile: u16, bpp: u8, palette: String, palette_row: u8 }`.
- Produces: `VariantAtlasEntry { id: String, key: VariantAtlasKey, rect: [u32; 4], sha1: String, duplicate_of: Option<String>, dynamic_policy: String }`.
- Produces: `ModernVariantAtlas { width: u32, height: u32, rgba: Vec<u8>, entries: Vec<VariantAtlasEntry> }`.
- Produces: `load_modern_variant_atlas(repo_root: &Path) -> Result<ModernVariantAtlas, String>`.

- [ ] **Step 1: Write failing loader tests**

Add tests in `modern_variant_atlas.rs` that create a temp `atlas/tile_variants.json` and `atlas/tile_variants.png`, then assert the loader reads width, height, RGBA bytes, and entries.

- [ ] **Step 2: Run tests and verify RED**

Run: `cargo test -p renderer modern_variant_atlas`

Expected: compile failure because module/export does not exist.

- [ ] **Step 3: Implement loader**

Implementation requirements:
- Use `serde::Deserialize` for JSON.
- Use the `png` crate already used by `modern_source_atlas.rs`.
- Reject non-RGBA PNGs with a clear error.
- Store RGBA bytes in row-major order.
- Add `pub mod modern_variant_atlas;` to `crates/renderer/src/lib.rs`.

- [ ] **Step 4: Run tests and build**

Run:

```bash
cargo test -p renderer modern_variant_atlas
cargo build --profile parity -p zelda3-bin
```

Expected: tests pass and parity-profile build succeeds.

- [ ] **Step 5: Commit**

```bash
git add crates/renderer/src/modern_variant_atlas.rs crates/renderer/src/lib.rs
git commit -m "feat(renderer): load RGBA variant atlas"
```

### Task 6: Add Variant Atlas Software Resolver

**Files:**
- Modify: `crates/renderer/src/modern_variant_atlas.rs`
- Modify: `crates/renderer/src/modern_software.rs`

**Interfaces:**
- Produces: `variant_key_for_index_tile(cell: &ModernIndexTile, palette_name: &str, palette_row: u8) -> Option<VariantAtlasKey>` if source provenance exists.
- Produces: `render_modern_frame_software_variant_atlas(frame: &ModernFrame, atlas: &ModernVariantAtlas) -> Vec<u8>` for stable entries.

- [ ] **Step 1: Write failing software resolver test**

Create a small `ModernFrame` with one BG `ModernIndexTileInstance`, one variant atlas entry, and assert the software variant renderer returns the same pixel as `render_modern_frame_software_indexed`.

- [ ] **Step 2: Run test and verify RED**

Run: `cargo test -p renderer variant_atlas_software`

Expected: fail because renderer function is missing.

- [ ] **Step 3: Implement software resolver**

Implementation requirements:
- Resolve only entries with `dynamic_policy == "stable"`.
- Return an explicit miss count or diagnostics struct for entries that require live palette or have no atlas match.
- Preserve alpha compositing semantics for index `0` transparency by using the atlas RGBA alpha channel.

- [ ] **Step 4: Run tests**

Run: `cargo test -p renderer variant_atlas_software`

Expected: pass.

- [ ] **Step 5: Commit**

```bash
git add crates/renderer/src/modern_variant_atlas.rs crates/renderer/src/modern_software.rs
git commit -m "feat(renderer): resolve stable draws through variant atlas"
```

### Task 7: Add Opt-In GPU Variant Renderer and Compare Mode

**Files:**
- Modify: `crates/renderer/src/modern_gpu.rs`
- Modify: `zelda3-bin/src/main.rs`
- Modify: `scripts/gpu_render_compare_windows.py`

**Interfaces:**
- Produces renderer mode: `ZELDA3_RENDERER=assets-variant-gpu`.
- Produces compare option: `--modern-variant-compare`.

- [ ] **Step 1: Write failing renderer unit test**

Add a `modern_gpu` test that uploads a tiny RGBA variant atlas, renders one tile, reads back RGBA, and asserts the exact color.

- [ ] **Step 2: Run test and verify RED**

Run: `cargo test -p renderer modern_gpu_variant`

Expected: fail because GPU variant path is missing.

- [ ] **Step 3: Implement GPU variant draw path**

Implementation requirements:
- Upload `tile_variants.png` as an `Rgba8Unorm` texture.
- Instances carry atlas rect and screen position.
- The shader samples pre-colored RGBA and alpha-blends like the current simple z-order path.
- Keep current GPU palette renderer unchanged.

- [ ] **Step 4: Wire opt-in mode**

In `zelda3-bin/src/main.rs`, add `assets-variant-gpu` as an explicit mode only. Do not make it default in this task.

- [ ] **Step 5: Add compare harness**

Add `--modern-variant-compare` or script support that compares:
- current final framebuffer
- variant-atlas final framebuffer
- miss count
- dynamic-palette fallback count

- [ ] **Step 6: Run verification**

Run:

```bash
cargo test -p renderer modern_gpu_variant
cargo build --profile parity -p zelda3-bin
python3 scripts/gpu_render_compare_windows.py --renderer assets-variant-gpu --max-windows 3
```

Expected: unit test passes; build passes; compare script reports mismatch counts and fallback counts without crashing.

- [ ] **Step 7: Commit**

```bash
git add crates/renderer/src/modern_gpu.rs zelda3-bin/src/main.rs scripts/gpu_render_compare_windows.py
git commit -m "feat(renderer): add opt-in RGBA variant atlas renderer"
```

### Task 8: Widen Parity Coverage and Default Stable Draws

**Files:**
- Modify: `scripts/gpu_render_compare_windows.py`
- Modify: `scripts/gpu_render_compare_oracle_windows.py`
- Modify: `zelda3-bin/src/main.rs`

**Interfaces:**
- Produces thresholded parity report: stable atlas draws, fallback draws, mismatched pixels by frame.
- Produces controlled default: stable atlas path can become the default inside modern GPU mode only after this task passes.

- [ ] **Step 1: Add parity report fields**

Report per frame:
- `variant_draws`
- `fallback_draws`
- `dynamic_palette_draws`
- `missing_variant_draws`
- `mismatch_pixels`

- [ ] **Step 2: Run representative windows**

Run:

```bash
python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv
```

Expected: report completes and identifies the first remaining mismatch/fallback class.

- [ ] **Step 3: Fix only missing stable-variant coverage**

If the report shows missing variants for stable palettes, update Phase 1 atlas generation so those variants are emitted. Do not force dynamic-palette effects into static atlas entries.

- [ ] **Step 4: Repeat until stable-variant fallback is zero for selected windows**

Run the oracle-window command after each fix.

Expected: stable-variant fallback count reaches zero for selected windows; dynamic-palette fallback may remain.

- [ ] **Step 5: Commit**

```bash
git add scripts/gpu_render_compare_windows.py scripts/gpu_render_compare_oracle_windows.py zelda3-bin/src/main.rs scripts/rgba_variant_atlas.py
git commit -m "test: widen RGBA variant atlas parity coverage"
```

---

## Phase 3: Semantic RGBA Authoring Compiles Into Atlas

Purpose: make the human-editable source assets normal RGBA PNGs while preserving `tile_variants.json` as the runtime contract.

### Task 9: Generate Initial Semantic Sheets From Atlas Provenance

**Files:**
- Create: `scripts/semantic_rgba_sheets.py`
- Create: `scripts/test_semantic_rgba_sheets.py`

**Interfaces:**
- Produces: `SemanticFrame(id: str, source_rect: tuple[int, int, int, int], emits: list[str])`.
- Produces: `write_initial_semantic_sheets(asset_dir: Path, out_dir: Path | None = None) -> list[Path]`.
- Output: `assets_src/semantic/sprites/*.png`, `assets_src/semantic/sprites/*.json`.

- [ ] **Step 1: Write failing semantic export test**

Test with a tiny atlas manifest containing two sprite variants and assert the generated semantic sheet contains both rects and sidecar `emits` IDs.

- [ ] **Step 2: Run test and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_semantic_rgba_sheets.py`

Expected: fail because module is missing.

- [ ] **Step 3: Implement initial semantic export**

Implementation requirements:
- Group by `source_kind` and `asset`.
- Start with mechanical names such as `sprite_kSprGfx_pack12.png`.
- Preserve exact `emits` links to variant IDs.
- Do not invent object names such as `guards` until coverage tooling can prove the mapping.

- [ ] **Step 4: Run tests and real export**

Run:

```bash
PYTHONPATH=scripts python3 scripts/test_semantic_rgba_sheets.py
PYTHONPATH=scripts python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets
```

Expected: test passes and semantic sheets are written under `generated/zelda3_assets/assets_src/semantic`.

- [ ] **Step 5: Commit**

```bash
git add scripts/semantic_rgba_sheets.py scripts/test_semantic_rgba_sheets.py
git commit -m "feat: export initial semantic RGBA sheets"
```

### Task 10: Compile Semantic Sheets Back Into Variant Atlas

**Files:**
- Modify: `scripts/semantic_rgba_sheets.py`
- Modify: `scripts/test_semantic_rgba_sheets.py`

**Interfaces:**
- Produces: `compile_semantic_sheets(asset_dir: Path, semantic_dir: Path) -> list[RgbaVariant]`.
- Produces coverage error type with fields `missing_variant_ids`, `duplicate_variant_ids`, `rect_out_of_bounds`.

- [ ] **Step 1: Write failing compile test**

Create a temp semantic PNG and JSON where one frame emits one variant ID. Assert `compile_semantic_sheets` returns an `RgbaVariant` with matching pixels and key.

- [ ] **Step 2: Run test and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_semantic_rgba_sheets.py`

Expected: fail because compiler is missing.

- [ ] **Step 3: Implement compiler**

Implementation requirements:
- Read semantic PNG as RGBA.
- Crop each frame rect.
- Emit pixels under the exact original `VariantKey`.
- Fail if a required variant ID is not covered exactly once.
- Reuse `pack_rgba_variants` so output atlas format is unchanged.

- [ ] **Step 4: Run tests**

Run: `PYTHONPATH=scripts python3 scripts/test_semantic_rgba_sheets.py`

Expected: tests pass.

- [ ] **Step 5: Commit**

```bash
git add scripts/semantic_rgba_sheets.py scripts/test_semantic_rgba_sheets.py
git commit -m "feat: compile semantic RGBA sheets into variant atlas"
```

### Task 11: Add Modded Atlas Build Mode

**Files:**
- Modify: `scripts/extract_assets.py`
- Modify: `scripts/semantic_rgba_sheets.py`
- Modify: `docs/assets/rgba-variant-atlas.md`

**Interfaces:**
- Produces command: `python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --compile`.
- Produces output: `generated/zelda3_assets/atlas/tile_variants.png` and JSON from semantic sources.

- [ ] **Step 1: Add end-to-end test with a temp semantic edit**

Test flow:
1. Build tiny baseline atlas.
2. Export semantic sheet.
3. Modify one semantic pixel in the PNG.
4. Compile semantic sheets.
5. Assert compiled atlas has the modified pixel in the emitted variant rect.

- [ ] **Step 2: Run test and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_semantic_rgba_sheets.py`

Expected: fail until compile mode writes atlas files.

- [ ] **Step 3: Implement compile CLI**

Add CLI args:
- `--asset-dir`
- `--semantic-dir`
- `--compile`
- `--out-dir`

Default `semantic_dir` to `asset_dir/assets_src/semantic`; default `out_dir` to `asset_dir/atlas`.

- [ ] **Step 4: Update docs**

Document the modder loop:

```bash
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets
# edit generated/zelda3_assets/assets_src/semantic/**/*.png
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --compile
ZELDA3_RENDERER=assets-variant-gpu cargo run --profile parity -p zelda3-bin
```

- [ ] **Step 5: Run tests**

Run:

```bash
PYTHONPATH=scripts python3 scripts/test_semantic_rgba_sheets.py
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --compile
```

Expected: tests pass and compile command writes atlas files.

- [ ] **Step 6: Commit**

```bash
git add scripts/semantic_rgba_sheets.py scripts/test_semantic_rgba_sheets.py docs/assets/rgba-variant-atlas.md
git commit -m "feat: compile modder RGBA sheets into runtime atlas"
```

---

## Phase 4: Native Modder Workflow and Default Path

Purpose: make semantic RGBA sheets the preferred workflow while keeping atlas parity gates and ROM fallback available.

### Task 12: Coverage Validator for Semantic Sources

**Files:**
- Modify: `scripts/semantic_rgba_sheets.py`
- Modify: `scripts/test_semantic_rgba_sheets.py`

**Interfaces:**
- Produces command: `python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --validate`.
- Produces machine-readable report: `assets_src/semantic/coverage_report.json`.

- [ ] **Step 1: Write failing validation test**

Create a temp baseline atlas with three variants and semantic sheets covering two. Assert validation fails with the missing third ID and writes `coverage_report.json`.

- [ ] **Step 2: Run test and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_semantic_rgba_sheets.py`

Expected: fail because validation mode is missing.

- [ ] **Step 3: Implement validator**

Validation rules:
- Every stable variant ID must be covered exactly once.
- Dynamic-policy variants may be absent only if report lists them under `dynamic_fallback`.
- Rects must be inside PNG bounds.
- Duplicate `emits` IDs are errors.

- [ ] **Step 4: Run validator on real generated assets**

Run:

```bash
PYTHONPATH=scripts python3 scripts/test_semantic_rgba_sheets.py
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --validate
```

Expected: tests pass; real validation writes report with stable coverage counts.

- [ ] **Step 5: Commit**

```bash
git add scripts/semantic_rgba_sheets.py scripts/test_semantic_rgba_sheets.py
git commit -m "feat: validate semantic RGBA sheet coverage"
```

### Task 13: Promote Variant Atlas Runtime as the Modern GPU Default

**Files:**
- Modify: `zelda3-bin/src/main.rs`
- Modify: `crates/renderer/src/renderer_mode.rs`
- Modify: `docs/assets/rgba-variant-atlas.md`

**Interfaces:**
- Default modern GPU path uses atlas variants for stable draws.
- Explicit fallback env remains available: `ZELDA3_RENDERER=assets-anim-gpu` or `ZELDA3_VARIANT_ATLAS=off`.

- [ ] **Step 1: Add failing mode-selection test**

Add or update existing mode-selection tests so no env chooses the full modern GPU path with variant atlas enabled, while explicit fallback env selects the old path.

- [ ] **Step 2: Run test and verify RED**

Run: `cargo test -p zelda3-bin renderer_mode`

Expected: fail until default selection changes.

- [ ] **Step 3: Change default only behind parity gate**

Change default selection after Phase 2 parity reports show:
- zero missing stable variants in selected oracle windows
- zero variant-renderer crashes
- known dynamic fallback count documented

- [ ] **Step 4: Run verification**

Run:

```bash
cargo test -p renderer
cargo build --profile parity -p zelda3-bin
python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv
```

Expected: tests/build pass and oracle compare is no worse than the accepted Phase 2 report.

- [ ] **Step 5: Commit**

```bash
git add zelda3-bin/src/main.rs crates/renderer/src/renderer_mode.rs docs/assets/rgba-variant-atlas.md
git commit -m "feat: default modern GPU renderer to RGBA variant atlas"
```

### Task 14: Replace CHR-Facing Docs With Modder Workflow

**Files:**
- Modify: `docs/assets/readable-sources.md`
- Modify: `docs/assets/rgba-variant-atlas.md`
- Create: `docs/assets/modder-rgba-workflow.md`

**Interfaces:**
- Produces human workflow docs that do not require understanding CHR, CGRAM, or CGX to replace stable art.

- [ ] **Step 1: Write workflow doc**

Create `docs/assets/modder-rgba-workflow.md` with:
- how to extract assets
- where semantic PNGs live
- how to edit
- how to validate
- how to compile
- how to run the atlas renderer
- what dynamic-palette fallback means

- [ ] **Step 2: Update existing readable source docs**

Point CHR docs at the new semantic workflow as the preferred route. Keep CHR docs as implementation notes.

- [ ] **Step 3: Run documentation sanity commands**

Run:

```bash
rg -n "NEEDS_DECISION|NOT_YET_DEFINED|CHR-only|palette blob" docs/assets
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --validate
```

Expected: no unresolved placeholder language; validator still runs.

- [ ] **Step 4: Commit**

```bash
git add docs/assets/readable-sources.md docs/assets/rgba-variant-atlas.md docs/assets/modder-rgba-workflow.md
git commit -m "docs: document semantic RGBA modder workflow"
```

---

## Final Verification Gate

Run after all four phases:

```bash
PYTHONPATH=scripts python3 scripts/test_rgba_variant_atlas.py
PYTHONPATH=scripts python3 scripts/test_semantic_rgba_sheets.py
PYTHONPATH=scripts python3 scripts/test_extract_asset_sources.py
python3 scripts/extract_assets.py --rom saves/zelda3.sfc --out-dir generated/zelda3_assets
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --validate
python3 scripts/semantic_rgba_sheets.py --asset-dir generated/zelda3_assets --compile
cargo test -p renderer
cargo build --profile parity -p zelda3-bin
python3 scripts/gpu_render_compare_oracle_windows.py --renderer assets-variant-gpu --windows docs/porting/oracle_windows.tsv
```

Expected:
- Python tests pass.
- Extractor writes `atlas/tile_variants.png` and `atlas/tile_variants.json`.
- Semantic validator reports all stable variants covered exactly once.
- Semantic compiler rewrites the atlas without schema drift.
- Renderer tests pass.
- Parity-profile build succeeds.
- Oracle-window compare reports zero missing stable variants and only documented dynamic-palette fallbacks.

## Migration Decision Points

- After Phase 1, stop if the atlas explodes in size or dynamic palette classification is too broad. The fallback is to keep indexed CHR sheets plus RGB previews.
- After Phase 2, stop if final-pixel parity regresses. The atlas remains useful as a preview/modding artifact even if runtime stays palette-indexed.
- After Phase 3, stop if semantic grouping is too mechanical. The compiled atlas still works, and semantic naming can improve incrementally.
- Phase 4 only happens after stable atlas parity is proven on representative replay/oracle windows.

## Self-Review Notes

- This plan has no dependency on NES_Ver2 at runtime.
- Each phase leaves a working artifact even if later phases are deferred.
- The runtime contract remains `tile_variants.json`, so semantic authoring cannot silently bypass provenance.
- Dynamic palette behavior is intentionally conservative until replay evidence proves a palette row is stable.
