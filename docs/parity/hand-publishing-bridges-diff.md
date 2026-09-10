# The torch, water-window and Trinexx palette bridges publish by diff

Three more bridges stored their bytes by hand next to the projection that
already encodes the same layout: the dungeon torch bridge (22 setters,
24 raw stores), the water HDMA window bridge (15 setters, 14 raw stores)
and the Trinexx shell palette bridge (8 setters, 8 raw stores). None
adopted the live WRAM at construction; each relied on a whole-state
assertion after every setter to prove the state still matched.

All three now adopt at construction and publish through the compare-on-write
target, like the Link bridge and the plain adopting bridges. The setters
keep their state calls and call `sync`. Raw stores that remain are the
ones into bytes the states do not model: the movable-block scratch the
torch initialiser fills, the two torch table imports (which copy the
cartridge table into WRAM, including bytes past the scanned range, and
then re-adopt), and the water window's four write-through-only words plus
the spotlight window buffer byte it writes on the spotlight state's
behalf.

Compatibility constraints remain explicit:

- Each setter publishes exactly the bytes its state call changed, at the
  same point the removed store ran.
- Adoption at construction reads the same bytes the assertion compared,
  so a bridge built over WRAM that another owner changed sees the live
  value rather than a stale field.
- The torch timer setters no longer carry an index guard around a store
  that is gone; the state's own bounds check remains.

## Verification

The torch and water-window runtime tests (86 in the focused set) and all
1,721 library tests pass under the dev profile. The ownership scanner is
unchanged. The library compiles with no warnings in the parity, dev, and
lib-test builds; readability and projection discovery pass.
