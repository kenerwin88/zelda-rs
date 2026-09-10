# Forwarder-shaped bridge setters use the macro crate-wide

After the diff-publication batches, 107 setters across five native
bridge files had the exact shape `forward_synced!` writes: one state call
with the arguments passed through, then `sync` (with the call's result
returned when there is one). They are now macro blocks grouped by state
path inside each bridge: 40 in the dungeon bridges, 38 in the display
bridges, 24 in the world bridges, 4 in the messaging bridges and 1 in the
follower bridge. Documented setters and setters with any other body are
unchanged. The five files lost 570 lines and gained 145.

Compatibility constraints remain explicit:

- Each forwarder calls the same state method with the same arguments and
  syncs once, as the hand-written setter did.
- The macro's single-field and two-level path arms expand to the same
  code the hand-written setters contained.

## Verification

The bridge tests in the focused set (673) and all 1,721 library tests
pass under the dev profile. The ownership scanner is unchanged. The library
compiles with no warnings in the parity, dev, and lib-test builds;
readability and projection discovery pass.
