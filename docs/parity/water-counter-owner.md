# The moving floor owns the water transition counter

Byte 0x0424 is the low byte of the moving floor's sixteen-bit Y offset
(`dung_floor_y_offs`) and, in the rooms that flood or drain
(`turn_on_off_water_ctr`), the water transition counter; no room has both a
moving floor and a water toggle. The port modelled the byte twice. The
dungeon environment state held the counter but excluded it from its
projection and wrote it through by hand in three bridge methods, with a
coherence exclusion to match, and the water-on room tag set both models,
after a stale projection had once re-stamped a zero over a fresh floor
offset (frame 606590) and a stale counter had once survived a room load
(frame 473650).

The moving floor state now owns the byte: the counter is a role accessor
over the low byte of the Y offset, with the same set, increment and
decrement the environment bridge offered, forwarded through the moving
floor bridge. The environment state lost the field, the write-throughs,
and the exclusion, and its bridge is now the plain adopting bridge. The
water-on room tag stores the counter once, and the room-load reset clears
the offsets once, since that already clears the counter. The constant
`TURN_ON_OFF_WATER_CTR` lives only in the address map. The checkpoint magic
is bumped to `Z3RSPC21`.

Compatibility constraints remain explicit:

- The counter's increment and decrement wrap within the low byte and leave
  the high byte of the offset word alone, as the original's byte
  arithmetic does.
- Every store lands at the same point in the flood and drain routines.

## Verification

The environment test that seeded the excluded counter and checked the
exclusion now checks only the trapdoor word it publishes. The ownership
scanner is unchanged. The library compiles with no warnings in the parity,
dev, and lib-test builds; readability and projection discovery pass. All
1,721 library tests pass under the dev profile.
