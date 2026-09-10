# Port-time forwarders, second pass

The first pass removed the pure forwarders in the world and Ganon modules
and the ancilla-local copies. A mechanical scan of every remaining
`*_for_<module>` helper whose body is exactly the canonical call with the
same arguments found 44 more across the guard, Helmasaur King, Hinox shop,
NPC, small-boss, draw, and prep modules, plus two remaining copies of the
plain OAM writer (draw and prep) and the draw module's cast adapter for
`Sprite_CorrectOamEntries`. All of them now call the canonical ports, and
the copies are gone: 93 OAM-writer and correction call sites in the draw
module alone.

The adapters that change a signature (spawn helpers returning `Option`,
the spawned-coordinate adapters, the draw-table conversions, the Ganon OAM
emitters, the cucco subtype continuation) are unchanged; 31 remain.

Compatibility constraints remain explicit:

- Every removed body was the canonical call with the same arguments in the
  same order, or the canonical body verbatim; no statement order or write
  changed.
- The correction adapter's `u8` count reached the canonical `i32` parameter
  through a widening cast; every caller passes a literal, which now types
  directly as `i32`.

## Verification

The library compiles with no warnings in the parity, dev, and lib-test
builds; readability and projection discovery pass; the scanner output is
unchanged (zero HIGH RISK overlaps, 12 bridge-sync overlaps, 60 overlapping
bytes). All 1,774 library tests pass under the dev profile.
