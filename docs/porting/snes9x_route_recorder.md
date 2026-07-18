# Human Snes9x route recorder

The route recorder makes Snes9x 1.63 the playable, audible oracle while it
captures deterministic inputs and reusable native boundaries. Rust gameplay,
rendering, and audio do not execute in the recording loop. Snes9x is never
used by the production game path.

## Start or continue recording

```sh
python3 scripts/snes9x_route_recorder.py record
```

The default project is `routes/default`. The launcher pins
the actual core and ROM hashes on every invocation and supplies
`saves/sram.dat`. Use `--blank-sram` for a clean SRAM image.

Treat one recorder project as one SRAM lineage. A new project is seeded from
the requested `--sram` file (default `saves/sram.dat`) or from blank SRAM.
Every boundary captures its own `sram.bin`; boundary reload and paired oracle
comparison verify that file's hash and restore it explicitly after loading the
native Snes9x state. Later edits to the original `.srm` therefore cannot alter
an existing project's saved boundaries. Use a different `--project` directory
when starting from a different SRAM file.

## Terminal browser

Use the TUI instead of remembering individual commands:

```sh
./scripts/snes9x_route_recorder.py
```

`python3 scripts/snes9x_route_recorder.py tui` is the equivalent explicit
form. All existing command-line subcommands remain available.

The left pane lists recorder projects. The right pane lists boundaries or
takes, and the bottom shows the absolute project, SRAM origin, and selected
artifact directory. Controls:

- `Tab` or left/right: switch panes.
- Up/down or `j`/`k`: browse; long lists scroll automatically.
- `Enter`: resume the selected boundary in Snes9x.
- `n`: rename the selected boundary.
- `o`: open its screenshot.
- `t`: switch between boundaries and takes.
- `m`: on an intermediate boundary, combine its incoming and outgoing takes
  into one active take. The boundary and source takes become hidden, while all
  original files remain available as provenance under `v`.
- `x`: archive/restore the selected boundary or discard/restore the selected
  take. Hidden items retain all state, SRAM, input, and receipts.
- `v`: show or conceal archived/discarded items so they can be restored.
- `a`: create and immediately record a new blank- or file-SRAM route.
- `r`: refresh; `q` or Escape: quit.

The browser establishes its own high-contrast dark palette instead of
inheriting a potentially low-contrast light terminal theme. Terminals without
color support retain the monochrome reverse-video fallback.

```sh
python3 scripts/snes9x_route_recorder.py record \
  --project routes/my-clean-route --blank-sram
python3 scripts/snes9x_route_recorder.py record \
  --project routes/my-completed-save \
  --sram saves/completed-game.srm
```

Recorder controls are host-only and never enter the SNES controller stream:

- `F5`: save a new Snes9x-native boundary, finish the current take, and begin
  the next take.
- `F9`: load the previous boundary and begin a branched take.
- `F10`: load the next boundary and begin a branched take.
- Close the window: finish the current take and auto-save its ending as a new
  Snes9x-native boundary. Reopening `latest` resumes from that exact point.

To start from an exact saved boundary instead of browsing with the function
keys:

```sh
python3 scripts/snes9x_route_recorder.py record --start 7
```

List every boundary and take with its milestone summary:

```sh
python3 scripts/snes9x_route_recorder.py list
```

Give a boundary a memorable name, then resume by either its number or name:

```sh
python3 scripts/snes9x_route_recorder.py name 7 "Eastern Palace entrance"
python3 scripts/snes9x_route_recorder.py record --start "Eastern Palace entrance"
```

Running `name` again for the same boundary renames it. Names are kept in the
oracle project without modifying or converting the native Snes9x state.

If a take was accidental, exclude it from the parity matrix without deleting
its evidence. It can be restored later:

```sh
python3 scripts/snes9x_route_recorder.py discard-take 12
python3 scripts/snes9x_route_recorder.py restore-take 12
```

## Captured artifacts

Each boundary contains:

- `oracle.state`: full Snes9x state created by `retro_serialize`; this includes
  CPU, PPU, VRAM, APU/S-DSP, timing, WRAM, and SRAM state.
- `wram.bin`, `vram.bin`, and `sram.bin`: direct memory dumps for inspection.
- `frame.png`: the visible boundary frame.
- SHA-256 provenance, core version, core hash, ROM hash, and a structured
  gameplay milestone summary.

Each take contains:

- `input.txt`: exact controller state once per Snes9x frame.
- `frame_receipts.jsonl`: per-frame input, normalized video hash, raw oracle
  PCM hash and sample count, modules, room/map, coordinates, health/magic,
  selected item, progression flags, follower, music/SFX controls, and ending
  state.
- Start/end boundary IDs and branch lineage in `manifest.json`.

Receipts flush every 60 frames. If the process is interrupted before a take is
finalized, reopening the project reconstructs its input script from the
receipt journal and marks the take `recovered_after_interruption`.

Raw video and PCM are not stored for every frame. The native start state plus
the input stream reproduces them exactly, while the hashes detect drift. This
keeps a million-frame route practical without discarding diagnostic state.

Projects under `routes/` are version-control artifacts. Native boundaries,
memory dumps, screenshots, compact inputs, labels, and manifests are included.
Per-frame receipt journals and regenerated comparison sessions remain local
and ignored because they scale linearly. `scripts/check_route_artifacts.py`
verifies referenced files and hashes and enforces 10 MiB per-file and 50 MiB
per-project limits for the versioned subset.

## Pair and compare a take

Pairing records paths and hashes only. It never converts a Rust state into a
Snes9x state or a Snes9x state into Rust:

```sh
python3 scripts/snes9x_route_recorder.py pair 7 \
  target/parity/snes9x-segment-matrix/segment-08/rust_start.z3state
```

Then run exact completed-video and continuous-PCM comparison with the modern
Rust renderer, modern audio backend, and native sequencer:

```sh
python3 scripts/snes9x_route_recorder.py compare 12
```

Compare every reset-start or paired take and write an aggregate, explicitly
segmented coverage result:

```sh
python3 scripts/snes9x_route_recorder.py compare-all
```

The aggregate is strict: it lists every nonempty recorded take, identifies
unpaired takes under `excluded_takes`, and exits unsuccessfully until every
recorded take is comparable and passes both video and exact modern-audio
parity. It never reports full recorded coverage by silently skipping a take.

The take's start boundary chooses both members of the pair automatically. A
take from reset boundary 0 needs no pairing and compares from clean ROM start.
Comparison failures do not affect recording or corrupt the oracle project.

## Recommended route workflow

Play normally in Snes9x. Press `F5` at stable chapter milestones such as an
overworld/dungeon entrance or immediately after a major item/crystal receipt.
If a segment should be replayed, use `F9`/`F10` or relaunch with `--start N` and
record another take. Once a corresponding Rust checkpoint exists, pair that
boundary and compare the take offline. This produces broad independently
restartable coverage while keeping each parity failure localized.
