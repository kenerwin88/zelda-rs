# Common-tile update takes 16-bit coordinates

`Dungeon_UpdateTileMapWithCommonTile` and `Dungeon_PrepSpriteInducedDma`
took `i32` coordinates because the original declares them `int`, but every
caller starts from a 16-bit tilemap position or sprite coordinate, and both
routines mask the coordinates below bit 16 before use (the plus-16 and
plus-1 offsets cannot change the masked result). The two now take `u16`,
which removes the four call-site conversions and the two per-module cast
adapters (garnish and Mothula), whose six callers use the port directly.

Compatibility constraints remain explicit:

- The masked position is identical for every input: `(x + 16) & 0x1f8` and
  `(y + 1) & 0x1f8` agree between `int` and wrapping 16-bit arithmetic
  because the mask lies below bit 16.
- The VRAM packet writes and their order are untouched.

## Verification

The library compiles with no warnings in the parity, dev, and lib-test
builds; readability and projection discovery pass; the scanner output is
unchanged (zero HIGH RISK overlaps, 10 bridge-sync overlaps, 60 overlapping
bytes). All 1,776 library tests pass under the dev profile.
