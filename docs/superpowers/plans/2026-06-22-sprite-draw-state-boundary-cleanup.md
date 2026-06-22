# Sprite Draw State Boundary Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Modernize the highest-risk remaining sprite draw raw-state access by replacing Lanmola trail address math with a named, parity-preserving `game_state` API.

**Architecture:** Keep byte parity as the authority. Add a narrow read-only raw trail view for the Lanmola 192-slot flat trail because the existing native Moldorm/Beamos models intentionally cover different 128-slot banks. Migrate the draw code to this API, then add a lexical ratchet so the raw address pattern does not return.

**Tech Stack:** Rust, Cargo tests, existing `zparity` route evidence gate, `scripts/check_ram_readability.py`.

## Global Constraints

- Do not merge or depend on `refactor/mothula-slot-views`; current `main` is the source of truth.
- Do not convert Lanmola trail reads to owned native state in this slice; the flat trail is an intentional raw WRAM alias surface.
- Preserve byte-backed parity: every migrated draw read must still read from `ram`, not from the 128-slot native history vectors.
- Keep raw address knowledge in `crates/zelda3/src/game_state/native/effects.rs` and expose a named API to draw code.
- Verification must include focused Rust tests, formatting, the route-evidence coverage gate, and a short `zparity check` window.

---

## File Structure

- Modify `crates/zelda3/src/game_state/native/effects.rs`
  - Add `LanmolaFlatTrailEntry`.
  - Add `lanmola_flat_trail_entry_from_ram(ram: &[u8], slot: usize) -> LanmolaFlatTrailEntry`.
- Modify `crates/zelda3/src/game_state/native.rs`
  - Re-export the new type/function.
  - Add unit tests proving the reader uses the 192-slot raw alias region.
- Modify `crates/zelda3/src/zelda_rtl.rs`
  - Add `ZeldaState::lanmola_flat_trail_entry(slot)`.
  - Add a lexical ratchet test that blocks direct Lanmola raw-address reads in `sprite_main_draw.rs`.
- Modify `crates/zelda3/src/sprite_main_draw.rs`
  - Replace direct Lanmola raw reads with `self.lanmola_flat_trail_entry(slot)`.
- Modify `docs/porting/semantic_parity_status.md`
  - Record the new boundary: Lanmola flat trail remains byte-backed and named, not graduated.

---

### Task 1: Add a Named Lanmola Flat Trail Reader

**Files:**
- Modify: `crates/zelda3/src/game_state/native/effects.rs`
- Modify: `crates/zelda3/src/game_state/native.rs`

**Interfaces:**
- Consumes: `MOLDORM_HISTORY_X_LO`, `MOLDORM_HISTORY_Y_LO`, `BEAMOS_LASER_HISTORY_X_HI`, `BEAMOS_LASER_HISTORY_Y_HI`.
- Produces:
  - `LanmolaFlatTrailEntry`
  - `lanmola_flat_trail_entry_from_ram(ram: &[u8], slot: usize) -> LanmolaFlatTrailEntry`

- [x] **Step 1: Write the failing unit tests**

Add these tests near `native_sprite_history_bridges_update_position_and_motion_banks` in `crates/zelda3/src/game_state/native.rs`:

```rust
#[test]
fn lanmola_flat_trail_entry_reads_raw_192_slot_alias_region() {
    let mut ram = vec![0; WRAM_SIZE];
    let slot = 0x82;
    ram[MOLDORM_HISTORY_X_LO + slot] = 0x34;
    ram[MOLDORM_HISTORY_Y_LO + slot] = 0x56;
    ram[BEAMOS_LASER_HISTORY_X_HI + slot] = 0x78;
    ram[BEAMOS_LASER_HISTORY_Y_HI + slot] = 0x09;

    let entry = lanmola_flat_trail_entry_from_ram(&ram, slot);

    assert_eq!(entry.x_low(), 0x34);
    assert_eq!(entry.y_low(), 0x56);
    assert_eq!(entry.z_offset(), 0x78);
    assert_eq!(entry.direction(), 0x09);
}

#[test]
fn lanmola_flat_trail_entry_prefers_raw_ram_over_native_128_slot_history() {
    let mut ram = vec![0; WRAM_SIZE];
    let slot = 0x82;
    ram[MOLDORM_HISTORY_X_LO + slot] = 0xaa;
    ram[MOLDORM_HISTORY_Y_LO + slot] = 0xbb;
    ram[BEAMOS_LASER_HISTORY_X_HI + slot] = 0xcc;
    ram[BEAMOS_LASER_HISTORY_Y_HI + slot] = 0xdd;

    let native = EffectState::load_from_ram(&ram);
    let native_moldorm = native.sprite_histories.moldorm_history(slot);
    let native_motion = native.sprite_histories.lanmola_segment_motion(slot);
    assert_eq!(native_moldorm.x(), 0);
    assert_eq!(native_moldorm.y(), 0);
    assert_eq!(native_motion.z_offset(), 0xcc);
    assert_eq!(native_motion.direction(), 0xdd);

    let entry = lanmola_flat_trail_entry_from_ram(&ram, slot);
    assert_eq!(entry.x_low(), 0xaa);
    assert_eq!(entry.y_low(), 0xbb);
    assert_eq!(entry.z_offset(), 0xcc);
    assert_eq!(entry.direction(), 0xdd);
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run:

```bash
cargo test -p zelda3 lanmola_flat_trail_entry
```

Expected: FAIL with `cannot find function lanmola_flat_trail_entry_from_ram`.

- [x] **Step 3: Add the reader implementation**

In `crates/zelda3/src/game_state/native/effects.rs`, add this after `LanmolaSegmentMotionState`:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct LanmolaFlatTrailEntry {
    x_low: u8,
    y_low: u8,
    z_offset: u8,
    direction: u8,
}

impl LanmolaFlatTrailEntry {
    pub(crate) fn x_low(&self) -> u8 {
        self.x_low
    }

    pub(crate) fn y_low(&self) -> u8 {
        self.y_low
    }

    pub(crate) fn z_offset(&self) -> u8 {
        self.z_offset
    }

    pub(crate) fn direction(&self) -> u8 {
        self.direction
    }
}

pub(crate) fn lanmola_flat_trail_entry_from_ram(
    ram: &[u8],
    slot: usize,
) -> LanmolaFlatTrailEntry {
    LanmolaFlatTrailEntry {
        x_low: ram[MOLDORM_HISTORY_X_LO + slot],
        y_low: ram[MOLDORM_HISTORY_Y_LO + slot],
        z_offset: ram[BEAMOS_LASER_HISTORY_X_HI + slot],
        direction: ram[BEAMOS_LASER_HISTORY_Y_HI + slot],
    }
}
```

In `crates/zelda3/src/game_state/native.rs`, add the re-export inside the existing `pub(crate) use effects::{ ... }` list:

```rust
lanmola_flat_trail_entry_from_ram, LanmolaFlatTrailEntry,
```

- [x] **Step 4: Run the tests to verify they pass**

Run:

```bash
cargo test -p zelda3 lanmola_flat_trail_entry
```

Expected: PASS for both new tests.

- [x] **Step 5: Format and commit**

Run:

```bash
cargo fmt --all
git add crates/zelda3/src/game_state/native/effects.rs crates/zelda3/src/game_state/native.rs
git commit -m "refactor(game_state): name Lanmola flat trail reads"
```

Expected: commit succeeds.

---

### Task 2: Route Lanmola Draw Through the Named Reader

**Files:**
- Modify: `crates/zelda3/src/zelda_rtl.rs`
- Modify: `crates/zelda3/src/sprite_main_draw.rs`

**Interfaces:**
- Consumes: `lanmola_flat_trail_entry_from_ram(ram, slot)`.
- Produces: `ZeldaState::lanmola_flat_trail_entry(&self, slot: usize) -> LanmolaFlatTrailEntry`.

- [x] **Step 1: Add the failing lexical ratchet**

In `crates/zelda3/src/zelda_rtl.rs`, add this test after the existing semantic-view migration tests:

```rust
#[test]
fn lanmola_draw_uses_named_flat_trail_reader() {
    let source = include_str!("sprite_main_draw.rs");
    for needle in [
        "self.ram[MOLDORM_HISTORY_X_LO +",
        "self.ram[MOLDORM_HISTORY_Y_LO +",
        "self.ram[BEAMOS_LASER_HISTORY_X_HI +",
        "self.ram[BEAMOS_LASER_HISTORY_Y_HI +",
    ] {
        assert!(
            !source.contains(needle),
            "sprite_main_draw.rs should use lanmola_flat_trail_entry instead of {needle}"
        );
    }
    assert!(
        source.contains("lanmola_flat_trail_entry("),
        "sprite_main_draw.rs should route Lanmola trail reads through the named API"
    );
}
```

- [x] **Step 2: Run the ratchet to verify it fails**

Run:

```bash
cargo test -p zelda3 zelda_rtl::tests::lanmola_draw_uses_named_flat_trail_reader -- --exact
```

Expected: FAIL because `sprite_main_draw.rs` still contains direct `self.ram[...]` Lanmola history reads.

- [x] **Step 3: Add the `ZeldaState` helper**

In `crates/zelda3/src/zelda_rtl.rs`, extend the native import list to include:

```rust
lanmola_flat_trail_entry_from_ram, LanmolaFlatTrailEntry,
```

Then add this helper after `lanmola_segment_motion_mut`:

```rust
pub(crate) fn lanmola_flat_trail_entry(&self, slot: usize) -> LanmolaFlatTrailEntry {
    lanmola_flat_trail_entry_from_ram(&self.ram, slot)
}
```

- [x] **Step 4: Replace the shrapnel trail read**

In `crates/zelda3/src/sprite_main_draw.rs`, replace the direct read block inside `sprite_54_lanmolas`:

```rust
let trail = self.lanmola_flat_trail_entry(i);
let xlo = trail
    .x_low()
    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
let ylo = trail
    .y_low()
    .wrapping_sub(trail.z_offset())
    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
```

Remove the local `use crate::game_state::constants::{ ... };` block that existed only for this raw read.

- [x] **Step 5: Replace both Lanmola draw loop reads**

In `lanmola_draw`, replace the first loop raw read with:

```rust
let trail = self.lanmola_flat_trail_entry(hist);
let entry_x = trail
    .x_low()
    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
self.oam_state_mut().set_entry_x(oam, entry_x);
if !sign8(trail.z_offset()) {
    let entry_y = trail
        .y_low()
        .wrapping_sub(trail.z_offset())
        .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
    self.oam_state_mut().set_entry_y(oam, entry_y);
}
let dir = usize::from(trail.direction());
```

Replace the second loop raw read with:

```rust
let trail = self.lanmola_flat_trail_entry(hist);
let entry_x = trail
    .x_low()
    .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_h_copy2_low());
self.oam_state_mut().set_entry_x(oam, entry_x);
if !sign8(trail.z_offset()) {
    let entry_y = trail
        .y_low()
        .wrapping_add(10)
        .wrapping_sub(self.game_state.display.ppu_scroll_copy.bg2_v_copy2_low());
    self.oam_state_mut().set_entry_y(oam, entry_y);
}
```

Remove the now-unused `use crate::game_state::constants::{ ... };` blocks in both loops.

- [x] **Step 6: Run the focused tests**

Run:

```bash
cargo test -p zelda3 lanmola_flat_trail_entry
cargo test -p zelda3 zelda_rtl::tests::lanmola_draw_uses_named_flat_trail_reader -- --exact
```

Expected: both commands PASS.

- [x] **Step 7: Run a short parity smoke check**

Run:

```bash
cargo build --profile parity -p zelda3-bin
./target/debug/zparity check --frames 20000
```

Expected: build succeeds and `zparity check` prints `MATCH  20000 frames`.

- [x] **Step 8: Commit**

Run:

```bash
git add crates/zelda3/src/zelda_rtl.rs crates/zelda3/src/sprite_main_draw.rs
git commit -m "refactor(sprite): route Lanmola draw trail reads through game_state"
```

Expected: commit succeeds.

---

### Task 3: Document the Boundary and Run the Full Cleanup Gate

**Files:**
- Modify: `docs/porting/semantic_parity_status.md`

**Interfaces:**
- Consumes: the new named Lanmola flat trail API from Task 1 and Task 2.
- Produces: documented cleanup boundary and final verification evidence.

- [x] **Step 1: Update the semantic parity status doc**

In `docs/porting/semantic_parity_status.md`, append this paragraph to the end of `## Active Layers`:

```markdown
- Lanmola flat trail reads now route through a named byte-backed reader. This
  remains a compatibility surface, not graduated owned state: the Lanmola trail
  uses 192 flat raw slots across Moldorm and Beamos history pages, while the
  native Moldorm/Beamos history models intentionally cover 128-slot banks.
```

- [x] **Step 2: Run formatting and lexical checks**

Run:

```bash
cargo fmt --all -- --check
python3 scripts/check_ram_readability.py --report-direct-ram
git diff --check
```

Expected:
- `cargo fmt --all -- --check` exits 0.
- `python3 scripts/check_ram_readability.py --report-direct-ram` exits 0.
- `git diff --check` exits 0.

- [x] **Step 3: Run focused and crate tests**

Run:

```bash
cargo test -p zelda3 lanmola_flat_trail_entry
cargo test -p zelda3 zelda_rtl::tests::lanmola_draw_uses_named_flat_trail_reader -- --exact
cargo test -p parity
```

Expected: all three commands pass.

- [x] **Step 4: Run the route-evidence coverage gate**

Run:

```bash
cargo build -p parity
./target/debug/zparity coverage \
  --from-json .cache/parity-golden/coverage-merged-route-probed.json \
  --route-report-json .cache/parity-golden/coverage-route-evidence-report.json \
  --route-worklist-json .cache/parity-golden/coverage-route-worklist.json \
  --require-route-full
```

Expected:

```text
main_modules: 25/25 (100.0%)
module_states: 1495/1495 (100.0%)
sprite_types: 232/232 (100.0%)
ancilla_types: 60/60 (100.0%)
indoor_rooms: 296/296 (100.0%)
overworld_screens: 83/83 (100.0%)
active_items: 20/20 (100.0%)
```

- [x] **Step 5: Run the parity smoke check**

Run:

```bash
cargo build --profile parity -p zelda3-bin
./target/debug/zparity check --frames 20000
```

Expected: build succeeds and `zparity check` prints `MATCH  20000 frames`.

- [x] **Step 6: Commit**

Run:

```bash
git add docs/porting/semantic_parity_status.md
git commit -m "docs(porting): record Lanmola draw state boundary"
```

Expected: commit succeeds.

---

## Final Review Checklist

- [x] `rg -n "self\\.ram\\[(MOLDORM_HISTORY|BEAMOS_LASER_HISTORY)" crates/zelda3/src/sprite_main_draw.rs` prints no matches.
- [x] `rg -n "lanmola_flat_trail_entry" crates/zelda3/src` shows the reader, the `ZeldaState` helper, the draw call sites, and the ratchet test.
- [x] `cargo test -p zelda3 lanmola_flat_trail_entry` passes.
- [x] `cargo test -p zelda3 zelda_rtl::tests::lanmola_draw_uses_named_flat_trail_reader -- --exact` passes.
- [x] `cargo test -p parity` passes.
- [x] `./target/debug/zparity coverage --from-json .cache/parity-golden/coverage-merged-route-probed.json --require-route-full` passes when invoked with the route report/worklist paths from Task 3.
- [x] `./target/debug/zparity check --frames 20000` prints `MATCH`.
