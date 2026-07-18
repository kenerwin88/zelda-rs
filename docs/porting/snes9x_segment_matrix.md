# Snes9x-native segmented oracle matrix

Native boundaries and replacement chapter takes can be recorded interactively
with [`snes9x_route_recorder.md`](snes9x_route_recorder.md).

This gate is designed for broad black-box video and audio parity without
pretending that the checkpoint-derived route is one continuous playthrough.
Its result is always labeled **segmented coverage**.

The matrix has 13 chapter input logs and 12 paired boundaries. At every
boundary the two lanes are separate:

- `rust_start.z3state` is the translated engine's chapter checkpoint. It is
  used only by the production renderer/audio lane.
- `oracle_start.state` is a Libretro serialization created by Snes9x itself.
  Rust snapshot bytes are never loaded into Snes9x.

Snes9x is a dynamically loaded test oracle only. None of its implementation is
linked into the game, renderer, or modern audio engine.

## Bootstrap the native state chain

Build the release binary, then run:

```bash
target/release/zelda3 --build-snes9x-segment-matrix \
  external/snes9x-libretro/local/snes9x_libretro.dylib \
  target/parity/audio-oracle-fixture/zelda3.sfc \
  saves/zelda3-combined-route-proof.json \
  /path/to/zelda3/saves/ref \
  saves/sram.dat \
  target/parity/snes9x-segment-matrix \
  --expected-core-sha256 1143f14ceac6be6e94f78ab16270590bc415532f200db1f7fece66994b4ce3d5 \
  --expected-rom-sha256 66871d66be19ad2c34c927d6b14cd8eb6fc3181965b6e517cb361f7316009cfb \
  --replace
```

The bootstrap feeds each chapter's controller stream to Snes9x, writes the
native end serialization, restores it before the next chapter, and checks the
next milestone from Snes9x WRAM. It fails closed on the first mismatch. Add
`--continue-after-mismatch` only for diagnosis; the resulting manifest remains
ineligible for parity coverage.

The manifest pins the core, ROM, SRAM, proof, every chapter replay, every input
stream, both paired starts, and every Snes9x end state. It explicitly records
`continuous_playthrough: false` and `converted_from_rust: false`.

## Supply independently recorded Snes9x states

If the chapter logs do not form a valid native chain, record each boundary in
Snes9x and place the 12 Libretro serializations in one directory. Add a
`provenance.json` with this shape:

```json
{
  "kind": "zelda3_snes9x_native_state_set_v1",
  "core_sha256": "<pinned core sha256>",
  "rom_sha256": "<pinned ROM sha256>",
  "states": [
    {
      "segment": 2,
      "path": "segment-02.state",
      "sha256": "<state sha256>",
      "created_by": "Snes9x libretro retro_serialize",
      "converted_from_rust": false
    }
  ]
}
```

The `states` array must contain segments 2 through 13 in order. Run the same
builder with `--oracle-start-dir /path/to/native-states`. It verifies all state
hashes and provenance, loads each state through Snes9x `retro_unserialize`,
replays that chapter independently, and requires all 13 terminal milestones.
This is the intended 13-segment matrix; the chain bootstrap is only the fastest
way to discover whether new native recordings are needed.

## Run production video and audio parity

Only an eligible capture manifest can start output comparison:

```bash
python3 scripts/snes9x_segment_matrix.py \
  target/parity/snes9x-segment-matrix/manifest.json \
  --binary target/release/zelda3 \
  --output-dir target/parity/snes9x-segment-matrix-results
```

Each chapter runs from its paired pre-frame states with exact completed-frame
video comparison and exact continuous PCM comparison. The Rust lane explicitly
uses `modern` audio plus the `native` sequencer. Dynamic alignment is disabled.
The aggregate result sums only chapters whose video and audio both pass.

Passing all 1,073,092 aggregate frames proves segmented coverage. It does not
replace the final uninterrupted, normal-gameplay Snes9x movie-to-credits gate.

## Current checkpoint-route result

The current chain bootstrap creates all 12 files through Snes9x serialization,
but it is not route-eligible. Segment 1 ends at module 7 with health `0x18`
instead of the proof's module 4 and health `0x00`; chaining the later chapter
inputs then misses every later milestone and finishes at module 14, not credits.
The output runner correctly refuses this manifest and reports zero verified
video/audio parity frames.
