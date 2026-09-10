# Bridges publish only what changed

A native state's bridge used to re-project the whole state on every setter:
`sync` called `write_to_ram` over live WRAM, so every field the state
held, changed or not, was stamped over whatever the byte held at that
moment. When another owner had written one of those bytes since the state
was last imported, the stale copy won. That is the mechanism behind the
stale-copy clobbers fixed this week, and it existed in 106 bridges across
795 forwarded setters plus their hand-written ones.

Each of those bridges now records its state's projection into a log when
it is constructed (`capture`), and on every sync projects the state again
and publishes only the bytes whose value differs from the recorded log,
then keeps the new log as the baseline (`publish_changes`). A field the
mutation did not touch is never written, so it cannot clobber another
owner's live byte, and a bridge that finds its state already equal to WRAM
writes nothing. When a mode gate opens or closes between the two
projections the address sets differ; the publication then falls back to an
address-keyed comparison so newly gated-in bytes are written and nothing
else is.

Reads inside projections (the indoors gate and the length clamps) come from
live WRAM at projection time, so both logs of one sync see the same gates.

A bridge also adopts live WRAM for its state when it is constructed, the
same reload-before-mutate several bridges already performed as individual
fixes. WRAM is the only truth in the original, and the native state is
re-imported from it at every frame boundary anyway; adopting it at the
bridge makes the state coherent before the mutation, so the coherence
asserts hold by construction and a native reader after the bridge sees what
the original would have read. Adoption respects each bridge's documented
exclusions (the fields it keeps native-authoritative because their byte is
reused by another system in the current mode): the dungeon environment's
water counter, the memorized tiles indoors, and the sprite workspace's room
map outdoors. Production code never mutates a native field except through
its bridge, so nothing is lost by adopting.

The 56 unit tests named `*_projects_native_state_over_stale_ram` asserted
the retired re-stamp (seed WRAM with one value, the state with another,
mutate a third field, expect the state's stale values stamped over WRAM).
They are replaced by one contract test in `ram_target.rs`: a bridge adopts
live WRAM and publishes only the bytes its mutation changed. Two spotlight
tests now seed WRAM instead of the native state, and one dialogue test
seeds its saved module through the frame bridge. Helpers that only those
tests used are removed.

This changes behavior only where a bridge previously re-stamped a stale
value over a foreign live write, which is the bug class, never a correct
path: for a state coherent with WRAM the published bytes are exactly the
bytes the mutation changed, with the same values the bulk projection would
have written. The frame-level master projection is unchanged by this
batch.

Compatibility constraints remain explicit:

- Write-through bridges (those without a bulk `sync`) are untouched.
- The order of the bytes a sync publishes follows the projection's own
  order, as before; only unchanged bytes are omitted.
- The per-bridge coherence asserts are unchanged, and are now strict: a
  state that drifted from WRAM fails them instead of being papered over by
  its own re-stamp.

## Verification

The library compiles with no warnings in the parity, dev, and lib-test
builds; readability and projection discovery pass; the scanner reports one
HIGH RISK overlap (the zero-page scratch pair from the previous batch), 10
bridge-published overlaps (informational now), and 70 overlapping bytes.
All 1,722 library tests pass under the dev profile.
