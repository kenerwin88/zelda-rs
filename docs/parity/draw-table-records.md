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
