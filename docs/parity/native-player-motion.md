# Native player motion

Player coordinate integration now runs on `PlayerPosition`, a native coordinate
and fractional byte, in `game_state/native/player/motion.rs`. Signed velocity
is scaled by sixteen; signed subpixel deltas carry into wrapping 16-bit
coordinates. Full movement and interrupted movement share this arithmetic.
The former helpers that calculated positions directly inside WRAM are removed.

Gameplay movement and its native return values use `PlayerAxis::{X, Y, Z}`.
The ROM's numeric passes are decoded at the timing boundary and encoded only
when retaining a compatibility scheduler continuation. Those scheduler fields
keep their existing representation. The interrupted loop uses static axis
slices instead of allocating a vector, retaining Z/X/Y order when airborne
and X/Y order otherwise. Ordinary uninterrupted movement retains X/Y/Z order.

## Publication and ownership

The player compatibility bridge reads native coordinates and X/Y fractions,
advances a native position, and publishes the exact affected bytes. It no
longer reads those values back from RAM to reconstruct the result. Suspension
still exposes each original boundary: fraction, coordinate low byte, and
coordinate high byte. The low-byte stage keeps the old high byte in native
state and retains the computed high byte in its continuation. Resuming the
high-byte stage preserves the low byte observed on reentry.

Z's fractional byte at `$002c` also serves as the attract throne-fade timer.
It remains shared storage, supplied to the native position for each operation
and written back only when movement changes the fraction. No persistent Z
fraction field or new projection is introduced. Existing native fields,
projection order, saves, and Rust checkpoint layout `Z3RSPC07` are unchanged.

The uninterrupted coordinate sequence now holds one player bridge across its
X/Y/Z mutations. Safe-return publication first observes legacy writes; the
movement inputs are then captured before the coordinate sequence. Coordinate
writes cannot change velocity or airborne state. The sequence removes one
whole-player reload on the ground and two in the air. Sand drag retains its
original pre-import input reads and Y/X/velocity write order within one bridge,
removing two more reloads. The uninterrupted fraction-to-low-byte prefix also
uses one bridge instead of two. None of these sequences invokes an external
observer between the consolidated operations.

Entry and resume observation remains necessary for legacy writers and mode
aliases. This change removes proven redundant reloads; removing the remaining
entry reload requires a separate migration of those writers.

## Regression evidence

`player-motion-69183423.txt` was captured against the old byte-based movement
before replacement. Its 32 deterministic cases hash all 131,072 WRAM bytes
after full and partial movement, return values, intervening writes during
suspension, signed velocity and delta extremes, and native projection. The
committed test only compares against that fixture; it cannot regenerate it.
Native tests also check coordinate wrap, signed velocity scaling, and the
mixed-coordinate/high-byte continuation contract.

The baseline binary rebuilt at `69183423` has SHA-256
`414a96d1c9b9d583c0fd8ff7687bfc4b3364ae6397b17608281e06e7858568a8`,
identical to the preceding full-validated player-components binary. Its
existing 200,000-frame and full-route WRAM endpoints provide the comparison
baseline. A fresh 180-frame live Snes9x baseline also passed before changes.

The completed four-stage batch passes all 1,755 library tests (two existing
ignored tests), the RAM readability guard, and projection-discovery regression.
The ownership scanner retains the same 114 projection writers, 86 reachable
writers, and 40 existing potentially overlapping bytes.

The candidate matched 200,000 consecutive cached Snes9x audio/video frames
from frame zero in 307.31 seconds. Both reached WRAM goldens and the entire
131,072-byte endpoint match the preceding player-components binary. Candidate
binary SHA-256:
`d666972564c99f92ba2f73e6fd4bb6b787a7972bdf083e769be6261607d3f4c4`.
