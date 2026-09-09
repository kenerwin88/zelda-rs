# Port-time forwarders and local duplicates

When the sprite handlers were split into per-family modules, each module
got private `*_for_<module>` shims for helpers that lived elsewhere, and a
few helpers were copied outright because their originals were private.
This batch removes the copies whose canonical port now exists and is
byte-for-byte the same:

- Eleven pure forwarders in the world module and four in the Ganon module
  (single-tile draws, OAM correction, light fountain, flute draws, the
  solicited message, the SP5F palette filters, trident draw, 16-bit
  coordinates, apply-speed-towards-Link) now call the canonical ports.
- Five ancilla-local copies (sparkle termination, forced garnish
  allocation, poof garnish, repulse spark, weapon tink) and the third copy
  of the bomb transmute in the overlord module are gone; the sprite
  module's `set_oam_plain_at` is the one plain OAM writer.
- The Ganon module's private port of `Dungeon_ExtinguishTorch`, which
  differed from the dungeon's only in reaching the fixed-color byte through
  the write-through setter rather than the owner bridge, is replaced by a
  call to the dungeon's routine, as the original does. The three identical
  lit-torch color tables are one table.
- The Hinox shop's copies of the HUD upgrade-limit tables import the HUD's.
- The draw module's file-local `PrepOamCoordsRet` copy is gone; the
  sprite module's struct gained the tuple conversion, and the two other
  modules that imported the copy use the canonical one.
- The draw module's copy of the cucco's `bawk_bawk` sound helper calls the
  dungeon NPC original.

The adapters that change a signature (spawn helpers returning `Option`,
draw-table conversions, the Ganon OAM emitters) are unchanged; 41 such
`_for_<module>` helpers remain and are candidates once their canonical
shapes settle.

Compatibility constraints remain explicit:

- Every removed body was identical to its canonical port apart from local
  variable style; no statement order or write changed.
- The fixed-color byte written by the Ganon torch path now goes through
  the room-effects owner bridge like every other dungeon torch path; both
  reach the same byte.

## Verification

Readability, projection discovery, and the ownership scanner pass with
zero same-mode overlaps and 33 overlapping bytes.

All 1,771 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`45880137bc21275fd3bc817e0f1de4fd4f3b422384e59786b87e2b69ee8ac834`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 314.17 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.
