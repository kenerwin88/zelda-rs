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

All 1,775 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`305ff910edef5e8d0433ea67344a8557c47bde278373b61df7b160b435070d0a`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 309.44 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `32afc708` passed the full cold route: 1,581,079 consecutive
exact audio/video frames in about 2456 seconds (40.9 minutes), starting at
frame zero with no frame limit or checkpoint resume. The comparison used the
immutable Snes9x oracle cache; it did not reload the live core. No RNG drift
was reported. The source commit skipped the commit hook at the user's
request; this full-route pass is the gate.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/f4-full-av.manifest.json`.
