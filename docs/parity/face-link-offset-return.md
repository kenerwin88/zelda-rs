# The face-Link helper returns its offset

`Sprite_DirectionToFaceLink` in the original returns the direction a sprite
must face to look at Link and, through an optional pointer argument, the
sprite's signed byte offset from Link. The port kept the pointer as an
`Option<&mut PointU8>` out-parameter: seventeen callers declared a zeroed
point, passed a mutable borrow, and read it back, and the guard module
carried two private adapters that wrapped the same call to hide the
argument.

The helper is now two functions. `sprite_direction_to_face_link(k)` returns
the direction, and `sprite_direction_and_offset_to_face_link(k)` returns
the direction together with the offset as a value; the former calls the
latter. The sixty-two direction-only callers lost their `None` argument,
the seventeen offset callers destructure the pair, and the guard adapters
and the stale comments about duplicated constants are gone. The ball-and-
chain trooper keeps its zeroed point for the frames on which it does not
recompute the direction, as before.

Compatibility constraints remain explicit:

- The scratch counter store inside the helper and the order of the two
  proximity probes are unchanged.
- Every caller performs the call at the same point in its routine; the
  recruit's comparison computes the direction before the comparison rather
  than inside it, which changes no observable state.

## Verification

The face-Link unit test destructures the pair and checks the same direction,
offset and scratch counter as before. The ownership scanner is unchanged.
The library compiles with no warnings in the parity, dev, and lib-test
builds; readability and projection discovery pass. All 1,721 library tests
pass under the dev profile.
