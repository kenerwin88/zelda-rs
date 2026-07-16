# Snes9x live parity oracle

The Snes9x harness is a black-box product-boundary oracle. It runs the original
ROM and the Rust engine from the same SRAM, feeds both the same 16-bit SNES
controller word once per game frame, and observes completed video frames plus
the continuous stereo PCM callback stream.

It deliberately does not reproduce Snes9x internals in production code. The
core is dynamically loaded only by parity commands.

The downloaded core remains an ignored, external test dependency; it is not
vendored into Zelda3-rs release artifacts. Review the upstream Snes9x license
before redistributing a core binary.

## Run an existing route

```bash
python3 scripts/full_parity.py \
  --with-snes9x \
  --rom /path/to/zelda3.sfc \
  --frames 120000 \
  --input-script scripts/inputs/file-select-enter-game-button-taps.txt \
  --load-sram /path/to/initial.srm
```

On macOS arm64 the first run downloads the current Snes9x Libretro dylib. Set
`SNES9X_LIBRETRO_CORE` or pass `--snes9x-core` to use a specific build. Every
receipt records the core SHA-256 and reported Snes9x version, so a changing
nightly cannot silently change an existing result.

The live gate has two audio lanes:

- `timing` runs the production native sequencer and compares cumulative
  duration, callback continuity, audible/silent windows, onset/offset edges,
  envelope error, and whole-stream hashes. A measured lag is reported as a
  failure; it is never aligned away.
- `exact` enables the local exact SPC/DSP oracle and compares every interleaved
  sample against Snes9x. This proves the reference side and isolates game/APU
  command timing from the modern sequencer.

Video is compared at the completed 256x224 frame boundary. `--scan-all` records
all mismatch ranges and still completes the audio comparison, rather than
letting an early boot-logo difference hide a later sound failure.

Supplying `--session-dir` implies `--scan-all`, because a replayable receipt must
represent the complete requested interval rather than an early-exit fragment.

## Capture a manual failure

```bash
ZELDA3_RECORD_INPUT_SESSION="$PWD/target/manual-pot-route" \
  cargo run -p zelda3-bin -- /path/to/zelda3.sfc
```

Play normally and quit after reproducing the problem. The recorder writes one
JSONL event per game-frame controller poll and flushes it every 60 frames. On a
clean exit it also emits a run-length encoded `input.txt`, the initial SRAM,
the initial Rust state, and a result manifest.
Developer checkpoint warps are rejected while recording because they mutate
Rust state in a way a clean ROM boot cannot reproduce.

Replay the capture with:

```bash
python3 scripts/replay_snes9x_session.py \
  target/manual-pot-route \
  --rom /path/to/zelda3.sfc
```

This runs both the production timing lane and the exact local-APU lane. The
output directory contains:

- `manifest.json`: ROM/core hashes, core version, frame rate, sample rate, and
  comparison thresholds.
- `input.txt`: the exact controller stream.
- `frame_receipts.jsonl`: per-frame input and callback sample counts.
- `audio_frame_ends.json`: absolute continuous sample positions for every game
  frame.
- `audio_report.json`: first mismatch, total sample counts, waveform hashes,
  activity mismatches, envelope error, and measured edge lag.
- `oracle_initial.state`, `oracle_last_before.state`, and
  `oracle_final.state`: real Libretro save states.
- `result.json`: complete audio and video verdicts plus video mismatch ranges.
- `replay.sh`: a self-contained rerun using the captured SRAM and input stream;
  it aborts if the ROM or core SHA-256 no longer matches the capture.

## Alignment policy

The harness permits only an explicit, fixed startup offset through
`--skip-oracle-frames`. It records that value in the manifest. Snes9x mode
rejects `--auto-align-video`; a late modern frame must remain visible as a late
frame. For a save-state route, start both sides from the captured SRAM and the
same controller stream instead of searching forward for a matching picture.

Host focus and alt-tab behavior is outside the ROM oracle boundary. Those
paths must be tested through the platform audio queue and resume lifecycle.
