# One macro for the adopting bridges

After the bridge change, 84 of the native bridges had become the same
twelve lines: a struct holding one state and WRAM, a constructor that
adopts the state from WRAM, a sync that projects through the
compare-on-write target and asserts coherence, and the plain coherence
assert. Each is now one `adopting_bridge!` invocation naming the bridge,
its field, and its state; the bridge's own setters stay in their `impl`
block beside it. The comments those constructors carried (the shared
ancilla scratch windows, the reserved save window) sit above the
invocation, and the eight bridges that had reloaded before mutating on
their own, and so carried a second adoption after the bridge change, lose
the duplicate.

The 31 bridges that differ (a nested state path, an exclusion in the
adoption or the assert, a second state, or no bulk sync at all) are
unchanged.

Compatibility constraints remain explicit:

- The macro expands to the same constructor, sync, and assert bodies the
  converted bridges had; no bridge changes what it adopts, publishes, or
  asserts.
- No projection, setter, or forwarder changed.

## Verification

`find_dual_ownership.py` reads `adopting_bridge!` invocations as bridge
owners, so its report is unchanged: zero HIGH RISK overlaps beyond the
known zero-page scratch pair, 10 bridge-published overlaps, 70 overlapping
bytes. The library compiles with no warnings in the parity, dev, and
lib-test builds; readability and projection discovery pass. All 1,722
library tests pass under the dev profile.
