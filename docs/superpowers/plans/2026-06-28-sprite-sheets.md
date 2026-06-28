# Sprite Sheets (palette-indexed OBJ rendering) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** Render OAM sprites (Link, NPCs, enemies, items) in the opt-in modern path by extracting palette-index **sprite sheets** keyed by sprite-graphics context, decoding OAM into sprite-tile instances, and drawing them with live CGRAM (palettes 8–15), composited over the BG.

**Architecture:** Mirror the palette-index BG pipeline for OBJ. Sprite CHR is per-area (4 `graphics_subset`s loaded to VRAM 0x4000+), so sprite cells are keyed by `(context, bank, tile, flip)` where `context` = a hash of the 4 graphics_subsets. A new `--dump-sprite-index-tiles` walks areas, decodes each visible OAM sprite's 8×8 tiles into index patterns, dedups, and writes `sprite_index_tiles.{bin,json}`. The extract decodes OAM (mirroring `sprite_renderer::resolve_obj_pixels` enumeration) into `ModernIndexSpriteInstance`s; the indexed renderers draw them after BG using CGRAM `0x80+pal*16`, index 0 transparent. Classic stays default/oracle.

**Tech Stack:** Rust, wgpu, existing `crates/renderer/src/modern_*`, `sprite_renderer.rs` (OAM-decode reference), the dump in `zelda3-bin/src/main.rs`.

## Global Constraints

- Opt-in; classic stays default/oracle. 256×224 boundary. Renderers stay byte-exact software↔GPU on tested cases (`assert_eq!`, never weakened).
- Reuse `decode_snes_4bpp_tile_indices`, `ModernIndexTile`, `cgram_words_to_rgba256`, the cells-slice indexed renderers, `ModernFrame.cgram_rgba`.
- Sprite palette color = `cgram_rgba[0x80 + pal*16 + index]`, `pal = (oam1 & 0x0e00)>>9`, index 0 transparent. (CGRAM 128–255 = OBJ palettes.)
- OAM decode MUST match `sprite_renderer::resolve_obj_pixels` (crates/renderer/src/sprite_renderer.rs:349): y=`(oam0>>8)+1 &0xff` (skip 0xf0), x=`(oam0&0xff)+(hi&1)*256`, size from `sizes[(hi>>1)&1]` (SPRITE_SIZES[obj_size]), hflip `oam1&0x4000`, vflip `oam1&0x8000`, priority `(oam1&0x3000)>>12`, bank `oam1&0x100 ? tile_adr2 : tile_adr1`, tile addressing `used_tile=((tile_row_base+(row>>3))<<4)|((tile_col_base+(col>>3))&0xf)`, `addr=obj_addr+used_tile*16+(row&7)`.
- Context = hash/tuple of `game_state.sprites.workspace.graphics_subset(0..4)` (the per-area sprite CHR identity). Same accessor at dump + runtime.
- New files committed; main.rs/lib.rs/zelda3 WIP edits uncommitted. `--no-verify` commits.

## File Structure
- Create `crates/renderer/src/modern_sprite_atlas.rs` — sprite index atlas + loader + `(context, bank, tile, flip)` lookup.
- Modify `crates/renderer/src/modern_frame.rs` — add `ModernIndexSpriteInstance { cell_id, screen_x, screen_y, palette, priority, hflip, vflip }` + `ModernFrame.index_sprites: Vec<...>`.
- Modify `crates/renderer/src/modern_software.rs` + `modern_gpu.rs` — draw `index_sprites` after BG (CGRAM 0x80+pal*16, index0 transparent, OAM-order painter within priority).
- Modify `crates/renderer/src/modern_extract.rs` — `extract_modern_sprites(frame, atlas, context)` populating `index_sprites`.
- Modify `crates/zelda3/src/*` (WIP) — sprite probe accessors (graphics_subset readout + OAM/CHR for the dump) if needed.
- Modify `zelda3-bin/src/main.rs` (WIP) — `--dump-sprite-index-tiles` + compare wiring.
- Asset: `zelda3-bin/developer_tilesets/sprite_index_tiles.{bin,json}` (committed).

---

### Task 1: Spike — pin sprite CHR addressing + context + one-sprite probe
**Files:** Modify `zelda3-bin/src/main.rs` (+ zelda3 accessors if needed). NO COMMIT (WIP).
**Deliverable:** `fn sprite_index_probe(game: &mut ZeldaState) -> (u64 context, Vec<(u16 bank_base, u16 tile, bool hflip, bool vflip, [u8;64])>)` covering the visible OAM sprites of a loaded area, + a test, + documented constants (context hash, obj bank bases, that `decode_snes_4bpp_tile_indices` decodes sprite CHR at the bank base correctly).

- [ ] Step 1: Load an area with sprites (e.g. reuse the dungeon probe `Dungeon_LoadAndDrawEntranceRoom`+`InitializeTilesets` for a room with sprites, or an overworld screen; sprites are in `game.ppu.oam`/`game.game_state` after a frame). Read `obj.tile_adr1/tile_adr2`, `obj.obj_size`, the OAM (`game.ppu.oam`), and `graphics_subset(0..4)`.
- [ ] Step 2: For each non-offscreen OAM sprite, enumerate its 8×8 tiles (size/2 per side) and decode each via `decode_snes_4bpp_tile_indices(&game.ppu.vram, <obj bank base in words>, used_tile)` — confirm non-degenerate patterns (a recognizable sprite). Pin the context hash = fold of the 4 graphics_subsets.
- [ ] Step 3: Implement `sprite_index_probe`; test asserts context!=0, some sprites, all idx<16, some idx!=0. `cargo test -p zelda3-bin sprite_index_probe -- --nocapture` PASS. If sprites can't be enumerated, report BLOCKED with evidence.

### Task 2: `--dump-sprite-index-tiles`
**Files:** Modify main.rs (WIP); Create `sprite_index_tiles.{bin,json}` (commit assets).
- Walk areas (dungeon entrances 0..0x128 and overworld screens 0..0x80) via the existing load paths; for each, `sprite_index_probe`; dedup cells by 64-byte pattern; key packed `(context_hash_low<<.. )|(bank_bit<<.. )|(tile<<.. )|flip` — define a packing that fits u64 keys: `{ "cells":[{id, keys:[u64]}] , cell_count, ... }`. Write to canonical `developer_tilesets/sprite_index_tiles.{bin,json}`. Validate (idx<16, len==count*64). Commit assets only.

### Task 3: Load sprite atlas
**Files:** Create `modern_sprite_atlas.rs`; lib.rs (uncommitted). `ModernSpriteIndexAtlas { cells: Vec<ModernIndexTile>, key_to_cell: HashMap<u64,usize> }`, `load_modern_sprite_index_atlas`, `sprite_index_cell(atlas, context, bank, tile, hflip, vflip)`. TDD with a real key from the manifest. Commit the new file.

### Task 4: Add `ModernIndexSpriteInstance` + OAM extract
**Files:** modern_frame.rs (`ModernIndexSpriteInstance{cell_id,screen_x,screen_y,palette,priority,hflip,vflip}` + `index_sprites: Vec`), modern_extract.rs (`extract_modern_sprites(frame, atlas, context)`). Mirror `resolve_obj_pixels` ENUMERATION (per-sprite, per-8×8-tile; NOT the per-pixel resolver): for each OAM sprite, for each tile, look up `sprite_index_cell`, push instance at the tile's screen x/y with palette `(oam1&0xe00)>>9`, priority, flip. TDD on a synthetic atlas + crafted OAM. Commit modern_frame.rs + modern_extract.rs.

### Task 5: Render sprites (software + GPU)
**Files:** modern_software.rs, modern_gpu.rs. After drawing BG `index_tiles`, draw `index_sprites`: color = `frame.cgram_rgba[0x80 + (palette as usize)*16 + index]`, index 0 transparent, in OAM order (later OAM = lower priority drawn... mirror resolve_obj_pixels "first nontransparent in OAM order wins" → draw in REVERSE OAM order so earlier OAM wins, OR track and skip already-written; match the classic). Keep byte-exact software↔GPU test (synthetic sprite). Commit both.

### Task 6: Wire + measure
**Files:** main.rs (WIP) compare block. Compute `context` from the live `graphics_subset`s; call `extract_modern_sprites`; render; log + PNG dump. Re-measure a frame with Link/NPCs; confirm sprites appear in `/tmp/modern_index_<n>.png` (Link visible). NO COMMIT (WIP).

## Final Verification
`cargo fmt --check -p renderer`; `cargo test -p renderer modern_ -- --nocapture`; `cargo test -p zelda3-bin sprite_index_probe -- --nocapture`; `cargo build --profile parity -p zelda3-bin`; sprite PNG shows Link/NPCs.

## Self-Review
- Coverage: spike (T1), dump (T2), load (T3), instance+extract (T4), render (T5), wire+measure (T6). OAM-decode constants + palette + context carried in Global Constraints.
- The riskiest unknowns (sprite CHR addressing, context key, OAM enumeration) are pinned by the T1 spike before the dump.
- Scope: OBJ tiles + colors + z-order; color-math/mosaic still out of scope. Classic stays oracle.
