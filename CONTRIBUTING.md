# Contributing

This repository is a Rust port of `snesrev/zelda3`. The main development rule is
to keep behavior verifiable against the reference implementation before refactoring
toward more idiomatic Rust.

## Setup

Clone with submodules:

```bash
git clone --recurse-submodules <repo-url>
cd zelda3-rs
```

Install the local hooks when you have the parity dependencies available:

```bash
scripts/install_hooks.sh
```

The pre-commit hook runs the standard smoke and Snes9x parity gate. By default it
expects the local parity fixtures and a ROM at the repo's standard location. This
is heavier than CI because GitHub runners do not have the ROM or parity route
artifacts.

## Checks

Run the public, ROM-free checks before opening a pull request:

```bash
cargo fmt --all -- --check
cargo check -p snes -p zelda3 -p platform -p renderer -p assets
cargo test -p snes -p zelda3 -p platform -p renderer -p assets
python3 scripts/create_ci_assets.py --out-dir "$PWD/target/ci-assets/zelda3_assets"
ZELDA3_ASSETS_DIR="$PWD/target/ci-assets/zelda3_assets" cargo check -p zelda3-bin
ZELDA3_ASSETS_DIR="$PWD/target/ci-assets/zelda3_assets" cargo run -p zelda3-bin -- --standalone-smoke 2
python3 -m py_compile scripts/*.py
python3 scripts/check_ram_readability.py
```

When changing gameplay, rendering, SRAM, replay, or oracle code, also run the
local parity gate:

```bash
.githooks/pre-commit
```

## Porting Discipline

- Prefer source-backed names and WRAM aliases over invented labels.
- Keep parity fixtures in their documented locations.
- Do not commit ROMs, generated asset packs, emulator installs, packaged
  binaries, or local trace captures.
- Keep the current Snes9x route baseline as the behavior reference until a subsystem
  is explicitly verified and refactored.
