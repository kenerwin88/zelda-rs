# Dungeon entrance backup ownership

`DungeonEntranceBackupState` owns the four bytes the original saves at
`Dungeon_LoadEntrance` (0xc164..0xc167: the overworld, main, and aux tile
theme indices and the sprite graphics index) and projects them. The world
palette theme state and the sprite system state each also loaded those
bytes into load-only mirror fields, never projected them, excluded them
from their coherence checks, and read the mirrors when the overworld
restored the themes on leaving the dungeon. A mirror refreshed only by a
full import is correct only while a full import happens between the save
and the restore; the restore itself cannot tell.

The two restores now take the owner's values as arguments, and the
overworld's cached-entrance load passes them from the entrance backup. The
four mirror fields, their imports, and both coherence exclusions are
removed. `runtime_entrance_backup.rs` drives the real save through the
owner's bridge, changes the live themes without a re-import, and checks
that the restore reproduces the saved values in both the native state and
RAM; on the previous code the restore read stale mirrors.

Because two serialized structs lost fields, the checkpoint magic moved from
`Z3RSPC09` to `Z3RSPC10`.

Compatibility constraints remain explicit:

- The restore order and the bytes written (the 0xaa0 theme indices, then the sprite
  graphics index) are unchanged.
- The special-exit save still lives in the palette theme state, as before;
  only the dungeon entrance backup changes owner.
- The projection of every other field is unchanged.

## Verification

All 1,767 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings. RAM readability, projection
discovery, and the ownership scanner pass with unchanged counts: 114
projection writers, 86 reachable writers, and 40 existing overlapping bytes.

Candidate binary SHA-256:
`ffcd8158ad72876e373034c19c4f1676aa40c63fad7f7251487e1add63d743ba`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 305.94 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.
