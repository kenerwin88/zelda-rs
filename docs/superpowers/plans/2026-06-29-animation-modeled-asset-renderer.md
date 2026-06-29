# Animation-Modeled Asset Renderer (off-VRAM, correctly animated) — Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Large multi-session project; execute milestone by milestone. Steps use `- [ ]`.

**Goal:** Render the game entirely from PNG **assets** + the game's **logical state** — correctly animated — WITHOUT reading VRAM CHR pixel content. This is the true "modernization": asset-based rendering that animates, replacing the live-VRAM parity renderer's dependence on SNES CHR memory.

**The key idea (why this is possible):** The game already *decides* which graphic to draw and *where it comes from in ROM* — that information is pure game-logic metadata, separate from the VRAM pixel bytes. If we capture, per VRAM CHR tile slot, the **logical CHR source** that filled it (which decompressed graphics pack + tile offset, or which Link pose), we can: (a) build a PNG asset library keyed by that logical source, and (b) at render time map each OAM/tilemap tile → its VRAM slot → logical source → asset cell, and blit from the PNG. No VRAM pixel reads — and because the logical source changes with the game's animation, the rendered result animates correctly.

**Architecture (3 layers):**
1. **CHR-source bookkeeping:** instrument the VRAM CHR write paths (`do3_to_4_high_to_vram`/`do3_to_4_low_to_vram` in load_gfx.rs for area/BG/sprite graphics; the per-frame Link pose DMA) to record a `vram_chr_source: [LogicalSrc; N_TILE_SLOTS]` table, where `LogicalSrc = { pack: u16, tile_off: u16 }` (BG/sprite) or a Link-pose id. Updated only when graphics are (re)written. Exposed on the game/GpuFrame.
2. **Asset library keyed by logical source:** a route-walk dump that, per drawn tile, records its `LogicalSrc` + palette and bakes the colored 8×8, deduped by `(LogicalSrc)` (NOT by appearance) → `assets_by_source.{png,json}` mapping `LogicalSrc → cell`.
3. **Asset render path (`ZELDA3_RENDERER=assets-anim`):** for each BG tilemap entry / OAM tile, read its VRAM slot's `LogicalSrc` (from the bookkeeping table, not the pixels), look up the asset cell, apply live palette (CGRAM colors are game state too — fine to use, or bake), blit from the PNG. Composite with the existing color-math pipeline. Animations follow because `LogicalSrc` follows the game's animation.

**Validation target:** Link walks/attacks/swims and water animates, rendered from PNG assets with the bookkeeping driving selection, matching the classic — proving off-VRAM-pixel animated rendering.

## Global Constraints
- Off-VRAM-PIXEL: the render path must not read `frame.vram` CHR bytes for graphics; it may use the bookkeeping table, OAM, tilemaps, CGRAM (all game logical state).
- Keep the existing live-VRAM `modern` mode + classic default intact. This is an additional `assets-anim` mode.
- Reuse: `render_modern_frame_full` (color-math), `decode_snes_4bpp/2bpp_tile_indices` (for the DUMP only), the PNG-write + dedup helpers.
- New files committed; WIP-file edits (load_gfx.rs, gpu_frame.rs, main.rs, lib.rs) left uncommitted per the WIP rule unless WIP-free. `--no-verify` commits.

---

### Milestone 1: CHR-source bookkeeping
**Files:** crates/zelda3/src/load_gfx.rs (instrument writes), a new source-table struct (e.g. crates/zelda3/src/chr_source.rs or on the ppu/game state), gpu_frame.rs (expose).
**Produces:** `pub struct LogicalChrSrc { pub pack: u16, pub tile_off: u16, pub kind: u8 }` and a `[LogicalChrSrc; 0x800]` table (one per 8×8 CHR tile slot, VRAM word_addr/16) accessible from the game; populated at every CHR→VRAM write.

- [ ] **Step 1 (test first):** unit/integration test: after loading a dungeon room (reuse `dungeon_room_index_probe` infra) and stepping a frame, assert the CHR-source table has nonzero entries for the BG CHR region (0x2000+) with plausible `pack` values, and that Link's sprite-CHR slots (0x4000+) get a Link-kind source after a play frame. RED before instrumentation.
- [ ] **Step 2:** Add the `LogicalChrSrc` table to game/ppu state. In `do3_to_4_high_to_vram`/`do3_to_4_low_to_vram(dst, data)`, also write, for each 8×8 tile slot covered by `[dst..dst+data.len()]`, the source `{pack: <the gfx_pack passed to load_*_graphics>, tile_off: <index within data>, kind: bg/sprite}`. Thread `gfx_pack` into the do3_to_4 calls (they currently take only dst+data — pass the pack through `load_sprite_graphics`/`load_background_graphics`). For Link's per-frame pose DMA, tag those slots with `kind=link, pack=link_dma_graphics_index, tile_off=pose tile`.
- [ ] **Step 3:** Run, verify the table populates (GREEN). Commit the new chr_source.rs (and leave load_gfx.rs/gpu_frame.rs edits uncommitted if WIP).

### Milestone 2: assets-by-source dump
**Files:** zelda3-bin/src/main.rs (`--dump-assets-by-source`), assets `assets_by_source.{png,json}`.
- [ ] Route-walk; per drawn BG/sprite tile, get its `LogicalChrSrc` (from M1 table) + palette; bake the colored 8×8 (decode from VRAM is fine HERE — this is the offline dump); dedup by `LogicalChrSrc` (keep one cell per logical source — palette applied at runtime, so dedup the INDEX pattern per source, store indices not RGBA, like the index atlases but keyed by source). Output: `.bin` (index cells) + `.json` (`source → cell` map). Commit assets.

### Milestone 3: assets-anim render path
**Files:** crates/renderer/src/modern_extract.rs (new `extract_modern_frame_from_sources(frame, source_table, asset)`), main.rs wiring.
- [ ] For each BG tilemap entry: VRAM slot from `tile_adr + tile#`; look up `source_table[slot]` → asset cell; emit index tile (palette from tilemap word). For each OAM tile: slot from obj bank + tile; `source_table[slot]` → cell; emit sprite instance (palette from OAM). NO `decode_*` from VRAM in this path. Render via `render_modern_frame_full` + sprites. Wire `ZELDA3_RENDERER=assets-anim`.
- [ ] Unit test on synthetic source table + asset.

### Milestone 4: validate animation
- [ ] Measure assets-anim vs classic across the route (reuse the compare harness, add an `assets-anim` branch). Dump PNGs at frames with Link moving + animated water; confirm animation is correct and mismatch is low. Report coverage gaps (slots with no recorded source → fall back / flag).

### Milestone 5: coverage + polish
- [ ] Handle remaining cases (ancilla/effect sprites, HUD BG3, mode-7), measure route-wide, drive mismatch down. Document residuals.

## Self-Review
- The crux (M1) is identified and hookable (do3_to_4_*_to_vram + Link DMA carry the ROM `pack`). M2 keys assets by source; M3 renders from source table (off VRAM pixels) so animation follows; M4 validates. Color-math reuses `render_modern_frame_full`. Scope: BG + sprites animated off-VRAM; classic/modern modes untouched.
