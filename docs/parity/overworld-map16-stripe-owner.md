# Overworld map16 stripe ownership

The world transient state carried a 0x400-word mirror of WRAM 0x500..0xd00,
named after the dungeon replacement-tile table, that it never projected and
excluded from coherence outside the dungeon module. Only two things ever
used it: the overworld map16 stripe builders, which walk 32 words with
`d = (d + 1) & 0x1f`, and a redundant second copy of the door-animation
word at 0x690, which the transient already owns as a scalar with its own
write-through. The rest of the mirror aliased the sprite and overlord tables
and was copied from RAM on every import.

The transient now owns exactly the 32-word stripe window as
`overworld_map16_stripe`, write-through as before, and nothing else of that
bank. The door-animation alias and its index constant are gone; the scalar
remains the single owner. `DungeonObjectTrackingState` remains the owner of
the same window while the dungeon module is active, which the coherence
check still respects.

Because the transient is part of the serialized checkpoint, the checkpoint
magic moved from `Z3RSPC07` to `Z3RSPC08`; older `rust.z3state` files are
rejected with a clear message. No committed checkpoints exist.

Compatibility constraints remain explicit:

- The stripe builders still write each word straight to RAM through the
  bridge setter and read the window back by index.
- WRAM projection of every other transient field is unchanged.
- The unused length constant left in the address map was folded into the
  owner.

## Verification

The transient's load, projection, and bridge tests were retargeted to the
stripe window; the stripe-builder regression captures all 32 crossed-page
words unchanged. Ownership scanner counts are unchanged at 114 projection
writers, 86 reachable writers, and 40 existing overlapping bytes. RAM
readability and projection discovery pass.

All 1,765 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`7584c2c73fa0b2de9b08c8c6eec2036ba43f152635144db8c67527de1ceb736f`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 301.18 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `b9263a5d` passed the full cold route: 1,581,079 consecutive
exact audio/video frames in 2,566.82 seconds (42.8 minutes), starting at
frame zero with no frame limit or checkpoint resume. The comparison used the
immutable Snes9x oracle cache; it did not reload the live core. No RNG drift
was reported. The source commit skipped the commit hook at the user's
request; this full-route pass is the gate.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/stripe-full-av.manifest.json`.
