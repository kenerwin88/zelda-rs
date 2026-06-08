# Mesen2 Oracle Trace

This directory holds a durable Mesen2 Lua oracle for SNES/APU timing work.
It is separate from the Rust runtime and is meant to answer questions that the
current bsnes libretro wrapper cannot expose through callbacks:

- when the S-CPU writes APUI ports `$2140-$2143`;
- when the SPC program writes DSP address/data registers `$f2/$f3`;
- which video frame those writes land in;
- whether the first title SFX command `$0a` is delayed, repeated, or paired
  with different DSP writes in a reference emulator.

The final audio/video oracle should still be bsnes or ares. Mesen2 is used here
as a trace/debug oracle because its Lua debugger API can hook SNES and SPC
memory writes.

## Script

`trace_apui_dsp.lua` writes JSON lines. Each event has at least:

- `kind`: `start_frame`, `end_frame`, `nmi`, `apui_write`,
  `spc_dsp_addr`, `spc_dsp_data`, `stop`;
- `frame`: frame counter maintained by the Lua script;
- `addr` / `value`: for memory-write events.

For DSP writes, the script watches SPC writes to `$f2` and `$f3`. `$f2` selects
the DSP register, and the following `$f3` write emits the selected register in
`dsp_reg`.

## Running

Set an output path and frame limit, then launch Mesen2 with the Zelda ROM and
this script. The bundled runner uses Mesen2 test-runner mode and enables Lua
file I/O for the process without requiring a persistent settings change.

The local Mesen2 app/binary can live under `external/mesen2-oracle/local/`.
That directory is ignored by git so the emulator download persists in this
workspace without being committed.

```sh
external/mesen2-oracle/run_trace.sh \
  /Users/missingno/Documents/zelda3/zelda3.sfc \
  180 \
  external/mesen2-oracle/startup-apui-dsp.jsonl
```

The equivalent raw command is:

```sh
M2_TRACE_OUT=/Users/missingno/Documents/zelda3-rs/external/mesen2-oracle/startup-apui-dsp.jsonl \
M2_TRACE_FRAMES=180 \
external/mesen2-oracle/local/Mesen.app/Contents/MacOS/Mesen \
  --testRunner \
  --enableStdout \
  --doNotSaveSettings \
  --timeout=180 \
  --debug.scriptWindow.allowIoOsAccess=true \
  external/mesen2-oracle/trace_apui_dsp.lua \
  /Users/missingno/Documents/zelda3/zelda3.sfc
```

Mesen2's `--testRunner` syntax is positional: Lua script first, ROM second.
Generated `*.jsonl` traces are ignored by git because startup captures can
quickly reach tens of megabytes.

If your build cannot run scripts from `--testRunner`, load the Lua file from
Mesen2's debugger UI and keep the same environment variables. If Lua file I/O is
disabled in the UI, enable Lua I/O access in Mesen2's scripting settings or
change the `output_path` constant at the top of the script.

## Comparing

The first useful comparison is startup:

1. Run Mesen2 for roughly 180 frames and save `startup-apui-dsp.jsonl`.
2. Run Rust:

```sh
cargo run -q -p zelda3-bin -- --trace-startup-audio \
  /Users/missingno/Documents/zelda3/zelda3.sfc 180
```

3. Compare the first APUI `$2143 = $0a` write frame with Rust frame 0's
   `ports=[00, 00, 00, 0a]`, then compare the subsequent DSP `$f2/$f3` writes.

The important question is whether the bad title SFX is caused by command timing
against video frames, DSP register programming, or sample playback after the
register writes match.
