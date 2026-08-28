# zparity — typed parity evidence tools for zelda3-rs

`zparity` is the Rust data plane beneath the repository's `./parity` Python
workflow. It provides streaming, typed operations for route coverage, trace
indexing/querying, semantic receipt comparison, canonical A/V hash comparison,
and immutable oracle-cache verification. It never provides game-runtime data
and does not replace the pinned live Snes9x A/V gate.

## Evidence subcommands

```sh
zparity trace-index TRACE --manifest MANIFEST --output TRACE.zpti
zparity trace-query TRACE.zpti [--host-frame N] [--run N] \
  [--internal-frame N] [--pc BB:AAAA] [--wram FIRST-LAST] \
  [--event EVENT] [--limit N]
zparity cache-verify CACHE_ROOT [--json]
zparity receipt-compare CANDIDATE_JSONL ORACLE_JSONL [--json]
zparity av-compare CANDIDATE_JSONL ORACLE_JSONL [--json]
```

The trace index is a compact seek table. It records typed filter fields and
byte offsets, then queries emit the exact original JSONL records. Both the
source trace and comparison manifest are SHA-256 verified before every query.
Cache verification recomputes the content-addressed identity and hashes every
listed artifact.

`receipt-compare` recursively localizes differences in the sampled semantic
receipts. Its report states whether recorded frames are contiguous; it must not
be interpreted as full-route coverage. `av-compare` checks the complete
per-compared-frame ledger of canonical visible RGB and exact interleaved i16
audio SHA-256 values. Hashes avoid retaining enormous raw frame streams while
remaining byte-exact for pass/fail. A cached match is a fast regression check,
not promotion authority; the cold pinned-core gate remains authoritative.

The repository wrapper adds `./parity cached-av CACHE`. It verifies a cold,
contiguous cache and replays the Rust engine only, using the cached input, RNG,
SRAM, and per-frame oracle audio schedule. Snes9x is not loaded. The resulting
candidate ledger is passed to `zparity av-compare`; this is the normal fast
iteration tier after one pinned-core cache capture.

Create that reference independently of the current Rust implementation with
`./parity oracle-av-capture SOURCE_SESSION --frames N`. The producer runs only
Snes9x, marks the session as oracle capture rather than parity proof, and then
places the oracle-only ledger in the immutable cache. This prevents a Rust
mismatch from truncating the reusable reference.

Set `ZELDA3_SNES9X_TIMING=1` on a live-oracle comparison or `cached-av` replay
to print an opt-in stage profile. The report separates engine execution,
source extraction, GPU submission, surface presentation, asynchronous readback,
video hashing, audio hashing, and serialization. It is diagnostic only, is off
by default, and does not weaken the cold live-Snes9x promotion gate.

## Retired workflow (historical)

The legacy `capture`, `check`, and `drill` subcommands below were retired with
the old golden/replay pipeline. This text remains only as historical context:

```
capture (route replay, once)  →  golden committed to parity-golden/
check   (C-free, sharded) →  DIVERGE at frame N  or  MATCH
drill   (C-free)          →  per-page/layer localization for frame N
coverage (Rust route)     →  route surface hit/miss report
```

## Subcommands

### `zparity capture [--full] [--detail]`

Runs the canonical route replay once and writes the two-tier golden:

- **Tier A** (`parity-golden/rollup.bin`, `merkle.bin`, `manifest.json`) — per-block
  rollup fingerprints committed to the repo. Small (~4.3 MB for the full route).
- **Tier B** (`.cache/parity-golden/detail/`) — per-frame per-region fingerprints,
  written only with `--detail`. Large (several GB). Cached locally, not committed.
  Needed by `drill`.

`--full` runs the full ~1,073,092-frame route (default: 30,000 frames).
`--detail` also writes Tier B to `.cache/parity-golden/detail/` (gitignored).

Re-capture when: the replay route changes, fixture/harness behavior changes, or
the fingerprint mask changes.

### `zparity check [--full] [--frames N] [--frame N]`

**C-free.** Shards the route across CPU cores using checkpoints seeded from the
Rust binary. Compares per-block fingerprints against the committed Tier A golden
(`parity-golden/rollup.bin`). Checkpoint seeds are cache-managed under
`.cache/parity-golden/ck/`: `check` reuses checkpoints whose `manifest.json`
matches the current Rust binary, ROM, replay save, timing-hack set, checkpoint
format, and seed command version. It regenerates only missing or manifest-
incompatible boundaries; do not delete the cache manually as routine cleanup.

Output:
- `DIVERGE at frame N, page P (WRAM/VRAM/SRAM/RENDER)  rust=0x...  golden=0x...` — first
  per-frame divergence found.
- `MATCH  K frames, S shards, Ts  root=0x...` — all frames match the golden.

**`check` is a STRICT per-frame route parity gate.** A green full-route check
means the Rust port matches the committed golden for every fingerprinted frame in
the current replay route. A failure remains the worklist loop: run `check` to
find the earliest divergence, run `drill <frame>` to localize it, fix the Rust
side, repeat.

Measured speed: ~30,000 frames in ~69 seconds across 4 shards (incl. checkpoint seeding).
Full-route check time is proportionally longer.

Use `--frame N` to compare one golden record index against Tier A without
replaying the full route. It uses the nearest compatible checkpoint already in
`.cache/parity-golden/ck/` when available; otherwise it runs from frame 0. This
is the fastest loop for confirming a suspected miss after `check` or `drill`
has identified a frame.

```bash
./target/debug/zparity check --frame 460431
```

### `zparity drill <frame>`

**C-free.** Localizes the divergence at frame `<frame>` per page and per layer
(WRAM/VRAM/SRAM/RENDER). Requires Tier B detail (`capture --full --detail`).

Output: table of diverging pages with golden vs rust fingerprints.

### `zparity coverage [--full] [--frames N] [--json PATH] [--input-script PATH] [--input-script-overlay PATH] [--load-state PATH] [--load-sram PATH] [--stop-replay-after-load] [--from-json PATH...] [--seed-from-c-assets] [--route-probes-from-worklist PATH] [--report-json PATH] [--route-report-json PATH] [--route-worklist-json PATH] [--diff-from-json PATH] [--delta-report-json PATH] [--require-full] [--require-route-full]`

Runs the Rust replay route and records which broad gameplay surfaces the route
actually touches: main modules, sprite types, ancilla types, indoor rooms,
overworld screens, and active items. The terminal report is intentionally
compact; `--report-json` keeps the complete covered/missed lists for planning
new route segments. Repeated `--from-json` paths are unioned into one report so
supplemental route logs can be measured together; add `--json` to that merged
mode when you also want to write the union as a new raw coverage log for later
`--diff-from-json` scoring. `--require-full` exits
non-zero if any source-backed expected category still has misses, making it the
CI gate for the merged route suite once supplemental routes exist.
`--diff-from-json` scores the current run or merged `--from-json` set against a
base coverage log and prints which previously missed surfaces became covered;
`--delta-report-json` writes that same delta in machine-readable form.
`--seed-from-c-assets` adds a provenance-tagged supplement for frame-sampled
indoor rooms and overworld screens discovered from the C asset filenames. When
used without `--from-json`, it writes only that source-seeded supplement and
does not run replay; when used with `--from-json`, it merges the supplement into
the provided route logs.
`--route-report-json` writes a route-evidence report where source-seeded values
without a `first_seen` replay frame still count as missed. `--require-route-full`
applies the full-coverage gate to that route-evidence report.
`--route-worklist-json` writes a source-guided JSON worklist for the remaining
route-evidence misses. It classifies missed rooms/screens with direct entrances,
stair/hole source rooms, overworld entrances, travel/whirlpool source screens,
or `unclassified` when the C asset metadata does not expose an obvious route.
`--route-probes-from-worklist` consumes that worklist and writes a targeted
route-surface coverage log. Direct-entrance misses are loaded through the
normal dungeon entrance loader, overworld misses are loaded through the
overworld screen property loader, and indoor misses without a direct entrance
are marked through a focused dungeon-room probe. This is much faster than
joystick sweeps and keeps source-seeded-only entries out of the route-evidence
gate because every probed surface gets a normal `first_seen` frame.

Use this to answer “what does the current parity route prove?” before treating a
green `check --full` as broad parity proof. A green organic replay route still
only proves the surfaces hit by that route; missed surfaces need either new
route coverage or targeted probes. A green source-seeded coverage gate proves
that every source-backed room/screen asset is represented in the coverage
inventory, but it does not prove route observation for those seeded surfaces.
Use `--require-route-full` when the question is whether replay or targeted probe
logs actually observed every required surface.
The expected universe excludes explicit no-route dispatch slots: main-module
assert paths, `NULL`/assert-only sprite slots, and empty/unused ancilla slots.

```bash
# Generate raw route coverage and a complete miss report.
./target/debug/zparity coverage --full \
  --json .cache/parity-golden/coverage.json \
  --report-json .cache/parity-golden/coverage-report.json

# Re-render a report from an existing raw coverage log without rerunning replay.
./target/debug/zparity coverage \
  --from-json .cache/parity-golden/coverage.json \
  --report-json .cache/parity-golden/coverage-report.json

# Generate a supplemental coverage log for a recorded input window. Use
# `--load-state` when extending from a replay checkpoint, or `--load-sram` when
# starting from an SRAM sidecar. They are mutually exclusive. Add
# `--stop-replay-after-load` when a checkpoint tail should follow the provided
# input script instead of the embedded replay stream. Use `--input-script-overlay`
# when the checkpoint should keep consuming replay input and only explicit
# script frames should replace that replay input.
./target/debug/zparity coverage --frames 122000 \
  --input-script scripts/inputs/file-select-enter-game-message-dismiss-and-wander.txt \
  --load-state target/lockstep-checkpoints/file-select-enter-game-107000-message.bin \
  --stop-replay-after-load \
  --json .cache/parity-golden/coverage-file-select-message-dismiss-wander.json

./target/debug/zparity coverage --frames 9000 \
  --input-script-overlay .cache/parity-probes/entrance-routes/hc-east-overlay.txt \
  --load-state .cache/parity-probes/entrance-routes/route-004000-ow001b-stable.sav \
  --json .cache/parity-golden/coverage-hc-east-overlay.json

# Score a supplemental log against the current merged baseline before keeping it.
./target/debug/zparity coverage \
  --diff-from-json .cache/parity-golden/coverage-merged-current.json \
  --from-json .cache/parity-golden/coverage-file-select-message-dismiss-wander.json \
  --delta-report-json .cache/parity-golden/coverage-file-select-message-dismiss-wander-delta.json

# Generate a provenance-tagged room/screen supplement from C assets. This is
# fast and does not run replay.
./target/debug/zparity coverage \
  --seed-from-c-assets \
  --json .cache/parity-golden/coverage-source-seeded-c-assets.json \
  --report-json .cache/parity-golden/coverage-source-seeded-c-assets-report.json \
  --diff-from-json .cache/parity-golden/coverage-merged-current.json \
  --delta-report-json .cache/parity-golden/coverage-source-seeded-c-assets-delta.json

# Merge the full-route log with supplemental route logs into one miss report.
./target/debug/zparity coverage \
  --from-json .cache/parity-golden/coverage.json \
  --from-json .cache/parity-golden/coverage-branch-357697-main-menu-modules.json \
  --from-json .cache/parity-golden/coverage-stop-after-load-no-input-30000.json \
  --from-json .cache/parity-golden/coverage-branch-89424-bush-poof-ancilla.json \
  --from-json .cache/parity-golden/coverage-branch-916218-somaria-fission-ancilla.json \
  --from-json .cache/parity-golden/coverage-branch-511196-archery-game.json \
  --from-json .cache/parity-golden/coverage-branch-496311-pikit-shield-drop.json \
  --from-json .cache/parity-golden/coverage-branch-872500-flute-stop4-ow2f.json \
  --from-json .cache/parity-golden/coverage-branch-591900-wilted-terrace-ow7c.json \
  --from-json .cache/parity-golden/coverage-source-seeded-c-assets.json \
  --json .cache/parity-golden/coverage-merged-current.json \
  --report-json .cache/parity-golden/coverage-report.json \
  --require-full

# Generate a strict route-evidence worklist from the source-seeded baseline,
# consume it with targeted probes, then merge the probe supplement back into the
# route report.
./target/debug/zparity coverage \
  --from-json .cache/parity-golden/coverage-merged-source-seeded.json \
  --route-report-json .cache/parity-golden/coverage-route-evidence-source-seeded-report.json \
  --route-worklist-json .cache/parity-golden/coverage-route-worklist-source-seeded.json

./target/debug/zparity coverage \
  --route-probes-from-worklist .cache/parity-golden/coverage-route-worklist-source-seeded.json \
  --json .cache/parity-golden/coverage-route-probes-worklist-all.json \
  --report-json .cache/parity-golden/coverage-route-probes-worklist-all-report.json

./target/debug/zparity coverage \
  --from-json .cache/parity-golden/coverage-merged-source-seeded.json \
  --from-json .cache/parity-golden/coverage-route-probes-worklist-all.json \
  --json .cache/parity-golden/coverage-merged-route-probed.json \
  --report-json .cache/parity-golden/coverage-report-route-probed.json \
  --route-report-json .cache/parity-golden/coverage-route-evidence-report.json \
  --route-worklist-json .cache/parity-golden/coverage-route-worklist.json \
  --require-route-full

# Re-run the route-only gate from the merged probed log without rerunning probes.
./target/debug/zparity coverage \
  --from-json .cache/parity-golden/coverage-merged-route-probed.json \
  --route-report-json .cache/parity-golden/coverage-route-evidence-report.json \
  --route-worklist-json .cache/parity-golden/coverage-route-worklist.json \
  --require-route-full
```

## Two-tier golden layout

```
parity-golden/
  manifest.json      # schema, frames, ROM sha256, save sha256, timing-hacks, mask, block_size
  rollup.bin         # Tier A: per-block rollup fingerprints (~4.3 MB full route)
  merkle.bin         # Tier A: merkle root over rollup blocks (28 bytes)
  .gitignore         # excludes detail/ and *.fp (Tier B)

.cache/parity-golden/
  detail/            # Tier B: per-frame per-region fingerprints (large; local only)
  coverage*.json     # Route coverage logs and hit/miss reports (local only)
  ck/                # Rust checkpoint cache for sharded check runs (local only)
    manifest.json    # seed identity; incompatible inputs trigger automatic reseeding
```

Tier A is committed. Tier B is local-only (`.cache/parity-golden/` is gitignored).

## Environment overrides

| Variable | Default | Description |
|---|---|---|
| `ZELDA3_NEW_BIN` | `target/parity/zelda3` | Rust binary (used by `check` for checkpoint seeding) |
| `ZELDA3_ROM` | `saves/zelda3.sfc` | ROM file |
| `ZELDA3_REPLAY_SAVE` | `saves/zelda3-combined-route.sav` | Replay save |

## Workflow

### Finding and fixing a divergence

```bash
# 1. Find the earliest diverging frame:
./target/debug/zparity check --frames 50000

# 2. Localize per page/layer (needs Tier B; capture once with --detail):
./target/debug/zparity capture --full --detail    # one-time; takes 10-30+ min
./target/debug/zparity drill 3739

# 3. Fix the Rust-side bug (see CLAUDE.md debugging loop).

# 4. Verify the fix pushed the divergence later:
./target/debug/zparity check --frames 50000
```

### Re-capturing after a route or fixture-harness change

```bash
cargo build --profile parity -p zelda3-bin
cargo build -p parity
./target/debug/zparity capture --full          # Tier A only; commit parity-golden/
# (Optional) also capture Tier B for drill:
./target/debug/zparity capture --full --detail
```

### Spec and plan

- Spec: `.git/sdd/spec.md`
- Implementation plan: `.git/sdd/plan.md`
- Task briefs and reports: `.git/sdd/task-*-{brief,report}.md`
