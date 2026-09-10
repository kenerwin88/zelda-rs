# Water HDMA window ownership

The original keeps six words at 0x680..0x68a (`water_hdma_var0..5`) for the
water HDMA window: its center, its two radii, the y target the swamp drain
animates toward, and the alternate y radius the flood uses. Two scenes write
them, the swamp palace drain in module 7 and the dam flood, and the room
draw seeds them for water rooms. The port modelled the same words twice:
`WaterHdmaWindowState` in the display (center and radii, master-projected
every frame, write-through setters for the watergate scene) and
`DungeonEnvironmentState` (radii, target, alternate radius, and center,
re-projected by its bridge from every environment setter). Neither updated
the other, so the HDMA window adjuster read the two radii from raw RAM with
comments explaining that the display model was stale during the swamp scene
(the f606612 and f606748 one-pixel fixes), and any environment setter could
re-stamp its stale radii over the watergate scene's live values.

The display's water window is the single owner of all six words. It gained
the y target and alternate y radius, word setters for the radii, and a
positioned-window setter that writes the center x then y as the original
does. The swamp drain, the flood, and the water room draws read and write it
through the owner; the adjuster's raw reads and their comments are gone. The
six environment fields, their accessors, setters, forwarders, and the
environment test that exercised them are removed. Because both states are
serialized in checkpoints, the checkpoint magic moved from `Z3RSPC14` to
`Z3RSPC15`.

`runtime_water_hdma_window.rs` sets a radius through the owner, then fires
an unrelated environment setter and checks the radius survives; on the
preceding commit the environment's bridge re-stamps zero over it.

Compatibility constraints remain explicit:

- The swamp drain still compares the y radius with the y target each step
  and the flood still combines the x radius and alternate y radius as before;
  only the owner they consult changed.
- The room draw still writes the radii and target before the window center,
  in the original order.
- The watergate scene's byte-wide y radius writes are unchanged.
- The y target and alternate y radius stay out of the master projection,
  as they always were (the environment only wrote them through its bridge);
  their setters write through. The projected byte set is unchanged, which
  the frozen player-collision and tile-behavior fixtures confirm.

## Verification

`runtime_water_hdma_window.rs` sets a radius through the owner, fires an
unrelated environment setter, runs the master projection, and checks the
radius at each step; the same scenario fails on the preceding commit, where
the environment's bridge re-stamps zero. Readability and projection
discovery pass; the scanner reports zero HIGH RISK overlaps, 13 bridge-sync
overlaps (down from 14), and 61 overlapping bytes (down from 69).

All 1,773 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`b6230379904fa447d2b2e9f25a789d34ab8a74aed0e708e2c5f2f10b8c1616df`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 310.95 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `282a4a1f` passed the full cold route: 1,581,079 consecutive
exact audio/video frames in about 2,465 seconds (41.1 minutes), starting at
frame zero with no frame limit or checkpoint resume. The comparison used the
immutable Snes9x oracle cache; it did not reload the live core. No RNG drift
was reported. The source commit skipped the commit hook at the user's
request; this full-route pass is the gate.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/water-full-av.manifest.json`.
