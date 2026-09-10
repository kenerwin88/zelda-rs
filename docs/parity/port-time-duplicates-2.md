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

All 1,774 library tests pass under both the parity and dev profiles, with two
existing ignored tests and no compiler warnings.

Candidate binary SHA-256:
`e3354f239541f969157bcfb703bcec8eecf40b32656d10f74e77fd8b8af8428d`.

The 200,000-frame cached Snes9x audio/video comparison passed from frame zero
in 318.03 seconds. Both reached WRAM goldens match, and the complete endpoint
is byte-identical to the preceding promoted build, SHA-256
`dd45975cee5acdd270d1b0c74c5d38f1ba3ce3bd7e0af648264b8f77e95f244d`.

Source commit `7d18381c` passed the full cold route: 1,581,079 consecutive
exact audio/video frames in about 2435 seconds (40.6 minutes), starting at
frame zero with no frame limit or checkpoint resume. The comparison used the
immutable Snes9x oracle cache; it did not reload the live core. No RNG drift
was reported. The source commit skipped the commit hook at the user's
request; this full-route pass is the gate.

All four WRAM goldens (60,000, 150,470, 500,000, and 732,000) match. The full
131,072-byte final WRAM image is identical to the preceding promoted build,
SHA-256:
`316193798ccb2f771546b25443df7d417bddac8a7cac65326fa189c1264fbdb6`.
The promoted receipt is
`routes/full_run/receipts/oam-full-av.manifest.json`.
