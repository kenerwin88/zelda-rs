# Native tile definitions

Player collision now reads an immutable tile definition rather than decoding
a cartridge attribute on each probe. The 256 identities have precomputed
indoor and outdoor behavior in `tile_definition`; the exhaustive byte mapping
is confined to `tile_definition/cartridge.rs` and evaluated at compile time.
Each `NativeTile` refers to one shared definition, preserving distinctions
between identities that happen to have the same player-collision behavior.

The indoor attribute map and the room parser's 512-entry tile catalog now
store native tiles. Asset loading, WRAM import, and checkpoint deserialization
resolve the original identity to a definition. The outdoor asset pack builds
its catalog, including horizontal-flip variants, when loaded. Editing its
attribute asset invalidates that catalog; a restored checkpoint rebuilds it
on first access. Other asset edits leave it intact.

Room layout recipes produce `NativeTile` and `TilePair` values. Lifted objects,
sprite-induced replacements, opened chests, rupee removal, pit overlays,
crystal switches, doors, and staircases all update that same native map.
Door transitions and staircase/torch identity sequences have named operations;
their cartridge encoding, including the original paired-word carry, lives
at the compatibility boundary. Production map writers accept native tiles.
The raw attribute setup helpers are test-only.

Compatibility constraints remain explicit:

- The room parser owns only the first 512 catalog entries. The next 512
  addresses alias live graphics memory and are imported at the lookup boundary.
- Indoor slope imports use both tile flip bits. Outdoor imports use only the
  horizontal bit. Identity distinctions are retained even for noncolliding tiles.
- The indoor map spans both contiguous layers. Existing cross-layer writes,
  zero-valued out-of-range reads, and publication after each original write
  remain intact.
- Overworld transitions reuse the attribute bank for packed map graphics;
  `import_aliased_map8_word` preserves that separate interpretation.
- WRAM projections and checkpoint serialization still encode the original
  bytes. Native tiles have no independently mutable attribute/behavior mirror.
- Legacy sprite/item interaction consumers can still request an encoded
  identity. Existing cartridge room-layout data retains its source encodings;
  the player collision path consumes predecoded behavior.

## Verification

The then-unchanged `tile-behavior-7ad23477.txt` fixture checks 131,072 attribute
executions and 32 resets against the pre-classification implementation's
full WRAM and native projections. It passed after the storage migration.

Additional regressions cover every attribute/flip combination, all 65,536
paired identity values and sequence/transition operations, all 65,536 map-tile
values for indoor/outdoor import, live graphics aliases, asset edits/clones/
restore, and 1,280 exact paired WRAM writes including layer crossings.
Both native tile vectors and the full indoor map retain byte-identical
checkpoint serialization.

All 1,766 library tests pass, with two existing ignored tests and no compiler
warnings (99.70 seconds). RAM readability and projection-discovery checks pass.
Ownership remains at 114 projection writers, 86 reachable writers, and 40
existing overlapping bytes; the only scanner-output change is recognition
of the newly named asset constant.

Candidate binary SHA-256:
`e1233c7c4e6d9204c4117a462a210bbfffc5a9e2a70327dd13c1a9cf41797117`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 327.59 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted tile-behavior build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `d0e92488a26087257a71d52053e608c2b60b07e3` passed the full
cold route: 1,581,079 consecutive exact audio/video frames in 2,551.67 seconds
(42.5 minutes), starting at frame zero with no frame limit or checkpoint
resume. The comparison used the immutable Snes9x oracle cache; it did not
reload the live core. No RNG drift was reported. The normal source commit
hook also passed its 500-frame standalone and fresh 180-frame live Snes9x
checks.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/tile-definitions-full-av.manifest.json`.
