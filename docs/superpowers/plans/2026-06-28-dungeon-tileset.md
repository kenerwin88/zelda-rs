# Dungeon Tileset (palette-indexed, blockset-keyed) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the opt-in modern palette-index BG renderer to dungeons, so dungeon BG tiles render with correct live-CGRAM colors in the modern path (measured against classic), with tiles keyed by `(blockset theme, graphics_key)` to handle per-room CHR.

**Architecture:** Mirror the overworld palette-index pipeline (spec: docs/superpowers/specs/2026-06-28-dungeon-tileset-design.md). A new `--dump-dungeon-index-tiles` command walks dungeon rooms, deduping unique 8×8 index patterns keyed by `(theme, graphics_key)`, into `dungeon_index_tiles.{bin,json}`. A new loader + a dungeon-aware extractor feed the EXISTING byte-exact indexed renderers (software + GPU), which are made atlas-agnostic via a small cell-lookup refactor. The classic renderer stays the default/oracle.

**Tech Stack:** Rust, wgpu, the existing `crates/renderer/src/modern_*` modules, the dungeon RTL in `crates/zelda3/src`, the atlas-dump command in `zelda3-bin/src/main.rs`.

## Global Constraints

- Opt-in only; classic renderer stays the DEFAULT and the parity oracle. Dungeon support adds an opt-in path; it does not replace classic.
- Parity boundary is the fixed 256×224 frame; the modern GPU renderer must stay byte-exact with the software reference on tested cases (`assert_eq!` over the full buffer, never weakened).
- Blockset key = `game_state.world.palette_theme.main_tile_theme_index()` (the BG-CHR determinant). Use the SAME accessor at dump time and at runtime.
- `graphics_key = tilemap_word & 0xC3FF` (tile_number bits 0–9 + hflip 0x4000 + vflip 0x8000; palette bits 10–12 and priority 0x2000 excluded). Packed cell key = `((theme as u32) << 16) | (graphics_key as u32)`.
- Mode = `game_state.frame.main_module`: 9 = overworld, 7 (and 16) = dungeon.
- CGRAM→RGBA uses the existing `crate::modern_palette::snes_cgram_to_rgba` (exact BGR15 each channel `<<3`). Index 0 = transparent.
- Reuse `ModernIndexTile`, `ModernFrame.cgram_rgba`, `ModernIndexTileInstance`, `render_modern_frame_software_indexed`, `ModernGpuIndexRenderer` — do not duplicate render logic.
- New files (`modern_dungeon_atlas.rs`, the two `dungeon_index_tiles.*` assets) are committed. Edits to pre-existing shared files carrying the user's WIP (main.rs, lib.rs, and any WIP-bearing zelda3 file) are left UNCOMMITTED unless the touched region is WIP-free — confirm per file at execution time. Commit with `--no-verify` (heavy parity pre-commit hook; user works the repo concurrently).

---

## File Structure

- Create `crates/renderer/src/modern_dungeon_atlas.rs` — dungeon index atlas type + loader + `(theme, word)` lookup.
- Modify `crates/renderer/src/lib.rs` — `pub mod modern_dungeon_atlas;` (uncommitted; WIP file).
- Modify `crates/renderer/src/modern_extract.rs` — `extract_modern_frame_with_dungeon_atlas`.
- Modify `crates/renderer/src/modern_software.rs` and `crates/renderer/src/modern_gpu.rs` — make the indexed renderers resolve cells from a `&[ModernIndexTile]` slice so both atlases share render logic.
- Modify `crates/zelda3/src/dungeon.rs` (or a sibling) — a `parity_probe`-style dungeon room reader returning room tilemap words + theme (mirrors `parity_probe_overworld_bg2_map8_entry`). WIP file — leave uncommitted if region overlaps WIP.
- Modify `zelda3-bin/src/main.rs` — the `--dump-dungeon-index-tiles` command + the `run_replay_save` mode-gated compare wiring. WIP file — leave uncommitted.
- Generated assets under `zelda3-bin/developer_tilesets/dungeon_index_tiles.{bin,json}` (committed).

---

### Task 1: Spike — pin the dungeon room load+draw, BG CHR base, and tilemap accessor (one room)

This de-risks the only true unknowns before building the dump. Deliverable: a tested helper that, for ONE known room, returns its theme + used `(word, [u8;64] index pattern)` pairs, and documents the load sequence + `DUNGEON_BG_CHR_BASE` in code comments.

**Files:**
- Modify: `zelda3-bin/src/main.rs` (add the probe helper + a test)
- Investigate (read-only): `crates/zelda3/src/dungeon.rs` (`parity_probe_dungeon_room`, `Dungeon_LoadAndDrawEntranceRoom`, `room_tilemaps` accessors), `crates/zelda3/src/load_gfx.rs` (`initialize_tilesets`), `zelda3-bin/src/main.rs` (`load_developer_synthetic_room`, `write_developer_room_visuals_to_ppu`, `OVERWORLD_BG_CHR_BASE = 0x2000`).

**Interfaces:**
- Produces: `fn dungeon_room_index_probe(game: &mut ZeldaState, room: u16) -> (u16 /*theme*/, Vec<(u16 /*word*/, [u8;64])>)` and a documented `const DUNGEON_BG_CHR_BASE: usize`.

- [ ] **Step 1: Investigate + reproduce the load+draw**

In a scratch test, construct a game (`load_translated_replay_state` as the overworld dump does, or `load_play_state`), then drive a real dungeon room load so that `game_state.dungeon.room_tilemaps` is populated AND the blockset CHR is in `game.ppu.vram`. Candidate sequence (verify which works): set the room via `parity_probe_dungeon_room(room)` / `developer_prepare_synthetic_room(room)`, ensure `initialize_tilesets()` ran for the room's theme, and the room tilemap was drawn (the RTL room-draw path, e.g. via `Dungeon_LoadAndDrawEntranceRoom(room as u8)` or the module-7 load submodule). Use room `0x0002` (a Hyrule Castle room) as the probe room.

Print: `theme = game.game_state.world.palette_theme.main_tile_theme_index()`, a count of nonzero `room_tilemaps` BG1 entries, and for the first nonzero entry its word + the bytes from `game.ppu.vram` at the suspected CHR base. Confirm the decoded 8×8 pattern is non-degenerate (not all-zero, not all-same).

- [ ] **Step 2: Pin the constants**

Once the tilemap reads correctly: record `DUNGEON_BG_CHR_BASE` (try `0x2000` first — the overworld value — and verify the decoded tile matches what the classic renderer would draw for that word by spot-comparing `render_snes_4bpp_tile_to_rgba` output to a hand-inspected tile). Record which `room_tilemaps` accessor yields BG1 words (`bg1_tile(word_index)` / `room_read_bg1` / `bg1_tile_by_byte_pos`) and the map dimensions (expect 64×64 tiles8 → word_index 0..0x1000, matching the overworld 64×64 walk).

- [ ] **Step 3: Implement `dungeon_room_index_probe`**

Write the helper that runs the pinned load+draw sequence for a given room and returns `(theme, Vec<(word, decode_snes_4bpp_tile_indices(&game.ppu.vram, DUNGEON_BG_CHR_BASE, word))>)` for every nonzero BG1 tilemap entry (dedup of words not required here).

- [ ] **Step 4: Test it**

```rust
#[test]
fn dungeon_room_index_probe_reads_a_real_room() {
    let mut game = load_translated_replay_state(concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc"));
    let (theme, tiles) = dungeon_room_index_probe(&mut game, 0x0002);
    assert!(theme != 0, "theme should be set for a dungeon room");
    assert!(!tiles.is_empty(), "room should have BG tiles");
    // patterns are real 4bpp indices (0..16) and not uniformly zero
    assert!(tiles.iter().all(|(_, p)| p.iter().all(|&i| i < 16)));
    assert!(tiles.iter().any(|(_, p)| p.iter().any(|&i| i != 0)));
}
```

Run: `cargo test -p zelda3-bin dungeon_room_index_probe_reads_a_real_room -- --nocapture`. Expected: PASS (after pinning the sequence). If the room can't be made to draw, report BLOCKED with what was tried — do NOT fake the data.

- [ ] **Step 5: Commit**

This touches main.rs (WIP) — leave it UNCOMMITTED. Record the pinned `DUNGEON_BG_CHR_BASE`, the BG1 accessor, and the load sequence in the report and as code comments. No commit for this task (WIP file). Note the pinned facts in `.superpowers/sdd/` for later tasks.

---

### Task 2: `--dump-dungeon-index-tiles` — walk rooms, dedup by `(theme, graphics_key)`, emit assets

**Files:**
- Modify: `zelda3-bin/src/main.rs` (new command + dispatch arm)
- Create (generated): `zelda3-bin/developer_tilesets/dungeon_index_tiles.{bin,json}`

**Interfaces:**
- Consumes: `dungeon_room_index_probe` + `DUNGEON_BG_CHR_BASE` (Task 1), `decode_snes_4bpp_tile_indices` (existing).
- Produces: the two asset files; manifest `{ "format":"zelda3_dungeon_index_tiles_v1", "tile_width_px":8, "tile_height_px":8, "cell_count":N, "cells":[{"id":u32,"keys":[u32]}] }` (`keys` packed `theme<<16|graphics_key`).

- [ ] **Step 1: Add a dispatch arm**

Mirror `--dump-unique-overworld-tiles` dispatch (main.rs ~205): `if args.get(1) == Some("--dump-dungeon-index-tiles") { run_dump_dungeon_index_tiles(&args[2..]); return; }`.

- [ ] **Step 2: Implement the collector + walk**

```rust
fn run_dump_dungeon_index_tiles(_args: &[String]) {
    use std::collections::HashMap;
    let rom = concat!(env!("CARGO_MANIFEST_DIR"), "/../saves/zelda3.sfc");
    let mut cells: Vec<[u8; 64]> = Vec::new();
    let mut index_by_pattern: HashMap<[u8; 64], usize> = HashMap::new();
    let mut keys_by_cell: Vec<std::collections::BTreeSet<u32>> = Vec::new();
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    for room in 0u16..0x128 {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut game = load_translated_replay_state(rom);
            dungeon_room_index_probe(&mut game, room)
        }));
        let (theme, tiles) = match result { Ok(v) => v, Err(_) => continue };
        for (word, pattern) in tiles {
            let key = ((theme as u32) << 16) | ((word & 0xC3FF) as u32);
            let id = *index_by_pattern.entry(pattern).or_insert_with(|| {
                cells.push(pattern);
                keys_by_cell.push(Default::default());
                cells.len() - 1
            });
            keys_by_cell[id].insert(key);
        }
    }
    std::panic::set_hook(original_hook);
    // write .bin (cells*64) + .json to the canonical developer_tilesets path
    // (hardcode like the overworld dump; see brief).
    // print: dumped dungeon index atlas cells=N
}
```

Write `.bin` (each cell's 64 bytes, cell 0 first) and `.json` to hardcoded `concat!(env!("CARGO_MANIFEST_DIR"), "/developer_tilesets/dungeon_index_tiles.{bin,json}")`. Serialize cells as `{id, keys: keys_by_cell[id] sorted}`.

- [ ] **Step 3: Build + run (regenerate)**

Run: `cargo build --profile parity -p zelda3-bin` then `target/parity/zelda3 --dump-dungeon-index-tiles` (may take 1–3 min; background if needed; macOS has no `timeout`). Expected: prints `dumped dungeon index atlas cells=N` with N>0.

- [ ] **Step 4: Validate the asset**

Run:
```bash
python3 -c "d=open('zelda3-bin/developer_tilesets/dungeon_index_tiles.bin','rb').read(); import json; m=json.load(open('zelda3-bin/developer_tilesets/dungeon_index_tiles.json')); assert all(b<16 for b in d), 'idx>=16'; assert len(d)==m['cell_count']*64; print('cells',m['cell_count'],'sample keys',m['cells'][0]['keys'][:4])"
```
Expected: no assertion error; cells>0; sample keys are packed `theme<<16|gkey`.

- [ ] **Step 5: Commit (assets only)**

```bash
git add zelda3-bin/developer_tilesets/dungeon_index_tiles.bin zelda3-bin/developer_tilesets/dungeon_index_tiles.json
git commit --no-verify -m "developer: emit dungeon palette-index atlas (cells=N)"
```
Leave main.rs UNCOMMITTED (WIP). Verify `git show --stat HEAD` lists only the two asset files.

---

### Task 3: Load the dungeon index atlas

**Files:**
- Create: `crates/renderer/src/modern_dungeon_atlas.rs`
- Modify: `crates/renderer/src/lib.rs` (`pub mod modern_dungeon_atlas;`, uncommitted)

**Interfaces:**
- Consumes: `crate::modern_index_atlas::ModernIndexTile`.
- Produces: `pub struct ModernDungeonIndexAtlas { pub tile_width_px:u16, pub tile_height_px:u16, pub cells: Vec<ModernIndexTile>, key_to_cell: std::collections::HashMap<u32,usize> }`; `pub fn load_modern_dungeon_index_atlas(repo_root:&Path)->Result<ModernDungeonIndexAtlas,String>`; `pub fn dungeon_index_cell<'a>(atlas:&'a ModernDungeonIndexAtlas, theme:u16, tilemap_entry:u16)->Option<&'a ModernIndexTile>`; and a `#[cfg(test)] pub fn from_keyed_cells_for_test(cells: Vec<ModernIndexTile>, keys: Vec<(u32, usize)>) -> ModernDungeonIndexAtlas`.

- [ ] **Step 1: Write the failing test (real asset + lookup)**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn loads_dungeon_atlas_and_resolves_by_theme() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let atlas = load_modern_dungeon_index_atlas(&root).expect("dungeon atlas loads");
        assert_eq!(atlas.tile_width_px, 8);
        assert!(!atlas.cells.is_empty());
        assert!(atlas.cells.iter().all(|c| c.indices.iter().all(|&i| i < 16)));
        // read a real (theme, gkey) from the manifest during impl and assert it resolves;
        // and that the SAME gkey under a different theme that isn't present returns None or a different cell.
    }
}
```
(During impl, read a real key from the generated manifest to make the resolve assertion concrete.)

- [ ] **Step 2: Run, verify it fails** — `cargo test -p renderer modern_dungeon_atlas -- --nocapture`.

- [ ] **Step 3: Implement** the loader (serde manifest + `.bin` indices by cell id; `key_to_cell` from each cell's `keys`) and `dungeon_index_cell` (`((theme as u32)<<16)|((word & 0xC3FF) as u32)`). Add `pub mod modern_dungeon_atlas;` to lib.rs.

- [ ] **Step 4: Run, verify pass.** Also `cargo build -p renderer` warning-free; rustfmt clean on the new file.

- [ ] **Step 5: Commit**

```bash
git add crates/renderer/src/modern_dungeon_atlas.rs
git commit --no-verify -m "renderer: load dungeon palette-index atlas"
```
Leave lib.rs uncommitted. Verify `git show --stat HEAD` lists only modern_dungeon_atlas.rs.

---

### Task 4: Make the indexed renderers atlas-agnostic (cell-lookup over a slice)

The software + GPU indexed renderers currently take `&ModernIndexAtlas`. Refactor them to resolve `cell_id → &ModernIndexTile` from a `&[ModernIndexTile]` slice so both the overworld and dungeon atlases feed them. No behavior change; keep the byte-exact tests.

**Files:**
- Modify: `crates/renderer/src/modern_software.rs`, `crates/renderer/src/modern_gpu.rs`
- Modify: `crates/renderer/src/modern_index_atlas.rs` (expose `cells()` if needed)

**Interfaces:**
- Produces: `render_modern_frame_software_indexed(frame: &ModernFrame, cells: &[ModernIndexTile]) -> Vec<u8>` and `ModernGpuIndexRenderer::new(device, queue, cells: &[ModernIndexTile], format)` (and `render`) keyed off a cells slice. Callers pass `&atlas.cells`.

- [ ] **Step 1: Update the existing tests to the new signature**

Change the existing `software_renderer_downsamples...`/`modern_gpu_downsampled...`/indexed tests to pass `&atlas.cells` (or the synthetic cells slice) instead of `&atlas`. Keep the full-buffer `assert_eq!` intact. Run them first to confirm they now FAIL to compile (signature changed) — that's the RED.

- [ ] **Step 2: Refactor the renderers** to take `cells: &[ModernIndexTile]` and look up `cells.get(cell_id as usize)` (ids are dense). GPU: build the R8Uint atlas texture from the `cells` slice. Update `ModernGpuIndexRenderer::new` accordingly.

- [ ] **Step 3: Run** `cargo test -p renderer modern_ -- --nocapture` — all pass (byte-exact preserved); `cargo build -p renderer` warning-free; rustfmt clean.

- [ ] **Step 4: Commit**

```bash
git add crates/renderer/src/modern_software.rs crates/renderer/src/modern_gpu.rs crates/renderer/src/modern_index_atlas.rs
git commit --no-verify -m "renderer: indexed renderers take a cells slice (atlas-agnostic)"
```
Verify `git show --stat HEAD` lists only those renderer files.

---

### Task 5: `extract_modern_frame_with_dungeon_atlas`

**Files:**
- Modify: `crates/renderer/src/modern_extract.rs`

**Interfaces:**
- Consumes: `ModernDungeonIndexAtlas`, `dungeon_index_cell`, `crate::modern_palette::cgram_words_to_rgba256`, `crate::modern_frame::ModernIndexTileInstance`.
- Produces: `pub fn extract_modern_frame_with_dungeon_atlas(frame: &GpuFrame<'_>, atlas: &ModernDungeonIndexAtlas, theme: u16) -> ModernFrame`.

- [ ] **Step 1: Write the failing test (synthetic atlas keyed by theme)**

Build a synthetic `ModernDungeonIndexAtlas` via `from_keyed_cells_for_test` with a cell keyed by `((THEME as u32)<<16)|(word & 0xC3FF)`. Use `test_gpu_frame` with `vram[0]=word`, `bg[0].tilemap_adr=0`, `screen_enabled=[0x01,0x00]`, some `cgram`. Assert: one `index_tiles` instance on layer 0 with the right `cell_id`, `palette==(word>>10)&7`, and `cgram_rgba[0]==snes_cgram_to_rgba(frame.cgram[0])`. Also assert that calling with a DIFFERENT theme yields zero tiles (the key won't resolve).

- [ ] **Step 2: Run, verify it fails.**

- [ ] **Step 3: Implement** — mirror `extract_modern_frame_with_index_atlas` but resolve via `dungeon_index_cell(atlas, theme, word)`; fill `cgram_rgba`; read the dungeon BG1 layer tilemap from `frame` (same 32×32 VRAM read the overworld index extract uses). Emit `ModernIndexTileInstance { cell_id, screen_x:(col*8) as i16 - h_scroll as i16, screen_y:(row*8) as i16 - v_scroll as i16, palette:((word>>10)&7) as u8, hflip:false, vflip:false }`.

- [ ] **Step 4: Run** `cargo test -p renderer modern_extract -- --nocapture` (pass incl existing), `cargo build -p renderer` warning-free, rustfmt clean.

- [ ] **Step 5: Commit**

```bash
git add crates/renderer/src/modern_extract.rs
git commit --no-verify -m "renderer: extract dungeon indexed bg tiles by theme"
```

---

### Task 6: Mode-gated dungeon compare in `run_replay_save` + measurement

**Files:**
- Modify: `zelda3-bin/src/main.rs` (extend the `--modern-index-compare` block from the palette plan)

**Interfaces:**
- Consumes: `load_modern_dungeon_index_atlas`, `extract_modern_frame_with_dungeon_atlas`, `render_modern_frame_software_indexed`, the overworld index path; `game.game_state.frame.main_module` and `game.game_state.world.palette_theme.main_tile_theme_index()`.

- [ ] **Step 1: Wire mode-gating**

In the `run_replay_save` index-compare block: load BOTH atlases once. Per measured frame, read `let module = game.game_state.frame.main_module; let theme = game.game_state.world.palette_theme.main_tile_theme_index();`. If `module == 9` use the overworld index path; if `module == 7 || module == 16` use `extract_modern_frame_with_dungeon_atlas(&gpu_frame, &dungeon_atlas, theme)`; else skip. Render via `render_modern_frame_software_indexed(&modern, &<atlas>.cells)`, count RGB-mismatched pixels vs classic, and log `modern_index_compare frame=N mode=<ow|dungeon> mismatch_px=K modern_tiles=T`. Keep the env-gated PNG dump.

- [ ] **Step 2: Build** `cargo build --profile parity -p zelda3-bin` — warning-free.

- [ ] **Step 3: Measure on a dungeon segment**

Run the route to a dungeon segment (Hyrule Castle is early) with the 7 timing hacks + `--modern-index-compare 500` (background if slow). Find a frame logged `mode=dungeon` with high `modern_tiles`; re-run with `ZELDA3_MODERN_INDEX_DUMP_FRAME=<n>` to dump `/tmp/classic_<n>.png` + `/tmp/modern_index_<n>.png`. Report the mismatch table and the dungeon PNGs.

- [ ] **Step 4: Commit**

No commit (main.rs WIP). Confirm `git status --short` shows only main.rs (+ prior WIP) modified.

---

## Final Verification

```bash
cargo fmt --check -p renderer
cargo test -p renderer modern_ -- --nocapture
cargo test -p zelda3-bin dungeon_room_index_probe_reads_a_real_room -- --nocapture
cargo build --profile parity -p zelda3-bin
python3 -c "import json; m=json.load(open('zelda3-bin/developer_tilesets/dungeon_index_tiles.json')); print('dungeon cells', m['cell_count'])"
```

Expected: renderer unit tests pass (incl. dungeon lookup + extract, and the byte-exact software/GPU tests on the slice signature); the dungeon atlas exists; the dungeon compare logs `mode=dungeon` frames with `modern_tiles>0`, and the dumped PNGs show dungeon BG tiles rendering with correct colors (remaining mismatch attributable to sprites/color-math/coverage — out of scope).

## Self-Review

- **Spec coverage:** capture/dump (T1 spike + T2), load (T3), renderer reuse via slice (T4), dungeon extract (T5), mode-gating + measurement (T6). Blockset key, graphics_key, packed key, mode values, CGRAM reuse all carried from the spec's Global Constraints.
- **Placeholder scan:** the only deferred specifics are the empirical constants in T1 (DUNGEON_BG_CHR_BASE, BG1 accessor, load sequence) — that is the explicit purpose of the spike, with a concrete tested deliverable, not a hidden placeholder. T3's real-manifest key is read at impl time (called out).
- **Type consistency:** `dungeon_room_index_probe` (T1) → consumed by T2; `ModernDungeonIndexAtlas`/`dungeon_index_cell`/`from_keyed_cells_for_test` (T3) → consumed by T5/T6; the cells-slice renderer signatures (T4) → used by T5/T6; `ModernIndexTile`/`ModernIndexTileInstance`/`cgram_words_to_rgba256` reused from the overworld palette plan.
- **Scope control:** dungeon BG color only; sprites/color-math/window/priority and BG2 (unless measurement demands) explicitly deferred. Classic stays default/oracle.
