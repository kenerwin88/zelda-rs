# Port-time forwarders, third pass

The remaining multi-line `*_for_<module>` helpers were compared with the
canonical ports by hand. Six were copies: the runtime scroll module's camera
property cache (identical to the player's apart from calling a duplicate
doorway setter), the guard's spawn probe (identical body and byte-identical
velocity tables), the NPC module's distress draw (identical body and
tables), the Hinox shop's empty-bottle search, the small-boss module's
segmented-sprite initializer, and its plain OAM writer. All now call the
canonical ports and the copies and their tables are gone, as are the
transient's duplicate doorway setter and the `ZeldaState` wrapper for it.

Twenty-two more pure forwarders in the Blind, Mothula, and dungeon-NPC
modules, which the second pass's name filter had skipped, are retired the
same way,
and the two boolean tile-collision adapters became `!= 0` at their four
call sites. The small-boss module's private copy of `PrepOamCoordsRet` is
replaced by the sprite module's struct, so its shadow draw no longer
converts between two identical types. `Dungeon_DeleteRupeeTile` moves from
the player module to the dungeon module under its original name.

The adapters that remain change a signature or carry logic of their own
(`soldier_func12`, the Ganon head patch, the Ganon and small-boss
draw-table conversions, the guard's facing-point wrapper, the cucco subtype
continuation).

Compatibility constraints remain explicit:

- Every retired body was the canonical body or call with the same
  arguments; the spawn probe's 64-entry velocity tables were compared
  entry by entry.
- The small-boss shadow draw still passes a copy of the caller's record,
  so callers keep their own coordinates as before.
- No write order changed.

## Verification

The library compiles with no warnings in the parity, dev, and lib-test
builds; readability and projection discovery pass; the scanner output is
unchanged (zero HIGH RISK overlaps, 12 bridge-sync overlaps, 60 overlapping
bytes). All 1,775 library tests pass under the dev profile.

All 1,775 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`0e0660b9030f256c75accf6fda2b8054edf973c5acafa880bcde2d8491be1001`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 318.60 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `212052c4` passed the full cold route: 1,581,079 consecutive
exact audio/video frames in about 2461 seconds (41.0 minutes), starting at
frame zero with no frame limit or checkpoint resume. The comparison used the
immutable Snes9x oracle cache; it did not reload the live core. No RNG drift
was reported. The source commit skipped the commit hook at the user's
request; this full-route pass is the gate.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/dups3-full-av.manifest.json`.
