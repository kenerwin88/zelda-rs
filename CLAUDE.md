# zelda3-rs — working notes for Claude

This repo is migrating `game_state` from raw-byte WRAM access to a semantic/native
layer (domain-named Rust structs) while preserving **exact C parity** with the
reference emulator. Hard constraint: SNES WRAM reuses the same bytes for different
systems by game mode, so a semantic write must touch only the byte/word it owns —
never bulk-project a range it shares with another system.

## Parity is driven by `zparity` vs the C oracle (the old Rust clone is gone)

Parity is now found and gated against the **C oracle `../zelda3`** only. The old Rust clone
(`zelda3-rs-old`) and its NEW-vs-OLD comparison scripts have been removed — `zparity`
(`crates/parity`, tool #10 below) supersedes them: it captures a per-frame all-layer golden
from C once, then finds the earliest per-frame divergence C-free and in parallel.

When fixing a logic divergence, compare the Rust fn against the **C source** (`../zelda3/src`,
same function names/structure as the disassembly). Note a real consequence of dropping the old
clone: divergences `zparity` now surfaces are ones where the Rust port and C differ per-frame —
the old clone usually *shared* the Rust behavior, so a NEW-vs-OLD diff would have shown nothing.
These need the C source as the reference, not a sibling-Rust diff. `whoowns.py` (tool #3) reads
THIS repo's own const map for address semantics; `validate_all_parity.py` remains the end-state
all-layer gate vs C.

## Common bug classes (almost every root is one of these) + fix recipes

The migration's dominant failure mode: **a native state projects bytes it doesn't exclusively
own, re-stamping a stale frame-start value over another system's live mid-frame write.** The
`GameState::write_to_ram` master projection runs every state unconditionally, last-writer-wins;
bridge `sync()` calls re-run a state's `write_to_ram` mid-frame on every setter. Four shapes:

1. **Stale bulk-projection clobber (mode-reuse / dual-ownership)** — THE most common. Two
   native states project the same SNES-reused byte; whichever projects last clobbers. Fixes,
   in order of preference:
   a. **Mode-gate the projection** on the active mode: `if ram[PLAYER_IS_INDOORS] == 0 {...}`
      (overworld-only) / `if ram[MAIN_MODULE] != 0x0b {...}`. (0x4bc star-tile vs dungeon torch,
      f314953.)
   b. **Write-through, don't bulk-project**: delete the byte from `write_to_ram`; write it to
      RAM directly in the setter; exclude it from the coherence check (add a
      `matches_ram_ignoring_X` / `from_ram.field = self.field` shim). The active subsystem
      writes through; nothing re-stamps it. (0x6a0 scroll-delta→mirror-warp f296375; 0x1dd80
      mapbak palette via PpuScrollCopy scroll-sync f335672; 0xc8 menu-animation-timer.)
   c. **Make ONE struct the sole owner; exclude the shared byte from the other's projection.**
      (0x39d ancilla `g[9]` == hookshot effect index f358784.)
2. **Oversized-table projection** — a native array models MORE slots than the real hardware
   array, so its bulk `for slot in 0..COUNT { ram[BASE+slot]=... }` spills past the array end
   into adjacent FOREIGN bytes. Detect: array span `BASE + count*stride` vs the next const's
   address (`whoowns.py` prints "next def at 0x.."). Fix: size the array to the REAL slot count
   (match the C array stride). Overlords are **8** slots not 16 → spawned_area
   (0xcca) spilled over SPRITE_BUMP_DAMAGE (0xcd2) and the work block over sprite_stunned
   (0xb58), f460431. (Earlier: star-switch table oversize.)
3. **Bounded native read where C reads raw RAM** — a fixed-size native model read where C
   indexes past its slots (mode-reuse beyond the model's range). Fix: read raw
   `ram[ADDR + i]` like C, not the bounded accessor. (Y-item multiselect scanned
   `inventory_item()` f255360; wishing-pond bottle clear f255446.)
4. **Missing/divergent write or branch** — the port omitted a write C performs, or
   a native-state-driven branch diverged. Fix: diff the fn against the C source; add the
   missing write. (Special-switch set both 0x410 AND 0x416, f241475.)

**Symptom → class cheat-sheet:**
- A byte is set CORRECTLY then reverts to its frame-start value later in the same frame →
  **class 1**. Find the re-stamper: `find_dual_ownership.py <addr>` for the co-owner; if its
  projection is a slice/loop (the finder's blind spot), read that state's `write_to_ram`
  directly. The re-stamp often fires from an unrelated setter's bridge `sync()` (e.g. a
  scroll-register write re-projecting a bundled palette/array field).
- Divergence at `BASE + k` where BASE is one array and the byte belongs to the NEXT
  system/const → **class 2** (oversized array; check the count vs the real stride).
- "Every persisted input matches at the frame boundary but the action diverges" → coherence
  gap (class 1/3). Run `ZELDA3_ASSERT_NATIVE_COHERENT` and trace the native read vs `ram[ADDR]`.

## Reference builds & ROM

**One reference, byte-exact: the C oracle `../zelda3`** (ground truth; override `ZELDA3_C_REPO`).
The C source build is byte-identical to the Rust port on WRAM, VRAM, SRAM, render-hash, and the
audio DSP trace (the old "cycle-accuracy" caveat was SNES9X-only; the C *source* build matches
exactly). Gate every layer with `scripts/validate_all_parity.py`; find per-frame divergences with
`zparity` (tool #10). Editing the C repo to add parity-oracle hooks IS permitted (audio-trace
default freq + `ZELDA3_REPLAY_WRAM_DUMP`/`ZELDA3_VRAM_DUMP`/`--fingerprint-log` dumps are committed
there). Build: `make -C ../zelda3 zelda3`. Invoke headless: `SDL_VIDEODRIVER=dummy
SDL_AUDIODRIVER=dummy SDL_RENDER_DRIVER=software ../zelda3/zelda3 --config
../zelda3/other/headless_replay.ini --replay-save <save> --smv-test-frames <N>`.

- ROM: `saves/zelda3.sfc` in THIS repo (gitignored via `*.sfc`); scripts default to it and
  accept `ZELDA3_ROM`. The C oracle uses its own `../zelda3/zelda3.sfc` via its config.
- **The all-layer gate:** `scripts/validate_all_parity.py [--frames N | --full]` compares
  WRAM (byte + hashes) / VRAM (byte) / SRAM / RENDER (per-frame) / AUDIO (DSP trace) vs the C
  oracle. Wired into `.githooks/pre-commit` (smoke budget; `--full` = exhaustive 170k).
- This repo: `cargo build --profile parity -p zelda3-bin` (deterministic like release,
  faster). Binary: `target/parity/zelda3 --replay-save saves/zelda3.sfc <save> <frames>`.
- Replay save: `saves/zelda3-combined-route.sav`. All replay runs need the timing hacks:
  `ZELDA3_SMV_{SELECT_FILE,LOADFILE,DUNGEON,OVERWORLD,MESSAGING,DEATH_INTRO,DEATH_RELOAD}_TIMING_HACKS=1`.
- Address semantics come from THIS repo's own const map: use `whoowns.py` (backed by
  `ram_ref.py`, which scans this repo's `const NAME: usize = 0xADDR;` definitions).

## Checkpoint resume — the ~250× per-probe speedup (USE THIS for any deep frame)

Every per-frame probe below re-replays from frame 0. For a divergence at frame ~460k that is
~90s per probe; with a checkpoint it is ~0.3s. The binary supports `--save-state`/`--load-state`
(and repeatable `--save-state-at <frame>:<path>`), and the snapshot includes the replay position
(`replay_pos`, `replay_next_cmd_at`), so you resume mid-replay exactly. (`zparity check` already
uses this internally to shard the route across cores.)

```bash
HACKS=(ZELDA3_SMV_SELECT_FILE_TIMING_HACKS=1 ... all 7 ...)   # see below
# Save once, a few thousand frames BEFORE the suspect frame:
env "${HACKS[@]}" target/parity/zelda3 --replay-save saves/zelda3.sfc saves/zelda3-combined-route.sav 460000 --save-state /tmp/ck_new_460000.sav
# Resume + dump/trace: pass the ABSOLUTE target frame and --load-state:
env "${HACKS[@]}" ZELDA3_REPLAY_WRAM_DUMP=/tmp/n.bin target/parity/zelda3 --replay-save ... 460431 --load-state /tmp/ck_new_460000.sav
```

Gotchas: (a) trace gates are checkpoint-aware via `trace_frame_matches` (RAM-watch, coherence,
step-dump match on `replay_frame_counter` too) — but `ZELDA3_WW_FRAME` still keys on
`frame_ctr_dbg` (RELATIVE on resume: target − checkpoint, e.g. 460431−460000 = 431). New
frame-gated `eprintln`s should use `self.trace_frame_matches(N)`, not `== frame_ctr_dbg`.
(b) snapshot restore has a tiny artifact (byte 0x654, HDMA scratch) — filter it, and confirm
a candidate fix with ONE from-scratch run before committing. (c) do not manually delete
checkpoint seeds as cleanup; `zparity check` owns `.cache/parity-golden/ck/manifest.json` and
regenerates needed checkpoints when the cache identity changes. If you change checkpoint
serialization, bump `CHECKPOINT_FORMAT_VERSION` in `crates/parity/src/checkpoint_cache.rs`.
See memory [[checkpoint-resume-debugging]].

## Parity-debugging tools (`scripts/`) — prefer these, in this order

0. **`ZELDA3_ASSERT_NATIVE_COHERENT` — the native↔RAM coherence checker (RUN THIS
   FIRST for any "inputs match but the action diverges" bug).** The dominant remaining
   bug class is a native sub-state model drifting out of sync with RAM mid-frame — a
   stale native field that re-projects over RAM, OR RAM written directly (a bridge that
   writes bytes it doesn't model, `clear_room_parser_words`, the cached-sprite uncache)
   leaving the native model stale so a later native *read* sees the wrong value. A WRAM
   dump diff CANNOT see this (both sides' RAM match at frame end; the divergence is a
   transient native≠RAM mismatch driving a different branch). `GameState::report_
   incoherent_with_ram` compares every sub-state to `load_from_ram(ram)`; it runs at the
   `replay_trace_ram_watch` step boundaries when `ZELDA3_ASSERT_NATIVE_COHERENT=1` (set
   `=panic` to abort on first drift; `ZELDA3_ASSERT_COHERENT_FRAME=<n>` to scope to one
   frame; `ZELDA3_ASSERT_COHERENT_IGNORE=a,b` to mute the gated-state baseline). Some
   states differ legitimately mid-frame (gated/mode-reuse projections) → baseline noise;
   a real bug is a *faithful* state (e.g. `sprites.sprite_slots`) going incoherent at the
   step that introduced it. Compare the suspect frame's list against an adjacent clean
   frame; the delta is the bug. **Diagnostic heuristic:** when every persisted input
   matches at the frame boundary but the action diverges, STOP tracing inputs — it's a
   coherence gap; run this, then trace the native read vs `ram[ADDR]` at the decision
   point. (`sprites` has leaf-level drill-down via `SpriteState::report_incoherent_with_
   ram`; add the same to other composites when needed.)

1. **`find_dual_ownership.py`** — STATIC finder for overlap bugs. The master
   `GameState::write_to_ram` projects every native state unconditionally in a fixed
   order (last-writer-wins), so two states writing the same byte mutually clobber.
   Lists all overlaps, mode-classified (CORE-vs-CORE = HIGH RISK; cross-mode =
   legitimate SNES reuse), an UNDERSIZED-TABLE lint (slice-write span vs the const-map
   array span), a ROOM-LOAD CLEAR COHERENCE audit, and a **BRIDGE WRITES IT DOESN'T
   MODEL** lint (a `*BridgeMut` method that writes a WRAM address owned by a native
   state it doesn't hold → that state's model goes stale unless resynced; the static
   form of the coherence bug above — it flags the cached-sprite uncache directly). Run
   after any migration edit; exit 1 = overlaps found (CI gate).
   *Gap:* its per-write parser misses runtime-length range writes
   `ram[BASE..BASE+self.vec.len()]` (tuple-array field-range tables ARE now resolved).

2. **`whoowns.py <addr>`** — address → this-repo const (+offset, span, mode-reuse
   aliases) + this repo's read/write sites + native owner struct (file:line). Collapses
   the root-cause grep chain; the fastest way to learn what an address *is* and who
   touches it. Pair it with `zparity drill <frame>` (tool #4): drill names the diverging
   WRAM page/address, whoowns names the variable. Shared address map in `ram_ref.py`
   (scans this repo's `const NAME: usize = 0xADDR;` definitions); `find_dual_ownership.py`'s
   undersized-table lint uses the same map.

3. **`ZELDA3_WW_ADDR=0x<addr> [ZELDA3_WW_FRAME=<n>]` — RAM write-watchpoint (function-level
   "who wrote this byte").** Pins the exact CALL SITE that writes a byte. Logs every
   write touching `<addr>` (optionally only on frame `<n>`, matched against `frame_ctr_dbg`)
   as `[WW] f=<frame> <descr> off=0x.. val=0x.. caller=<file:line>`. Centralized in
   `write_le_u16` (types.rs) via `#[track_caller]`; for direct `ram[x]=` / slice writers call
   `crate::types::ww_check(offset, len, descr, val)` (already wired into
   `write_expanded_graphics_tile_row`; add to other hot writers as needed, and mark the
   writer chain `#[track_caller]` so the caller propagates to the real trigger). Disabled =
   one atomic load + compare per write (negligible). Used to prove the 0x10000 gfx cluster
   is a graphics-load *timing* divergence, not a byte-ownership bug.

4. **`zparity` (`crates/parity`) — the per-frame C-oracle divergence finder (PRIMARY worklist tool).**
   Three subcommands: `capture --full` (needs C oracle) builds `parity-golden/` once and commits
   Tier A (`rollup.bin`/`merkle.bin`/`manifest.json`, ~4.3 MB); `check [--frames N | --full]` is
   **C-free**, shards the full route across cores via checkpoints, and finds the **earliest per-frame
   divergence vs C** — STRICTER than `validate_all_parity.py` (which is end-state only); `drill <frame>`
   localizes per page/layer (needs `capture --full --detail` for Tier B). Currently **RED by design**
   (first divergence: frame 3739, WRAM page 3, `RAW_SFX_PAN_VALUE`); use as a worklist:
   `check` → `drill <frame>` → `whoowns <addr>` → fix Rust side vs the C source → repeat. **Not wired
   as a blocking pre-commit gate.** Checkpoint seeds are cache-managed: `check` writes
   `.cache/parity-golden/ck/manifest.json` and reuses matching checkpoints; it reseeds only missing
   or manifest-incompatible boundaries. Measured: ~30k frames / ~69s across 4 shards. Full docs in
   `crates/parity/README.md`; spec/plan in `docs/superpowers/{specs,plans}/2026-06-20-zparity-*`.

### Tracing env vars

- `ZELDA3_REPLAY_WRAM_DUMP=<path>` — dump full 128KB WRAM at the final frame (this repo
  and the C oracle both support it).
- `ZELDA3_FINGERPRINT_LOG=<path>` (or `--fingerprint-log`) — append a fixed 788-byte
  per-frame all-layer fingerprint (this repo + C oracle, byte-identical); the stream
  `zparity` captures/checks.
- `ZELDA3_REPLAY_RAM_WATCH_FRAME=<f> ZELDA3_REPLAY_RAM_WATCH_ADDR=<addr>` — print the
  watched byte + module state at every labeled step within a frame (find WHICH step
  writes a value).
- `ZELDA3_REPLAY_RAM_DUMP_PAGE=0x<addr>` — dump nonzero bytes of a 0x400 page.
- `ZELDA3_REPLAY_RAM_PAGE_DUMP=1` — 128 per-1KB page checksums.

## The debugging loop that works

1. `zparity check [--full]` → the EARLIEST per-frame diverging frame vs C.
2. `zparity drill <frame>` → the diverging WRAM page/address (or VRAM/sram/render/audio leaf).
   (Needs Tier B: `zparity capture --full --detail` once.)
3. `whoowns.py <addr>` → this-repo const, read/write sites, native owner struct.
4. Classify into one of the four **Common bug classes** above: overlap/clobber
   (`find_dual_ownership.py`), oversized table (array span vs the next const), mode-reuse
   (same address, two states), bounded-read-vs-raw, or missing write/branch — by comparing
   the fn **against the C source** (`../zelda3/src`). Note: `zparity` finds divergences where
   the Rust port differs from C per-frame, so the C source is the reference (a sibling-Rust
   diff would show nothing).
5. Apply the matching **fix recipe** above (mode-gate / write-through / sole-owner / size to
   real slot count / read raw RAM). Make ONE struct the sole owner and redirect all
   readers/setters — do NOT just delete a duplicate field (the owners leapfrog across
   frames; verify).
6. Verify: `zparity check` (the diverging frame moved later / is gone), the specific bytes
   now match, `cargo test --profile parity -p zelda3 game_state` (280 pass), and
   `scripts/validate_all_parity.py` for the end-state all-layer gate vs C.

## Gotchas

- Never `git checkout <file>` — it nukes unstaged WIP. Surgically revert your own edits.
- Do not delete or recapture parity seeds as routine cleanup. `parity-golden/` is the committed
  C-oracle golden; re-capture it only when the replay route, C oracle hooks, fingerprint
  format/mask, ROM/save, or timing-hack contract changes. `.cache/parity-golden/detail/` is
  local drill detail and should stay unless the golden was recaptured. `.cache/parity-golden/ck/`
  is Rust checkpoint cache; `zparity check` invalidates it by manifest and regenerates the needed
  boundaries. If checkpoint serialization/layout changes, bump `CHECKPOINT_FORMAT_VERSION` in
  `crates/parity/src/checkpoint_cache.rs` instead of telling agents to delete the cache by hand.
  If cache corruption is suspected, run `scripts/verify_seeded_boundaries.sh` or remove only the
  affected checkpoint files with a concrete reason.
- Raw RAM hashes flag BENIGN scratch divergence (matches behavior, fails the gate).
  The gate (`test_standard_replay_parity.py`) compares raw `ramhash`/`ram0..7`/`sramhash`
  vs the C oracle, so even benign shadow divergence must eventually be eliminated.
- `check_ram_readability.py` no longer false-positives on length constants
  (`_LEN/_LENGTH/_CAPACITY/_SIZE`) or C-style names in `//` comments, so the pre-commit
  hook passes clean — `--no-verify` is no longer required for that reason. (The full hook
  now also runs `validate_all_parity.py`, which builds + replays, so it is slow; use
  `--no-verify` only to skip the heavy gate during rapid iteration.) macOS has no
  `timeout`/`gtimeout` — use a background pid + watchdog kill.

See `~/.claude/projects/.../memory/` for the running log of fixes and the current
front (the persistent memory index is loaded each session).
