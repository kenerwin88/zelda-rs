# Replay Fixtures

This repository tracks a small set of replay fixtures used by the parity tools.
It does not track ROMs, generated game assets, packaged binaries, emulator
installs, or trace captures.

## Tracked Fixtures

### `saves/zelda3-combined-route.sav`

Canonical standard-route replay save used by the C/Rust replay parity gate.
The accompanying proof manifest is `saves/zelda3-combined-route-proof.json`.

This file is not a ROM or generated asset pack. It is a route fixture for
regression testing, kept in `saves/` because the parity scripts use it as their
default input.

### `scripts/inputs/tas-us-full-completion-smv.sram`

SRAM sidecar extracted from the corresponding input script
`scripts/inputs/tas-us-full-completion-smv.txt`. The input script references it
with a `# sramPath` header so lockstep/TAS bootstrap checks can start from the
same SRAM state on each run.

This fixture is used for oracle windows. It is not a ROM and does not contain
generated runtime assets.

## Ignored Local Files

These paths are ignored and should not be committed:

- `generated/`
- `dist/`
- `target/`
- `.cache/`
- `.codegraph/`
- `*.sfc`, `*.smc`, `*.zip`, `*.srm`, `*.dat`
- `external/mesen2-oracle/local/`
- `external/snes9x-libretro/local/`
- `external/tas/`
