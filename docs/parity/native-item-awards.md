# Native equipment and item awards

Equipment now lives in named Rust fields in `InventoryItemsState`, selected
by `EquipmentItem`. Bomb count and equipped-bottle index live only in
`PlayerResourcesState`; the aggregate legacy inventory reader routes their
indices to that owner. Equipment accessors read their fields directly.

The compatibility layout contains 26 equipment bytes plus four bottles.
The reserved equipment byte remains lossless without inventing gameplay
meaning. Removing the two resource copies eliminates the projection skips
and the coherence check's masked comparisons. The equipment codec only
projects its own fields and bottles.

Item awards use equipment grants, arrow refills, and typed dungeon-item
grants. The duplicate 76-entry WRAM-target tables, generic item-memory
read/write/absorb functions, and address-dispatched dungeon-flag mutation
are removed. Three typed chest-alternate rules replace the sparse numeric
alternate table. The legacy target table, rather than its inaccurate
compass/big-key comment, determines the retained behavior: item `0x25`
updates the compass word, and `0x32` updates the big-key word.

## Timing and compatibility boundary

Effects execute at their original points in the item-receipt routine.
The initial sword still grants the shield first. Conditional armor grants
retain the original WRAM observation inside the compatibility adapter.
Equipment awards publish only their selected byte; other existing bridge
operations retain their publication behavior. Graphics, sound, animation,
and suspension/resumption order are unchanged.

This is a step toward native gameplay ownership. Remaining save/load,
HUD/menu indexing, raw-memory consumers, and frame-boundary synchronization
have not all migrated. WRAM remains a compatibility image, and its eager
publication cannot be removed until those consumers use the native owners.
This change does not claim a WRAM-free inventory system.

Removing serialized resource copies changes positional bincode layout.
Playable Rust checkpoint magic advances from `Z3RSPC04` to `Z3RSPC05`; old
Rust checkpoints are rejected before decoding and must be recreated.
The original Snes9x oracle cache and cartridge save layout are unchanged.

## Evidence

Baseline `850202d4` completed a from-zero 200,000-frame cached Snes9x A/V
replay with temporary assertions at aggregate inventory reads comparing
the bomb/bottle-index mirrors to the resource owner. All assertions and
A/V frames matched. Both reached WRAM goldens and the full 128 KiB endpoint
matched the preceding validated build. The probes were removed with the
duplicate fields.

`item-award-effects-850202d4.txt` freezes the old implementation's outcomes
for all 256 item IDs, four receipt methods, and three inventory/resource
states: 3,072 cases. Each per-item SHA-256 covers the complete WRAM before
and after native projection for all twelve method/state combinations.
The fixture uses deterministic minimal music assets for the sword receipt;
it is a CPU-only runtime regression, not a graphics/audio authority test.
It was captured before replacing the address tables and has no regeneration
switch in the committed tests.

Additional contracts freeze equipment offsets independently, preserve all
byte values tested (including high bits), verify single-byte award writes,
foreign-byte preservation, resource-owner reads, conditional observation,
clone/bincode isolation, and all chest-alternate IDs. Full-route acceptance
is recorded after validation in the promoted receipt.

The final candidate passed all 1,741 library tests (two existing ignored
tests), checkpoint rejection for layouts 01 through 04, and the RAM
readability guard. The existing 34 overlapping-byte ownership findings are
unchanged. Its from-zero 200,000-frame replay matched exact audio/video,
both reached WRAM goldens, and the baseline's complete WRAM endpoint.
