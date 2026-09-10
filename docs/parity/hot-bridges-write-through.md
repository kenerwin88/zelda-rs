# The OAM and BG2 attribute bridges write through

Sampling the cached route after the bridge change showed the adoption read
concentrated in two bridges that gameplay constructs per element: the OAM
bridge, built for every sprite entry write, reloaded and re-projected its
three shadow tables (over 700 bytes) each time, and the dungeon BG2
attribute bridge, built for every tile of a room draw, decoded the 4 KiB
attribute table each time. Together they explained most of the difference
between 341 and 318 seconds for the 200,000-frame comparison.

Every setter on both bridges maps to bytes it owns, so both are now
write-through: each setter updates its native field and stores exactly the
byte or word (or, for the attribute table, the encoded bytes of the tiles it
changed) and asserts coherence, as the priority and region setters already
did. Neither bridge adopts WRAM at construction or projects the whole state
any more. The values stored are the ones the bulk projection would have
produced for the same fields; the attribute bytes come from the same
`export_slice` encoding the projection uses.

Compatibility constraints remain explicit:

- The OAM entry writers store the same bytes at the same addresses as the
  projection did, including the four bytes of a whole entry in one store.
- The priority value and region tables keep their existing write-through
  handling and coherence exclusions.
- The attribute bridge still bounds every store by the table length.

## Verification

One unit test seeded the OAM cursor by writing WRAM directly and then used
the bridge; it now seeds through the bridge, and the runtime core's private
duplicate of the cursor constant that only that test used is gone. The
library compiles with no warnings in the parity, dev, and lib-test builds;
readability and projection discovery pass; the scanner output matches the
preceding batch. All 1,722 library tests pass under the dev profile.

The main tree validated this batch together with the macro batch that
follows it, on parity binary
`5baabc80bf7a4b559eb2c2d5e8e82e9c6e849e33bb25f208dda97d4faeda2575`: the
200,000-frame cached comparison matched every video and audio hash in
322.66 seconds (341 seconds for the preceding batch, 310 to 318 before the
projection target), the frame 60000 and 150470 WRAM goldens match, the
200,000-frame WRAM endpoint is the recorded `dd45975c…` image, and all
1,722 library tests pass under the parity profile.
