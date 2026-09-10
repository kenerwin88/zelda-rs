# Boss prize countdown ownership

WRAM 0x4c2 is one variable in the original (`byte_7E04C2`, unnamed even in
the decompilation): `RoomTag_GetHeartForPrize` stores 128 there when the
boss's heart container starts falling, and the falling-prize ancilla counts
it down every frame, swapping in the item graphics when it reaches 1. The
port modelled that byte twice under two guessed names. The world transient
held it as a "milestone item graphics countdown" and projected it every
frame; the dungeon room-effects state held it as a "moving wall torch update
flag", set it to 0x80 for the room tag, and re-projected it from every
room-effects setter. The prize ancilla read and decremented raw RAM to dodge
the transient's copy going stale, which left the room-effects copy stale
instead: any room-effects mutation in the same frame re-stamped the old
value over the live count. `runtime_boss_prize_countdown.rs` reproduces that
against the previous code (the byte reverts from 127 to 128) and passes now.

The world transient is the single owner, under the byte's real name. The
room tag begins the countdown and the prize ancilla reads and decrements it
through the owner; both mutations write only their byte rather than
re-projecting the transient. The room-effects field, its load, projection,
setter, and bridge forwarder are gone, along with the misnamed constant in
both constant maps and the ancilla-local copy. Because the room-effects
state is serialized in checkpoints, the checkpoint magic moved from
`Z3RSPC13` to `Z3RSPC14`.

## Scanner: bridge syncs are live writers

`find_dual_ownership.py` had reported this byte as safe because its
reachability walk followed only the master projection, and the dungeon
composite never projects the room-effects state. But every `*BridgeMut`
whose `sync` calls the state's `write_to_ram` re-projects that state on
each setter, mid-frame. The scanner now collects those bridge-synced owners
(25 states the master projection never reaches) and reports same-mode
overlaps involving them in a separate `BRIDGE-SYNC` section, so the
existing `HIGH RISK` count keeps its meaning. On the previous commit that
section named the 0x4c2 pair; on this one it lists 14 overlaps that were
invisible before:

- 0x4bc, the star-tile restore phase against the moving-wall torch blink
  phase (already mode-gated in the room-effects projection).
- 0x680..0x687, the water HDMA window, held by both the dungeon environment
  and the display's water HDMA state.
- 0x6a0 and 0x6b0, the mirror-warp target and reserved words against the
  dungeon parser's star-switch tile and the inter-room staircase list
  (overworld against dungeon, mis-tagged as one mode).
- 0x10000, the dungeon messaging buffer against the blast-wall phase.
- 0x15800..0x15812, the spell effect scratch bank shared by Bombos, Ether,
  Quake, the tower seal, the weather vane debris, and the angle scratch,
  which the original also reuses across those effects.

Those are a triage worklist for later batches, not part of this one.

Compatibility constraints remain explicit:

- The room tag still stores 128 before spawning the prize, and the ancilla
  still swaps graphics at 1 and decrements through 0, wrapping as before.
- The byte keeps its every-frame projection from the transient; only the
  second owner is removed.
- No other room-effects byte changed owner or projection.

## Verification

`runtime_boss_prize_countdown.rs` starts the countdown through the owner,
decrements it, fires a room-effects setter, runs the master projection, and
checks the byte at every step; the same scenario through the old API fails
on the preceding commit. Readability and projection discovery pass; the
scanner reports zero HIGH RISK overlaps, 14 bridge-sync overlaps, and 69
overlapping bytes (33 before the bridge-sync owners were counted).

All 1,772 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`637c2364f10181163bd41a9e99f89f42997d5eb04636b6c7badddb82fd957aac`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 304.47 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `7d15a220` (test-only follow-up `5fb83173`) passed the full
cold route: 1,581,079 consecutive exact audio/video frames in about 2,453
seconds (40.9 minutes), starting at frame zero with no frame limit or
checkpoint resume. The comparison used the immutable Snes9x oracle cache; it
did not reload the live core. No RNG drift was reported. The source commit
skipped the commit hook at the user's request; this full-route pass is the
gate.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/prize-full-av.manifest.json`.
