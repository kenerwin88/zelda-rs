# RAM Naming Policy

Use this when renaming or adding `ZeldaState::ram` offsets.

## Source Order

Prefer names that can be justified by current behavior and source context:

1. Existing Rust behavior and tests in `crates/zelda3/src`.
2. The local C checkout under `../zelda3`.
3. Original-source structure documented by RetroReversing, especially stable module families such as `z00_play.asm`, `z00_title.asm`, `z00_ending.asm`, and regional `zel_*` branches.

Do not use leaked-source labels blindly. Treat them as naming evidence, then verify the Rust behavior against the local oracle.

## Rules

- Do not introduce address-derived names such as `BYTE_7E1234`.
- Do not introduce direct hex RAM indexing such as `self.ram[0x1234]`; add a named offset first.
- Put new subsystem-owned offsets in `crates/zelda3/src/game_state/constants.rs`.
- Preserve old call-site names through imports when that keeps a move mechanical.
- Rename weak names only when surrounding behavior proves a better name.
- Leave uncertain weak names in place and track them with:

  ```sh
  python3 scripts/check_ram_readability.py --warn-weak-names
  python3 scripts/check_ram_readability.py --write-weak-doc
  ```

## Verification

For pure naming or helper changes, run:

```sh
python3 scripts/check_ram_readability.py
cargo check -p zelda3-bin
```

For player, messaging, NMI, or OAM-adjacent names, also run a lockstep oracle window. Prefer the saved-slot route when practical:

```sh
python3 scripts/oracle_windows.py --run --only file-select-enter-game
```

The shorter `title-start` window is acceptable for a quick smoke check while iterating, but not as the final proof for broad shared RAM renames.
