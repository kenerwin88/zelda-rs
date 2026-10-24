# ROM timing feature gate

Branch: `codex/timing-gate`; base/unchanged HEAD: `480d0494`. No commit or push.

## Feature boundary

`rom-timing` is a default feature of `zelda3-bin`, `zelda3`, `snes`, and `parity`.
The binary forwards it to the game, source-machine marker, and parity tool crate;
`zelda3` also forwards it to `snes`. The reduced binary's Cargo dependency graph
contains zero `rom-timing` feature edges.
`parity-debug` remains independent. Default builds retain the existing tooling.
The `snes` feature is a forwarding marker: the source CPU interpreter, source
machine, logical frame ledger example, and their dependencies remain ungated.

The feature excludes the libretro/Snes9x comparison and recording modules,
cached-A/V entry point, semantic receipt producer and installation entry point,
ROM-shadow host execution/checkpoint entry points, cartridge play/replay commands,
live input recording, isolated cycle models, and their exclusive test fixtures.
Large receipt-install and shadow-execution methods were moved intact into
feature-only files. Shared data layouts were preserved; no checkpoint magic bump
was needed. The player forwarding macro now carries a method's cfg attribute.

A reduced binary rejects cartridge commands with exit 2. The runtime rejects
attempts to enable cartridge timing, including flags restored from a checkpoint.
The shared foreign-memory seeding API remains available to logical replay;
only the Snes9x wrapper that re-arms cartridge timing is gated.

The `parity` crate's A/V comparison, cold evidence, cache/index verification,
receipt comparison, runner, and pinned Snes9x trace decoder are gated as whole
modules. The `zparity` tool target requires the feature; `memmap2` and that crate's
`zstd` dependency are optional behind it. Hashing and frame digest records were
extracted into ungated `hash` and `digest` modules, with compatibility reexports
from the default-feature APIs. Logical replay keeps these hardware-independent
utilities and native coverage types.

Timing fixtures are gated through every timing activation path, including the
checkpoint-restoration API and inline helpers; the restoration API itself is
feature-only. Native checkpoint loading does not re-arm cartridge timing in a
reduced build.

The comparison lock and native crash-checkpoint reader were moved out of the
excluded comparison modules because surviving tools use them. Renderless logical
replay does not acquire the GPU comparison lock.

## Deletable pool

**41,305 nonblank production Rust lines** are behind `rom-timing`.
This counts physical lines, including comments, in gated module files and gated
items/command arms in shared files, plus the Cargo feature-gated tool target.
Inline test modules, test-only items, test
files, blank lines, and cfg-attribute glue are excluded. A file listed below may
be only partially gated. Shared timing code below is excluded from this total;
the pool is not a claim that every cartridge-era dependency can already be deleted.

| Module / file | Gated production lines |
| --- | ---: |
| `crates/parity/src/av.rs` | 339 |
| `crates/parity/src/cold_evidence.rs` | 614 |
| `crates/parity/src/evidence.rs` | 730 |
| `crates/parity/src/lib.rs` | 6 |
| `crates/parity/src/main.rs` | 517 |
| `crates/parity/src/parity_coverage.rs` | 300 |
| `crates/parity/src/receipts.rs` | 410 |
| `crates/parity/src/runner.rs` | 128 |
| `crates/parity/src/trace_format.rs` | 815 |
| `crates/zelda3/src/cycle_models.rs` | 5 |
| `crates/zelda3/src/cycle_models/dungeon_common.rs` | 58 |
| `crates/zelda3/src/cycle_models/dungeon_landing.rs` | 12 |
| `crates/zelda3/src/cycle_models/dungeon_module.rs` | 9 |
| `crates/zelda3/src/cycle_models/dungeon_push_blocks.rs` | 11 |
| `crates/zelda3/src/cycle_models/ground_animation_return.rs` | 1 |
| `crates/zelda3/src/cycle_models/link_main.rs` | 114 |
| `crates/zelda3/src/cycle_models/link_normal_prefix.rs` | 109 |
| `crates/zelda3/src/cycle_models/link_transform_guard.rs` | 71 |
| `crates/zelda3/src/cycle_models/overworld_player.rs` | 98 |
| `crates/zelda3/src/cycle_models/overworld_properties_caller.rs` | 14 |
| `crates/zelda3/src/cycle_models/poly_vertices.rs` | 1 |
| `crates/zelda3/src/cycle_models/spotlight_caller_return.rs` | 12 |
| `crates/zelda3/src/cycle_models/text_buffer.rs` | 276 |
| `crates/zelda3/src/cycle_models/y_item_lamp.rs` | 13 |
| `crates/zelda3/src/game_state/native/display.rs` | 10 |
| `crates/zelda3/src/game_state/native/dungeon.rs` | 5 |
| `crates/zelda3/src/game_state/native/inventory.rs` | 6 |
| `crates/zelda3/src/game_state/native/player.rs` | 1 |
| `crates/zelda3/src/game_state/native/player/presentation.rs` | 3 |
| `crates/zelda3/src/game_state/native/player_magic.rs` | 6 |
| `crates/zelda3/src/hud.rs` | 3 |
| `crates/zelda3/src/messaging.rs` | 22 |
| `crates/zelda3/src/nmi.rs` | 25 |
| `crates/zelda3/src/overworld.rs` | 12 |
| `crates/zelda3/src/poly.rs` | 100 |
| `crates/zelda3/src/timing_receipts.rs` | 3 |
| `crates/zelda3/src/zelda_rtl.rs` | 421 |
| `crates/zelda3/src/zelda_rtl/rom_timing_receipt_install.rs` | 1,003 |
| `crates/zelda3/src/zelda_rtl/rom_timing_shadow_execution.rs` | 1,240 |
| `crates/zelda3/src/zelda_rtl/rtl_actors.rs` | 40 |
| `crates/zelda3/src/zelda_rtl/rtl_checkpoint.rs` | 7 |
| `crates/zelda3/src/zelda_rtl/rtl_diagnostics.rs` | 1 |
| `crates/zelda3/src/zelda_rtl/rtl_native_exact_cpu.rs` | 1,349 |
| `crates/zelda3/src/zelda_rtl/rtl_original_timing.rs` | 9 |
| `crates/zelda3/src/zelda_rtl/rtl_save.rs` | 3 |
| `crates/zelda3/src/zelda_rtl/rtl_types_display.rs` | 43 |
| `zelda3-bin/src/audio_trace.rs` | 57 |
| `zelda3-bin/src/dsp_phase_ledger.rs` | 744 |
| `zelda3-bin/src/gpu_capture.rs` | 50 |
| `zelda3-bin/src/image_output.rs` | 6 |
| `zelda3-bin/src/libretro_core.rs` | 1,106 |
| `zelda3-bin/src/libretro_timeline.rs` | 408 |
| `zelda3-bin/src/live_input_recording.rs` | 134 |
| `zelda3-bin/src/main.rs` | 1,121 |
| `zelda3-bin/src/play_commands.rs` | 57 |
| `zelda3-bin/src/play_renderer.rs` | 20 |
| `zelda3-bin/src/replay_diagnostics.rs` | 407 |
| `zelda3-bin/src/replay_save_config.rs` | 247 |
| `zelda3-bin/src/smp_opcode_ledger.rs` | 659 |
| `zelda3-bin/src/snes9x_apu_tools.rs` | 1,185 |
| `zelda3-bin/src/snes9x_compare.rs` | 6,802 |
| `zelda3-bin/src/snes9x_compare/cached_av.rs` | 142 |
| `zelda3-bin/src/snes9x_compare/display_oracle.rs` | 1,299 |
| `zelda3-bin/src/snes9x_compare/libretro_session.rs` | 1,693 |
| `zelda3-bin/src/snes9x_compare/paired_resume.rs` | 995 |
| `zelda3-bin/src/snes9x_compare/replay_bundle.rs` | 630 |
| `zelda3-bin/src/snes9x_compare/rng_trace.rs` | 140 |
| `zelda3-bin/src/snes9x_compare/smp_trace.rs` | 1,933 |
| `zelda3-bin/src/snes9x_compare/video_compare.rs` | 151 |
| `zelda3-bin/src/snes9x_frame_dump.rs` | 113 |
| `zelda3-bin/src/snes9x_presented_bg_scroll.rs` | 56 |
| `zelda3-bin/src/snes9x_presented_bg_tilemaps.rs` | 138 |
| `zelda3-bin/src/snes9x_presented_dialogue_text.rs` | 31 |
| `zelda3-bin/src/snes9x_presented_mode7.rs` | 37 |
| `zelda3-bin/src/snes9x_presented_window_mask.rs` | 105 |
| `zelda3-bin/src/snes9x_record_commands.rs` | 986 |
| `zelda3-bin/src/snes9x_route_recorder.rs` | 674 |
| `zelda3-bin/src/snes9x_segment_matrix.rs` | 293 |
| `zelda3-bin/src/snes9x_semantic_receipts.rs` | 8,200 |
| `zelda3-bin/src/snes9x_semantic_receipts/consume_event_arms.rs` | 1,711 |

## Retained entanglements, largest first

Sizes are conservative source envelopes (nonblank production lines, inline tests
excluded), not additive deletion estimates. A group includes ordinary shared
logic as well as timing machinery; its whole size is **not** deletable.

| Size | Source boundary | What lagless still uses | Suggested native replacement |
| ---: | --- | --- | --- |
| ~83,900 | `crates/zelda3/src/cycle_models.rs:9`; `zelda_rtl/rtl_poly.rs:16`; `messaging.rs:85` | Native routine/caller types, graphics conversion/decompression work, VWF work accounting, polygon worker instruction stepping, and their transitive models. Many routines still reference these even when cartridge scheduling is off. | Separate semantic return/progress types from instruction pricing; give lagless VWF and polygon work explicit native work budgets and keep conversion/decompression as ordinary native transforms. |
| ~23,700 | `crates/zelda3/src/zelda_rtl/rtl_original_timing.rs:7292`; `rtl_frame_lanes.rs:15`; `game_execution_scheduler.rs:1`; `zelda_rtl/rtl_cpu_schedule.rs:8` | The public frame path still calls the common dispatcher. Its ordinary main iteration, display snapshot, trailing NMI, audio, and scheduler bookkeeping share the same body and lane helpers with cartridge-only branches. | Extract a native frame orchestrator around main iteration, logical publication and NMI/audio completion; split the remaining cartridge continuation dispatcher afterward. This would be a substantial behavioral refactor, so it was not attempted here. |
| ~5,300 plus shared state fields | `crates/zelda3/src/timing_receipts.rs:1`; `zelda_rtl/rtl_types_cpu_plans.rs:1`; `zelda_rtl/rtl_types_original_timing.rs:1`; `zelda_rtl.rs:10407` | Public presentation/continuation vocabulary and runtime state still name timing plans, checkpoints and receipt types. This includes inactive option/queue fields and common NMI/publication helpers. | Split native publication/progress records from cartridge authority records, then gate the cartridge-only fields and migrate checkpoint layouts deliberately. |
| ~3,200 | `crates/zelda3/src/zelda_rtl/native_item_graphics.rs:5`; `native_overworld_exit.rs:1`; `native_overworld_properties.rs:1`; `native_dungeon_main_return.rs:1`; `native_spotlight_caller.rs:1`; `native_push_blocks.rs:1` | Shared gameplay helpers refer to retained CPU handoff/return types and native progress machinery; many timing-specific operations are no-ops without a retained owner. | Keep the semantic graphics/item/transition operations, and replace retained CPU handoffs with native continuation enums at caller boundaries. |
| ~1,800 | `crates/zelda3/src/zelda_rtl/rtl_native_exact_cpu.rs:1`; `zelda_rtl.rs:11062` | Ungated gameplay accessors inspect optional source observations and retained cursor/trace types. The actual shadow host runner and its checkpoint entry points are gated. | Split observational query vocabulary from the shadow executor, then gate the optional owner fields and their query callers together. Narrow no-feature dead-code allowances on record-only types preserve the shared vocabulary; no broad module-level warning suppression was added. |
| ~1,600 | `crates/zelda3/src/spc_driver_clock.rs:1`; `zelda_rtl/rtl_audio.rs:1` | Shared music/driver state and audio publication still use source clock/port contracts independently of cartridge frame scheduling. | Separate native song/port state from CPU-master cadence; preserve the real sample clock and audio state machine. |
| ~1,100 | `crates/zelda3/src/zelda_rtl/rtl_poly.rs:113`; `cycle_ledger.rs:1` | `lag_free_main_master` and the lag-free poly phases use measured main/VWF work to advance the polygon thread and its logical publication. | Native work counters plus an explicit polygon-worker budget; validate publication against the surviving logical ledger before changing costs. |
| ~550 | `crates/zelda3/src/rom_cpu_timing.rs:1`; `zelda_rtl.rs:14549` | Shared checkpoint/address helpers and routine timing adapters remain referenced by the native work/handoff family. Profiling host notification also remains in the common frame entry. | Separate ROM data/address helpers from probe execution, then replace shared native caller inputs with semantic operands. |
| ~210 | `crates/zelda3/src/raster_timing.rs:1` | Shared timing position/budget vocabulary remains in native publication and continuation paths. | Native phase/work-budget types for lagless; retain hardware raster types only in the source/oracle layer. |

The `snes` source interpreter and synchronous source machine are intentional
survivors, **not timing-code deletion candidates**. Their hardware clocks and
instruction semantics remain available to `logical_frame_ledger.rs`.
The logical replay's optional rendering utilities also remain shared; no GPU
lane was run for this work.

## Verification

Completed build checks (no warnings):

- `cargo build --profile parity -p zelda3-bin --no-default-features --features parity-debug`
- `CARGO_TARGET_DIR=target/release-check cargo check --release -p zelda3-bin --no-default-features`
- `cargo test --profile parity -p zelda3 --lib --no-default-features --features parity-debug --no-run`
- `cargo build --profile parity -p zelda3-bin`

The reduced binary was copied to `/tmp/zelda3-no-rom-timing-gate` before rebuilding
defaults. SHA-256: `ba09d9caa9ab6cd6319840f93ff012c6e76df13632afc39a94fe04f7d49b5c7a`.
The final default binary is `target/parity/zelda3`, SHA-256:
`085c958b6bd0efa045a4152cea5da63dcecbec315a44237f5dd99ed5119a4301`.
Their sizes are 24,947,248 and 31,680,928 bytes respectively.

The reduced binary rejects `--rom-timing` and a live input recording request
with exit 2 before launching a frontend or opening a recording session.
All eight moved runtime/loader function bodies were compared with HEAD and were
unchanged apart from visibility and qualified module/type names. The extracted
hashing tests pass: 2/2 without `rom-timing`; the default `parity` library suite
passes 29/29, including the A/V, receipt, and cache fixtures.

Completed 200k-frame comparisons against the supplied baselines:

| Final binary | Mode | Poison owned WRAM | Result |
| --- | --- | --- | --- |
| Reduced | lag-free | 0 | All records and EOF match |
| Reduced | lag-free | 1 | All records and EOF match |
| Default | default | 0 | All records and EOF match |
| Default | default | 1 | All records and EOF match |
| Default | lag-free | 0 | All records and EOF match |
| Default | lag-free | 1 | All records and EOF match |

Default library suite (`ZELDA3_ASSETS=$PWD/zelda3_assets.dat cargo test --profile
parity -p zelda3 -p snes --lib`): SNES 504 passed / 7 ignored; Zelda 2,454 passed /
3 ignored / 1 failed. The sole failure is the allowed existing
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
The default timing fixture remains enabled; it is gated out of the reduced suite.

Reduced Zelda library suite (`ZELDA3_ASSETS=$PWD/zelda3_assets.dat cargo test
--profile parity -p zelda3 --lib --no-default-features --features parity-debug`):
**1,989 passed, 2 ignored, zero failures**. The reduced SNES suite passed
504 tests with 7 ignored; its source CPU modules remain ungated.

All six comparisons matched every record and EOF against
`/tmp/zelda3-ownership-2c.zP2HU7/{default,lag-free}-200000.jsonl.zst`.
The main-binary re-record fallback was not needed. Both full renderless logical censuses exited 0. They replayed all 1,473,123
source-ledger iterations with `--report 0 --seed-every 2000`, using
`/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full` and the supplied
read-only cache's `initial.srm`. Separate `ZELDA3_SAVE_DIR` directories beneath
`/tmp` isolated SRAM persistence.

`scripts/logical_census_summary.py` produced byte-identical summaries. All **766**
segment, polygon, and final-result records also matched exactly. Both binaries
ended with `1466939 clean iteration(s); first WRAM divergence None`.

```text
timing-defined segments: 1 (2000 iterations), reported apart
segments: 738; iterations: 1471123; fully clean: 738; wram-clean: 738; panicked: 0
polygon-thread iterations: 26500; polygon WRAM differences: 5931
lag-caused WRAM differences: 227
RNG unconsumed / extra: 0 / 0; APU reads unconsumed / extra: 2 / 0
APU commands engine / ROM: 65240 / 67254; unmatched: 0 / 0
lag-only commands: 270; polygon-thread commands: 180; held re-sends: 1884
```

The separately classified polygon/lag differences and two unconsumed APU reads
are identical between feature configurations. This is a preservation check;
no new source-parity improvement is claimed. Video comparison was disabled.

Evidence logs and summaries:

- `/tmp/timing-{default,norom}-census-current.log`
- `/tmp/timing-{default,norom}-census-summary.txt`
- `/tmp/timing-default-current-{default,lag-free}-poison-{0,1}.log`
- `/tmp/timing-norom-ledger-poison-{0,1}.log`
- `/tmp/timing-default-tests-final.log`
- `/tmp/timing-norom-tests-verified.log`

No GPU, `--video`, cached-A/V, Snes9x-core, or Cargo GPU-test run was performed.
No oracle-cache writes, `cargo fmt`, commit, or push were performed.
`git diff --check` passed.
The source logical ledger, supplied regression baselines, and oracle cache are
read-only; no new baseline has been recorded.

## LOC accounting

`python3 scripts/loc.py HEAD` before edits:

```text
rev          date            rust files rust tests files  python   wgsl  shell     total
480d0494     2026-10-23   558,655   564     92,383   111  41,564  1,270    618   694,490
```

The same command after the edits produced exactly the same row:

```text
480d0494     2026-10-23   558,655   564     92,383   111  41,564  1,270    618   694,490
```

`HEAD` remains unchanged because this task explicitly forbids a commit; the
command counts a revision, not the working tree. The feature pool above counts
the working tree.
