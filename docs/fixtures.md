# Fixture Provenance

This repository intentionally tracks a small number of replay fixtures that are
needed for parity work. It intentionally does not track ROMs, generated game
assets, packaged binaries, emulator local installs, or local trace captures.

## Tracked Fixtures

### `saves/zelda3-combined-route.sav`

Canonical standard-route replay save used by the C/Rust replay parity gate.
The accompanying proof manifest is `saves/zelda3-combined-route-proof.json`.

This file is not a ROM or generated asset pack. It is a route fixture produced
for regression testing and kept in `saves/` because the parity scripts use it as
their stable default input.

### `scripts/inputs/tas-us-full-completion-smv.sram`

SRAM sidecar extracted from the corresponding input script
`scripts/inputs/tas-us-full-completion-smv.txt`. The input script references it
with a `# sramPath` header so lockstep/TAS bootstrap checks can start from the
same SRAM state on each run.

This fixture is used for deterministic oracle windows. It is not a ROM and does
not contain generated runtime assets.

## Local-Only Artifacts

These paths are intentionally ignored and should not be committed:

- `generated/`
- `dist/`
- `target/`
- `.cache/`
- `.codegraph/`
- `*.sfc`, `*.smc`, `*.zip`, `*.srm`, `*.dat`
- `external/mesen2-oracle/local/`
- `external/bsnes-libretro/local/`
- `external/tas/`

