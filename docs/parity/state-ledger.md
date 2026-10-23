# Native state regression ledger

This is a deterministic Rust-vs-Rust regression check. It replays the recorded full
route's controller inputs and initial SRAM through standalone native play, without
Snes9x, timing/RNG receipts, a window, or a GPU. It detects changes to this Rust
execution; it does **not** prove ROM parity or that the native engine reaches the same
route milestones as the recorded oracle.

## Record and compare

```sh
cargo build --profile parity -p zelda3-bin
python3 scripts/state_ledger.py record
python3 scripts/state_ledger.py compare
python3 scripts/state_ledger.py record --mode lag-free
python3 scripts/state_ledger.py compare --mode lag-free
```

Defaults: 1,581,079 frames, interval 1, the pinned `ed0121e1…` read-only cache, embedded
extracted assets (the original ROM-less smoke mode), and `target/state-ledger/baseline-default.jsonl.zst`. The wrapper clears inherited `ZELDA3_*` overrides, isolates
SRAM persistence in a temporary directory, validates launch identities on comparison,
and writes a `.meta.json` sidecar with input/SRAM/ROM/binary hashes, git revision, dirty
flag and timing. It only reads controller inputs and initial SRAM from the oracle cache.
Use `--frames`, `--interval`, `--cache`, `--rom`, or `--binary` to override these. Add
`--mode lag-free` for the LAGLESS engine. Default and lag-free are **separate
baselines** (`baseline-default.jsonl.zst` and `baseline-lag-free.jsonl.zst`); mode is
validated in the metadata and recorded as `lag_free` in every record. The smoke calls
`ZeldaState::set_lag_free(true)` immediately after loading a fresh engine, before SRAM
application and the first frame, exactly like `logical_replay.rs`. Add `--rgb` to both actions to hash CPU diagnostic
RGB as well (slower).

```sh
python3 scripts/state_ledger.py record target/state-ledger/candidate.jsonl.zst
python3 scripts/state_ledger.py compare target/state-ledger/baseline-default.jsonl.zst
```

Raw binary flags (following `--standalone-smoke <frames>`): `--state-ledger
<new.jsonl[.zst]>`, `--compare-state-ledger <baseline.jsonl[.zst]>`, `--ledger-interval
N`, `--ledger-rgb`, `--lag-free` (also usable with smoke alone). Recording and
comparison may be combined. Existing output files are refused. Set `ZELDA3_SMOKE_INPUT_LEDGER`, `ZELDA3_SMOKE_SRAM`, `ZELDA3_SMOKE_ROM`, and
an isolated `ZELDA3_SAVE_DIR` for a direct launch. Ledger input loading rejects missing,
malformed or noncontiguous frames.

## Record contract and limits

Each record-schema-3 JSON line describes the **post-frame** state, after game update, 735
stereo PCM sample frames, unused-audio discard and headless display publication. `frame`
is the zero-based input index (frame 0 is after the first update). Records contain
numeric `input`, boolean `lag_free`, hexadecimal FNV-1a-64 `state`, `display`, `sram`
and `audio`, optional `rgb`, plus `main`, `sub` and little-endian dungeon `room` (`$a0/$a1`). Audio hashes signed 16-bit
samples in little-endian interleaved order at the smoke's fixed sample cadence, not the
oracle's variable audio cadence.

`ZeldaState::canonical_state_hash()` uses **state domain v2**. Record schema 3
and metadata schema 3 (`state_domain: "zelda3-state-v2"`) reject v1 baselines
with an explicit re-record message. No engine/checkpoint layout changes are involved.

The exact canonical state wire order is:

1. Literal domain bytes `zelda3-state-v2`, all `wram_image()` bytes, all `sram` bytes.
2. A bincode-1 tuple, in order: `initialized`,
   `apply_links_movement_to_camera_called`, `wanted_zelda_features`,
   `previous_host_controller_input`, `dialogue_blk_index as u64`,
   `dialogue_font_blk_index as u64`, `dialogue_flags`,
   `dialogue_scroll_continuation`, `dialogue_fast_forward_hold_pending`,
   `dialogue_fast_forward_hold_active`, `lag_free`, the retired thread-credit slot
   (`0u8`, retained from v1), `gloves_color`.
3. `game_state.hash_non_wram_fields`, recursively in the explicit child order
   declared by each state's `non_wram_hash!` invocation. **Currently emits no bytes:**
   the audited game fields all have cartridge WRAM homes. This is not a blanket
   default: each invocation exhaustively classifies fields, so a new field fails
   compilation until explicitly classified. Equipment's classification expands
   from its existing field macro.

The WRAM audit follows `load_from_ram` and all publication paths, including
write-through setters and partial/mode-gated codecs; omission from the bulk
`write_to_ram` alone does not make a field native-only. Coverage by sub-state:

| Group | WRAM representation checked |
|---|---|
| frame, system signals, enhanced features | module/counter/flags, sound controls, both MSU resume records, feature word |
| scratch counter, minigame, intro sword, archery, sprite battle | scratch bytes/words, timers, counters, positions and RGB bytes |
| memorized tiles, dungeon secret, save transfer, dungeon map | count-sized address/value tables, secret fields, transfer offset, map state and marker words |
| dungeon | header/scratch/entrance/torch, savegame, attributes, stair tables/counters, doors, room buffers/parser/effects/items/tracking, movable blocks |
| sprites | slot banks, workspace, cached banks (including aliased borrowed slots), trails/history, ether, maze/prize/damage/hitbox tables, presence/loaded flags, garnish/follower runtime |
| player | special exit, follower-link input/presentation/movement/actions, swim acceleration, pushed blocks, subpixel accumulator and tile detection |
| inventory | equipment/bottles, keys, resources including all magic fields, mirror warp, progress/rooms/deaths/checksum/HUD |
| world | location/scroll/camera/palette/region/transient, overworld tables/UI/weather/travel/map16/entrance/exit/transition, room bounds |
| poly | runtime, projected vertices, face coordinates and raster edges |
| display | register copies, source pointers, palette/HUD/HDMA/scroll buffers and filter state |
| effects | angle scratch, quake/bombos/tower, pond/debris, split sprite histories, entrance effects and digging prize |
| ending, messaging, OAM | scenes/actors/credits, file menus/dialogue/VWF buffers, shadow/extended tables, allocator regions and priority words |

Included non-WRAM `GameState` fields: **none**. The sole metadata exclusion is
`display.palette_provenance`: its type explicitly defines it as derived metadata,
skips it in snapshots and ignores it for coherence. It holds palette mirror
values/source tags/Unknown markers, not additional cartridge execution state;
including it would fingerprint annotation/cache history. Immutable defaults,
vector capacities and unused gaps in physical-address work banks likewise do
not create native execution fields. Mode-reused bytes remain represented once
by the exported cartridge image regardless of the number of stale native views.

Bincode 1 uses fixed-width little-endian encoding; host-sized dialogue indices
are cast to u64. No raw struct bytes, pointers or unordered debug dumps are hashed.
Refreshing any WRAM-backed mirror, including deferred resource adoption after a
save clear, must leave v2 unchanged when the image and other inputs are unchanged.

The owned-WRAM registry remains empty in this step. `wram_image` borrows RAM in
that case; a nonempty registry makes one backing copy and calls each narrow
`export_owned` callback directly on it. `import_owned` reads only that owner's
registered bytes; unowned bytes copy to RAM. Codecs may not invoke broad legacy
projection. Unit coverage tests export-after-import identity and checks all
foreign bytes against multiple sentinels for every production range, plus a
nonempty test owner before production owners are enabled.

**All other `ZeldaState` fields are outside `state`.** In particular, this excludes
`ppu` (including VRAM/CGRAM/OAM/registers/latches), display publication/history queues,
CPU registers and CPU/timing/scheduler continuations not present in `game_state`,
all audio/APU/SPC/DSP internals, immutable assets/ROM/source art, diagnostic counters,
and lag-free timing/poly-thread continuations other than `lag_free`
(the retired credit wire slot above is a constant). The separate `sram` hash duplicates SRAM coverage for
localization; `audio` checks emitted PCM, not hidden audio state.

`display` always runs without a renderer. It hashes the **already captured presented
PPU generation**, not the mutable live PPU after display retirement, using domain
`zelda3-display-v1` and explicit fixed-width little-endian bincode fields, in the
following order (vector/slice lengths and option tags are included):

- `vram`, `cgram`, capture `cgram`, all `oam` words (including high table words
  0x100–0x10f), `bg_vram_latch`, `obj_vram_latch`, `obj_previous_frame_vram`.
- `screen_enabled`, `screen_windowed`, `mosaic_enabled`, `mosaic_size`,
  `obj_tile_adr1`, `obj_tile_adr2`, `obj_size`, `window1_left`, `window1_right`,
  `window2_left`, `window2_right`, `windowsel`.
- `clip_mode`, `prevent_math_mode`, `add_subscreen`, `subtract_color`, `half_color`,
  `math_enabled`, `fixed_color_r`, `fixed_color_g`, `fixed_color_b`.
- `forced_blank`, `forced_blank_scanlines`, `forced_blank_from_scanline`,
  `retain_active_display_history`, `brightness`, `scanout_brightness_override`,
  `scanout_top_crop`, `mode`, all four `bg_layer` structures (H/V scroll,
  tilemap dimensions/address, tile address), `m7_matrix`, `m7_large_field`,
  `m7_char_fill`, `m7_x_flip`, `m7_y_flip`, `m7_ext_bg_always_zero`,
  `m7_start_x`, `m7_start_y`.
- All 224 captured scanline register tuples: window edges, main-screen enable,
  BG H/V scroll, Mode 7 matrix, forced blank. This includes HDMA scanline changes.

Display excludes renderer buffers/caches/flags, PPU write-port pointers and partial
write latches, beam read latches, frontend asset/source metadata and historical
surfaces outside the captured PPU. These can escape detection until they affect the
covered display/state/PCM or optional pixels. It is a cheap memory/register checksum,
not a pixel-equivalence claim.

Optional RGB uses the modern CPU variant-atlas renderer and source atlas, hashing
256×224 RGB bytes (alpha omitted). It creates no GPU device/window and does not claim
equivalence to the production GPU frontend. Source art must be unchanged between
comparisons; a change to art or software rendering is intentionally observable. RGB is
off by default to keep this state regression check fast.

Optional `--rom saves/zelda3.sfc` selects the existing ROM-backed smoke mode and also
requires an extracted `zelda3_assets.dat` beside the ROM or in the working directory. Do
not compare across launch modes. FNV-1a is fast and noncryptographic, with a finite
collision risk. This ledger does not replace the Snes9x A/V gates.

Comparison streams the baseline, stops with exit 1 at the first differing record, and
prints its frame, differing fields, actual main/sub/room and both records. Missing and
extra records are errors, including truncated compressed streams. Interval N samples
frames 0,N,2N,… and the final frame; only interval 1 can name the first divergent frame.
EOF must match the requested replay length.

## Re-bless and preserve the baseline

Before a refactor, record from the accepted source revision. Keep the baseline and its
metadata outside git under `target/state-ledger/` (full per-frame ledgers are expected
to exceed 20 MB), and retain/copy them with the source revision when moving machines.
After an intended behavior or hash-definition change, review the divergence and record
to a **new filename**. Compare a second cold run, then deliberately adopt that file as
the new baseline. Never overwrite the accepted baseline in place. Changes to the state or display hash
definition require updating its domain version and the record schema, then re-blessing; serialization
layout changes are not behavior-neutral to the hash. No checkpoint layout or engine-
state fields are changed by this tooling.

## Validation and timing

### Reviewer revision (record schema 2, historical state v1)

2026-10-22, parity profile, interval 1, ROM-less launch, CPU-only. The longest
completed prefix is **200,000 frames per run**, twice in default mode and twice
in lag-free mode. Each pair is byte-identical as compressed files and as decompressed
JSONL. Each file was decompressed successfully and checked for exactly 200,000
contiguous records (frames 0–199999), schema 2, correct mode, and a display hash on
every record. The two modes have distinct ledger contents.

Full 1,581,079-frame repetitions were bounded to this prefix: measured throughput
was 758–852 frames/s, estimating **31–35 minutes per full run** (over two hours for
four). No revised full-route determinism result is claimed.

Child-process wall times include input parsing/startup and exclude the wrapper's
provenance hashing; the first default run also overlapped the short integration checks.

| Mode | Run | Frames | Seconds | Frames/s |
|---|---|---:|---:|---:|
| default | a | 200,000 | 263.834 | 758.1 |
| default | b | 200,000 | 237.852 | 840.9 |
| lag-free | a | 200,000 | 235.301 | 850.0 |
| lag-free | b | 200,000 | 234.827 | 851.7 |

Commands used for the recordings (existing files are refused; choose fresh names
when repeating):

```sh
cargo test --profile parity -p zelda3-bin state_ledger::tests
cargo build --profile parity -p zelda3-bin
for mode in default lag-free; do
  for repeat in a b; do
    python3 scripts/state_ledger.py record \
      "target/state-ledger/revision-${mode}-${repeat}-200000.jsonl.zst" \
      --frames 200000 --mode "$mode"
  done
done
python3 scripts/state_ledger.py compare \
  target/state-ledger/revision-probe-default-10000.jsonl.zst --frames 10000
python3 scripts/state_ledger.py record target/state-ledger/revision-rgb-10.jsonl.zst \
  --frames 10 --mode lag-free --rgb
python3 scripts/state_ledger.py compare target/state-ledger/revision-rgb-10.jsonl.zst \
  --frames 10 --mode lag-free --rgb
```

The default 200,000-frame recording also matches every previously covered field
(`state`, `sram`, `audio`, `frame`, `input`, `main`, `sub`, `room`) in the retained
schema-1 `proof-a-200000.jsonl.zst`. This checks canonical-state stability and default
behavior neutrality over that prefix, independently of the new display field.

All four recordings used executable `target/parity/zelda3`, SHA-256
`a93309d482868507cdfde2a4135334793d531197e808c8283ef1ed9faf46cf7a`.
Proof details are retained in `target/state-ledger/revision-determinism-proof.json`,
with individual ledgers, metadata and logs named `revision-<mode>-<a|b>-200000.*`.

default pair: 6,633,471 compressed bytes per file.

Compressed SHA-256: `777f4edd9551bc8d240ff732261363f4a0ae1bcc8515b4e0e2c4d33895d7123c`.

JSONL SHA-256: `81837c823b3e6b72230ca32aa298ba0b1586616b55ea54d419745175a5813ea4`.

lag-free pair: 6,657,852 compressed bytes per file.

Compressed SHA-256: `3a1ca3ad8cade3483396e7760e4a4917429882b2f4669480b2b30bb12b9a44d1`.

JSONL SHA-256: `e0b9a06c9b2be10831c0d6830f970308b7210d3f16a366b781440218ae89e63d`.

Six ledger unit tests passed, including memory/register/latch/scanline display
mutations, mode and display difference reporting, rejection of legacy records without
`display`, compressed roundtrip and missing/trailing records. A deliberately altered
`display` failed at frame 17 with `fields=display`. Cross-mode comparison failed before
launch via metadata, and at frame 0 via `lag_free,state` when metadata was absent.
A separate 10,000-frame default compare matched through EOF (20.794 seconds, while a
long recording was also running). Lag-free optional RGB record/compare matched ten
frames (0.849/0.856 seconds), and lag-free smoke without a ledger passed.

Python compilation, `git diff --check` and the compact atlas guard passed. The
RAM-readability guard still fails on the same six pre-existing direct-hex accesses,
confirmed in HEAD (see historical note below). No GPU execution, oracle-cache writes, commits or pushes were performed. All source changes remain uncommitted for
the reviewer. The full route, ROM parity, unseen future-state continuations, and
production pixel equivalence remain outside this completed prefix proof.

### Previous schema-1 measurements (historical, no display coverage)

2026-10-22, parity profile, interval 1, default ROM-less launch. The practical
proof covers **200,000 frames twice**, not the full 1,581,079 frames. A full
attempt was interrupted to bound the session; its incomplete output is named
`target/state-ledger/aborted-full.jsonl.zst` and is not a baseline. Full-route
recording remains the default command above; no full-route baseline is blessed
by this change.

The two long recordings use the same retained executable
`target/state-ledger/proof-zelda3` (SHA-256
`a623ba9f61a850fbd5abf52fc6fac472dc329c0b6cdc637665f2954ee188b8da`).
The final tooling additionally provides optional CPU RGB and stricter CLI/EOF
validation; its default mode matched the earlier 10,000-frame ledger through
EOF (12.696 seconds, 787.7 frames/s). The canonical state definition did not
change between these builds.

Measured child-process wall time includes input parsing and startup, excluding
wrapper provenance hashing. First 200k recording: 206.657 seconds, 967.8 frames/s.
Second 200k recording: 214.082 seconds, 934.2 frames/s.
Both contain exactly 200,000 contiguous records (0–199999) and are byte-identical
as compressed files and as decompressed JSONL. Compressed size: 5,321,745 bytes.

Compressed SHA-256: `4d5e55a4ed7e10ceaa6b5b9d1e5923038a91bcc690f46640469a1c317fa4fee0`.

JSONL SHA-256: `03af8fc9a09bd0c5f5bfb5c02563ec5380318ccbded7a4e0d4c8af52eb501c33`.

Proof artifacts and metadata remain under `target/state-ledger/`; the hashes and
regeneration instructions are documented here. At the measured 934–968 frames/s,
a full-route run is estimated at 27–28 minutes (not measured).

Other checks: four ledger unit tests passed using
`cargo test --profile parity -p zelda3-bin state_ledger::tests` (the unrestricted
binary suite contains GPU tests and was not run). An altered audio hash failed
at frame 17 with `fields=audio`; a truncated ledger failed at frame 10 with
`fields=missing_record`; an extra baseline record failed at frame 9 with
`fields=extra_record`. Interval 4 over ten frames recorded 0,4,8,9 and compared
successfully. Optional RGB recorded 1,000 frames in 65.623 seconds (15.2 frames/s);
a separate ten-frame RGB record/compare matched. No GPU comparisons were run.

The compact atlas guard passed. The RAM-readability pre-commit guard reported
six pre-existing direct-hex RAM accesses (confirmed in HEAD) in
`zelda_rtl/rtl_original_timing.rs`, `zelda_rtl/rtl_poly.rs`, and the startup debug
print in `zelda_rtl.rs`; this tooling change adds none. These remain unresolved.
