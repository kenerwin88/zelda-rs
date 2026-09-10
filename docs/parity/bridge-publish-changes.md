# Bridges publish only what changed

A native state's bridge used to re-project the whole state on every setter:
`sync` called `write_to_ram` over live WRAM, so every field the state
held, changed or not, was stamped over whatever the byte held at that
moment. When another owner had written one of those bytes since the state
was last imported, the stale copy won. That is the mechanism behind the
stale-copy clobbers fixed this week, and it existed in 106 bridges across
795 forwarded setters plus their hand-written ones.

Each of those bridges now adopts its state from live WRAM when it is
constructed and, on every sync, projects the state through a compare-on-write
target (`DiffTarget`) that stores a byte only when its value differs from
what WRAM already holds. Because the state equals WRAM at construction, a
projected byte differs from WRAM exactly when the mutation changed it, so a
sync publishes the mutation's bytes and nothing else: a field the mutation
did not touch is never written and cannot clobber another owner's live byte.
No log or allocation is involved; the sync costs one projection, as the bulk
write did, plus the adoption read.

Reads inside projections (the indoors gate and the length clamps) come from
live WRAM at projection time, so both logs of one sync see the same gates.

The adoption is the same reload-before-mutate several bridges already
performed as individual fixes. WRAM is the only truth in the original, and the native state is
re-imported from it at every frame boundary anyway; adopting it at the
bridge makes the state coherent before the mutation, so the coherence
asserts hold by construction and a native reader after the bridge sees what
the original would have read. Adoption respects each bridge's documented
exclusions (the fields it keeps native-authoritative because their byte is
reused by another system in the current mode): the dungeon environment's
water counter, the memorized tiles indoors, and the sprite workspace's room
map outdoors. Production code never mutates a native field except through
its bridge, so nothing is lost by adopting.

`ProjectionLog`, which records the bytes a projection would write, stays
in the trait module with its replay test; it is the tool for offline
projection inspection and is not on any production path.

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

All 1,722 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`b647afc7d09441ee1bd71e385deea8ab755e4e96fab057b6cafce6b05e43758d`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 341.15 seconds (the preceding batches ran it in 310 to 318 seconds; the
difference is the adoption read at every bridge construction, a follow-up
profiling target). Both reached WRAM goldens match, and the complete
endpoint is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.
