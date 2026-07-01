# Palette-Indexed Modern BG Rendering Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the modern renderer correct per-frame BG tile *colors* by storing tiles as palette-INDEX patterns and applying the live CGRAM at render time (instead of the current baked-RGBA atlas).

**Architecture:** Today's overworld atlas (`overworld_unique_tiles.png/json`) bakes a fixed palette into RGBA and dedups by rendered appearance, so it cannot match runtime palette changes (area/day-night/swaps). This plan adds a parallel **index atlas**: each unique tile *graphics* pattern (tile_number + hflip + vflip, palette-agnostic) is stored as 64 4-bit indices (0–15). At render time the tile's palette (from the tilemap word's bits 10–12) selects a 16-color slice of the live CGRAM; color = `CGRAM[palette*16 + index]`, index 0 = transparent. Both the software oracle and the GPU renderer apply the same lookup, preserving the existing byte-exact software↔GPU contract.

**Tech Stack:** Rust, `wgpu`, the existing modern renderer modules (`crates/renderer/src/modern_*`), the existing atlas-dump command in `zelda3-bin/src/main.rs`.

**Scope:** BG tile color parity ONLY. Out of scope (separate future plans): sprite/OAM rendering, color math, window masking, mosaic, brightness, BG-vs-sprite priority interleaving. Those remain reasons the full-frame parity gate stays red; this plan removes the *palette* reason for BG tiles.

## Global Constraints

- Preserve the fixed 256×224 render target as the parity boundary.
- Keep the existing classic renderer the DEFAULT; the index path is additive/opt-in (reuses the `ModernFrame`/`RendererMode` machinery).
- The software renderer is the oracle; the GPU renderer MUST match it byte-for-byte on tested cases (`assert_eq!` over the full buffer, never weakened).
- CGRAM (BGR15) → RGBA8 conversion MUST exactly match the classic renderer `cgram_to_wgpu_color` (crates/renderer/src/gpu_renderer.rs:554): for a `u16` entry, `r=((entry & 0x1F)<<3)`, `g=(((entry>>5)&0x1F)<<3)`, `b=(((entry>>10)&0x1F)<<3)`, `a=0xff`. (Each 5-bit channel left-shifted by 3; no +smear. Replicate EXACTLY — any rounding difference breaks parity.)
- Index 0 of any palette is transparent (skip the pixel; backdrop/lower layer shows).
- New committed files only follow the project convention; edits to pre-existing shared files (lib.rs, main.rs) are left uncommitted if they carry unrelated WIP (confirm with the controller at execution time).
- Every task adds a focused test before production changes (TDD).

---

## File Structure

- Create `crates/renderer/src/modern_palette.rs`
  - `snes_cgram_to_rgba(entry: u16) -> [u8;4]` (the exact conversion above) and `cgram_words_to_rgba256(cgram: &[u16]) -> [[u8;4];256]`.
- Create `crates/renderer/src/modern_index_atlas.rs`
  - `ModernIndexTile` (64 indices + dims), `ModernIndexAtlas` (cells + `graphics_key → cell` map), `load_modern_overworld_index_atlas(repo_root) -> Result<ModernIndexAtlas,String>`, `index_cell_for_tilemap_entry(&atlas, word) -> Option<&ModernIndexTile>` (keyed by `word & 0xC3FF`).
- Modify `crates/renderer/src/modern_frame.rs`
  - Add `cgram_rgba: [[u8;4];256]` to `ModernFrame`; add an indexed instance kind (see Task 4).
- Modify `crates/renderer/src/modern_extract.rs`
  - `extract_modern_frame_with_index_atlas(frame, index_atlas) -> ModernFrame` that fills `cgram_rgba` from `frame.cgram` and emits indexed tile instances with their palette number.
- Modify `crates/renderer/src/modern_software.rs`
  - Render indexed instances via CGRAM lookup.
- Modify `crates/renderer/src/modern_gpu.rs` + `crates/renderer/src/modern_bg.wgsl`
  - R8Uint index atlas texture + 256-entry CGRAM uniform; shader does the lookup.
- Modify `zelda3-bin/src/main.rs`
  - Extend the overworld-tile dump to also emit the index atlas (`overworld_index_tiles.bin` + `.json`); add `--modern-render-compare` to use the index path (controller decides wiring).
- Generated index-atlas assets live under `zelda3-bin/developer_tilesets/`.

---

### Task 1: Exact CGRAM→RGBA conversion

**Files:**
- Create: `crates/renderer/src/modern_palette.rs`
- Modify: `crates/renderer/src/lib.rs` (add `pub mod modern_palette;`)

**Interfaces:**
- Produces: `pub fn snes_cgram_to_rgba(entry: u16) -> [u8;4]`; `pub fn cgram_words_to_rgba256(cgram: &[u16]) -> [[u8;4];256]`

- [ ] **Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cgram_conversion_matches_classic_formula() {
        assert_eq!(snes_cgram_to_rgba(0x0000), [0, 0, 0, 0xff]);
        assert_eq!(snes_cgram_to_rgba(0x7fff), [248, 248, 248, 0xff]); // 31<<3 = 248
        assert_eq!(snes_cgram_to_rgba(0x001f), [248, 0, 0, 0xff]);     // R=31
        assert_eq!(snes_cgram_to_rgba(0x7c00), [0, 0, 248, 0xff]);     // B=31
        assert_eq!(snes_cgram_to_rgba(0x03e0), [0, 248, 0, 0xff]);     // G=31
        let pal = cgram_words_to_rgba256(&[0x001f, 0x7c00]);
        assert_eq!(pal[0], [248, 0, 0, 0xff]);
        assert_eq!(pal[1], [0, 0, 248, 0xff]);
        assert_eq!(pal[255], [0, 0, 0, 0xff]); // missing entries default to opaque black
    }
}
```

- [ ] **Step 2: Run and verify it fails**

Run: `cargo test -p renderer modern_palette -- --nocapture`
Expected: compile failure (module/functions missing).

- [ ] **Step 3: Implement**

```rust
/// SNES CGRAM BGR15 -> RGBA8, byte-exact with classic `cgram_to_wgpu_color`
/// (each 5-bit channel left-shifted by 3). Index/entry alpha always 0xff.
pub fn snes_cgram_to_rgba(entry: u16) -> [u8; 4] {
    let r = ((entry & 0x1f) as u8) << 3;
    let g = (((entry >> 5) & 0x1f) as u8) << 3;
    let b = (((entry >> 10) & 0x1f) as u8) << 3;
    [r, g, b, 0xff]
}

pub fn cgram_words_to_rgba256(cgram: &[u16]) -> [[u8; 4]; 256] {
    let mut out = [[0, 0, 0, 0xff]; 256];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = snes_cgram_to_rgba(cgram.get(i).copied().unwrap_or(0));
    }
    out
}
```

Add `pub mod modern_palette;` to lib.rs.

- [ ] **Step 4: Run and verify pass**

Run: `cargo test -p renderer modern_palette -- --nocapture` → PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/renderer/src/modern_palette.rs
git commit -m "renderer: exact SNES CGRAM to RGBA conversion"
```

---

### Task 2: Extend the dump to emit an index atlas

**Files:**
- Modify: `zelda3-bin/src/main.rs` (the `run_dump_unique_overworld_tiles` collector + `draw_snes_4bpp_tilemap_entry_to_rgba`)
- Test: a unit test on the new pure decode helper

**Interfaces:**
- Produces: `fn decode_snes_4bpp_tile_indices(vram: &[u16], chr_base_words: usize, tilemap_entry: u16) -> [u8; 64]` (row-major, post-flip, values 0–15); a written `zelda3-bin/developer_tilesets/overworld_index_tiles.bin` (N×64 bytes) + `overworld_index_tiles.json` (`{ cell_count, cells: [{ id, graphics_key: u16, graphics_keys: [u16] }] }`, where `graphics_key = tilemap_entry & 0xC3FF`).

- [ ] **Step 1: Write the failing test for the index decoder**

Add near `draw_snes_4bpp_tilemap_entry_to_rgba` tests (create one if absent):

```rust
#[test]
fn decode_4bpp_indices_reads_planar_bits_and_flips() {
    // tile 0 at chr base 0: set bitplane 0 row 0 = 0b1010_0000 so x=7->1, x=5->1
    let mut vram = vec![0u16; 0x4000];
    vram[0] = 0b1010_0000; // w01 low byte = bp0
    let idx = decode_snes_4bpp_tile_indices(&vram, 0, 0x0000);
    assert_eq!(idx[7], 1); // bit0 of bp0 at the rightmost pixel
    assert_eq!(idx[5], 1);
    assert_eq!(idx[6], 0);
    // hflip mirrors columns
    let idxf = decode_snes_4bpp_tile_indices(&vram, 0, 0x4000);
    assert_eq!(idxf[0], 1);
    assert_eq!(idxf[2], 1);
}
```

- [ ] **Step 2: Run and verify it fails**

Run: `cargo test -p zelda3-bin decode_4bpp_indices -- --nocapture`
Expected: fails (function missing).

- [ ] **Step 3: Implement the decoder by factoring it out of the existing RGBA path**

Extract the index computation already inside `draw_snes_4bpp_tilemap_entry_to_rgba` (main.rs:9438+) into a pure helper, and call it from both places:

```rust
fn decode_snes_4bpp_tile_indices(
    vram: &[u16],
    chr_base_words: usize,
    tilemap_entry: u16,
) -> [u8; 64] {
    let tile_number = usize::from(tilemap_entry & 0x03ff);
    let hflip = tilemap_entry & 0x4000 != 0;
    let vflip = tilemap_entry & 0x8000 != 0;
    let tile_base = chr_base_words + tile_number * 16;
    let mut out = [0u8; 64];
    for y in 0..8usize {
        let source_y = if vflip { 7 - y } else { y };
        let w01 = vram.get(tile_base + source_y).copied().unwrap_or(0);
        let w23 = vram.get(tile_base + 8 + source_y).copied().unwrap_or(0);
        let (bp0, bp1) = ((w01 & 0xff) as u8, (w01 >> 8) as u8);
        let (bp2, bp3) = ((w23 & 0xff) as u8, (w23 >> 8) as u8);
        for x in 0..8usize {
            let source_x = if hflip { x } else { 7 - x };
            let bit = 1u8 << source_x;
            let index = ((bp0 & bit != 0) as u8)
                | (((bp1 & bit != 0) as u8) << 1)
                | (((bp2 & bit != 0) as u8) << 2)
                | (((bp3 & bit != 0) as u8) << 3);
            out[y * 8 + x] = index;
        }
    }
    out
}
```

Then make `draw_snes_4bpp_tilemap_entry_to_rgba` call this and apply `palette_base + index` for color (keeping the existing RGBA atlas behavior byte-identical — verify the existing RGBA dump is unchanged by re-running it on a couple of screens and diffing the manifest counts).

- [ ] **Step 4: Run and verify decoder test passes**

Run: `cargo test -p zelda3-bin decode_4bpp_indices -- --nocapture` → PASS.

- [ ] **Step 5: Add index-atlas collection + emit to the dump command**

In `run_dump_unique_overworld_tiles`, alongside the existing RGBA collector, add a second collector keyed by `[u8;64]` index pattern. For each visited tilemap word, compute `decode_snes_4bpp_tile_indices`, dedup by the 64-byte pattern, record `graphics_key = word & 0xC3FF` (tile + flips, NOT palette/priority) in the cell's `graphics_keys`. After the walk, write `overworld_index_tiles.bin` (each cell's 64 bytes concatenated) and `overworld_index_tiles.json` (the manifest above). Print the cell count.

- [ ] **Step 6: Regenerate the index atlas**

Run the existing dump command (same invocation used to produce `overworld_unique_tiles.*`; see `--dump-unique-overworld-tiles` handling). Confirm `zelda3-bin/developer_tilesets/overworld_index_tiles.{bin,json}` are written and the cell count is plausible (≤ the RGBA atlas's 6140, since palette variants now collapse).

- [ ] **Step 7: Commit**

```bash
git add zelda3-bin/src/main.rs zelda3-bin/developer_tilesets/overworld_index_tiles.bin zelda3-bin/developer_tilesets/overworld_index_tiles.json
git commit -m "developer: emit palette-index overworld atlas"
```

---

### Task 3: Load the index atlas

**Files:**
- Create: `crates/renderer/src/modern_index_atlas.rs`
- Modify: `crates/renderer/src/lib.rs`

**Interfaces:**
- Produces: `pub struct ModernIndexTile { pub id: u32, pub indices: [u8;64] }`; `pub struct ModernIndexAtlas { pub tile_width_px: u16, pub tile_height_px: u16, pub cells: Vec<ModernIndexTile>, key_to_cell: HashMap<u16,usize> }`; `pub fn load_modern_overworld_index_atlas(repo_root: &Path) -> Result<ModernIndexAtlas,String>`; `pub fn index_cell_for_tilemap_entry<'a>(atlas: &'a ModernIndexAtlas, tilemap_entry: u16) -> Option<&'a ModernIndexTile>` (looks up `tilemap_entry & 0xC3FF`).

- [ ] **Step 1: Write the failing test** (loads the real asset)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn loads_index_atlas_and_resolves_graphics_key() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let atlas = load_modern_overworld_index_atlas(&root).expect("index atlas loads");
        assert_eq!(atlas.tile_width_px, 8);
        assert!(!atlas.cells.is_empty());
        // every cell has 64 indices in range 0..16
        assert!(atlas.cells.iter().all(|c| c.indices.iter().all(|&i| i < 16)));
        // a known graphics key resolves; palette/priority bits are ignored
        let some_word = atlas.cells[0].id; // placeholder; replace with a real graphics_key from the manifest during impl
        let _ = some_word;
    }
}
```
(During implementation, replace the placeholder with an actual `graphics_key` read from the generated manifest so the lookup is asserted on real data.)

- [ ] **Step 2: Run and verify it fails** — `cargo test -p renderer modern_index_atlas -- --nocapture` (function missing).

- [ ] **Step 3: Implement the loader** (read the `.json` manifest with serde + the `.bin` index blob; build `key_to_cell` from each cell's `graphics_keys`; `index_cell_for_tilemap_entry` masks the word with `0xC3FF`). Add `pub mod modern_index_atlas;` to lib.rs.

- [ ] **Step 4: Run and verify pass.**

- [ ] **Step 5: Commit**

```bash
git add crates/renderer/src/modern_index_atlas.rs crates/renderer/src/lib.rs
git commit -m "renderer: load palette-index overworld atlas"
```

---

### Task 4: Indexed tile instance + CGRAM on ModernFrame

**Files:**
- Modify: `crates/renderer/src/modern_frame.rs`

**Interfaces:**
- Produces: `ModernFrame.cgram_rgba: [[u8;4];256]` (defaults to all `[0,0,0,0xff]` in `empty()`); `pub struct ModernIndexTileInstance { pub cell_id: u32, pub screen_x: i16, pub screen_y: i16, pub palette: u8, pub hflip: bool, pub vflip: bool }` and `ModernBgLayer.index_tiles: Vec<ModernIndexTileInstance>`. (hflip/vflip stay false for atlas-sourced tiles — the index pattern already baked flip in Task 2's `graphics_key`; kept for completeness/Task-6 reuse.)

- [ ] **Step 1: Failing test** — assert `ModernFrame::empty().cgram_rgba[0] == [0,0,0,0xff]` and `bg_layers[0].index_tiles.is_empty()`.
- [ ] **Step 2: Run, verify fail.**
- [ ] **Step 3: Add the fields/types; update `empty()` and `ModernBgLayer::new`.**
- [ ] **Step 4: Run, verify pass.**
- [ ] **Step 5: Commit** `git commit -m "renderer: add indexed tile instance + cgram to ModernFrame"`

---

### Task 5: Software renderer applies live CGRAM to index tiles

**Files:**
- Modify: `crates/renderer/src/modern_software.rs`

**Interfaces:**
- Produces: `pub fn render_modern_frame_software_indexed(frame: &ModernFrame, atlas: &ModernIndexAtlas) -> Vec<u8>` (256×224 RGBA). For each enabled_main layer, each `index_tiles` instance: for screen pixel (sx,sy) in 0..8, `index = cell.indices[sy*8+sx]`; if `index == 0` skip (transparent); else `color = frame.cgram_rgba[(palette as usize)*16 + index as usize]`; write at `(screen_x+sx, screen_y+sy)` with clipping; backdrop/forced_blank as in the existing software renderer.

- [ ] **Step 1: Failing test** — synthetic `ModernIndexAtlas` with one cell whose indices are e.g. `1` at (0,0) and `2` at (1,0), else 0; a `ModernFrame` with `cgram_rgba[palette*16+1]=[10,20,30,255]`, `[..+2]=[40,50,60,255]`, one index instance at (0,0) palette P; assert the two pixels get those colors and index-0 pixels are backdrop. Choose P!=0 so the palette offset is exercised.
- [ ] **Step 2: Run, verify fail.**
- [ ] **Step 3: Implement** `render_modern_frame_software_indexed`.
- [ ] **Step 4: Run, verify pass.**
- [ ] **Step 5: Commit** `git commit -m "renderer: software render indexed tiles via live cgram"`

---

### Task 6: Extraction → index instances + CGRAM

**Files:**
- Modify: `crates/renderer/src/modern_extract.rs`

**Interfaces:**
- Consumes: `GpuFrame`, `ModernIndexAtlas`, `cgram_words_to_rgba256`.
- Produces: `pub fn extract_modern_frame_with_index_atlas(frame: &GpuFrame<'_>, atlas: &ModernIndexAtlas) -> ModernFrame` — fills `cgram_rgba = cgram_words_to_rgba256(frame.cgram)`; for layers 0..3 enabled on main, reads the 32×32 tilemap (as in `extract_modern_frame_with_atlas`), looks up `index_cell_for_tilemap_entry(atlas, word)`, pushes a `ModernIndexTileInstance { cell_id, screen_x: col*8 - h_scroll, screen_y: row*8 - v_scroll, palette: (word>>10)&7, hflip:false, vflip:false }`.

- [ ] **Step 1: Failing test** — synthetic index atlas keyed by a word's `graphics_key`; a `GpuFrame` (reuse `test_gpu_frame`) with `vram[0]=word`, `screen_enabled=[1,0]`, `cgram` set; assert one index instance with `palette == (word>>10)&7` and `cgram_rgba` populated from the frame's cgram.
- [ ] **Step 2: Run, verify fail.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run, verify pass.**
- [ ] **Step 5: Commit** `git commit -m "renderer: extract indexed bg tiles + cgram"`

---

### Task 7: GPU index renderer (R8Uint atlas + CGRAM uniform) matches software

**Files:**
- Modify: `crates/renderer/src/modern_gpu.rs`, `crates/renderer/src/modern_bg.wgsl`

**Interfaces:**
- Produces: `ModernGpuRenderer::new_indexed(device, queue, atlas: &ModernIndexAtlas, format)` (uploads the index cells into an `R8Uint` 2D texture, one cell per 8×8 region in a grid) and `render_indexed(device, queue, frame: &ModernFrame, output_view)` (uploads the 256-entry CGRAM as a uniform/storage buffer + the per-tile instances {cell grid xy, screen xy, palette}; shader fetches `index = textureLoad(atlas_u32)`, `if index==0 { discard }`, else `color = cgram[palette*16 + index]`).

- [ ] **Step 1: Failing test** — render the SAME synthetic `ModernFrame`+`ModernIndexAtlas` as Task 5 through `ModernGpuRenderer` indexed path, read back, `assert_eq!(gpu_rgba, render_modern_frame_software_indexed(&frame, &atlas))` over the full 256×224×4 buffer (NOT weakened). Use `R8Uint` + `textureLoad` (integer, no filtering) and `Rgba8Unorm` target.
- [ ] **Step 2: Run, verify fail** (placeholder/black).
- [ ] **Step 3: Implement the index texture upload, CGRAM buffer, instance buffer, and shader lookup.** Mirror the existing `modern_gpu.rs` instance/packing pattern; add the CGRAM as a `uniform`/`storage` array of 256 `vec4<f32>` (each = rgba/255) OR a 256×1 `Rgba8Unorm` texture sampled with `textureLoad` for exact bytes (prefer the texture for byte-exactness).
- [ ] **Step 4: Run, verify pass** (gpu == software exactly).
- [ ] **Step 5: Commit** `git commit -m "renderer: gpu indexed tile rendering via cgram"`

---

### Task 8: Wire the index path into the compare harness + measure

**Files:**
- Modify: `zelda3-bin/src/main.rs` (the modern compare block in `run_play_gpu_render_compare`, and/or `run_replay_save`)

**Interfaces:**
- Consumes: `extract_modern_frame_with_index_atlas`, `render_modern_frame_software_indexed`, `load_modern_overworld_index_atlas`.
- Produces: a `--modern-index-compare <stride>` (or reuse `--modern-render-compare` switched by `ZELDA3_RENDERER`) that logs `modern_index_compare frame=<n> old=0x<h> modern=0x<h> match=<bool>` using the index path.

- [ ] **Step 1:** Add the flag/env wiring (mirror the existing modern compare block; controller decides committed-vs-uncommitted per the WIP rule).
- [ ] **Step 2:** Build `cargo build --profile parity -p zelda3-bin`.
- [ ] **Step 3:** Measure on an OVERWORLD frame. NOTE: needs an overworld checkpoint loadable by the harness — the `--play-gpu-render-compare` resume currently renders blank after a `state_recorder` resume (a separate known issue); the reliable path is to add the index compare to `run_replay_save` (which renders correctly along the route) and run it over an overworld segment. Record compared/matched/mismatched and, for a representative overworld frame, dump both the classic and modern-index frames to PNG and report the mismatched-pixel count and whether the remaining diff is sprites-only (expected) vs BG-color (should now be fixed).
- [ ] **Step 4: Commit** the script/wiring as appropriate.

---

## Final Verification

```bash
cargo fmt --check -p renderer
cargo test -p renderer modern_ -- --nocapture
cargo test -p zelda3-bin decode_4bpp_indices -- --nocapture
cargo build --profile parity -p zelda3-bin
```

Expected: modern unit tests pass (incl. the new index software/GPU parity tests, byte-exact); the index compare reports that BG tile COLORS now match the classic renderer on overworld frames (remaining mismatches attributable to sprites/color-math/window, which are out of scope here).

## Self-Review

- **Spec coverage:** conversion (T1), index extraction+atlas emit (T2), atlas load (T3), frame/instance types (T4), software CGRAM render (T5), extraction (T6), GPU CGRAM render (T7), measurement (T8).
- **Placeholder scan:** the only deferred specifics are the manifest `graphics_key` value used in T3's test (read from the real generated manifest at impl time) and the T8 measurement frame — both are explicitly called out, not hidden.
- **Type consistency:** `ModernIndexAtlas`/`ModernIndexTile`/`index_cell_for_tilemap_entry` (T3) are consumed unchanged in T5–T7; `ModernIndexTileInstance`/`cgram_rgba` (T4) are produced in T6 and consumed in T5/T7; `snes_cgram_to_rgba`/`cgram_words_to_rgba256` (T1) consumed in T6.
- **Scope control:** BG color only; sprites/color-math/window/priority explicitly deferred to separate plans. The classic renderer stays the default oracle throughout.
