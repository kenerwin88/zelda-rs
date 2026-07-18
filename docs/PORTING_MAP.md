# Porting Map

This is the working ledger for porting the original C project into
`crates/zelda3`. The C source at `../zelda3/src` remains
authoritative; this map only tracks what has been copied, verified, deferred, or
left untouched.

## Status Legend

| Status | Meaning |
|---|---|
| `done` | Ported and covered by tests or a passing oracle window |
| `partial` | Some behavior exists in Rust, but the C surface is not complete yet |
| `seed` | Types/helpers/constants exist to support later ports |
| `stub` | Placeholder exists and intentionally does not match C yet |
| `deferred` | Out of the current correctness path |
| `not-started` | No meaningful Rust port yet |

## Current Oracle Boundary

Verified command:

```bash
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 30000
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 5000 --input-script scripts/inputs/title-start.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 5000 --input-script scripts/inputs/file-select-new-game.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 107000 --input-script scripts/inputs/file-select-enter-game.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 122000 --input-script scripts/inputs/file-select-enter-game-message-dismiss-and-wander.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 116000 --input-script scripts/inputs/file-select-enter-game-diagonal-sweeps.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 112000 --input-script scripts/inputs/file-select-enter-game-button-taps.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 36810 --input-script scripts/inputs/opening-uncle-message-dismiss-and-move.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 45610 --input-script scripts/inputs/opening-uncle-message-extended-move.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 45610 --input-script scripts/inputs/opening-uncle-message-diagonal-sweeps.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 50000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 32613 --input-script scripts/inputs/tas-us-rta-ace.txt
```

The no-input intro/attract loop matches WRAM/SRAM/VRAM through 30,000 frames
against `/path/to/zelda3.sfc`
(`WRAM fnv1a64 = a5077d3d9ba1c106`). The current verified surface includes the
title intro, story text, world map, throne room, Zelda prison scene, maiden-warp
Agahnim altar scene, palette flash, and the end-of-story transition back into
intro state.

The recorded title Start path in `scripts/inputs/title-start.txt` also matches
through 5,000 frames (`WRAM fnv1a64 = 51cc44fb5e0d1655`). That input leaves the
title loop, runs the initial file-select setup, loads file-select graphics,
emits the first stripe uploads, and idles on the file-select main screen.

The recorded new-file path in `scripts/inputs/file-select-new-game.txt` matches
through 5,000 frames (`WRAM fnv1a64 = 6e3be721daf0d42b`). That input selects
the first empty slot, enters name-file mode, types one character, saves the
new file, returns to file select, and idles while drawing the occupied slot.

The TAS-derived full-completion SMV now carries its extracted reset SRAM sidecar
and matches through 50,000 frames (`WRAM fnv1a64 = cc117ab564b664e4`). The
embedded SRAM is a uniform 0x20000-byte `0x60` buffer, and the oracle remains in
`Module04_NameFile` through that window, so this is name/file-screen stress
coverage rather than broad gameplay coverage.

The US BizHawk/snes9x RTA TAS route in `scripts/inputs/tas-us-rta-ace.txt`
matches through 32,613 frames (`WRAM fnv1a64 = f01523b0bba49471`). This is the
current broad gameplay route: it starts from reset, skips through title/file
setup, advances Uncle's opening sequence, leaves Link's house, crosses the
outside transition, and exercises early overworld movement. It is a better
starting-house exit and overworld-scroll parity gate than the Snes9x
full-completion SMV above.

The canonical full-playthrough replay-save route is the stitched C reference
replay save at `saves/zelda3-combined-route.sav`, with its proof manifest at
`saves/zelda3-combined-route-proof.json`. Keep it in `saves/`; do not use
`/tmp/zelda3-combined-route.sav`, because `/tmp` is disposable.

Generate or re-check the C oracle route from the C checkout with:

```bash
cd ../zelda3
SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy SDL_RENDER_DRIVER=software \
    ./zelda3 --config other/headless_replay.ini \
    --replay-save ../zelda3-rs/saves/zelda3-combined-route.sav \
    --smv-test-frames 1073092 \
    --dump-frame /tmp/c-combined-final.ppm \
    zelda3.sfc
```

Use this exact Rust command when validating the full route:

```bash
cd .
env ZELDA3_SMV_SELECT_FILE_TIMING_HACKS=1 \
    ZELDA3_SMV_LOADFILE_TIMING_HACKS=1 \
    ZELDA3_SMV_DUNGEON_TIMING_HACKS=1 \
    ZELDA3_SMV_OVERWORLD_TIMING_HACKS=1 \
    ZELDA3_SMV_MESSAGING_TIMING_HACKS=1 \
    ZELDA3_SMV_DEATH_INTRO_TIMING_HACKS=1 \
    ZELDA3_SMV_DEATH_RELOAD_TIMING_HACKS=1 \
    target/release/zelda3 --replay-save \
    /path/to/zelda3.sfc \
    saves/zelda3-combined-route.sav \
    1073092 \
    --dump-frame /tmp/rust-combined-final.png
```

The C reference route completes all `1,073,092` frames and reaches the ending
with `ending=1 main=26 sub=38 saved=25`, `big=0x77fc`, `item=0x01`, and
`active=0x03`. Rust currently consumes all frames with `active=false` but ends
early at `ending=0 main=7 sub=0 saved=7`, room `0x008c`, Link position
`x=0x18ac y=0x1030`, `item=0x08`, `active_item=0x0f`, `hp=0x88`, and
`big=0x77f8`. `scripts/replay_bisect.py --good 216606 --bad 1073092`
currently finds the first normalized C/R checkpoint divergence at frame
`241932`: C enters `main=11 sub=42`, while Rust remains at `main=11 sub=0`.
Treat the replay save and its proof manifest as the canonical full-playthrough
replay-save input, not as proof that Rust already matches the C reference. It
is separate from reset-start oracle windows and from true video reference
checks; direct visual reference still needs the C renderer or the snes9x/libretro
path.

The recorded opening movement path in
`scripts/inputs/opening-uncle-message-dismiss-and-move.txt` matches through
36,810 frames (`WRAM fnv1a64 = 893772a0cd71d8ce`). It enters the new game,
advances Uncle's opening messages, dismisses the telepathy message, and covers
indoor cardinal movement, directional Link OAM/DMA, and the pre-velocity
direction-limit/slope probes. It now includes later Uncle-message dismissal,
post-dismissal cardinal movement, and repeated grab/pull probes including the
left-facing grab/pull body and shadow OAM/DMA path.

The recorded gameplay-entry path in `scripts/inputs/file-select-enter-game.txt`
extends that flow by selecting the occupied slot. It now matches through 107,000
frames (`WRAM fnv1a64 = 3159b761a47fa0d6`). The Rust path matches the
high-level load-file/player-bed state, C-style `dung_load_ptr_offs`, active
south/exit doors, entrance tileset rows, room-entry camera cache, dungeon
palette setup, active HUD rebuild/upload, room quadrant save state, active
dungeon tile-attribute overrides, first gameplay-frame setup, Uncle message
trigger/dismissal, repeated saved-slot indoor movement/A probes, chest-tile
A-action classification, and later in-room message trigger/dismissal cycles
from checkpointed frontier probes.

The extension route
`scripts/inputs/file-select-enter-game-message-dismiss-and-wander.txt` includes
that base script and matches through 122,000 frames
(`WRAM fnv1a64 = ed37eeb35721bf24`). It dismisses the active 107,000-frame
message frontier, exercises more in-house movement/OAM churn and A-button
probes, then enters another in-room message cycle.

The extension route `scripts/inputs/opening-uncle-message-extended-move.txt`
includes the existing opening movement path and matches through 45,610 frames
(`WRAM fnv1a64 = 4d3c07b597e279db`). It adds longer in-house movement sweeps
and more A-button probes after the prior opening route frontier.

Additional green branches cover diagonal saved-slot movement
(`file-select-enter-game-diagonal-sweeps.txt`, 116,000 frames,
`573328193bac854d`), isolated saved-slot button taps
(`file-select-enter-game-button-taps.txt`, 112,000 frames,
`3098304aa5048c6f`), diagonal opening movement
(`opening-uncle-message-diagonal-sweeps.txt`, 45,610 frames,
`e9bec02dd1924404`), and combined saved-slot button+direction probes
(`file-select-enter-game-button-probes.txt`, 114,000 frames,
`aefe8ff680a60dea`). The combined probe route clears the previous frame
108,662 `X+UP` movement frontier by matching the C cardinal-collision nudge
helpers.

The machine-readable oracle ledger is `docs/porting/oracle_windows.tsv`.
Validate it with:

```bash
python3 scripts/oracle_windows.py --check
```

Run every passing oracle window with:

```bash
python3 scripts/oracle_windows.py --run
```

By default, `--run` uses recorded checkpoints and Cargo's release profile so
long windows replay only the current unverified tail. The slow authoritative
cold/debug path is still available when needed:

```bash
cargo build --release -p zelda3-bin
python3 scripts/oracle_windows.py --run --only file-select-enter-game
python3 scripts/oracle_windows.py --run --cold --debug --only file-select-enter-game
```

The checkpoint ledger is `docs/porting/oracle_checkpoints.tsv`. Validate both
the oracle windows and local checkpoint files with:

```bash
python3 scripts/oracle_windows.py --check --check-checkpoints
```

## How To Use This Map

1. Pick the first `partial` item in the active queue.
2. Port one C function or one small branch exactly, keeping C-shaped names where
   it helps comparison.
3. Update `docs/porting/status.tsv` for that symbol.
4. Run:

```bash
cargo test -p zelda3
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 30000
```

5. When a frame window passes, update this file and `PROGRESS.md`.

## Active Queue

| Priority | C surface | Rust surface | Status | Notes |
|---|---|---|---|---|
| 1 | next oracle-selected gameplay frontier | `crates/zelda3/src/zelda_rtl.rs` / split modules | `not-started` | Add or run a route beyond the current saved-slot movement/button probes, then port the first exposed C branch directly |
| 2 | longer no-input oracle window | `zelda3-bin --lockstep` | `not-started` | 30,000 frames are green; run a larger window to check repeated attract cycles |
| 3 | broader recorded gameplay routes | `scripts/inputs/`, oracle runner | `partial` | input scripts support `include`; saved-slot routes now cover post-message wander, diagonal sweeps, isolated button taps, and combined button+direction probes |
| 4 | `dungeon.c` first gameplay-frame setup | `crates/zelda3/src/dungeon.rs` / `zelda_rtl.rs` | `partial` | Gameplay-entry setup now matches through first in-room message state; keep this as the target if a longer saved-slot route exposes deeper room/dungeon behavior |
| 5 | `select_file.c` file-select module | `crates/zelda3/src/select_file.rs` | `partial` | Initial setup, graphics, SRAM validation, stripe uploads, new-file naming/save init, occupied-slot drawing, and saved-slot selection into load-file are started |

## Module Map

Approximate function counts are generated by `scripts/porting_inventory.py` from
single-line C definitions. Treat the counts as navigation aids, not compiler
truth.

| C file | Lines | Approx funcs | Status | Current Rust target |
|---|---:|---:|---|---|
| `zelda_rtl.c` | 889 | 45 | `partial` | `crates/zelda3/src/zelda_rtl.rs` |
| `zelda_cpu_infra.c` | 585 | 18 | `partial` | `crates/zelda3/src/oracle.rs` |
| `types.h` / `util.c` | 279 | 10 | `seed` | `crates/zelda3/src/types.rs`, `crates/zelda3/src/util.rs` |
| `attract.c` | 1063 | 41 | `partial` | `crates/zelda3/src/attract.rs` |
| `dungeon.c` | 8796 | 395 | `partial` | `crates/zelda3/src/dungeon.rs` |
| `nmi.c` | 461 | 37 | `partial` | `crates/zelda3/src/nmi.rs` |
| `load_gfx.c` | 2182 | 147 | `partial` | `crates/zelda3/src/load_gfx.rs` |
| `messaging.c` | 2935 | 131 | `partial` | `crates/zelda3/src/zelda_rtl.rs` |
| `poly.c` | 323 | 16 | `partial` | `crates/zelda3/src/zelda_rtl.rs` |
| `hud.c` | 1554 | 64 | `not-started` | TBD |
| `overworld.c` | 4093 | 157 | `not-started` | TBD |
| `player.c` | 6664 | 203 | `partial` | `crates/zelda3/src/player.rs`, `crates/zelda3/src/zelda_rtl.rs` |
| `player_oam.c` | 1289 | 11 | `partial` | `crates/zelda3/src/zelda_rtl.rs` |
| `sprite.c` | 4328 | 230 | `not-started` | TBD |
| `sprite_main.c` | 25878 | 961 | `not-started` | TBD |
| `ancilla.c` | 7156 | 248 | `not-started` | TBD |
| `overlord.c` | 653 | 36 | `not-started` | TBD |
| `tagalong.c` | 756 | 26 | `not-started` | TBD |
| `tile_detect.c` | 527 | 14 | `partial` | `crates/zelda3/src/zelda_rtl.rs` |
| `select_file.c` | 929 | 40 | `partial` | `crates/zelda3/src/select_file.rs` |
| `ending.c` | 2658 | 84 | `not-started` | TBD |
| `audio.c` / `spc_player.c` | 1988 | 69 | `deferred` | audio milestone |
| `config.c`, `main.c`, `opengl.c`, `glsl_shader.c` | 2336 | 70 | `deferred` | frontend/platform milestone |

## Generated Inventory

Run:

```bash
python3 scripts/porting_inventory.py
python3 scripts/porting_inventory.py --list-functions
python3 scripts/porting_inventory.py --format tsv
python3 scripts/porting_inventory.py --format json
python3 scripts/porting_inventory.py --check
```

The script reads `docs/porting/status.tsv`, scans the upstream C files, and
prints the current status summary. `--check` validates the TSV against the
upstream C tree and local Rust targets so stale symbol/path rows fail fast. Add
symbol rows to the TSV as functions or tables are ported so the inventory stays
useful.

In JSON output, use `completion_summary` for the concrete completed-vs-left
function count. The older `approximate_functions_by_module_status` rollup is a
navigation aid: it assigns every function in a partially ported module to
`partial`, so it intentionally overstates actual port coverage.
