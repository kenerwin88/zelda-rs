# ROM-Derived Editable CHR Assets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first shippable ROM-derived editable CHR sheet pipeline: `extract_assets.py --rom ...` emits CGX-named PNG sheets and sidecar JSON from ROM-derived graphics packs.

**Architecture:** Keep this in the existing Python asset extraction layer. Add a focused `scripts/chr_editable_sheets.py` module that decodes the already-extracted `kSprGfx`/`kBgGfx` pack assets, arranges 64-tile blocks into CGX-informed sheet names, writes normal-color indexed PNGs, and writes sidecars with source-pack provenance. Integrate it into `scripts/extract_assets.py` after asset files are written.

**Tech Stack:** Python 3, `unittest`, Pillow for PNG writes, existing `scripts/extract_assets.py` pack/decompression helpers.

## Global Constraints

- The user-facing extraction path must start from a supported ROM, not from `~/Documents/NES_Ver2`.
- CGX/NES_Ver2 files may guide constants and tests but must not be required at runtime.
- Preserve SNES palette-index semantics through sidecar metadata; PNG colors are preview colors.
- Do not change renderer loading behavior in this first slice.
- Do not stage unrelated existing changes in `crates/renderer/src/modern_source_atlas.rs`.

---

### Task 1: Add CHR Sheet Writer Module

**Files:**
- Create: `scripts/chr_editable_sheets.py`
- Test: `scripts/test_chr_editable_sheets.py`

**Interfaces:**
- Produces: `decode_planar_tile_indices(data: bytes, bpp: int) -> list[bytes]`
- Produces: `pack_tiles_to_sheet(tiles: list[bytes], columns: int) -> tuple[int, int, bytes]`
- Produces: `write_chr_sheet_png(path: Path, tiles: list[bytes], columns: int, palette: list[int]) -> None`
- Produces: `write_chr_sheet_sidecar(path: Path, manifest: dict[str, object]) -> None`

- [ ] **Step 1: Write failing unit tests**

Add `scripts/test_chr_editable_sheets.py` with tests for:

```python
def test_decode_2bpp_single_tile_returns_indices() -> None:
    data = bytes([0x80, 0x00] + [0x00, 0x00] * 7)
    assert chr_editable_sheets.decode_planar_tile_indices(data, 2)[0][0] == 1
```

```python
def test_pack_tiles_to_sheet_places_tiles_left_to_right() -> None:
    tiles = [bytes([1] * 64), bytes([2] * 64)]
    width, height, pixels = chr_editable_sheets.pack_tiles_to_sheet(tiles, columns=2)
    assert (width, height) == (16, 8)
    assert pixels[:8] == bytes([1] * 8)
    assert pixels[8:16] == bytes([2] * 8)
```

- [ ] **Step 2: Run tests and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_chr_editable_sheets.py`

Expected: fail with `ModuleNotFoundError` or missing functions.

- [ ] **Step 3: Implement minimal decoder/sheet writer**

Create `scripts/chr_editable_sheets.py` with planar 2bpp/3bpp/4bpp tile decoding, sheet packing, PNG writing via Pillow, and JSON sidecar writing.

- [ ] **Step 4: Run tests and verify GREEN**

Run: `PYTHONPATH=scripts python3 scripts/test_chr_editable_sheets.py`

Expected: pass.

---

### Task 2: Build CGX-Informed Sheet Layout From ROM Packs

**Files:**
- Modify: `scripts/chr_editable_sheets.py`
- Test: `scripts/test_chr_editable_sheets.py`

**Interfaces:**
- Produces: `CHR_SHEET_BLOCKS: list[ChrSheetBlock]`
- Produces: `build_editable_chr_sheets(sprite_packs: list[DecodedPack], bg_packs: list[DecodedPack]) -> list[EditableChrSheet]`
- Produces: `DecodedPack(kind: str, pack_index: int, bpp: int, tiles: list[bytes])`
- Produces: `EditableChrSheet(name: str, tiles: list[bytes], blocks: list[dict[str, object]])`

- [ ] **Step 1: Write failing layout tests**

Add tests that pass synthetic decoded packs and assert:

```python
def test_build_editable_chr_sheets_uses_cgx_names() -> None:
    packs = [
        chr_editable_sheets.DecodedPack("sprite", i, 3, [bytes([i % 8] * 64)] * 64)
        for i in range(222)
    ]
    sheets = chr_editable_sheets.build_editable_chr_sheets(packs, [])
    assert [sheet.name for sheet in sheets][:3] == ["2m-2q", "2r-2w", "a-h"]
```

```python
def test_build_editable_chr_sheets_preserves_block_provenance() -> None:
    packs = [
        chr_editable_sheets.DecodedPack("sprite", i, 3, [bytes([i % 8] * 64)] * 64)
        for i in range(222)
    ]
    sheet = chr_editable_sheets.build_editable_chr_sheets(packs, [])[1]
    assert sheet.blocks[0]["source_kind"] == "sprite"
    assert sheet.blocks[0]["source_pack"] == 1
    assert sheet.blocks[0]["block"] == "2r-2w.DAT1"
```

- [ ] **Step 2: Run tests and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_chr_editable_sheets.py`

Expected: fail because layout types/functions are missing.

- [ ] **Step 3: Implement CGX-informed layout**

Hardcode the `chr-cnv4.TBL` grouping as sheet/block constants derived from NES_Ver2:
`2m-2q`, `2r-2w`, `a-h`, `i-p`, `q-x`, `y-1f`, `1g-1n`, `1o-1v`, `1w-2d`, `2e-2l`,
`2x-3b`, `3c-3j`, `3k-3r`, `3s-3z`, `4a-4h`, `4i-4p`, `4q-4s`, `4t-4x`.
Map blocks to decoded ROM packs sequentially and include source provenance in sidecars.

- [ ] **Step 4: Run tests and verify GREEN**

Run: `PYTHONPATH=scripts python3 scripts/test_chr_editable_sheets.py`

Expected: pass.

---

### Task 3: Decode Current Asset Bundle Packs Into Editable Sheets

**Files:**
- Modify: `scripts/chr_editable_sheets.py`
- Test: `scripts/test_chr_editable_sheets.py`

**Interfaces:**
- Produces: `read_decoded_chr_packs(asset_dir: Path) -> tuple[list[DecodedPack], list[DecodedPack]]`
- Produces: `write_editable_chr_sheets(asset_dir: Path, out_dir: Path | None = None) -> list[Path]`

- [ ] **Step 1: Write failing asset-dir tests**

Use temporary packed arrays with `extract_assets.pack_arrays` and assert `write_editable_chr_sheets` writes `a-h.png` and `a-h.json`.

- [ ] **Step 2: Run tests and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_chr_editable_sheets.py`

Expected: fail because asset-dir functions are missing.

- [ ] **Step 3: Implement asset-dir reading/writing**

Use `extract_assets.unpack_packed_arrays` and `extract_assets.decomp_asset`. Decode sprite packs as 3bpp except 2048-byte special packs as 2bpp; decode BG packs the same way for 2048-byte special packs. Write to `assets_src/chr/`.

- [ ] **Step 4: Run tests and verify GREEN**

Run: `PYTHONPATH=scripts python3 scripts/test_chr_editable_sheets.py`

Expected: pass.

---

### Task 4: Integrate With `extract_assets.py`

**Files:**
- Modify: `scripts/extract_assets.py`
- Test: `scripts/test_extract_asset_sources.py`

**Interfaces:**
- Consumes: `chr_editable_sheets.write_editable_chr_sheets(out_dir)`
- Produces: extraction manifests include an optional `chr_source_sheets` list.

- [ ] **Step 1: Write failing integration test**

In `scripts/test_extract_asset_sources.py`, create a temporary asset output containing fake `kSprGfx` and `kBgGfx` packed arrays, call the new integration helper, and assert `assets_src/chr/a-h.png` exists.

- [ ] **Step 2: Run test and verify RED**

Run: `PYTHONPATH=scripts python3 scripts/test_extract_asset_sources.py`

Expected: fail because integration helper does not exist.

- [ ] **Step 3: Add integration helper and call it from main extraction**

Import `chr_editable_sheets`. Add `write_chr_source_sheets(out_dir: Path) -> list[dict[str, str]]` in `extract_assets.py`, call it after `write_asset_outputs`, and add the result to the top-level manifest under `chr_source_sheets`.

- [ ] **Step 4: Run tests and verify GREEN**

Run: `PYTHONPATH=scripts python3 scripts/test_extract_asset_sources.py`

Expected: pass.

---

### Task 5: Verify Real Generated Assets

**Files:**
- No source change expected unless verification exposes a bug.

**Interfaces:**
- Consumes: `generated/zelda3_assets/assets/064-kSprGfx.bin`
- Consumes: `generated/zelda3_assets/assets/065-kBgGfx.bin`

- [ ] **Step 1: Run focused tests**

Run:

```bash
PYTHONPATH=scripts python3 scripts/test_chr_editable_sheets.py
PYTHONPATH=scripts python3 scripts/test_extract_asset_sources.py
```

Expected: both pass.

- [ ] **Step 2: Generate editable sheets from existing generated assets**

Run:

```bash
PYTHONPATH=scripts python3 scripts/chr_editable_sheets.py --asset-dir generated/zelda3_assets --out-dir /tmp/zelda3_chr_sheets
```

Expected: writes PNG/JSON files including `/tmp/zelda3_chr_sheets/a-h.png`.

- [ ] **Step 3: Build parity binary**

Run: `cargo build --profile parity -p zelda3-bin`

Expected: build succeeds.

