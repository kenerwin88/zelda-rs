# Recorded routes

This directory contains version-controlled human routes recorded directly in
the pinned Snes9x 1.63 libretro oracle. One child directory is one independent
SRAM lineage.

Tracked route evidence includes:

- `manifest.json`, `labels.json`, and `sram-origin.json`;
- every Snes9x-native boundary state and its WRAM, VRAM, SRAM, and screenshot;
- each take's compact controller `input.txt` and `status.json`.

Per-frame `frame_receipts.jsonl` and regenerated comparison sessions remain on
disk but are ignored because they grow linearly with route length. The native
boundary plus input stream can reproduce them with the pinned core and ROM.

Run the browser from the repository root:

```sh
./scripts/snes9x_route_recorder.py
```

Run `python3 scripts/check_route_artifacts.py` before committing route changes.
The guard rejects any versioned file above 10 MiB or route package above
50 MiB; move newly oversized diagnostics to the ignored/generated set instead
of silently bloating Git history.

## Oracle-only segment replay

The native Snes9x boundary and its take input can also be replayed without
waiting for the translated runtime to reach that boundary:

```sh
cargo run -p zelda3-bin -- \
  --run-snes9x-script <trace-core> <rom.sfc> \
  routes/clean/boundaries/0003/oracle.state \
  routes/clean/takes/0005/input.txt <frames> \
  --load-sram routes/clean/boundaries/0003/sram.bin \
  --expected-core-sha256 <sha256> \
  --expected-rom-sha256 <sha256>
```

Use `--input-frame-offset <frames>` when replaying a suffix of the take and
`--save-state <path>` to create a disposable native checkpoint. Trace-core
environment variables work normally, so this command can collect CPU, NMI,
DMA, and DSP evidence from later route segments independently of current Rust
parity.
