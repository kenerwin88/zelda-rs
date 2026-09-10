# The Link bridge publishes by diff

The Link bridge (`NativeFollowerLinkBridgeMut`, the largest compatibility
boundary in the crate) already adopted the live WRAM at construction and
asserted whole-state coherence after every setter, but each of its 426
setters still published by hand: a state call, one or more raw stores of
the same value into WRAM, and the assertion. Six hundred and twenty-three
such stores encoded the WRAM layout a second time, next to the projection
that already encodes it once.

The bridge now has the same `sync` as every other adopting bridge: it
projects the state through the compare-on-write target and asserts
coherence. Every setter keeps its state call and calls `sync`; the
hand-written stores are gone. Because the state equalled WRAM before each
setter, the bytes the diff publishes are exactly the bytes the removed
stores wrote, in the same setter, with the same values.

Nine raw stores remain on purpose. They are writes the original performs
from Link code into bytes the Link state does not model: the exit
coordinates, the three Link DMA offsets, the Z subpixel, the reset ancilla
work byte, and two tile-detection scratch words. The ownership scanner's
foreign-write lint already knows them, and a first pass that dropped one
of them (the tile-detection scratch word, whose address another state
projects) was caught by restricting the rewrite to the Link state's own
projection tree.

Three accessors that only served the removed stores are gone from the
movement state. The file shrank from 3,736 to 3,116 lines.

Compatibility constraints remain explicit:

- Partial stores (a low byte, a mirror word) publish only the bytes the
  state call changed, as the removed stores did.
- The foreign stores keep their positions relative to the state calls.
- Adoption at construction and the whole-state assertion are unchanged.

## Verification

The 146 Link runtime tests in the focused set and all 1,721 library tests
pass under the dev profile. The ownership scanner is unchanged. The library
compiles with no warnings in the parity, dev, and lib-test builds;
readability and projection discovery pass.

The main tree validated this batch on parity binary
`bc32ad49de8afd96b2a6211dd3fc4387a0ea36a8cecb776de2abb6211f39581d`: the
200,000-frame cached comparison matched every video and audio hash in
318.06 seconds (311 seconds for the batch before it on a quieter machine),
the frame 60000 and 150470 WRAM goldens match, the 200,000-frame WRAM
endpoint is the recorded `dd45975c…` image, and all 1,721 library tests
pass under the parity profile.

The full cold route on the same binary matched all 1,581,079 frames of video
and audio in 2,542 seconds, with the four WRAM goldens and the full-route
WRAM endpoint (`31619379…`) unchanged; the run is promoted in
`routes/full_run/parity-frontier.json`.
