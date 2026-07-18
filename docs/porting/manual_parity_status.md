# Manual Parity Status

This is the scan-friendly index for the manual 1:1 C parity audit. The detailed
evidence remains in `docs/porting/manual_parity_audit.md`; this file is only a
human-readable status board and should be updated when a manual pass lands.

For the machine-readable per-function and line-span ledger across the
authoritative C tree, see `docs/porting/c_function_ledger.json`. Maintain it
with `python3 scripts/c_parity_ledger.py generate`, `mark`, `summary`, `list`,
`progress`, and `show`.

No progress/signature script is used for this status. A module is only marked
covered here when the ledger has direct C/Rust comparison evidence.

## How to Read This

- `covered`: source compared directly against the matching C file, with any
  found drift fixed or classified.
- `partial`: meaningful clusters were compared, but the whole file or runtime
  route is not yet fully proven.
- `open`: known non-1:1 behavior or missing external/runtime proof.
- `runtime-open`: source comparison is done for the slice, but route/oracle
  coverage still needs to exercise it.

## Covered or Largely Covered

| Area | Rust surface | Status | Notes |
|---|---|---|---|
| SNES core shell/bus | `crates/snes/src/snes.rs` | covered | Reset, bus, registers, auto joypad, saveload, timing counters covered in the ledger. |
| SNES CPU state | `crates/snes/src/cpu.rs` | covered | State layout, reset, flags, and C saveload covered. |
| SNES CPU opcode stepper | `crates/snes/src/cpu_step.rs` | source-covered/runtime-open | Addressing helpers, BRK patch/restart hooks, branches/control flow, load/store/transfer, arithmetic/compare, logical/BIT/TRB/TSB, shift/rotate/inc-dec, stack/flag/block/misc, and index inc/dec opcode families now have direct C/Rust comparison evidence. Runtime-open until broader opcode execution traces prove all families on real routes. |
| SNES DMA | `crates/snes/src/dma.rs` | covered | Register layout, general DMA, HDMA, saveload covered. |
| SNES input | `crates/snes/src/input.rs` | covered | Latch/shift and auto joypad call sites covered. |
| SNES cart/loader | `crates/snes/src/cart.rs`, `crates/snes/src/loader.rs` | covered | Header scoring, cart RAM, LoROM/HiROM behavior covered. |
| SNES tracing | `crates/snes/src/tracing.rs` | covered | CPU/SPC trace formatting and disassembly covered. |
| SNES PPU renderer | `crates/snes/src/ppu.rs` | source-covered/runtime-open | Reset/saveload/register surface, scanline shell, windows, BG pixel loops, sprites, mode 7, upsampled mode 7, final color composition, mosaic, legacy removal, C brightness formula, and focused snes9x name-entry visual parity have direct source comparison evidence. Runtime-open until broader external pixel-oracle routes cover more gameplay scenes. |
| SNES/APU/DSP/SPC | `crates/snes/src/apu.rs` | source-covered/runtime-open | APU reset/cycle/register shell, C saveload prefix, DSP register writes, DSP cycle/mix/echo/BRR/gain/noise path, DSP sample extraction, fixed C envelope arithmetic and pitch-modulation wrapping, SPC core helpers, and SPC dispatch families have direct C/Rust comparison evidence. Runtime-open because exact startup/external audio parity still needs snes9x/Mesen-style sample proof. |
| Config | `crates/zelda3/src/config.rs` | covered | Full-file pass recorded. |
| Utility/types | `crates/zelda3/src/util.rs`, `crates/zelda3/src/types.rs` | covered | Full-file/header compatibility passes recorded. |
| NMI | `crates/zelda3/src/nmi.rs` | covered | Full-file pass recorded. |
| Audio high-level runtime | `crates/zelda3/src/audio.rs` | covered/open | C high-level queue/MSU/PCM paths mostly covered; OPUZ and external startup timing remain open. |
| Overworld | `crates/zelda3/src/overworld.rs` | largely covered | Many direct passes now cover scroll, entrances, map16/map8, overlays, exits, pits/tools, mirror/spotlight, events, music, and transitions. |
| Dungeon | `crates/zelda3/src/dungeon.rs` | partial | Important main-loop gate, entry/re-entry, entrance load state, custom tile attrs, attribute table/object attrs, bunny recoil helper, liftable/pot and bomb destructibles, door animation state, quadrants, camera scroll helpers, room layout/offset helpers, transition scroll/subtile landing, push-block interactions, pushed-block motion/collision, straight/spiral staircase movement, layer effects, moving-floor/palette/color-math layer handlers, doors, torches/Ganon torch helpers, exploding-wall cleanup, transitions, room draw, brightness, swamp pool, watergate/flood-dam/water tags, activated-water attrs, chest reveal tags, moving-wall tags, boss/prize tags, room-tag tile probes, spiral-stair, straight-stair, landing-wipe, fall-recovery, warp-pad, peg-toggle, room tags/staircase detection, switch/pressure-plate tags, rescued-maiden, mirror-fade, Triforce-door, crystal cutscene setup, and falling-entrance clusters covered; full-file proof still not claimed. |
| Messaging/HUD/menu | `crates/zelda3/src/messaging.rs`, `crates/zelda3/src/hud.rs` | partial | `hud.rs` is source-covered/runtime-open: every `hud.c` function and static helper now has direct manual comparison evidence, including menu state, HUD indicators, item switching, bottle menu, draw primitives, progress/equipment/Y-button tables, refill/update helpers, and rebuild/update helpers. `messaging.rs` remains partial with world map, menu state, Y-item icon preview, recovery paths, and dispatch assertions covered. |
| Player | `crates/zelda3/src/player.rs` | partial | Movement/item/A-button/collision/assert clusters covered; full-file proof still not claimed. |
| Player OAM | `crates/zelda3/src/player_oam.rs` | source-covered/runtime-open | Link OAM control flow, tables, and helpers covered; needs broader route coverage for pose combinations. |
| Overlord | `crates/zelda3/src/overlord.rs` | source-covered/runtime-open | Full-file source pass recorded; needs route coverage for overlord states. |
| Poly | `crates/zelda3/src/poly.rs` | source-covered/runtime-open | Full-file source pass recorded; focused runtime tests absent. |
| Attract | `crates/zelda3/src/attract.rs` | source-covered/runtime-open | Full-file source pass recorded; focused runtime tests absent. |
| Tagalong | `crates/zelda3/src/tagalong.rs` | source-covered/runtime-open | Full-file source pass recorded; follower movement/drop/draw/spawn helpers and tables covered. Needs focused gameplay/oracle routes for follower states. |
| Ending intro/credits slices | `crates/zelda3/src/ending.rs` | source-covered/runtime-open | Every `ending.c` function now has direct manual comparison evidence: intro module/memory setup, Triforce intro animation, sword/flash helpers, credits scene loaders, `Module18_GanonEmerges`, `Module19_TriforceRoom`, Triforce poly helpers, Triforce/credits triangle helpers, credits dispatch, ending sprite prep, scroll/fade draw cases, credits sprite draw helpers, camera scroll helper, draw table fallbacks, late credits tail side effects, final fade/hang helpers, and credits text/attribution table indexing. Runtime-open until late-ending/credits routes exercise this against an external visual oracle. |
| Select file/name entry | `crates/zelda3/src/select_file.rs`, renderer support | source-covered/runtime-open | Every `select_file.c` function now has direct C/Rust comparison coverage: loader/SRAM validation, shared background stripe builders, saved-slot display helpers, main file-select state, copy/kill-file state machines, name-entry setup/cursor/finalize helpers, checksum, defensive index, immediate SRAM persistence, copy-player stripe upload offset, and scanline-128 BG3 split-scroll behavior. Runtime-open for broader snes9x visual route coverage. |
| Runtime host | `zelda3-bin/src/main.rs`, `crates/platform/src/lib.rs` | partial/open | Native host, lockstep/render/audio diagnostics covered; C lockstep now avoids false sample/command-port audio diffs from its silent SPC/DSP harness. Playable lockstep can now run a snes9x pixel/audio oracle in parallel, isolate snes9x SRAM, force neutral snes9x video output, auto-align video-only route phase, and write first-diff artifacts; exact startup audio parity remains open in the snes9x oracle path. |
| Oracle compatibility wrapper | `crates/zelda3/src/oracle.rs` | classified | Rust-only re-export of `zelda_cpu_infra`; not a C game-logic port surface. |

## Known Open Items

| Area | What remains |
|---|---|
| External startup parity | Focused snes9x startup/name-entry/saved-select video now matches after SRAM isolation, neutral video options, and video-only route phase alignment; snes9x/Mesen-style startup audio still has known divergence around reset/bootstrap timing and the first `$0a` SFX. |
| Raw ROM timing | Rust raw-ROM tracer does not yet advance full h/v/vblank/NMI/autojoy timing, so it cannot produce exact frame-level APUI command timing. |
| Full APU exactness | High-level SPC player matches the C model for many paths, but exact snes9x-like audio likely needs the real bootstrapped SPC program/timing path or an instrumented external oracle. |
| Lockstep audio samples | C lockstep validates game RAM/PPU/SRAM/render state, not APUI command ports or final samples. Sample-exact checks live in the snes9x/Mesen-style external oracle path. |
| Lockstep render blind spot | `--compare-lockstep-render` compares two states through the Rust renderer, so it cannot detect renderer bugs shared by both sides. Use the snes9x oracle path for true pixel parity. |
| PPU runtime proof | Renderer source is now source-covered, but final confidence requires more visual route/oracle coverage beyond the covered startup/name-entry/lockstep slices. |
| Dungeon full-file audit | Several critical clusters are covered, but remaining room/object/door/helper surfaces still need direct file-by-file audit. |
| Sprite module family | Many dispatch/draw/helper clusters are covered, but the split sprite files are not all full-file certified. |
| Ending runtime coverage | Source audit is now complete for `ending.c`, but runtime routes for ending text/credits/final fade/full credits sprite scenes remain open. |

## Practical Next Audit Queue

1. `crates/zelda3/src/dungeon.rs`: continue uncovered room/object/door helper surfaces.
2. `crates/zelda3/src/sprite*.rs`: work split sprite files one family at a time.
3. Pixel oracle routes: add/extend snes9x/Mesen visual routes for overworld, house entry/exit, pot pickup, menus, and dungeons.
4. External audio timing: replace the high-level startup shortcut with bootstrapped SPC timing or a stronger snes9x/Mesen sample oracle.

## Useful Commands

List detailed audit sections:

```sh
rg -n '^## ' docs/porting/manual_parity_audit.md
```

Show open/partial/runtime-open ledger rows:

```sh
rg -n 'open|partial|limited|runtime-open|unverified' docs/porting/manual_parity_audit.md
```

Inspect the machine-readable C function ledger without opening the full JSON:

```sh
python3 scripts/c_parity_ledger.py summary
python3 scripts/c_parity_ledger.py progress --sort open
python3 scripts/c_parity_ledger.py list --file select_file.c --status verified
python3 scripts/c_parity_ledger.py show --file select_file.c --function Intro_CheckCksum
```

List current Rust source surfaces:

```sh
rg --files crates/zelda3/src crates/snes/src zelda3-bin/src crates/platform/src | sort
```
