# Native player collision probes

`game_state/native/player/collision.rs` owns the player probe geometry and
moving-floor collision-order decision. These operations take native positions,
directions, and velocities. They contain no WRAM addresses or persistent state.
`tile_detect.rs` masks pixel coordinates for tile queries and publishes the
existing scratch values before executing each ordered set of probes.

Four movement-probe builders now share one geometry calculation and execution
adapter. Body and mirror-clearance queries share named footprint corners.
Horizontal and vertical movement checks share one dispatcher, retaining their
distinct indoor/outdoor handlers. The two slope-order helpers are replaced by
an explicit `CollisionOrder`; the primary-layer pass chooses its order once
after applying moving-floor velocity.

## Preserved observation boundaries

- Axis and direction remain independent. Every formerly accepted direction
  works on either probe axis, including ledge callers using orthogonal offsets.
- Coordinate addition wraps at 16 bits before location masking. X becomes a
  tile column; Y remains the masked pixel coordinate expected by tile lookup.
- Cardinal Y publishes unmasked probe Y, then the masked high-side X column
  as the probe anchor. Cardinal X publishes center Y, then high-side Y into
  the legacy probe-X word. Slopes publish neither position word nor anchor.
- Cardinal samples accumulate bits 1/2/4 in low/center/high order. Slopes
  omit the center and use bits 1/2. Footprint samples retain top-left,
  bottom-left, top-right, bottom-right order and bits 8/2/4/1.
- Nearby probes reset detection before clearing the pit word; mirror probes
  clear the pit word before resetting. The existing repeated clear remains.
- Slope eligibility is read again after the first axis, because a slope can
  block the second axis during that first check.
- Moving-floor decisions retain exact byte comparisons before low-bit tests.
  The player velocity import/publication boundaries and tile-execution side
  effects are unchanged.

No state owner, bulk projection, checkpoint layout, timing continuation, or
indoor/outdoor collision rule changes. Tile classifications still accumulate
in their existing native owner; this extraction does not replace shared
compatibility scratch with a second stored result.

## Regression evidence

`player-collision-869fdb0c.txt` freezes 704 pre-refactor cases across eleven
runtime entry points. It hashes all WRAM plus the resulting native projection.
Probe cases start with nonzero randomized RAM; the cases cover all directions,
coordinate wrapping, both layers, doorways, zero/positive/negative movement,
slopes, and room collision modes. The test has no regeneration path.
The fixture was re-frozen once, as `player-collision-zero-page-scratch.txt`,
when the sprite workspace stopped projecting four zero-page bytes no state
owns; every case's WRAM and every owned projected byte were shown equal
before the re-freeze (see `zero-page-scratch-owner.md`).

Native geometry is compared against the pre-refactor formulas for every
16-bit coordinate value, both axes, all four directions, and both probe kinds.
The moving-floor decision is compared for every collision-mask byte and signed
floor-Y velocity, with zero/nonzero positive/negative player velocities.
A separate runtime regression compares both dispatch orders with the original
two-if sequence, including cases where the first axis changes second-axis
eligibility.

The pre-change binary was rebuilt and matched the preceding promoted binary:
`1a2abf0ad04d6b276c941d3d4426448626e28b2ee768c2ce960d9aa2f81f022c`.
A fresh 180-frame live Snes9x comparison passed before runtime edits. The
preceding promoted run's 200,000-frame and full-route WRAM endpoints are the
byte-for-byte comparison baselines.

All 1,761 library tests pass, with two existing ignored tests. The build has
no warnings. RAM readability and projection-discovery checks pass. The
ownership scanner output is identical before and after: 114 projection
writers, 86 reachable writers, and 40 existing overlapping bytes.

The completed batch matched 200,000 consecutive cached Snes9x audio/video
frames from frame zero in 296.47 seconds. Both reached WRAM goldens and the
entire 131,072-byte final image match the preceding promoted build.
Candidate binary SHA-256:
`8c87791df7c00151d4d483097333070ff733092d20aa6248afe8995f0d43cc1b`.

Source commit `ef110e9514850aeaaa61089414a271bfd3c7992f` passed the full cold
cached Snes9x gate in 2348.68 seconds. All 1,581,079 consecutive frames matched
exact audio and video from frame zero, without a frame limit or checkpoint
resume and with no reported RNG drift. All four WRAM goldens and the entire
final 131,072-byte image match the preceding promoted player-transitions build.
Final WRAM SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.

The validated binary is the candidate hash above. The source commit's normal
hook also passed the standalone 500-frame smoke and a fresh 180-frame live
Snes9x comparison. The full-route proof uses the immutable cached oracle;
the promoted receipt is
`routes/full_run/receipts/collision-full-av.manifest.json`.
