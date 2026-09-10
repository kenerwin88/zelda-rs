# Spawn records carry the parent's height

`Sprite_SpawnDynamically` fills a spawn record from the parent (x, y, z, and
the overlord position) and `Sprite_SetSpawnedCoordinates` copies x, y, and z
from that record into the child, so a projectile or fragment spawned by an
airborne sprite starts at the parent's height. When the sprite handlers were
split into per-module files, each module got its own `Option`-returning
spawn adapter that returned only the record's x and y, and its own
coordinate adapter that rebuilt a record from those two words with z zero.
Ten spawn sites (the Master Sword's light well and fountain, Blind's head
and laser, Ganon's spiral bat and blue flame, the Helmasaur King's fireball,
Mothula's beams, the bush guard's foliage, and the yellow Stalfos head)
therefore placed the child on the floor regardless of the parent's height.
The full route is exact because every covered case has a grounded parent;
the divergence is latent.

`spawn_sprite_dynamically` and its `_ex` form are now the one Option-returning
port, returning the slot and the whole spawn record. Every former adapter
site destructures the record and reads its fields; the coordinate sites pass
the record to the canonical `sprite_set_spawned_coordinates`, which carries
z. The eleven spawn adapters and five coordinate adapters are removed.

`runtime_spawn_record_carry.rs` gives a bush guard a height of five and
spawns its foliage: the child's x, y, and z must equal the parent's. On the
preceding commit the child's z is zero.

Compatibility constraints remain explicit:

- The Stalfos-knight spawn that reuses the record's x and y even when no
  slot was free still reads zero for both in that case, as before.
- The mantle spawn still runs its slot-15 marker sequence around the spawn.
- Every other spawn site keeps its slot search start and its sequence of
  child writes; only the z reaching the child changes, and only when the
  parent is airborne.

## Verification

The library compiles with no warnings in the parity, dev, and lib-test
builds; readability and projection discovery pass; the scanner output is
unchanged (zero HIGH RISK overlaps, 12 bridge-sync overlaps, 60 overlapping
bytes). All 1,775 library tests pass under the dev profile, including the
new proof, which fails on the preceding commit with a child z of zero.
