# Native Somaria pipes

The Cane of Somaria platform and the pipe travellers were the last gameplay
consumers reading tile identities as bytes: the probe returned an encoded
attribute, the sprite's E slot stored it, and the junction handler switched
on `0xb2..0xbe`. The pipe identities are now a `PipeJunction` family on the
Somaria pipe role of the shared tile definition: straight segments, the two
zig-zag slopes, transit, the four T junctions, the no-back and question
transits, the endpoint, and the boundary identity that the platform poof
counts as pipe but the path search never stops on.

The sprite slot exposes the E byte as `somaria_pipe` for these travellers,
and the probe returns the tile. The path search, the platform's re-probe on
every eighth step, the junction handler, the pipe traveller's endpoint turn,
and its slope sound all read the junction. One original quirk stays
explicit: after an endpoint the pipe traveller parks its new direction in
the E slot, which imports as a non-pipe identity and forces the next probe
to differ.

Compatibility constraints remain explicit:

- The E slot keeps its byte projection and the parked direction.
- The platform poof still counts every pipe identity, including the
  boundary, when it looks for closed sides.
- The junction handler ignores straight segments and the boundary, as the
  original switch's default did.

## Verification

The dungeon role contract test now checks every pipe identity's junction
and the path-search rule against the original ranges. Existing Somaria and
pipe sprite tests pass unchanged.

All 1,765 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings. RAM readability, projection
discovery, and the ownership scanner pass with unchanged counts: 114
projection writers, 86 reachable writers, and 40 existing overlapping bytes.

Candidate binary SHA-256:
`08dfc0397ef1bda5ab846708e39537c552e567fcc4fa0c551f618b44320ea887`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 298.90 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `fa21ef90` passed the full cold route: 1,581,079 consecutive
exact audio/video frames in 2,394.13 seconds (39.9 minutes), starting at
frame zero with no frame limit or checkpoint resume. The comparison used the
immutable Snes9x oracle cache; it did not reload the live core. No RNG drift
was reported. The source commit skipped the commit hook at the user's
request; this full-route pass is the gate.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/pipes-full-av.manifest.json`.
