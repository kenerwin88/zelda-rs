# Native entity tile probes

Sprites, ancillae, overlords, the guard's forward probe, and the hammer's
water splash now share one tile lookup, `entity_tile_at`, which selects the
indoor attribute layer by floor or reduces an outdoor x coordinate to its
map8 column, exactly as the original `GetTileAttribute` did. The five
per-file copies of that lookup are gone. Publishing the probed tile to the
sprite scratch (`sprite_tiletype`) is a separate explicit step,
`probe_entity_tile`, because the original did not publish on every path:
the ancilla outdoor path records the tile on the slot but leaves the
scratch untouched, and its slope test then reads whatever the scratch last
held. The guard probe keeps its distinct outdoor source, the whole-map16
attribute table, through `overworld_map16_tile_definition_at_location`.

Entity interaction is now a predecoded property of the shared tile
definition, alongside the player's collision behavior. `EntityCollision`
names the families the original kept in four ROM tables (two of which were
byte-identical copies): sprite probe, sprite blocking, ancilla, and
ancilla ground layer. The four straight slopes carry an entity slope
profile; `entity_slope_blocks` replaces the two copies of the sloped-tile
height check, and reports `None` for tiles without a profile so the
ancilla's off-table case keeps its solid result. Conveyor direction and the
statue floor switches are also decoded once.

The sprite workspace scratch and the dual-layer tile cache store native
tiles rather than bytes, with byte-identical WRAM projection and checkpoint
serialization. Sprite handlers compare named identities (pit, deep and
shallow water, water staircase, moving floor, spike cactus, grass, ground)
instead of hexadecimal attributes. Sprite draw bytes that the original loads
from the scratch tile still receive the encoded identity at the slot
boundary. The pipe network keeps its encoded identity in the sprite's E
slot, and the Zoro spawner keeps its nest identity.

Compatibility constraints remain explicit:

- Publication order is unchanged: sprite, overlord, and indoor ancilla
  probes publish the scratch tile; the hammer splash and outdoor ancilla
  probes do not.
- The outdoor x reduction stays visible to callers, since the sprite slope
  test reads the reduced column.
- The ancilla's wall identity 3 and boomerang interactable range checks are
  identity comparisons at the read point, as before.
- The replay summary's encoded overworld attribute export remains a public
  method; gameplay reads the definition.

## Verification

`runtime_entity_tiles.rs` freezes the four original tables and the slope
height table verbatim and checks every attribute's decoded families, slope
profile at every fine position, conveyor index, and floor-switch identity
against them. That contract caught a merged range at 0x3a-0x3b during
development. Further regressions cover floor-layer selection, outdoor
column reduction, scratch publication only from the probe, the guard's
map16 source and publication, the ancilla's indoor-only publication with
stale-scratch slope reads outdoors, and the sprite property classification
through the deflecting and ordinary paths. Existing lookup, workspace,
dual-layer cache, and timing-split tests were retyped without changing
their expectations.

All 1,771 library tests pass under the parity profile, with two existing
ignored tests and no compiler warnings (99.70 seconds). Under the dev profile
ten pre-existing tests trip debug assertions and an OAM index underflow on
untouched main as well; they are unrelated to this batch.

RAM readability, projection discovery, and the ownership scanner pass;
the scanner reports the same 114 projection writers, 86 reachable writers,
and 40 existing overlapping bytes.

Candidate binary SHA-256:
`4018c5f0a429f490755d5632631df73e085992b8c829fce84a362a7912ce9e29`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 311.54 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted tile-definitions build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.
