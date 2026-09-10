# The star-tile phase has one owner

Byte 0x04bc is `SWYKPT` in the original, the star-switch floor phase: a
pressed star tile toggles it, and the star-tile CHR restore reads it to
choose which half of the graphics to copy. The port modelled the byte
twice: `DisplayState` held it as an overworld "star-tile restore phase"
whose projection was gated to the overworld, and the dungeon room-effects
state held it as a "moving wall torch blink phase" gated to indoors. Both
uses are dungeon uses of the same byte, so the split was a misattribution,
and it cost a coherence exclusion, a double clear in the star-tile reset
(after a stale copy chose the wrong graphics half at frame 317704), and a
raw WRAM read to bypass whichever model was stale.

The dungeon room-effects state now owns the byte as `star_tile_phase`, with
the toggle and clear it already had and a reader for the CHR restore. The
display state no longer holds a copy, the two mode gates and the exclusion
are gone, the star-tile reset clears the byte once, and the graphics-half
choice reads the native field, which is always fresh at both call sites
because each follows a bridge write of the same byte. The single constant
is `STAR_TILE_PHASE`; the two old names are retired. The checkpoint magic is
bumped to `Z3RSPC19` because the display state lost a serialized field.

Compatibility constraints remain explicit:

- The room-load clear and the switch toggle store the byte at the same
  points as before.
- The projection is no longer gated on the indoors flag; the display copy
  it guarded against no longer exists, and the byte's value is the same in
  both models whenever both were loaded from WRAM.

## Verification

The display test that seeded the indoor copy and excluded it from the
coherence check is gone with the copy. `find_dual_ownership.py` reports
zero HIGH RISK overlaps, 9 bridge-published overlaps (10 before) and 61
overlapping bytes (62 before). The library compiles with no warnings in
the parity, dev, and lib-test builds; readability and projection discovery
pass. All 1,721 library tests pass under the dev profile.
