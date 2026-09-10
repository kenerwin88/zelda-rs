# Room-draw helpers take the destination they use

The room object drawers in the original share the signature
`(src, uint16 *dst, dsto)`, and most of them advance `dst` for the next
object. The port kept that signature on eight helpers that do not: six
took a `_dst: &mut u16` they never read (the two type-1 subtype loaders,
the single pot, the single hammer peg, the bombable-floor helper and the
big gray segment), the bombable floor read the cursor but never advanced
it, and the rightward shelf end returned the same reference it was given.
Their callers declared a cursor solely to lend it, and two pot and two peg
loops advanced a cursor nothing read beside the position they did use.

Each helper now takes exactly the destination it uses. The one drawer
that does advance the cursor, the many-32x32-blocks routine, takes it by
value and returns the advanced cursor; its six callers assign the result
or pass the computed start directly. The dead cursor declarations and the
dead advances in the four loops are gone.

Compatibility constraints remain explicit:

- Every tile write still lands at the same destination: the helpers read
  `dsto` where they read `dsto` before, and the bombable floor's hole is
  placed at the cursor value its caller always passed, which equals `dsto`.
- The many-blocks routine advances the cursor by the same strides in the
  same order; only the way the caller receives it changed.

## Verification

The room-draw unit tests pass unchanged. The ownership scanner is
unchanged. The library compiles with no warnings in the parity, dev, and
lib-test builds; readability and projection discovery pass. All 1,721
library tests pass under the dev profile.
