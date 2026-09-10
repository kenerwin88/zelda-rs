# Primitive out-parameters become return values

Three families of helpers still returned a value through a mutable
primitive, as their C originals did through a pointer.

- **The HUD item cursor.** Six helpers that step the selected item
  (previous, next, above, below, and the two that keep stepping until an
  owned item is found) took `&mut u8`. Each now takes the item by value
  and returns the new item; the nine callers assign the result.
- **The entity tile probe.** `GetTileAttribute` in the original reduces
  the caller's x to its map8 column outdoors and leaves it alone indoors,
  and some callers then use that x in a slope test. The port carried the
  `&mut u16`. `entity_tile_at`, `probe_entity_tile`, the sprite and
  ancilla wrappers now return the tile together with the x the caller's
  collision continues with: the pixel indoors, the column outdoors. The
  three callers that used the reduced x (the sprite tile classification
  and the two ancilla collision checks) destructure it; the others take the
  tile and drop the x. The hammer splash no longer copies x into a probe
  variable it discarded.
- **The interactive-ancilla cleanup.** The per-slot prefix wrote the slot
  index into a `&mut u8` when the slot held the kept interactive. It now
  returns `Option<u8>`; the one caller that keeps the index takes it, and
  the two that ignored it no longer declare a dummy.

Compatibility constraints remain explicit:

- The outdoor column is still `x >> 3` computed before the overworld
  lookup, and the indoor lookup still reads the pixel.
- The HUD steps still run in the same order with the same wrap points,
  and the callers store the result where they stored the mutated value.
- The cleanup loops still visit the slots in the same descending order,
  so the lowest matching slot still wins.

## Verification

The tile-probe unit tests assert the returned pair in place of the
mutated coordinate, with the same tiles and columns. The ownership scanner
is unchanged. The library compiles with no warnings in the parity, dev, and
lib-test builds; readability and projection discovery pass. All 1,721
library tests pass under the dev profile.
