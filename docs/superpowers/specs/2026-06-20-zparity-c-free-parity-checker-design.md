# zparity — C-free, parallel, all-layer parity checker

**Date:** 2026-06-20
**Status:** Approved design, ready for implementation plan

## Problem

Parity against the C oracle (`../zelda3`) is today verified by a fleet of slow,
C-dependent scripts (`validate_all_parity.py`, `full_divergence_scan.py`, …). Each
run requires a built C binary and re-replays the canonical route. We want a single,
fast, **C-free** checker that verifies **all** parity layers (WRAM, VRAM, SRAM,
render, audio) against a pre-captured golden artifact, parallelized across cores,
optimized for speed *and* accuracy.

The C oracle and the Rust binary already emit **byte-identical** dumps/traces for
matching state, so a golden captured once from C can be diffed against the Rust
binary forever without rebuilding C.

## Goals

- One command (`zparity check`) that gates **every** parity layer, no C required.
- Parallel across cores for the full ~1.07M-frame route (target: large wall-clock
  speedup over the current single-threaded C-paired runs).
- Pass/fail **gate** *and* drill-down **localization** (first diverging frame →
  layer → region) without re-running.
- Committable, small gate artifact; heavy detail artifact opt-in/cached.

## Non-goals

- Replacing deep-debug scripts (`step_diff.py`, `whoowns.py`, etc.) — they stay.
- Parallelizing the forward simulation itself (impossible; frame N depends on N−1).
  Parallelism comes from **sharding** the route via checkpoints.
- Capturing golden without C — `capture` still needs the C oracle (run rarely).

## Architecture

A new workspace crate **`crates/parity`** producing one binary **`zparity`** with
three subcommands. The crate is pure orchestration + golden I/O + merkle/compare; it
does **not** link the game crate. It spawns `target/parity/zelda3` (and, for capture,
`../zelda3/zelda3`) subprocesses, reusing the existing replay + checkpoint infra.

```
zparity capture   (needs C)  -> builds golden artifact (Tier A + optional Tier B)
zparity check     (C-free)   -> sharded parallel replay of Rust binary vs golden
zparity drill <f> (C-free)   -> per-layer/region breakdown at a frame (needs Tier B)
```

### Component 1 — Unified fingerprint stream (game binary + C oracle)

A new flag `--fingerprint-log <path>` (env `ZELDA3_FINGERPRINT_LOG`) added to **both**
the Rust binary and the C oracle. At the existing end-of-frame
`after-run-frame-internal` checkpoint it appends one **fixed-size 788-byte** record:

| field | size | source |
|---|---|---|
| `frame` | u32 | sanity; stream index is implicit/sequential |
| `wram[128]` | 128×u32 | 1 KB pages (reuses existing page-dump layout) |
| `vram[64]` | 64×u32 | 1 KB pages |
| `sram` | u32 | battery save |
| `render` | u32 | existing pixel-frame hash (`--render-hash-log` source) |
| `audio` | u32 | hash of the per-frame audio-trace line |
| `rollup` | u32 | FNV-1a over all the leaf hashes above |

All hashes are **FNV-1a/u32**, matching the existing page-dump hash so the formats
are interoperable.

**Determinism / snapshot-artifact fix (critical):** before hashing the affected WRAM
page, the hook zeroes a small `FINGERPRINT_MASK` byte set — currently `{0x654}`, the
HDMA snapshot-restore scratch byte. This makes a checkpoint-resumed shard produce
fingerprints **byte-identical** to a from-scratch run, so sharded `check` is
authoritative (no "confirm with one from-scratch run" caveat). The mask is a shared
constant; a unit test pins it to the documented artifact set so it can never silently
mask a real divergence.

### Component 2 — Golden artifact format (two-tier + merkle)

Committed tier in `parity-golden/`; cached tier in `.cache/parity-golden/`.

**Tier A — committed (~4.3 MB), the gate works from this alone:**
- `rollup.bin` — the `rollup` u32 column, one per frame (index implicit). ~4.3 MB.
- `merkle.bin` — frames chunked into **8192-frame blocks**; per-block hash + a single
  root. ~130 block hashes + root.
- `manifest.json` — schema version, frame count, ROM sha256, save sha256, C-oracle
  git rev, timing-hack set, `FINGERPRINT_MASK` set, block size, page granularity.
  `check`/`drill` refuse to run on a mismatched ROM/save/schema.

**Tier B — cached, not committed (~800 MB raw → zstd):**
- `detail/<block>.zst` — full 788-byte/frame records, one zstd file per 8192-frame
  block. Produced by `capture --detail`. Needed only by `drill` and `check --detail`.
  Absent on a fresh machine → gate still runs from Tier A; `drill` reports the detail
  tier is missing.

Merkle gives the gate an O(blocks) compare with O(log n) descent to the first
diverging frame, and a single root line as the PASS receipt.

### Component 3 — `zparity check` (sharded parallel run, C-free)

1. **Validate** manifest vs. local ROM/save/schema; abort on mismatch.
2. **Checkpoints:** ensure `.cache/parity-golden/ck/<frame>.sav` exist at the K−1
   shard boundaries (even split of the frame count). Generated once by a single
   sequential Rust run that emits `--save-state` at each boundary; cached and reused.
   Invalidated when the manifest's save/schema changes.
3. **Fan out** K worker processes (default K = physical cores). Worker *i* runs:
   `zelda3 --replay-save <rom> <save> <end_i> --load-state ck_i --fingerprint-log shard_i.bin`
   (shard 0 starts at frame 0 with no `--load-state`). All runs carry the 7 timing
   hacks.
4. **Compare** each `shard_i.bin` rollup column against `golden/rollup.bin[start_i..end_i]`
   (mmap + memcmp). With `--detail`, also diff the per-region columns vs Tier B.
   Workers compare as they finish; first mismatch recorded per shard.
5. **Aggregate:** all match → print merkle root + `MATCH (N frames, K shards, T s)`.
   Else → **globally-first** diverging frame, diverging layer(s) from the rollup
   descent, and a hint to run `zparity drill <frame>`.

Modes: `--full` (whole route), `--frames N` (prefix), `--shards K`. Below a frame
threshold, auto-drop to 1 shard (checkpoint overhead not worth it for smoke runs).

### Component 4 — `zparity drill <frame>` (C-free, needs Tier B)

Loads the detail block covering `<frame>` from Tier B, re-runs (or reuses) the Rust
fingerprint at that frame, and prints per-layer/region divergence: which WRAM page
(mapped to its const name via the existing `old_rust_ref`/`whoowns` const map), which
VRAM page, and sram/render/audio booleans.

## CLI

```
# one-time / when route or C oracle changes (needs C built):
zparity capture --full [--detail]

# everyday gate (C-free, parallel):
zparity check                  # smoke (e.g. 3000 frames, 1 shard)
zparity check --full           # whole route, sharded across cores
zparity check --full --detail  # also diff per-region (needs Tier B)

# on failure:
zparity drill 460431           # per-layer/region breakdown at a frame
```

Pre-commit hook swaps `validate_all_parity.py` → `zparity check` (smoke budget);
`--full` for the exhaustive gate. The legacy C-dependent scripts remain for deep
debugging.

## Data flow

```
C oracle ──capture──> golden (Tier A committed, Tier B cached)
                          │
Rust binary ──check (sharded)──> fingerprints ──compare──> MATCH | first-diverging
                          │
                          └──drill──> per-region breakdown (Tier B)
```

## Error handling

- Manifest mismatch (ROM/save/schema/mask) → hard abort with the differing field.
- Missing Tier B for `drill`/`--detail` → clear message, gate still runs from Tier A.
- Stale checkpoints (manifest save/schema changed) → auto-regenerate, log it.
- Worker subprocess nonzero exit → fail the check, surface last stderr (mirrors
  `validate_all_parity.py`'s behavior). macOS has no `timeout`; use a watchdog.
- Short/over-long fingerprint stream (frame-count mismatch) → reported as a failure
  with expected vs actual frame counts.

## Testing & validation

- **Self-consistency:** `capture` from C, then `check --full` against the current
  passing Rust binary → `MATCH`.
- **Equivalence vs. legacy:** on a deliberately buggy/reverted commit, `check` flags
  the same first-diverging frame as `full_divergence_scan.py` + `validate_all_parity.py`.
  This is the accuracy acceptance test.
- **Shard invariance:** `check --shards 1` and `check --shards 16` give identical
  results (validates the mask/determinism fix).
- **Mask audit:** unit test asserting `FINGERPRINT_MASK` equals the documented
  snapshot-artifact set.
- **Unit tests:** merkle build/descent, manifest validation, fingerprint record
  (de)serialization in the crate.

## Defaults / parameters

- Block size: **8192 frames**
- Page granularity: **1 KB** (WRAM + VRAM)
- Shards: **physical cores**
- Golden dir: **`parity-golden/`** (committed Tier A), `.cache/parity-golden/` (Tier B + checkpoints)
- Hash: **FNV-1a / u32**
- `FINGERPRINT_MASK`: `{0x654}` (extensible, test-pinned)

## Rollout

1. Add the `--fingerprint-log` hook to the Rust binary; add the matching hook to the
   C oracle (`../zelda3`, oracle-hook edits are permitted).
2. Build `crates/parity` with `capture`/`check`/`drill`.
3. Capture the golden; commit Tier A under `parity-golden/`.
4. Wire `zparity check` into `.githooks/pre-commit` alongside (then replacing) the
   smoke path of `validate_all_parity.py`.
