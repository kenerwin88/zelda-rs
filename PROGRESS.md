# Progress

Source of truth for what's done and where to pick up. If you cleared context,
read [GOALS.md](GOALS.md) first, then this.

For the systematic C-to-Rust module/function ledger, see
[docs/PORTING_MAP.md](docs/PORTING_MAP.md). The editable status source is
[docs/porting/status.tsv](docs/porting/status.tsv), and
`scripts/porting_inventory.py` regenerates and validates the current upstream
inventory. Oracle windows live in [docs/porting/oracle_windows.tsv](docs/porting/oracle_windows.tsv)
and are checked/run by `scripts/oracle_windows.py`.

## Workspace layout

```
zelda3-rs/
├── Cargo.toml                  # workspace
├── crates/
│   ├── snes/                   # SNES emulator (port of C zelda3/snes/)
│   ├── zelda3/                 # game logic (port of C zelda3/src/) — skeleton + oracle
│   ├── platform/               # Frontend trait, no impl yet
│   └── assets/                 # asset extractor port — EMPTY
└── zelda3-bin/                 # headless oracle binary
```

All five crates compile. `cargo test --workspace` is the current verification
command.

## Done

### SNES emulator (`crates/snes/`)

Ported faithfully from the C in `zelda3/snes/`. Sub-state structs are pure
data with no back-pointers; bus access is methods on the top-level `Snes`
struct. This collapses the C `Cpu*->Snes*` / `Dma*->Snes*` back-pointers.

| File | Status | Notes |
|---|---|---|
| `cart.rs` | ✓ full | LoROM + HiROM mapping; preserves the C `bank > 0xf0` vs `bank >= 0xf0` asymmetry between write/read (intentional, for oracle parity) |
| `snes.rs` | ✓ full | Bus dispatch (A-bus, B-bus, $42xx regs), mul/div regs, WRAM data port, reset, auto-joypad |
| `loader.rs` | ✓ full | Header heuristic, power-of-two padding, LoROM/HiROM detection |
| `cpu.rs` | ✓ data | `CpuState`, flag pack/unpack |
| `cpu_step.rs` | ✓ full | All 256 opcodes, all addressing modes, BRK patch table (game-specific UB workarounds — see `handle_brk_patch`) |
| `dma.rs` | ✓ full | DMA + HDMA, register R/W |
| `ppu.rs` | ✓ partial | Register R/W, VRAM/CGRAM/OAM updates, reset. **Renderer is stubbed** (`run_line` is no-op) — see "Stubbed" below |
| `apu.rs` | ◯ stub | Just `out_ports`/`in_ports` so the bus doesn't surprise the CPU. See "Stubbed" |
| `input.rs` | ✓ full | Shift-register serial protocol on $4016/$4017 |
| `consts.rs` | ✓ | `PPU_EXTRA_LEFT_RIGHT`, `PPU_X_PIXELS` (mirror of `src/types.h`) |

Tests: 31 unit tests in `crates/snes/src/*.rs` plus `crates/snes/tests/synthetic_program.rs` (build a tiny LoROM image, load it, step CPU, verify WRAM).

### Game logic skeleton (`crates/zelda3/`)

Initial lockstep-facing skeleton is in place:

| File | Status | Notes |
|---|---|---|
| `zelda_rtl.rs` + split modules | ◯ skeleton | Owns WRAM/SRAM/PPU/DMA and exposes `ZeldaState::run_frame_internal`, the Rust equivalent of `ZeldaRunFrameInternal`. Startup, intro memory/text/palette setup, item/follower graphics staging, initial title graphics load, NMI sprite/animated-tile prep, polyhedral intro rendering, intro sprite/OAM animation, intro palette fades, sword/title handoff, attract polka-dot story text/rendering, attract world-map setup/zoom, throne-room story behavior, Zelda-prison scene setup/runtime, maiden-warp Agahnim altar setup/runtime, palette flash, end-of-story reset, initial title-to-file-select setup, and the first load-file/new-game bed setup slice are ported. C-shaped groups now live in `attract.rs`, `dungeon.rs`, `hud.rs`, `load_gfx.rs`, `misc.rs`, `nmi.rs`, `overworld.rs`, `player.rs`, and `select_file.rs`. |
| `oracle.rs` | ✓ harness | Ports the `EmuRunFrameWithCompare` shape: snapshots WRAM/SRAM/VRAM, applies the C normalization rules, patches the oracle ROM like the C harness, runs the original-ROM frame entry points in `crates/snes`, then compares after the native frame. |
| `types.rs` / `util.rs` | ◯ seed | Starts the C-shaped helper surface (`types.h`, small `util.c` helpers) so later module ports have the expected vocabulary. |

### Binary (`zelda3-bin/`)

Headless oracle harness — `./target/release/zelda3 <rom> [budget]` loads the
ROM, seeds the reset vector, runs the opcode budget, prints a FNV-1a64 digest
of WRAM. Smoke-tested against real `zelda3.sfc`: ~1M opcodes execute, CPU
settles into the game's spin-wait-for-vblank loop at $8894.

There is also a guarded lockstep mode:

```bash
./target/release/zelda3 --lockstep <rom> [frames]
```

That mode uses `LockstepOracle`. Until `ZeldaState::run_frame_internal` contains
the matching game logic, it reports the first divergence. Current real-ROM
status with `/path/to/zelda3.sfc`: `--lockstep ... 30000`
matches WRAM/SRAM/VRAM when `zelda3_assets.dat` is available next to the ROM or
in the current working directory. Last verified digest:
`WRAM fnv1a64 = a5077d3d9ba1c106`.

The recorded title Start path also matches through file-select idle:

```bash
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 5000 --input-script scripts/inputs/title-start.txt
```

Last verified digest: `WRAM fnv1a64 = 51cc44fb5e0d1655`.

The recorded new-file path matches through creating a one-character save and
returning to file select:

```bash
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 5000 --input-script scripts/inputs/file-select-new-game.txt
```

Last verified digest: `WRAM fnv1a64 = 6e3be721daf0d42b`.

Route-extension scripts now reuse existing routes with `include`:

```bash
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 122000 --input-script scripts/inputs/file-select-enter-game-message-dismiss-and-wander.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 45610 --input-script scripts/inputs/opening-uncle-message-extended-move.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 116000 --input-script scripts/inputs/file-select-enter-game-diagonal-sweeps.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 112000 --input-script scripts/inputs/file-select-enter-game-button-taps.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 45610 --input-script scripts/inputs/opening-uncle-message-diagonal-sweeps.txt
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 114000 --input-script scripts/inputs/file-select-enter-game-button-probes.txt
```

Last verified digests: `WRAM fnv1a64 = ed37eeb35721bf24` and
`WRAM fnv1a64 = 4d3c07b597e279db` for the first two; the newer three are
`573328193bac854d`, `3098304aa5048c6f`, `e9bec02dd1924404`, and
`aefe8ff680a60dea`. Use
`scripts/oracle_windows.py --run` so these resume from equivalent checkpoints
instead of replaying from frame 0.

The previous `file-select-enter-game-button-probes.txt` frontier at frame
108,662 is cleared. The fix matched the C `HandlePushingBonkingSnaps_Y/X`
nudge direction, restored the `fallhole_var1` collision side effect, and
matched the indoor `TileDetection_Execute` force-move mask side effect.

The TAS-derived US full-completion SMV route now matches through 50,000 frames
with the embedded SRAM block extracted and loaded:

```bash
cargo run -p zelda3-bin -- --lockstep /path/to/zelda3.sfc 50000 --input-script scripts/inputs/tas-us-full-completion-smv.txt --load-sram scripts/inputs/tas-us-full-completion-smv.sram
```

Last verified digest: `WRAM fnv1a64 = cc117ab564b664e4`. The SMV bootstrap is
now represented faithfully, including its 0x20000-byte reset SRAM payload, but
that payload is uniform `0x60` bytes and does not contain a prepared save. In
the current oracle run this route remains in `Module04_NameFile` through frame
50,000, so keep using it only as file-select/name-file stress coverage until we
add a TAS route that demonstrably reaches gameplay.

## Stubbed (intentionally, with rationale)

| Stub | Why | When to revisit |
|---|---|---|
| `Snes::dma_start_real` cycle accounting | Implemented; full lockstep cycle timing isn't validated yet | When the full lockstep oracle is wired up |
| Raw-ROM scanline/frame scheduler | The playable path mirrors the C direct-frame game entrypoints; raw-ROM timing currently does not advance vblank/NMI/autoj relative to real scanlines | Needed only for a true raw-ROM timing oracle beyond the current bsnes frame/audio comparison tools |

## Not started

| Area | Source | Approx size |
|---|---|---|
| Game logic modules beyond the initial intro skeleton (`crates/zelda3/src/`) | C `zelda3/src/` | ~40 files, ~30K LoC |
| Asset extractor (`crates/assets/src/`) | Python `zelda3/assets/restool.py` & co | ~5K LoC of Python |
| Broader lockstep coverage | C `zelda3/src/` | Harness exists; 30,000 no-input frames are green; continue with longer windows and recorded input paths |

## How to pick up next session

In rough order of dependency:

1. **Port the next oracle-selected blocker** — run the broader passing windows or add a new route that pushes beyond the current saved-slot movement/button probes, then port the first exposed C branch directly from `../zelda3`.
2. **Add broader route coverage** — route scripts can now `include` a previous script, so new probes should branch from known-green paths without duplicating the base input.
3. **PPU renderer** (`ppu.c` ~1300 lines) — port `PpuDrawWholeLine`, `ppu_handlePixel`, `ppu_getPixel*`, `ppu_evaluateSprites`, `ppu_runLine`. Required for visible output, not for oracle correctness.
4. **Asset extractor** — separate dependency, can be parallel; port Python `restool.py` so `zelda3_assets.dat` is produced from Rust.
5. **Audio / APU** — port `snes/spc.c` + `snes/dsp.c` + `src/audio.c`.
6. **Frontend** — pick crate, wire up window/audio/input/save files.

Each module port should be: read the C file, write the Rust file with the same structure, run `cargo test`, run the oracle binary against `zelda3.sfc`, fix divergence.

## Key design decisions worth preserving

- **No back-pointers in sub-state structs.** `Cart`, `CpuState`, `DmaState`, `PpuState`, `ApuState`, `InputState` are pure data. Bus access is methods on `Snes`. (C: `Cpu* mem -> Snes*` pointer chase.)
- **`Snes::read` / `Snes::write` take a 24-bit `u32` full address** matching `snes_read` in C. CPU helpers add cycle bookkeeping in `cpu_read` / `cpu_write`.
- **WRAM and VRAM are `Vec<u8>` / `Vec<u16>`** — the oracle only compares those memory regions, not C struct layouts, so we don't need `#[repr(C)]` anywhere.
- **The BRK patch table** (`cpu_step.rs:handle_brk_patch`) preserves the game-specific UB workarounds from `cpu.c case 0x00`. These are load-bearing for the oracle — do not remove them.
- **`bank > 0xf0` vs `bank >= 0xf0`** asymmetry in `cart.rs` LoROM write/read is intentional and documented in the source. Matches C exactly. Don't "fix" it.

## Source of truth

The C codebase at `../zelda3` is authoritative.
Re-read C files for each module before porting; don't infer behavior from
this doc.
