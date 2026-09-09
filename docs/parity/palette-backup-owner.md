# Overworld palette backup ownership

The overworld/death palette backup at WRAM 0x1dd80 had two native models.
`PaletteBufferState.overworld_backup` is projected by the master projection
every frame. `PpuScrollCopyState.mapbak_palette` was a 512-byte shadow that
was written through to RAM by the game-over and mirror paths, excluded from
the coherence check, and read back by the game-over palette restores. The
two paths never updated each other, so after a game-over copy the next
master projection re-stamped the palette buffer's stale backup, usually
zeros, over RAM while the restores kept reading the shadow. Audio and video
parity could not see this because nothing reads that RAM window; the
Rust-vs-Rust WRAM goldens carried the same stale image on both sides.

`runtime_palette_backup.rs` reproduces the defect against the previous
code: copy a backup, run the master projection, and the RAM window is zero.
It passes on this change.

The palette buffer is now the single owner. The game-over fade and the
mirror-warp backup go through `backup_overworld_palette_from_tagged`, which
updates the owner, RAM, and the provenance mirror together; the restores
read `overworld_palette_backup`. The shadow field, its length constant, its
load, accessor, copy helpers, the coherence exclusion, and the
mirror-tagging shim that existed only to follow the shadow are removed.
This is bug class 1 (stale bulk projection over a write-through owner),
fixed by the sole-owner recipe.

Because the scroll copy is serialized in checkpoints, the checkpoint magic
moved from `Z3RSPC08` to `Z3RSPC09`.

Compatibility constraints remain explicit:

- The backup keeps the original 512-byte window; the game-over restore
  still copies the first 256 bytes into the visible aux bank, as before.
- The overworld's existing backup path is unchanged; it already used the
  owner.
- The fix changes the WRAM image at 0x1dd80 after the first game-over or
  mirror backup in a route, so the WRAM goldens and endpoint are expected to
  differ from the preceding promoted build only inside that window. Any
  other difference is a regression.

## Verification

All 1,766 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings. RAM readability, projection
discovery, and the ownership scanner pass with unchanged counts: 114
projection writers, 86 reachable writers, and 40 existing overlapping bytes.

Candidate binary SHA-256:
`a593778b0b6c296b3aff2a83bc6d9e97ce581eb49be05e062fd371a17a01eb3c`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 334.89 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build (no backup copy occurs in
that window), SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.
