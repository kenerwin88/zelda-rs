# Lag-free logical parity

The engine's lag-free mode (`ZeldaState::set_lag_free(true)`) runs every frame
as one complete main-loop iteration: work the cartridge spreads over lag frames
finishes inside the iteration. Host-frame parity against Snes9x cannot judge
such an engine, because a lag frame on the cartridge is simply a slow
iteration. Lag-free parity compares **per main-loop iteration** instead.

## The two tools

The corrected full-route ledger is
`/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2`.
`full` remains the immutable backup. The recorder fix, exact ledger comparison,
continuous pass and both seeded census passes are documented in
[Continuous lagless 2](cost-pacing.md#continuous-lagless-2). Use a fresh output
directory when recording another ledger.

```sh
# 1. ROM ledger: the cartridge on the in-repo source CPU, cut into iterations.
cargo build --profile parity -p snes --example logical_frame_ledger
C=.git/parity-oracle-cache/<key>
ZELDA3_LEDGER_SEED_EVERY=2000 ZELDA3_SOURCE_PROGRESS_EVERY=100000 \
  target/parity/examples/logical_frame_ledger saves/zelda3.sfc \
  $C/initial.srm $C/input.txt 1581079 <ledger-dir>

# 2. Lag-free replay of the engine against it.
target/parity/zelda3 --logical-replay <ledger-dir> $C/initial.srm \
  --seed-every 2000 --video $C --report 0 > census.txt

# 3. Worklist: totals and first divergences grouped by module and variable.
scripts/logical_census_summary.py census.txt
```

`--start K` seeds at iteration K and skips everything before it (a segment
repro in seconds); `--dump-wram K:DIR` writes both sides' WRAM after
iteration K; `--apu-trace lo-hi` prints both sides' APU port activity.

The ledger (`crates/snes/examples/logical_frame_ledger.rs`) runs the cartridge
from reset on `Snes9xColdCpuExecutor`. An iteration ends at an NMI accepted
while `$12` is clear (Main finished; the handler then performs the frame's
updates); an NMI with `$12` set is a lag NMI inside the iteration. Testing the
interrupted PC against the `$00:8034` wait loop instead merges iterations: Main
can finish and be interrupted before it reaches the loop. The handler ends at
its `RTI` (IRQs can nest). A source receipt can also accept a new interrupt
after that RTI; the recorder retires the return before accounting for the new
interrupt, so the post image cannot slip to the next NMI. Per iteration it
records, in `iterations.jsonl`:

- the joypad word the opening NMI latched (`$4218`, the bit-reversed libretro
  word);
- the cartridge RNG outputs (the `STA $0FA1` at `$0D:BA7C`, with carry);
- APU port writes and reads, lag NMI and IRQ counts;
- `picture_host`: the host whose scanned-out frame first shows the
  iteration's work (cache frame N is `retro_run` N).

It also writes XOR-delta WRAM streams (images at each closing NMI's acceptance
and return) and, with `ZELDA3_LEDGER_SEED_EVERY`, full machine seeds
(WRAM, VRAM, CGRAM, OAM, SRAM, `PpuState`) to restart the engine from.

The replay (`zelda3-bin/src/logical_replay.rs`) runs a lag-free romless
engine and checks after every iteration as below. At each seed boundary an
engine that drifted (any WRAM, RNG, APU command or video content miss in the
last 200 iterations, or a panic) is replaced by a fresh one seeded from the
ledger; an engine still in step continues, because a seed into the middle of
some sequences cannot rebuild every runtime owner (`--eager-seed` reseeds at
every boundary). Seeding moves the cartridge's live HDMA table (`$1B00`) into
the port's dynamic table.

| Check | Against | Notes |
|---|---|---|
| WRAM | post-NMI ROM image | `PORT_REPRESENTATION` lists bytes the port legitimately stores differently (scratch, stack, own flags, ROM pointers, relocated HDMA table) |
| Video | Snes9x cache frame `picture_host` | misses classed `early`/`late` (one-iteration shift), `content_lag` (the ROM lagged: HDMA tearing a lag-free frame never shows), `content_tear` (the closing NMI handler returned inside active display, recorded as `nmi_return_line`, so its scroll and register writes apply partway down that frame), `content` |
| APU commands | ledger port writes | port transitions to nonzero values, matched within one iteration; commands a lag NMI sent and Main then replaced are `unmatched_lag` |
| RNG | ledger outputs | draws past an iteration's outputs (`extra`) or outputs left unused (`unconsumed`) |
| APU reads | ledger `$2140` reads | replayed into the song-end polls; values never reached (`unconsumed`) or polls with none installed (`extra`) |

With `--video`, the replay first verifies the ledger's RNG sequence against the
cache's `rom-random.txt` and refuses a ledger that disagrees.

## Conventions that matter

- The engine runs Main and then the NMI that reads the pad, so engine frame k
  is handed the joypad the cartridge latched for iteration k + 1.
- A lag-free frame presents the live PPU state its own NMI wrote; the display
  snapshot's generation composition exists for lagged host frames and is
  bypassed (`LiveGpuFrameCapture::from_game_at_comparison_frame`).
- Timing-derived inputs are replayed per iteration, like the joypad: the RNG
  outputs (beam-counter seeded) and the `$2140` values the song-end polls read
  (elapsed SPC time, which lag changes). A busy-wait spins on one value for as
  long as the SPC takes, so only the sequence of distinct values is replayed
  and the last one holds; song-bank upload handshakes are not polls.
  `--live-apu-reads` lets the engine's own SPC answer instead.
- The replay renders one audio frame after each engine frame, as a frontend
  does: the audio side takes the SPC's acknowledgements, and without them the
  NMI never clears a port, so a repeated command is never sent.
- Receipt-era host-timing rules must be off under `lag_free`. Gated so far:
  the dialogue renderer's per-frame CPU budget (the cartridge finishes an
  overrunning render in the same, lagging, iteration), the interface-exit BG
  stripe deferral (the next NMI consumes it), and the Module09 sprite reload's
  deferred slot publication (an untimed reload runs in place; the untimed
  whirlpool case runs `Overworld_LoadOverlays2` to completion).

## Timing-defined sequences

Some cartridge behavior advances per NMI rather than per iteration: the
polygon thread (the intro triforce, the crystals, the ending) runs on the NMI
clock while `NMI_THREAD_ACTIVE` (`$012A`) is set, so a lagging iteration
advances it twice. The lag-free engine matches such sequences in frames, not
iterations, and its iteration count drifts there (file select appears at
iteration 807 instead of 659). The replay does not compare the thread's WRAM
while it is active and classes video misses then as `content_poly`. The
ledger also seeds the first iteration after each timing-defined stretch (the
thread running, or the intro or attract module) and lists it in
`seeds/resync.txt`; the replay reseeds there when the engine drifted or is
still inside its own stretch, so the drift ends with the sequence instead of
shifting the rest of the segment. The summary reports intro/attract segments
(modules 00, 14) apart.

## Picture oracle

`ZELDA3_LEDGER_PICTURES=1` draws every frame with the port's line renderer from
the ROM's PPU state, line by line, and writes `host-pictures.txt`; its RGB
hashes equal the Snes9x cache on most frames. `ZELDA3_LEDGER_FRAMES=lo-hi`
also writes `frames/host-<n>.ppm` and the fixed color and CGRAM, so a video
miss can be inspected as an image diff against the engine's capture
(`ZELDA3_CACHED_AV_CAPTURE_DIR` / `_FRAMES` with the replay). It is a
diagnostic: the Snes9x cache remains the authority.

## Lagless ending polygon picture

Investigation on `codex/poly1`, base `7db31d92`; CPU evidence is in
`target/poly1/`. **There are two independent effects, and the picture mismatch
is not fully resolved.**

The proven register bug is `EndSequence_32`: `$0E:BD21` loads `$12` and
`$0E:BD23` stores it to BG2SC (`$2108`). The translated initializer stored
`$13`, selecting a 64x64 tilemap instead of the cartridge's 32x64 tilemap.
This is corrected at the store. The later `$0E:BDAB..BDB0` word load/store
(`LDY #$1300; STY $2107`) really does change BG2 to 64x64; that transition
remains intact. A unit test checks both transitions against the ROM bytes.

At iteration 1,470,000, the corrected native PPU and the ledger seed agree
in all VRAM, CGRAM, OAM, BG descriptors, mode, brightness, screen enables,
color-math enable and fixed colors. Before the fix, BG2's width was their
only differing BG descriptor. Static line-renderer snapshots become identical
when that descriptor is corrected. Static snapshots omit HDMA and are **not**
cache-validated scanouts. The native software compositor, which includes the
scanline register capture, produces RGB SHA-256
`2aec28b10801343d2e7ba6a0950d478eae51b9febc4cbba126eacfceedf06dfe`, equal to
authoritative cache host 1,577,955 (one before the iteration's picture host).

The baseline worker/upload investigation also finds real phase drift. From a
single seed at iteration 1,466,000, all **1,915** distinct consecutive native VRAM
uploads equal the first 1,915 of **1,918** completed cartridge uploads, in
order, including their `$1F04/$1F05` angles. The 256 distinct bitmaps repeat
through that sequence. At every one of those 1,915 corresponding uploads,
the complete `$1F00..$1F09` inputs and `$1F3F..$1FFF` worker workspace also
agree byte for byte, including rotation products, projected vertices and
raster edges (`complete-worker-equality.json`). Comparing the logged NMI host numbers for corresponding
uploads gives offsets -1/0/+1/+2/+3 on 406/517/547/404/41 uploads. No angle or
pixel correction was applied. The native sequence ends three uploads before
the ROM sequence; equal ordered bitmaps are **not per-NMI state equality**.
The 42 initial native NMI records precede the first new bitmap and retain the
previous VRAM image. See `upload-phase.json` and `nmi-bitmap-analysis.json`.

After the width fix, the CPU compositor over iterations 1,466,000..1,473,122
finds **3,269 exact**, **1,486 within three cache hosts**, and **2,368 absent**
from cache hosts after 1,560,000. Thus a correct bitmap sequence alone does not
establish correct composite pictures: the scrolling background/fade and the
worker can occupy different phases. Full ending picture acceptance remains
open. The intro does not execute the faulty ending store; its remaining
picture misses do not share this BG2SC cause.

The intro ROM-side slice (`intro-source/`) renders 29 hosts around 172/180
and 966; **all 29 RGB hashes equal the cache at the same host**. At native
iteration 16 / oracle host 172, `$012A` is zero, BG descriptors/CGRAM/OAM
agree, and 218 VRAM bytes differ. Its failure is therefore not an active
polygon rasterizer or the ending-only BG2SC store. That separate VRAM
generation discrepancy is not repaired here.

Diagnostic additions (all CPU-only): `ZELDA3_DEBUG_POLY_NMI_STATE=<jsonl>`
records worker storage, bitmap, uploaded VRAM, palette, OAM, mode and brightness
at polygon NMI callbacks; `ZELDA3_LOGICAL_CPU_PICTURES=<path>` records RGB hashes
from the software compositor. These files append, so use fresh paths.
`--dump-wram` also saves the complete PPU JSON. The ledger example's
`--ppu-snapshot <json> <ppm>` draws a static snapshot, explicitly without CPU
or HDMA execution. None of these diagnostics changes census classifications
or the picture matcher.

Verification: the targeted ROM-witness regression test passes; default and
`--no-default-features` parity builds are warning-free. The library-test build
reports existing unrelated dead-code warnings. Both the seeded ROM census
(`--seed-every 2000`) and continuous recorded replay complete **1,473,123**
iterations and pass the unchanged gate with **unexplained 0**. ROM-lane missing
holds / excess / FP remain **2,398 / 0 / 0** (2,365 iterations have missing
holds). `git diff --check` passes. Evidence occupies approximately 255 MB.
No GPU, live replay, full library suite, protected-cache/shared-ledger write,
formatting, commit or push was performed. This is a verified partial repair,
not full ending picture acceptance.

GPU confirmation (run these **serially**; not run during this investigation):

```sh
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
L=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
mkdir -p target/poly1/gpu-saves
ZELDA3_SAVE_DIR="$PWD/target/poly1/gpu-saves" target/poly1/zelda3-default \
  --logical-replay "$L" "$C/initial.srm" --seed-every 2000 --single-seed \
  --start 1466000 --video "$C" --video-misses --report 0 \
  > target/poly1/ending-confirm-gpu.txt 2>&1
ZELDA3_SAVE_DIR="$PWD/target/poly1/gpu-saves" target/poly1/zelda3-default \
  --logical-replay "$L" "$C/initial.srm" --frames 2000 \
  --video "$C" --video-misses --report 0 \
  > target/poly1/intro-confirm-gpu.txt 2>&1
```

## Ending polygon phase

Follow-up on `codex/poly2`, base `1e7e6a81`. Evidence is under
`target/poly2/`; the cache, `full-v2`, and `target/poly1/` were read-only.
**This is a partial repair. Per-NMI upload equality and complete ending
picture acceptance remain open.** The supplied GPU confirmation is retained
at `target/ending-confirm-gpu-1e7e6a81.txt`; no GPU confirmation was run here.

Two missing main-work prices are now generated ROM block markers:

* `SaveGameFile`, `$00:894A..89C1`: **190,242 instruction clocks**, including
  128 paired SRAM-copy passes and 639 checksum words. Its continuation helpers
  partition those same blocks without replaying the prefix.
* `TransferFontToVRAM`, `$00:E556..E582`: **483,682 instruction clocks** for
  setup, 2,048 CPU word stores, and return. This is not DMA. File select's
  separately priced inline `$00:E52B..E535` copy mutes the shared helper so
  that it retains its own source price.

The worker scheduler also omitted main's recorded `$420B` DMA work from its
activation/GO positions and total work. It now includes that bus work. Before
activation, only the opening NMI uses the ordinary handler; later unfinished-main
NMIs use the retained lag handler (`$00:813A` skips updates and joypads).
Previously the worker scheduler deducted the ordinary handler on every such
frame. No polygon instruction price, `POLY_THREAD_*` constant, classifier,
scene table, or fitted threshold was changed.

These changes restore the initializer's **41** main holds (previously 39) and
its missing initial worker slice. The fresh cold CPU ROM trace shows NMI host
**1,577,170**, interrupting `$00:899D` in the checksum, returning into the worker,
then IRQ interrupting it at `$09:FB47`; main finishes on line 208. The original
native worker phase plan contained zero lag slices for that initializer.
`source/iterations.jsonl` and `source/wram-pre-nmi.xor.zst` are byte-identical
to their authoritative `full-v2` counterparts, SHA-256 respectively:

```text
9227f21db6148389146db7871447af6ab09f5a1771b15b87810b60073574a47d
b69d8c8851a1320dbf5e507045132c9054bd4db4237258c2c2a0157a084a365e
```

The residual owner is still the worker's NMI-to-IRQ scheduling and publication
phase, not its ordered raster output. The whole-worker ROM witness checks all
256 angle pairs instruction by instruction, including bus costs, divider,
workspace, bitmap, and completion flag. All **1,917** resulting native uploads
equal the first 1,917 of **1,918** ROM uploads, including `$1F00..$1F09` and
`$1F3F..$1FFF` (`dma-completed-worker-equality.json`). However, matching each
worker phase to its ROM phase gives only **2,861 / 3,908** equal upload
decisions, **828 / 3,908** equal workspaces, and **582 / 3,908** equal complete
worker cursors. Stack-frame storage is reported separately from native retained
registers; it is not silently discarded from the byte-difference receipts.

`source.log`, `source-phases.json`, `dma-phase-comparison.json`, and
`dma-workspace-comparison.json` retain the per-NMI comparisons. For example,
the first threaded lag handler takes **3,238** source clocks versus the native
constant **3,284**. An early upload handler takes **51,514** versus **51,534**.
Source NMI acceptance and instruction-boundary IRQ retirement also vary;
the native phase window still uses raster cycle 86. These observations do not
justify replacing a constant with an averaged or fitted value. At the first
remaining picture miss, source and native already disagree about whether the
previous worker slice published a bitmap.

For iterations **1,466,000..1,473,122**, `ZELDA3_LOGICAL_CPU_PICTURES` produces
**5,401 exact cache hashes**, **1,133 within three cache hosts**, and **589
absent** from cache hosts after 1,560,000. The first non-exact picture is
iteration **1,470,865**, cache host **1,578,821**. This improves the baseline
3,269 / 1,486 / 2,368 but does not meet full picture acceptance. Callback-host
offsets for corresponding uploads are -1/0/+1/+2 on 406/641/602/268 uploads;
callbacks can contain zero or two accepted NMIs, so this is not a physical
per-NMI equality proof. Pre-NMI stream image `j` closes iteration `j-1`.

A broader shared-wall-clock trial was rejected and removed: despite the same
picture improvement, its full census regressed to **2,859 missing / 437 excess /
75 false-positive iterations** by shifting crystal-cutscene work. Its patch
and logs remain labeled `shared-clock-trial.patch` / `clock-*`; its continuous
diagnostic was interrupted. They are not evidence for the retained candidate.

Verification: the retained candidate's full seeded ROM census
(`--seed-every 2000`) and continuous recorded replay each complete
**1,473,123 iterations**, **PASS**, **unexplained 0**. Final ROM missing / excess /
FP is **2,393 / 0 / 0**, with missing holds in 2,363 iterations; every runtime
hold equals P2. Predictions are identical to the pricing-only intermediate,
so the narrow worker correction adds no census regression. See `dma-counts.json`,
`dma-census-gate.log`, and `dma-continuous-gate.log`.

All **14 targeted Rust tests** and **7 block-generator Python tests** pass;
default and `--no-default-features` parity builds are warning-free. Existing
library-test dead-code warnings remain separate. The final rebuilt default
binary reproduces all 7,123 candidate CPU picture hashes byte-for-byte
(`verified-pictures.txt` versus `dma-pictures.txt`). Full gates used `dma-trial`;
the final rebuild adds only the DMA regression test and a corrected source-address
comment. `git diff --check` passes. Evidence occupies approximately **1.8 GB**.
No GPU, live replay, full library suite, protected-directory write, formatting,
commit, or push was performed.

GPU confirmation command (not executed):

```sh
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
L=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
mkdir -p target/poly2/gpu-saves
ZELDA3_SAVE_DIR="$PWD/target/poly2/gpu-saves" target/poly2/zelda3-default \
  --logical-replay "$L" "$C/initial.srm" --seed-every 2000 --single-seed \
  --start 1466000 --video "$C" --video-misses --report 0 \
  > target/poly2/ending-confirm-gpu.txt 2>&1
```

## Lagless intro picture

Investigation on `codex/intro1`, base `e38a8be3`; CPU receipts are in
`target/intro1/`. **Partial repair: the opening VRAM generation and one
visible picture are repaired. The late pictures remain unresolved.** The
cache, `full-v2`, and `target/poly1/` were read-only.

The 218-byte difference at iteration 16 / cache host 172 belongs to the
**opening NMI at host 81**, before the first main iteration. It is not
decompression or polygon content. Native logical bootstrap initialized WRAM
but omitted this handler, then main disabled core updates, preserving the
missing uploads throughout the boot card. The source's DMA register trace
(`source.log`) identifies all destinations; VRAM addresses below are bytes:

| Source instruction starting DMA | A-bus source | Differing destination bytes |
| --- | --- | --- |
| `$00:8A35`, `$00:8A59` | Six Link body/head/hand transfers from `$10:0000`, the low-WRAM mirror | 24 in `$8000..$829F` |
| `$00:8A9F`, `$00:8AD8` | Ten equipment/auxiliary transfers from `$7E:0000` | 40 in `$80A0..$83BF` |
| `$00:8AFE`, `$00:8B22` | Four pointer transfers from `$7E:0000`, plus expanded planes from `$7E:BD40` / `$7E:BD80` | 16 in `$8400..$867F`, plus 128 at `$8480..$84BF` / `$8680..$86BF` |
| `$00:8B64` | Animated BG transfer, `$7E:0000`, length `$0400` | 10 in `$0000..$01FE` |

The repeated low-WRAM bytes are `$0001=80`, `$0012=01`, `$0013=80`,
`$0015=01`. The last transfer additionally sees six nonzero live handler
stack bytes; the expanded-plane bytes retain the reset `$55` fill. Native
had zero at every one of the 218 differing destinations. The repair runs
the existing native reset/upload, memory, wait and NMI owners through the
opening handler before logical main starts. No ROM executor, recorded stack,
host-number threshold or copied oracle bytes enter production. The bootstrap
does not consume an intro work-schedule entry or logical iteration. The
existing synchronous logical audio bank remains its playback owner; the
temporary reset CPU/APU clock and scanline history retire at this handoff.

An unmodified-ROM CPU test independently executes reset through the first
NMI return and compares **all VRAM, CGRAM and OAM** with native bootstrap.
The final native VRAM also equals the source at hosts **172 and 966**.
At host 180, the source has already begun the following main's graphics
reload, so its scanout-time VRAM differs from native iteration 24's closing
state by 19,037 bytes; this is not reported as equality.

The original 218-byte discrepancy was **not the visible host-172 miss**.
After repairing it, the modern picture still had eight magenta pixels absent
from both the ROM scanout and native static line-renderer image. Its owner
was `snes9x_boot_material`: the host-timed overlay inserts stack-corner
pixels when `frame_ctr_dbg == 83`. Lagless iteration 16 happens to reach
that callback number. Lag-free capture now uses its actual PPU surface
without that host-timed overlay. The final host-172 picture is byte-exact,
RGB SHA-256 `8481fbad99d28b079d4260ec1e0223f0dc9c3e4c1965e4c2a6fd8b056b776834`.
`--dump-wram` alongside `ZELDA3_LOGICAL_CPU_PICTURES` now saves the selected
software picture as `engine-<iteration>.cpu.ppm` for pixel-level evidence.

All **2,320** fresh ROM-side CPU scanouts equal their same-host cache hashes
(`census-source/host-pictures.txt`). On iterations **0..1,999**, the modern
CPU baseline is **1,628 exact / 364 previous-iteration matches / 8 other**;
the final build is **1,629 / 364 / 7**. These are literal CPU hash counts,
not a rerun of the older GPU result **1,627 / 365 / 8**. No GPU-count
equivalence is claimed.

The reproduced late population has two owners (`late-causes.json`):

* **359 file-name scroll pictures**, first iteration **798 / host 1,094**.
  Native and ROM closing WRAM agree on `$0630=$01F0`. The *following*
  iteration's `$0C:DA98` writes `$01EF` in the preceding vblank (line 254),
  and `$00:82FB/$00:8301` consume it into BG3HOFS at the line-128 V-IRQ.
  The ROM picture therefore already contains next-main state; the logical
  capture still uses `$01F0`. Source witnesses retain every `$0630/$0631`
  write. A pre-main Link-DMA operand trial changed no picture hashes and
  was removed; the incidental 20-byte OBJ difference was not this cause.
* **Five screen-clear transitions**, first iteration **762 / host 1,057**,
  then hosts **1,579, 1,652, 1,761 and 2,215**. The following main reaches
  `$00:893F` (`STA $2100`, forced blank) / `$00:8942` (`STA $13`) before
  visible rendering. Host 1,652 reaches the store on line 1. The current
  logical closing image still shows the preceding menu. The same next-main
  blanking explains the remaining early misses at hosts **180 and 966**.

These are event-order/publication differences, not demonstrated missing
instruction prices. Replacing scroll with a one-step prediction, shifting
every capture, or fitting a scene cost would hide that ownership boundary.
No such change, new price, scene table or classifier change is retained.
The seven remaining non-exact/non-previous CPU pictures are iterations
24, 728, 765, 1118, 1186, 1465 and 1885. Full intro picture acceptance and
the exact reconciliation with the older GPU census remain open.

Verification of the retained changes: five targeted CPU unit tests pass
(opening-NMI surface equality, two reset-memory tests, NMI entry bus cost,
and logical capture's boot-overlay exclusion). Default and
`--no-default-features` product builds are warning-free; the library-test
build retains unrelated dead-code warnings. The full seeded ROM census
(`--seed-every 2000`) passes **1,473,123 iterations**, unexplained **0**.
Its missing/excess/false-positive counts remain **2,393 / 0 / 0**;
all pacing rows equal the retained `target/poly2/dma-census.csv` baseline,
and runtime held counts agree with `cost_p2` on every row. Receipts:
`rom-gate.log`, `final-pacing-comparison.json`, the `test-*.log` files and
`build-final-*.log` under `target/intro1/`. `source-identity.json` binds
the production sources, binaries and picture receipts to this verification.
The continuous recorded ROM run also passes all **1,473,123 iterations**
with unexplained **0** (`continuous-gate.log`). `git diff --check` passes;
task evidence occupies approximately **210 MB**. No GPU, live replay, full
library suite, protected-directory write, formatting, commit or push was
performed. This is a verified partial repair, not intro picture acceptance.

GPU confirmation from cold (not executed):

```sh
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
L=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
mkdir -p target/intro1/gpu-saves
ZELDA3_SAVE_DIR="$PWD/target/intro1/gpu-saves" target/intro1/zelda3-default \
  --logical-replay "$L" "$C/initial.srm" --frames 2000 \
  --video "$C" --video-misses --report 0 \
  > target/intro1/intro-confirm-gpu.txt 2>&1
```

## Lagless picture publication

CPU-only audit on `codex/picture2`, based on `9c155031`. The retained changes
add diagnostic coverage; they do **not** repair production publication order.
The authoritative cache and `full-v2` ledger were read only. All new evidence
is under `target/picture2/`.

`--logical-replay --cpu-video CACHE_DIR` now runs the existing picture
comparison with the software compositor. Three bounded workers render
immutable captures; results retire in submission order. It creates no GPU
renderer or Snes9x process. `ZELDA3_LOGICAL_CPU_PICTURES` retains the hashes,
and `--dump-wram` can retain a selected CPU PPM. The neighbor, held-picture,
and torn-picture classifiers are unchanged (`matcher-identity.json`).

Two complete seeded runs (`--seed-every 2000`) produce these identical counts:

| Picture class | Before diagnostics finalized | Final binary |
| --- | ---: | ---: |
| Exact | 1,462,997 | 1,462,997 |
| Late | 3,952 | 3,952 |
| Early | 563 | 563 |
| Torn | 4,702 | 4,702 |
| Polygon | 905 | 905 |
| During lag | 4 | 4 |
| Other content | 0 | 0 |
| Total | 1,473,123 | 1,473,123 |

Every picture hash and every pacing row is byte-identical between the two
runs (`before-after-verification.json`). These are before/after **diagnostic
changes**, not an improved after-repair result.

The supplied `d6eb2365` GPU census reconciles exactly in its named categories
(`gpu-cpu-reconciliation.json`). Retained ending fixes add 2,150 exact,
add 685 late, and remove 2,835 polygon pictures. The retained intro fix adds
one exact and removes one polygon picture. CPU rendering adds one further
exact and removes one late picture at iteration 1,355 / host 1,653: CPU and
the authoritative cache show the black frame, while the retained GPU result
matches host 1,652. Early and torn counts are unchanged. The supplied buckets
leave four pictures unassigned; the current CPU census classifies four as
during-lag. Their original GPU classification is not established by the
supplied totals. No new GPU run was performed.

The ledger example accepts `--picture-events HOSTS_FILE`, a list of decimal
host indices. It retains selected software scanout hashes and compressed
store events, including PC, register, old/new value where available, absolute
bus-access time, beam position, and interrupt context. The observer runs
before interrupt-acceptance bookkeeping: a store belongs to the instruction
that executed it, even when an NMI or IRQ is accepted at retirement.
`source-full/` follows the full ROM route and observes every missed picture
host plus its neighbors. Source scanout hashes are diagnostic; the pinned
cache remains the picture authority.

The following exhaustive partition covers all 9,217 late/early/torn misses.
Only the two intro rows have an isolated causal owner. The remaining rows
state observed conditions, with precedence: polygon phase, active-display
NMI return, P2 deficit, candidate writes, unresolved. They must not be read
as 9,217 causally explained pictures. `picture-owners.jsonl.zst` retains
each miss, its current/next P2 rows, and source store witnesses;
`publication-audit.json` contains first examples for every row/class.

| Owner or observed condition | Late | Early | Torn | First relevant iteration / host |
| --- | ---: | ---: | ---: | --- |
| Confirmed next-main file-name scroll | 359 | 0 | 0 | 798 / 1,094 |
| Confirmed next-main screen clear | 5 | 0 | 0 | 762 / 1,057 |
| Other: polygon phase | 829 | 2 | 0 | 1,453,021 / 1,558,056 (early) |
| Closing NMI returns during active display | 88 | 409 | 114 | 765 / 1,061 (torn) |
| Current/next P2 missing-hold deficit | 12 | 1 | 14 | 147,082 / 160,470 (late) |
| Other: next-main write candidates | 2,643 | 141 | 4,557 | 1,118 / 1,414 (torn) |
| Other: unresolved | 16 | 10 | 17 | 192,069 / 210,154 (torn) |
| Total | 3,952 | 563 | 4,702 | |

Including overlaps with other rows, only 61 late/early/torn pictures have a
current or next-iteration P2 deficit (33 late, 7 early, 21 torn). No extra
holds are observed. A P2 deficit establishes a pacing discrepancy, not its
causal responsibility for the picture.

Changed next-main HDMA-table stores are the largest candidate population:
7,163 missed pictures (2,672 late, 142 early, 4,345 torn, 4 during-lag).
This count overlaps the partition above. The mirror shift PCs each occur on
3,045 missed pictures; wave insertion PCs each on 2,327; dewaving PCs each
on 1,824; spotlight `$00:F3BB` on 1,738. The same picture can contain several
of these writers. Forced-blank transitions appear on 233 misses, HDMA setup
stores on 170, file-name scroll stores on 360, and other PPU stores on 1,855.
These are observed writer counts, not a demonstrated dominant causal owner.

The source renderer matches the authoritative cache on 10,121 of the 10,126
missed hosts. Exceptions are 5,934, 27,888, 28,215, 775,619, and 776,210;
they remain explicit in the audit. Source execution/WRAM equality does not
make these five software pictures authoritative.

Source witnesses distinguish several mechanisms:

* File-name scroll: iteration 798 / host 1,094 is the first of 359 previously
  isolated late pictures. Next main writes `$0630` at `$0C:DA98` in vblank;
  `$00:82FB/$00:8301` consume it into `$2111` at the line-128 IRQ.
* Screen clear: iteration 762 / host 1,057 is the first of five isolated
  late pictures. Next-main `$00:893F` writes forced blank to `$2100` before
  scanout. The retained intro CPU hashes preserve both regression groups.
* Spotlight/window transitions: the first gameplay torn picture, iteration 4,280 /
  host 4,804, has next-main `$00:F3BB` stores to the dynamic HDMA table.
  Channel 7 consumes this table into `$2126/$2127` (window bounds).
* Raster publication: the first early picture, iteration 5,077 / host 5,722,
  has closing-NMI register publication on lines 0–1. A mid-display forced
  blank also occurs at iteration 5,289 / host 5,934, from `$00:893F` on
  line 49. The source software picture at host 5,934 differs from the cache;
  its store witness alone does not establish pixel equivalence.
* HDMA setup: iteration 5,394 / host 6,044 has next-main
  `$0A:BAF5/$0A:BAF8` stores to `$4362/$4372` during vblank.
* Mirror waves: iteration 191,996 / host 210,035 has table-shift stores at
  `$00:FE82/FE85/FE88/FE8B` and new entries at `$00:FF20/FF23/FF26/FF29`;
  dewaving uses `$00:FF4D/FF50/FF53/FF56`. Channels 6/7 consume these tables
  into BG1HOFS/BG2HOFS (`$210D/$210F`).

These store inventories are not counterfactual pixel proofs. In particular,
the native mirror routines currently perform four stores and then price a
shared ROM block. That aggregate P2 price is not an individual store time.
A faithful general repair still needs a native clock for individual stores
across main-iteration boundaries, plus timed HDMA/IRQ consumption. Feeding
recorded ROM writes into native execution, predicting the next table, or
globally shifting capture would not provide that model. No such change is
retained. The causal ownership and publication repair remain incomplete.

Verification: twelve focused CPU tests pass, including file-name input/scroll,
the screen-clear and forced-blank suffix cases, opening NMI, capture, the
software queue, and ledger boundary tests. Default and
`--no-default-features` builds and the ledger-example build are warning-free.
The final binary's seeded ROM census and continuous **recorded** ROM run each
pass all 1,473,123 iterations with unexplained 0. Seeded missing/excess/false
positive counts remain **2,393 / 0 / 0**; every pacing row equals the retained
intro baseline and every native held count equals P2. The first 2,000 CPU
picture hashes also equal the retained intro result. Receipts include
`test-results.json`, `build-results.json`, `after-seeded-gate.log`,
`final-continuous-gate.log`, `intro-regression.json`, and
`source-identity.json`. All 1,473,123 source ledger records match `full-v2`
after excluding optional picture hashes, and both compressed WRAM streams
are byte-identical (`source-full-identity.json`). No GPU run, live replay,
full library suite, protected-directory write, formatting, commit or push
was performed. `git diff --check` passes. Evidence occupies approximately
1.8 GB, below the 3 GB cap. This is a verified audit stop, not a completed
publication repair or full causal classification.

Full-route GPU confirmation, seeded with misses (not executed):

```sh
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
L=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
mkdir -p target/picture2/gpu-saves
ZELDA3_SAVE_DIR="$PWD/target/picture2/gpu-saves" target/picture2/zelda3-default \
  --logical-replay "$L" "$C/initial.srm" --seed-every 2000 \
  --video "$C" --video-misses --report 0 \
  > target/picture2/full-confirm-gpu.log 2>&1
```

## Lagless HDMA store timing

CPU investigation on `codex/picture3`, base `5d1ec663`. **HDMA table timing is
confirmed for the dominant populations, but the native publication repair was
withdrawn after exact-picture regressions.** Only diagnostic code is retained.
All evidence is under `target/picture3/`; the oracle cache and `full-v2` were
read-only.

The audit reconstructs each table byte at its HDMA consumption deadline from
the ROM lane's `--picture-events` stores, absolute bus timestamps and field
coordinates. `$00:F2FB` supplies two contiguous 120-line runs; `$0A:BDDD`
supplies two 112-line runs. Output row `y` consumes `$1B00 + 2*y` through the
HDMA event due at physical V=`y`, H=1106, before rendering line `y+1`.
The audit's `read_at` records this event deadline; the source drains the event
after the current CPU transaction, so it is not a separately observed DMA
bus timestamp. The source's direct WRAM
word transaction commits both bytes before draining its 16 clocks. A store
starting at the HDMA deadline is too late; none of the selected witnesses
starts exactly there (`equality-boundary-audit.json`).

Three software pictures are rendered over the same native capture: old table,
timed table and new table. Every old-table control reproduces the native
baseline, and all reconstructed old table bytes equal native bytes. A case is
confirmed only when the timed picture equals the authoritative cache hash.
Neither the recorded stores nor these diagnostic pictures enter the engine,
publication queue or matcher. `ZELDA3_LOGICAL_HDMA_AUDIT=<directory>` enables
the diagnostic with that directory's `witnesses.jsonl` input and `audit.jsonl`
output. The input retains the source event fields, grouped by host, with the
iteration, field, DMA setup and writer group.

| Tagged population | Confirmed | Table-only explanation refuted | Total |
| --- | ---: | ---: | ---: |
| Spotlight | 1,737 | 1 | 1,738 |
| Mirror waves, insertion and dewaving | 4,830 | 81 | 4,911 |

These are exhaustive counts for both tagged populations. A failed table-only
counterfactual does not establish that table timing contributes nothing.
The spotlight exception is iteration **357,032 / host 388,747**, which also
has `$00:F3E2` forced blanking on line 221. Two other cases, iterations
**1,126,653 and 1,132,238**, require the later spotlight-reset stores as well
as `$00:F3BB`; both confirm with that complete table-store sequence. Their
supplemental probes reproduce the full-route baseline hashes exactly.

At **iteration 4,280 / host 4,804**, only physical lines **222–224** consume
changed next-main window bounds. The timed picture is
`997743f9c48d1c071856ef53bb35e172f4fa6ee46e9decf81060a8a8a1dfdf13`, exactly
the cache; both uniform generations fail. Mirror iteration **191,998 / host
210,037** similarly requires mixed generations and matches
`c5c3c52221e4a1f1903ea3063dd805a31baf78ad7f7fb62b4df159e1058e8219`.
`hdma-causality.jsonl.zst` records every case and all 224 line reads;
`spread-samples.json` retains **20 spotlight and 22 mirror examples** spread
over the route. `table-only-counterexamples.json` lists all 82 failures.

The withdrawn implementation split the spotlight and mirror ROM blocks at
each store, retained native store clocks and deferred rendering until the
next main's stores could be consumed. It gained seven exact pictures in the
cold 6,000-iteration prefix. The longer trial lost previously exact pictures
at iterations **48,550, 50,896, 59,220 and 101,022** in its first **108,702**
iterations, so all native changes were removed. This trial was stopped and
is not a full-route after result. Its patch, binary and receipts are retained
as `withdrawn-native-publication.patch`, `zelda3-candidate`, `withdrawal.json`
and `candidate-*`. The separate `invalid-zero-seed-*` run used a foreign seed
at iteration zero and is excluded from regression evidence.

The first regression has a fresh ROM-side witness. At host **54,177**, the
source copy starts at **V195/H940**, while the native P2 trial places it at
**V193/H820**, **2,848 clocks early**. Both have the same **39,884-clock
first-to-last-store interval**. The source's timed table preserves the exact
old picture; the early native stores damage it. All seven source scanout
hashes around this probe equal the cache. See `regression-cause.json` and
`regression-source/`. The unresolved owner is the wall phase before the copy;
splitting its stores does not repair that phase. No fitted offset is retained.

Both full-route seeded CPU censuses complete **1,473,123 iterations**:

| Picture class | Before | Retained after |
| --- | ---: | ---: |
| Exact | 1,462,997 | 1,462,997 |
| Late | 3,952 | 3,952 |
| Early | 563 | 563 |
| Torn | 4,702 | 4,702 |
| Polygon | 905 | 905 |
| During lag | 4 | 4 |
| Other content | 0 | 0 |

Every picture hash and pacing row is byte-identical. **Retained exact-picture
regressions: none.** The original owner partition is retained; its class
counts are identical before and after:

| Owner row | Non-exact before | Non-exact after | Late | Early | Torn | Polygon | During lag |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| File-name scroll | 359 | 359 | 359 | 0 | 0 | 0 | 0 |
| Screen clear | 5 | 5 | 5 | 0 | 0 | 0 | 0 |
| Polygon phase | 1,736 | 1,736 | 829 | 2 | 0 | 905 | 0 |
| Active-display NMI return | 611 | 611 | 88 | 409 | 114 | 0 | 0 |
| P2 deficit | 27 | 27 | 12 | 1 | 14 | 0 | 0 |
| Next-main candidates | 7,345 | 7,345 | 2,643 | 141 | 4,557 | 0 | 4 |
| Unresolved | 43 | 43 | 16 | 10 | 17 | 0 | 0 |

Verification: seeded ROM census (`--seed-every 2000`) and continuous recorded
ROM replay both **PASS**, all **1,473,123 iterations**, **unexplained 0**.
Missing/excess/false-positive counts remain **2,393 / 0 / 0**; native holds
equal P2 on every row. All **14 focused Rust tests** and **7 block-generator
tests** pass, including store boundaries, intro scroll, screen clear and
forced-blank publication. Default and `--no-default-features` product builds
are warning-free. The final diagnostic binary also reproduces all 2,000 intro
hashes. Full gates used `zelda3-default`; subsequent audit-only equality/reset
coverage is bound by zero equality witnesses and the two supplemental probes.
`verification.json`, `test-results.json` and `build-results.json` retain the
counts. `git diff --check` passes; evidence occupies approximately **1.2 GB**.
No GPU, live replay, full library suite, protected-directory write, formatting,
commit or push was performed. This is a verified causal audit and withdrawal,
not a completed native publication repair.

Exact full-route GPU confirmation command for the retained behavior (not run):

```sh
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
L=/Users/missingno/Documents/zelda3-rs/target/logical-ledger/full-v2
mkdir -p target/picture3/gpu-saves
ZELDA3_SAVE_DIR="$PWD/target/picture3/gpu-saves" target/picture3/zelda3-default \
  --logical-replay "$L" "$C/initial.srm" --seed-every 2000 \
  --video "$C" --video-misses --report 0 \
  > target/picture3/full-confirm-gpu.log 2>&1
```

## Lagless missing-hold owners

CPU investigation on `codex/lag1`, base `f011a2aa`. Evidence is under
`target/lag1/`; the cache and `full-v2` are read-only. The baseline has
**2,393 missing holds in 2,363 iterations, zero excess and zero false
positives**. Clusters join missing rows at most 120 logical iterations apart
with the same module/submodule and room (and overworld area). This is an
analysis grouping, never a runtime condition. `clusters.json` retains the
complete ranked partition and its membership.

There are **685 clusters**. `cluster-routine-comparison.json` and
`cluster-routines.csv` compare every exclusive routine in the leading **76
ordinary-gameplay clusters**, covering **1,438 holds (60.09%)**. Their source
and native main totals are **446,178,280 / 414,464,956 clocks**. They retain
negative alias balances and zero-native-cost bomb/tile-body witnesses, not
just a positive-only ranking. The remaining clusters are counted but do not
all have a source-routine attribution. Even within the profiled population,
the audit does not establish a unique causal routine for every missing hold.

The concentration is ordinary main work: `07/00` has **1,195** missing holds,
`09/00` has **561**, together **73.38%**. The retained ROM routine and
structural profiles cover all 1,756 of these iterations. Source main executes
**557,446,168 instruction clocks**, versus **518,918,760** natively: a
**38,527,408-clock** gap. Every row has positive instruction undercharge;
both lanes have zero main DMA and the source has zero raster-wait reads.
Thus these rows do not support an omitted large DMA/upload/wait explanation.
Appending the measured instruction gap to the frozen completed-model inverse
would reach 1,748 of the 1,756 holds. That is a diagnostic bound, not a price
or permission to add the difference at runtime. The eight remaining inverse
residuals are **6..690 clocks** (`phase-residuals.json`); interrupt/HDMA/refresh
placement remains unresolved there.

The three largest clusters are repeated visits to room `$003E`:

| Iterations | Missing before | Missing after | Source main minus baseline native main | Unpriced Bomb_Draw self work |
| --- | ---: | ---: | ---: | ---: |
| 787,135–787,528 | 114 | 108 | 2,619,920 | 237,074 |
| 820,396–820,779 | 113 | 102 | 2,660,240 | 249,174 |
| 841,603–841,778 | 62 | 59 | 1,385,388 | 135,598 |

The source and native routine clocks are exclusive, with signed differences
retained. A positive parent label alone is not an untimed body: collision,
velocity, control-handler and drawing tails can carry compensating negative
labels. The matcher’s unresolved call/scalar aliases remain unresolved; its
classifiers and pricing-admission rules are unchanged. A 2,000-iteration
diagnostic from seed 786,000 reproduces canonical main costs on every missing
row in that window (`room3e-pacing.csv`).

`Bomb_Draw` is a concrete whole untimed caller body, `$08:9E9E..9FB5`:
native coordinates, allocation, animation selection and shadow dispatch
execute without their own instruction charge. Its already priced coordinate
and explosion children are separate. Across the ordinary missing population,
308 iterations execute **677,072** source self clocks here; its isolated
frozen-inverse estimate reaches 20 holds. The candidate uses generated ROM
block markers, actual branch outcomes, and no scene/count fitting. The ROM
fixture also exposes an easily misread branch: `$9EB1 STA $04` preserves
`CMP #$0080` / `ORA #$FF00` flags, so `$9EB3 BEQ` is not a zero-altitude test.
The pricing follows those flags. Existing gameplay/output behavior is not
changed by this marker addition.

A fresh **1,100-host cold source-CPU probe** then locates an earlier whole
omission, `Intro_Clear1kbBlocksOfWRAM`, `$0C:C1A0..C1F4`. The source routine
profiler labels its tail under `Module_MainRouting`; the per-PC witness,
not that parent label, identifies the work. Each boot call clears fifteen
WRAM word columns 512 times. Native performs those stores but previously
charged none of the loop. Generated markers now retire the setup, each
actually executed fifteen-store column and its actual taken branch, and
the cursor/return suffix: **410,878 instruction clocks per normal call**.
No fixed hold count is installed. `early-source/instruction-profile.csv`,
`timing-audit.csv`, and `early-owner-comparison.json` retain the source
breakdown, including unheld controls. The eight full ranges and three short
boundary fixtures compare both source instruction work and resulting WRAM.
The cold 2,000-iteration trial restores all eight holds at iterations 0–7,
with zero excess/FP and all 2,000 picture hashes unchanged. The boot shortfall
before the first recorded poll falls **23 → 15** in that trial.

The next early loading gap is a different, unadmitted group. At iteration
729, source/native main is **19,589,454 / 16,385,366 clocks**. Positive
exclusive labels include `$00:E766`, `$00:E843` and the dispatcher; the
`$00:E6B7` positive cancels `$00:E583` exactly. Existing decompression and
packing claims need a new instruction/caller partition before adding work.
The report does not turn those raw deltas into a new loader surcharge.
`load_gfx.rs::charge_sheet_decompression` explicitly documents another lead:
the packed input for junk sprite sheet 7 ends before its source stream, so
the existing instruction model stops pricing early. This investigation has
not isolated that truncated-input contribution from the other loader aliases
at iteration 729; it is a named next investigation, not an admitted correction.
`remaining-unpriced-screen.json` separately records the inspected ancilla
helpers still lacking self prices. On remaining profiled misses, the one-floor
tile body `$08:8A03` has **799,316** source clocks and its wrapper `$08:8981`
has **755,246**; their independent frozen-inverse screens reach **24 / 19**
holds. Shadow drawing reaches 12, bomb damage and Link collision seven each.
These overlapping screens have neither all-path fixtures nor a full-route
zero-FP trial and are not production prices or promised recoveries.

`poll-iterations.csv` retains cumulative baseline missing holds before every
recorded APU-read iteration; `poll-deficits.json` groups contiguous reads.
The ten long song-end windows have the following cumulative deficits (the
count does not change within these windows):

| Song-end window | Cumulative missing before repair | Cumulative missing after repair |
| --- | ---: | ---: |
| 94,334–94,854 | 79 | 71 |
| 141,568–142,088 | 115 | 107 |
| 228,499–229,035 | 253 | 245 |
| 380,243–380,881 | 407 | 399 |
| 461,989–462,628 | 486 | 478 |
| 626,618–627,256 | 796 | 788 |
| 735,229–735,867 | 1,059 | 1,051 |
| 881,095–881,734 | 1,541 | 1,513 |
| 1,064,096–1,064,734 | 1,894 | 1,866 |
| 1,317,697–1,318,335 | 2,129 | 2,101 |

The nominal missing-hold contribution is `count * 357366 * 15664 / 328125`
SPC cycles (about 17,059.904 per hold), also retained per poll. This is not a
measured absolute chip deficit. Masked upload callbacks, host/acceptance cuts,
bootstrap origin, fractional sample phase, and command publication phase are
separate terms in the sound proof. No continuous live replay was used to
infer equality between that nominal contribution and actual SPC state.

Both repairs together finish the seeded ROM census (`--seed-every 2000`) and
continuous recorded ROM replay: **1,473,123 iterations, PASS, unexplained 0**.
The full timing results are:

| Stage | Missing | Excess | FP |
| --- | ---: | ---: | ---: |
| Fresh baseline | 2,393 | 0 | 0 |
| Bomb_Draw only | 2,373 | 0 | 0 |
| Bomb_Draw plus intro WRAM clear (withdrawn, see merge note) | 2,365 | 0 | 0 |

Every runtime held count equals P2. There are no newly missing iterations.
`cluster-results.csv` / `.json` report all 685 clusters: the three bomb
clusters above and boot clusters **0: 1→0**, **1–7: 7→0** improve; the other
680 are unchanged. `hold-changes.json` lists all 28 restored iterations.
`poll-deficits-after.json` retains the after count at both ends of every read
episode. This is a partial owner repair; the remaining early song-end deficit
is still 71 holds, and no continuous live acceptance is claimed.

The combined full-route seeded CPU picture census also completes all
**1,473,123 iterations, PASS, unexplained 0**. Exact pictures remain
**1,462,997**; every picture hash is identical to the retained baseline.
**Lost exact pictures: none.** `verification.json` binds the final binaries,
source files, predictor rows and picture hashes to the checks; the full
gate logs and per-cluster evidence remain under `target/lag1/`.

Validation: 12 targeted Rust unit tests and seven generator tests pass;
default and `--no-default-features` product builds are warning-free.
`git diff --check` passes. Evidence occupies less than 1 GB, below the 3 GB
limit. No GPU run, continuous live replay, seeded live census, full library
suite, protected-cache/ledger write, formatting pass, commit or push was
performed. This is a verified partial owner repair, not complete attribution
of the remaining 2,365 holds or live acceptance.

**Merge note.** The seeded live-APU census separated the two repairs:
`Bomb_Draw` alone gives **2,647 / 270 / 75** (baseline 2,667 / 270 / 75), while
`Bomb_Draw` plus the intro WRAM clear gives **2,753 / 360 / 80**. Only the
`Bomb_Draw` markers were merged (ROM **2,373 / 0 / 0**). The intro-clear markers
and their ROM fixture are withdrawn pending a live-lane explanation: they
are ROM-correct in the ROM lane, but shifting boot time moves the live APU
read schedule and adds 90 excess holds.

## Lagless missing-hold owners 2

CPU investigation on `codex/lag2`, base `7c66aca9`. Evidence is under
`target/lag2/`; the oracle cache and `full-v2` remain read-only. Starting
ROM missing/excess/FP is **2,373 / 0 / 0**, seeded live **2,647 / 270 / 75**.

The intro-clear price remains **withdrawn**. The retained complete live
predictors first differ at iteration **0**: main work **220,648 → 631,526**,
holds **0 → 1**. The same 410,878-clock source price restores each of
iterations 0–7. Fresh seeded live diagnostic prefixes reproduce **407,846
baseline** and **397,581 intro-clear** complete predictor rows exactly.
Both were deliberately interrupted after the first regression window;
these are prefix reproductions, not new full census passes.
`reused-identities.json` binds the binaries and full retained predictors;
`intro-live-diagnosis.json` records the comparison.

Iteration zero requests `$2143=$0A` on both binaries. The added held callback
then clears it, so the final port vectors differ (`[00,00,00,0A]` versus
zeros). No song-end poll or masked upload occurs during the eight clears.
Row 8 retains a changed legacy preceding-handler diagnostic, with unchanged
P2. The first post-boot P2 handler-budget difference is **1,917**: opening/lag handler
budgets **35,428/3,018 → 35,474/3,064**, with holds still **1/1**. The
preceding NMI's ambient echo is `$05` versus `$03` while the last request
is `$03`; the existing `$00:8108..8113` acknowledgement branch accounts
for the 46-clock difference. See `boot-ports.json`, `boot-*.jsonl.zst`, and
`first-handler-echo.json`.

The first changed upload is **10,425**: **23 → 22 masked callbacks**, both
with **58 accepted holds**. Fresh upload traces contain **25,510 identical
ordered bank/cursor/port/value/phase writes**, including headers, payload
counters and four final clears. First-header-to-last-clear spans are
**7,487,012 / 7,487,220 master clocks**. Equal payloads do not establish
equal acknowledgement timing or chip state (`upload-identity.json`). Before
380,880, eight extra holds and four fewer masked callbacks leave a net
**four callbacks / 2,136 returned sample pairs**. This is playback-duration
accounting, not absolute SPC-state equivalence.

The first changed main-work path after boot is **380,880**. Baseline reads
APUI00 **$01**, intro-clear reads **$00**, and the source reads **$13**.
Both native rows are unheld; main work is **56,846 / 57,068** clocks. Zero
takes the crystal's song-end branch one iteration early. Work then moves
onto source-unheld iterations; reseeding at 382,000 changes the subsequent
loader lineage. The complete excess increase is:

| Iteration | Source holds | Baseline → intro-clear holds | Added excess |
| --- | ---: | ---: | ---: |
| 380,945 | 0 | 0 → 1 | 1 |
| 380,955 | 0 | 0 → 4 | 4 |
| 381,425 | 0 | 0 → 8 | 8 |
| 383,178 | 0 | 0 → 7 | 7 |
| 383,346 | 0 | 0 → 7 | 7 |
| 383,634 | 1 | 1 → 59 | 58 |
| 383,635 | 1 | 1 → 6 | 5 |

These rows explain **90 excess holds and five new FP iterations** through
changed execution, not an extra intro-clear price on those rows. The boot
CPU/PPU handoff already executes the native cold upload, memory initialization
and opening NMI under ledger muting, then restores logical playback's separate
audio state. Charging that bootstrap inside iteration zero would put
pre-iteration work in the wrong owner. No missing boot counterpart or census
comparison defect is isolated. The evidence demonstrates carried SPC and
acknowledgement phase sensitivity; remaining work omissions and unequal audio
origins are unresolved. No fitted delay, source-answer substitution, matcher
change or audio reset conceals it. The full retained intro-clear live result
is still **2,753 / 360 / 80**, so that entire candidate remains withdrawn.

Two further source owners are tested: shadow drawing `$08:F897..F909` and
Link collision `$08:F76B..F7DB`. Generated ROM markers price actual branches
and returns. Shadow's `$08:F702` coordinate child keeps separate ownership.
Link collision includes the `$0BFA,X` page crossing for slots 6–9 and omits
horizontal work when the vertical test rejects. Fixtures cover **210 shadow
cases** (all seven shapes, clipping and OAM page boundaries) and **360 Link
cases** (all five hitboxes, slot-page cases, signed altitudes/distances and
exact rejection boundaries), comparing instruction work plus OAM outputs
or collision results. All **3,828 existing generated blocks** are unchanged;
**17 blocks** are added. No scene/count fitting is used.

The one-floor tile body/wrapper and bomb-damage prices remain unadmitted.
The loader audit confirms sprite sheet 7 begins at `$11:9A00` and reaches
its terminator after **1,705 bytes**, beyond the native **1,536-byte** window;
it describes **6,056 output bytes**. `sheet7-extent.json` preserves the
command walk and iteration 729's source routine counts. This proves the
truncation, not an exclusive additive price for `$00:E766/$00:E843` or
the surrounding loader. No loader surcharge or partial tile-body price is
retained.

The full seeded ROM and live censuses and the continuous **recorded** ROM
run each finish **1,473,123 iterations, PASS, unexplained 0**. Counts are
missing holds / excess holds / zero-source-lag FP iterations:

| Lane | Before | Retained two-owner result |
| --- | ---: | ---: |
| Seeded ROM | 2,373 / 0 / 0 | **2,354 / 0 / 0** |
| Seeded live APU | 2,647 / 270 / 75 | **2,628 / 270 / 75** |
| Continuous recorded ROM | 2,373 / 0 / 0 | **2,354 / 0 / 0** |

All 19 ROM changes restore a source hold, with no newly missing iteration:
**14 / 3 / 2** on the three room `$003E` visits around 787k / 820k / 841k.
`restored-rom-holds.json` records every changed iteration. These are the
joint result of the two complete owner prices, not isolated recovery claims
for either owner or a runtime scene rule.

All runtime held counts equal P2. ROM and continuous recorded predictors
are byte-identical, with **100,323 accepted holds / 5,148 masked callbacks**.
Live has **100,319 / 5,146**. Live aggregate excess/FP does not rise, but
two existing false-positive events move **881,793 → 881,794** and
**881,803 → 881,804** (one and four holds). This is not unchanged live
phase or continuous-live acceptance. The intro-clear remains withdrawn;
these two independent owner prices do not repair its audio-phase problem.

Validation: **39 targeted CPU marker tests** (including the 570 new ROM
cases) and **seven generator tests** pass. Default and
`--no-default-features` product builds are warning-free; the library-test
build retains unrelated existing dead-code warnings. The three timing gates
are `candidate-{rom,live,continuous}-gate.log`; `pricing-audit.json` and
`candidate-identities.json` bind the source/table/binary evidence. No GPU,
Snes9x process, continuous live replay, full library suite, protected-cache
or shared-ledger write, formatting, commit or push was performed.

The full seeded **CPU picture census** also completes **1,473,123 iterations,
PASS, unexplained 0**: **1,462,997 exact**, 3,952 late, 563 early, 4,702 torn,
905 polygon and four during ROM lag, with zero other content misses. Every
picture hash is byte-identical to the retained baseline: **zero exact
pictures lost**. `candidate-cpu-video-gate.log` and `verification.json` bind
the final counts and identities. `git diff --check` passes. Evidence occupies
less than **0.8 GiB**, below the 3 GB limit. This is a verified partial
owner-pricing stop; the intro-clear, other screened owners, loader partition
and continuous-live sound acceptance remain unresolved.

## Lagless missing-hold owners 3

CPU investigation on `codex/lag3`, base `1008a128`. Evidence is under
`target/lag3/`; the oracle cache and shared `full-v2` ledger remain read-only.
The intro WRAM-clear candidate remains withdrawn and was not retried.

The sprite decompression accounting now reuses the existing native
`sprite_graphics_decompression_stream` reconstruction. Raw sheets 0–11 are
contiguous source windows; `$00:E7A3..E852` walks commands through `$FF`,
including a command crossing a native sheet boundary. No tail surcharge,
new instruction price, source-ROM runtime read or sheet-specific exception
is introduced. The existing instruction model prices the reconstructed
stream; non-lag-free prices and graphics outputs are unchanged.

The asset-backed extent fixture checks all **108 sprite and 114 background
sheets** against their ROM streams, including the complete consumed bytes
and terminator. **Only sprite sheet 7 is truncated** in its individual asset:
1,536 bytes versus 1,705 through the terminator (6,056 output bytes).
The existing instruction-by-instruction CPU fixture passes **336 cases**
across the three decoder entries, with zero unfinished cases or mismatches.
The new fixture also compares the actual charged work in lag-free and
non-lag-free modes.

This correction restores **no holds on this route**. Both complete sheet-only
predictors are byte-identical to the retained main predictors, including
iteration 729: the active graphics loader already reconstructs the full
stream. The truncated accounting helper was a real local defect, but its
repair is not an explanation of that route's remaining loader gap. The
separate sheet-only ROM and live censuses both pass with unexplained zero.

The retained new owner is the complete tile-collision **wrapper**
`$08:8981..8A02`, excluding its `$08:8A03` children. Generated markers cover
outdoor-priority rejection, header selection, coordinate save/translation/
restore, both child calls, the first child's carry-dependent branch, and
the return. `$0BFA,X` page crossings are charged for the actual slot.
`$89F7 PHP` is consumed as data by `$89FB PLA`; the generator entries split
that source boundary rather than pretending it is a PHP/PLP restoration.
**480 unmodified-ROM fixtures** cover both worlds, all header branches,
priority rejection, hit/miss and off-screen paths, and both slot-page cases.
They compare wrapper instruction work and restored coordinates/floor.
The fixtures load the real asset pack so outdoor child results agree.
All **15,729 existing generated instructions** remain unchanged;
**13 blocks / 66 instructions** are added, without fitting to hold counts.

Full-route results are missing holds / excess holds / zero-source-lag FP
iterations; every run covers **1,473,123 iterations**:

| Candidate / lane | Missing | Excess | FP |
| --- | ---: | ---: | ---: |
| Main / seeded ROM | 2,354 | 0 | 0 |
| Main / seeded live | 2,628 | 270 | 75 |
| Sheet stream only / seeded ROM | 2,354 | 0 | 0 |
| Sheet stream only / seeded live | 2,628 | 270 | 75 |
| Sheet stream plus wrapper / seeded ROM | **2,336** | **0** | **0** |
| Sheet stream plus wrapper / seeded live | **2,583** | **243** | **70** |
| Sheet stream plus wrapper / continuous recorded ROM | **2,336** | **0** | **0** |

All five timing gates **PASS, unexplained 0**; every runtime hold equals P2.
The final seeded ROM and continuous recorded predictors are byte-identical,
with **100,341 accepted holds / 5,151 masked callbacks**. Live has
**100,337 / 5,143**. The wrapper restores 18 source holds without any newly
missing ROM iteration: one each at 625,401 and 767,843, twelve on the 787k
room `$003E` visit and four on its 820k visit. `wrapper/rom-changes.json`
lists them. Live adds **no excess rows** and removes five existing FP events
at 627,316 / 627,326 / 627,799 / 629,327 / 629,458, totaling 27 excess holds
(`wrapper/live-excess-changes.json`). These seeded improvements do not prove
equal SPC phase, strict sound identity or continuous-live acceptance.

Re-grouping the remaining misses with the same room/area key and 120-iteration
gap yields **684 clusters**. `07/00` and `09/00` retain **1,138 / 561** missing
holds; the largest room `$003E` clusters retain **95 / 82 / 57** holds on
the 820k / 787k / 841k visits. `wrapper/remaining-clusters.json` retains all
membership. This is a current population partition, not a new attribution of
positive parent-label differences to untimed bodies.

The one-floor **body** `$08:8A03` and bomb damage remain unadmitted. The
corrected body decoding is retained in `tile-body-source.json`: its outdoor
lookup `$00:882E` returns M8 after the caller entered M16, so `$08:8A53`
needs an explicit width boundary. The older listing's post-call mixed-width
decode cannot establish a price. Complete native/source all-path body and
bomb-damage fixtures, then separate zero-FP route trials, remain next work.
No partial body price, classifier/matcher change or new environment switch
is retained.

Validation: **12 targeted Rust tests** and **seven generator tests**
pass; default and `--no-default-features` product builds are warning-free.
Library-test builds retain the existing unrelated dead-code warnings.
`candidate-identities.json`, `candidate.patch`, `sheet-only.patch`, and
`wrapper-pricing-audit.json` bind the code, binaries and reused evidence.
The full seeded **CPU picture census** completes **1,473,123 iterations,
PASS, unexplained 0**: **1,462,997 exact**, 3,952 late, 563 early, 4,702 torn,
905 polygon, four during ROM lag and zero other content misses. Every hash
is byte-identical to the retained baseline: **zero exact pictures lost**.
`wrapper/pictures-gate.log` and `verification.json` retain the final checks.
`git diff --check` passes; evidence occupies less than **0.9 GiB**, below
the 3 GB limit. No GPU, Snes9x process, continuous live replay, full library
suite, protected-cache/shared-ledger write, formatting, commit or push was
performed. This is a verified partial owner-pricing stop; the one-floor body,
bomb damage, remaining loader partition and continuous-live sound acceptance
remain unresolved.

## Lagless missing-hold owners 4

CPU investigation on `codex/lag4`, base `9b601d84`. Evidence is under
`target/lag4/`; the oracle cache and shared `full-v2` ledger remain read-only.
The intro WRAM clear remains withdrawn and was not retried.

The fresh baseline completes all **1,473,123 iterations**, passes with
unexplained zero, and reproduces the retained main predictor byte-for-byte:
**2,336 missing / 0 excess / 0 FP**. Fresh native routine profiles are joined
to the retained unmodified-ROM self profiles from `target/block-worklist/source/`.
`iteration-routine-comparison.csv` retains every selected missing iteration
and every signed routine difference, including negative balances and dispatch
aliases. `cluster-profiles.json` contains the aggregate comparisons:

| Cluster start / context | Missing holds | Source main clocks | Native main clocks |
| --- | ---: | ---: | ---: |
| 820,396 / room `$003E`, `07/00` | 95 | 29,659,114 | 27,986,372 |
| 734,399 / room `$00AC`, `07/00` | 50 | 15,532,558 | 14,634,454 |
| 912,671 / area `$010F`, `09/00` | 42 | 12,888,918 | 12,311,000 |
| 1,399,071 / room `$006B`, `07/00` | 40 | 12,489,878 | 11,629,194 |
| 761,077 / room `$001F`, `07/00` | 38 | 11,728,480 | 11,300,386 |

On the selected room `$003E` visit, the one-floor body `$08:8A03`, bomb
damage `$08:8287`, and Y/altitude latch `$08:9C7F` have respectively
**222,680 / 28,930 / 62,890 source self clocks**, with zero native self price.
These exposures select source bodies for investigation; they are not prices
or independent predictions of recovered holds.

The one-floor body candidate uses the corrected `$00:882E` return-width
boundary from `target/lag3/tile-body-source.json`. Its ROM fixture finds a
pre-existing result discrepancy on tile `$18`, with priority/floor2/type/U
all zero: native returns hit while the ROM returns clear. The candidate's
self-work check passes at that case, but its carry-result assertion fails.
The native slope helper has profiles for `$10..$13` and falls back to hit
for `$18..$1B`, which the collision table also routes through the slope test.
The whole body candidate was withdrawn before a route trial; its patch,
binary and failing fixture log remain `withdrawn-body.patch`, `zelda3-body`
and `withdrawn-body-test.log`. No body price or slope-behavior change is retained.

Two complete source bodies are retained:

* `Bomb_CheckSpriteDamage`, `$08:8287..832A`: all sixteen slot tests,
  frame/hit/projectile/floor/state rejection paths, hitbox rejection, the
  special-sprite exemption, successful damage/recoil calls, and loop/return.
  Hitbox, overlap, damage and reflexive-speed children retain separate ownership.
  **1,920 unmodified-ROM fixtures** cover every slot, both ancilla page cases,
  every parent branch and representative altitudes. They compare self clocks,
  recoil and hit-timer results, and check that non-lag-free cost stays zero.
* `Ancilla_LatchYCoordToZ`, `$08:9C7F..9CC2`: saved Y, directional signed
  altitude, the `$FF` sentinel, coordinate stores and return.
  **61,440 ROM fixtures** cover all 256 altitude bytes, all 20 native
  directions, slot-page boundaries and coordinate wrap boundaries. They
  compare self clocks, saved/adjusted coordinates and the uncharged default lane.

The generated table adds **22 blocks / 109 instructions**; all **15,795**
pre-existing generated instructions are unchanged. Prices come only from ROM
blocks and actual branches/indexed crossings. There are no fitted counts,
scene tables, thresholds, new environment switches, or classifier/matcher changes.

Every timing run below covers **1,473,123 iterations**, **PASS, unexplained 0**:

| Candidate / lane | Missing | Excess | FP |
| --- | ---: | ---: | ---: |
| Main / seeded ROM | 2,336 | 0 | 0 |
| Main / seeded live | 2,583 | 243 | 70 |
| Bomb damage / seeded ROM | **2,332** | **0** | **0** |
| Bomb damage / seeded live | **2,579** | **243** | **70** |
| Bomb damage plus latch / seeded ROM | **2,325** | **0** | **0** |
| Bomb damage plus latch / seeded live | **2,572** | **243** | **70** |
| Bomb damage plus latch / continuous recorded ROM | **2,325** | **0** | **0** |

The final candidate restores **11** source holds with no newly missing ROM
iteration: seven on the 787k room `$003E` visit and four on its 820k visit.
The largest three room `$003E` clusters now retain **91 / 75 / 57** holds
on the 820k / 787k / 841k visits. `*-changes.json` retains each changed row;
`remaining-clusters.json` retains the remaining population. Bomb-only live
timing moves the existing one/four-hold FP events **881,794 / 881,804 →
881,793 / 881,803**, with unchanged excess/FP totals. The final two-owner
candidate leaves every baseline live excess row unchanged.

Every runtime hold equals P2. Final seeded ROM and continuous recorded
predictors are byte-identical, with **100,352 accepted holds / 5,149 masked
callbacks**; seeded live has **100,348 / 5,143**. These source-work and seeded
census results do not establish equal SPC phase or continuous-live sound acceptance.

The next-cluster PC partition names Blind head/control tails under the
`$1D:C222` dispatcher and Lumberjack Tree code under `$05:FBF7`, including
inherited JumpTableLocal work. `next-cluster-source-pages.json` preserves that
partition. The finer `next-owner-ranges.json` isolates **189,608 source self
clocks** at Blind head `$1D:A118..A1EC` and **119,900** at Lumberjack Tree
control/states `$05:FD4D..FE22` on their respective selected missing iterations.
Those bodies and the remaining multi-body label differences still need
exclusive all-path fixtures and separate route trials; no aggregate surcharge
is admitted.
Iteration 729's `$00:E766/$00:E843` loader partition supplies no new exclusive,
fixture-backed body in this batch and remains unchanged.

Validation: **12 targeted Rust tests** and **seven generator tests**
pass; default and `--no-default-features` product builds are warning-free.
The library-test build retains the existing unrelated dead-code warnings.
The final default rebuild is byte-identical to the census binary after the
test-only ordinary-sprite damage coverage correction.

The full seeded **CPU picture census** completes **1,473,123 iterations,
PASS, unexplained 0**: **1,462,997 exact**, 3,952 late, 563 early, 4,702 torn,
905 polygon, four during ROM lag and zero other content misses. Every picture
hash is byte-identical to the retained main baseline: **zero exact pictures
lost**. Its predictor is byte-identical to the final seeded ROM predictor.
`final-pictures-gate.log`, `pricing-audit.json`, `candidate.patch` and
`verification.json` retain the results and source/binary identities.

`git diff --check` passes; evidence occupies less than **1.2 GiB**, below
the 3 GB cap. No GPU, Snes9x process, continuous live replay, full library
suite, protected-cache/shared-ledger write, formatting, commit or push was
performed. This is a verified partial owner-pricing stop. The one-floor body,
next-cluster bodies, remaining loader partition and continuous-live sound
acceptance remain unresolved.

## Lagless missing-hold owners 5

CPU investigation on `codex/lag5`, base `490b0408`. Evidence is under
`target/lag5/`; the authoritative cache and shared `full-v2` ledger remain
read-only. Main starts at **2,325 / 0 / 0** ROM and **2,572 / 243 / 70** seeded
live missing/excess/FP. A fresh **628,474-complete-row** baseline prefix is
byte-identical to main's retained ROM predictor. That deliberately interrupted
prefix is not a new full census pass. The intro WRAM clear was not retried.

The retained source owners are:

* Lumberjack Tree control and states, `$05:FD4D..FE22`, including the inherited
  `$00:8784..879B` JumpTableLocal tail. The shared `$00:8781` entry keeps its
  own scope. Drawing, inactivity, damage, hookshot/dash response, movement
  and leaf-spawning children are excluded from the caller's self price.
  **216 unmodified-ROM fixtures** cover all **28 generated blocks** and both
  outcomes of all **eight conditional branches**, and compare state results.
  The state-table bytes are excluded; the width-independent `$FDCD SEP`
  convergence is explicit. The non-lag-free self price remains zero.
* The tree's Z-movement child, `$05:FA2E..FA4F`, bound at its actual `$FD75`
  call. It reuses the existing native Z algorithm with its own generated
  source entry. **4,608 ROM fixtures** cover all 256 velocity bytes, slot
  boundaries, and altitude/subpixel wrap boundaries, comparing clocks and
  resulting altitude/subpixel bytes. Other bank-5 Z call sites are unchanged.
  Rebased markers use entry-relative offsets, preserving the existing
  bank-6/1D/1E entries; the non-lag-free price is zero.
* Blind laser, `$1D:A268..A2CA`, excluding coordinate preparation, inactivity,
  damage, tile collision and trail-garnish children. **1,536 ROM fixtures**
  cover every head direction, inactive and delay returns, signed X/Y movement,
  and wall hit/miss. They visit all **16 generated blocks**, both outcomes of
  all **five conditional branches**, compare result bytes, and check zero
  non-lag-free self work.

`ownership-audit.json` records the partitions and ancestor dispatch audit.
`pricing-audit.json` finds **146 new instructions / 47 new blocks**, with all
**15,904 existing instructions unchanged**. No fitted prices, per-scene tables,
thresholds, P2 changes, matcher/classifier edits or new environment switches
are introduced. Each candidate has its own saved binary and ROM/live trial.
The separate 912,000–913,999 Tree-only window adds source work but restores no
holds: both baseline and candidate have **43 missing / 0 excess**. It is a
seeded diagnostic, not full-route acceptance.

Three behavior discrepancies prevent further body admission in this batch:

* **Blind head `$1D:A118..A1EC`: withdrawn whole before a route trial.**
  With sprite `$CE`, F=14, the ROM's bank-1D recoil tail `$1D:E93A..E943`
  returns to the head, but the shared native helper exempts only `$7A` and
  exits the head early. The candidate fixture measures **696 native / 1,836
  ROM self clocks**. A separate unpriced ROM witness confirms the consequence:
  the ROM continues and decrements subtype 2 to 1; native leaves it at 2.
  `withdrawn-blind.patch`, `withdrawn-blind-fixture.txt` and the fresh
  `test-laser-and-blockers-final.log` preserve the candidate and evidence.
  No charge for the unexecuted suffix or recoil behavior change is retained.
* **One-floor tile body `$08:8A03`: still unadmitted.** The existing native
  classification routes `$18..$1B` to slope collision, but `EntitySlope`
  decodes only `$10..$13`; the ancilla fallback returns hit. The new ROM
  witness checks every fine-coordinate pair for all four tiles: respectively
  **53 / 56 / 60 / 64 carry differences out of 64**. This is an actual
  behavior difference, not just a marker-width issue. Correcting it would
  change behavior, so neither a partial price nor a decoding change is made.
  `test-tile-witness.log` retains all four counts.
* **Next room `$006B` cluster: SpikeBlock statue collision `$1E:BE19..BE7D`
  is screened but unpriced.** ROM tests Y with `delta + 8 < 16`; native uses
  `delta + 16 < 32`. The ROM witness at Y delta 12 returns clear while native
  returns colliding. This is another behavior repair, not an additive clock
  omission. No price is inferred from its positive routine-label difference.

The raw first Blind-pricing failure log was accidentally overwritten by a
later existing-test run; its observed assertion is explicitly labeled in
`withdrawn-blind-fixture.txt`. The independent fresh behavior witness and the
complete withdrawn patch remain reproducible. Other Blind control/state tails,
DashItem dispatch, the loader partition and remaining clusters are not claimed
resolved. Source work and seeded timing counts do not establish equal SPC
phase, strict sound identity or continuous-live acceptance.

Every timing run below completes **1,473,123 iterations**, **PASS,
unexplained 0**. Counts are missing holds / excess holds / zero-source-lag
FP iterations:

| Candidate / lane | Missing | Excess | FP |
| --- | ---: | ---: | ---: |
| Tree control / seeded ROM | 2,325 | 0 | 0 |
| Tree control / seeded live | 2,572 | 243 | 70 |
| Tree control plus Z child / seeded ROM | 2,324 | 0 | 0 |
| Tree control plus Z child / seeded live | 2,571 | 243 | 70 |
| Tree control, Z child and Blind laser / seeded ROM | **2,323** | **0** | **0** |
| Tree control, Z child and Blind laser / seeded live | **2,570** | **243** | **70** |
| Final / continuous recorded ROM | **2,323** | **0** | **0** |

The Z child restores iteration **912,688**; the laser restores **734,471**.
Both had one source hold and no native hold. There are no newly missing ROM
iterations. Tree-only changes no hold row in either lane. The final live
aggregate excess/FP stays unchanged, but its existing one/four-hold FP events
move **881,794 / 881,804 → 881,793 / 881,803**. Every added and removed row is
retained in `final-live-excess-changes.json`; this is not unchanged live phase.

Every runtime held count equals P2. Final seeded ROM and continuous recorded
predictors are byte-identical, with **100,354 accepted holds / 5,156 masked
callbacks**; live has **100,350 / 5,145**. Accepted holds and masked callbacks
remain separate sound-proof terms. `timing-verification.json` binds the counts
and predictor equality. Per-candidate `*-changes.json` and
`*-remaining-clusters.json` retain the re-clustering after each candidate.
There are still **684 clusters**. The largest room `$003E` visits remain
**91 / 75 / 57** holds; the selected Blind and Tree clusters become **49 / 41**.
The next room `$006B` cluster retains 40 and its screened behavior gap above.

Validation: **32 focused Rust tests** and **seven generator tests**
pass. Default and `--no-default-features` product builds are warning-free;
the library-test build retains 24 pre-existing unrelated dead-code warnings.
The delivered default rebuild is byte-identical to the final census binary.
The intermediate continuous/picture runs were deliberately interrupted and
are explicitly labeled diagnostic prefixes; they are not final passes.

The final full-route seeded **CPU picture census** completes **1,473,123
iterations, PASS, unexplained 0**: **1,462,997 exact**, 3,952 late, 563 early,
4,702 torn, 905 polygon, four during ROM lag and zero other content misses.
Every picture hash is byte-identical to main's retained baseline: **zero exact
pictures lost**. Its predictor is byte-identical to both the final seeded ROM
and continuous recorded ROM predictors. `final-pictures-gate.log` and
`verification.json` retain the full results and source/binary identities;
`candidate.patch` retains the delivered diff.

`git diff --check` passes. Evidence occupies approximately **1.34 GB**, below
the 3 GB cap. No GPU, Snes9x process, continuous live replay, full library
suite, protected-cache/shared-ledger write, formatting, commit or push was
performed. This is a verified partial owner-pricing stop: the behavior gaps
and remaining owner partitions listed above remain unresolved.

## Lagless native behavior fixes 1

CPU investigation on `codex/behavior1`, base `d872c800`. Evidence is under
`target/behavior1/`; the authoritative cache and shared `full-v2` ledger are
read-only. The starting timing totals are **2,323 / 0 / 0** ROM and
**2,570 / 243 / 70** seeded live missing/excess/FP.

The native behavior repairs follow the unmodified cartridge:

* Blind's bank-1D recoil entry compares sprite type `$CE` at `$1D:E93D`.
  Its exemption now comes from the explicit ROM entry, in both ordinary and
  lag-free modes. Bank-1E retains its `$7A` exemption. A head entering with
  F=14 therefore continues after recoil and decrements subtype 2 to 1.
* Ancilla slope calls at `$08:8A87` and `$08:8CB5` have no `$10..$13` guard.
  Tiles `$18..$1B` index past the normal slope rows into `$06:E916..E935`;
  those cartridge bytes are unsigned thresholds, not additional geometric
  slopes. The ancilla helper preserves that lookup locally. The sprite
  callers' explicit `$10..$13` guards and `EntitySlope` decoder are unchanged.
* SpikeBlock's statue scan uses X delta + 16 < 32 and **Y delta + 8 < 16**,
  including 16-bit coordinate wrap, as `$1E:BE58..BE72` specifies.

The expanded one-floor fixture also exposed a separate copied exemption:
tile `$03` on the upper floor returned clear before the one-floor body could
classify it. Only class2's `$08:8CA2..8CAB` has that exemption. It is removed
from the one-floor/targeted body. All three Somaria callers enter the same
`$08:8A26` suffix (`$08:E1C7`, `$E401`, `$E793`); a separate 64-case ROM
fixture checks their carry and owned WRAM results. Class2 keeps its own rule.

The source matrices cover **384 Blind head cases** (all 28 body blocks and
both outcomes of ten feasible branches; two `AND #0` branches cannot be
taken), **8,992 one-floor cases** (23 blocks, 13 branch pairs), and
**12,096 statue-scan cases** (ten blocks, six branch pairs). They compare
owned state on every path. All three self-price matrices passed before the
timing trials; the withdrawn patches retain those price witnesses. Final
Blind and statue fixtures assert zero admitted self price; the retained
ancilla fixture compares self work to the ROM and checks zero new self price
outside lag-free mode. The recoil-bank matrix checks **5,120 cases**, including every
sprite type in both modes. The slope matrix checks all 64 fine-coordinate
pairs for `$10..$13` and `$18..$1B` at both ancilla entries: **1,024 cases**.

Branch coverage caught an inherited fixture setup error: a coordinate setter
re-projected ancilla fields after the test wrote its raw input bytes. Moving
coordinate setup before those writes made the previously missed branches
execute and exposed the tile-3 discrepancy. The early runs in
`withdrawn-body-prefix/` were interrupted diagnostics, not acceptance passes.

`recoil-all-callers.json` records all **70 native recoil call sites** and
their ROM call targets. No other reachable Blind-type exemption caller was
found. Bank-5 and bank-6 copies have unconditional double returns; their
ordinary callers are not `$7A`. The unpriced helper's legacy exemption and
the generic stunned path's unproved `$7A` reachability are not changed here.
`Statue_BlockSprites` at `$06:C277` correctly uses X delta + 12 < 24 and
Y delta + 12 < 36; directional push-block/player collision is a separate
algorithm. Those sibling boxes are unchanged. The class2 ancilla slope
sibling is repaired by the same fixture-proven helper.

The first combined pricing candidate added **61 blocks / 225 instructions**,
leaving all **16,050** existing instructions unchanged. It was withdrawn:
seeded live reached **255 excess / 73 FP**, above the allowed 243/70. New
events at **735,936 / 735,946 / 736,422** contributed **1 / 4 / 7** holds;
the changes near 881k only moved existing FP events. Its ROM prefix had zero
excess/FP and its completed CPU picture prefix was unchanged, but none of
these interrupted runs is a full-route pass. `withdrawn-combined/` retains
the binary, source patch, commands, counts and withdrawal reason. Complete
owner trials and a behavior-only control separate pricing from behavior.

Both isolated Blind-head and SpikeBlock self-price trials also reached
**255 / 73** live excess/FP. Each complete owner price is withdrawn; neither
is clipped, discounted or fitted to counts. Their behavior repairs remain
independent of those prices. The retained addition is the one-floor body:
**23 blocks / 87 instructions**, with all 16,050 prior instructions unchanged.
All four isolated seeded-live runs completed 1,473,123 iterations with their
logical gates passing, unexplained state zero, and no P2 disagreements:

| Candidate (behavior repairs in all rows) | Missing | Excess | FP | Pricing decision |
| --- | ---: | ---: | ---: | --- |
| Behavior only | 2,570 | 243 | 70 | Control |
| Blind head only | 2,571 | 255 | 73 | Withdraw whole owner |
| SpikeBlock statue scan only | 2,568 | 255 | 73 | Withdraw whole owner |
| Ancilla one-floor only | 2,517 | 243 | 70 | Retain for final route gates |

The behavior-only control's hold/elapsed rows equal lag5 exactly. Ancilla
pricing changes 57 timing rows: it removes 53 missing holds and moves the
existing 1/4-hold FP events at 881,793/881,803 to 881,794/881,804. Their total
is unchanged; this is not a claim that every individual hold stays in place.

Fresh final runs use the cleaned retained source and each cover 1,473,123
iterations:

| Final run | Missing | Excess | FP | Logical gate |
| --- | ---: | ---: | ---: | --- |
| Seeded recorded ROM | 2,270 | 0 | 0 | PASS, unexplained 0 |
| Seeded live | 2,517 | 243 | 70 | PASS, unexplained 0 |
| Continuous recorded ROM | 2,270 | 0 | 0 | PASS, unexplained 0 |
| Seeded CPU pictures | 2,270 | 0 | 0 | PASS, unexplained 0 |

Continuous ROM reports all 1,473,123 iterations WRAM-clean. Its timing CSV
equals seeded ROM byte for byte; final live equals the isolated ancilla
trial. Every hold equals P2. ROM accepts 100,407 holds and separately reports
5,152 elapsed NMIs; live accepts 100,403 and reports 5,145 elapsed NMIs.
No ROM iteration gains missing holds. All 737 seeded segments are WRAM-clean,
including polygon/song/lag comparisons. The CPU census preserves all
**1,462,997 exact pictures**: 563 early, 3,952 late, 4 during-lag, 905 polygon,
4,702 torn, and **zero content differences**. Its complete picture-hash file
equals lag5 byte for byte; its timing CSV equals the final recorded-ROM CSV.

All **42 focused Rust unit tests** and **seven generator tests** pass. Default
and `--no-default-features` product builds are warning-free; the library-test
build retains its 24 pre-existing warnings. The restored default executable
is byte-identical to the one used for the final runs. Source/binary hashes,
commands, caller audits, gate logs and timing counts are retained locally.

`git diff --check` passes. Evidence occupies approximately **1.8 GiB**, below
the 3 GB cap. No GPU run, Snes9x process, continuous live replay, full library
suite, protected-cache/shared-ledger write, formatting, commit or push was
performed. This is a verified behavior-repair stop with one admitted pricing
owner; the Blind-head and statue-scan prices remain withdrawn in full.

No matcher, classifier, P2 formula, per-scene table, threshold, polygon
schedule or environment switch changed. New self prices are lag-free only.
No exact SPC phase, strict sound identity or continuous-live acceptance is
claimed.

## Lagless missing-hold owners 6

CPU investigation on `codex/lag6`, base `5799319c`. Evidence is under
`target/lag6/`; the authoritative cache and shared `full-v2` ledger remain
read-only. Baseline ROM missing/excess/FP is **2,270 / 0 / 0**, seeded live
**2,517 / 243 / 70**. The intro WRAM clear remains withdrawn.

The missing rows were re-clustered with the existing 120-iteration analysis
gap, module/submodule and room/area keys. There are **684 clusters**. The
three leading room `$003E` visits contain **59 / 58 / 56** missing holds;
Blind's room `$00AC` has 49, the leading `09/00` Tree cluster has 41, and
room `$006B` has 40. The source routine comparisons are retained as signed
exclusive evidence, not additive prices: in particular, DashItem's large
profile label contains child work. No matcher or classifier changed.

The withdrawn Blind-head and SpikeBlock prices were compared against their
complete behavior-only seeded live control. Their first main-work changes
are **732,237** and **329,419** respectively. Blind restores 11 direct holds;
SpikeBlock restores 14. Both also move these existing live events:

| Control iteration | Candidate iteration | Held NMIs moved |
| --- | --- | ---: |
| 735,937 | 735,936 | 1 |
| 735,947 | 735,946 | 4 |
| 736,420 | 736,422 | 7 |

The destinations have zero source holds; the old positions lose their holds.
The first pair moves the same **617,764-clock** main work. The next pairs
move the loader/cutscene work, rather than charging the head/statue body at
the false-positive row. This accounts for the 12 extra holds and three new
FP iterations in both archived trials (**255 / 73**). No missing counterpart
price is established. Both prices remain withdrawn whole.

Fresh 732,000-start live windows reproduce Blind's 12 excess holds, but the
SpikeBlock window has zero excess: its earlier carried SPC history is absent.
The Blind/control APU probes show `$1A` publication at **735,866 / 735,867**,
respectively. The source writes it at 735,868. These are command-publication
observations, not exact SPC-cycle or PCM equality. Current main and the saved
behavior-only control have identical accepted-hold and masked-callback rows
through 737,999. The full saved predictors, identities, changed rows and
paired event counterparts are retained; local windows do not replace the
full seeded-live constraint.

Four complete, independently trialled pricing owners are retained:

* Bomb player-damage caller `$08:9815..9912`, including the shared `$97B5`
  return, excluding sprite damage, hitbox, displacement and projection
  children. **320 ROM fixtures** cover all **22 blocks** and both outcomes
  of all **12 conditional branches**, checking owned player state.
* Bomb distance `$08:9CCE..9D0F`: **100 fixtures** cover both signed absolute
  values, zero/sign/wrap boundaries and slots on either side of the indexed
  read page crossing. The source's second call at `$98E7` is retired by a
  second pure distance query; its result is unchanged.
* Bomb projection wrapper `$04:EB63..EBA9`: **100 fixtures** cover coordinates,
  directions and speeds, compare returned velocities and restored sprite
  coordinates/altitude, and exclude the existing projection child.
* DashItem dispatcher `$05:FBF7..FBFD`, with the inherited `$00:8784..879B`
  tail and separately scoped `$8781` entry. **Six fixtures** cover all three
  valid selectors and slot boundaries. Its ownership ends before the selected
  child; unscoped child markers must not leak into the dispatcher price.

All new prices are lag-free only, use generated ROM blocks and actual branch
outcomes, and have source-address comments. The generator adds **30 blocks /
184 instructions**, leaving all **3,950 existing blocks / 16,137 instructions**
unchanged. No gameplay behavior difference was exposed or repaired. No P2
formula, scene table, fitted threshold, polygon schedule or environment switch changed.

The incremental trials add the caller, distance, projection and dispatcher
one at a time. Each has a saved binary, source patch and seeded ROM window;
short live windows also pass. The caller restores **787,256** in the first
room `$003E` visit. Its retained block profile records the actual phase-rejection
path: `$9815`, `$981A`, taken `$981C`, `$983E`, `$97B5`, totaling **158
source clocks**. The other three prices add source work but restore no
additional holds in their tested windows. The room `$006B` window remains
unchanged; its statue price is still withdrawn. These are complete prices,
not prices selected or scaled to recover a target count.

Both final gates complete **1,473,123 iterations, PASS, unexplained 0**:

| Final lane | Missing | Excess | FP |
| --- | ---: | ---: | ---: |
| Eight-slice seeded recorded ROM | **2,269** | **0** | **0** |
| Single full seeded live SPC | **2,516** | **243** | **70** |

Both predictors change only the hold at **787,256** relative to their main
baselines. No iteration gains missing holds; every runtime hold equals P2.
ROM reports **100,408 accepted holds**, live **100,404**. Live masked callbacks
remain **5,145**. The segmented ROM run reports **5,150 masked callbacks**;
its seed-boundary diagnostics/elapsed callbacks are not claimed identical to
the earlier single-run predictor. The chunked proof is the logical gate and
held/lag rows, not continuous sound timing. The restored default executable
is byte-identical to the frozen census binary. `rom-counts.json`,
`live-counts.json`, `rom-runner.log`, `live-gate.log` and `verification.json`
bind the completed results. The required live CSV is `target/lag6/live.csv`.

The final partition still has **684 clusters**. Room `$003E` becomes
**59 / 57 / 56**; Blind remains 49, Tree 41, and the leading room `$006B`
cluster 40. These residuals are not silently assigned to parent labels or
claimed repaired by the small dispatcher price.

All **12 focused CPU unit tests** and **seven generator tests** pass,
including **526 new executed-ROM fixtures**. Default and
`--no-default-features` product builds are warning-free; the library-test
build retains its pre-existing dead-code warnings. `git diff --check` passes.
Evidence is below the 3 GB cap. No gameplay behavior changed, so the CPU
picture census and continuous recorded run were skipped as requested.
No GPU, Snes9x process, continuous live replay, full library suite,
protected-cache/shared-ledger write, formatting, commit or push was performed.
This is a verified partial pricing stop, not exact SPC phase, strict sound
identity or continuous-live acceptance.

## Lagless live sound timing 1

CPU-only investigation on `codex/live1`, base `5750947e`. **Diagnosis only:
no runtime repair is admitted.** Evidence is under `target/live1/`. The
retained lag6 predictors and binary are SHA-256-bound in `census.json`;
their full logical gates were rechecked, not rerun. Both gates PASS with
unexplained zero. ROM remains **2,269 / 0 / 0**, live **2,516 / 243 / 70**.
The requested improvement constraint is therefore **not achieved**.

Comparing all 1,473,123 corresponding rows isolates **141 changed hold
rows**. Every row is retained in `hold-differences.json`, with its source,
ROM/native holds, signed missing/excess/FP changes, main work and originating
poll. No missing hold is removed. The complete differences are:

| Origin / source module | Changed rows | Added missing | Added excess | Added FP | First changed hold |
| --- | ---: | ---: | ---: | ---: | ---: |
| Room `$0006`, `07/18` | 4 | 5 | 5 | 2 | 462,699 |
| Room `$0006`, `07/00` | 1 | 0 | 8 | 1 | 463,179 |
| Room `$0006`, `0E/02` | 1 | 8 | 0 | 0 | 463,180 |
| Room `$00DE`, `07/18` | 4 | 5 | 5 | 2 | 881,794 |
| `$00DE` aftermath, `0E/02` | 125 | 142 | 135 | 62 | 882,275 |
| Exit aftermath, `16/04` | 3 | 0 | 90 | 3 | 884,319 |
| Exit aftermath, `08/00` | 1 | 58 | 0 | 0 | 884,322 |
| Exit aftermath, `08/01` | 1 | 5 | 0 | 0 | 884,323 |
| Exit aftermath, `08/02` | 1 | 24 | 0 | 0 | 884,324 |
| **Total** | **141** | **247** | **243** | **70** | **462,699** |

The initiating song-end branches are `$08:C606..C610`: native reads zero
at **462,627** and **881,733**, while the source reads **$13**; source zero
arrives at **462,628** and **881,734**. The retained full live log names
these two drift origins. Fresh seeded windows **458,000..463,999** and
**878,000..885,999** reproduce every affected hold row exactly. They also
retain the poll values and command queues. The first episode accounts for
13 missing/13 excess/3 FP; the second accounts for 234/230/67.

This is a causal execution/reseed partition, not 141 independent SPC faults.
The first episode reseeds at 464,000. The second reseeds at 882,000, 884,000
and 884,327; its later text and loader work follows those fresh-engine
boundaries. No independent upload/handshake fault is isolated by this
comparison. The `$1A` queue publications are **462,627 / 881,733**, versus
source port stores **462,629 / 881,735**. They follow the early zero branch;
they are not independently proved causes. Queue observations are not
cycle-level delivered-port witnesses.

The preceding `$13` requests do **not** reproduce the old premature-slot
failure. Fresh `producer-*.log` traces place them at the item-receipt store
in `misc.rs`, after main prefixes **1,317,082 / 1,325,588** clocks. Existing
P2 places them at wall clocks **1,444,034 / 1,452,780**, slot **4**, and
publishes each exactly once on the closing callback after all four holds.
The source ledger likewise contains four lag NMIs at 461,988 / 881,094,
then the `$13` port stores in the following opening handlers at
461,989 / 881,095. The adjacent iteration labels reflect the existing
closing/opening convention; they do not justify adding one callback.

A retrospective poll-window experiment was drafted and withdrawn before
execution. The cache-verified source trace documented in
[Live sound continuous 1](lagless-audio-pcm.md#live-sound-continuous-1)
places the native poll at interval-start +22 SPC cycles and the source at
+2,444 in that older witness. Looking back a callback would introduce an
unsupported delay. `rejected-retrospective-poll.patch` is rejected evidence,
not a candidate. Current inherited driver/timer phase, exact synchronized
port delivery and in-iteration poll timing remain unpartitioned; neither
an opcode/timer defect nor a source-faithful correction is established.

The first continuous-live divergence **inferred from the retained seeded
run is 462,627**, last clean song branch 462,626: there is no actual reseed
before it. No continuous replay was run. A fresh seeded 0..95,999 prefix
also reaches the first song's zero at 94,854, agreeing with the source;
the separate 92,000-start diagnostic ends one iteration early, illustrating
why short-window SPC phase cannot replace the carried prefix.

Intro WRAM clear and Blind/SpikeBlock prices remain **inadmissible**: no
timing repair or new combined-candidate verification establishes otherwise.
No sound behavior remains changed, so strict delivered-command counters
were not rerun and no new sound-proof improvement is claimed. No final
candidate exists, so the one final full live census was not spent and
`target/live1/live.csv` was not fabricated from prior evidence. No fresh
full ROM census is claimed. Default and reduced builds and final evidence
identities are recorded separately under `target/live1/`.

Both final builds are warning-free; the restored default binary is
byte-identical to lag6's census binary (`55aa75f5...`). `git diff --check`
passes. Only this documentation section remains changed; no touched runtime
module requires a new unit test. Evidence is below 3 GB. No GPU, Snes9x
process, continuous live replay, full library suite, protected-cache or
shared-ledger write, formatting, commit or push was performed.

## Lagless live sound timing 2

CPU-only measurement on `codex/live2`, base `3951d3c6`. **No runtime repair
is admitted; the two decisive polls remain unfixed.** All new evidence is in
`target/live2/`. This investigation excludes a sustained sample-clock error
and missing holds *inside these songs*. It does not establish a single
source-faithful correction for the inherited driver and publication phases.

Fresh seeded live runs from **458,000** and **878,000** reproduce native zero
at **462,627 / 881,733**, versus source **462,628 / 881,734**. The fresh
zero-start control agrees at **94,854**. Starts at 460,000 and 880,000 instead
produce zero at 462,626 and 881,732: even these substantial lead-ins do not
reconstruct the carried SPC phase. The earlier starts are therefore the
decisive witnesses, not the shorter diagnostics. Native instruction snapshots
were captured in separate repetitions of those same runs.

### Per-iteration clocks and the host-cut distinction

`clock-{458000,878000,0}.csv` contains every iteration in the requested
461,900..462,700, 881,000..881,800 and 94,000..94,900 windows: actual native
SPC execution deltas (including rebase origin), output-clock deltas, generated
chip pairs, held/masked counts, source lag, source host coordinates and cached
sample counts. Source sample counts multiplied by 32 are **sample-lattice
cycles**, not an invented instruction-level source execution trace. Native
execution deltas differ from the sample budget by small changes in instruction
lookahead; they are reported separately.

Both source cuts are retained. `open_host..close_host` can cover zero or two
host callbacks when an NMI is accepted on the other side of host return.
For a stable main/poll comparison, the second column uses consecutive
`display_host` boundaries (the first visible scanline associated with that
iteration). The cached poll receipts independently locate the three selected
polls at hosts **500,858 / 947,003 / 103,726**, respectively. Treating every
`open_host` as the poll's host would introduce a false whole-field discrepancy.

| Requested window | Native generated pairs | Source pairs between display-host boundaries | Native/source holds | Masked callbacks |
| --- | ---: | ---: | ---: | ---: |
| 461,900..462,700 | 431,829 | 431,828 | 9 / 9 | 0 |
| 881,000..881,800 | 429,697 | 429,696 | 5 / 5 | 0 |
| 94,000..94,900 | 492,605 | 492,604 | 23 / 23 | 0 |

More decisively, all three spans from the `$13` opening iteration to the
selected poll contain **638 ordinary iterations**, **zero source lag NMIs,
zero native holds and zero masked callbacks**. Both native output-clock
advancement and the corresponding cached host interval are exactly
**340,132 chip pairs = 10,884,224 SPC cycles** in every case. There is no
missing hold to name inside these spans. The harness's 534 returned pairs
are resampled output; they do not advance the SPC by 534 pairs per callback.
The existing fractional hardware clock passes its two-million-callback test.

### Delivered commands, poll phase and the actual ending event

The delivery observer and instruction snapshots locate each native `$13`
store and the `$08E5` SPC output-port-zero store. Each native final all-voice
KOFF follows that zero store by **81 cycles**, also checked against the actual
DSP event. `command-phase-brackets.json` retains instructions bracketing the
input store, including timer state and software accumulator `$43`; the three
native accumulator values there are **8 / 176 / 192**. They are observations,
not values to install or reset.

The retained full-route source DSP capture was located under
`zelda3-rs-codex3/target/lagless-audio-notes3/original-full`. All **1,581,079**
of its PCM hash/count rows were freshly compared with the authoritative
cache, and its compressed DSP SHA-256 matches the previously bound capture
(`a1c95628...`). This is verified retained evidence, not a new Snes9x run.
`source-dsp.jsonl` and `source-receipts.jsonl` retain the selected source
events and poll offsets.

The following comparison uses an explicit boundary: native audio cursor at
entry to the `$13` opening iteration, and source cumulative PCM at that
iteration's display host. It does **not** claim equal chip state at that
boundary. KOFF is directly observed on both sides; source output-port-zero
times are not inferred from KOFF for the two new windows.

| Poll | Native KOFF cycles after boundary | Source KOFF cycles after boundary | Native minus source | Native poll execution offset | Source poll sample offset |
| --- | ---: | ---: | ---: | ---: | ---: |
| 462,627 | 10,867,901 | 10,895,395 | −27,494 | +22 cycles | +92 pairs |
| 881,733 | 10,874,550 | 10,890,880 | −16,330 | +19 cycles | +89 pairs |
| 94,854 control | 10,873,821 | 10,886,384 | −12,563 | +22 cycles | +75 pairs |

Native command-to-KOFF durations are **10,884,915 / 10,891,562 / 10,890,835
cycles**. Their `$13` input stores precede the chosen native iteration
boundary by **17,014 / 17,012 / 17,014 cycles**, on the preceding iteration's
closing callback after its four holds. This quantifies the publication phase;
the adjacent closing/opening labels alone still do not authorize adding a
callback. Exact source input-store cycles and driver/timer state at those
stores are not present in the cache or this retained full-route DSP stream.
They remain unmeasured for the two decisive windows. The older exact source
control witness places its store at host-start +47 cycles and its
command-to-KOFF duration at 10,886,337 cycles; current native control duration
is 4,498 cycles longer, despite the agreeing logical poll.

Thus equal elapsed sample budgets coexist with unequal ending-event phase,
including in the agreeing control. A rate change, a missing-hold surcharge,
or a universal one-callback shift is not established as the repair. A complete
partition still needs source input-store/driver-state witnesses at the two
deliveries and a shared source-owned publication/read timeline. No recorded
port answer, retrospective poll override, phase reset, scene table or fitted
offset was tried or retained.

### Admission and verification

The intro-clear patch and the behavior1/lag6 Blind and SpikeBlock rejection
evidence were re-read. There is **no fixed term against which to readmit
them**. Intro clear remains withdrawn (its earlier 380,880 poll regression);
Blind/SpikeBlock remain withdrawn whole (their earlier 12 added excess holds
and three FP iterations). The new seed-phase sensitivity is consistent with
those history-dependent effects, but does not prove their individual causes
or replace a combined-candidate census.

The zero-start 94,901-iteration control passes the unchanged gate with
unexplained zero. Both bounded decisive windows have unexplained zero;
the unchanged gate's full-prefix completeness check rejects their nonzero
starts, so they are not labeled full-route passes. Default, reduced-feature
and restored-default product builds are warning-free. The focused clock test
passes; its library-test build retains existing dead-code warnings.

Only documentation changed. Retained lag6 full-route gates and identities
were rechecked: ROM **2,269 / 0 / 0**, live **2,516 / 243 / 70**, both PASS
with unexplained zero. These are retained results, not fresh censuses. No
final runtime candidate exists, so the single final full-live census was not
spent and `target/live2/live.csv` was not manufactured. No fresh full ROM
census or delivered-command proof is claimed. Sound behavior, prices, P2,
classifiers and matcher are unchanged. The requested fixed-poll acceptance
constraint is **not met**.

`verification.json` binds the measurements and checks. Evidence is below
3 GB; `git diff --check` passes. No GPU, Snes9x process, continuous live
replay, full library suite, protected-cache/shared-ledger write, formatting,
commit or push was performed.

## Lagless live sound timing 3

Diagnosis on `codex/live3`, base `02a6f95a`. **The 94,854 control's
command is one callback early on the aligned logical clock. No runtime
repair is admitted: the two decisive source delivery witnesses are still
missing.** New evidence and the ready-to-run capture plan are in
`target/live3/`. No Snes9x process was run.

### One clock for the control

The retained `zelda3-rs-codex3/target/live-songend1/original/ports.jsonl`
places the source port-0 `$13` store at host **103,088**, V225/H530,
PC `$00:80F6`, `apu_cycle_after=47`. Its 103,741 PCM hash/count rows were
freshly checked against the authoritative cache: all match, covering
55,306,534 pairs. Summing cache counts strictly before host 103,088 gives
54,958,405 pairs, hence source host start **1,758,668,960 SPC cycles**.

The current retained zero-start live2 trace delivers at native absolute
**1,755,822,410** (`baseline-0.log`, host 102,921). The cursor after the
closing callback of 94,215, and therefore at entry to 94,216, is
**1,755,839,424** (`state-0-94215/4-spc.txt`). The outcome row for 94,216
is its *end*, one further callback later; using it as entry would introduce
another false field. Align this entry with source host 103,088 by the
explicit translation **+2,829,536**:

| Store | Aligned SPC coordinate | Cycles from source host 103,088 start |
| --- | ---: | ---: |
| Native `$13` | 1,758,651,946 | −17,014 |
| Source `$13` | 1,758,669,007 | +47 |
| Native minus source | **−17,061** | **−17,061** |

This is a logical-boundary alignment, not an assertion that the two chips
share absolute reset time or state. The raw unaligned difference is
−2,846,597 cycles and includes their earlier history. The aligned difference
is approximately one NTSC callback (17,059.9 cycles). It also reconciles
the prior KOFF result: **−17,061 + 4,498 = −12,563**. Thus the agreeing poll
does not establish correct command phase. `control-alignment.json` records
the arithmetic, cache proof and capture executable/core hashes.

### Publication owner and scope

`rtl_native_pacing.rs::native_time_apu_request` assigns a producer to
`floor(P2 elapsed / (1364*262))`. `native_finish_main_cost` restores entry
latches and starts callback zero. `native_publish_apu_requests`, called
at the beginning of `nmi.rs::interrupt_nmi_audio_parts`, consumes requests
whose slot is at most the current callback. For these item-receipt requests,
four held callbacks precede slot 4, the ordinary closing handler. It samples
the new latch, `zelda_push_apu_state` queues it, and `zelda_render_audio`
pops that batch **before advancing that callback's SPC interval**.
`spc_driver_clock.rs` schedules the `lagless-cycle` bus stores near that
interval's beginning. The observed control store is 42 cycles after the
closing interval starts, but 17,014 before the next iteration's entry.

In the ROM, `$00:80F6 STA $2140` in the closing NMI of the producer's
iteration is also the next iteration's opening-handler store. It follows
the four lag NMIs and the actual `$012C` producer. The APU portion precedes
the `$00:813A` update-latch branch, so held handlers can deliver commands
too. Calling a callback "closing" does not place its store at the proper
end of the audio interval: that ownership mismatch explains the control's
measured early delivery.

The queue/interval arrangement is shared by all four ordinary timed APU
latches, not a `$13` or song-specific branch. A request authored after the
last held NMI and admitted in the closing slot follows this same path.
This does **not** prove every command after any held sequence is one
callback early: earlier slots can publish during holds, multiple stores
collapse with last-write-wins, music can be suppressed as already playing,
and upload-return, timed CPU-access and VWF paths have separate owners.
The analogous ordinary no-hold path also needs its boundary checked; a
blanket delay keyed on "had holds" is not justified.

### Required source capture; stop here

The retained full-route DSP stream proves ending events, and the ledger
identifies source delivery hosts **500,220 / 946,365**. Neither supplies
their exact CPU-to-SPC store clocks and arrival-state instruction brackets.
The exact control cannot fill these missing witnesses. Per the requested
stop rule, no publication change, price trial or final census was run.

Run the following **yourself**, once, from this checkout. It uses the retained
oracle-only instrumented capture executable and core, never the gameplay
frontend. The prepared plan retains CPU APUI receipts and SMP instructions
only at hosts 103083–103090, 500214–500224, 500854–500862,
946359–946369 and 946999–947007. The core runs cold through 947,007 to
preserve phase; there are no intermediate savestates. Every PCM callback
must match the cache, and all new output goes under `target/live3`.

```sh
cd /Users/missingno/Documents/zelda3-rs-codex
C=/Users/missingno/Documents/zelda3-rs/.git/parity-oracle-cache/ed0121e1b093be4c1c69efb6c75057fede3eeda1a88201f9795a20040b307f18
S=/Users/missingno/Documents/zelda3-rs-codex3/target/spc-songlength1
ZELDA3_DEBUG_PCM_WINDOWS="$PWD/target/live3/source-plan.json" \
  "$S/capture-original-zelda3" --render-original-audio-cpu \
  "$C" "$S/core-source/libretro/snes9x_libretro.dylib" \
  "$PWD/saves/zelda3.sfc" 947008 \
  > target/live3/source-capture.log 2>&1
```

The plan pins core SHA-256
`3ea10f7ab7e1b7a12600763f359afefb4c8342bb025a6ee1016740a4c177672a`;
the retained executable is
`8b634ea5b319eb94698e2178484f7bd0b515163534e323785ceaa11d080c192b`.
Both were freshly hashed. The harness verifies ROM, SRAM, initial state
and PCM authority and fails on trace overflow. Its output directory must
be fresh. Inspect `ports.jsonl` for the `$13` stores and the selected polls;
join `instructions.jsonl.zst` using sample/phase and absolute SMP cycles,
retaining timer stages and `$43`. This separates publication displacement
from inherited driver phase before choosing a repair.

Runtime, pricing, P2, classifiers and matcher remain unchanged. Intro clear
and Blind/SpikeBlock remain withdrawn. ROM **2,269/0/0**, live
**2,516/243/70**, seeded-ROM PASS/unexplained 0 and sound command counts
remain prior evidence, not newly executed verification. No touched runtime
module requires a new unit test; builds and final-candidate censuses are
deferred at this explicit source-evidence stop. `git diff --check` passes.
New evidence is below 3 GB. No GPU, continuous replay, library suite,
protected-directory write, formatting, commit or push was performed.
