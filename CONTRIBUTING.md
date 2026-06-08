# Contributing

This repository is a Rust port of `snesrev/zelda3`. The main development rule is
to keep behavior verifiable against the original C project before refactoring
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

The pre-commit hook runs the standard replay parity gate. By default it expects
the C checkout at `../zelda3` and a USA ROM at `../zelda3/zelda3.sfc`. This is
heavier than CI because GitHub runners do not have the ROM or local C checkout.

## Checks

Run the public, ROM-free checks before opening a pull request:

```bash
cargo fmt --all -- --check
cargo check -p snes -p zelda3 -p platform -p renderer -p assets
cargo test -p snes -p zelda3 -p platform -p renderer -p assets
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
- Keep the C checkout as the oracle for behavior until a subsystem is explicitly
  verified and refactored.
