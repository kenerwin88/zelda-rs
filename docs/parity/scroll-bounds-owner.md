# Camera boundary ownership

`RoomBoundsState` owns the camera boundary words at 0x600..0x606 (the
original's `ow_scroll_vars0`) and projects them. `WorldScrollState` also
loaded three of them (the x start and end and the y end), excluded them
from its coherence check, carried test-only setters for them, and exposed
them to the renderer's side-space computation. That computation re-imports
the whole native state before reading, so the mirror was never stale in
practice; it was simply a second model of bytes it did not own, with the
usual exclusion to hide that.

The three mirror fields, their imports, getters, setters, and the
coherence exclusion are removed, along with the unit test that only
verified the exclusion. The side-space computation reads the room bounds
owner. Because the scroll state is serialized in checkpoints, the
checkpoint magic moved from `Z3RSPC11` to `Z3RSPC12`.

Compatibility constraints remain explicit:

- The side-space margins still derive from the same three words (left
  from 0x604, right from 0x606, bottom from 0x602) against the BG2 scroll.
- The scroll receipt decoder still reads those addresses from RAM.

## Verification

`runtime_scroll_bounds.rs` sets the bounds through the owner's bridge and
checks the resulting side-space margins. The ownership scanner reports one
same-mode overlap and 34 overlapping bytes, down from 35; readability and
projection discovery pass.

All 1,770 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`842d219ed30d32a055aecd3237daadd31c9678fad7d084c4351f2dd7bbacd8e7`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 312.66 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `ec8934ed` passed the full cold route: 1,581,079 consecutive
exact audio/video frames in 2,538.84 seconds (42.3 minutes), starting at
frame zero with no frame limit or checkpoint resume. The comparison used the
immutable Snes9x oracle cache; it did not reload the live core. No RNG drift
was reported. The source commit skipped the commit hook at the user's
request; this full-route pass is the gate.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/bounds-full-av.manifest.json`.
