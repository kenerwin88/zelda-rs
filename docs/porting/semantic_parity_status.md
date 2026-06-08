# Semantic Parity Status

This file records the current implementation status for
`semantic_parity_migration.md`.

## Active Layers

- Byte parity remains the default strict oracle. It still compares WRAM, SRAM,
  VRAM, CGRAM, OAM, and PPU-visible registers unless a caller explicitly chooses
  a graduated semantic comparator.
- Semantic snapshots are available beside byte parity and can be printed with
  `--trace-semantic-state`. They name frame, player, world, Map16 load, sprite,
  ancilla, and PPU-facing fields.
- Typed RAM views provide the source-address-backed extraction layer for frame
  control, player state, world/camera state, sprite slots, ancilla slots, and
  overworld Map16 load state.

## Graduated Subsystems

### OverworldMap16Load

`OverworldMap16Load` is the first graduated subsystem. Rust stores the typed
runtime state in `OverworldMap16LoadState` while keeping the legacy WRAM bytes
materialized for checkpoints, replay tooling, and the default byte gate.

The explicit graduated comparator may narrow raw WRAM comparison for the
private Map16 load bytes only when semantic snapshots prove these fields match:

- `world.map16_load_src`
- `world.map16_load_dst`
- `world.map16_load_y_unit`

The default lockstep path does not use this narrowing, so existing byte parity
remains the strongest regression gate.

## Subsystems Still Requiring Byte Parity

All other game state still requires strict byte parity. Player movement,
camera/scroll, inventory/menu state, sprites, ancilla, shared scratch, and
PPU/audio-facing effects have semantic fields for diagnostics. Sprite and
ancilla semantic extraction now uses typed slot views over WRAM, but they are
not graduated because their owned-state storage, edge-route coverage, and
output-specific parity gates are not yet strong enough to replace raw WRAM
comparison.

Shared scratch remains byte-only until call-site lifetime proves it is local.
PPU and audio behavior remain output-gated separately from gameplay semantics.
