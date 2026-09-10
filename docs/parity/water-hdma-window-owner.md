# Water HDMA window ownership

The original keeps six words at 0x680..0x68a (`water_hdma_var0..5`) for the
water HDMA window: its center, its two radii, the y target the swamp drain
animates toward, and the alternate y radius the flood uses. Two scenes write
them, the swamp palace drain in module 7 and the dam flood, and the room
draw seeds them for water rooms. The port modelled the same words twice:
`WaterHdmaWindowState` in the display (center and radii, master-projected
every frame, write-through setters for the watergate scene) and
`DungeonEnvironmentState` (radii, target, alternate radius, and center,
re-projected by its bridge from every environment setter). Neither updated
the other, so the HDMA window adjuster read the two radii from raw RAM with
comments explaining that the display model was stale during the swamp scene
(the f606612 and f606748 one-pixel fixes), and any environment setter could
re-stamp its stale radii over the watergate scene's live values.

The display's water window is the single owner of all six words. It gained
the y target and alternate y radius, word setters for the radii, and a
positioned-window setter that writes the center x then y as the original
does. The swamp drain, the flood, and the water room draws read and write it
through the owner; the adjuster's raw reads and their comments are gone. The
six environment fields, their accessors, setters, forwarders, and the
environment test that exercised them are removed. Because both states are
serialized in checkpoints, the checkpoint magic moved from `Z3RSPC14` to
`Z3RSPC15`.

`runtime_water_hdma_window.rs` sets a radius through the owner, then fires
an unrelated environment setter and checks the radius survives; on the
preceding commit the environment's bridge re-stamps zero over it.

Compatibility constraints remain explicit:

- The swamp drain still compares the y radius with the y target each step
  and the flood still combines the x radius and alternate y radius as before;
  only the owner they consult changed.
- The room draw still writes the radii and target before the window center,
  in the original order.
- The watergate scene's byte-wide y radius writes are unchanged.
- The y target and alternate y radius stay out of the master projection,
  as they always were (the environment only wrote them through its bridge);
  their setters write through. The projected byte set is unchanged, which
  the frozen player-collision and tile-behavior fixtures confirm.

## Verification
