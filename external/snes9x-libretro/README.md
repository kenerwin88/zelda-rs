# Snes9x trace oracle

The exact Snes9x instrumentation used for parity work is stored in
`patches/zelda3-trace.patch`. The source checkout and compiled core are local
build products and are intentionally ignored.

Build the pinned core from any checkout:

```sh
python3 scripts/prepare_snes9x_trace_oracle.py
```

The builder clones `libretro/snes9x` at the pinned revision, applies the tracked
patch, compiles the libretro core, and writes both the core and a SHA-256 receipt
under `external/snes9x-libretro/local/`.

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

Available event domains are `frame`, `nmi`, `rng`, `pc`, `dma`, `hdma`, `ppu`,
and `wram`. `frame,nmi,rng` is the default. `hdma` emits paired start/end
records for every active scanline, including the active-channel mask and exact
Snes9x CPU position, so bus-stall cost can be measured without instruction
tracing. Optional filters:

- `ZELDA3_SNES9X_TRACE_FRAMES=FIRST-LAST` filters Snes9x's completed-frame
  counter. That counter can advance during one `retro_run` call.
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

Generate a strict Rust RNG replay script from a route trace with:

```sh
python3 scripts/extract_snes9x_rom_random.py /tmp/rng.jsonl \
  --output routes/clean/takes/0004/rom-random.txt
```

Each replay row records `run`, the RNG byte, and `carry=0|1`. The carry bit is
part of the routine's observable result because ROM callers can preserve it
across logical instructions and consume it in a following `ADC`.
