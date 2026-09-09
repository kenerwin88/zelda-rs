# World transient duplicate owners

The ownership scanner reported five same-mode overlaps, four of them
between the world transient state and another projected owner. Three were
plain duplicate models of one byte or word:

- `hud_current_item_x` (0x656) duplicated the save progress HUD item slot.
  The transient had no setter, so after the save progress bridge wrote the
  slot, the transient's stale copy re-stamped it at the next master
  projection (the transient projects after the inventory).
- `travel_bird_flag` (0xaf4) duplicated the display's travel-bird tile
  offset, the owner the travel-bird code already had to target explicitly
  to survive the projection.
- `tilemap_layer_copy` (0x1c-0x1d) duplicated the display's main and sub
  screen layer masks. Every display layer setter mirrored its value into
  the transient, and the special-exit restore synced the display back from
  the transient by hand.

The fourth was an oversized write: the region state modelled
`which_entrance` as a word, so its projection spilled onto 0x10f, the
overworld hole-scan step owned by the transient. The original keeps
`which_entrance` as a byte; only the ending's scene table and the
starting-point entrance load store a word across both bytes.

The transient now keeps only the exit and special-exit layer-mask backups,
saved from and restored into the display owner. The HUD slot readers use the
save progress owner. The travel-bird byte has one owner. The entrance id is
a byte, and the two original word stores go through
`set_which_entrance_word`, which writes the entrance owner and the hole-scan
owner explicitly. The eight layer-mask mirror calls, the mirror helper, the
manual special-exit display sync, and the byte-merging entrance setter are
removed.

Because the transient and region states lost fields, the checkpoint magic
moved from `Z3RSPC10` to `Z3RSPC11`.

Compatibility constraints remain explicit:

- The layer-mask backups keep their words at 0xc102 and 0xc142; only the
  live copy changed owner.
- The starting-point entrance load and the ending's scene load still write
  the hole-scan byte, as the original word stores did.
- The remaining scanner overlap at 0x1b is a read-only mode gate in the
  display projection, not a second owner.

## Verification

`runtime_world_transient_owners.rs` checks that the HUD slot survives the
master projection (it did not before), that the travel-bird byte and the
layer masks have one owner with working exit and special-exit backups, and
that the ending's entrance word reaches both owners and survives projection.
The scanner now reports one same-mode overlap and 35 overlapping bytes,
down from five and 40; readability and projection discovery pass.

The HUD-slot proof test was also run against untouched main in a worktree
and failed there (the projected byte read 0 instead of 0x2b), confirming
the re-stamp.

All 1,770 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`a1ba89c7910c8a0f738391ba6fecb1dc990f6b3a3948d1d08d2cd9132440f43e`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 312.35 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.
