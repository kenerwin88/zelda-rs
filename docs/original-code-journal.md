# Journal: what the original code did

A running record of the most interesting things the 1991 game does at the
code level, collected as the modernization touches each area. Each entry
says what the original did, why it is remarkable, and what the modern
codebase does about it. Entries are grouped by theme rather than by date;
new findings go under the matching heading with the batch that found them.

The point of the journal is to keep the tricks visible after the code that
embodied them is gone. Most of them were forced by 128 KiB of WRAM, a
65816 with no cache, and a cartridge whose tables were cheaper than code.

## One byte, several jobs

The dominant pattern. WRAM is reused by game mode, so one address means
different things depending on which module is running. The port kept these
as one address with several native owners and paid for it in stale-copy
bugs; the modern rule is one owner per byte, with mode gates or explicit
write-through where the SNES reuse is real.

- **0xc8 is four things.** The dungeon scratch word `R16`, the intro's
  menu counter, the overworld entrance-sequence counter, and the menu
  animation timer all live at 0xc8 in the original (`some_menu_ctr`,
  `overworld_entrance_sequence_counter`, `R16` are three names for it).
  The dungeon code even decrements it as two separate bytes in one
  expression (`--BYTE(R16) || --HIBYTE(R16)`). Modern: each user writes
  through directly; nothing bulk-projects it.
- **Z's subpixel byte is also the throne-fade timer.** `link_subpixel_z`
  and `attract_var13` are both 0x2c. The attract sequence borrows Link's
  vertical fraction byte as a fade timer because Link is not moving during
  attract. Modern: the native position takes the fraction as an input and
  writes it back only when movement changes it. (native-player-motion)
- **The boss-prize countdown.** `byte_7E04C2` has no name even in the
  decompilation. The boss room tag stores 128 there when the heart
  container starts falling, and the falling-prize ancilla counts it down,
  swapping in the item graphics when it reaches 1. The port modelled it
  twice under two guessed names (a "milestone item graphics countdown" and
  a "moving wall torch update flag"), neither of which is what it is.
  Modern: one owner, one countdown, named for the prize.
  (boss-prize-countdown-owner)
- **The map8 attribute bank doubles as packed map graphics** during
  overworld transitions. The same words are tile attributes in one phase
  and compressed graphics in the next; the port imports the aliased word
  with a separate interpretation at the lookup boundary.
  (native-tile-definitions)
- **The room parser's tile catalog is 512 entries, but code indexes
  1024.** Entries past 512 alias live graphics memory. The original never
  bounds-checks; it simply reads whatever is there. Modern: the parser owns
  512, and the aliased range is imported at the lookup boundary.
- **The replacement-tile table window (0x500..0xd00)** is the dungeon's
  object replacement table indoors and the sprite and overlord tables the
  rest of the time; the overworld map16 stripe builder walks 32 words of
  it with `d = (d + 1) & 0x1f`. (overworld-map16-stripe-owner)
- **Ancilla scratch `g[9]` is the hookshot's effect index** and the game
  over screen's letter cursor at 0x39d. Mode-reuse, not a bug.
- **The palette swap flag 0xabd** belongs to the world in one phase and
  the follower system in another.
- **One HDMA window serves two floods.** The swamp palace drain and the
  dam flood animate the same six window words (`water_hdma_var0..5`) with
  different formulas: the drain steps both radii toward a target, the
  flood widens an alternate y radius. The port had split them into a
  "display" and a "dungeon environment" model and then patched the
  adjuster to read raw RAM. (water-hdma-window-owner)
- **The overworld's big doors are dungeon doors.** The four-tile opening
  of an overworld entrance (the Hyrule Castle gate, the pyramid) runs on
  the dungeon's door animation step (0x690) and door open counter (0x692),
  stepping a 56-entry tile table two tiles at a time. There is no overworld
  door system; the overworld borrows the dungeon's.
  (door-step-owner)
- **The exploding wall lives in the text buffer.** The blast wall's
  phases, timers, center, direction, and fragment positions
  (`blastwall_var5..11`) are the first 64 bytes of the dialogue buffer at
  0x10000, free while no message is rendering. The Skull Woods entrance
  fire and the tower seal use the same bytes for their own animations.
  (blast-wall-owner)
- **The sparkle garnish spawner parks its slot index in R15**, the high
  byte of the sixteen-bit collision word tile detection had just filled.
  Nothing reads it back; the byte simply stays visible in WRAM until the
  next collision probe clears it. (zero-page-scratch-owner)
- **The star-switch floor phase is one byte, `SWYKPT`.** A pressed star
  tile toggles it and the CHR restore reads it to choose which half of the
  star graphics to copy back. The port had split it into an "overworld
  restore phase" and a "torch blink phase", each gated on the indoors
  flag. (star-tile-phase-owner)

## Passing data through the scratch registers

- **A spawn hands its child the parent's coordinates through $00..$08.**
  `Sprite_SpawnDynamically` leaves the parent's x, y, z, and overlord
  position in the direct-page scratch words, and the caller's next routine
  (`Sprite_SetSpawnedCoordinates`, or its own arithmetic) reads them from
  there. The port names the record fields after those offsets (`r0_x`,
  `r2_y`, `r4_z`, `r5`, `r7`). The child inherits the parent's height this
  way; the port's per-module adapters had dropped it. (spawn-record-carry)

## Arrays that are deliberately indexed past their end

The 65816 has no bounds; the original treats adjacent arrays as one
address space and uses that on purpose.

- **Overlord slot 8's X low byte is slot 0's X high byte.** The overlord
  bank has eight physical slots with parallel byte arrays (type, X low, X
  high, ...). Boss code indexes slot 8 and beyond so that the "X low" read
  lands in the next array. The port had sized the native table at 16
  slots, which spilled the projection over the sprite bump-damage and
  stunned bytes. Modern: views over WRAM with no slot limit, writes
  rejected outside the bank. (overlord-single-owner)
- **Armos and Arrghus home positions are 27-entry arrays laid over a
  bank that only has room for 24**, with bases one byte apart between the
  two bosses. Entries 24..26 fall into sprite-position and sprite-stunned
  bytes. The writer reaches them; the reader gets zeros for the Y high
  byte. Both are preserved as the alias contract.
- **The credits death table has 14 palaces and indexes 15.** The
  per-palace death counts are fourteen words; the credits lookup table
  ends in 15, which addresses the total death counter that happens to sit
  after them at 0xf405. Modern: the table explicitly selects "total" for
  that entry. (native-save-progress)
- **Saved room flags are 320 words, indexed up to 640.** The room-flag
  index can run into the rest of the save block (overworld events,
  inventory). The original relies on it. Modern: ordinary rooms hit native
  records and extended indices go through the cartridge codec.
- **The save checksum is computed across an NMI.** The checksum loop can
  be interrupted, so it observes a prefix from before the interrupt and
  the rest from the current image. A single snapshot would compute a
  different checksum than the hardware. (native-save-progress)

## Encoding meaning into bits and masks

Tile attributes and object records carry several facts in one byte or
word, decoded at every read point with mask arithmetic. The modern code
decodes each family once into a definition and reads names.

- **Tile attribute families by mask.** `& 0xf0 == 0x70` is a tracked
  object, `& 0xfc == 0x6c` a curtain panel, `>= 0xf0` a closed door with
  its slot in the low nibble, `0x26` and `0x5e/0x5f` the spiral and wall
  spiral stair heads, `0x38/0x39` straight stairs up and down, `0x3b` a
  star switch whose toggle is the low bit, `0xb2..0xbe` the Somaria pipe
  junctions. The transition-landing class is one bit tested on every tile,
  not only doors. (native-dungeon-tile-roles, native-somaria-pipes)
- **The same attribute means different things indoors and outdoors**,
  and the sprite, sprite-blocking, ancilla, and ancilla-ground tables are
  four separate ROM tables, two of which are byte-identical copies.
  (native-entity-tile-probes)
- **Indoor slopes read both flip bits; outdoor slopes read only the
  horizontal one.** Vertical flip does not exist for outdoor collision.
- **Object records pack kind and phase in one word.** `& 0xf0f0 == 0x1010`
  is a liftable with the kind in the low nibble, `0x2020` one segment of a
  big gray rock, `0x4040` a hammer peg, and the push block counts 1..5
  then 0xffff through its phases with a plain increment that wraps a
  vanished block back to idle. The attribute loader tests only the low
  byte for bombable floors. (native-object-records)
- **The lift path masks the slot nibble without checking the family**, so
  a closed door is accepted as an object slot there. The modern query
  exposes the nibble for every identity because three callers depend on
  that.
- **The pipe traveller parks its next direction in the tile-identity
  slot.** After an endpoint the sprite writes a direction into the E byte
  that normally holds the pipe tile it is on. The next probe therefore
  always sees a change and the traveller turns. (native-somaria-pipes)
- **Bit 15 of a room's save-state word is the boss-defeated flag.** The
  falling-prize ancilla, the boss room tag, the dungeon loader, and three
  sword-handling paths in the player all test the same bit of the same
  word for unrelated decisions.

## Doing two things with one write

- **The heart/refill word.** The heart counter and the magic refill byte
  are adjacent, and the original adds to them as one 16-bit word so a
  refill carry spills into hearts. (native-player-components)
- **The paired-word carry in staircase identity sequences.** Stair tiles
  are written as words, and the increment that advances one stair's
  identity carries into the next tile's byte. Modern: a named sequence
  with the carry confined to the cartridge encoding.
- **The door-animation word at 0x690 sits inside the replacement-tile
  window** that the map16 stripe builder also walks, so any bulk copy of
  that window silently carries a second copy of it. Modern: a scalar owner
  with write-through, and no bulk copy of the window.
- **`which_entrance` is a byte, but two places store a word** across it
  and the overworld hole-scan step at 0x10f: the ending's scene table and
  the starting-point entrance load. Modern: those two stores write both
  owners explicitly. (world-transient-owners)
- **The initial sword grants the shield first**, as a side effect ordered
  inside the item receipt routine, before the sword itself.
  (native-item-awards)
- **Reset A clears only the low byte of the bunny timer; initialization
  clears only Z's high byte.** Partial-byte resets that leave the other
  half of a word intact are common and must be preserved exactly.
  (native-player-transitions)

## Probes, ordering, and observable intermediate state

Because every routine writes straight to WRAM, the order of writes is
observable to anything that reads mid-routine, including the NMI handler.

- **The player detection reset is 27 ordered clears**, each visible
  before the next, with the dungeon moving-floor clear last and the
  key-lock clear touching only the low byte before a later spike clear
  touches the high byte. (native-tile-behavior)
- **Probe publication is asymmetric.** Sprite, overlord, and indoor
  ancilla probes publish the probed tile to the sprite scratch; the hammer
  splash and the outdoor ancilla probe do not, and the ancilla's slope
  test then reads whatever the scratch last held.
  (native-entity-tile-probes)
- **Cardinal collision probes accumulate bits in low/center/high order
  (1/2/4); slope probes skip the center. Footprint probes are top-left,
  bottom-left, top-right, bottom-right with bits 8/2/4/1.** The bit
  assignment is a fixed ROM convention consumers depend on.
  (native-player-collision)
- **Slope eligibility is read twice**, because the first axis's probe can
  block the second axis before its own probe runs.
- **Movement axis order changes when airborne**: Z, X, Y in the air, X, Y
  on the ground, and movement can be suspended between the fraction, the
  low byte, and the high byte of a coordinate, with the resumed stage
  observing whatever was written in between. (native-player-motion)
- **The chest handlers use a Misc-then-identity-then-Collision sequence**
  while every other solid interaction is Collision-then-Misc.
- **Coordinates wrap at 16 bits before masking.** Probe X becomes a tile
  column, probe Y stays a masked pixel coordinate; the original never
  clamps.
- **Torch targets store whatever the fire hit**, not just torches; the
  lighting test is a separate mask on that stored byte.

## Timing by side effect

- **Long CPU operations have observable midpoints.** The dialogue engine's
  click sound is emitted at a particular program counter inside glyph
  setup, not at the end of the operation; audio commands cross the NMI
  boundary depending on where the routine is when vblank arrives. (bug
  class 5 in CLAUDE.md)
- **The Zelda bug the decompilation fixed.** After a boss dies, the room
  tag `RoomTag_GetHeartForPrize` drops the heart container as an ancilla and
  then disarms itself. The prize allocator uses only ancilla slots 0..4 and
  may evict only sparkles and arrows stuck in walls, so five live bombs,
  flying arrows, or magic effects at the killing blow make the spawn fail.
  In the original the tag disarms itself anyway, and the heart container
  never appears. The decompilation keeps the tag armed on a failed spawn so
  it retries next frame, and the port inherits that fix; the parity route
  never reaches the failure branch, so `runtime_boss_prize_retry.rs` pins
  it.
- **The iris spotlight's tick-versus-radius offset** is decided by the
  exact cycle cost of building the per-radius table racing vblank. It is
  correct per frame count and wobbles per radius within a frame; it is a
  property of the CPU, not of any discrete decision.

## Tables over code

- **The four entity tile tables** (sprite, sprite blocking, ancilla,
  ancilla ground) and the equipment target tables (76 entries mapping
  item IDs to WRAM addresses) replace conditional code with data. The
  port kept the address tables and dispatched item awards through them;
  modern code uses typed grants. (native-item-awards)
- **Chest alternates were a sparse numeric table** of three real rules
  (which item you get instead when you already have the first).
- **The lit-torch color is one constant** used from three modules; the
  port copied it three times. (port-time-duplicates)

## Original bugs the modern code preserves on purpose

- The Armos home reader's zero Y-high for slots 24..26 while the writer
  reaches those bytes.
- The credits death index 15 alias.
- The lift path accepting a closed door as an object slot.
- The pipe traveller's parked direction in the tile slot.
- Partial-word clears that leave the other byte stale.
- The HUD item-slot clamp: an out-of-range slot clamps the stored value
  while the published value falls back to the first slot, and the next
  bridge observes the published one. (native-save-progress)

## Adding to this journal

Add an entry when a batch uncovers something the original did that a
reader of the modern code would not guess. State what it did, why (when
known), and the modern counterpart, and name the batch doc under
`docs/parity/` that carries the evidence.
