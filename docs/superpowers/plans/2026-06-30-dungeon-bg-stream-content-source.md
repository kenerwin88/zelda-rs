# Dungeon BG-Stream Content-Source Tagging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive the off-VRAM ("assets-anim") dungeon renderer to literal-0 pixel parity with the classic renderer by giving the per-frame-streamed dungeon BG CHR a robust, injective logical-source tag, so the floor/room tiles resolve to the correct asset cell instead of a stale one.

**Architecture:** The dungeon re-streams room-specific BG CHR into VRAM every room transition via NMI char-update DMAs (`nmi_update_bg_char3and4` → 0x2c00, `nmi_run_tile_map_update_dma` → 0x2000/0x2800/0x3000/0x3800, etc.) **without updating the `vram_chr_source` table**, leaving each affected slot tagged with the stale `(BG, pack, off)` value from the room-entry `do3_to_4` load. The off-VRAM renderer then resolves those slots to the *initial* blockset cell (wrong floor). The fix tags the streamed slots at the **streaming writer** (the authoritative last writer of those slots) with a **24-bit content hash** of the bytes just written — `kind = CHR_KIND_BG_STREAM`, `pack = (hash>>8)&0xffff`, `tile_off = hash&0xff`. This is injective by content (identical pixels → identical key), self-healing (re-hashed on every stream so it can never go stale), needs no fill-pipeline provenance tracing, and — because the existing dump keys cells by the live source-table entry and the existing `modern_source_key` generic packing already encodes a 24-bit payload — needs **no atlas-format, key-packing, or dump-logic changes**, only an asset re-dump.

**Tech Stack:** Rust; crates `zelda3` (game/NMI/source-table), `renderer` (off-VRAM extract/atlas), binary `zelda3-bin` (dump + compare harness). Parity build profile `parity`. No new dependencies.

## Global Constraints

- **Off-VRAM-PIXEL invariant:** the off-VRAM render path (`extract_modern_frame_from_sources` / `extract_modern_sprites_from_sources`) must not read `frame.vram` CHR pixel content for game art. Hashing of streamed CHR happens at the **write** site (in the `zelda3` crate), recorded into the source table; the renderer only reads the table. (BG3 HUD remains the one documented live-VRAM exception — do not touch it.)
- **Overworld must stay at literal 0.** The overworld off-VRAM path is currently pixel-exact (`mismatch_px=0`). All new tagging is **indoor-gated** (`self.game_state.world.location.indoor_flag() != 0`) so the overworld keys are untouched. Every task that changes tagging MUST re-verify OW `mismatch_px=0`.
- **Classic + live-VRAM modern modes stay intact and default.** This work only changes the off-VRAM (`ZELDA3_RENDERER=assets-anim`) results and the streamed-slot source tags. The live-VRAM path (`via=vram`) is the oracle and must remain `mismatch_px=0` at every dungeon frame.
- **CHR kinds:** existing kinds are `CHR_KIND_NONE=0, CHR_KIND_BG=1, CHR_KIND_SPRITE=2, CHR_KIND_LINK=3, CHR_KIND_BG3=4, CHR_KIND_BG_ANIM=5`. New kind is `CHR_KIND_BG_STREAM=6`.
- **Key packing is fixed:** `modern_source_key(kind, pack, tile_off)` for non-Link kinds is `(kind<<24)|((pack as u32)<<8)|((tile_off as u32)&0xff)`. BG_STREAM reuses this generic packing; the 24-bit hash MUST be split exactly as `pack=(hash>>8)&0xffff`, `tile_off=hash&0xff` so the key equals `(6<<24)|(hash&0x00ff_ffff)`.
- **Commit conventions:** new files (chr_source additions are in an existing file) and the regenerated `assets_by_source.{bin,json}` are committed. Edits to files carrying the user's uncommitted WIP (`nmi.rs`, `load_gfx.rs`, `lib.rs`, `main.rs`, `zelda_rtl.rs`) are left **uncommitted** unless the touched region is WIP-free — confirm per-file at execution time. Use `git commit --no-verify` (heavy parity pre-commit hook; the user works the repo concurrently). **Never `git checkout <file>`** (nukes WIP) — revert your own edits surgically.
- **Timing-hack env for every replay** (all 7): `ZELDA3_SMV_{SELECT_FILE,LOADFILE,DUNGEON,OVERWORLD,MESSAGING,DEATH_INTRO,DEATH_RELOAD}_TIMING_HACKS=1`.

## Proven Root Cause (from systematic-debugging Phase 1 — do not re-investigate)

- Frame 27800 (worst dungeon frame): live-VRAM `mismatch_px=0`, off-VRAM `mismatch_px=19462`. Identical CGRAM + identical renderer ⇒ the residual is **purely a cell/source mismatch, not palette/coverage**.
- Diagnostic at 27800: 4096 BG tiles → **942 wrong_cell, 0 gap**. Wrong tiles are the floor (`tile 0x0ee/0ef/0fe/0ff` → source slots `0x2ee/2ef/2fe/2ff` → VRAM word ~0x2ee0, in the 0x2c00 region), tagged `(BG, pack=0x06, off=46/47/62/63)` by `initialize_tilesets`’s `load_background_graphics(0x2c00, aux_bg_subset(0), …)`.
- VRAM write-watch on word 0x2ee0: the **only** writer is `copy_bytes_to_vram dst=0x2c00 words=0x800` = `nmi_update_bg_char3and4` (fires 10× over the route, on room transitions), streaming `background_character_buffer()` (WRAM `0x10000`). It does **not** call `record_tiles` ⇒ stale tag.
- This is the same class the prior agent fixed for `nmi_update_bg_char5and6` (0x3400) with a theme key in commit `8bb68cb`; it fixed only one of several untagged streaming writers, and theme keys are non-injective for room-specific content (which is why this plan uses a content hash, not a theme key).

## File Structure

- `crates/zelda3/src/chr_source.rs` — add `CHR_KIND_BG_STREAM`, a `chr_content_hash24(&[u16]) -> u32` helper, and a `record_tile_content_hash(slot, kind, hash24)` method on `VramChrSourceTable`. (Pure, unit-testable; no WIP here — committable.)
- `crates/zelda3/src/nmi.rs` — in the streaming BG-char writers, after the VRAM copy, hash each streamed 8×8 tile from the just-written VRAM words and record `(BG_STREAM, hash)` for indoor frames. (Carries user WIP — leave uncommitted.)
- `crates/renderer/src/modern_extract.rs` — add the permanent env-gated `ZELDA3_SRC_DEBUG` diagnostic (verification harness) to `extract_modern_frame_from_sources`. (Committed file, no WIP — committable.)
- `zelda3-bin/developer_tilesets/assets_by_source.{bin,json}` — regenerated to include the new `kind=6` content-hash cells. (Committed assets.)
- No changes required to `modern_source_atlas.rs`, the `modern_source_key` packing, or `run_dump_assets_by_source`’s keying logic (it already keys by the live source-table entry).

---

### Task 1: Permanent `ZELDA3_SRC_DEBUG` verification harness

**Files:**
- Modify: `crates/renderer/src/modern_extract.rs` (inside `extract_modern_frame_from_sources`, the `fn` at the `pub fn extract_modern_frame_from_sources` definition)
- Test: covered by the binary measurement below (this is diagnostic instrumentation, not new product logic)

**Interfaces:**
- Consumes: existing `extract_modern_frame_from_sources(frame, src_table, atlas)`, `decode_snes_4bpp_tile_indices(vram, chr_base_words, tilemap_entry)`, `source_cell(atlas, kind, pack, tile_off)`, `SourceTableView::get(slot) -> (u8,u16,u16)`.
- Produces: env-gated `eprintln!` lines `"[SRC_DEBUG] bg_tiles=N wrong_cell=W gap=G"` plus up to 24 sample lines `"[SRC_DEBUG]   slot=0x.. tile=0x.. src=(kind=..,pack=0x..,off=..)"`. No signature change.

- [ ] **Step 1: Add the diagnostic accumulators.** In `extract_modern_frame_from_sources`, immediately after `let mut cells: Vec<ModernIndexTile> = Vec::new();`, insert:

```rust
    // Permanent env-gated diagnostic (ZELDA3_SRC_DEBUG): per BG1/BG2 tile, compare
    // the off-VRAM resolved atlas cell (unflipped) against the live-VRAM decode of
    // the same slot. wrong_cell>0 means a STALE/WRONG source tag; gap>0 means the
    // recorded key is absent from the atlas. Off when the var is unset (one getenv).
    let dbg = std::env::var("ZELDA3_SRC_DEBUG").is_ok();
    let mut dbg_total = 0usize;
    let mut dbg_mismatch = 0usize;
    let mut dbg_gap = 0usize;
    let mut dbg_samples: Vec<(usize, usize, u8, u16, u16)> = Vec::new();
```

- [ ] **Step 2: Add the per-tile comparison.** In the BG1/BG2 (`else`, i.e. non-BG3) branch, locate `let (kind, pack, tile_off) = src_table.get(slot);` and insert immediately after it (before the `let Some(src) = source_cell(...) else { … };`):

```rust
                    if dbg && layer_index < 2 {
                        dbg_total += 1;
                        let live = decode_snes_4bpp_tile_indices(
                            frame.vram,
                            frame.bg[layer_index].tile_adr as usize,
                            tile_number as u16,
                        );
                        match source_cell(atlas, kind, pack, tile_off) {
                            None => {
                                dbg_gap += 1;
                                if dbg_samples.len() < 24 {
                                    dbg_samples.push((slot, tile_number, kind, pack, tile_off));
                                }
                            }
                            Some(s) if s.indices != live => {
                                dbg_mismatch += 1;
                                if dbg_samples.len() < 24 {
                                    dbg_samples.push((slot, tile_number, kind, pack, tile_off));
                                }
                            }
                            _ => {}
                        }
                    }
```

- [ ] **Step 3: Add the summary print.** Immediately before the function’s final `(modern, cells)` return, insert:

```rust
    if dbg {
        eprintln!("[SRC_DEBUG] bg_tiles={dbg_total} wrong_cell={dbg_mismatch} gap={dbg_gap}");
        for (slot, tile, kind, pack, off) in &dbg_samples {
            eprintln!(
                "[SRC_DEBUG]   slot=0x{slot:03x} tile=0x{tile:03x} src=(kind={kind},pack=0x{pack:04x},off={off})"
            );
        }
    }
```

- [ ] **Step 4: Build.**

Run: `cargo build --profile parity -p zelda3-bin`
Expected: `Finished` with no errors.

- [ ] **Step 5: Capture the baseline (this is the failing state).**

Run (with the 7 timing-hack env vars exported as `HACKS`):
```
env "${HACKS[@]}" ZELDA3_RENDERER=assets-anim ZELDA3_SRC_DEBUG=1 \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 27801 \
  --modern-index-compare 27800 2>&1 | rg 'SRC_DEBUG] bg_tiles|mode=dungeon'
```
Expected: `[SRC_DEBUG] bg_tiles=4096 wrong_cell=942 gap=0` and `modern_index_compare frame=27800 mode=dungeon mismatch_px=19462 via=sources`. This is the RED baseline the rest of the plan drives to 0.

- [ ] **Step 6: Verify renderer unit tests still pass.**

Run: `cargo test -p renderer modern_`
Expected: all pass (the diagnostic is inert without the env var).

- [ ] **Step 7: Commit.**

```bash
git add crates/renderer/src/modern_extract.rs
git commit --no-verify -m "renderer: ZELDA3_SRC_DEBUG off-VRAM cell-vs-live diagnostic"
```

---

### Task 2: `CHR_KIND_BG_STREAM` + content-hash helpers

**Files:**
- Modify: `crates/zelda3/src/chr_source.rs`
- Test: `crates/zelda3/src/chr_source.rs` (inline `#[cfg(test)] mod tests`, or the existing test module if present)

**Interfaces:**
- Produces:
  - `pub const CHR_KIND_BG_STREAM: u8 = 6;`
  - `pub fn chr_content_hash24(words: &[u16]) -> u32` — FNV-1a over the little-endian bytes of `words`, masked to 24 bits; deterministic; returns the same value for identical slices.
  - `VramChrSourceTable::record_tile_content_hash(&mut self, slot: usize, kind: u8, hash24: u32)` — sets `entries[slot] = LogicalChrSrc { kind, pack: ((hash24 >> 8) & 0xffff) as u16, tile_off: (hash24 & 0xff) as u16 }` (bounds-checked, no-op if `slot >= entries.len()`).
- Consumes: existing `LogicalChrSrc { kind: u8, pack: u16, tile_off: u16 }`, `self.entries: [LogicalChrSrc; VRAM_CHR_SLOTS]`.

- [ ] **Step 1: Write the failing tests.** Add to `chr_source.rs`:

```rust
#[cfg(test)]
mod bg_stream_tests {
    use super::*;

    #[test]
    fn content_hash_is_deterministic_and_distinguishes_patterns() {
        let a = [0x1234u16; 16];
        let b = [0x1234u16; 16];
        let c = [0x1235u16, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234,
                 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234, 0x1234];
        assert_eq!(chr_content_hash24(&a), chr_content_hash24(&b));
        assert_ne!(chr_content_hash24(&a), chr_content_hash24(&c));
        assert_eq!(chr_content_hash24(&a) & 0xff00_0000, 0, "must be 24-bit");
    }

    #[test]
    fn record_tile_content_hash_round_trips_through_key_split() {
        let mut t = VramChrSourceTable::default();
        let hash = 0x00ab_cdefu32;
        t.record_tile_content_hash(3, CHR_KIND_BG_STREAM, hash);
        let e = t.get(3);
        assert_eq!(e.kind, CHR_KIND_BG_STREAM);
        // pack = hash>>8 & 0xffff, tile_off = hash & 0xff
        assert_eq!(e.pack, 0xabcd);
        assert_eq!(e.tile_off, 0xef);
        // reconstruct the 24-bit hash from the stored fields
        let recon = ((e.pack as u32) << 8) | (e.tile_off as u32);
        assert_eq!(recon, hash);
    }
}
```

(If `VramChrSourceTable` has no `Default`, construct it the same way the existing tests in the file do; check the top of `chr_source.rs` for the existing constructor and mirror it.)

- [ ] **Step 2: Run the tests; verify they fail.**

Run: `cargo test -p zelda3 bg_stream_tests`
Expected: FAIL — `CHR_KIND_BG_STREAM`, `chr_content_hash24`, and `record_tile_content_hash` are not defined.

- [ ] **Step 3: Implement.** Add near the other `CHR_KIND_*` consts:

```rust
/// Per-frame-streamed dungeon BG CHR (`nmi_update_bg_char*`) is room-specific and
/// re-DMA'd over the same VRAM slots without a stable pack identity, so it is
/// tagged by a 24-bit hash of the streamed pixels. The hash is split into the
/// `(pack, tile_off)` fields so `modern_source_key` encodes it as
/// `(6<<24)|(hash&0xffffff)` with no special key path.
pub const CHR_KIND_BG_STREAM: u8 = 6;

/// FNV-1a 24-bit hash of the little-endian bytes of `words`. Deterministic and
/// identical for identical content (so identical streamed tiles dedup to one
/// asset cell). Masked to 24 bits to fit the `(pack<<8)|tile_off` key payload.
pub fn chr_content_hash24(words: &[u16]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for &w in words {
        for b in [(w & 0xff) as u8, (w >> 8) as u8] {
            h ^= b as u32;
            h = h.wrapping_mul(0x0100_0193);
        }
    }
    (h ^ (h >> 24)) & 0x00ff_ffff
}
```

And add the method in the `impl VramChrSourceTable` block:

```rust
    /// Tag a single CHR slot with a content hash (see [`chr_content_hash24`]),
    /// splitting the 24-bit hash into `pack=(hash>>8)&0xffff`, `tile_off=hash&0xff`.
    pub fn record_tile_content_hash(&mut self, slot: usize, kind: u8, hash24: u32) {
        if slot < self.entries.len() {
            self.entries[slot] = LogicalChrSrc {
                kind,
                pack: ((hash24 >> 8) & 0xffff) as u16,
                tile_off: (hash24 & 0xff) as u16,
            };
        }
    }
```

- [ ] **Step 4: Run the tests; verify they pass.**

Run: `cargo test -p zelda3 bg_stream_tests`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit.**

```bash
git add crates/zelda3/src/chr_source.rs
git commit --no-verify -m "zelda3: CHR_KIND_BG_STREAM + 24-bit content-hash source tagging"
```

---

### Task 3: Tag `nmi_update_bg_char3and4` streamed slots (the floor writer)

**Files:**
- Modify: `crates/zelda3/src/nmi.rs` (`nmi_update_bg_char3and4`, around line 334) — **leave uncommitted (WIP file)**

**Interfaces:**
- Consumes: `CHR_KIND_BG_STREAM`, `chr_content_hash24`, `VramChrSourceTable::record_tile_content_hash` from Task 2; `self.vram_chr_source` (field, same one the existing `record_tiles` calls use); `self.ppu.vram: Vec<u16>`; `self.game_state.world.location.indoor_flag()`.
- Produces: after the 0x2c00 stream, slots `0x2c00/16 .. (0x2c00+0x800)/16` (i.e. `0x2c0 .. 0x3c0`, 256 tiles) are tagged `(CHR_KIND_BG_STREAM, hash)` for indoor frames.

- [ ] **Step 1: Read the current writer.** Confirm `nmi_update_bg_char3and4` is:

```rust
    pub(super) fn nmi_update_bg_char3and4(&mut self) {
        let buf = self.background_character_buffer().to_vec();
        self.copy_to_vram_slice(0x2c00, &buf, 0x1000);
        self.clear_core_update_disable_flag();
    }
```

- [ ] **Step 2: Add indoor-gated content-hash tagging after the copy.** Replace the body with:

```rust
    pub(super) fn nmi_update_bg_char3and4(&mut self) {
        let buf = self.background_character_buffer().to_vec();
        self.copy_to_vram_slice(0x2c00, &buf, 0x1000);
        // Animation-modeled asset renderer: this NMI DMA re-streams room-specific
        // dungeon BG CHR over VRAM 0x2c00-0x3bff every room transition, leaving the
        // room-entry do3->4 `(BG, pack, off)` tag stale. Re-tag the 256 streamed
        // tiles with a 24-bit content hash of the just-written VRAM words so the
        // off-VRAM path resolves the live floor/room cell, not the initial blockset.
        // Indoor-only: the overworld owns these slots via its own keys and is at 0.
        if self.game_state.world.location.indoor_flag() != 0 {
            const BASE: usize = 0x2c00;
            const TILES: usize = 0x100; // 0x800 words / 16 words-per-tile
            for t in 0..TILES {
                let word0 = BASE + t * 16;
                let hash = crate::chr_source::chr_content_hash24(
                    &self.ppu.vram[word0..word0 + 16],
                );
                self.vram_chr_source.record_tile_content_hash(
                    BASE / 16 + t,
                    crate::chr_source::CHR_KIND_BG_STREAM,
                    hash,
                );
            }
        }
        self.clear_core_update_disable_flag();
    }
```

(If `self.vram_chr_source` is accessed by a different path in this file — check the existing `self.vram_chr_source.record_tiles(...)` call in `nmi_update_bg_char5and6` — mirror that exact access.)

- [ ] **Step 3: Build.**

Run: `cargo build --profile parity -p zelda3-bin`
Expected: `Finished`, no errors.

- [ ] **Step 4: Verify the stale tag is gone (the spike gate).**

Run:
```
env "${HACKS[@]}" ZELDA3_RENDERER=assets-anim ZELDA3_SRC_DEBUG=1 \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 27801 \
  --modern-index-compare 27800 2>&1 | rg 'SRC_DEBUG] bg_tiles'
```
Expected: `wrong_cell` drops from 942 toward **0**, with `gap` RISING by roughly the same amount (the floor tiles are now correctly tagged `kind=6` but their hash cells are not yet in the atlas → gaps until Task 4). **This proves `nmi_update_bg_char3and4` is the authoritative last writer of these slots** (its hash tag now wins). If `wrong_cell` does NOT drop, STOP: a later `do3_to_4` is re-tagging after the stream — escalate (the ordering assumption is wrong) instead of proceeding.

- [ ] **Step 5: Verify game-state tests unaffected.**

Run: `cargo test --profile parity -p zelda3 game_state`
Expected: all pass (tagging does not change WRAM/VRAM bytes, only the side-table).

- [ ] **Step 6: Do NOT commit (WIP file `nmi.rs`).** Record progress in the ledger only. If the touched region is confirmed WIP-free at execution time, the controller may stage just these lines; otherwise leave uncommitted.

---

### Task 4: Regenerate `assets_by_source` and verify frame 27800 → 0

**Files:**
- Modify (regenerate): `zelda3-bin/developer_tilesets/assets_by_source.{bin,json}` — **commit assets**
- (No code change; uses `--dump-assets-by-source` from `main.rs` as-is)

**Interfaces:**
- Consumes: the `kind=6` tags produced by Task 3 during the dump replay; `run_dump_assets_by_source` (already keys cells by the live source-table entry via `modern_source_key`).

- [ ] **Step 1: Re-run the asset dump.** This replays the route and bakes one cell per distinct source key, now including the `kind=6` content-hash floor/room cells:

```
env "${HACKS[@]}" target/parity/zelda3 --dump-assets-by-source saves/zelda3.sfc saves/zelda3-combined-route.sav
```
Expected: writes `zelda3-bin/developer_tilesets/assets_by_source.{bin,json}`; cell count grows vs the committed file (new kind-6 cells added).

- [ ] **Step 2: Verify off-VRAM frame 27800 is now 0.**

Run:
```
env "${HACKS[@]}" ZELDA3_RENDERER=assets-anim ZELDA3_SRC_DEBUG=1 \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 27801 \
  --modern-index-compare 27800 2>&1 | rg 'SRC_DEBUG] bg_tiles|mode=dungeon'
```
Expected: `[SRC_DEBUG] bg_tiles=4096 wrong_cell=0 gap=0` and `modern_index_compare frame=27800 mode=dungeon mismatch_px=0 via=sources`.

- [ ] **Step 3: Verify the overworld is still literal 0 (regression gate).**

Run:
```
env "${HACKS[@]}" ZELDA3_RENDERER=assets-anim \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 28000 \
  --modern-index-compare 200 2>&1 | rg 'mode=ow' | awk -F'mismatch_px=' '{print $2+0}' | sort -rn | head -3
```
Expected: top value `0` (every overworld frame still pixel-exact).

- [ ] **Step 4: Verify no dungeon regression and measure improvement.**

Run:
```
env "${HACKS[@]}" ZELDA3_RENDERER=assets-anim \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 28000 \
  --modern-index-compare 200 2>&1 | rg 'mode=dungeon' | awk -F'mismatch_px=' '{s+=$2+0} END{print "dungeon_total="s}'
```
Expected: `dungeon_total` far below the committed-baseline 7965 for the 0–28k window (the 27600/27800/28000 cluster collapses). Any frame that *rose* vs baseline is a regression → STOP and diagnose with `ZELDA3_SRC_DEBUG` at that frame before continuing.

- [ ] **Step 5: Verify the live-VRAM oracle is untouched.**

Run:
```
env "${HACKS[@]}" \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 28000 \
  --modern-index-compare 200 2>&1 | rg 'via=vram' | awk -F'mismatch_px=' '{s+=$2+0} END{print "vram_total="s}'
```
Expected: `vram_total=0`.

- [ ] **Step 6: Commit the assets.**

```bash
git add zelda3-bin/developer_tilesets/assets_by_source.bin zelda3-bin/developer_tilesets/assets_by_source.json
git commit --no-verify -m "assets: kind-6 content-hash dungeon BG-stream cells (floor parity at 27800)"
```

---

### Task 5: Extend to the remaining streaming BG-char writers; drive route-wide to 0

**Files:**
- Modify: `crates/zelda3/src/nmi.rs` — `nmi_run_tile_map_update_dma` (0x2000/0x2800/0x3000/0x3800 via `nmi_update_bg_char0/1/2/3`), `nmi_update_bg_char_half`, and **reconcile** `nmi_update_bg_char5and6` (replace the non-injective theme key with the content hash for indoor). **Leave uncommitted (WIP file).**
- Modify (regenerate): `zelda3-bin/developer_tilesets/assets_by_source.{bin,json}` — **commit assets**

**Interfaces:**
- Consumes: Task 2 helpers. Same indoor-gated, post-copy hashing pattern as Task 3, applied per writer over the exact words each one streams.

- [ ] **Step 1: Tag `nmi_run_tile_map_update_dma`.** It is shared by `nmi_update_bg_char0/1/2/3` (BG) and `nmi_update_obj_char2/3` (OBJ at 0x5000/0x5800 — sprites, OUT of scope). Tag **only BG destinations** (`dst < 0x4000`), indoor-gated. Change `nmi_run_tile_map_update_dma`:

```rust
    pub(super) fn nmi_run_tile_map_update_dma(&mut self, dst: usize) {
        let buf = self.background_character_buffer().to_vec();
        self.copy_to_vram_slice(dst, &buf, 0x1000);
        if dst < 0x4000 && self.game_state.world.location.indoor_flag() != 0 {
            const TILES: usize = 0x100; // 0x800 words / 16
            for t in 0..TILES {
                let word0 = dst + t * 16;
                let hash = crate::chr_source::chr_content_hash24(
                    &self.ppu.vram[word0..word0 + 16],
                );
                self.vram_chr_source.record_tile_content_hash(
                    dst / 16 + t,
                    crate::chr_source::CHR_KIND_BG_STREAM,
                    hash,
                );
            }
        }
        self.clear_core_update_disable_flag();
    }
```

- [ ] **Step 2: Reconcile `nmi_update_bg_char5and6`.** In the **indoor** branch (currently records `(CHR_KIND_BG, 0x4000 | ((main&0x3f)<<6) | (aux&0x3f))` over 0x80 tiles at 0x3400), replace the theme key with the content hash so it is injective like the others. Keep the **overworld** branch (the `0x8000 | theme` key) exactly as-is (OW is at 0). Replace the indoor branch body:

```rust
        } else {
            // DUNGEON: re-stream of room BG CHR over 0x3400; content-hash like the
            // other streaming writers (theme key was non-injective for room-specific
            // tiles). 0x80 tiles = 0x800 words.
            const TILES: usize = 0x80;
            for t in 0..TILES {
                let word0 = 0x3400 + t * 16;
                let hash = crate::chr_source::chr_content_hash24(
                    &self.ppu.vram[word0..word0 + 16],
                );
                self.vram_chr_source.record_tile_content_hash(
                    0x3400 / 16 + t,
                    crate::chr_source::CHR_KIND_BG_STREAM,
                    hash,
                );
            }
        }
```

- [ ] **Step 3: Tag `nmi_update_bg_char_half`.** It streams `background_character_half_buffer()` to a runtime `dst = nmi_load_target_page() * 256`, 0x200 words. Add indoor-gated hashing after its copy loop, tagging `0x200/16 = 0x20` tiles, only when `dst < 0x4000`:

```rust
    pub(super) fn nmi_update_bg_char_half(&mut self) {
        let dst = self.game_state.display.nmi_load_target_page() as usize * 256;
        let buf = self.background_character_half_buffer().to_vec();
        for i in 0..0x200 {
            self.ppu.vram[dst + i] = read_word_from_slice(&buf, i * 2);
        }
        if dst < 0x4000 && self.game_state.world.location.indoor_flag() != 0 {
            const TILES: usize = 0x20; // 0x200 words / 16
            for t in 0..TILES {
                let word0 = dst + t * 16;
                let hash = crate::chr_source::chr_content_hash24(
                    &self.ppu.vram[word0..word0 + 16],
                );
                self.vram_chr_source.record_tile_content_hash(
                    dst / 16 + t,
                    crate::chr_source::CHR_KIND_BG_STREAM,
                    hash,
                );
            }
        }
    }
```

- [ ] **Step 4: Build.**

Run: `cargo build --profile parity -p zelda3-bin`
Expected: `Finished`, no errors.

- [ ] **Step 5: Regenerate assets (now covering all streaming regions).**

Run: `env "${HACKS[@]}" target/parity/zelda3 --dump-assets-by-source saves/zelda3.sfc saves/zelda3-combined-route.sav`
Expected: writes the two asset files; cell count grows again.

- [ ] **Step 6: Route-wide measurement (0–28k) — dungeon to ~0, OW still 0, oracle still 0.**

Run:
```
env "${HACKS[@]}" ZELDA3_RENDERER=assets-anim \
  target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 28000 \
  --modern-index-compare 50 2>&1 \
  | awk -F'[ =]' '/mode=dungeon/{d+=$10} /mode=ow/{o+=$10} END{print "dungeon="d" ow="o}'
```
Expected: `dungeon` near 0 (document any residual frames), `ow=0`. If any dungeon frame remains nonzero, run `ZELDA3_SRC_DEBUG=1` with `--modern-index-compare <thatframe>` to classify (wrong_cell vs gap) and record it as a documented residual (e.g. a streaming writer not yet covered, or a hash collision).

- [ ] **Step 7: Verify full unit-test suites.**

Run: `cargo test -p renderer modern_` then `cargo test --profile parity -p zelda3 game_state`
Expected: all pass (renderer 120, zelda3 game_state 280).

- [ ] **Step 8: Commit assets; leave `nmi.rs` uncommitted (WIP).**

```bash
git add zelda3-bin/developer_tilesets/assets_by_source.bin zelda3-bin/developer_tilesets/assets_by_source.json
git commit --no-verify -m "assets: content-hash all dungeon BG-stream regions (route-wide dungeon parity)"
```
Record the final route-wide dungeon/OW numbers and any residuals in `.superpowers/sdd/progress.md`.

---

## Self-Review

**Spec coverage:** Root cause (stale tag on streamed dungeon BG CHR) → Tasks 3+5 tag every streaming BG-char writer. Injective key requirement → Task 2 content hash (Global Constraints fix the packing). No-dump-change requirement → relies on existing `run_dump_assets_by_source` keying by source-table entry (Task 4/5 just re-dump). OW-must-stay-0 → indoor gating in every tagging change + explicit OW gates in Tasks 4.3 and 5.6. Live-VRAM oracle untouched → Task 4.5. Verification harness → Task 1.

**Placeholder scan:** No TBD/TODO; every code step shows full code; every run step shows the exact command and expected output. The one conditional ("if `self.vram_chr_source` is accessed differently, mirror the existing call") points at a concrete existing call site (`nmi_update_bg_char5and6`) to copy, not an open-ended instruction.

**Type consistency:** `chr_content_hash24(&[u16]) -> u32`, `record_tile_content_hash(slot, kind, hash24)`, and `CHR_KIND_BG_STREAM=6` are defined in Task 2 and used verbatim in Tasks 3 and 5. The 24-bit split (`pack=hash>>8`, `tile_off=hash&0xff`) matches `modern_source_key`'s generic packing in Global Constraints, so `source_cell` resolves kind-6 keys with no atlas change. `[SRC_DEBUG]` line format defined in Task 1 is the gate read in Tasks 3–5.

**Risk note:** The single architectural assumption — the streaming writer is the *last* writer of its slots (so the hash tag wins over the room-entry `do3_to_4` tag) — is explicitly gated in Task 3 Step 4 with a STOP-and-escalate if it does not hold. This is the exact failure mode that defeated the prior agent's content-hash attempt (partial coverage + `do3_to_4` overwrite), now made a hard checkpoint.
```
