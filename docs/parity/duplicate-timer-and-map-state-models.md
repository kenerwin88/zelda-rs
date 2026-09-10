# The mode-7 zoom timer and the map-state word have one owner each

Two more bytes had two native models of the same meaning, not two modes'
worth of reuse.

The mode-7 zoom timer at 0x0637 (`timer_for_mode7_zoom` in the original)
belongs to the overworld map zoom state, which the map screen drives. The
attract sequence zooms the same map with the same timer, and the port gave
the attract scene state its own copy, with its own setter and decrement on
the attract bridge. The attract code now reads and writes the timer through
the map zoom state's accessors, as the map screen does.

The map-state word at 0x0200 (`overworld_map_state`) belongs to the
overworld map UI state, which the HUD and the ending drive. The chest
reveal room tag walks the same word as its cursor over the room's chests.
The port gave the dungeon room-item state a copy named
`chest_reveal_cursor_x2`, then kept the two models coherent by re-stamping
the dungeon copy from every map-state setter, after a stale copy had once
left the HUD counter stuck across a room transition. The room-item copy and
the re-stamp are gone; the chest reveal reads and writes the word through
two thin wrappers that name its role there, and the room-item state keeps
only the chest count the cursor is compared against.

Compatibility constraints remain explicit:

- The attract sequence sets the timer to 0xff, reads it for the zoom
  factor and the brightness ramp, and decrements it on the same frames as
  before, through the map zoom bridge.
- The chest reveal clears the cursor before and after its loop and stores
  each advanced cursor, as before; the stores now go through the map UI
  bridge and the word is published once.
- The checkpoint magic is bumped to `Z3RSPC20` because two serialized
  fields are gone.

## Verification

`find_dual_ownership.py` reports zero HIGH RISK overlaps, 9 bridge-published
overlaps, 19 cross-mode overlaps (20 before) and 60 overlapping bytes (61
before). The attract-mode overworld test seeds and reads the zoom timer
through the map zoom accessors. The library compiles with no warnings in
the parity, dev, and lib-test builds; readability and projection discovery
pass. All 1,721 library tests pass under the dev profile.
