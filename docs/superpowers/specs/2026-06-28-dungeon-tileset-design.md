# Dungeon Tileset (palette-indexed, blockset-keyed) — Design

**Status:** Approved (brainstorming). Next: implementation plan via superpowers:writing-plans.

## Goal

Extend the opt-in modern renderer's palette-indexed BG path to **dungeons**, so dungeon BG tiles render with correct live-CGRAM colors in the modern path, measured against the classic renderer. Classic stays the default/oracle. Sprites, color-math, window masking, brightness, and BG-vs-sprite priority remain out of scope (separate future work), same as the overworld palette plan.

## Background / why this is non-trivial

The overworld palette-index atlas (`docs/superpowers/plans/2026-06-26-palette-indexed-modern-bg.md`) works because overworld BG CHR is a single global graphics set. **Dungeon BG CHR is per-room**: each room loads a graphics blockset (`initialize_tilesets` keys off `world.palette_theme.main_tile_theme_index()`), so the same `tile_number` indexes **different** graphics in different blocksets. A dungeon atlas keyed by `tile_number` alone collides (observed: applying the overworld atlas to dungeon frame 19000 produced garbage). Cells must therefore be keyed by **`(blockset, graphics_key)`**.

## Decisions (from brainstorming)

- **Scope:** all unique blocksets ("dedup by gfx set"), not every individual room redundantly. Achieved naturally by walking rooms and deduping cells by index pattern keyed by `(blockset, graphics_key)`.
- **Goal:** opt-in modern parity (dungeon BG colors match classic); classic remains default.
- **Approach:** **separate dungeon index atlas, blockset-keyed** (chosen over a unified overworld+dungeon atlas, and over full-CHR blockset enumeration). Keeps the proven overworld atlas/loader untouched; reuses the working dev-room load+draw to capture real used tiles.

## Key identifiers

- **Blockset key** = `game_state.world.palette_theme.main_tile_theme_index()` (u8/u16; the BG-CHR determinant used by `initialize_tilesets`). Overworld uses 0x20 (light) / 0x21 (dark); dungeons use their room-header themes.
- **graphics_key** = `tilemap_word & 0xC3FF` (tile_number bits 0–9 + hflip 0x4000 + vflip 0x8000; palette bits 10–12 and priority 0x2000 excluded — palette is applied at render time from live CGRAM, flip is baked into the cell pattern by the decoder).
- **Packed cell key** = `((theme as u32) << 16) | (graphics_key as u32)`.
- **Mode** = `game_state.frame.main_module`: 9 = overworld; 7 (and 16) = dungeon play.

## Architecture

Reuse the overworld palette-index data model and renderers verbatim:
- `ModernIndexTile { id, indices:[u8;64] }` (existing).
- `ModernFrame.cgram_rgba:[[u8;4];256]` + `ModernBgLayer.index_tiles: Vec<ModernIndexTileInstance>` (existing).
- `render_modern_frame_software_indexed(frame, atlas)` and `ModernGpuIndexRenderer` — **unchanged**; they render `index_tiles + cgram_rgba` and are atlas-agnostic. (Both take a `&ModernIndexAtlas` only to resolve `cell_id → indices`; the dungeon atlas reuses `ModernIndexTile`, so the renderers need a cell-lookup that works for both. See Components.)

New pieces, mirroring the overworld ones:

### Components

1. **Capture — `--dump-dungeon-index-tiles` (zelda3-bin/src/main.rs)**
   - Walk dungeon rooms `0..0x128`. For each room, load+draw it into VRAM by reusing the developer-room path the WIP already uses (`developer_prepare_synthetic_room` + the room load/draw, e.g. `Dungeon_LoadAndDrawEntranceRoom`). Determine empirically (during implementation) which call sequence yields a populated BG tilemap + loaded blockset CHR in `game.ppu.vram`; this is the main implementation unknown.
   - Read `theme = main_tile_theme_index()` for the room.
   - Read the room's BG tilemap entries from VRAM via a **dungeon analog of `parity_probe_overworld_bg2_map8_entry`** (new helper in the zelda3 crate; dungeon room maps are 64×64 tiles8 like overworld). Start with the primary room BG layer (BG1); BG2 (background plane) added only if the end-to-end measurement shows gaps.
   - For each used word, decode the index pattern with `decode_snes_4bpp_tile_indices(vram, DUNGEON_BG_CHR_BASE, word)` (the dungeon BG CHR base is determined empirically during the dump; the existing overworld constant is 0x2000).
   - Dedup cells by the 64-byte index pattern; for each cell record the set of `(theme, graphics_key)` packed keys that produced it.
   - Write canonical assets (hardcoded path, like the overworld dump):
     - `zelda3-bin/developer_tilesets/dungeon_index_tiles.bin` — `cell_count * 64` bytes, cell 0 first, values 0–15.
     - `zelda3-bin/developer_tilesets/dungeon_index_tiles.json` — `{ "format":"zelda3_dungeon_index_tiles_v1", "tile_width_px":8, "tile_height_px":8, "cell_count":N, "cells":[ {"id":u32, "keys":[u32]} ] }` (`keys` = packed `theme<<16|graphics_key`).
   - Run RGBA/other existing dump outputs to throwaway /tmp paths so the user's existing atlases are untouched.

2. **Load — `crates/renderer/src/modern_dungeon_atlas.rs`**
   - `pub struct ModernDungeonIndexAtlas { tile_width_px, tile_height_px, cells: Vec<ModernIndexTile>, key_to_cell: HashMap<u32, usize> }`.
   - `pub fn load_modern_dungeon_index_atlas(repo_root: &Path) -> Result<ModernDungeonIndexAtlas, String>`.
   - `pub fn dungeon_index_cell<'a>(atlas: &'a ModernDungeonIndexAtlas, theme: u16, tilemap_entry: u16) -> Option<&'a ModernIndexTile>` — lookup by `((theme as u32)<<16)|((tilemap_entry & 0xC3FF) as u32)`.

3. **Extract — `crates/renderer/src/modern_extract.rs`**
   - `pub fn extract_modern_frame_with_dungeon_atlas(frame: &GpuFrame<'_>, atlas: &ModernDungeonIndexAtlas, theme: u16) -> ModernFrame` — mirrors `extract_modern_frame_with_index_atlas` but resolves cells via `dungeon_index_cell(atlas, theme, word)`. Fills `cgram_rgba = cgram_words_to_rgba256(frame.cgram)`; emits `ModernIndexTileInstance { cell_id, screen_x, screen_y, palette:(word>>10)&7, hflip:false, vflip:false }` for the dungeon BG layer(s).

4. **Render — reuse**
   - The renderers look up `cell_id → indices`. To serve both atlases, factor the renderers' cell lookup over a small trait or pass a `&[ModernIndexTile]` cells slice (whichever is least invasive — likely add a method that accepts `&[ModernIndexTile]`). No new rendering logic; preserve the existing byte-exact software↔GPU contract.

5. **Mode-gate — `run_replay_save` compare (zelda3-bin/src/main.rs, uncommitted WIP)**
   - Load both the overworld and dungeon index atlases once when the index compare flag is set.
   - Per measured frame, read `main_module` and `theme` from the game: module 9 → overworld index path; module 7/16 → `extract_modern_frame_with_dungeon_atlas(.., theme)`. Render via the indexed renderer; compare to classic; log `modern_index_compare frame=N mode=<ow|dungeon> mismatch_px=K modern_tiles=T`.

## Data flow

dump (per room): load+draw room → VRAM tilemap + blockset CHR + theme → decode used words → cells keyed by (theme, graphics_key) → `dungeon_index_tiles.{bin,json}`.

runtime (per dungeon frame): GpuFrame (VRAM tilemap words + CGRAM) + theme + mode → `extract_modern_frame_with_dungeon_atlas` → `ModernFrame{index_tiles, cgram_rgba}` → software/GPU indexed render → compare vs classic.

## Testing (TDD)

- Unit: `dungeon_index_cell` resolves a `(theme, word)` to the right cell and ignores palette/priority bits (synthetic atlas built via a test constructor); a different theme with the same `tile_number` resolves to a different cell.
- Unit: `extract_modern_frame_with_dungeon_atlas` emits an instance with the right `cell_id`/`palette` for a given theme, and fills `cgram_rgba`.
- Reuse the existing byte-exact `render_modern_frame_software_indexed` / `ModernGpuIndexRenderer` tests (rendering is atlas-agnostic; if the renderer signature changes to accept a cells slice, update those tests to the new signature, keeping the full-buffer `assert_eq!`).
- Dump validation: `cell_count > 0`, every `.bin` byte `< 16`, `bin len == cell_count*64`, and a sample `(theme, word)` lookup resolves.
- End-to-end measurement: run `run_replay_save --modern-index-compare` over a dungeon segment; dump one dungeon frame's classic vs modern PNGs; report `mismatch_px` and whether the remaining diff is sprites-only (expected) vs BG-color/coverage.

## Commit / WIP conventions

- New files (`modern_dungeon_atlas.rs`, the two `dungeon_index_tiles.*` assets) are committed.
- Edits to pre-existing shared files with the user's WIP (main.rs dump command + compare wiring, the zelda3-crate dungeon BG reader if it lands in a WIP file, lib.rs `pub mod`) are left **uncommitted** unless they are in WIP-free regions — confirm per-file at execution time. The user has approved editing the atlas-dump/dungeon WIP for this work.
- `--no-verify` commits (heavy parity pre-commit hook); the user works the repo concurrently.

## Risks / unknowns to resolve in the plan

1. **Dungeon room load+draw for the dump:** the exact call sequence that populates a clean per-room BG tilemap + blockset CHR in `game.ppu.vram` (reusing the WIP dev-room path). Highest-risk item; validate early on one room before walking all.
2. **Dungeon BG CHR base** and **which BG layer(s)** hold room tiles (BG1 first; BG2 if needed).
3. **Renderer cell-lookup refactor** to serve both atlases without duplicating render logic or breaking the byte-exact tests.
4. **Theme readout at runtime** vs at dump time must use the same accessor so keys match.
