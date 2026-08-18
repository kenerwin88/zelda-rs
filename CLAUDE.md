# zelda3-rs — working notes for Claude

This repo is migrating `game_state` from raw-byte WRAM access to a semantic/native
layer (domain-named Rust structs) while preserving the game's exact behavior.
Hard constraint: SNES WRAM reuses the same bytes for different systems by game
mode, so a semantic write must touch only the byte/word it owns — never
bulk-project a range it shares with another system.

## Parity target: fully-modern runtime vs Snes9x

The legacy parity apparatus (zparity capture/check/drill, parity-golden/,
`--fingerprint-log`, validate_all_parity.py, the classic CPU/wgpu renderers, the
legacy SPC/DSP audio oracle, AND the old 1,073,092-frame
`<replay.sav>` replay route) has been **fully removed**. The route was recorded
against timing hacks that do not progress in Snes9x — do not resurrect it for
parity.

**Parity lives in `routes/` now**: human routes recorded directly in the pinned
Snes9x 1.63 libretro core (`routes/clean` is the good lineage). Each project
holds Snes9x-native boundary states (savestate + WRAM/VRAM/SRAM + screenshot)
plus per-take compact input streams. Drive it with
`scripts/snes9x_route_recorder.py` (record / pair / compare / compare-all /
compare-route); `compare-route` replays the continuous human route through the
`--compare-snes9x-oracle` harness with exact video+audio comparison.
`scripts/full_parity.py --with-snes9x` still provides the 180-frame cold-boot
live A/V gate. **GPU comparisons must run serially** — concurrent offscreen GPU
runs produce nondeterministic render flakes and stomp the shared comparison
session directory. The zelda3 binary itself enforces this: every
`--compare-snes9x-oracle` / `--record-snes9x-route` run takes an exclusive
flock on `/tmp/zelda3-snes9x-compare.lock` and refuses to start a second
session (do not run cargo GPU tests alongside a comparison either).
`target/parity-failures/` is auto-pruned to the newest 20 run dirs.
The lockstep oracle was removed 2026-07-18 (snes9x is the only oracle).
Internal regression tools that remain:
`ZELDA3_ASSERT_NATIVE_COHERENT`, `find_dual_ownership.py`, `whoowns.py`,
RAM write-watchpoints, and Rust-vs-Rust WRAM dumps
(`ZELDA3_REPLAY_WRAM_DUMP` across two builds of this repo).
Do NOT consult or reference the old parity baseline tooling; this repo's own code
plus the
Snes9x oracle are the only references.
`zparity` provides route coverage plus the typed Rust evidence plane used by
`./parity trace-index`, `./parity trace-query`, and `./parity cache-verify`.
It does not replace the pinned live Snes9x A/V authority.

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
4. **Missing/divergent write or branch** — the port omitted a write the original game
   performs, or a native-state-driven branch diverged. Fix: trace the divergent WRAM
   write with the RAM watchpoints and add the missing write. (Special-switch set both
   0x410 AND 0x416, f241475.)
5. **Collapsed timed side-effect phase** — a long CPU operation has the right total cost,
   so video publication matches, but an observable entry/midpoint side effect is assigned
   to the operation's completion boundary. Audio commands and register writes can then
   cross NMI one frame late without an immediate video mismatch. Fix: use the Snes9x
   `pc,wram` trace domains to measure the side-effect PC independently, split the timing
   state at that point, and test that resuming the post-effect phase does not emit it
   twice. (VWF `$0E:CACC` dialogue click vs remaining glyph setup, f20623.)
   When two resumed slices appear to require contradictory caller-return thresholds, do
   not classify them only as "resumed": preserve the suspended CPU phase (`Entering`,
   `PreparingDrawing`, or `Drawing`) and calibrate the return from that phase. Use
   `ZELDA3_DEBUG_VWF_BUDGET_FRAME=<host-frame>` for one frame of per-glyph phase/cycle
   receipts, then pair it with Snes9x `pc,nmi` events at `$0E:C9F9` and `$00:F861`.
   The unfiltered `ZELDA3_DEBUG_VWF_BUDGET=1` remains available for short windows only.
   Identical long calls can also cross different totals when the preceding call returns at a
   different raster phase. Compare the entry and return PCs with their V-counters before
   attributing the difference to data size; keep both a shorter control case and the longer
   case in tests. (`LoadTransAuxGFX_sprite`: room `$22` enters after V=27 and returns after
   seven slices at V=158; room `$21` enters after V=124 and returns on the eighth boundary at
   V=257, f31243.)
6. **Aliased hardware phase lanes** — a compact modulo formula appears to describe a wrapped
   pipeline but aliases consumers that remain in the unwrapped part of the callback's
   phase schedule. The error stays latent until a register write lands between the real and
   aliased latch phases, then creates a tiny persistent cursor drift. Fix: compare the oracle
   register receipt and interpolation cursor, bracket the event-clock phase with writes on
   both sides **and at equality**, name/table-test that mapping, and retain each threshold
   event as a behavioral regression. When identical event-clock coordinates disagree across
   a song-bank upload, compare the raw S-DSP phase and preserve the upload epoch explicitly;
   do not force one scalar threshold across both epochs. Recheck the route from cold boot after
   changing a phase lane; a checkpoint can begin after the first bad latch. Expand the mapping only
   for oracle-proven lanes so an attractive global rewrite cannot regress earlier exact audio.
   (S-DSP voice-0/1/3/4 V2 pitch reads, f20782-f20792.)

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
- Video remains exact while a command/register edge moves by one NMI → **class 5**. Trace
  the producer PC and target WRAM write in the same oracle run before changing the consumer.
- Audio is exact until a pitch/register write, then differs only when interpolation crosses
  a fractional bucket → **class 6**. Compare oracle DSP phase/register receipts against the
  Rust cursor; test before, equality, and after for the affected lane plus the complete
  hardware phase table, then cold-start the route rather than relying only on a checkpoint.

## Reference builds & ROM

**The legacy oracle is retired** — do not reference the old parity source
references at all. The external
parity reference is the **Snes9x libretro core** (`--compare-snes9x-oracle`,
`scripts/full_parity.py --with-snes9x`), and this repo's own code is the behavioral
source of truth.

- ROM: `saves/zelda3.sfc` in THIS repo (gitignored via `*.sfc`); scripts default to it and
  accept `ZELDA3_ROM`.
- This repo: `cargo build --profile parity -p zelda3-bin` (deterministic like release,
  faster). Binary: `target/parity/zelda3 --replay-save saves/zelda3.sfc <save> <frames>`.
- The old combined-route replay save was removed with legacy parity cleanup. `--replay-save` still
  works for ad-hoc `.sav` replays (legacy `ZELDA3_SMV_*_TIMING_HACKS=1` env applies to
  such saves), but recorded parity routes live in `routes/` as Snes9x-native
  boundary states + input takes.
- Address semantics come from THIS repo's own const map: use `whoowns.py` (backed by
  `ram_ref.py`, which scans this repo's `const NAME: usize = 0xADDR;` definitions).
- Regression baseline for refactors: run the route on the pre-change and post-change
  binaries with `ZELDA3_REPLAY_WRAM_DUMP` and byte-compare the dumps (Rust-vs-Rust).

## Checkpoint resume — the ~250× per-probe speedup (USE THIS for any deep frame)

Every per-frame probe below re-replays from frame 0. For a divergence at frame ~460k that is
~90s per probe; with a checkpoint it is ~0.3s. The binary supports `--save-state`/`--load-state`
(and repeatable `--save-state-at <frame>:<path>`), and the snapshot includes the replay position
(`replay_pos`, `replay_next_cmd_at`), so you resume mid-replay exactly.

```bash
HACKS=(ZELDA3_SMV_SELECT_FILE_TIMING_HACKS=1 ... all 7 ...)   # see below
# Save once, a few thousand frames BEFORE the suspect frame:
env "${HACKS[@]}" target/parity/zelda3 --replay-save saves/zelda3.sfc <replay.sav> 460000 --save-state /tmp/ck_new_460000.sav
# Resume + dump/trace: pass the ABSOLUTE target frame and --load-state:
env "${HACKS[@]}" ZELDA3_REPLAY_WRAM_DUMP=/tmp/n.bin target/parity/zelda3 --replay-save ... 460431 --load-state /tmp/ck_new_460000.sav
```

Gotchas: (a) trace gates are checkpoint-aware via `trace_frame_matches` (RAM-watch, coherence,
step-dump match on `replay_frame_counter` too) — but `ZELDA3_WW_FRAME` still keys on
`frame_ctr_dbg` (RELATIVE on resume: target − checkpoint, e.g. 460431−460000 = 431). New
frame-gated `eprintln`s should use `self.trace_frame_matches(N)`, not `== frame_ctr_dbg`.
(b) snapshot restore has a tiny artifact (byte 0x654, HDMA scratch) — filter it, and confirm
a candidate fix with ONE from-scratch run before committing.
See memory [[checkpoint-resume-debugging]].

## Parity-debugging tools (`scripts/`) — prefer these, in this order

Before the numbered root-cause tools, run `./parity status` plus
`./parity doctor` and use
`./parity microscope --frontier <frame> ...` for the focused evidence window.
The microscope pins the replay bundle and binary, expands both LoROM PC mirrors,
maps route frames only through zero-based `retro_run`, symbolicates against the
local C checkout for development evidence, and caches oracle-only artifacts by
content hash. It refuses unfiltered PC/WRAM traces. See
`docs/parity/frontier-microscope.md`. A microscope result is diagnostic; only
two cold exact A/V receipts followed by `./parity promote` can update the
commit-bound `routes/full_run/parity-frontier.json` ledger.
Checkpointed traces default to only the last 12 internal frames, and the
automatic CPU-checkpoint join maps Rust's absolute host frame through the
session manifest before comparing it to Snes9x `retro_run`. `./parity inspect
<session>` rejects legacy resumed traces that mislabeled the absolute frame as
a window-relative run. Completed microscope runs also build a hash-bound Rust
seek index when `target/parity/zparity` is available. Use `./parity trace-query
<session> --host-frame N [--pc PC] [--wram RANGE]` to return exact source JSONL
records without reparsing the whole trace; the query rejects changed trace or
manifest bytes. `./parity cache-verify` independently verifies the immutable
oracle cache in Rust. New sessions also write a complete canonical A/V hash
ledger for every compared frame. `./parity av-compare <session>` checks its Rust
RGB/audio hashes against the oracle-only cached ledger; `./parity
receipt-compare <session>` recursively checks the sampled semantic receipts.
These are fast regression/diagnostic tiers. They never promote the frontier;
two cold pinned-core exact A/V receipts are still required. `./parity cached-av
<cache>` is the no-Snes9x iteration path: it verifies a cold contiguous cache,
replays Rust from its bound input, RNG, SRAM, and audio schedule, and stops at
the first canonical A/V hash mismatch. It intentionally rejects resumed caches
until a Rust starting checkpoint is independently bound to that cache.
Create a complete cache with `./parity oracle-av-capture <source-session>
--frames N`. It runs only pinned Snes9x, binds input, recorded RNG, SRAM, core,
and ROM provenance, and explicitly records `oracle_captured` rather than a
parity pass. A Rust regression therefore cannot truncate the reusable oracle.

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
   touches it. Shared address map in `ram_ref.py`
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

4. **`zparity` (`crates/parity`)** — typed, streaming parity evidence tools.
  `coverage` produces route-coverage reports/worklists; `trace-index` and
  `trace-query` provide provenance-bound random access to original trace JSONL;
  `cache-verify` validates cache identities and every artifact hash. (The former
  capture/check/drill parity subcommands were retired with the legacy pipeline.)

5. **`parity_probe.py --around <frame>`** — one-command repro of a Snes9x divergence
  window. Reuses a coverage-sufficient cold pre-commit `run-*/` input by default;
  `--use-checkpoint` opts into a faster, explicitly diagnostic-only paired resume.
  With `--capture`, it writes/summarizes `display_oracle.jsonl` (which display
  domain diverges, decoded OAM slots) using the receipt-verified instrumented core.
  **Verify the replayed position before trusting a capture:** at f28837 the default
  run-dir/input selection replayed a route that was in module `09` room `$55` at
  the target frame while the gate's authoritative route was module `07` room `$42`,
  making the whole capture (cgram 112/256, presented_oam 383/544) meaningless. Check
  the reported `entry=`/`room=` context against the failing gate run's
  `main=/sub=/room=`, and pass `--run-dir routes/<project>/comparisons/precommit/
  run-<frontier>-video-preflight` explicitly to pin the same inputs.

6. **`vram_write_attribution.py <failure-dir>` (or `--latest`) — RUN THIS FIRST on
  any video divergence.** Reads only the artifacts a failing run already wrote
  (no replay) and answers the question a byte diff cannot: *which side ran a
  transfer the other did not, and with which operand generation?* It compares
  three write sets — `O` = words the ORACLE changed this frame
  (`oracle_before_vram` vs `oracle_after_vram`), `L` = rust live vs oracle,
  `P` = rust presented vs oracle — and classifies:
  `L` ⊆ `O` → both wrote the same words with different data (operand *values* or
  source buffer differ); `L` ∩ `O` = ∅ → **every word the oracle wrote matches**,
  so the engine is fine and this is a DMA scheduling/operand-generation bug;
  live-only words → an early upload held back in the presented generation, not
  the visible bug. For the Link OBJ block (`$4000-$43ff`) it then resolves each
  divergent destination against the `LinkDmaSourceSlot` table (scraped from the
  Rust sources so it cannot drift) and checks both sides' VRAM against
  `ram[pointer..+len]` at the frame's **entry** and **exit** pointer, printing
  which generation each emulator used — the direct input to
  `rom_graphics_dma_plan`'s `link_obj_operands`. It then diffs **OAM** — locating
  Snes9x's OAM inside `oracle_after.state`'s PPU chunk by best-window match
  against `rust_visible_oam.bin` (it warns when the winner is not clearly clear
  of the runner-up) and printing per-slot `x/y/tile/attr` deltas; small position
  deltas across several slots mean an `oam_operands`/`oam_scanout` generation bug
  rather than wrong sprite data. **This is the only way to diff the oracle's OAM
  without the instrumented core.** Finally it prints the newly-divergent WRAM set
  (after-set minus before-set), which separates "this frame's bug" from standing
  benign noise. Named the f28837 mechanism
  (spiral-stair `link_obj_operands`) end-to-end in one run.
  **Do NOT trust `diff.json`'s `trace_mine`/`trace_theirs` `sub=`/`subsub=` for
  the module a display bug belongs to** — it is a different readout from the
  console divergence line and disagreed with the operand decision at f28837
  (`trace_mine` said `7/$14/1`, the actual decision frames were entry `7/$0e/$00`
  → exit `7/$0e/$01`). Get the phase from `ZELDA3_DEBUG_LINK_DMA` (below), which
  prints the entry AND exit module/submodule/subsubmodule the plan keys on.

### Tracing env vars

- `ZELDA3_REPLAY_WRAM_DUMP=<path>` — dump full 128KB WRAM at the final frame (the
  Rust-vs-Rust regression baseline for refactors).
- `ZELDA3_REPLAY_RAM_WATCH_FRAME=<f> ZELDA3_REPLAY_RAM_WATCH_ADDR=<addr>` — print the
  watched byte + module state at every labeled step within a frame (find WHICH step
  writes a value).
- `ZELDA3_REPLAY_RAM_DUMP_PAGE=0x<addr>` — dump nonzero bytes of a 0x400 page.
- `ZELDA3_REPLAY_RAM_PAGE_DUMP=1` — 128 per-1KB page checksums.
- `ZELDA3_DEBUG_PREP=1` — log every `nmi_prepare_sprites` call (`[PREP]
  fc_dbg=<host frame> fc=<0x1a> cd=<0xc00d> mod=<module/sub>`); add
  `ZELDA3_DEBUG_PREP_BT=<fc_dbg>` for a backtrace of the call on that frame.
  The core-update/prep cadence drives BG_TILE_ANIMATION_COUNTDOWN and
  link-DMA phase; use this to compare rust's prep schedule against the
  oracle's countdown trajectory when an animation phase drifts by one.
- `ZELDA3_AUDIT_OAM_LAW=1` — per-frame differ between the presented OAM table
  and the trace-verified hardware OAM law (a $2104 transfer runs at each
  vblank whose main completed, carries the shadow as of that vblank, and is
  visible from the following scanout; leading NMIs are visible immediately).
  Prints `oam_law_delta host=<n> words=<k> ... source=<rule>` for every frame
  where a publication rule deviates from the law — the migration map for
  retiring per-window OAM rules. `ZELDA3_OAM_LAW_PRESENTATION=1` goes further
  and PRESENTS the law lane's OAM table (diagnostic only): video-compare that
  build against the oracle to see where the law itself is incomplete.
- `ZELDA3_DEBUG_LINK_DMA=1` — one line per NMI Link OBJ DMA with the **entry and
  exit** `main/sub/subsub` the operand rules key on, the resolved
  `operands=HostBoundaryBeforeMain|LiveAfterMain`, `captured=<bool>` (false means
  the DMA silently fell back to live sources), and both the captured and live
  `BodyPointerUpper`/`HeadTop` source words. This is the authoritative readout of
  `link_obj_operands_across_main`'s decision; renderless replays reproduce it
  faithfully, so pair it with `--ignore-video --ignore-audio` for a ~50s probe.
  Remember `host = comparator frame + 1`.

## The debugging loop that works

1. Reproduce the regression as a Rust-vs-Rust divergence: run the route on a known-good
   build and the current build with `ZELDA3_REPLAY_WRAM_DUMP` (bisect the frame with
   checkpoints) → the diverging WRAM address.
2. `whoowns.py <addr>` → this-repo const, read/write sites, native owner struct.
3. Classify into one of the four **Common bug classes** above: overlap/clobber
   (`find_dual_ownership.py`), oversized table (array span vs the next const), mode-reuse
   (same address, two states), bounded-read-vs-raw, or missing write/branch.
4. Apply the matching **fix recipe** above (mode-gate / write-through / sole-owner / size to
   real slot count / read raw RAM). Make ONE struct the sole owner and redirect all
   readers/setters — do NOT just delete a duplicate field (the owners leapfrog across
   frames; verify).
5. Verify: the WRAM dumps match again, `cargo test --profile parity -p zelda3 game_state`,
   and for audio/video behavior the Snes9x gates (`scripts/full_parity.py --with-snes9x`).

## Gotchas

- Never `git checkout <file>` — it nukes unstaged WIP. Surgically revert your own edits.
- `check_ram_readability.py` no longer false-positives on length constants
  (`_LEN/_LENGTH/_CAPACITY/_SIZE`) or C-style names in `//` comments, so the pre-commit
  hook passes clean. macOS has no `timeout`/`gtimeout` — use a background pid +
  watchdog kill.

See `~/.claude/projects/.../memory/` for the running log of fixes and the current
front (the persistent memory index is loaded each session).
