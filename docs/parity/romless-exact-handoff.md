# Romless exact play: handoff

Read this first, then `docs/parity/romless-exact-play.md` for the program's
history and evidence, and `docs/parity/cycle-ledger-recipe.md` before
annotating any routine.

## What the program is

The engine reproduces the game exactly, but only while it is driven by
per-host timing receipts captured from Snes9x. Those receipts tell it how
much CPU work each host frame performed. The goal is an engine that
derives the same timing itself, so the game runs bit-exact with no ROM and
no receipts present. The user's constraint is explicit and has been tested
against twice: **do not embed the ROM**. The ROM may be read at
development and test time only. Its data-dependent constants may be
extracted into the asset pack; its control flow must be modelled in Rust.

## The two gates

**Acceptance gate (never regress this).** A full-route cached comparison
against the pinned Snes9x oracle cache: every per-frame video and audio
hash must match.

```sh
./parity cached-av .git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18 \
  --output <run-dir> --paired-checkpoint-interval 20000
```

This drives the receipt path. It is currently exact over all 1,581,079
frames and is promoted in `routes/full_run/parity-frontier.json`.

**Development gate (the thing being moved).** The same route with no
receipts installed, so the engine must derive its own timing. The first
mismatching frame is the frontier.

```sh
ZELDA3_CACHED_AV_NATIVE_TIMING=1 ./parity cached-av <cache> \
  --binary target/alt/parity/zelda3 --frames 200000
```

Build that binary with `CARGO_TARGET_DIR=target/alt` so a frontier run can
never rebuild the binary a comparison is using.

History of the frontier: 8889 → 4660 → 2507 → 8716 → 7330 → 2507 → 8890 → **11444**.
It moves backwards whenever a newly exact cost exposes a wrong one
downstream; that is normal and not a regression of the acceptance gate.

## Where it stands

| | |
|---|---|
| branch | completed batch merged into `main`; use a working branch for the next batch |
| promoted ledger | full route exact, four goldens, recorded endpoint |
| library suite | 1,736 passing under the parity profile, ~13 s |
| native frontier | frame 11444, audio exact throughout |

The scroll-return milestone is complete. The native lane carries the scroll's
remaining CPU work, finishes a fitting return after the held NMI, and marks
the existing scheduler's main-wait phase so the next leading NMI publishes
text before another scroll starts. The caller's obsolete addition of
refresh/HDMA stall time to CPU headroom was also removed. See the final
section of `romless-exact-play.md` for the original timestamps, regression
test, exact validation and the remaining coarse pixel-copy limitation.

## Rules that are not negotiable

- **Never push.** The user pushes.
- Never embed the ROM. It was implemented once and reverted on request.
- Never regenerate frozen test fixtures.
- Run GPU comparisons serially. The binary takes an exclusive lock; do not
  run cargo GPU tests beside one, and never rebuild `target/parity/zelda3`
  while a comparison is running.
- Subagents must not run `cached-av` or any GPU comparison. They annotate
  and verify against the recorded shadow profiles; the lead measures.
- Never `git checkout <file>`; revert your own edits surgically.

## The immediate next task: frame 11444

This is Module0F spotlight close, after Module09, in room `$55`. The first
visible mismatch is 11444; the CPU-state clue is one frame earlier:

- At 11440–11442, receipt/native WRAM differs only at `$12/$16/$1F00`.
- At 11443, the receipt lane's `SPOTLIGHT_WINDOW_Y_BUFFER` (`$067A`) is
  `$000C` versus native `$007E`, and 44 HDMA-table bytes differ.
- At 11444, WRAM is back to only `$12/$16/$1F00`, but video differs.
  Audio remains exact.

The 2026-09-11 investigation confirmed the original Module0F entry at
V=255/cycle 480: **do not retune the envelope**. Two shadow-model errors
were isolated: the dungeon checkpoint incorrectly rewinds the frame counter,
and both spotlight models count loops at `$F39B`, an operand byte, instead
of the `$F396` loop test. Correcting both reproduces the original table entry
and first NMI exactly, but does **not** repair the visible mismatch and
exposes an earlier native video mismatch at 4785. All experiments were
reverted; none is an accepted fix.

Continue at the CPU-table-to-display publication boundary. Compare the
shadow's first copy and consumed channel-7 rows with the original, then trace
`LiveSpotlightScanout` through `begin_dungeon_exit_spotlight_entry`,
`complete_dungeon_exit_spotlight_entry_before_link`, and the following-field
publication slots. At experimental frame 4785 the CPU WRAM matches the
receipt lane except `$1F00`, while video differs. Establish which presented
generation is wrong before changing a publication rule. The counter and
loop-count corrections need to be revisited together with that mechanism;
neither is independently sufficient. Do not add a frame/room exception.

See "Spotlight investigation: exact CPU entry still leaves a display mismatch"
in `romless-exact-play.md` for timestamps, rejected experiments and artifacts.
The `target/alt` binary is an **experimental** build after this investigation;
rebuild it from the current source before measuring a new baseline.

Local evidence retained for resumption:
`target/romless-8890-native-final` (native frontier and WRAM at 11443/11444),
`target/romless-11444-receipt-final` (same-binary receipt WRAM at 11440–11444), and
`target/romless-8890-original-timestamps` (cold original scroll trace and
compact `scroll-boundaries.jsonl`). The native and receipt final comparisons
use binary SHA-256 `75aaed1128771423b6a47fbb51e468a1adca4dbfeb0346e904f4f5ddb4d9042b`.

## How to diagnose a frontier frame

1. **Rust against Rust first.** Run the same binary with and without
   `ZELDA3_CACHED_AV_NATIVE_TIMING=1`, dumping `ZELDA3_DEBUG_WRAM_FRAMES`
   at the frontier and a few frames before, and compare the dumps. The
   receipt path is the ground truth. At the repaired 8889/8890 boundary only
   the known scratch byte remains; at 11443 this isolates the spotlight work
   buffer and table. `$1f00` differs benignly; so can `$12`.
2. **Then the subsystem's own trace.** For dialogue:
   `ZELDA3_DEBUG_SCROLL_STAGE=1 ZELDA3_DEBUG_SCROLL_RETAIN=1` in both
   modes gives the scroll phase machine's decisions side by side;
   `ZELDA3_DEBUG_VWF_BUDGET_FRAME=<host>` gives one frame of per-glyph
   receipts; `ZELDA3_DEBUG_VWF_BUDGET=1` gives all frames, short windows
   only.
3. **Then the original hardware.** Drive the instrumented Snes9x core
   directly; the recipe and its gotchas are in
   `docs/parity/romless-exact-play.md` under "Ground truth for the
   fresh-entry prefix". At most ten PC filters, the frame filter must
   start at 0, and `./parity microscope --cold` refuses this route, so
   call the comparison harness yourself with the `target/alt` binary.

Engine host N corresponds to Snes9x run N−1.

## Batch fixes before full-route validation

The user requested batching on 2026-09-11 because a full-route check takes
about 25 minutes. Aim for 3–5 tractable, independently explained frontier
fixes per batch, with one root cause per commit. Run the expensive acceptance
and promotion sequence once for the completed batch, rather than once per
fix. End a batch sooner if a regression cannot be isolated confidently.

For each fix, add reference-backed regression coverage, run the relevant
tests, and measure the native frontier again. Run a focused receipt-driven
cached A/V comparison through the affected window as well; a native frontier
improvement alone does not prove acceptance was preserved. Keep each fix
independently revertible, and do not commit unresolved experiments. When
committing these intermediate fixes, `ZELDA3_PRECOMMIT_SKIP_SNES9X=1` avoids
the legacy long gate; the hook's build and standalone smoke still run, and
the complete acceptance sequence below remains mandatory before merging.

Keep `main` at the last fully validated batch while work continues on the
working branch. Freeze the completed batch's binary and commit for its full
run; do not rebuild that binary during comparison. Preserve serial GPU runs
and use `target/alt` for independent development builds.

## Validating and promoting a completed batch

1. `cargo check -p zelda3`, then `cargo test -p zelda3 --lib --no-run`
   with zero warnings. That build is the dead-code detector.
2. `cargo test --profile parity -p zelda3 --lib`.
3. A 200,000-frame cached comparison with the WRAM goldens and the
   endpoint compare.
4. The full route, then
   `./parity promote --cached-av <run> --binary target/parity/zelda3`. The
   tree must be clean at the validated commit; if the branch has moved,
   promote from a detached worktree at that commit and copy
   `routes/full_run/parity-frontier.json` and the receipt manifest back.
5. Commit the evidence, move `main`, prune the run directories.

## The backlog after 11444

- March the frontier. Each divergence is now a single named mechanism.
- Finish the ledger census. The remaining classes are hosts where the
  profile records two engine iterations against one ledger host, the split
  between `Sprite_ExecuteSingle` and the inactive-sprite path, room-object
  drawers, and the interrupt handler's own attribution.
- Replace the eleven ROM-driven timing plans with native models. The
  dialogue initialization plan is closest: its cost is a fixed graphics
  decompression plus a message-dependent part, and the decompression,
  character-buffer and variable-width-font models are already exact. The
  room-load plan is hardest and needs a per-object cycle model.
- Only then does live play with no ROM become reachable; it currently
  starts from a different boot path with no receipts at all.
