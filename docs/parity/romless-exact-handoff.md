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

History of the frontier: 8889 → 4660 → 2507 → 8716 → 7330 → 2507 → 8890 → 11444 → 4785 → **11444** (completed spotlight batch).
It moves backwards whenever a newly exact cost exposes a wrong one
downstream; that is normal and not a regression of the acceptance gate.

The user explicitly reaffirmed this on 2026-09-11: an earlier native display
failure is acceptable when the change improves fidelity to the original.
Keep source-proven corrections with regression coverage even if they expose
an earlier native A/V frontier. Report CPU-model fidelity and native A/V
coverage separately. Receipt-driven acceptance must still pass; do not revert
a demonstrated timing correction solely to preserve the old native frame
number. This supersedes the overly conservative rejection in the spotlight
investigation below.

## Where it stands

| | |
|---|---|
| branch | locally merged to `main`; three source-backed spotlight fixes |
| promoted ledger | full route exact, four WRAM goldens and endpoint matched |
| library suite | 1,738 passing, 3 ignored; local-ROM row test also passed explicitly |
| native frontier | frame 11444, audio exact through that frontier |

Validated runtime commit: `1ad47d722fd844cedfbdc1e383e04a7655cbbbcb`.
Binary SHA-256: `4be88538777fd29181872a6e75f87b63889925fce5372cceb534b77312426f5d`.
The full-route receipt is
`routes/full_run/receipts/spotlight-batch-full.manifest.json`.

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

The completed batch retains three independently explained fixes:

- `66e59bee` preserves the already-incremented frame counter at Module0F.
- `90d68e8f` counts spotlight row pairs at executable `$F396`, rather than
  operand byte `$F39B`.
- `1ad47d72` keeps the last copied spotlight table on screen while the next
  build is unfinished. `$F383/$F392` modify working rows; the hardware table
  changes only at the later `$F3B7` copy. Exact scanline receipts retain
  priority, and live CPU working RAM is unchanged.

At the repaired frame 4785, native and receipt presentation already agreed
on VRAM, CGRAM, OAM and window controls. The native dynamic working table
had 25 wrong scanout rows, while all 224 reserved-table rows matched the
original. Publishing that completed table restores native frontier 11444
without undoing either CPU correction. This is improved source fidelity at
the same native A/V frontier, not fully receipt-free execution.

Next compare the final presented state at 11444. Both final native WRAM
tables (`$1DBA0` and `$17000`) match all 224 original window bounds there;
that does not establish which generation or window controls were displayed.
Original window layers are `[22, 1]`, with predicates `[3, 3, 3, 0, 3, 3]`.
Inspect the final snapshot and first-entry/copy publication boundaries before
changing a rule. The source supports the existing Module0F entry envelope:
**do not retune it**. A pending within-row decrement remains a separate CPU
continuation limitation. Do not add a frame/room exception.

See "Completed spotlight batch: publish the last copied table" in
`romless-exact-play.md` for the root cause, regression and acceptance evidence.
Rebuild `target/alt` from `main` before further runtime development.

Local evidence retained for resumption:
`target/spotlight-batch-native-final` contains the frontier and WRAM dumps;
`target/spotlight-batch-cold-av` contains the exact 11,500-frame live-Snes9x
A/V check (6,130,827 stereo sample frames, zero differences).
`target/spotlight-batch-validation` retains compact manifests, logs and
200k/full endpoint dumps after the large acceptance runs are pruned.
`target/spotlight-display-native`, `target/spotlight-display-receipt` and
`target/spotlight-display-source` retain the repaired display-generation
proof; `target/spotlight-next-source` retains original windows around 11444.
`target/romless-11444-original-timestamps` retains original source boundaries.

## How to diagnose a frontier frame

1. **Rust against Rust first.** Run the same binary with and without
   `ZELDA3_CACHED_AV_NATIVE_TIMING=1`, dumping `ZELDA3_DEBUG_WRAM_FRAMES`
   at the frontier and a few frames before, and compare the dumps. The
   receipt path is the ground truth. At the repaired 8889/8890 boundary only
   the known scratch byte remains. At repaired frame 4785, matching CPU tables
   narrowed the mismatch to display publication. At current frame 11444,
   compare the actual presented generation and controls as well as live WRAM.
   `$1f00` differs benignly; so can `$12`.
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

## The backlog after the spotlight frontier

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
