# The Link bridge's plain setters are macro forwarders

Once the Link bridge published by diff, 363 of its setters had the shape
every other adopting bridge writes with `forward_synced!`: one state call
with the arguments passed through, then `sync`. The macro only accepted a
single field name, and the Link state is composed of five components
(`movement`, `actions`, `presentation`, `input`, and the state itself), so
it gained a two-level path arm. The 363 setters are now five
`forward_synced!` blocks, one per component: 189 movement, 101 action, 49
presentation, 10 input, and 14 on the state itself. Documented setters and
setters with any other body (a computed argument, two state calls, a
conditional) are unchanged.

The file is 1,585 lines, down from 3,736 before the diff publication.

Compatibility constraints remain explicit:

- Each forwarder calls the same state method with the same arguments and
  syncs once, as the hand-written setter did.
- The macro expansion for the single-field form is unchanged, so the 84
  plain bridges are unaffected.

## Verification

The 146 Link runtime tests in the focused set and all 1,721 library tests
pass under the dev profile. The ownership scanner is unchanged. The library
compiles with no warnings in the parity, dev, and lib-test builds;
readability and projection discovery pass.
