# Door animation step ownership

The original keeps one door animation step (`door_animation_step_indicator`,
WRAM 0x690) and uses it for both the dungeon's door open and close animation
and the overworld's big entrance doors, whose four-tile opening animation
the overworld drives through the dungeon door counters. The port modelled
the word twice: `DungeonDoorState` held it for the dungeon's 25 sites, and
`WorldTransientState` held a second copy for the overworld's big-door
modules. Both were write-through, so each side grew a repair: every one of
the transient's 30 setters re-read the word from RAM before syncing so its
coherence check would not fail after a dungeon door write, and the door
bridge re-read the word from RAM before each of its 23 syncs so it would not
re-stamp a stale step over the transient's write. A step set through the
dungeon owner was still invisible to the overworld's big-door module until
the next full import, because that module read the transient's copy.

`DungeonDoorState` is the single owner. The overworld's big-door modules and
the 32x32 door update read and write it; the transient's field, accessors,
setters, test-only flush, the adopt-live-then-sync helper, and both
`ZeldaState` wrappers are gone, and the transient's setters sync plainly.
The door bridge's reload-before-sync is gone too, since nothing else writes
the word. Because the transient is serialized in checkpoints, the checkpoint
magic moved from `Z3RSPC15` to `Z3RSPC16`.

With one owner the overworld's 32x32 door update and the player module's
smash-door copy of it became identical, so this batch also retires the
copies that the port-time duplicates batch left behind: the smash module's
map16 draw, persist, VRAM address, and 32x32 update (and its duplicate
door-tile table), and the sprite draw module's map16 draw, persist, and VRAM
address helpers, whose hand-written packet was the canonical packet writer
word for word. `CreatePyramidHole` and `OpenGargoylesDomain` move to the
overworld module under their original names; the bat crash and the Thieves'
Town grate call them there.

`runtime_door_step_owner.rs` sets the step to 3 through the dungeon door
owner and runs the overworld's big-door-from-exiting module; on the
preceding commit the module read the transient's stale zero and kept
animating instead of advancing.

Compatibility constraints remain explicit:

- The 32x32 door update still writes the four memorized tiles, the VRAM
  packets, the terminator, the step increment (2 at counter 32, else 1), the
  VRAM load mode, and the counter increment in the original order.
- The overworld's three clears of the step still clear only its low byte.
- The pyramid hole, the smash door, and the draw module's map16 writers
  produce the same VRAM packets as before.

## Verification

`runtime_door_step_owner.rs` sets the step through the dungeon door owner and
runs the overworld's big-door-from-exiting module, which must advance; on the
preceding commit it reads the transient's stale zero and takes the animation
branch instead. Readability and projection discovery pass; the scanner
reports zero HIGH RISK overlaps, 13 bridge-sync overlaps, and 61 overlapping
bytes, unchanged.
