# Goals

Rewrite the [zelda3](https://github.com/snesrev/zelda3) C codebase
(`../zelda3`, ~50K LoC of reverse-engineered game logic
plus an embedded SNES emulator) in Rust.

## Scope (user-chosen, see initial brainstorm)

- **Game logic** (C: `src/`, ~40 files)
- **SNES emulator** (C: `snes/`, ~10 files) — used as the verification oracle
- **Native frontend / main loop** — `winit` + `pixels` + `cpal`, with bsnes
  libretro kept as a separate parity oracle
- **Python asset extractor** (C: `assets/restool.py` and friends) → also Rust

## Approach: hybrid

1. **Faithful port first.** Translate C structures and functions 1:1 so the
   `EmuRunFrameWithCompare`-style byte-for-byte WRAM/SRAM/VRAM check against
   the original SNES ROM still works. Read like C-in-Rust for now.
2. **Refactor toward idiomatic Rust after verification.** Once a module has a
   green oracle check, it's safe to clean it up — break globals into structs,
   replace asserts with `Result`, etc.

Why hybrid: a pure idiomatic rewrite has to trust its own judgment across
~50K lines of obscure reverse-engineered game code. The faithful port keeps
the SNES emulator as ground-truth, so every divergence shows up as a concrete
byte-diff at a specific frame.

## Verification oracle (the load-bearing design choice)

The C project includes a SNES emulator (`zelda3/snes/`) that runs the
**original ROM in parallel** with the C reimplementation each frame, then
`memcmp`s WRAM (`0x20000`), SRAM (`0x2000`), and VRAM (`0x8000` words)
byte-for-byte. See `zelda3/src/zelda_cpu_infra.c:EmuRunFrameWithCompare`.

We're keeping that mechanism in Rust:

- `crates/snes` ports `zelda3/snes/` — must produce identical WRAM/VRAM
  byte-for-byte after the same input sequence.
- `crates/zelda3` (not started yet) will port `zelda3/src/`. Each module ported
  is then validated by the oracle.

Caveat: the C build does *not* compare APU state, so we can keep audio stubbed
during oracle validation.

## Folder

- Sibling to the C source: `../zelda3-rs/` (i.e. `.`).
- Single Cargo workspace, one crate per logical area.

## Frontend crate choice

The playable host uses the `platform` crate's `NativeFrontend`, backed by
`winit` for window/input, `pixels` for presentation, and `cpal` for audio.
SDL is not part of the Rust dependency graph; remaining `SDL_*` names are
compatibility constants from the C configuration surface unless they are
explicitly linked by a future change.

## Non-goals (for now)

- No netcode, no new features, no save-state migration tooling, no balance
  changes. The point is a faithful Rust replacement, not a rewrite of the
  game design.
- We do not preserve C struct memory layout — only **memory regions that the
  oracle compares** (WRAM, SRAM, VRAM) need to match the C build byte-for-byte.
  Internal field layouts of `CpuState`, `DmaState`, etc. are free to be
  idiomatic.

## Source of truth

`../zelda3` is read every time we port a module —
don't trust this repo's prose over the C source.
