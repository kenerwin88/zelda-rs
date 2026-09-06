# Z3TRACE1: the pinned Snes9x trace core's binary record format

The instrumented Snes9x core writes its trace as compact binary records
instead of JSON Lines. The format is produced by
`external/snes9x-libretro/patches/zelda3-trace-binary-format.patch` (the last
patch in the maintained set) and decoded by exactly two mirrors:

- Rust: `crates/parity/src/trace_format.rs` (used by the `zelda3` compare
  binary's semantic-receipt adapter and live-RNG reader, by `zparity
  trace-index`/`trace-query`, and by `zparity trace-decode`/`trace-encode`).
- Python: `scripts/snes9x_trace_format.py` (used by every script that reads a
  trace; `iter_events(path)` yields the same dictionaries the JSON Lines
  format used, and still accepts JSON Lines input for fixtures).

Why: a full-route live-RNG calibration emitted about 100 events per frame at
~730 bytes of JSON each (78 KB/frame, ~118 GB for the 1,581,079-frame route).
The binary record is 106 bytes plus a few tagged fields, 6-8x smaller, and
skips number formatting and JSON parsing on both sides.

## Layout (all integers little-endian)

File: 8-byte magic `Z3TRACE1`, then records. The core appends; it writes the
magic only when it opens an empty file.

Record: `u16 length` (bytes that follow), then:

| Offset | Size | Field |
|---|---|---|
| 0 | 1 | kind (see below) |
| 1 | 1 | stage: 0 none, 1 entry, 2 return, 3 presented |
| 2 | 8 | run (`retro_run` index) |
| 10 | 4 | frame (Snes9x completed-frame counter) |
| 14 | 2 | v (i16, V counter) |
| 16 | 4 | cycles (i32, CPU cycle) |
| 20 | 4 | pc (24-bit PB:PC) |
| 24 | 8 | a, x, y, s (u16 each) |
| 32 | 2 | carry, p |
| 34 | 4 | main, sub, subsub, frame_counter |
| 38 | 2 | room |
| 40 | 3 | lights_out, palette_countdown, palette_direction |
| 43 | 8 | link_y, link_x, bg2_v, bg2_h (u16 each) |
| 51 | 1 | mosaic_target |
| 52 | 4 | spotlight_radius, spotlight_state (u16 each) |
| 56 | 1 | spotlight_var4_low |
| 57 | 2 | spotlight_lower_cursor |
| 59 | 4 | rng_seed, nmi_latch, nmi_disable, nmi_pending |
| 63 | 4 | joypad_high, joypad_low, joypad_high_filtered, joypad_low_filtered |
| 67 | 31 | nmi_ppu_register_operands |
| 98 | 4 | return_address (24-bit) |
| 102 | 4 | stack1..stack4 |
| 106 | ... | tag/length/value tail |

Kinds: 1 frame, 2 video, 3 nmi, 4 nmi-resume, 5 pc, 6 dma, 7 hdma-start,
8 hdma-end, 9 rng-ppu-read, 10 ppu-read, 11 ppu-write, 12 rng-write,
13 wram-write, 14 pixel-write.

Tail entries are `u8 tag, u8 length, payload`. Scalar tags: 1 address (u32),
2 value (u32), 3 h_latched (u16), 4 channel (u8), 5 source (u32), 6 b_address
(u8), 7 bytes (u32), 8 mode, 9 fixed, 10 decrement (u8), 11 vram_address
(u16), 12 channels (u8); pixel-write fields use tags 20-35 (i32/u32). Tag 13
is one HDMA channel state per entry: `channel u8, source u32, table_address
u16, indirect, line_count, repeat, do_transfer, b_address, mode, data_len u8,
data[]`.

## Tools

```sh
# Render a trace (or a filtered slice) as canonical JSON Lines.
target/parity/zparity trace-decode SESSION/oracle-rom-random.jsonl --run 1371214
python3 scripts/snes9x_trace_format.py decode SESSION/oracle-rom-random.jsonl --event rng-write

# Convert a JSON Lines fixture into the binary format.
target/parity/zparity trace-encode fixture.jsonl fixture.bin
```

The session artifact keeps its historical name `oracle-rom-random.jsonl`
(manifests, caches, and provenance hashes refer to it by name) even though
its contents are now binary; `zparity trace-decode` is the way to read it.
Changing the trace patch changes the instrumented core's SHA-256, so the RNG
cache, cold receipts, and route authority bound to the previous core are
invalid and the pre-commit gate replays cold.
