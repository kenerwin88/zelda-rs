# The multi-part draw helpers return the prepared coordinates

`Sprite_DrawMultiple` in the original takes a pointer to a coordinate
record; it always fills the record with the sprite's prepared OAM
coordinates, even when the sprite is off screen and nothing is drawn, so
the caller can position extra parts (shadows, tongues, tails) from it. The
port carried the pointer as an `Option<&mut PrepOamCoordsRet>`: sixty-one
callers declared a zeroed record, passed a mutable borrow, and read it
back, and seventy-three passed `None`.

The five helpers (`sprite_draw_multiple`, the encoded-record and WRAM-record
variants, the player-deferred variant, and the shared preparation step) now
return the prepared record. Preparation returns the record together with
the draw triple when the sprite is on screen; each helper draws when it
has the triple and returns the record either way. Direction-only callers
lost their `None`; the callers that used the record bind the return value
(fifty-five as a fresh binding, six as an assignment into a record the
routine also fills elsewhere). Two Helmasaur routines that copied the
result field by field into their caller's record now assign it whole. Two
routines whose branches each drew and then cast the same shadow
(the uncle's departure and Kiki) bind the record from the branch
expression and cast the shadow once after it.

Compatibility constraints remain explicit:

- Every helper still prepares the coordinates before deciding whether to
  draw, so the record a caller receives is the same one the out-parameter
  carried, including off screen.
- The fish and Pikit routines drew twice into a record they never read
  again; those calls now discard the return value.
- Four `PrepOamCoordsRet` imports that only the out-parameters used are
  gone.

## Verification

The sprite draw tests (124 in the focused set) pass unchanged. The
ownership scanner is unchanged. The library compiles with no warnings in
the parity, dev, and lib-test builds; readability and projection discovery
pass. All 1,721 library tests pass under the dev profile.

The main tree validated this batch stacked with the batches before and after
it, on parity binary
`780f0036f42b0911911012627006783dabf7c730e8637f4dc27e32ba4d5aa813`: the
200,000-frame cached comparison matched every video and audio hash in
319.96 seconds, the frame 60000 and 150470 WRAM goldens match, the
200,000-frame WRAM endpoint is the recorded `dd45975c…` image, and all
1,721 library tests pass under the parity profile.
