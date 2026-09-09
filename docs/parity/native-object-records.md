# Native dungeon object records

Each dungeon object slot carries a replacement record word. The room draw
registers what the slot holds (a liftable kind, one segment of a big gray
rock or a bombable floor, a hammer peg, an idle push block), and the
push-block handler then counts the same word through its phases. Twenty
read and write points decoded that word with masks: `& 0xf0f0 == 0x1010`
for a liftable, `== 0x2020` for a rock segment, `== 0x4040` for a peg,
`& 0x00f0 == 0x0030` for a floor segment at attribute load, `& 0x0f` for
the kind or segment, and bare integers 1 through 5 and 0xffff for the
push-block phases with `+ 1` and a low-byte clear as transitions.

`ObjectRecord` now owns that word. Constructors produce the original words
for each object kind and the named push phases (idle, pushed, sliding,
arrived, falling, resting on a plate, vanished). Queries keep the
original's masks exactly for every one of the 65,536 words, including the
attribute loader's low-byte-only test and the kind nibble read without a
family check. The two transitions are named: `advanced` is the original
increment, which wraps a vanished block back to idle, and `settled` is the
low-byte clear at the end of the falling animation. The tracking state
stores the same `u16` array, so WRAM projection and checkpoint layout are
unchanged and no magic bump is needed.

Compatibility constraints remain explicit:

- The room draw registration order and the misc-object index arithmetic
  are untouched; only the registered value is typed.
- The push-block handler keeps its original sequence: a landing rewrites
  the record before the advance that follows the arrival check.
- The liftable item code table is still indexed by the kind nibble.

## Verification

`runtime_object_records.rs` checks the constructors' words and every query
and transition against the original masks over all 65,536 words.
Readability, projection discovery, and the ownership scanner pass with
zero same-mode overlaps and 33 overlapping bytes.

All 1,771 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`f2ddacc4a3c457455fe2146b24897b12c9b79cfd31d0207bc229e90b3f6303bf`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 310.58 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `a98ccd6e` passed the full cold route: 1,581,079 consecutive
exact audio/video frames in 2,500.93 seconds (41.7 minutes), starting at
frame zero with no frame limit or checkpoint resume. The comparison used the
immutable Snes9x oracle cache; it did not reload the live core. No RNG drift
was reported. The source commit skipped the commit hook at the user's
request; this full-route pass is the gate.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/records-full-av.manifest.json`.
