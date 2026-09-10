# Projection targets

Every native state publishes its bytes through `write_to_ram`, and until now
that meant writing into a WRAM slice directly, so a projection could only
ever be run against live WRAM. The 167 projection functions and their
helpers are now generic over a `RamTarget`: a small trait with byte, word,
range, and fill writes plus the three reads the mode gates need (a byte, an
optional byte, and the length). Live WRAM (`[u8]`, `Vec<u8>`) implements it
by writing in place, byte for byte as before. A second implementation,
`ProjectionLog`, records the bytes a projection would write while reading
its gates from live WRAM.

The trait is the foundation for the next change: a bridge can project its
state through a compare-on-write target and publish only the bytes whose
value changed, instead of re-stamping the whole state on every setter. That re-stamp is the mechanism behind every stale-copy clobber
found this week (the boss-prize countdown, the water HDMA window, the door
animation step, the blast-wall trigger words), and 103 bridges still do it
across 795 setters.

The rewrite is mechanical: `ram[X] = v` became `ram.write_byte(X, v)`,
`write_le_u16(ram, X, v)` became `ram.write_word(X, v)`, slice copies became
`ram.write_range(A..B, &s)` with the range kept explicit, fills became
`fill_bytes`, and the two projections that encode tile attributes into a
sub-slice encode into a scratch vector and write that range. Nothing about
which bytes are written, in which order, or with which values changed; the
trait's slice implementation performs the identical stores.

`find_dual_ownership.py` reads the new forms (and still reads the slice
forms the write-through bridges use) and accepts the generic signature. Its
range pattern now also accepts a range without a named start, which the old
`ram[..N]` form never matched, so it newly reports one same-mode overlap:
the sprite workspace projects the sixteen zero-page scratch bytes at
0x0000..0x000f, and the tile detector projects its slope-collision words at
0x000c and 0x000e inside them. Both model the original's shared scratch
registers; that is a candidate for a single owner, and the write-what-
changed publication removes its clobber risk regardless.

Compatibility constraints remain explicit:

- Every projection writes the same bytes, in the same order, with the same
  values; only the mechanism of the store is abstracted.
- The log is not used by any production path yet.
- The MSU resume-slot writer stores its 32- and 64-bit words as explicit
  byte ranges; the map32 decoder reads its words through the target.

## Verification

`ram_target.rs` carries a test that projects a state into the log and
replays it, which must equal the direct slice projection. The library
compiles with no warnings in the parity, dev, and lib-test builds;
readability and projection discovery pass; the scanner reports one HIGH
RISK overlap (the newly visible zero-page scratch pair), 10 bridge-sync
overlaps, and 70 overlapping bytes. All 1,776 library tests pass under the
dev profile.

The projection rewrite was validated together with the bridge change that
follows it; see `bridge-publish-changes.md` for the shared evidence.
