# Native dungeon tile roles

Dungeon room logic used to recover meaning from the attribute byte at each
read point with mask arithmetic: `& 0xf0 == 0x70` for a tracked object,
`& 0xfc == 0x6c` for a curtain, `>= 0xf0` for a closed door, whole-word
compares such as `0x2323` for a switch cell, and a bit test for the
transition landing class. Those families are now a predecoded `DungeonRole`
on the shared tile definition, next to the player and entity behaviors:
in-room staircase kind, pressure plate and held switch variants, spiral,
straight, and wall-spiral staircase heads, stair landings with their index,
star switches with their toggle state, bombable floor, minigame chest,
curtain panels, tracked objects, open and closed doors with their slot,
Somaria pipes, and torches with their slot. The transition landing class is
decoded once for all 256 identities, since the original masked every tile,
not only doors. The push-block target acceptance list is decoded the same
way.

Nineteen read points in the dungeon module, the Somaria platform poof, the
push-block target probe, and the torch target now read the role. The room
tag switch probes share one search that requires a uniform 2x2 cell, as the
original whole-word compares did, and return the switch tile (or whether a
star switch is untoggled) instead of an out-parameter byte. The stair, star
switch, and wet-staircase recipes write named identities; the descending
landing sequence and per-stair identity increments were already native.

The torch logic's target byte is a native tile. It keeps any probed
identity (the original stored whatever the fire hit), so `targets_torch`
is the lighting test and `attr_index` remains the masked slot nibble.

Compatibility constraints remain explicit:

- `object_slot` exposes the low nibble for every identity, because the
  lift-and-replace, pot-reveal, and locked-door paths masked it without
  checking the family; the lift path keeps its original assertion, which
  also admits closed doors.
- The wet-staircase recipes write the wall/landing layer pairs in the
  original low-cell-first order.
- WRAM projections, checkpoint layout, the door attribute table, and the
  encoded liftable item codes are unchanged. `bg2_attr` remains for the
  map preview and overworld probe exports.

## Verification

`runtime_dungeon_roles.rs` writes out the original mask arithmetic per read
point and checks every attribute's role, slot nibble, transition landing
class, push-block acceptance, and statue switch identity against it, along
with the recipe words. Behavior tests cover the landing calculation through
the room state, the switch probes' uniform-cell requirement for plates, held
switches, and both star-switch states, and the torch target's byte and
lighting test.

All 1,765 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings. RAM readability, projection
discovery, and the ownership scanner pass with unchanged counts: 114
projection writers, 86 reachable writers, and 40 existing overlapping bytes.

Candidate binary SHA-256:
`3cf1cc7fdc97b1107b82299cc88f85e8dfaa030c444b2fea1a0ee7067987026c`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 324.91 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.
