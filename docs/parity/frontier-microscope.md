# Parity frontier microscope

`./parity` is the safe entry point for work at the `routes/full_run` frontier.
It distinguishes diagnostic evidence from commit authority, pins every replay
source, maps host frames through `retro_run`, expands LoROM PC mirrors, and keeps
oracle-only evidence in a tamper-evident content-addressed cache. None of these
artifacts are read by the Zelda runtime.

## Start every investigation here

```sh
./parity status
./parity doctor
./parity microscope --frontier 31287 \
  --pc UncacheAndExecuteSprite --pc 1d:eb43 \
  --wram 0d00-0fff
```

The default microscope run uses a paired checkpoint and is **diagnostic only**.
On first use it runs a low-volume live-oracle-RNG calibration, materializes that
same run's cartridge samples, binds their hash to the diagnostic checkpoint,
then resumes the narrow trace with the recorded samples. This prevents stale
RNG call-order scripts from being paired with a changed Rust binary.
It writes one directory under `target/parity-microscope/` containing:

- `microscope-plan.json`: binary, Git, route, core, ROM, input, RNG, SRAM, and
  trace identity selected before execution;
- `replay-microscope.sh`: the exact reproducible command and environment;
- `snes9x-trace.jsonl` and `rust-cpu-checkpoints.jsonl` when those events occur;
- `snes9x-trace.zpti`: a compact Rust-generated seek index bound to the exact
  trace and session manifest hashes;
- `cpu-checkpoint-correlation.json`: a manifest-derived join from Rust's
  absolute host frame to Snes9x's window-relative `retro_run`; resumed traces
  with the old mislabeled coordinate are rejected instead of offset-guessed;
- `timeline.json` and `timeline.txt`: explicit
  `host frame = comparison start + retro_run` mapping, Snes9x internal frames,
  NMI/resume/DMA/WRAM events, canonical PCs, and symbols;
- `report.json`: the first reported lane cause and immutable-cache location.

Use `--dry-run` to audit selection without launching either engine. Use `--cold`
only for a focused from-zero diagnostic; it is still not the cold exact A/V
promotion gate.

By default the checkpointed trace is limited to the last 12 internal frames
before the frontier. This keeps narrow PC/WRAM investigations small even when
the rolling checkpoint is hundreds of frames behind. Use
`--trace-tail-frames N` to widen it or `--trace-tail-frames 0` to retain the
whole checkpoint window. The derived range is recorded in the plan and cache
identity. The low-volume `$00:8051` CPU checkpoint is included automatically;
`--no-cpu-checkpoint` disables that correlation.

Before a long run, `./parity doctor` verifies binary freshness, trace-core and
replay provenance, the paired checkpoint's RNG binding, every immutable cache
hash, available disk, and the serial GPU/oracle lock. Inspect any completed
session without rerunning it:

```sh
./parity inspect target/parity-microscope/run-31287-YYYYMMDD-HHMMSS
```

The inspection reports event volume and busiest host frames, validates frame
boundaries, verifies the referenced cache, and refuses unsafe CPU-coordinate
joins.

## Rust evidence engine

Python remains the workflow control plane: it selects and launches the pinned
replay, manages checkpoints, and renders reports. `zparity` is the typed,
streaming evidence plane for high-volume data. Build it with:

```sh
cargo build --profile parity -p parity
```

Every completed microscope run automatically builds a trace index when that
binary is available. Build or rebuild one explicitly, then query the original
JSONL without reparsing the entire trace:

```sh
./parity trace-index target/parity-microscope/run-31287-YYYYMMDD-HHMMSS
./parity trace-query target/parity-microscope/run-31287-YYYYMMDD-HHMMSS \
  --host-frame 31285 --pc 00:8051 --limit 5
./parity trace-query target/parity-microscope/run-31287-YYYYMMDD-HHMMSS \
  --run 85 --internal-frame 87 --event wram --wram 0d00-0fff
```

The index stores typed coordinates, event names, PCs, WRAM addresses, and byte
offsets—not copied evidence. A query first verifies both the source trace and
the comparison manifest hashes, then seeks and emits the exact original JSONL
records. It rejects a changed source, changed manifest, truncated/extended
index, out-of-range offsets, ambiguous duplicate options, and unknown options.
`--event wram` intentionally matches the oracle's concrete `wram-write` event;
LoROM PC matching canonicalizes low/high mirrors.

Verify the entire immutable cache independently through the Rust engine with:

```sh
./parity cache-verify
```

This checks each directory key against the canonical cache identity and hashes
every artifact. These operations accelerate inspection; they do not elevate a
focused trace or cached artifact to A/V parity authority.

### PC and frame safety

Symbol names come from the local C checkout's `other/names.txt` by default.
Override that development-only input with `ZELDA3_ROM_SYMBOLS`. A requested
LoROM PC automatically traces both mirrors, so `06:eb43` becomes
`06:eb43,86:eb43`. Inspect the mapping directly with:

```sh
./parity pc 06:eb43
./parity pc UncacheAndExecuteSprite
```

The tool refuses unfiltered `pc` and `wram` trace domains. It never equates the
trace's internal `frame` field with the route frame. Convert an existing trace
with an explicit comparison start:

```sh
./parity timeline /tmp/trace.jsonl --start-frame 29010 \
  --strict --output /tmp/frame-31287
```

`--strict` fails if any selected `retro_run` lacks exactly one entry and one
return boundary. A trace is stopped at 128 MiB by default; use a narrower filter
instead of raising `--max-trace-mib` casually. Cold PC/WRAM tracing is refused
unless `--trace-internal-frames FIRST-LAST` explicitly bounds it.

## Immutable oracle cache

Every completed microscope session extracts only immutable replay sources and
oracle evidence into `.git/parity-oracle-cache/<sha256>/`. The key includes the
core, ROM, input, recorded RNG, SRAM, frame origin/range, and trace schema. Cache
reuse verifies every artifact hash and stops on tampering.

Cache any existing comparison session explicitly:

```sh
./parity cache routes/full_run/comparisons/precommit/run-29627
```

The cache eliminates repeated extraction and interpretation of oracle states,
semantic frame receipts, narrow CPU/NMI/DMA traces, and (for sessions produced
by the current harness) the complete per-compared-frame canonical A/V hash
ledger. `./parity doctor` verifies every cache entry in one pass.

Use the typed Rust comparators rather than hand-joining JSON:

```sh
./parity receipt-compare SESSION --cache CACHE
./parity av-compare SESSION --cache CACHE
./parity cached-av CACHE --rom saves/zelda3.sfc
./parity oracle-av-capture SOURCE_SESSION --frames 29505
```

Semantic frame receipts remain sampled on long runs, and their report says
whether the recorded frames are contiguous. The A/V ledger covers every
compared frame and hashes visible row-major RGB (excluding alpha and libretro
pitch padding) plus exact interleaved stereo i16 little-endian samples. This is
cryptographically exact pass/fail evidence without multi-gigabyte raw video.
The cache identity binds its evidence schema to core, ROM, input, recorded RNG,
SRAM, and frame origin/range, and extraction strips all Rust hashes.

The authoritative video/audio gate still runs the pinned core. Cached hashes
are a fast regression tier below promotion authority; they do not replace two
cold exact A/V passes or preserve raw mismatching pixels/samples. The ordinary
live comparison continues to write full first-mismatch artifacts for diagnosis.

`cached-av` is the fast iteration path. It verifies the content-addressed cache
and ROM, then replays only the Rust engine from the cached input, RNG, SRAM, and
per-frame oracle audio schedule. It never loads the Snes9x core, stops at the
first canonical video/audio hash difference, and invokes the typed comparator
on the resulting candidate ledger. The initial implementation intentionally
accepts only cold, contiguous caches whose start and comparison frames are both
zero; supporting resumed caches requires a separately provenance-bound Rust
checkpoint and must not silently reuse an oracle-only state.

`oracle-av-capture` is the one-time reference producer. It runs only the pinned
Snes9x core from a source session's input and SRAM, validates and binds the
recorded RNG script as replay provenance, and records every oracle video/audio
hash plus its exact audio callback size. It does not run Rust, so a current Rust
regression cannot truncate or contaminate the reference ledger. The wrapper
immediately extracts the result into the immutable cache. This capture is
oracle evidence, not a parity pass, and its `result.json` says so explicitly.

## Evidence tiers

1. **Focused diagnostic:** `./parity microscope`, normally checkpointed. Use it
   to prove the first causal phase, not parity.
2. **From-zero renderless:** replay from cold through the frontier plus a margin
   with exact input/RNG provenance.
3. **Recorded-RNG video:** cold stock-core video comparison through the required
   ratchet window.
4. **Cold exact A/V promotion:** run the ordinary pre-commit gate twice. Each
   successful cold run records a receipt under `.git/parity-cold-passes/`.

Do not skip from tier 1 to a parity claim. Do not run GPU comparisons in
parallel; the existing exclusive lock remains authoritative.

## Durable frontier promotion

`routes/full_run/parity-frontier.json` separates three facts that were formerly
conflated:

- a durable, commit-bound promoted frontier;
- the pre-existing local ratchet, which was not commit-bound;
- newer experimental failures, which are observations rather than promotions.

After a fix is committed and the tree is clean, promote two independent cold
exact A/V receipts for the still-current binary:

```sh
./parity promote
```

Promotion refuses a dirty tree, a different binary, resumed runs, disabled
lanes, one cold run, or two receipts for the same session. It promotes only the
lower frontier proved by both runs and writes the exact commit and receipt
hashes. Commit the resulting ledger change as metadata with `--no-verify`.

## Change-size discipline

One parity implementation commit should explain one root cause and identify its
semantic entry/return points, interruptible work, NMI crossings, and observable
side effects. Add tests immediately before, at equality, and after the boundary.
If two trace iterations do not prove a hypothesis, stop editing production code
and improve the microscope evidence instead. Route/frame/room predicates and
fixed observed NMI counts are diagnostic assertions, not runtime fixes.
