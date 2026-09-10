# The zero-page scratch words have one owner each

The sprite workspace modelled the first sixteen bytes of WRAM as one
scratch array, although the sprite system only fills the first two words
(the OAM-prep coordinates, or a killed sprite's load block and pointer)
and one byte at 0x0f. Tile detection models the same page's collision
words at 0x0c and 0x0e as sixteen-bit values, so bytes 0x0c..0x0f had two
native owners, the one HIGH RISK overlap the ownership scanner still
reported. Byte 0x0f was the substantive case: the sparkle garnish spawner
records the slot it used in R15, which is the high byte of the collision
word. The port kept two models of that byte coherent by hand, writing the
index into the workspace array and then copying it into the collision
word's high byte through a second bridge.

The sprite workspace now holds exactly the two zero-page words it fills,
and the collision word is the one owner of 0x0e..0x0f. The garnish spawner
writes the slot index through the tile-detection bridge's new
`set_last_garnish_index`, which stores the high byte and keeps the low
byte, so the bytes stored are the ones the two-model version produced. A
compile-time assertion pins the alias between the garnish index address
and the collision word's high byte. The checkpoint magic is bumped to
`Z3RSPC18` because the workspace's serialized array shrank.

Compatibility constraints remain explicit:

- The OAM-prep coordinates and the killed sprite's load block still share
  the same two words, as the original's R0 and R1 do.
- The collision word's low-byte setter still preserves the high byte, and
  the new high-byte setter preserves the low byte; every full-word setter
  still stores both bytes.
- Nothing reads the garnish index back; it is a WRAM-visible leftover.

## Verification

The two frozen runtime fixtures that hash the master projection into a
`0xa5` canvas (704 player-collision cases and 131,104 tile-behavior
executions plus 32 resets) changed, because the canvas no longer receives
bytes 0x04, 0x05, 0x08 and 0x09 from the workspace. Before re-freezing
them, a temporary probe dumped, for every case in both trees, the WRAM
hash, the hash of the projection with its first sixteen bytes masked, and
those sixteen bytes. Every WRAM hash and every masked projection hash was
equal between the base commit and this change; the sixteen bytes differed
only at those four offsets, where the base tree projected values no native
state owns and this tree leaves the canvas. The fixtures are re-frozen as
`player-collision-zero-page-scratch.txt` and
`tile-behavior-zero-page-scratch.txt` from this tree; the tests still have
no regeneration path.

`find_dual_ownership.py` now reports zero HIGH RISK overlaps, 10
bridge-published overlaps and 62 overlapping bytes (70 before). The
library compiles with no warnings in the parity, dev, and lib-test builds;
readability and projection discovery pass.
All 1,722 library tests pass under the dev profile.

The main tree validated this batch stacked with the two batches that follow
it, on parity binary
`310051682ebdaac62dee0198567287fae7005d8d8b8db68f296983bcd9d71a56`: the
200,000-frame cached comparison matched every video and audio hash in
319.56 seconds, the frame 60000 and 150470 WRAM goldens match, the
200,000-frame WRAM endpoint is the recorded `dd45975c…` image, and all
1,721 library tests pass under the parity profile.

The full cold route on the same binary matched all 1,581,079 frames of video
and audio in 2,466 seconds, with the four WRAM goldens and the full-route
WRAM endpoint (`31619379…`) unchanged; the run is promoted in
`routes/full_run/parity-frontier.json`.
