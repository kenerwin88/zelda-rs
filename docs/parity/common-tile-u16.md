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

All 1,776 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`869df9964c1a2201b5e8433283d64efd180ec9f3f5e8417774e86e39f7e0aed4`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 313.91 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.
