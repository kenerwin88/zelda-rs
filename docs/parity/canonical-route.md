# Canonical Snes9x route parity

The canonical route is a continuous 155,384-frame playthrough compared against
the official Snes9x 1.63 Libretro release. The exact upstream source tag and
revision are pinned in
[`external/snes9x-libretro/oracle-lock.json`](../../external/snes9x-libretro/oracle-lock.json).
Video uses the production modern Rust renderer. Audio uses the production modern
backend and native sequencer.

The checked-in proof receipt is
[`routes/clean/continuous-result.json`](../../routes/clean/continuous-result.json).
It records the ROM, input, ROM-random, core, and state handoff hashes needed to
distinguish a real continuous proof from a stale or segmented receipt. The
canonical input is retained separately from the actively extended recorder route
at
[`comparisons/canonical-155384/continuous-input.txt`](../../routes/clean/comparisons/canonical-155384/continuous-input.txt).

## Proof chain

The checked-in receipt records a compatible cold-derived checkpoint at frame
16,384, including the core hash that created it. The stable-release verification
then uses two exact A/V comparisons with a one-frame overlap:

- the tracing core verifies frames 16,384 through 27,649;
- the non-tracing core verifies frames 27,648 through 155,384.

The overlap proves that both Snes9x cores and both saved-state lineages agree at
the handoff. Do not replace this chain with a reset-only `compare-route` run
unless it also recreates the recorded cold-derived checkpoint and its generated
asset inputs.

Provide a legally obtained USA ROM whose SHA-256 is recorded in the receipt,
build `zelda3-bin` in release mode, and run the two comparisons:

```bash
target/release/zelda3 --compare-snes9x-oracle \
  external/snes9x-libretro/local/snes9x_libretro_trace.dylib \
  saves/zelda3.sfc 27649 \
  --expected-core-sha256 1b978832991521de2b0e1e25d40c4e9ed57eb8c458f9aa9cf21fc8621e128328 \
  --expected-rom-sha256 66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb \
  --input-script routes/clean/comparisons/canonical-155384/continuous-input.txt \
  --rom-random-script routes/clean/takes/0004/rom-random.txt \
  --resume-paired target/parity-checkpoints/canonical-main-window-reset/frame-00016384 \
  --audio-comparison exact \
  --scan-all

target/release/zelda3 --compare-snes9x-oracle \
  external/snes9x-libretro/local/snes9x_libretro.dylib \
  saves/zelda3.sfc 155384 \
  --expected-core-sha256 f82658246a0c1fed7d53c485f3de79fcc671a45d9ed0a0ac6bf6eace9d46ff9a \
  --expected-rom-sha256 66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb \
  --input-script routes/clean/comparisons/canonical-155384/continuous-input.txt \
  --rom-random-script routes/clean/takes/0004/rom-random.txt \
  --resume-paired target/parity-checkpoints/canonical-main-7653d6a2/frame-00027648 \
  --audio-comparison exact \
  --scan-all
```

Both commands fail on any enabled video or audio divergence. The receipt records
the restored runtime commit and the exact stable stock and trace core hashes used
to reverify both legs.

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
