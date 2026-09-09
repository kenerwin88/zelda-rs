# Test-only production helpers

A scan of every `pub(crate) fn` in the non-test sources found 75 whose
only references were in tests: field setters and readers on native states
(the tile-detection result accumulators superseded by `TileResult`, item
and crystal setters, transient counters, save-progress predicates), a few
projection helpers that only round-trip tests call, and two engine seams
(`project_native_game_state_to_ram`, `bird_travel_destination`). The
crate's dead-code policy (`allow(dead_code)` outside tests, so the lib-test
build is the detector) could not flag them because the tests count as
users.

Each is now `#[cfg(test)]`, with the one type that only such a helper
returned gated the same way. Compiling the non-test library confirmed that
no production caller existed for any of them; the dead-code detector will
now report a helper as soon as its last test goes away. No behavior
changes; the parity binary still changes hash because fewer functions are
compiled.

Compatibility constraints remain explicit:

- No function body changed; only visibility to non-test builds.
- The three helpers that were already `#[cfg(test)]` are unchanged.

## Verification

The non-test library, the binary, and the lib-test build compile with no
warnings. Readability and projection discovery pass.

All 1,771 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`1bef436d2613fd68432f35193dcf14c9839cccf65bb5050657486cb0f35e8e02`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 309.99 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `75efe245` passed the full cold route: 1,581,079 consecutive
exact audio/video frames in 2,451.51 seconds (40.9 minutes), starting at
frame zero with no frame limit or checkpoint resume. The comparison used the
immutable Snes9x oracle cache; it did not reload the live core. No RNG drift
was reported. The source commit skipped the commit hook at the user's
request; this full-route pass is the gate.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/helpers-full-av.manifest.json`.
