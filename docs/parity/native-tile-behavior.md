# Native tile behavior classification

The tile detector decodes cartridge attributes into native behavior families
in `game_state/native/player/tile_behavior.rs`. The decoder covers all 256
values, including their indoor/outdoor meanings. Terrain, slopes, ledges,
stairs, hazards, liftables, chests, push blocks, and doors now have named
behavior handlers. The original attribute remains available for interaction
identity, and context-dependent chest, neighbor, platform, and hazard checks
stay at their existing read points.

`TileResult` centralizes native result accumulation, including byte truncation
and the shared key-lock/spike word. Its bridge publishes after each result,
preserving the old intermediate projections. Common solid interactions share
one Collision-then-Misc sequence. Chest handlers retain their distinct
Misc-then-identity-then-Collision sequence. Liftable indices use the original
attribute ordering, and door variants retain transition flags, forced movement,
direction parity, and dashability.

`reset_probe_results` owns the detection reset sequence. A source comparison
confirmed the same 27 ordered clear operations; each still publishes, and the
dungeon moving-floor clear remains last. Scratch coordinates and interaction
identity retain their existing lifetime. Key-lock clearing affects only the
low byte, followed later by the spike high-byte clear.

The old attribute switch and liftable reverse lookup are removed. The compiler
identified 24 superseded bridge forwarding methods and 13 unused native methods,
which were removed. No new persistent result state, projection range, state
owner, serialization layout, or timing continuation is introduced.

## Regression evidence

`tile-behavior-7ad23477.txt` was captured before runtime edits (re-frozen
once as `tile-behavior-zero-page-scratch.txt`; see
`zero-page-scratch-owner.md`). It freezes full
WRAM and native-projection hashes for 131,072 attribute executions:

- All 256 attribute values and both indoor/outdoor contexts.
- Eight probe masks: zero, each individual footprint bit, combined bits,
  a shifted nibble, and the full word.
- Thirty-two combinations of menu blocking, save-state hazard blocking,
  Somaria platform state, chest/neighbor state, and walk-through-walls mode.
- Randomized nonzero initial RAM, preserving visibility of unrelated bytes
  and accumulated results. Chest records straddle the 0x8000 threshold;
  the neighboring attribute both matches and differs.

Another 32 frozen cases cover the complete detection reset. The committed
test has no fixture regeneration path. Existing player-collision regressions
also exercise repeated probes, both axis orders, and the full cardinal handler.

The baseline rebuilt to the preceding promoted collision binary, SHA-256
`8c87791df7c00151d4d483097333070ff733092d20aa6248afe8995f0d43cc1b`.
A fresh 180-frame live Snes9x baseline passed before runtime changes. The
preceding promoted run's 200,000-frame and full-route WRAM endpoints are the
byte-for-byte comparison baselines.

All 1,762 library tests pass, with two existing ignored tests and no compiler
warnings. The exhaustive attribute test makes the suite take about 106 seconds.
RAM readability and projection-discovery checks pass. Ownership scanner output
is identical before and after: 114 projection writers, 86 reachable writers,
and 40 existing overlapping bytes.

The completed batch matched 200,000 consecutive cached Snes9x audio/video
frames from frame zero in 314.63 seconds. Both reached WRAM goldens and the
entire 131,072-byte final image match the preceding promoted collision build.
Candidate binary SHA-256:
`da8d26f66592110012c4959ecf5c7a790945f83c647388c200fe7213777195e4`.

Source commit `d979dddbd08b4365a96da63a08124be7d2e309e7` passed the full
cold route: 1,581,079 consecutive exact audio/video frames in 2,361.95 seconds
(39.4 minutes), from frame zero with no frame limit or checkpoint resume.
The comparison used the immutable Snes9x oracle cache; it did not reload the
live core. No RNG drift was reported. The normal source commit hook also
passed its 500-frame standalone and fresh 180-frame live Snes9x checks.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted collision
build, SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/tile-behavior-full-av.manifest.json`.
