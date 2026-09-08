# Native player transitions

Player initialization, action resets, recoil landing, sword/item cancellation,
and quiet cape removal use native operations in `player/transitions.rs`.
The compatibility bridge publishes their cartridge byte sequences. Shared
item-action and grabbing resets are composed once in native code and once in
the codec; initialization and cancellation retain their distinct masks and
partial-byte behavior. The former A/B/C field-helper names are replaced by
names describing the state they reset. ROM-facing gameplay entry names remain.

## Observation boundaries

| Uninterrupted sequence | Previous player imports | Current imports |
| --- | ---: | ---: |
| Recoil landing after layer handling | 10 | 1 |
| Sword/item cancellation | 6 | 1 |
| Quiet cape removal | 4 | 1 |
| Initialization tail after swimming reset | 11 | 1 |
| Reset C including sword/item cancellation | 7 | 1 |
| Clearing both swim stroke counters | 2 | 1 |

These counts overlap: initialization includes cape removal and cancellation.
They describe source-level imports, not a measured frame-rate improvement.

Layer handling still precedes the recoil transition. Swimming acceleration
still publishes each axis before the player stroke counters. Initialization
retains its swimming boundary and subsequent feature, palette, and inventory
checks. Reset A still invokes minigame, follower, and tile-detection owners in
order; reset C still clears custom spell animation before importing the player.
The incoming native reads, external calls, and compatibility write order remain
at their original boundaries. Entry imports still observe legacy writes.

## Removed repair dependency

The old reset C bridge cleared `LINK_ELECTROCUTE_ON_TOUCH`, `LINK_CAPE_MODE`,
and `RELATED_TO_HOOKSHOT` in RAM while leaving their native action fields stale.
The following cancellation entry reloaded the whole player and silently repaired
those three fields. `reset_action_state` now updates the owning native fields
directly, then publishes the original reset and cancellation byte sequence.
This is a missing native mutation repaired at the existing owner, with no
extra WRAM write. A regression verifies native/RAM equality immediately after
the transition, without a repairing import.

Reset A still clears only the temporary bunny timer's low byte. Initialization
still clears only Z's high byte. Masked input, direction, and defense writes
retain unrelated bits. No persistent state layout, checkpoint version, shared
storage owner, or projection order changes.

## Validation

`player-transitions-de9235e8.txt` was captured before runtime edits. It freezes
288 cases across nine complete gameplay entry points, hashing full WRAM and
the resulting native projection. Cases include nonzero initial bytes and stale
native state at entry. The committed test has no fixture regeneration path.

All 1,757 library tests pass, with two existing ignored tests. RAM readability
and projection-discovery checks pass. The ownership scanner reports the same
114 projection writers, 86 reachable writers, and 40 existing overlapping bytes.
A source comparison expanded shared codec helpers and confirmed identical
ordered publication statements for all ten changed transition sequences.

Baseline `de9235e8` rebuilt to binary SHA-256
`d666972564c99f92ba2f73e6fd4bb6b787a7972bdf083e769be6261607d3f4c4`,
identical to the preceding full-validated player-motion binary. Its preserved
200,000-frame and full-route WRAM endpoints are the comparison baseline. A
fresh 180-frame live Snes9x baseline also passed before runtime changes.

The completed batch matched 200,000 consecutive cached Snes9x audio/video
frames from frame zero in 302.31 seconds. Both reached WRAM goldens and the
entire 131,072-byte final RAM image match the preceding player-motion build.
Candidate binary SHA-256:
`1a2abf0ad04d6b276c941d3d4426448626e28b2ee768c2ce960d9aa2f81f022c`.

Source commit `bf88197c2d457712d76bcf5dd954d92b94ce7c01` passed the full cold
cached Snes9x gate in 2442.50 seconds. All 1,581,079 consecutive frames matched
exact audio and video from frame zero, without a frame limit or checkpoint
resume and with no reported RNG drift. All four WRAM goldens and the entire
final 131,072-byte image match the preceding promoted player-motion binary.
Final WRAM SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.

The validated binary is the candidate hash above. The source commit's normal
hook also passed the standalone 500-frame smoke and a fresh 180-frame live
Snes9x comparison. The full-route proof uses the immutable cached oracle;
the promoted receipt is
`routes/full_run/receipts/native-transitions-full-av.manifest.json`.
