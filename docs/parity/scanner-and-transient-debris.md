# Ownership scanner comparison fix and transient debris

`find_dual_ownership.py` counted `ram[X] == 0` as a write of `X`, because
its scalar and indexed write patterns matched any `=` after the index. The
display projection gates the star-tile restore phase on
`ram[PLAYER_IS_INDOORS] == 0`, so the scanner reported the indoors flag as
owned by both the display and the world location state. The two patterns
now require an assignment that is not a comparison. With that and the
preceding owner batches, the scanner reports zero same-mode overlaps
(down from five at the start of the day) and 33 overlapping bytes, all
cross-mode SNES reuse.

The world transient state also carried two vestigial fields, the HUD
super-bomb indicator timer (0x4b4) and the floor-changed timer (0x4a0),
that were never loaded from RAM, never projected, always zero, and existed
only so the coherence checker would not compare them. Both bytes belong to
the display's HUD tilemap runtime, which every reader and writer already
used; the transient-level clear was a no-op on the transient. The fields,
their comments, and the no-op are removed. Because the transient is
serialized in checkpoints, the checkpoint magic moved from `Z3RSPC12` to
`Z3RSPC13`.

Compatibility constraints remain explicit:

- No WRAM projection changed; the removed fields never reached RAM.
- The scanner's other write patterns (word writes, slices, helper spans)
  are unchanged.

## Verification

Readability and projection discovery pass; the scanner reports zero
same-mode overlaps and 33 overlapping bytes.

All 1,770 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`b436fc02abba91df657d2d41ea6871c057bd67fdc92d8f69692ea73d45e123df`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 308.84 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.
