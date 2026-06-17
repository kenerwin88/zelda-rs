# zelda3-rs — working notes for Claude

This repo is migrating `game_state` from raw-byte WRAM access to a semantic/native
layer (domain-named Rust structs) while preserving **exact C parity** with the
reference emulator. Hard constraint: SNES WRAM reuses the same bytes for different
systems by game mode, so a semantic write must touch only the byte/word it owns —
never bulk-project a range it shares with another system.

## Reference builds & ROM

- C oracle: `../zelda3` (ROM `../zelda3/zelda3.sfc`). `variables.h` maps every
  `#define name (g_ram+0xADDR)` — the ground truth for who owns an address.
- Known-good Rust clone at commit `1183dee`: `~/Documents/zelda3-rs-old`
  (build: `ZELDA3_ROM=.../zelda3.sfc cargo build --release -p zelda3-bin`).
- This repo: `cargo build --profile parity -p zelda3-bin` (deterministic like
  release, faster). Binary: `target/parity/zelda3 --replay-save <rom> <save> <frames>`.
- Replay save: `saves/zelda3-combined-route.sav`. All replay runs need the timing
  hacks: `ZELDA3_SMV_{SELECT_FILE,LOADFILE,DUNGEON,OVERWORLD,MESSAGING,DEATH_INTRO,DEATH_RELOAD}_TIMING_HACKS=1`.

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
   legitimate SNES reuse), an UNDERSIZED-TABLE lint (slice-write span vs the OLD-clone
   array span), a ROOM-LOAD CLEAR COHERENCE audit, and a **BRIDGE WRITES IT DOESN'T
   MODEL** lint (a `*BridgeMut` method that writes a WRAM address owned by a native
   state it doesn't hold → that state's model goes stale unless resynced; the static
   form of the coherence bug above — it flags the cached-sprite uncache directly). Run
   after any migration edit; exit 1 = overlaps found (CI gate).
   *Gap:* its per-write parser misses runtime-length range writes
   `ram[BASE..BASE+self.vec.len()]` (tuple-array field-range tables ARE now resolved).

2. **`first_diverging_frame.py <addr> [--width N] [--max F] [--linear]`** — binary-
   searches the first frame a specific WRAM address diverges old-vs-new, comparing
   ONLY those bytes. Use this for "when did byte X first go wrong." It is immune to
   the non-monotonic raw-hash flicker that makes `old_new_parity.py`'s full-hash
   bisection land on arbitrary/wrong frames. Needs both binaries' `ZELDA3_REPLAY_WRAM_DUMP`.

3. **`whoowns.py <addr>`** — address → OLD-clone (`zelda3-rs-old`) const (+offset,
   span, mode-reuse aliases) + the OLD clone's parity-correct read/write sites +
   this-repo Rust constant + native owner struct (file:line). Collapses the
   root-cause grep chain; the fastest way to learn what an address *is*, who should
   own it, and how the known-good code uses it. References the OLD Rust clone, NOT
   the C source (the clone has perfect C parity). Shared map in `old_rust_ref.py`
   (override clone path with `ZELDA3_OLD_REPO`); `find_dual_ownership.py`'s
   undersized-table lint uses the same OLD-clone layout.

4. **`stable_page_diff.py <frames...>`** — deterministic per-1KB-page old-vs-new diff
   at fixed frames. The regression metric: a correct fix turns mismatching pages →
   matching and must NEVER flip a matching page to mismatching.

5. `old_new_parity.py` — the original sweeping harness. Use `--semantic-only` to find
   the first BEHAVIORAL divergence (ignores benign scratch-RAM shadow). Its default
   raw-hash bisection is UNRELIABLE (flicker) — don't trust its "first divergence"
   frame; confirm with `first_diverging_frame.py`.

### Tracing env vars

- `ZELDA3_REPLAY_WRAM_DUMP=<path>` — dump full 128KB WRAM at the final frame (both
  this repo and the `1183dee` clone support it; the clone's hook is a local debug aid).
- `ZELDA3_REPLAY_RAM_WATCH_FRAME=<f> ZELDA3_REPLAY_RAM_WATCH_ADDR=<addr>` — print the
  watched byte + module state at every labeled step within a frame (find WHICH step
  writes a value).
- `ZELDA3_REPLAY_RAM_DUMP_PAGE=0x<addr>` — dump nonzero bytes of a 0x400 page.
- `ZELDA3_REPLAY_RAM_PAGE_DUMP=1` — 128 per-1KB page checksums.

## The debugging loop that works

1. `old_new_parity.py --semantic-only` → first BEHAVIORAL divergence frame.
2. Pick a diverging semantic field's address → `first_diverging_frame.py <addr>` →
   exact first frame + old/new values.
3. `whoowns.py <addr>` → OLD-clone const, OLD-clone read/write sites, native owner.
4. Classify: overlap/clobber (`find_dual_ownership.py`), undersized/oversized table
   (C span vs native span), mode-reuse (same address, two `#define`s / two states), or
   logic divergence (compare the Rust port line-by-line against the C site).
5. Fix so the semantic write owns ONLY its byte (or gate mode-reused projections on
   the active mode, e.g. `PLAYER_IS_INDOORS`). Make ONE struct the sole owner and
   redirect all readers/setters — do NOT just delete a duplicate field (the owners
   leapfrog across frames; verify).
6. Verify: `stable_page_diff.py` (no regression), the specific bytes now match,
   `cargo test --profile parity -p zelda3 game_state` (280 pass), and re-run
   `--semantic-only` to confirm the behavioral high-water moved.

## Gotchas

- Never `git checkout <file>` — it nukes unstaged WIP. Surgically revert your own edits.
- After changing any serde-serialized native struct's field layout, DELETE
  `.cache/old-new-parity/ckpt` — old `--load-state` checkpoints become incompatible
  and silently corrupt runs.
- Raw RAM hashes flag BENIGN scratch divergence (matches behavior, fails the gate).
  The gate (`test_standard_replay_parity.py`) compares raw `ramhash`/`ram0..7`/`sramhash`
  vs the C oracle, so even benign shadow divergence must eventually be eliminated.
- A pre-existing guardrail flags `messaging.rs:8` (a `0x1000` length const the
  `check_ram_readability.py` pre-commit hook mistakes for an address); commits use
  `--no-verify` until that's addressed separately. macOS has no `timeout`/`gtimeout`
  — use a background pid + watchdog kill.

See `~/.claude/projects/.../memory/` for the running log of fixes and the current
front (the persistent memory index is loaded each session).
