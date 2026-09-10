# Draw tables as records

The Phantom Ganon, Vitreous, yellow Stalfos, and Mothula draw tables were
tuples of four integers, and each of their modules carried an adapter that
copied a slice of the tuple table into a vector of `DrawMultipleData`
records before calling the canonical `Sprite_DrawMultiple` port; Blind's
module wrapped the port with a fixed `None` record, and the credits draw
copied one of its 32 tuple tables into a vector on every call. The dungeon-NPC module
already had a `const` record constructor for its tables; it now lives in the
sprite module and the three tables use it, so the tables are records and
the four adapters, the credits copy, and the three tuple aliases are gone. The callers slice the
tables directly and keep each adapter's out-of-range behavior (an empty
draw where the adapter clamped, a panic where it indexed), and the yellow
Stalfos and Mothula draws receive their coordinate records from the
canonical port.

Compatibility constraints remain explicit:

- Every table entry keeps its four values; only the element type changed.
- The canonical port fills the coordinate record even when the sprite is
  out of bounds, where the adapter returned zeros; that record is consumed
  only while the sprite is not paused, and an out-of-bounds prep pauses it,
  so the difference is unobservable.

## Verification

The library compiles with no warnings in the parity, dev, and lib-test
builds; readability and projection discovery pass; the scanner output is
unchanged (zero HIGH RISK overlaps, 12 bridge-sync overlaps, 60 overlapping
bytes). All 1,775 library tests pass under the dev profile.

All 1,775 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`4f185b1c36bc6d630ff04ab9f6b63f52de12070522dc025587ad5f52491304af`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 314.01 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.
