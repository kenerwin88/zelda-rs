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

History of the frontier: 8889 → 4660 → 2507 → 8716 → 7330 → 2507 → 8890 → 11444 → 4785 → 11444 → 20257 → 20262 → 23203 → 23206 → 23935 → **23945** (completed quadrant/audio batch).
It moves backwards whenever a newly exact cost exposes a wrong one
downstream; that is normal and not a regression of the acceptance gate.

The user explicitly reaffirmed this on 2026-09-11: an earlier native display
failure is acceptable when the change improves fidelity to the original.
Keep source-proven corrections with regression coverage even if they expose
an earlier native A/V frontier. Report CPU-model fidelity and native A/V
coverage separately. Receipt-driven acceptance must still pass; do not revert
a demonstrated timing correction solely to preserve the old native frame
number. This supersedes the overly conservative rejection in the spotlight
investigation recorded in `romless-exact-play.md`.

## Where it stands

| | |
|---|---|
| branch | locally merged to `main`; three source-backed quadrant/audio fixes |
| promoted ledger | full route exact, four WRAM goldens and endpoint matched |
| library suite | 1,743 passing, 3 ignored; dev builds have zero warnings |
| native frontier | frame 23945, audio exact through that frontier |

Validated runtime commit: `e70152ae55c787ed3c53a86014dc805d604cd097`.
Binary SHA-256: `1245aeb75f851063dcb0610b06789f2efefb48f39e18c086b5a6005c199e6814`.
The full-route receipt is
`routes/full_run/receipts/quadrant-batch-full.manifest.json`.

The scroll-return milestone is complete. The native lane carries the scroll's
remaining CPU work, finishes a fitting return after the held NMI, and marks
the existing scheduler's main-wait phase so the next leading NMI publishes
text before another scroll starts. The caller's obsolete addition of
refresh/HDMA stall time to CPU headroom was also removed. See the scroll-return
history in `romless-exact-play.md` for the original timestamps, regression
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

## The immediate next task: frame 23945

The completed batch fixes cached sprite-conversion retirement (`6704a050`),
dungeon NMI_PrepareSprites main-wait retirement (`63443ece`), and an obsolete
room-specific spiral audio sample (`e70152ae`). The native frontier advances
23203 → 23206 → 23935 → **23945**, now video-only. The 24,100-frame cold
live-Snes9x check, 200k/full receipt gates, WRAM goldens and endpoints all
pass. See "Completed quadrant-return and spiral-audio batch" in
`romless-exact-play.md` for source contracts and regressions.

At 23940-23944 native and receipt WRAM differ only at known scratch $1f00.
At 23945 both modes remain in 07/0e/0f. Native/receipt differences are
$12=01/00, $15=00/02, $16=00/01, $19=00/58 and scratch $1f00=00/01.
Original host 23945 completes its carried handler, Sprite_Main and common
suffix, then accepts an Open NMI. Investigate publication at that trailing
acceptance after the grayscale palette caller returns. This is a starting
diagnosis, not a proven cause; do not add another frame/room exception.

Use `ZELDA3_DEBUG_PRESENTED_FRAMES=23946` and
`ZELDA3_DEBUG_PRESENTED_DIR=<dir>` to capture fully composed display VRAM,
OBJ VRAM, CGRAM, OAM, presentation RAM and registers. These dumps describe
the renderer's selected generations; use WRAM dumps separately for live CPU
state. See CLAUDE.md for the reusable diagnostic added in `0a99f7f9`.

Evidence: `target/quadrant-batch-validation` (compact proofs and
`next-frontier.md`), `target/quadrant-batch-native-final`,
`target/quadrant-batch-cold-av`, and `target/quadrant-batch-next-source`.
Earlier quadrant/spiral probes remain under `target/quadrant-*`. Rebuild
`target/alt` from `main` before beginning the next batch. The earlier
Module0F entry envelope and pending within-row CPU decrement remain
separate source-backed work; do not retune them to move this frontier.

## How to diagnose a frontier frame

1. **Rust against Rust first.** Run the same binary with and without
   `ZELDA3_CACHED_AV_NATIVE_TIMING=1`, dumping `ZELDA3_DEBUG_WRAM_FRAMES`
   at the frontier and a few frames before, and compare the dumps. The
   receipt path is the ground truth. At the repaired 8889/8890 boundary only
   the known scratch byte remains. At repaired frame 4785, matching CPU tables
   narrowed the mismatch to display publication. At current frame 23945,
   trace the trailing Open acceptance and compare its composed palette and
   display generations; live module state agrees but four control bytes
   differ. `$1f00` differs benignly; interpret `$12` against the actual NMI
   boundary rather than dismissing it as scratch.
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
fix. End a batch sooner if an acceptance regression cannot be isolated
confidently. An explained backward move of the native frontier is not itself
a reason to reject a source-proven correction or end the batch.

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

## The backlog after the current native frontier

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
