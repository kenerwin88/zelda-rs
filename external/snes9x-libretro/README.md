# Snes9x trace oracle

The oracle is pinned by `oracle-lock.json` to the official Snes9x 1.63 release
tag and its exact source commit. The exact instrumentation used for parity work
is stored in `patches/zelda3-trace.patch`. The source checkout and compiled
cores are local build products and are intentionally ignored.

Build the pinned core from any checkout:

```sh
python3 scripts/prepare_snes9x_trace_oracle.py
```

The builder clones the pinned upstream release, builds a pristine stock
libretro core, applies the tracked patch, and builds the instrumented core.
Both cores receive SHA-256 receipts under `external/snes9x-libretro/local/`.
The receipts are reproducible for the same source, patch, and toolchain; on
macOS the linker retains its content-derived UUID so the cores remain loadable.

Tracing is off unless `ZELDA3_SNES9X_TRACE` names an output file. Output is
JSON Lines so it can be filtered with `jq` or compared mechanically.

```sh
ZELDA3_SNES9X_TRACE=/tmp/rng.jsonl \
ZELDA3_SNES9X_TRACE_EVENTS=frame,nmi,rng,pc \
ZELDA3_SNES9X_TRACE_FRAMES=9468-9482 \
ZELDA3_SNES9X_TRACE_PCS=05:bcce,0d:ba71-0d:ba80 \
python3 scripts/snes9x_route_recorder.py compare-route \
  --project routes/clean \
  --core external/snes9x-libretro/local/snes9x_libretro_trace.dylib \
  --no-build
```

Available event domains are `frame`, `nmi`, `nmi-resume`, `rng`, `pc`, `dma`,
`hdma`, `ppu`, and `wram`. `frame,nmi,rng` is the default. `nmi-resume` emits
one record when RTI restores the interrupted PC and stack, without enabling
high-volume instruction logging. `hdma` emits paired start/end
records for every active scanline, including the active-channel mask and exact
Snes9x CPU position, so bus-stall cost can be measured without instruction
tracing. Optional filters:

- `ZELDA3_SNES9X_TRACE_FRAMES=FIRST-LAST` filters Snes9x's completed-frame
  counter. That counter can advance during one `retro_run` call. In
`--live-oracle-rng` comparisons the required `rom-rng` events deliberately
  remain route-wide, while every other requested trace domain still obeys
  this window.
- `ZELDA3_SNES9X_TRACE_PCS=BB:AAAA,...` selects up to 64 instruction-address
  ranges. A range is matched directly, so a large routine does not consume one
  filter slot per byte.
- `ZELDA3_SNES9X_TRACE_PPU=2100,212c-2132` selects PPU writes. RNG
  counter reads are emitted by the `rng` domain.
- `ZELDA3_SNES9X_TRACE_WRAM=0010-0017,0fa1` selects WRAM writes.
- `ZELDA3_SNES9X_TRACE_PIXEL=52,56` captures the final winning draw operands
  for one active-display pixel in the parity receipt. Leave it unset for the
  ordinary zero-overhead render path.

Every record includes `run`, the zero-based `retro_run` invocation that owns
the event, as well as Snes9x's `frame` counter, CPU position, registers, the top
four stack bytes, the decoded long-call return address, CPU carry, and the
Zelda main/NMI/RNG state. Use `run` for host-frame timing; use `frame` for the
emulator's internal video boundary. The trace patch also exports the optional
PPU inspection functions consumed by the parity harness, including OAM/OBJ
evaluation, resolved clip spans, a selected pixel's palette/color-math
operands, and the per-scanline Mode 7 matrix and Window 1/2 edges captured at
the completed-screen publication boundary.

Turn a narrow checkpointed trace into an explicit NMI/DMA receipt with:

```sh
python3 scripts/snes9x_dma_receipt.py /tmp/snes9x-trace.jsonl \
  --host-frame 29644 --resume-frame 29010
```

With an instrumented core built from the current trace patch, add
`--hdma-channel 7` to include the exact source address and bytes consumed on
every active scanline. This distinguishes the table present after a host frame
from the generation HDMA actually presented during that frame.

The tool deliberately maps `host frame - resume frame` to `run` and rejects
missing or ambiguous runs. Its report gives the ordered DMA source,
destination, byte count, initiating PC, and raster position, followed by the
decoded OBJ cache provenance for a traced pixel. This avoids the common error
of treating the restored core's internal `frame` counter as the absolute route
frame.

For CPU continuation work, collapse a narrow `pc,nmi,nmi-resume,frame` trace
into a provenance-checked semantic ledger:

```sh
python3 scripts/snes9x_cpu_phase_ledger.py \
  /tmp/compare/oracle-rom-random.jsonl \
  --first-run 27238 --last-run 27251 \
  --output /tmp/module7-ledger.json
```

The ledger coalesces hot-loop PC events into ordered spans while retaining NMI
entry/resume boundaries. It records the trace, manifest, core, and ROM hashes,
and uses the zero-based `retro_run` coordinate explicitly. It is offline
reference evidence only; the Rust game runtime must use native semantic work
and cycle budgets rather than reading the ROM or a route ledger.

To compare the oracle's actual main-thread checkpoint against the isolated
Rust timing shadow without guessing a frame offset, enable the Rust checkpoint
trace during the same comparison. Rust records an absolute host frame; the
join uses `manifest.json`'s `timing.start_frame` to derive Snes9x's
window-relative `retro_run`:

```sh
ZELDA3_CPU_CHECKPOINT_TRACE=/tmp/rust-cpu-checkpoints.jsonl \
ZELDA3_SNES9X_TRACE_EVENTS=frame,nmi,nmi-resume,pc \
ZELDA3_SNES9X_TRACE_PCS=00:8051 \
ZELDA3_SNES9X_TRACE_FRAMES=33347-33352 \
target/parity/zelda3 --compare-snes9x-oracle ...

python3 scripts/compare_snes9x_cpu_checkpoints.py \
  /tmp/compare/oracle-rom-random.jsonl \
  /tmp/rust-cpu-checkpoints.jsonl \
  --manifest /tmp/compare/manifest.json \
  --first-host-frame 33347 --last-host-frame 33352
```

The join rejects duplicate/missing runs and semantic state disagreement. This
is deliberate: a report is only valid when the Rust and Snes9x records prove
that they describe the same host invocation and game state. The Rust hook is
limited to timing shadows that begin at the current host's real main wait;
future-envelope predictions are deliberately not emitted as capture-host
checkpoints.

Do not add `$00:8034` to a multi-frame PC trace: it is the hot main wait loop
and produces millions of redundant records. The `nmi-resume` event already
records the interrupted/resumed wait PC once per NMI.

For an every-frame comparison of the state leading to OBJ scanout, use the
instrumented core with `ZELDA3_CAPTURE_OBJ_STATE_LEDGER=1` and a session
directory. `obj_state_ledger.jsonl` records compact hashes and exact mismatch
counts for WRAM, raw OBJ VRAM, live OAM, presented OAM, and the Snes9x-valid
portion of the decoded OBJ tile cache. It reports modeled semantic WRAM fields
separately; the full-WRAM hash is observational because scratch and unmodeled
bytes are intentionally not a parity gate. On the first presented-cache mismatch,
the harness also writes full Rust/oracle WRAM and VRAM plus a detailed display
publication receipt. That receipt automatically includes every competing Rust
publication generation and its valid-cache mismatch count, so an exact source
is visible without a second run or another environment flag. Invalid Snes9x
cache entries are deliberately excluded: their retained bytes were not
eligible for scanout and otherwise create false early alarms.

```sh
ZELDA3_CAPTURE_OBJ_STATE_LEDGER=1 target/parity/zelda3 \
  --compare-snes9x-oracle \
  external/snes9x-libretro/local/snes9x_libretro_trace.dylib \
  saves/zelda3.sfc 15000 \
  --replay-bundle routes/full_run/comparisons/precommit/run-37900-video-preflight \
  --ignore-audio --session-dir /tmp/obj-state-ledger
```

`--replay-bundle` is the preferred cold-route input. It selects `input.txt`,
`rom-random.txt`, and `initial.srm` together, verifies their ROM identity and
recorded `frames_completed` coverage, and records hashes for all three in the
new session manifest. Legacy per-file flags are rejected when their files come
from different directories. `--allow-mixed-replay-provenance` exists only for
an intentional diagnostic whose mixed origin is understood and documented.

For a long oracle-only survey, pass both `--ignore-video` and `--ignore-audio`
to the comparison command. This retains deterministic input, paired-resume,
and trace handling while skipping Rust video/audio comparison and its artifact
cost.

Generate a strict Rust RNG replay script from a route trace with:

```sh
python3 scripts/extract_snes9x_rom_random.py /tmp/rng.jsonl \
  --output routes/clean/takes/0004/rom-random.txt
```

Each replay row records `run`, the RNG byte, and `carry=0|1`. The carry bit is
part of the routine's observable result because ROM callers can preserve it
across logical instructions and consume it in a following `ADC`.
