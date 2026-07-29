# Canonical Snes9x route parity

The canonical route is a continuous 155,384-frame playthrough compared against
Snes9x 1.63 Libretro. Video uses the production modern Rust renderer. Audio uses
the production modern backend and native sequencer.

The checked-in proof receipt is
[`routes/clean/continuous-result.json`](../../routes/clean/continuous-result.json).
It records the ROM, input, ROM-random, core, and state handoff hashes needed to
distinguish a real continuous proof from a stale or segmented receipt.

## Run the complete route

Provide a legally obtained USA ROM whose SHA-256 is recorded in the receipt,
build the non-tracing Snes9x core, and run:

```bash
python3 scripts/snes9x_route_recorder.py compare-route \
  --project routes/clean \
  --binary target/release/zelda3 \
  --core external/snes9x-libretro/local/snes9x_libretro.dylib \
  --rom saves/zelda3.sfc
```

The command rebuilds `zelda3-bin`, reconstructs the continuous input and
ROM-random streams from the route package, and fails at the first video or audio
divergence. A passing run must complete all 155,384 frames with exact video and
continuous audio.

Do not infer whole-route parity from an individual take's `frames` field in
`manifest.json`. For example, take 0004 contains 11,735 frames; it is one source
segment in the complete continuous route.

## Retained restart checkpoints

Local checkpoints are ignored build artifacts, not proof receipts. The three
lineages used to establish the current cold-derived proof are:

- `canonical-main-window-reset`
- `canonical-main-7653d6a2`
- `canonical-full-7653d6a2`

Preview superseded artifact cleanup:

```bash
python3 scripts/prune_parity_artifacts.py
```

Delete only the artifacts listed by that dry run:

```bash
python3 scripts/prune_parity_artifacts.py --apply
```

The pruning tool always retains the three canonical checkpoint lineages above.
