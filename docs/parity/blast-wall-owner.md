# Blast wall trigger ownership

The exploding dungeon wall keeps its state in the dialogue text buffer: the
original's `blastwall_var5..11` are the first 0x40 bytes of `messaging_buf`
at 0x10000, reused while no text is being drawn. The room tag that triggers
the wall stores the blast direction as a word at 0x1001c, then the center x
and y words at 0x1001a and 0x10018, and the blast-wall ancilla it spawns
reads them back the same frame. `EntranceEffectState` owns that block (the
entry state, the center, the direction, and the explosion slots), and the
ancilla mutates it through write-through bridges. The dungeon room-effects
state held a second copy of the three trigger words and of the first
explosion slot's phase, loaded from RAM but projected only while a wall was
already open, so the trigger's stores through it never reached RAM. The port
worked around that by writing the three words to raw RAM in a different
order (y, x, direction) and reloading the entrance-effects owner from RAM.

The entrance-effects owner now has write-through trigger setters, and the
room tag stores the direction, then the center, in the original order, then
spawns the ancilla; the raw writes and the reload are gone. The room-effects
copy (four fields, their load, gated projection, setters, forwarders, the
coherence exclusion that hid the gate, and the in-file test that only
exercised the gate) is removed, and the pull-switch handler that checked the
first explosion phase reads the owner's slot. Because the room-effects
state is serialized in checkpoints, the checkpoint magic moved from
`Z3RSPC16` to `Z3RSPC17`.

`runtime_blast_wall_owner.rs` stores the trigger words through the owner
and checks RAM and the native state, then fires an unrelated room-effects
setter and checks again; on the preceding commit the same stores through
the room-effects setters leave RAM at zero.

Compatibility constraints remain explicit:

- The direction is still stored as a word, clearing 0x1001d, as the
  original's `messaging_buf[0x1c / 2]` store did.
- The trigger sequence is direction, door tilemap offset, center x, center
  y, sound effect, crush-wall progress low byte, ancilla spawn, as in the
  original.
- The explosion slots, fragments, and fireballs keep their owner and
  bridges unchanged.

## Verification

`runtime_blast_wall_owner.rs` stores the trigger words through the owner and
checks RAM and the native state before and after an unrelated room-effects
setter; the same stores through the old room-effects setters leave RAM at
zero on the preceding commit. Readability and projection discovery pass; the
scanner reports zero HIGH RISK overlaps, 12 bridge-sync overlaps (down from
13, the 0x10000 pair), and 60 overlapping bytes (down from 61).

All 1,774 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`14c05f01ec0c0c8131fb563631876f23e578f65fa76eadc3efc8d6a709d809bc`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 315.02 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `53b87e39` passed the full cold route: 1,581,079 consecutive
exact audio/video frames in about 2450 seconds (40.8 minutes), starting at
frame zero with no frame limit or checkpoint resume. The comparison used the
immutable Snes9x oracle cache; it did not reload the live core. No RNG drift
was reported. The source commit skipped the commit hook at the user's
request; this full-route pass is the gate.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/blast-full-av.manifest.json`.
