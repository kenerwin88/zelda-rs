# Port-time forwarders, fourth pass

Four helpers outside the `_for_<module>` naming that the earlier scans keyed
on were the same shape: the prep module's verbatim copy of the 16-bit
coordinate fetch, the ending module's forwarders for that fetch and for the
sprite main dispatch, and the NPC module's forwarder for the inactive-sprite
return. All fifteen call sites use the canonical ports and the copies are
gone.

`find_dual_ownership.py` now tags the mirror-warp state as overworld-only.
Its two scratch words at 0x6a0 and 0x6b0 alias the dungeon parser's
star-switch tile and the inter-room staircase list, which are live only in
dungeons; the scanner had listed both pairs as same-mode bridge-sync
overlaps. They now classify as cross-mode SNES reuse, leaving 10
bridge-sync overlaps: the mode-gated torch phase and the spell effects'
shared scratch bank.

Compatibility constraints remain explicit:

- Every removed body was the canonical body or call with the same
  arguments; no write order changed.
- The scanner change affects classification only; the projection code is
  untouched.

## Verification

The library compiles with no warnings in the parity, dev, and lib-test
builds; readability and projection discovery pass; the scanner reports zero
HIGH RISK overlaps, 10 bridge-sync overlaps (down from 12), and 60
overlapping bytes. All 1,775 library tests pass under the dev profile.
