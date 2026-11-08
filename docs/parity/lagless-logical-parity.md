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

## Lagless live sound timing 4

CPU-only investigation on `codex/live4`, base `8562ef00`. **The new source
capture confirms the early publication in both decisive windows. A general
closing-boundary transport trial still fails poll 462,627 and is withdrawn
whole. No runtime or pricing change is retained.** Evidence is under
`target/live4`; `target/live3` was read only.

### Joined source and native clocks

All **947,008** source PCM hash/count rows were independently compared with
the authoritative cache: **504,870,726 pairs**, all equal. The capture's
`reference_verified=true` agrees with this fresh check. `source-verification.json`
binds the capture files. The selected APUI receipts and instruction snapshots
are joined by absolute SMP cycle, including the instruction before, at, and
after each store; `alignment.json` preserves the raw timer stages and `$43`.

As in timing 3, native entry to the iteration after the producer's closing
callback is translated onto the source delivery host's sample boundary.
This aligns logical boundaries, not chip state or reset history. Native
stores and KOFFs use the retained live2 instruction/DSP witnesses. The new
source DSP stream independently reproduces both decisive KOFF coordinates
(`source-koff.json`). The control KOFF remains the retained verified witness.

| Poll | Source delivery host / raster | Source store after host start | Native publication minus source | Native command-to-KOFF duration minus source | Total KOFF difference |
| --- | --- | ---: | ---: | ---: | ---: |
| 94,854 control | 103,088 / V225 H530 | 47 | -17,061 | +4,498 | -12,563 |
| 462,627 | 500,220 / V225 H530 | 47 | -17,061 | -10,433 | -27,494 |
| 881,733 | 946,365 / V225 H528 | 41 | -17,053 | +723 | -16,330 |

All differences are SPC cycles. Source absolute store clocks are
**1,758,669,007 / 8,533,702,831 / 16,144,893,737**. At their arrival
instruction boundaries, source timer-0 stage1/stage2/stage3 is
**79/15/0, 47/15/0, 41/12/0**, with `$43` **$B0/$E0/$50**.
The retained native arrival brackets have `$43` **$C0/$08/$B0**;
their timer fields are countdown/divider/counter, not the source's
count-up stage representation. They must not be compared as identical fields.
The duration residual contains inherited driver phase; this arithmetic does
not isolate a timer/opcode defect or authorize a phase reset.

`source-polls.json` joins the decisive `$08:C609` reads to instruction
brackets. Source host **500,858** reads `$13` at +2,961 cycles; **500,859**
reads zero at +2,825. Hosts **947,003 / 947,004** read `$13` / zero at
**+2,889 / +2,922**. `control-source-polls.json` preserves the older control
capture's reads, including host 103,726 zero at +2,437; that older recorder's
PC labeling differs from the new capture. These bus timestamps are separate
from the earlier sample-offset receipts.

### Transport trials and rejection

Two bounded, diagnostic changes used the existing cycle-level bus-store
queue for all nominal ports, preserving CPU path spacing and leaving timed
CPU actions and upload-return scheduling on their separate branches. Neither
changed prices, P2, producers, matcher, classifiers, or environment switches.

The first trial mapped V225 relative to a scanline-zero callback origin.
That origin does not match the source's V225 callback cut and is rejected
(`rejected-raster-origin.patch`). The second assigned the queued closing or
held batch to the output interval's closing boundary, with the existing
handler store offsets (`boundary-candidate.patch`). Neither is an admitted
complete owner repair; opening/no-hold, VWF and upload-return acceptance
would still require the wider checks if the decisive polls passed.

Both trials were run from **458,000**, **878,000**, and **zero**, through
462,700, 881,800, and 94,900 respectively. All six runs exit zero. Both retain
zero at **94,854**, fix the second branch to zero at **881,734**, but still
return zero at **462,627**, one iteration before the source. The closing-boundary
control passes the unchanged gate, **unexplained 0**. The bounded nonzero
starts are not full-route gate passes.

A repeated closing-boundary 458,000-start trace observes `$13` at native
**69,007,317**, just **+21** after entry to iteration 461,989, versus source
**+47**: publication error is reduced to **-26 cycles**. Yet its KOFF at
**79,889,865** remains **12,826 cycles early**; the command-to-KOFF residual
is **-12,800**. This rejects the claim that correcting the whole-callback
publication displacement alone fixes this poll. The residual changes with
earlier transport history; it is not a fixed per-song correction.
See `boundary-failure.json` and `boundary-state-*`.

Both patches are withdrawn whole. The first continuous-live song-branch
divergence **implied by the seeded evidence remains 462,627** (last clean
462,626), as in the retained baseline. No continuous live replay was run.

### Verified stopping boundary

The fixed-two-poll requirement is **not met**. Intro WRAM clear and the
Blind/SpikeBlock prices remain withdrawn; no combined candidate was tested
or admitted. The single final full live census was not spent on a rejected
candidate, and no `live.csv` was manufactured. No fresh full ROM census,
delivered-command comparison, waveform/chip-stream proof, or picture census
is claimed. ROM **2,269/0/0**, live **2,516/243/70**, and sound-proof counters
remain prior evidence for unchanged runtime, not new results of these trials.

Restored-code focused clock tests and product-build results are retained in
`test-clock.log`, `build-reduced.log`, `build-final.log`, and `checks.json`.
All **42 focused tests pass**; default and reduced-feature product builds
are warning-free. The final binary is byte-identical to the retained lag6
baseline (`55aa75f5...`). `verification.json` binds the trial artifacts and
restoration. Evidence is approximately **134 MiB**; `git diff --check` passes.
No GPU, Snes9x process, continuous live replay, full library suite, protected
cache/shared-ledger write, formatting, commit or push was performed.

## Lagless live sound timing 5

CPU-only continuation on `codex/live5`, base `fb0b2c0f`. **The complete
ordinary queued-boundary candidate fails the revised full-live limits and
is withdrawn whole. No runtime or pricing change is retained.** Dropping
the fixed-two-poll requirement does not admit this candidate: its full
carried history does not reproduce the successful second bounded window.
Evidence is under `target/live5/`; cache and `full-v2` remained read-only.

### Complete queue owner and focused verification

The candidate schedules every ordinary queued NMI's nominal port stores
after its output interval's closing boundary. This covers the no-hold
closing/next-opening boundary, each held batch, and the final closing batch,
without a hold-count condition or an added callback. ROM handler offsets
and CPU-to-SPC fractional conversion remain unchanged. VWF's retained-value
selection stays in `frame_writes`; upload-return deadlines and handled
timed CPU accesses retain their existing branches. No producer, price,
P2 rule, classifier, matcher, environment switch or driver phase changed.

Tests exercise actual bus-queue publication before/after callback closure
with zero, one and four holds, music single delivery, VWF/timed-access
ownership, and all four latches' last-write-wins across held/closing slots.
Three prior assertions were updated to inspect the handler's stores after
the closing boundary as well as the unchanged latches before it. All
**45 clock tests and 18 pacing tests pass**. The complete withdrawn source
and tests are preserved in `rejected-candidate.patch`.

Fresh live windows from 458,000 and 878,000 reproduce timing 4: 462,627 is
early, while 881,734 agrees. The zero-start 94,901-iteration control has no
song-poll mismatch and passes the unchanged gate with unexplained zero.
`window-results.json` binds these bounded results; they do not establish
the full route's carried SPC phase.

### Full-route admission: rejected

The supplied eight-chunk ROM census and **one single full seeded live
census** each cover **1,473,123 iterations**. The live CSV is `live.csv`.
Both logical gates PASS with unexplained zero; every runtime hold equals
P2. The separate numerical admission check fails:

| Lane | Missing | Excess | FP |
| --- | ---: | ---: | ---: |
| ROM baseline and candidate | 2,269 | 0 | 0 |
| Live admission ceiling | 2,516 | 243 | 70 |
| Full seeded live candidate | **2,660** | **386** | **84** |
| Live regression | **+144** | **+143** | **+14** |

The ROM candidate changes **zero accepted-hold rows** from lag6. The live
poll origins and their subsequent hold differences against that ROM lane
are retained in `live-origin-hold-differences.json`:

| Poll origin | Native / source | Added missing | Added excess | Added FP |
| --- | --- | ---: | ---: | ---: |
| 229,035 | `$01` / zero | 9 | 9 | 2 |
| 462,627 | zero / `$13` | 13 | 13 | 3 |
| 627,256 | `$01` / zero | 27 | 27 | 5 |
| 735,867 | `$01` / zero | 12 | 12 | 3 |
| 881,733 | zero / `$13` | 234 | 230 | 67 |
| 1,064,734 | `$01` / zero | 96 | 95 | 4 |

Thus both original episodes survive the full run, and four additional
episodes account for the regression. The 878,000-start success is sensitive
to its shorter SPC history; it cannot be substituted for the full seeded
result. These are observed episode groups, not new runtime classifications
or a proof isolating each timer/driver-state cause.

The candidate's first continuous-live divergence **implied by seeded polls
is 229,035**, last clean song branch 229,034. No actual reseed precedes it;
the first post-boot reseed is 464,000. No continuous live replay was run.
Live gameplay paths do change at these polls. A ROM-lane picture census
would not validate those live paths; no picture census was run.

### Delivered sound proof

A separate full seeded **ROM-answer** capture completes all 1,473,123
iterations with zero actual reseeds, exit zero, both compressor exits zero,
and logical gate PASS/unexplained zero. It uses the sound-proof delivery
observer and the unchanged `scripts/lagless_audio_timing.py` matcher
(SHA-256 `7734b9387c35c2d2ddf94a56ec4b9ceef64959d23a3304a59525380232c2052d`).
The scratch adapter follows `/tmp/sound2-delivered.py` with only its output
directory changed. Source commands are the retained proof-1 inventory.

| Delivered transitions | Sound proof 1 | Sound fix 1 | Candidate |
| --- | ---: | ---: | ---: |
| Matched | 67,510 | 67,598 | **67,598** |
| Native-only | 644 | 13 | **13** |
| Source-only | 133 | 45 | **45** |
| Same-iteration order-only rows | 39 | 39 | **39** |

The candidate has 67,611 native and 67,643 source normalized commands.
Its identity counters are no worse. `sound/delivered-identity.json` retains
all classes and **116 raw observer timestamp inversions**: scheduled stores
are logged before later-emitted receipts for earlier stores. The trace was
not sorted or the matcher altered to hide these. Matching these counters
does not establish equal delivery phase, chip state or waveform.

The unchunked sound run has **100,408 holds, 5,147 masked callbacks and
1,578,678 total callbacks**. Its fractional clock implies **841,627,976
generated chip pairs**, versus the retained route-relative source's
842,864,820: **−1,236,844 pairs**. Returned harness PCM is 843,014,052 pairs
(+149,232); it is a different sample grid. `sound/sample-counts.json`
records this length accounting only. Waveform and chip-content equality
were not recaptured. The chunked ROM census has 5,153 masked callbacks;
its independently seeded chunks are not the sound stream's phase history.

### Prefix accounting and restored stop

Before iteration **462,627**, the ROM lane lacks **486 holds across 469
iterations**, with zero excess. At the average NTSC field duration this
missing-hold term is
`486 * (1364*262 - 2) * 15664 / 328125 = 8,291,113.38 SPC cycles`.
It is an approximately **8.291-million-cycle deficit contribution**, not
a measured net phase difference or a value to install in `$43`: masked
uploads, callback cuts and inherited driver state remain separate terms.
`rom-prefix-phase.json` and the empty `rom-held-changes.json` establish that
the same count applies after restoring the baseline. No price was fitted
to this number.

Because transport admission failed, intro WRAM clear and Blind/SpikeBlock
pricing were **not retried or retained**. Their retry was conditional on
an admitted transport baseline. The original runtime is restored; the final
default binary is byte-identical to lag6/timing-4's retained baseline,
SHA-256 `55aa75f553f79101be1986442c940b72b7274773d44939b94bf3ce33abf8cfa1`.
The restored baseline's earlier 462,627 frontier remains retained evidence,
not a second freshly run full-live census.

Candidate and restored default/reduced-feature product builds are
warning-free. Restored **42 clock and 17 pacing tests pass**; library-test
compilation retains the existing unrelated dead-code warnings. Only this
documentation is changed. `verification.json` binds the candidate, proofs,
tests and restoration; evidence remains below 3 GB and `git diff --check`
passes. No GPU, Snes9x process, continuous live replay, full library suite,
protected-cache/shared-ledger write, formatting, commit or push occurred.

## Lagless missing-hold owners 7

CPU investigation on `codex/lag7`, base `ab09182c`. Evidence is under
`target/lag7/`. **The four-owner candidate is withdrawn whole: its full ROM
census introduces seven excess holds and seven false-positive iterations.**
No runtime, pricing, generated-table or fixture change is retained. The
cache and shared `full-v2` ledger were read-only.

The baseline prefix contains **486 missing holds in 469 iterations**, grouped
earliest-first into **202 clusters** using the existing 120-iteration gap and
module/submodule/room/area keys. `prefix-clusters-before.json` preserves every
membership, the ownership leads and explicit unresolved cases. Signed source
routine comparisons cover 230 ordinary-gameplay missing iterations in 56
clusters; their native comparison is the retained historical lag1 baseline,
not a claim that all those historical gaps remain unpriced on current main.
Later admitted prices and unresolved call/scalar aliases are not erased.
No matcher or classifier changed.

The first clusters and source owners are:

| Iterations | Baseline missing | Owner / remaining proof boundary |
| --- | ---: | --- |
| 0–7 | 8 | Whole untimed `Intro_Clear1kbBlocksOfWRAM`, `$0C:C1A0..C1F4` |
| 617, 672–688 | 6 | Whole untimed sword/flash `$0C:FE56`, its setup `$FE45`, and backdrop `$0E:D60B`; intro animation/dispatcher self work remains separately unresolved |
| 729 | 9 | Loader/decompression partition: `$00:E766`, `$00:E843`, `$02:FEBB/$FF5F`; the `$00:E6B7` positive and `$00:E583` negative alias balance cancels exactly |
| 12,996; 16,237 | 1 each | Dungeon-map parent/model undercharge lead; no complete outside-child partition established |
| 15,313; 31,095; 34,318 | 1 each | Room transition, spiral stairs and overworld loader partitions; no additive parent difference admitted |

The early source profile identifies `Scene_AnimateEverySprite` `$0C:C412`,
`Intro_RunStep` `$C448`, `Intro_HandleAllTriforceAnimations` `$C404`, and
`Intro_AnimateTriforce` `$C435` as remaining self-work leads at 684/688.
Their aggregate parent differences are not prices. The sheet-7 input extent
remains the separately proved 1,705-byte source stream versus a 1,536-byte
native window; the complete loader instruction partition is still open.
Later ordinary-gameplay, loader and interrupt/raster-phase clusters remain
in the ordered artifact, including cases without a unique routine attribution.

Four complete bodies were trialled together, with generated ROM block markers
and source-address comments: the WRAM clear, sword setup, all three periodic
sword/flash phases, and the shared backdrop helper. The clear retires its
actual word-column loop and branch; the sword body prices clipping, flash,
landing and both sparkle phases, excluding the independently owned backdrop.
The setup ends at its fallthrough into the periodic body. **739 executed-ROM
fixtures** check instruction work and owned WRAM/OAM results, including all
conditional paths, all eight clear ranges and short loop boundaries. No
native behavior difference was exposed. The generator adds **45 blocks /
199 instructions**, preserving all **3,980 blocks / 16,321 instructions**
already present. No price is fitted to holds, and no scene exception, P2
change, environment switch or sound-transport change was introduced.

The seeded 0–2,000 trial passes the unchanged gate, unexplained zero, with
missing/excess/FP **23/0/0 → 11/0/0**. It restores iterations 0–7, 617,
672, 676 and 680; 684/688 and the loader gap remain. The full eight-slice
seeded ROM census completes **1,473,123 iterations**, logical gate **PASS,
unexplained 0**, but timing admission **FAILS: 2,248 / 7 / 7**. Every runtime
hold equals P2. All seven excess holds are on source-unheld later-intro
iterations **146,782, 146,784, 146,788, 146,792, 146,796, 146,800, 146,804**.
Their opening-handler work changes **9,344 → 26,234** clocks and lag-handler
work **3,210 → 20,140**, alongside the new sword instruction work. A fresh
146,000-start ROM window has zero excess: the full-route failure depends on
inherited execution/worker phase. That diagnostic does not waive the full
ROM gate or justify selectively suppressing a valid body price.

The supplied 71-hold first-window figure belongs to the previously withdrawn
intro-clear result. Both retained lag6 and live5 restored ROM CSVs instead
establish the following current-main baseline. The rejected trial is shown
separately from the final restored result:

| Before song-end window | Baseline missing | Rejected trial missing | Final restored missing |
| --- | ---: | ---: | ---: |
| 94,334–94,854 | 79 | 67 | 79 |
| 141,568–142,088 | 115 | 103 | 115 |
| 228,499–229,035 | 253 | 232 | 253 |
| 380,243–380,881 | 407 | 386 | 407 |
| 461,989–462,628 | 486 | 465 | 486 |

The rejected trial also has seven cumulative excess/FP at each of the last
three windows. Missing-hold reductions alone therefore do not qualify it.
`candidate-prefix.json`, `rom-held-changes.json`, `rom-excess-discovered.json`
and `rom-runner.log` retain these results.

The single full seeded-live census completes **1,473,123 iterations**, logical
gate **PASS, unexplained 0**, with **2,503 missing / 258 excess / 79 FP**.
It therefore also fails the live **243/70** limits, by **15 excess holds /
nine FP iterations**. Every runtime hold equals P2. This is the frozen rejected
candidate's result, retained at `target/lag7/live.csv`, not a result of the
restored baseline. `candidate-live-counts.json`, `live-held-changes.json` and
`live-gate.log` bind it. No continuous live replay or second live census ran.

The restored default binary is byte-identical to the retained main baseline,
SHA-256 `55aa75f553f79101be1986442c940b72b7274773d44939b94bf3ce33abf8cfa1`.
Its fresh seeded 0–2,000 ROM check is clean. The retained baseline full-route
results remain ROM **2,269/0/0**, live **2,516/243/70**; these are reused,
hash-bound full results, not new full censuses of the restored binary.
Candidate verification passes **22 focused Rust tests and seven generator
tests**. No behavior difference was exposed, so no CPU picture census was
needed; the retained 1,462,997 exact-picture result is not claimed rerun.
`rejected-candidate.patch` preserves the complete rejected implementation and
fixtures. This is a verified withdrawal, not a missing-hold improvement or
sound-phase proof. The next pricing attempt must account for the later intro's
inherited worker/NMI phase without fitting prices or weakening admission.

Candidate and restored default/reduced-feature builds are warning-free;
library-test compilation retains the existing unrelated dead-code warnings.
The active default binary is restored too. `verification.json` and
`artifact-hashes.json` bind the source patch, binaries, reused evidence and
completed results. Evidence occupies less than **0.5 GiB** and
`git diff --check` passes. Only this documentation is changed. No GPU,
Snes9x process, full library suite, protected-cache/shared-ledger write,
formatting, commit or push occurred.

## Lagless missing-hold owners 8

CPU-only continuation on `codex/lag8`, base `aad8c3f7`. Evidence is under
`target/lag8/`; the oracle cache and shared `full-v2` ledger remained read-only.
**The carried owner is identified and the ROM repair is verified, but the
combined candidate is withdrawn whole: live excess/FP exceed 243/70. No
runtime, pricing, diagnostic, fixture or generated-table change is retained.**

### Carried owner

The carried field is `DisplayState::crystal_rotation_counter`, the legacy
frontend accumulator at `$0649`. It is not a missing cartridge reset.
`zelda_run_frame` in `rtl_diagnostics.rs` still advanced that accumulator
while `lag_free_intro_schedule` independently owned the cold worker's
before-main and pre-NMI work. At the boot exit (iteration 729), native
`$0649` was **144**, versus cartridge **0**. A foreign seed restores zero.

On the later intro's submodule-6 entry, the legacy scheduler adds the V-IRQ
line, 144. The continuous run carries (144 + 144); the seeded run does not
(0 + 144). Their worker state first differs at **146,739**: the continuous
run completes `poly_job_in_flight` and leaves `$1F0C=$FF`, whereas the seeded
run retains the job and `$1F0C=0`. Main disables `$012A` during the sword
descent. When it resumes, the continuous run uploads the pending bitmap in
the closing NMI of **146,781**. The cartridge's pre-NMI image has no pending
upload there. The otherwise identical main work at 146,782 therefore enters
through **26,234**, rather than **9,344**, clocks of opening-handler budget.
The difference is **16,410 DMA clocks** (2,048 bytes plus setup), increased
to **16,890** by the existing refresh accounting.

The candidate bypasses legacy dispatch while the existing cold logical
schedule owns the worker. It neither resets `$0649` at a scene boundary nor
changes the schedule, P2, prices, classifiers or matcher. The cartridge's
worker selection is the `$012A/$1F0A` stack exchange at `$00:82C9..82CF` and
`$00:8324..832A`; the legacy accumulator must not run alongside its native
owner. The lag7 four-owner pricing patch is reapplied unchanged.

`carried-owner.json`, `wram-diff.json`, `source-poly-boundaries.json`, and
the paired `trace-continuous/` / `trace-seed/` snapshots retain the evidence.
The runtime observer includes native pacing, APU request slots, worker
continuations and job state. An initial probe omitted `--seed-every` and
therefore did not actually seed; `invalid-seed-probe.json` explicitly excludes
it. The corrected seeded probes and both continuous reproductions agree.

### Earliest remaining work

The repaired **0..148,000 continuous prefix** passes the logical gate with
**121 missing / 0 excess / 0 FP**, every hold equal to P2. The eight-slice
full seeded ROM census completes **1,473,123 iterations**, gate PASS,
unexplained zero, **2,256 / 0 / 0**, versus baseline **2,269 / 0 / 0**.
Its five prefix deficits are **67 / 103 / 240 / 394 / 473**, before windows
starting 94,334 / 141,568 / 228,499 / 380,243 / 461,989 respectively.
These counts are missing-hold contributions, not measured net SPC phase.

Fresh early profiles leave **nine holds at 729** and **one each at 684/688**.
The sheet-7 extent lead in the previous worklist is stale: current code
already reconstructs the contiguous raw-art stream, and the 222-sheet ROM
extent test passes, including the 1,705-byte sheet-7 stream. A concrete
remaining omission is `load_file_select_graphics`: its direct decodes of
sheets `$5E/$5F/$6B` bypass decompression pricing. The source has six
`$00:E766` calls at 729; native has three. The signed `$E583/$E6B7` balances
still cancel by 366,550 clocks and must not be added as missing work.
`decompress_enemy_damage_subclasses` and caller self-work remain separate.

At 684/688, current native main is 3,232 clocks below the source profile.
The complete wrapper self-work leads are `$0C:C404` (282), `$C412` (804),
and `$C435` (232); `$C448` also contains a computed dispatcher with an inline
jump table. Linear decoding crosses that data and is not a valid price.
`earliest-owner-comparison.json` and `earliest-followup.json` preserve the
signed comparisons and remaining proof boundaries. No follow-up price was
added from aggregate differences.

### Live rejection and restored stop

The single full seeded-live census first exceeds admission by iteration
900,729: **251 excess holds / 72 FP**, eight holds and two FP above the
limits. The first new poll mismatch is **94,853**, native zero versus source
`$13`; it shifts work to source-unheld 94,854 and 94,857 (five and three
holds). It precedes the replayed intro and is independent of the `$0649`
carry. A logical gate pass does not waive these numerical limits or prove
SPC phase, command delivery, chip state or waveform equality.

The candidate source and tests are preserved in `rejected-candidate.patch`;
`owner-fix.patch` isolates the cold-scheduler correction for a future trial.
The CPU picture run was stopped after **584,876 predictor rows** once live
rejection was definitive. It is a partial diagnostic, not a full picture
census or a new 1,462,997-exact receipt. The final tree restores all original
runtime behavior, so the earlier exact-picture count remains prior evidence.

Final restored prefix deficits remain **79 / 115 / 253 / 407 / 486**.
The candidate's **67 / 103 / 240 / 394 / 473** are rejected-trial results.
Further pricing is deferred at this verified withdrawal; the next loader
and sword owners above are concrete follow-up work, not admitted changes.

The full live run completed all **1,473,123 iterations**: **2,511 missing /
251 excess / 72 FP**, with zero P2 disagreements. Its logical census gate
passes (zero unexplained differences), but admission fails. No second live
run was made.

After withdrawal, default and `--no-default-features` product builds are
warning-free; all **20 restored focused tests** pass (existing lib-test
dead-code warnings remain). The restored 0..2,000 check passes with
**23 / 0 / 0**, and its predictor CSV is byte-identical to lag7's restored
check. The rebuilt executable SHA-256 is
`55aa75f553f79101be1986442c940b72b7274773d44939b94bf3ce33abf8cfa1`,
identical to the retained baseline. Candidate validation had passed 25
focused tests and seven generator tests. `verification.json` and
`artifact-hashes.json` record the retained evidence. Only this documentation
is changed; `git diff --check` passes. Evidence occupies about 604 MiB.
No GPU run, commit, push or formatting pass was performed.

## Lagless missing-hold owners 9

CPU-only continuation on `codex/lag9`, base `18506713`. Evidence is under
`target/lag9/`; the oracle cache and shared `full-v2` ledger were read-only.

### Revised admission rule

The user's revised rule replaces the historical fixed live excess/FP limits.
ROM admission requires fewer missing holds, zero excess/FP, and a complete
chunked seeded census passing with unexplained zero. For live admission,
every changed live-only hold row versus the baseline must be attributed to
an observed song-end poll episode and its execution/reseed-bounded aftermath,
using the live1 hold-difference partition. A non-poll hold change rejects the
candidate. Live totals and episode contributions remain reported; this rule
does not establish source-equal SPC phase, delivery times or waveform.
Any new ROM excess or FP withdraws the entire trial batch.

### Re-admitted repair and prices

The retained implementation is exactly lag8's `$0649` owner repair plus the
lag7 four-owner prices and their ROM fixtures. Cold logical worker scheduling
bypasses the independent legacy rotation accumulator; it introduces no reset,
recorded port answer or scene threshold. The rebuilt executable is byte-identical
to the retained lag8 candidate, SHA-256
`53244f39b876059bafa5d3d1710c1472b1e5cf92acb87f429cd212f6b9e5f824`.
The earlier lag8 ROM/live logs were freshly re-gated and their identities bound
before beginning new runs. `readmission-partition.json` includes baseline and
candidate values for every affected row, rather than just net totals.

Fresh full seeded ROM and the **one single full seeded live census** each
complete **1,473,123 iterations**, gate **PASS**, unexplained zero, and zero
P2 disagreements. Both fresh CSVs are byte-identical to their retained lag8
counterparts. ROM is **2,256 / 0 / 0**, versus baseline **2,269 / 0 / 0**;
live is **2,511 / 251 / 72**, versus baseline **2,516 / 243 / 70**.
`live-partition.json` records the complete fresh live1-style partition:

| Origin poll | Changed live-versus-ROM hold rows | Added missing | Added excess | Added FP |
| --- | ---: | ---: | ---: | ---: |
| 94,853 (new) | 6 | 8 | 8 | 2 |
| 462,627 (inherited) | 6 | 13 | 13 | 3 |
| 881,733 (inherited) | 135 | 234 | 230 | 67 |
| **Total** | **147** | **255** | **251** | **72** |

Exactly **six** live-only hold rows change relative to baseline; none is
unattributed and no inherited episode is removed. The fixed 243/70 ceiling
is deliberately not applied under the revised rule.

| Before window starting | Baseline ROM missing | Retained ROM missing | Retained live missing |
| --- | ---: | ---: | ---: |
| 94,334 | 79 | 67 | 67 |
| 141,568 | 115 | 103 | 111 |
| 228,499 | 253 | 240 | 248 |
| 380,243 | 407 | 394 | 402 |
| 461,989 | 486 | 473 | 481 |

These are cumulative missing-hold contributions, not measured net SPC phase.

The fresh first ROM chunk has no foreign reseed after iteration zero. Its
0–148,000 carried prefix has **121 missing / 0 excess / 0 FP**, directly
covering the later-intro failure that originally prevented admission.

The first additional song branch is `$08:C606..C610` at **94,853**: native zero
versus source `$13`, one iteration before source zero. Its six changed hold
rows are **94,854, 94,855, 94,857, 94,858, 94,859 and 94,860**. The baseline's
462,627 and 881,733 episodes retain their hold differences. The first new
main-work difference is the poll itself. Main-work changes from this episode
end at 94,887; handler-budget differences are separately retained, including
later differences before the first actual live reseed at 464,000. They are
not silently relabeled as extra missing or excess holds.

The two inherited episodes retain live1's boundaries: the first reseeds at
464,000; the second includes reseeds at 882,000, 884,000 and 884,327, with its
last changed hold at 884,324. No sound transport, P2, classifier or matcher
changed. The live acceptance here concerns the requested hold partition;
strict delivered-command, chip-state and waveform proofs were not recaptured.
The sound-timing 1–5 publication and inherited-phase limitations remain open.

### Earliest-first follow-up: entire batch withdrawn

Six additional owners were implemented and tested without fitted prices:

* File select's three `$00:E766` entries for sheets `$5E/$5F/$6B`: generated
  destination-setup markers plus the existing instruction-derived decoder
  and its `$E843` child accounting. All three complete ROM calls and decoded
  outputs match; the existing decoder tests cover every cartridge sheet.
* Enemy-damage caller `$02:FE71..FEBA`: setup, all 2,048 nibble-expansion
  passes, actual taken branches and return. Its `$02:FEBB/$FF5F` decoder
  child remains separately unpriced, not charged from the parent delta.
* Intro wrappers `$0C:C404`, `$C412` and `$C435`: bank/call work, all eight
  actor calls, and both worker-GO branches, excluding the existing children.
* Intro dispatcher `$0C:C448`: the six bounded code arms exclude the inline
  jump table. `$00:8781..8783` owns its child prefix; its popped-frame tail
  `$8784/$8799` belongs to the caller. Fixtures cover all six arms with every
  timer/configuration byte and angle branch boundaries.

The additional fixtures comprise **1,567 executed-ROM cases**, alongside the
739 restored lag7 cases. The generator audit preserves all **16,321** baseline
instruction prices; the combined trial adds 369 instructions (199 restored
lag7 instructions and 170 follow-up instructions). Source listings and the
complete rejected implementation/tests remain under `source-prices/` and
`rejected-owner-candidate.patch`.

The 0–2,000 trial improves missing holds **23 → 2**, with no excess/FP:
684 is restored; 688 retains one missing hold, and 729 falls from nine to one.
The source/native main gap at 684/688 is reduced to **1,364 clocks**; the
remaining main-routing/dispatch ownership is not priced from that difference.
At 729 the remaining decoder child accounts for **383,678 + 146,870** source
clocks, while the signed `$E583/$E6B7` alias balance remains intact.

The carried 0–148,000 prefix passes with **104 / 0 / 0**, unexplained zero.
Focused boot and later-intro CPU checks reproduce all **4,000** repair-only
picture hashes. These are trial diagnostics, not final retained improvements.

The full eight-slice ROM trial passes the logical gate but **fails timing
admission: 2,225 missing / 2 excess / 0 FP**. Runtime holds equal P2 everywhere.
At **696,237** and **898,078**, both `01/00` file-select loads, source has 60
holds; native rises **53 → 61**. Main work rises by **2,670,232** clocks in
both cases. Opening/lag-handler budgets remain **26,164 / 20,070** in both
baseline and trial; they did not change with the prices. The complete
caller/decoder fixtures do not waive this full-route failure. The entire
follow-up batch was withdrawn, without selectively suppressing a caller,
fitting a price or changing P2. No full-live census was spent on this rejected
batch; the one fresh full-live census belongs to the retained candidate.

The rejected prefix deficits are **58 / 94 / 223 / 377 / 456**, distinct from
the retained deficits above. The later ordered worklist was also inspected:
`early-cluster-worklist.json` covers **42 clusters** before 141,568 (26 before
94,334), with fresh native profiles and signed source comparisons for **159
iterations**. Collision, sprite-dispatch and loader/model aliases remain
explicitly unresolved. No aggregate routine delta was turned into a price.

The required **single full CPU picture census** of the retained repair
completes **1,473,123 iterations**, gate PASS/unexplained zero, preserving
**1,462,997 exact pictures**. Every one of its 1,473,123 RGB hashes is identical
to the prior `target/behavior1/pictures-hashes.txt` reference; **no exact picture
is lost**. The full picture, live and fresh chunked ROM runs all use the same
hash-bound retained executable. The rejected batch's two bounded picture
checks are kept separately and do not substitute for this full census.

All **23 retained focused Rust tests and seven generator tests pass**.
Default and `--no-default-features` product builds are warning-free; the
library-test build retains existing unrelated dead-code warnings. The active
product executable is restored to the retained default binary. The gate,
sound matcher and price-generator source files are unchanged from HEAD.
`verification.json`, `artifact-hashes.json`, the retained/rejected patches,
and the complete CSVs bind the results. Evidence is approximately **1.0 GiB**,
below the 3 GB limit; `git diff --check` passes.

This is a verified partial repair and revised-rule admission, with **2,256
ROM missing holds still open**, not full sound or exact A/V acceptance. No GPU,
Snes9x process, continuous live replay, full library suite, protected-cache or
shared-ledger write, formatting pass, commit or push was performed.

## Lagless missing-hold owners 10

CPU-only continuation on `codex/lag10`, base `66f745f8`. Evidence is under
`target/lag10/`; the oracle cache and shared `full-v2` ledger remain read-only.
Admission uses owners 9's revised rule: ROM excess/FP must remain zero, and
every changed live-only hold row must have an observed poll episode and a
bounded execution/reseed explanation. No classifier or matcher is changed.

### Later file-select overcharge: retained handler, not main prices

The ordered source profile at **696,237** accounts for every clock in the
rejected candidate's **+2,670,232** main-work increase:

| Executed source owner | Added native clocks | Executed source clocks |
| --- | ---: | ---: |
| `$00:E500` call to `$E766`, sheet `$5E` | 555,206 | 555,206 |
| `$00:E50C` call to `$E766`, sheet `$5F` | 583,258 | 583,258 |
| `$00:E53E` call to `$E766`, sheet `$6B` | 839,018 | 839,018 |
| Enemy-damage caller `$02:FE71..FEBA` | 692,750 | 692,750 |
| Intro wrappers/dispatcher on this visit | 0 | 0 |
| **Total** | **2,670,232** | **2,670,232** |

Native main rises **17,302,916 → 19,973,148**, still below source
**20,507,004**. The remaining 533,856-clock deficit includes the separate
bank-02 decoder children (**383,678 + 146,870**). The `$E583/$E6B7` signed
aliases cancel (**−366,550 / +366,550**); neither is another price. All three
file-select decodes execute on this later visit. No correct price is removed.
`overcharge-investigation.json`, `source-profile.json` and the extracted
`source-696237.bin` retain this breakdown. The slice conserves **688,833
instructions / 20,507,004 clocks** and matches the retained source sequence
digest; it was read from the existing ordered matcher evidence.

The overcharge is the preceding threaded NMI's **polygon-upload tail**, which
the predictor continued repeating after main cleared `$012A`. Source's 60 lag
handlers total **186,836 wall clocks**, with **zero DMA**; the preceding entry
really does contain **20,788 DMA clocks**, and its worker slice is real too.
The repair preserves that opening handler and completed prefix. It retains
the measured threaded-tail boundary, checkpoints the current segment before
deactivation, then selects the ordinary `$00:8144..822C` register tail for
future lag handlers. Each segment retains its own handler budget under the
existing P2 rule. There is no source-count lookup, phase reset or new timing
threshold.

At both **696,237** and **898,078**, the opening native budget remains
**26,164**. The repeated handler changes **20,070 → 3,040 wall clocks**:
the retained 976-clock head plus the ordinary 1,984-clock tail replaces the
2,084-clock threaded tail and 16,410-clock polygon DMA. These values are
observations of the generic ownership repair, not installed scene prices.
Main prices remain unchanged; native holds become **58**, versus source
**60**, rather than the rejected 61. Both 4,000-iteration seeded windows are
logically clean with zero excess/FP.

One regression fixture executes a threaded NMI with a real pending polygon
upload, advances main before deactivation, and proves that the completed
prefix and opening handler are unchanged. A second checks the shared ordinary
tail against unmodified ROM execution for **12 mode/IRQ combinations**. The
ROM CPU fixture runs in-process; no Snes9x process is launched.

The corrected six-owner checkpoint passes the full eight-slice ROM census:
**2,229 missing / 0 excess / 0 FP**, unexplained zero, and no P2 disagreement.
Its prefix deficits are **58 / 94 / 223 / 377 / 456**. The continuous
0–148,000 prefix also passes (**104 / 0 / 0**). This checkpoint is preserved
separately in `six-owner-verified.patch`, `six-owner-rom-counts.json` and
`rom-census/`; no full-live census was spent on it.

### Earliest-first map caller

The 42-cluster `early-cluster-worklist.json` now includes a signed source
partition for each cluster's first iteration. The first owner is the complete
two-row dungeon-map caller **`$0A:E384..E428`**, first needed at **12,996**
and again at **16,237**. Its executed source work is **1,834 clocks**. The
aggregate `$F89A` delta of 1,782 is not used as a price: another 52 clocks are
attributed to the jump helper, and the `$80B5/$F800` parent aliases cancel.
The signed `$8034` residual remains **−22** and is not absorbed into this
caller's price.

ROM block markers follow both floor-selection branches, mask branches, row
calls and restoration. Each drawing child keeps its existing price. The
generator entries split at child returns because those children return
**M8/X8**, even when called from a **M16/X16** portion of the parent. The
executed-ROM fixture covers **52 dungeon/floor/map combinations**. The final
generator audit adds **56 blocks** across the restored owners and this caller,
without changing or removing any of the **4,025** existing block prices.

The complete map-owner trial is admitted on the ROM lane: **2,227 missing /
0 excess / 0 FP** over **1,473,123 iterations**, gate PASS/unexplained zero,
with every runtime hold equal to P2. Relative to the corrected six-owner
checkpoint, only the holds at 12,996 and 16,237 change. The final continuous
0–148,000 prefix passes with **102 / 0 / 0**. The next unresolved cluster is
**15,313**; its dungeon/sprite caller and child aliases remain signed in the
worklist, without converting their aggregate difference into a price.

The final default executable is SHA-256
`d75103e9acc985569950e1a250d673ebc7963af9504b0dc8f87135b6c4802356`.
`final-rom-census/`, `final-carried-prefix.csv` and the three bounded final
windows use that executable. The fresh default and `--no-default-features`
product builds are warning-free, and rebuilding default restores the same
binary hash. All **37 focused Rust tests** and **seven generator tests** pass;
the library-test build retains pre-existing unrelated dead-code warnings.

### Final live census and poll partition

The **one single full seeded live census** completes **1,473,123 iterations**,
gate PASS/unexplained zero, with no P2 disagreement. Live totals are
**2,702 missing / 446 excess / 83 FP**, versus baseline **2,511 / 251 / 72**.
This is a **worse live total**, despite the 29-hold ROM improvement. Admission
uses the requested poll-episode rule, not a claim that live timing improved.

`live.csv`, `live-partition.json` and `live-partition-bounds.json` retain all
baseline/candidate values and the observed poll witnesses. The table gives
the candidate's live-minus-ROM contribution, not the change from baseline:

| Origin poll | Candidate hold-difference rows | Added missing | Added excess | Added FP | Changed rows versus baseline |
| --- | ---: | ---: | ---: | ---: | ---: |
| 94,853 (removed) | 0 | 0 | 0 | 0 | 6 removed |
| 142,087 (new) | 5 | 9 | 9 | 2 | 5 |
| 229,034 (new) | 4 | 9 | 9 | 2 | 4 |
| 380,880 (new) | 15 | 114 | 90 | 5 | 15 |
| 462,627 (inherited) | 6 | 13 | 13 | 3 | 0 |
| 881,733 (inherited) | 135 | 234 | 230 | 67 | 0 |
| 1,064,733 (new) | 12 | 96 | 95 | 4 | 12 |
| **Total** | **177** | **475** | **446** | **83** | **42** |

All six candidate poll witnesses are native **zero** versus source **`$13`**
at `$08:C606..C610`. The removed episode is witnessed in the baseline log;
its six formerly different hold rows now equal the ROM lane. The two inherited
episodes retain every hold differential. Every changed live-only hold row is
inside one of the following observed execution/reseed bounds; none is merely
accepted because some earlier poll existed:

* **94,853 removal:** changed holds 94,854–94,860; the baseline main-work
  differences end at 94,887.
* **142,087:** changed holds 142,088–142,094; changed main work starts at the
  poll and ends at 142,121.
* **229,034:** changed holds 229,035–229,041; changed main work starts at the
  poll and ends at 229,068.
* **380,880:** changed main work starts at the poll; hold differences end at
  **383,635**, with actual reseeds at **382,000** and **383,636**.
* **462,627:** inherited hold differences end at 463,180, before the actual
  reseed at 464,000.
* **881,733:** inherited hold differences end at 884,324; actual reseeds are
  882,000, 884,000 and 884,327.
* **1,064,733:** changed main work starts at the poll; hold differences end
  at **1,066,961**, main differences at 1,066,962, and actual reseeds occur at
  **1,066,000** and **1,066,963**.

The secondary cost comparison separately retains **1,889 main-work**,
**1,267 opening-handler** and **1,232 lag-handler** changed differentials.
Some budget-only differences outlive reseeds, reaching 1,469,218. They are
not relabeled as hold differences or assigned causality merely by the nearest
preceding poll. The admission check concerns the 42 changed hold rows; it
does not establish source-equal SPC phase, delivered-command timing, chip
state or waveform. Those strict sound proofs were not recaptured.

| Before window starting | Baseline ROM missing | Final ROM missing | Baseline live missing | Final live missing |
| --- | ---: | ---: | ---: | ---: |
| 94,334 | 67 | 56 | 67 | 56 |
| 141,568 | 103 | 92 | 111 | 92 |
| 228,499 | 240 | 221 | 248 | 230 |
| 380,243 | 394 | 375 | 402 | 393 |
| 461,989 | 473 | 454 | 481 | 586 |

These are cumulative missing-hold contributions, not measured net SPC phase.

The required **single full CPU picture census**, on the ROM-answer lane,
completes **1,473,123 iterations**, gate PASS/unexplained zero. It preserves
**1,462,997 exact pictures**; all **1,473,123 RGB hashes** are identical to
`target/lag9/picture-hashes.txt`, with no exact picture lost. This does not
claim matching live pictures through the newly observed poll episodes.

`verification.json`, `artifact-hashes.json`, `final-candidate.patch` and
`picture-identity.json` bind the final sources, executable and evidence.
The active default product binary has the same hash as all final censuses.
Evidence remains below **3 GB**, and `git diff --check` passes. No GPU run,
Snes9x process launch, continuous live replay, full library suite,
protected-cache/shared-ledger write, formatting pass, commit or push was
performed.

This is a verified partial repair under the revised admission rule, with
**2,227 ROM missing holds still open** and worse live totals. The next
earliest unresolved cluster is **15,313**; strict sound and exact A/V
acceptance remain open.

## Lagless missing-hold owners 11

CPU-only continuation on `codex/lag11`, base `caa943ff`. Evidence is under
`target/lag11/`; the authoritative oracle cache and shared `full-v2` ledger
remain read-only. This batch prices four complete door-priority leaves from
the earliest unresolved cluster. It does **not** close that cluster or reduce
the five song-window prefix deficits.

The ordered inputs are `target/lag10/early-cluster-worklist.json` and
`target/lag7/prefix-clusters-before.json`. The admitted source boundaries are
`$01:AF8B..AFC7` (north), `$01:AFD4..B00C` (south), `$01:B017..B04F` (west),
and `$01:B05C..B091` (east). Each is a real JSR leaf with no children. Generated
ROM markers charge setup, every executed four-tile pass, actual taken loop
branches, and RTS. No aggregate caller delta supplies a price.

At **15,313**, south/west/east contribute exactly **3,010 / 2,916 / 2,890**
source clocks; north is not called. The native profile adds exactly those
**8,816 clocks**, with no other exclusive owner changing. Main work rises
**4,962,046 → 4,970,862**, still below source **5,002,268**. The signed residual
is **31,406**, and the diagnostic main-work shortfall to the next hold falls
**18,146 → 9,330**. Neither residual is used as a price. The caller/JMP and
sprite aliases remain explicit in `first-owner-partition.json`; they are not
absorbed by these leaves. `source-partitions.json` also retains the witnesses
at 72,710 and 81,253. The updated ordered worklist keeps all three clusters open.

The executed-ROM fixture covers **504 cases**: every terminating north/west
distance and south/east loop length, across four tilemap quadrants. It checks
instruction clocks and complete BG2 tile output. The first fixture attempt
failed because its setup did not import raw RAM into the native tilemap owner;
correcting that test setup requires no gameplay change. The final fixture,
**nine marker-ownership tests**, and **seven generator tests** pass. The audit
adds **81 instructions / 12 blocks**, preserving all **16,760 instructions /
4,081 blocks** already priced.

The full eight-slice seeded ROM census completes **1,473,123 iterations**,
gate **PASS**, unexplained zero, with **2,226 missing / 0 excess / 0 FP**,
versus **2,227 / 0 / 0** before. Every runtime hold equals P2. Only iteration
**577,651** changes its hold count, **14 → 15**, equal to source; its main work
rises **4,976,592 → 4,988,204**. A focused 576,000–578,000 ROM window is
logically clean; the recovered iteration's profile assigns the entire
**11,612-clock** increase to two west and two east leaf calls, with no child
work. No later overcharge was observed.

| Before window starting | Baseline ROM missing | Candidate ROM missing |
| --- | ---: | ---: |
| 94,334 | 56 | 56 |
| 141,568 | 92 | 92 |
| 228,499 | 221 | 221 |
| 380,243 | 375 | 375 |
| 461,989 | 454 | 454 |

The continuous **0–148,000** recorded-answer prefix also passes, unexplained
zero, with unchanged **102 / 0 / 0**. The bounded 14,000–16,000 trial is
logically clean with zero excess/FP; its nonzero start is not a complete-route
summary-gate pass.

The **one full seeded live census** completes **1,473,123 iterations**,
gate **PASS**, unexplained zero, with no P2 disagreements. Live totals are
**2,728 missing / 473 excess / 88 FP**, versus **2,702 / 446 / 83** before.
This is a worse live total. Admission uses owners 9's poll-episode rule;
it does not establish improved live timing or source-equal sound phase.

| Origin poll | Candidate live-versus-ROM hold rows | Added missing | Added excess | Added FP | Changed rows versus baseline |
| --- | ---: | ---: | ---: | ---: | ---: |
| 142,087 | 5 | 9 | 9 | 2 | 0 |
| 229,034 | 4 | 9 | 9 | 2 | 0 |
| 380,880 | 15 | 114 | 90 | 5 | 0 |
| 462,627 | 6 | 13 | 13 | 3 | 0 |
| 627,255 (new) | 10 | 27 | 27 | 5 | 10 |
| 881,733 | 135 | 234 | 230 | 67 | 0 |
| 1,064,733 | 12 | 96 | 95 | 4 | 0 |
| **Total** | **187** | **502** | **473** | **88** | **10** |

All ten changed live-only hold rows belong to the new observed song-end
poll: native **zero** versus source **`$13`** at **627,255**, on the existing
`$08:C606..C610` path. Changed main work starts at that poll and ends at
**629,601**; changed holds run **627,316–629,462**. Actual reseeds at
**628,000** and **629,845** bound the execution aftermath. All six inherited
episodes retain their complete hold differentials. `live-partition.json`
contains every baseline/candidate row, poll witness and reseed, with **zero
unattributed changed rows**. The recovered hold at 577,651 changes both ROM
and live equally and is not a live-only change.

The secondary partition retains **960 main-work**, **723 opening-handler**
and **712 lag-handler** changed differentials separately. Budget-only changes
extend from 565,444 to 744,395; they are not called hold changes or attributed
to the nearest preceding poll. The five live prefix deficits remain
**56 / 92 / 230 / 393 / 586**, unchanged from lag10. Delivered-command,
chip-state and waveform proofs were not recaptured; their prior limitations
remain open.

Default and `--no-default-features` product builds are warning-free. The
library-test build retains 24 existing unrelated dead-code warnings. The
restored default executable is byte-identical to the census candidate,
SHA-256 `024f3525d22f466562cfdf65112dd1efe795819b86e660c7769947d2dbaa4b54`.
`verification.json`, `identity.json`, the source partitions and artifact
hashes bind the retained result. `git diff --check` passes; evidence is below
3 GB. Only prices, generated entries, fixtures and this documentation change.
No classifier, matcher, P2 rule, environment switch or gameplay behavior was
changed. The conditional CPU picture census was therefore not rerun; no new
picture or exact A/V proof is claimed.

This is a **verified partial pricing stop**, admitted under owners 9's rule,
with **2,226 ROM missing holds** and all five prefix deficits still open.
Iteration **15,313** remains the next unresolved cluster. No GPU run,
Snes9x process, continuous live replay, full library suite, protected-cache
or shared-ledger write, formatting pass, commit or push was performed.

## Lagless pricing coverage 1

CPU-only diagnostic continuation on `codex/coverage1`, base `6c4f0bf8`.
Evidence is under `target/coverage1/`. **Partial diagnostic checkpoint: no
new prices are admitted, and the requested bulk-pricing step is not complete.**
Neither a missing marker nor an exclusive routine difference proves that a
source instruction is unpriced. The retained matcher and classifiers are
unchanged; this diagnostic does not relax their ownership requirements.

The baseline contains **2,222 missing iterations / 2,226 missing holds**.
The census has per-PC source evidence for **1,967** of those iterations,
including **all 453 missing rows before 462,627** and a route-spanning sample
through **1,447,695**. The other **255** later rows are listed explicitly;
sample totals are not extrapolated to the full route.

### Durable source extraction and native census

`logical_frame_ledger --coverage-source ORDERED.bin.zst COHORT.json FRESH.csv`
projects the retained ordered source stream without executing a CPU. The
cohort is a JSON object containing `rows`, an array of logical iteration IDs.
It preserves PC, M/X, count, instruction clocks and exclusive JSR/JSL owner;
JMP tails retain their caller. Entry/return, adjacency, call-stack width,
truncation and clock conservation checks fail closed. Output creation is
exclusive, and the companion receipt lists unavailable iterations.

The retained matcher1 stream supplies **1,023** current missing rows within
its verified 742,248-iteration boundary. Its manifest verifies unchanged.
The source-hash-verified `target/block-worklist/source/instruction-profile.csv`
supplies the remaining sampled rows. Every overlapping iteration agrees at
the complete `(routine, PC, M/X, count, clocks)` level. Source PC totals
reconcile to the ordered headers and the older independent routine totals,
with **zero residual**. No protected cache or shared ledger was written.

`scripts/pricing_coverage.py` consumes those CSVs, the current native
`ZELDA3_SHADOW_AUDIT_SUMMARY` receipts and predictor CSVs. It decodes generated
block footprints from the ROM, observes actual marker/model execution, and
keeps legacy scalar charges and separate VWF work explicit. The native charge
classes reconcile to each observer total; observer plus VWF reconciles to the
predictor's main total. Every selected native iteration has a receipt.
Its required `--source-totals` inputs are the extraction receipt JSON and/or
source routine-profile CSV; it rejects a partial per-PC row whose sum differs
from that independent total.

This is **same-iteration PC membership**, not ordered occurrence matching:
one native visit does not prove every repeated source visit was priced.
Marker widths are checked; model PC presence does not establish its complete
call/width/retirement identity. Any remaining opaque native work leaves
unmatched source PCs in `unresolved_alias`, rather than declaring them free.

`scripts/pricing_scalar_footprints.json` starts the reviewed scalar catalog
with the complete `ClearOamBuffer` leaf `$00:841E..8489`. Its existing source
comment enumerates setup, four 32-store passes, three taken branches and RTS,
giving the existing **5,300-clock** scalar. The catalog binds the native file
hash, observed charge site, routine scope, complete decoded leaf and fixed
charge contract. This maps an existing price; it adds no runtime charge.
Changed source, scope or charge contracts require re-review.

| Sampled source instruction class | Clocks |
| --- | ---: |
| Generated marker touches PC at the recorded width | 380,650,416 |
| Executed instruction-model witness touches PC | 702,556,794 |
| Reviewed legacy scalar footprint touches PC | 10,425,100 |
| Unresolved scalar/model/domain aliases | 867,391,080 |
| Proved uncovered / admitted new pricing | 0 |
| **Source main total** | **1,961,023,390** |

All 1,967 classification equations have zero residual. Native main totals
**1,692,138,630**, leaving a signed **268,884,760-clock** difference. That
difference is not an additive price. `coverage.json` retains each iteration,
every routine, prefix windows, scalar sites and input identities;
`top50.csv` and `top50-before-462627.csv` provide the requested rankings with
clocks and missing-row counts. They rank **unresolved** work: there is no
proved-uncovered top-50 list.

The largest unresolved entry is `$00:8888` (**212,320,844 clocks / 27 rows**),
the song-bank upload protocol, whose separate timing paths are documented in
`spc_driver_clock.rs`. `$02:F691` (**35,904,696 / 23**) already has cache
hit/miss and output scalars in `overworld.rs`. `$00:F312`
(**24,587,840 / 56**) is charged across resumable spotlight phases in
`load_gfx.rs`. `$00:841E` initially appeared unresolved despite its complete
existing scalar; the reviewed footprint accounts for all its **10,425,100**
source clocks. These are concrete double-pricing risks, not new price owners.
Complete scalar/model and separate-domain ownership remains necessary before
converting this ranking into bulk pricing; no candidate was trialled or
selectively suppressed.

### Verification boundary

The fresh eight-slice seeded ROM census completes **1,473,123 iterations**,
gate **PASS**, unexplained zero: **2,226 missing / 0 excess / 0 FP**.
Every predictor row is byte-identical to lag11, including all main/handler
budgets, and every runtime hold equals P2. The five prefix deficits remain
**56 / 92 / 221 / 375 / 454** before and after. This verifies that enabling
the observer does not change the route's predictor results.

Default and `--no-default-features` product builds are warning-free; the
restored default executable is byte-identical to lag11, SHA-256
`024f3525d22f466562cfdf65112dd1efe795819b86e660c7769947d2dbaa4b54`.
All **six ledger-example tests** and **six coverage-tool tests** pass.
No gameplay or runtime pricing changed, so the conditional CPU picture census
was not rerun; no new picture, delivered-command, chip-state or waveform proof
is claimed. The existing 1,462,997-exact-picture result remains prior evidence.

The **one full seeded live census** also completes **1,473,123 iterations**,
gate **PASS**, unexplained zero: **2,728 missing / 473 excess / 88 FP**,
with no P2 disagreement. `live.csv` is byte-identical to lag11's live CSV.
All **187** live-versus-ROM hold rows retain their prior bounded episode,
with **zero changed live-only hold rows and zero unattributed rows**. Every
origin poll is observed again in the fresh log as native zero versus source
`$13`. `live-partition.json` checks every differing row against the retained
episode's execution/reseed bound; no nearest-preceding-poll inference is used.

| Origin poll | Hold rows | Added missing | Added excess | Added FP |
| --- | ---: | ---: | ---: | ---: |
| 142,087 | 5 | 9 | 9 | 2 |
| 229,034 | 4 | 9 | 9 | 2 |
| 380,880 | 15 | 114 | 90 | 5 |
| 462,627 | 6 | 13 | 13 | 3 |
| 627,255 | 10 | 27 | 27 | 5 |
| 881,733 | 135 | 234 | 230 | 67 |
| 1,064,733 | 12 | 96 | 95 | 4 |
| **Total** | **187** | **502** | **473** | **88** |

Live prefix deficits remain **56 / 92 / 230 / 393 / 586**. These unchanged
controls do not satisfy owners 9's fewer-missing-holds requirement for a new
pricing admission. The result is a verified diagnostic stop, with bulk
pricing and complete instruction ownership still open.

`verification.json` and `artifact-hashes.json` bind the checks and artifacts.
Evidence remains below **1 GB** and `git diff --check` passes. No GPU,
Snes9x process, continuous live replay, full library suite, protected-cache
or shared-ledger write, formatting pass, commit or push was performed.

## Lagless pricing coverage 2

CPU-only continuation on `codex/coverage2`, base `98272549`. Evidence is
under `target/coverage2/`. **Verified partial diagnostic stop: no new pricing
is retained. Per-routine ownership and all-path correction of every residual
remain incomplete.** The OAM scalar error below is proved, but its complete
trial fails owners 9's requirement to reduce missing holds and is withdrawn.

### Routine comparison and ownership limits

`routine-table.csv` / `routine-table.md` report **151 routines** occurring in
at least 500 of the **1,967 source-covered missing rows**. Each has controls:
**629 non-missing rows** from ten seeded windows, after excluding 11 rows whose
seed-boundary budgets differ from the baseline. All 640 captured control rows
reconcile source PC totals to ordered-stream receipts, native charge classes
to the observer, and observer plus separate VWF to the predictor, with zero
residual. Twenty-two reused input hashes and the source manifest verify.

The source sample still lacks **255 of 2,222 missing rows**. Therefore this
is not a complete-route eligibility census: `threshold-uncertainty.json`
lists 38 additional routines that could cross 500 if every unavailable row
contained them. No sample quantity is extrapolated. `paired-routine-rows.csv`
retains source-exclusive clocks, actual native-scope clocks and signed deltas
per iteration. `routine-summary.json` resolves observed scalar/model charge
sites and names; `ownership-notes.json` records reviewed boundaries.

Delta below is **native minus source**, in master clocks per iteration.
These are actual scope totals; a negative cell is not an additive price.
The full 151-row table also gives source/native means and the supplemental
ownership projection.

| Routine | Missing rows | Mean delta | Median | Native < source | Control rows | Mean delta | Median | Native < source |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| $05:DF6C | 1463 | -722.14 | 0 | 18.46% | 116 | 0.00 | 0 | 0.00% |
| $06:83F2 | 1871 | 0.00 | 0 | 0.00% | 457 | 0.00 | 0 | 0.00% |
| $06:84E2 | 1911 | -2555.92 | -2194 | 99.48% | 628 | -2629.01 | -2272 | 100.00% |
| $06:E41E | 1851 | 947.65 | 346 | 2.97% | 429 | 220.08 | 346 | 3.50% |
| $07:D9D8 | 1602 | -33.32 | 0 | 4.43% | 318 | -33.67 | 0 | 3.46% |
| $07:E245 | 1345 | -2546.40 | -2390 | 100.00% | 294 | -2406.18 | -2416 | 100.00% |
| $0D:A18E | 1885 | 0.00 | 0 | 0.00% | 613 | 0.00 | 0 | 0.00% |
| $0D:BB0A | 1909 | -0.14 | 0 | 2.20% | 604 | 0.00 | 0 | 0.00% |
| $0D:F0F7 | 1852 | 0.00 | 0 | 0.00% | 383 | 0.00 | 0 | 0.00% |
| $0D:FB94 | 1825 | 0.00 | 0 | 0.00% | 378 | 0.00 | 0 | 0.00% |
| $0D:FDAB | 1852 | 0.00 | 0 | 0.00% | 378 | 0.00 | 0 | 0.00% |
| $0D:FDD9 | 1852 | 0.00 | 0 | 0.00% | 378 | 0.00 | 0 | 0.00% |

**87 native scopes match every sampled missing row exactly**, accounting for
238,572,382 source clocks. The hypothesized blanket undercharge is absent in
LinkOam_Main, sprite timers, HUD decimal conversion and heart drawing.

A separate conservative diagnostic projects a native scope entry, marker PC
or conserving model-step PC only where that PC has one source owner in the
same iteration. It conserves the charged total and leaves ambiguous or
nonconserving model witnesses opaque. This is PC membership, **not ordered
occurrence matching**, and zero unmapped work does not prove source identity.
Neither the classifier nor matcher changes, and no projection becomes a price.

This resolves `$07:E245`: native `$07:E373` and `$07:E3E0` scopes own its
fallthrough/JMP tails. Including those existing charges makes every sampled
missing/control delta zero. Adding its raw 2,546-clock mean deficit would
charge existing work twice. The `$06:84E2` projection reduces the mean missing
scope deficit to 705.93 clocks, but its dispatched bodies still need precise
occurrence ownership. `$06:E41E` has a positive mean, not a consistent
undercharge; its continuation/caller aliases remain explicit.

The canonical inactive-helper family (`$06:D9EC`, `$05:F94E`, `$1A:F954`,
`$1D:E8A2`, `$1E:FE78`) matches exactly on **2,184 / 2,204 paired rows**, with
**zero native-under-source rows**; the other 20 have native excess work.
The paused family (`$06:D9F3`, `$05:F955`, `$1E:FE7F`) matches all **131** rows.
`family-rows.csv` and `family-summary-paired.json` retain this check. Individual
negative bank-specific scopes must not be priced independently of their
existing canonical helper charges; mixed occurrences are not silently split.

`routine-input-buckets.json` records delta ranges/medians by actual source
entry visits, without fitting prices. Only the OAM input relationship below
has a new complete executed-ROM fixture. The residual worklist includes every
routine with any negative projected row in either cohort; no consistency
threshold supplies a price. Examples include the bunny, toss, cape and Y-button
helpers, sword swing, collision/drawing wrappers and dispatched sprite bodies.
For example, the sampled zero-timer bunny exit is the ROM's
LDA/ORA/taken-BEQ/CLC/RTS, **142 clocks**; the initial toss guard is
LDA/AND/taken-BEQ/CLC/RTS, **118 clocks**. Their semantic helpers have no direct
charge, and the reviewed caller prices exclude these bodies. Full alternate
paths and child ownership still require fixtures; no early-exit-only scalar
was added. Source block inventories and `residual-worklist.json` retain the
next work. Input/path causality for every residual is not claimed complete.

The Coverage 1 double-pricing boundaries remain intact: `$00:8888` has
separate song-upload timing, `$02:F691` has cache/output scalars, `$00:F312`
has resumable spotlight prices, and `$00:841E` retains its exact 5,300-clock
leaf. None receives a new charge.

### Proved OAM error; whole trial withdrawn

`Oam_GetBufferPosition` `$0D:BB0A..BB5A` is a leaf. Its overflow scalar
omits the indexed-read page crossing at **`$0D:BB39 LDA $BAAA,Y`**. The source
byte index is `2 * (region * 8 + (cursor & 7))`; region F crosses when that
index is at least 86. The existing instruction model already prices this
crossing; the ordinary scalar did not. The sampled **42** deficient rows
contain **43 crossings / 258 clocks**, exactly equal to their complete
source-minus-native discrepancy (`oam-cause.json`). No residual supplied a price.

The trial replaces the entire scalar with **44 decoded instructions / four
generated ROM blocks**, including the taken BCC, the crossing marker and
RTS. No child work or previous scalar is retained. The unmodified-ROM fixture
passes **2,304 cases**, covering all six regions, eight fallback cursors,
fitting/equality/overflow paths and wrapping sums; it checks instruction
clocks and OAM pointers/counters. Existing allocator/model tests also pass.

The focused 636,000–638,000 trial has zero excess/FP. The complete eight-slice
seeded ROM census passes, unexplained zero, **2,226 missing / 0 excess / 0 FP**.
All runtime holds equal P2. Main prices change on **752 iterations**, adding
4,908 clocks, but **no hold row changes**. Thus the entire trial is withdrawn
under owners 9: correct source pricing alone does not meet its fewer-missing
holds admission rule. `oam-trial.patch`, the tested binary, fixture logs and
`oam-trial-comparison.json` retain the rejected candidate; none of its runtime,
generated-entry or fixture edits remains in the checkout.

| Before window starting | Baseline ROM deficit | Trial ROM deficit | Restored ROM deficit |
| --- | ---: | ---: | ---: |
| 94,334 | 56 | 56 | 56 |
| 141,568 | 92 | 92 | 92 |
| 228,499 | 221 | 221 | 221 |
| 380,243 | 375 | 375 | 375 |
| 461,989 | 454 | 454 | 454 |

### Restored verification and live partition

The rebuilt default executable is byte-identical to the original baseline:
`024f3525d22f466562cfdf65112dd1efe795819b86e660c7769947d2dbaa4b54`.
The restored ROM evidence reuses the hash-verified Coverage 1 CSV and freshly
re-gates its log, PASS/unexplained zero; it is **not a second fresh ROM run**.
Default and `--no-default-features` builds are warning-free. Three targeted
OAM tests, nine ledger-ownership tests and seven generator tests pass; the
library-test build retains existing unrelated dead-code warnings.

The **one full seeded live census** on the restored binary completes
**1,473,123 iterations**, gate PASS/unexplained zero, with **2,728 missing /
473 excess / 88 FP**, and zero P2 disagreements. `live.csv` is byte-identical
to Coverage 1. All **187** live-versus-ROM hold rows retain their existing
execution/reseed-bounded episode, with **zero changed live-only rows and zero
unattributed rows**. All seven origin polls are freshly observed as native
zero versus source `$13`; `live-partition.json` checks the exact row values
and bounds, rather than choosing the nearest preceding poll.

| Origin poll | Hold rows | Added missing | Added excess | Added FP |
| --- | ---: | ---: | ---: | ---: |
| 142,087 | 5 | 9 | 9 | 2 |
| 229,034 | 4 | 9 | 9 | 2 |
| 380,880 | 15 | 114 | 90 | 5 |
| 462,627 | 6 | 13 | 13 | 3 |
| 627,255 | 10 | 27 | 27 | 5 |
| 881,733 | 135 | 234 | 230 | 67 |
| 1,064,733 | 12 | 96 | 95 | 4 |
| **Total** | **187** | **502** | **473** | **88** |

Live prefix deficits remain **56 / 92 / 230 / 393 / 586**. These are missing
hold contributions, not net SPC phase. No delivered-command, chip-state or
waveform proof was recaptured. No behavior change survives, so the conditional
CPU picture census was not rerun; **1,462,997 exact pictures remains prior
evidence**, not a new picture claim.

`verification.json`, input identities and artifact hashes bind the stopping
point. Evidence is below 3 GB; `git diff --check` passes. No GPU run, Snes9x
process, continuous live replay, full library suite, protected-cache/shared-
ledger write, new environment switch, formatting pass, commit or push occurred.
The general per-routine model/alias and all-path pricing work remains open.

## Lagless pricing coverage 3

CPU-only continuation on `codex/coverage3`, base `97760d5c`. Evidence is under
`target/coverage3/`. **Verified partial pricing stop: five of the 58 residual
worklist entries are priced, plus their shared cape tail; 53 entries remain
open. The requested complete bulk worklist is not finished.**

The user's revised admission rule permits exact prices to accumulate without
requiring recovered holds: generated blocks or corrected all-path ROM models,
children excluded, no double charge, full-route ROM missing not increased,
zero ROM excess/FP, gate PASS/unexplained zero, and owners 9's bounded
live-only hold-row poll partition. No classifier, matcher or P2 rule changes.
The first combined batch passes, so no census-failing owner required bisection.

### Prices retained

| Entry | Complete priced body and proof |
| --- | --- |
| `$0D:BB0A` Oam_GetBufferPosition | Re-admits Coverage 2's complete scalar replacement, including `$BB39` indexed-read crossing. All 2,304 region/cursor/fitting/equality/overflow/wrapping ROM cases pass. |
| `$07:82DA` Link_HandleBunnyTransformation | All guards, transformation setup, five-slot loop, countdown and returns through `$8364`, including shared `$82D8`. 144 executed-ROM cases cover every branch; reset, dash, poof, sound and palette children excluded. |
| `$07:B5D6` Link_HandleToss | Complete `$B5D6..B608` leaf, all three guards and `$0305` branch; 300 ROM cases. |
| `$07:B073` CheckYButtonPress | Complete `$B073..B086` leaf, all guards and success; 300 ROM cases. |
| `$07:AE88` cape lift guard | Complete guard and early return; the actual fallthrough uses the shared `$AE8F` owner once. Both guard paths pass. |
| `$07:AE8F` cape passive tail | Complete `$AE8F..AEBF` body and shared `$AE87` return; 216 ROM cases cover cape/item/timer/magic branches. `$AE47` child excluded. |

The generator decodes the unmodified ROM, whose SHA-256 matches the protected
cache manifest. Markers carry source addresses and branch outcomes; no sampled
count or deficit supplies a price. Existing caller charges exclude these
bodies. The OAM scalar is replaced, not retained alongside its markers.
`candidate-owners.json`, `candidate.patch`, fixture logs and
`rom-identity.json` retain the proof boundaries.

The inactive/paused canonical helper families remain intact. Inspection also
retains the already-priced `$06:84E2` dispatch and `$00:8781` split
(52 own clocks plus 362 caller clocks). Their residuals are not additive
parent prices. Sword swing's control/SFX/reset/spin tails, collision/drawing
wrapper occurrences, dispatched sprite bodies and `$00:80B5` aliases still
need complete exclusive proofs. The quiet-cape body additionally appears in
the existing file-load model, so a global helper charge would need that owner
resolved first. `remaining-worklist.json` preserves all 53 pending entries
and the reviewed boundaries; none is assigned a fitted residual price.

### Full-route ROM and prefix results

The fresh eight-chunk seeded census covers **1,473,123 iterations**, gate
**PASS**, **unexplained 0**. Missing / excess / FP improves from
**2,226 / 0 / 0** to **2,189 / 0 / 0**. Exactly **37** hold rows recover one
hold each; no row loses a hold, and every runtime hold equals P2.
Main prices change on **1,051,384 iterations**, adding **487,200,574 clocks**
over the route. OAM's isolated retained Coverage 2 trial recovered no holds;
that is no longer grounds to withdraw its exact repair.

| Before iteration | Baseline ROM deficit | Candidate ROM deficit | Candidate live deficit |
| --- | ---: | ---: | ---: |
| 94,334 | 56 | 53 | 53 |
| 141,568 | 92 | 89 | 89 |
| 228,499 | 221 | 216 | 216 |
| 380,243 | 375 | 367 | 367 |
| 461,989 | 454 | 445 | 559 |

These are cumulative missing-hold contributions, not measured net SPC phase.
`rom-comparison.json` retains every changed hold and the full price-delta
distribution. The 14,000–16,000 seeded trial also has zero excess/FP.

### One final live census

The single full seeded live run covers **1,473,123 iterations**, gate
**PASS**, **unexplained 0**. Missing / excess / FP is
**2,550 / 333 / 75**, versus baseline
**2,728 / 473 / 88**, with zero runtime/P2 disagreements.
There are **31 changed live-only hold rows**,
all within observed baseline/candidate poll episodes and their retained
execution/reseed bounds; **zero unattributed hold rows**. Removed episodes
retain their baseline witnesses and rows; they are not erased from the partition.

| Origin poll | Candidate live-versus-ROM hold rows | Changed rows versus baseline | Added missing / excess / FP |
| --- | ---: | ---: | ---: |
| 142,087 | 0 | 5 | 0 / 0 / 0 |
| 229,034 | 0 | 4 | 0 / 0 / 0 |
| 380,880 | 15 | 0 | 114 / 90 / 5 |
| 462,627 | 6 | 0 | 13 / 13 / 3 |
| 627,255 | 0 | 10 | 0 / 0 / 0 |
| 881,733 | 135 | 0 | 234 / 230 / 67 |
| 1,064,733 | 0 | 12 | 0 / 0 / 0 |

`live-partition.json` retains the actual row values, fresh poll observations
and actual reseeds. Secondary budget differentials are reported separately:
main_master: **1794**; cost_entry_handler: **1386**; cost_lag_handler: **1358**; cost_main_dma: **0**; cost_main_hdma: **1449**; cost_raster_wait: **17**. `live-secondary-rows.json` retains every such differential,
including handler-budget changes outside immediate episode execution windows.
Those are not counted as changed holds or assigned to the nearest preceding
poll. The hold partition is not a proof that every secondary budget difference
has a unique poll owner. Delivered-command, chip-state and waveform proofs
were not recaptured; the earlier sound-proof limitations remain open.

### Remaining sampled coverage and verification

The unchanged coverage tool was rerun on fresh native receipts for the same
**1,967 source-covered rows** of the original **2,222-row** missing cohort.
The **255 source-unavailable rows** remain explicit. All source/native
accounting equations have zero residual; every sampled native row is present.
All eight audit predictor CSVs are byte-identical to the default candidate's
corresponding full-route CSVs.

| Source instruction classification | Before clocks | After clocks |
| --- | ---: | ---: |
| marker_touched | 380,650,416 | 398,560,956 |
| model_touched | 702,556,794 | 702,556,794 |
| scalar_touched | 10,425,100 | 10,425,100 |
| unresolved_alias | 867,391,080 | 849,480,540 |

Source main remains **1,961,023,390 clocks**; sampled native main increases
from **1,692,138,630** to **1,692,801,808**, reducing the signed difference
from **268,884,760** to **268,221,582 clocks**. Generated-marker membership
gains **17,910,540 source clocks**, largely by replacing the previously opaque
OAM scalar; it is not that much new runtime work. Remaining unresolved aliases
are **849,480,540 clocks**. This is still same-iteration PC membership, not
ordered occurrence equivalence or a list of proved-unpriced instructions.
`coverage.json`, `coverage-comparison.json` and `top50-residual.csv` retain
all classifications and rankings.

All **19 player-module tests**, **three OAM allocator tests**, **seven block
generator tests** and **six coverage-tool tests** pass. Default,
`--no-default-features`, audit and restored-default builds are warning-free;
the library-test compilation retains 24 existing unrelated dead-code warnings.
The restored default binary is byte-identical to the census candidate.
`verification.json` and `artifact-hashes.json` bind source, binary and evidence;
`git diff --check` passes and evidence remains below 3 GB.

Only pricing, generated entries, fixtures and this documentation change.
No gameplay write or runtime rule changed, so the conditional CPU picture
census was not rerun: **1,462,997 exact pictures remains prior evidence**.
No GPU, Snes9x process, continuous live replay, full library suite, protected
cache/shared-ledger write, new environment switch, formatting, commit or push
was performed. Complete bulk residual pricing remains unfinished.


## Lagless pricing coverage 4

CPU-only continuation on `codex/coverage4`, base `a2366e9e`. Evidence is in
`target/coverage4/`. **Verified partial pricing stop: one complete collision
wrapper is added at 21 source-confirmed call sites; three existing drawing
owners and their shared entries are converted from scalars to ROM markers.
The complete 53-entry residual worklist is not finished.**

### Owners and exclusive proof

* `$05:B88D..B891` collision near wrapper: generated JSL `$06:E496` and RTS,
  **104 source instruction clocks**. Eight executed-ROM fixtures cover slot
  endpoints and indoor/outdoor/layer inputs, checking clocks and returned
  collision value. Twenty-one native sites now name the actual bank-$05
  wrapper. Nineteen already charged the `$06:E496` child; the rat and fireball
  sites now also invoke that existing complete child wrapper. `$06:E4AB`
  collision work remains its separate owner. Caller JSR prices remain intact.
* `$05:DF6C`, shared `$DF70`, and `$DF78..DFE4`: replaces the existing entry,
  complete drawing-loop and return scalars, including clipping, priority,
  repeated-loop branches and the off-screen return. No scalar executes beside
  its replacement markers. The ordinary non-lagless ledger totals are retained.
* `$05:DFE9..E00A`: replaces the priority-preparation scalars on every branch.
  The coordinate child remains excluded. Its existing `$05:FA57/FA58`
  double-return caller charge remains separate and unchanged; it is not added
  again or misrepresented as new uncovered work.
* `$1D:8549..854D`: complete drawing bank wrapper replacement, child excluded.
  `$05:DF75` and `$05:DFE5` deferred entries use the same verified drawing and
  preparation owners, excluding the existing OAM allocator child.

The drawing fixtures execute **210 direct/R6/bank-wrapper cases** and
**24 deferred-entry cases** against the unmodified ROM. They independently
check body, preparation and wrapper prices across the priority-state branches,
clipped Y, off-screen double return, and first/repeated/last loop iterations.
The initial fixture caught an omitted `$05:DF73 BRA` marker in the conversion;
it was corrected from the decoded ROM before any census candidate was built.
Prices come from ROM instructions, never from sampled deficits or hold counts.
The development ROM matches the read-only cache manifest identity.

`candidate-owners.json`, `collision-sites.json`, `candidate.patch`, fixture
logs and the generated block table retain the admitted boundaries. The largest
sampled owner, DrawMultiple, already had scalar prices: its coverage gain is
not new instruction work. Remaining `$06:E41E` coordinate/continuation aliases,
`$06:84E2` dispatched bodies, `$00:8781`'s existing 52-own/362-caller split,
and `$00:80B5` parent aliases receive no additive residual price. Sword swing
still needs complete control/SFX/reset/spin/recoil tail ownership. Canonical
inactive/paused families are preserved. `ranked-worklist.json` records the
source-clock ranking; `remaining-worklist.json` preserves all 53 residual
entries with their review status rather than claiming scalar conversion closed
every occurrence difference.

### ROM admission and prefixes

Both seeded trial windows, 14,000–16,000 and 636,000–638,000, have zero excess/FP.
The full eight-chunk seeded ROM census completes **1,473,123 iterations**,
**PASS**, **unexplained 0**. Missing/excess/FP improves from **2,189 / 0 / 0**
to **2,185 / 0 / 0**. The four changed hold rows
(49,037; 683,818; 683,820; 1,440,055) each recover one hold; none loses a hold.
Every runtime hold equals P2. The combined batch passes, so no census-failing
owner required withdrawal or bisection.

Main prices change on **104,876 iterations**, adding **20,488,122 clocks**
over the route. `rom-comparison.json` retains every changed column and hold.

| Before iteration | Baseline ROM deficit | Candidate ROM deficit | Candidate live deficit |
| --- | ---: | ---: | ---: |
| 94,334 | 53 | 52 | 52 |
| 141,568 | 89 | 88 | 88 |
| 228,499 | 216 | 215 | 215 |
| 380,243 | 367 | 366 | 366 |
| 461,989 | 445 | 444 | 444 |

These are cumulative missing-hold contributions, not net measured SPC phase.

### Single final live census

The one full seeded live census completes **1,473,123 iterations**, gate
**PASS**, **unexplained 0**, and zero runtime/P2 disagreements. Missing/excess/FP
is **2,432 / 243 / 70**, against Coverage 3's
**2,550 / 333 / 75**. There are **15 changed live-only
hold rows**, with **0 unattributed changed rows**
and **0 unattributed current rows**.

| Origin poll | Candidate live-versus-ROM hold rows | Changed rows versus baseline | Added missing / excess / FP |
| --- | ---: | ---: | ---: |
| 380,880 | 0 | 15 | 0 / 0 / 0 |
| 462,627 | 6 | 0 | 13 / 13 / 3 |
| 881,733 | 135 | 0 | 234 / 230 / 67 |

`live-partition.json` retains baseline and candidate row values, observed polls
and actual reseeds within the established execution/reseed episode bounds.
`live-secondary-rows.json` reports budget differences separately; they are not
silently assigned to the nearest preceding poll. This hold partition is not
source-equal delivered-command, SPC-state or waveform proof. Those strict
sound proofs were not recaptured.

### Sampled coverage and verification

The unchanged coverage tool consumes fresh native audit receipts for all
**2,222 selected rows**, of which **1,967** have retained source
instructions. The **255 unavailable rows** remain explicit.
The retained ordered source stream ends at iteration 742,247; no later retained
per-instruction stream was found for cheap extraction. The existing independent
per-PC sample supplies the later available rows; no cold ROM rerun was substituted
for missing source evidence. `source-availability.json` and hash-verified reused
source identities record this boundary.

An initial library-only audit feature build emitted empty receipt streams; it
is explicitly invalidated under `invalid-audit-library-feature/` and supplies
no coverage claim. The corrected executable-feature build passed a two-row
receipt pilot before the complete audit.

All eight audit predictor CSVs equal their default-candidate full-route chunks
byte for byte. Every native receipt is present; source class totals and native
observer/predictor equations have zero residual.

| Source instruction class | Coverage 3 clocks | Coverage 4 clocks |
| --- | ---: | ---: |
| marker_touched | 398,560,956 | 418,667,218 |
| model_touched | 702,556,794 | 702,556,794 |
| scalar_touched | 10,425,100 | 10,425,100 |
| unresolved_alias | 849,480,540 | 829,374,278 |
| uncovered | 0 | 0 |

Source main remains **1,961,023,390 clocks**. Sampled native main is
**1,692,878,744**, and the signed difference is **268,144,646**,
versus **268,221,582** before. These are same-iteration PC-touch
classes, not ordered occurrence equivalence or admitted additive prices.
`coverage-comparison.json`, `audit-price-delta.json` and `top50-residual.csv`
retain the classifications, native change accounting and residual ranking.
Across all 2,222 sampled native rows, the entire 76,936-clock increase is
59,176 clocks in the new near wrapper plus 17,760 in the newly invoked
existing child wrapper, with zero unexplained native price delta. The drawing
replacements add no native work in this sample.

All **227 sprite-module tests**, the additional deferred-entry test,
**seven generator tests** and **six coverage-tool tests** pass. Default,
`--no-default-features`, audit and restored-default builds are warning-free;
the test compilation retains the 24 pre-existing dead-code warnings. The
restored default binary is byte-identical to the ROM/live census candidate.
`git diff --check` passes; `verification.json` and `artifact-hashes.json` bind
the stopping point. Evidence remains below 3 GB.

Only pricing, source-entry selections, generated blocks, fixtures and this
note change. No gameplay operation or runtime pacing rule changed, so the
conditional CPU picture census was not rerun: **1,462,997 exact pictures is
prior evidence**, not a fresh picture claim. No classifier/matcher change,
GPU run, Snes9x process, continuous live replay, full library suite, protected
cache/shared-ledger write, new environment switch, cargo fmt, commit or push
occurred. Complete residual ownership and bulk pricing remain open.


## Lagless pricing coverage 5

CPU-only continuation on `codex/coverage5`, base `cf211ee8`. Evidence is in
`target/coverage5/`. **Verified partial coverage stop: three complete existing
scalar owners are replaced with generated ROM markers. No new runtime clocks
are added and no additional holds recover. Bulk residual ownership remains open.**

### Owners and ROM proof

* `$02:F691..F7CA`, map32 definition expansion: cache-hit/miss prefix, complete
  four-definition expansion, and four output-store fragments. The 80 executed-ROM
  cases cover both cache paths, all eight output indices and five definition
  positions; cache bytes and destination words also agree. Caller `$F600..F603`
  establishes DB `$7F`; `$F7AE AND #$0007` bounds the output index.
* `$02:FD87..FDF6`, map16 stripe preparation, plus its `$FDF7..FE46` fallthrough:
  both destination branches, every paired stripe pass, header/setup and second
  output half. The first `$FDF7` JSR child is excluded. The existing caller JSR
  remains separate. Original per-store continuation boundaries are preserved.
* `$02:FDF7..FE46`, map8 upload preparation: complete first-half child, header,
  four-word loop and return. Twenty-four executed-ROM calls are evaluated once
  for the parent and once for the child (48 exclusive comparisons); stack depth
  distinguishes the JSR child from the parent's fallthrough. Both map pages,
  both destination branches, wrapping source/stripe indices and first/repeated/
  last loops are covered. Stripe inputs and upload headers agree.

The generator now accepts an explicit reviewed indexed-read bound and checks
**every effective byte** for a uniform slow-memory bus class. Unknown, mixed
or conflicting bounds fail closed. The new bound is seven, from the ROM mask,
not a sampled count. Generator regression tests include rejection across bus
classes. No classifier or matcher changes. All replaced scalar fragment prices
match their generated replacements exactly; no scalar executes alongside them.

The initial map8 test's clock check passed but an additional output comparison
encountered unused ROM map8 words normalized by the packed assets. The final
fixture explicitly checks clock ownership, stripe inputs and headers; it does
not claim all packed table words are byte-identical to the raw ROM. No runtime
behavior was changed to accommodate the fixture.

The ranked review retains `$00:8888`'s separate song-upload protocol ownership,
`$0D:A18E`'s existing instruction model, and `$00:F312`'s resumable spotlight
work. `$01:893C` mixes dispatcher and JMP-tail drawer aliases; full exclusive
path accounting remains open. No residual difference was added as a price.
`candidate-owners.json`, `ranked-worklist.json` and `remaining-worklist.json`
retain these boundaries, including the prior 53-entry occurrence worklist.

### Admission, prefixes and live partition

The 14,000–16,000 and 636,000–638,000 seeded windows pass with zero excess/FP.
The eight-chunk seeded ROM census covers **1,473,123 iterations**, gate **PASS**,
**unexplained 0**. Missing/excess/FP remains **2,185 / 0 / 0**; every runtime
hold equals P2. All predictor columns are unchanged from Coverage 4. No owner
required withdrawal or bisection, and zero holds recovered.

| Before iteration | Baseline ROM deficit | Candidate ROM deficit |
| --- | ---: | ---: |
| 94,334 | 52 | 52 |
| 141,568 | 88 | 88 |
| 228,499 | 215 | 215 |
| 380,243 | 366 | 366 |
| 461,989 | 444 | 444 |

The single full seeded live census covers **1,473,123 iterations**, gate
**PASS**, **unexplained 0**. Missing/excess/FP remains **2,432 / 243 / 70**;
every runtime hold equals P2. Changed live-only hold rows: **0**;
unattributed current or changed rows: **0**.

| Origin poll | Current live-versus-ROM hold rows | Changed rows | Added missing / excess / FP |
| --- | ---: | ---: | ---: |
| 462,627 | 6 | 0 | 13 / 13 / 3 |
| 881,733 | 135 | 0 | 234 / 230 / 67 |

`live-partition.json` retains actual row values, observed polls and execution/
reseed bounds; secondary budget differences are in `live-secondary-rows.json`.
Prefix deficits are missing-hold contributions, not measured SPC phase.
No new delivered-command, chip-state or waveform proof is claimed.

### Sampled residual and verification

Fresh native audit receipts cover all **2,222 selected rows**, with retained
source instructions for **1,967**; **255 source-unavailable rows** remain explicit.
Source identities were rehashed. All eight audit predictor CSVs equal the
candidate default chunks byte for byte; every accounting equation has zero
residual and no native receipt is missing. The unchanged coverage tool reports:

| Source instruction class | Coverage 4 clocks | Coverage 5 clocks |
| --- | ---: | ---: |
| marker_touched | 418,667,218 | 482,669,386 |
| model_touched | 702,556,794 | 702,556,794 |
| scalar_touched | 10,425,100 | 10,425,100 |
| unresolved_alias | 829,374,278 | 765,372,110 |
| uncovered | 0 | 0 |

Source main remains **1,961,023,390** clocks; sampled native main remains
**1,692,878,744**, signed difference **268,144,646**.
This is same-iteration PC membership, not ordered occurrence equivalence.
The **64,002,168-clock** marker gain includes **33,543,804 clocks before
462,627**. Each of the three converted source owners now has zero unresolved
clocks in this sample. This proves existing charge coverage, not new runtime work.

Top 20 remaining unresolved aliases (`top20-residual.csv`):

| Routine | Unresolved source clocks | Before 462,627 |
| --- | ---: | ---: |
| `$00:8888` | 212,320,844 | 141,005,650 |
| `$0d:a18e` | 28,935,000 | 6,021,584 |
| `$00:f312` | 24,587,840 | 10,930,526 |
| `$01:893c` | 24,053,720 | 14,720,412 |
| `$06:e41e` | 18,511,452 | 3,247,412 |
| `$06:83f2` | 17,562,454 | 2,967,576 |
| `$00:e9e4` | 14,687,028 | 13,023,258 |
| `$06:84e2` | 14,147,678 | 3,027,322 |
| `$09:c6f5` | 13,288,600 | 5,496,256 |
| `$00:df4f` | 12,989,356 | 10,020,372 |
| `$0d:f0f7` | 12,617,688 | 1,926,306 |
| `$07:d9d8` | 12,570,954 | 1,687,306 |
| `$0d:fdab` | 11,098,308 | 1,459,092 |
| `$0e:cb5e` | 10,039,322 | 4,655,784 |
| `$09:b0b6` | 9,664,876 | 689,102 |
| `$06:e72f` | 9,617,840 | 1,212,776 |
| `$0d:fdd9` | 8,974,968 | 1,086,050 |
| `$02:febb` | 7,626,922 | 4,083,872 |
| `$0d:fb94` | 6,929,684 | 1,342,182 |
| `$00:882e` | 6,674,802 | 2,407,062 |

All **45 overworld tests**, **eight generator tests** and **six coverage-tool
tests** pass. Default, reduced-feature, audit and restored-default builds are
warning-free; the test compilation retains existing unrelated dead-code warnings.
The restored binary is byte-identical to the ROM/live census candidate.
`verification.json` and `artifact-hashes.json` bind the stopping point;
`git diff --check` passes and evidence remains below 3 GB.

Only pricing annotations, their generator, fixtures and documentation changed.
The conditional CPU picture census was not rerun: **1,462,997 exact pictures is
prior evidence**, not a new picture claim. No GPU, Snes9x, continuous live replay,
full library suite, protected cache/shared-ledger write, new environment switch,
formatting, commit or push occurred.

## Lagless pricing coverage 6

CPU-only continuation on `codex/coverage6`, base `d36e2726`. Evidence is under
`target/coverage6/`. The upload audit follows every route upload, including
source-unheld iterations, rather than just Coverage 5's 27 missing rows.

### Upload ownership and measurement

The ledger contains **228 actual `$00:8888` protocols**. Three additional
iterations (571,209, 693,051 and 1,394,043) exceed the census's 256-write
classification boundary through repeated NMI stores, but have no upload
handshake reads. They are retained separately; the classifier is unchanged.

ROM-answer replay does **not** install upload acknowledgements: the harness
excludes these iterations when installing `$2140` song-end answers. Both
native lanes run `SongBankHostTransfer` against their own executing SPC.
The lagless path starts the deferred upload from `rtl_native_pacing.rs`,
advances its bus-access coroutine in `spc_driver_clock.rs`, and suppresses
NMI callbacks until its actual return forecast. The separate instruction
model in `cycle_models/song_bank.rs` is not a replacement for this runtime
protocol. A difference in upload wait counts is therefore not a price charged
from replayed answers in the ROM lane.

`source-upload-terms.json` reconstructs the executed paths from the ledger's
ordered port reads and writes. Every acknowledgement is consumed in order;
headers, counters, payload lengths and the terminal clears are checked.
The three payload shapes contain 12,747 / 13,366 / 8,344 bytes, each with
two data blocks and one terminal header. None takes `$88AD`'s bank wrap or
`$88CA`'s zero-token retry. `source-reconstruction-proof.json` compares these
totals with the 27 retained instruction profiles: **all deltas are zero**.

The existing `ZELDA3_DEBUG_SONG_UPLOAD` switch now optionally reports each
completed native transfer's access counts, CPU gaps and refresh-inclusive
wall gaps. Preview clones never emit completion records. The observation
fixture checks unchanged port writes, deadlines and serialized state, plus
conservation of every first-read-to-final-clear gap. This instrumentation
does not add upload work to the main cycle ledger.

The CPU total includes `$8888..8890` and the first CMP's fetches (116 clocks
before the first bus read), and the final store's remaining access plus
`PLP/RTS` (76 after the last clear). The taken receiver-ready loop costs
**58 clocks**; the header and byte acknowledgement loops cost **52 clocks**.
`rom-upload-comparison.csv` and `live-upload-comparison.csv` retain every
upload's signed native-minus-source total and its ready/header/byte terms.
Native refresh is separate from instruction clocks. The source masked count
is `close_host - open_host - lag_nmis - 1`; it is compared with native masked
callbacks, never with accepted P2 holds. Neither callback counts nor a total
CPU delta establish equal command phase, chip state or waveform.

### Residual owner retained

`$00:DF4F..DFB7` (`Do3To4High16Bit`) replaces its existing scalar with generated
ROM blocks: entry, per-tile setup, four two-row passes, destination advance,
both loop branches and return. There are no children. All **16 complete
executed-ROM fixtures** agree on clocks and output bytes, covering first and
repeated tiles and aligned/unaligned source and destination addresses.
The scalar remains exclusive to the ordinary timing lane; lagless uses the
markers once. The complete clock total is unchanged.

`residual-review.json` retains the top-20 review. `$0D:A18E` already has an
instruction model and continuation prices; `$00:F312` partitions resumable
spotlight work and its raster barrier. `$01:893C` includes JMP-tail object
drawer aliases. `$00:E9E4` already prices the aux-zero guard, three channel
branches and both color-index ranges. No unresolved parent total is added
on top of those owners. Their remaining exclusive marker proofs stay open.

### ROM admission and upload results

The eight-chunk seeded ROM census completes **1,473,123 iterations**, gate
**PASS**, unexplained **0**. Its CSV is byte-identical to Coverage 5's:
**2,185 missing / 0 excess / 0 FP**, zero runtime/P2 disagreements, and zero
recovered holds. Prefix deficits remain **52 / 88 / 215 / 366 / 444** before
94,334 / 141,568 / 228,499 / 380,243 / 461,989. The 10,000–12,000 seeded trial
also has zero missing/excess/FP; only its opening row's handler history
differs from the carried full-route baseline.

All **228** ROM-lane native transfers pair with the source upload iterations.
Source upload instruction work totals **1,809,325,130 clocks**; native totals
**1,805,988,672**, a signed difference of **−3,336,458**. The partition is:

| Term | Native minus source CPU clocks |
| --- | ---: |
| Receiver-ready waits, 58 clocks per taken loop | −3,337,030 |
| Header-ack waits, 52 clocks per taken loop | +780 |
| Byte-ack waits, 52 clocks per taken loop | −208 |
| Other instruction work | **0** |

Every individual upload has zero remaining term, not merely a cancelling
aggregate. Ready/header/byte counts differ on 228 / 187 / 140 uploads.
Individual total deltas range from −400,480 to +435,602 clocks. Native
refresh contributes a separately reported **54,562,200 clocks**. Source
masked intervals total **5,202**, versus **5,147** native callbacks; those
55 callbacks are not recovered or missing accepted lag holds. Entry phase,
refresh and callback cuts remain separate from the instruction-cost proof.
No route-executed byte-loop or handshake-loop unit price is found wrong,
and no upload timing repair or additional main price is admitted.

### Fresh marker coverage

All eight fresh audit predictor CSVs are byte-identical to their default
census chunks. Native receipts cover all **2,222** selected rows; source
instructions cover **1,967**, with **255** source-unavailable rows explicit.
All accounting equations conserve clocks. The unchanged coverage tool reports:

| Source instruction class | Coverage 5 clocks | Coverage 6 clocks |
| --- | ---: | ---: |
| marker_touched | 482,669,386 | 495,658,742 |
| model_touched | 702,556,794 | 702,556,794 |
| scalar_touched | 10,425,100 | 10,425,100 |
| unresolved_alias | 765,372,110 | 752,382,754 |
| uncovered | 0 | 0 |

The **12,989,356-clock** marker gain is exactly `$00:DF4F`'s formerly
unresolved sampled work, including **10,020,372 clocks before 462,627**.
Source main remains **1,961,023,390**; sampled native main remains
**1,692,878,744**; signed difference remains **268,144,646 clocks**.
This is existing-price coverage, not new runtime work or ordered retirement
equivalence. `coverage.json`, `coverage-comparison.json`, `audit-equality.json`
and `top20-residual.csv` retain the results.

All **43 SPC-clock tests**, **35 graphics-loader tests**, **eight generator
tests** and **six coverage-tool tests** pass. Default, reduced-feature, audit
and restored-default builds are warning-free; the lib-test build retains
existing unrelated dead-code warnings. The restored default binary is
byte-identical to the census candidate. The ROM, SRAM and route input hashes
match the protected cache manifest. Classifiers and matchers are unchanged;
the replay harness diff contains only the opt-in observation.

**Completion note.** The batch's single full seeded live census was interrupted when
the Codex session ended at about 1.17M iterations. It was rerun in full on the same
source: **1,473,123 iterations, PASS, unexplained 0, 2,432 missing / 243 excess /
70 FP**, with zero held/lag rows changed against Coverage 5's live CSV. The focused
SPC-clock, graphics-loader and `Do3To4High16Bit` fixture tests pass (78).

## Lagless pricing coverage 7

CPU-only continuation on `codex/coverage7`, base `ac340898`. Evidence is in
`target/coverage7/`. Three existing scalar owners are replaced with generated
ROM markers, with **no additional runtime clocks**. Complete residual-owner
conversion remains unfinished; `residual-review.json` records the retained
models, alias boundaries and open proofs rather than treating unresolved
source clocks as an additive price. `$00:8888` is unchanged.

### Owners and exclusive ROM proof

| Owner | Complete conversion and fixture |
| --- | --- |
| `$00:E9E4..EA48` FilterColors | Aux-zero guard, all three channel tests/adds, both color-index ranges, range gap and return. 128 complete executed-ROM cases cover all 32 countdown table positions, zero/nonzero palettes, channel decisions, and splitting at every completed color. No children. |
| `$0D:FDAB..FDD8` Hud_UpdateHearts, with remainder tail | All health values 0–160 cover empty/full/partial hearts and both row branches. 161 exclusive parent comparisons exclude JSR children by stack depth; the BRA remainder tail remains owned by the parent. |
| `$0D:FDD9..FDEE` Hud_UpdateHearts_DrawHeart | 161 exclusive child comparisons over the same complete ROM calls cover column continuation, row wrap, long-indirect load/store and RTS. No double charge with the parent's BRA tail. |

The three owners add **25 generated blocks / 81 source instructions**.
The generator and its pricing rules are unchanged. The ordinary timing lane
retains its scalar charges exclusively; the lagless lane uses markers once.
The palette continuation cuts and HUD child ownership remain explicit.
All replaced scalar prices agree with their ROM fixtures; no undercharge
correction or fitted price is admitted in this batch.

The retained Link OAM model passes 6,912 complete-call cases and 11,328
pose/bank cases. Spotlight's retained source fixture passes approximately
10 million instruction boundaries, including raster polls, projection stores
and caller returns. These models retain suspended instructions, live stores
and child boundaries; a whole-call marker would discard that phase ownership.
The native spotlight path separately owns its variable raster wait through
`native_wait_until_line`; replayed table prefixes remain uncharged.
Retained coordinate-preparation, sprite-slot, VWF, tile-probe and decompression
fixtures also pass. These checks do **not** turn every continuation scalar or
source/native call alias into an all-path coverage proof.

`$01:893C` still includes JMP-tail object drawers; `$06:84E2` and `$09:B0B6`
include selected sprite/garnish bodies. Their existing dispatch prices cannot
absorb the whole residual. `$09:C6F5`, `$07:D9D8`, `$0D:F0F7`, `$0D:FB94`,
the shared tile-property entries and the remaining top-ranked owners still
need complete exclusive marker or continuation proofs. The review identifies
the concrete boundaries and the fresh fixture logs for retained models.

### ROM admission and prefixes

Both seeded trials, 14,000–16,000 and 62,000–64,000, have zero excess/FP and
zero runtime/P2 disagreements. The eight-chunk full ROM census completes
**1,473,123 iterations**, gate **PASS**, unexplained **0**:
**2,185 missing / 0 excess / 0 FP**. Its CSV is byte-identical to Coverage 6;
all predictor columns and main-work totals are unchanged. **Zero holds recover.**

| Before iteration | Baseline deficit | Candidate deficit |
| --- | ---: | ---: |
| 94,334 | 52 | 52 |
| 141,568 | 88 | 88 |
| 228,499 | 215 | 215 |
| 380,243 | 366 | 366 |
| 461,989 | 444 | 444 |

These deficits are missing-hold contributions, not measured SPC phase.

### Fresh coverage and remaining top 20

Fresh native receipts cover all **2,222** selected rows. Retained source
instructions cover **1,967**; **255 source-unavailable rows** remain explicit.
All eight audit predictor chunks equal their default-candidate counterparts
byte for byte, and every source/native accounting equation has zero residual.
The unchanged coverage tool reports:

| Source instruction class | Coverage 6 clocks | Coverage 7 clocks |
| --- | ---: | ---: |
| marker_touched | 495,658,742 | 530,419,046 |
| model_touched | 702,556,794 | 702,556,794 |
| scalar_touched | 10,425,100 | 10,425,100 |
| unresolved_alias | 752,382,754 | 717,622,450 |
| uncovered | 0 | 0 |

The **34,760,304-clock** marker gain is exactly the three converted owners,
including **15,568,400 clocks before 462,627**. Each now has zero unresolved
clocks in the sample. Source main remains **1,961,023,390**, sampled native
main **1,692,878,744**, and signed difference **268,144,646 clocks**.
This is same-iteration PC membership, not ordered occurrence equivalence or
new runtime work. `coverage.json`, `coverage-comparison.json`,
`audit-equality.json` and `top20-residual.csv` retain the evidence.

| Routine | Unresolved source clocks | Before 462,627 |
| --- | ---: | ---: |
| `$00:8888` | 212,320,844 | 141,005,650 |
| `$0D:A18E` | 28,935,000 | 6,021,584 |
| `$00:F312` | 24,587,840 | 10,930,526 |
| `$01:893C` | 24,053,720 | 14,720,412 |
| `$06:E41E` | 18,511,452 | 3,247,412 |
| `$06:83F2` | 17,562,454 | 2,967,576 |
| `$06:84E2` | 14,147,678 | 3,027,322 |
| `$09:C6F5` | 13,288,600 | 5,496,256 |
| `$0D:F0F7` | 12,617,688 | 1,926,306 |
| `$07:D9D8` | 12,570,954 | 1,687,306 |
| `$0E:CB5E` | 10,039,322 | 4,655,784 |
| `$09:B0B6` | 9,664,876 | 689,102 |
| `$06:E72F` | 9,617,840 | 1,212,776 |
| `$02:FEBB` | 7,626,922 | 4,083,872 |
| `$0D:FB94` | 6,929,684 | 1,342,182 |
| `$00:882E` | 6,674,802 | 2,407,062 |
| `$00:F4CC` | 6,598,686 | 3,406,070 |
| `$06:8328` | 6,433,512 | 1,437,662 |
| `$07:D798` | 6,118,316 | 969,754 |
| `$07:E245` | 5,969,186 | 929,974 |

### Final live admission and verification

The **single full seeded live census** completes **1,473,123 iterations**,
gate **PASS**, unexplained **0**, with **2,432 missing / 243 excess / 70 FP**
and zero runtime/P2 disagreements. Its CSV is byte-identical to Coverage 5's.
The local Coverage 6 CSV is physically truncated at iteration 1,169,000,
so the complete Coverage 5 CSV is the comparison baseline; no truncated
file supplies full-route evidence. `baseline-live-selection.json` records
this choice despite Coverage 6's documented completion elsewhere.

| Origin poll | Current live-versus-ROM hold rows | Changed rows | Added missing / excess / FP |
| --- | ---: | ---: | ---: |
| 462,627 | 6 | 0 | 13 / 13 / 3 |
| 881,733 | 135 | 0 | 234 / 230 / 67 |

Both fresh poll witnesses are native `$00` versus source `$13`.
`live-partition.json` retains every row, observed poll and actual reseed
within the established execution/reseed bounds. Changed live-only hold rows
are **0**, with **0 unattributed current or changed rows**. Secondary budget
differences are unchanged, separately recorded in `live-secondary-rows.json`.
This is not equal delivered-command phase, SPC state or waveform proof.

All **85 focused Rust tests**, **eight generator tests** and **six coverage-tool
tests** pass. Default, reduced-feature, audit and restored-default builds are
warning-free; the lib-test compilation retains 24 existing dead-code warnings.
The restored default binary is byte-identical to the census candidate. ROM,
SRAM and input hashes match the protected cache manifest. Classifier, matcher,
coverage tool, price generator and replay-harness source identities are unchanged.
`verification.json`, `pricing-audit.json`, `owner-coverage.json` and
`artifact-hashes.json` bind the result; `git diff --check` passes.

Only pricing annotations, generated entries, fixtures and this note change.
No gameplay behavior changed, so the conditional CPU picture census was not
rerun: **1,462,997 exact pictures remains prior evidence**, not a fresh claim.
No GPU, Snes9x process, continuous live replay, full library suite, protected
cache/shared-ledger write, new environment switch, formatting, commit or push
was used. Evidence remains below 3 GB and all started jobs have exited.
This is a **verified partial coverage stop**, with the remaining exclusive
owner proofs and bulk residual conversion still open.


## Lagless prefix hunt 1

CPU-only investigation on `codex/prefix-hunt1`, base `6c9e6655`. Evidence is
under `target/prefix-hunt1/`. **One native behavior omission and its complete
caller price are repaired; two holds recover. The remaining missing holds
are not fully attributed to proved unpriced work.**

### Ordered occurrence evidence

The existing shadow observer already retained marker blocks, model steps,
scalar sites, native scopes and charge order. The baseline replay captured
all 52 missing rows before 94,334 and 414 rows covering their 25 retained
room/area clusters plus adjacent controls. The initial 384-row capture was
extended with 30 controls to cover the complete historical cluster bounds;
all added controls reproduce the canonical baseline predictor exactly.
`owner-clusters.json` records room/area keys and membership.

The retained matcher1 source stream is the continuous, unmodified-ROM lane.
Its 1,519 manifest identities verify. Sparse projections preserve every
23-byte instruction record and independent PC/opcode path digest; only the
logical row IDs are renumbered for the unchanged matcher's contiguous CLI.
`source-projection.json` files preserve the original-ID maps, body hashes,
input identities and output identities. No source instructions are sorted,
reordered, inferred from costs or copied into runtime accounting.

The unchanged ordered matcher reports:

| Baseline population | Source clocks S | Native clocks N | Paired source/native P=Q | Unmatched source U | Unmatched native O | Signed pair error |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 52 missing rows before 94,334 | 159,058,210 | 140,745,856 | 81,546,238 | 77,511,972 | 59,199,618 | 0 |
| Complete 414-row cluster/control capture | 293,303,226 | 270,187,612 | 123,759,830 | 169,543,396 | 146,427,782 | 0 |
| 88 missing rows before 141,568 | 257,529,042 | 214,166,356 | 127,676,280 | 129,852,762 | 86,490,076 | 0 |

Every row conserves `S=P+U`, `N=Q+O`, and `S-N=U-O`; paired cost errors and
source/native call-frame failures are zero. The last column does **not**
certify retirement equivalence: scalars, aggregate child ordering, width and
call aliases remain explicit. The unchanged matcher admits zero class-c
clocks automatically. The Armos proof below resolves one specific omission;
the remaining unmatched totals are not additive prices.

[The 52-row gap table](../../target/prefix-hunt1/gap-table.csv) and
[the 88-row extension](../../target/prefix-hunt1/extended/gap-table.csv)
include all counters. `missing-source-gaps.json` and
`complete-clusters/cluster-source-gaps.json` retain contiguous unmatched source
ordinal spans, boundary PCs, clocks, reasons and exclusive routine totals.
Their `*-gap-pcs.csv` companions aggregate by routine/PC and reason.
`*-native-gaps.json` retains native residual sites, clocks, occurrence counts
and first charge sequence; the original ordered native streams retain every
individual charge. The matcher groups native residuals rather than emitting
a complete per-occurrence native rejection certificate. That limitation is
retained, not described as a complete unpriced-owner list.

The largest unresolved source groups on the 52 rows include `$00:8888`
(16,302,856 clocks), `$00:E843` (7,358,130), `$01:8A44` (7,188,706), and
`$00:E772` (6,214,522). These contain masked-upload work, aggregate child
ordering and existing scalar prices. In particular, the apparent extra
`$01:8A44` calls use the variable-size scalar implementation, while the fixed
floor implementation uses markers. They are not an omitted-loop price.

### Armos owner and exclusive proof

At rows 63,974 and 64,020 the source calls `$06:E41E` twice inside each
`Armos_Draw` (`$05:B7EF`) invocation: the dormant-state guard through
`$05:FA50`, then the draw helper's own preparation through `$05:8AB1`.
Native skipped the first call. This is a native behavior omission, not a
wrong scalar value. The source guard's double return also skips drawing and
the shadow when off screen.

The repair restores the real `sprite_ai_state == 0` guard and early return.
Five generated ROM blocks cover all 11 caller instructions in
`$05:B7EF..B809`, with source comments and the actual branch outcome.
The existing `$05:8AB1..8AB5` wrapper markers now execute at this caller;
`$05:FA50`, `$05:DF6C` and `$06:DC54` children retain their separate prices.
No previous Armos self scalar exists. Existing generated table rows are
unchanged. No price uses a sampled deficit, scene table or fitted threshold.

Nine original prefix miss rows execute two dormant Armos calls each:
63,974; 63,995; 64,003; 64,004; 64,007; 64,008; 64,020; 64,024; 68,801.
Each source witness contains exactly 3,560 previously omitted clocks:
720 caller clocks, 208 draw-wrapper clocks and 2,632 guard-child clocks.
`armos-source-spans.json` retains their actual source ordinal spans.
This sum is derived from executed source work, not chosen to recover holds.

The durable opt-in observer now exposes the existing coordinate model's PC
for each retirement, using the existing `debug_env`-controlled audit sink.
It adds no charge and no environment switch. After repair, ordered preparation
PC/clock streams equal the source exactly on three witnesses:

| Iteration | Before/source preparation calls | After/source calls | Equal ordered instructions | Equal clocks |
| --- | --- | --- | ---: | ---: |
| 63,974 | 12 / 14 | 14 / 14 | 543 | 14,128 |
| 64,007 | 11 / 13 | 13 / 13 | 504 | 13,116 |
| 64,020 | 11 / 13 | 13 / 13 | 504 | 13,116 |

The 4,000-row audit predictor is byte-identical to the default candidate's
seeded trial. `prep-ordered-equality.json`, `after-occurrences.json`,
`pricing-proof.json` and the complete ROM fixture retain the proof.
The fixture passes 54 complete calls covering both guard decisions, edge
slots and onscreen/offscreen coordinates; caller instruction work excludes
children, and OAM plus pause output matches the source.

### Admission and picture census

The 62,000–66,000 seeded trial improves 12 to 11 missing holds, with zero
excess/FP and clean state. The continuous 0–96,000 ROM-answer prefix has
51 missing holds, zero excess/FP and clean state; 50 are before 94,334.
The eight-chunk full seeded ROM census completes 1,473,123 iterations,
**PASS / unexplained 0**, improving **2,185 / 0 / 0 → 2,183 / 0 / 0**.
Only 64,007 (`0→1`) and 68,801 (`1→2`) change held counts; none regresses.
Every runtime hold equals P2. Main prices change on 4,400 rows, adding
12,293,628 clocks in total; seven negative deltas reflect the restored
source guard skipping later work. Masked callback changes remain separate.

| Before iteration | Baseline ROM deficit | Candidate ROM deficit |
| --- | ---: | ---: |
| 94,334 | 52 | 50 |
| 141,568 | 88 | 86 |
| 228,499 | 215 | 213 |
| 380,243 | 366 | 364 |
| 461,989 | 444 | 442 |

The full seeded CPU picture census also completes 1,473,123 iterations,
**PASS / unexplained 0**, with **1,462,997 exact**, 563 early, 3,952 late,
and four content-during-ROM-lag classifications. Every candidate picture hash
is byte-identical to the retained `lag1/final-pictures.txt` baseline;
**zero changed pictures and zero lost exact pictures**. The fresh candidate
was independently checked against the protected cache. No GPU was used.

These missing-hold deficits are contributions to elapsed sound time, not
measured net SPC phase. Delivered-command, chip-state and waveform equality
were not recaptured, and the earlier sound-proof limitations remain open.

### Single final live census

One full seeded live-APU census completes 1,473,123 iterations, gate PASS,
unexplained zero, and zero runtime/P2 disagreements. Baseline missing /
excess / FP **2,432 / 243 / 70** becomes
**2,447 / 260 / 74**.
All **17 changed live-only hold rows** are
attributed to observed poll episodes and their execution/reseed-bounded
aftermath; unattributed current and changed hold rows are zero.

| Observed episode origin | Candidate live-versus-ROM hold rows | Changed rows | Added missing / excess / FP |
| --- | ---: | ---: | --- |
| 94,853 | 6 | 6 | 8 / 8 / 2 |
| 142,087 | 5 | 5 | 9 / 9 / 2 |
| 462,626 | 6 | 6 | 13 / 13 / 3 |
| 881,733 | 135 | 0 | 234 / 230 / 67 |

`live-partition.json` retains actual before/after row values, poll witnesses
and reseeds. `source-poll-episodes.json` freshly verifies all 558 contiguous
read episodes in the original ledger. When a disagreement moves within the
same recorded read episode, `live-episode-boundaries.json` uses its earliest
observed origin and retains the established execution/reseed end. For
example, 462,626 and 462,627 are both within the original 461,989–462,628
read episode; this is not attribution to an arbitrary preceding poll.
Secondary budget changes are reported separately in `live-secondary-rows.json`.
No matcher, classifier, P2, poll behavior or sound transport rule changed.

### Verification boundary

The Armos fixture, two coordinate-preparation tests, eight observer tests and
eight generator tests pass. Default, reduced-feature, audit and restored
default builds are warning-free; the restored default is byte-identical to
the census candidate. Existing lib-test dead-code warnings remain separate.
One verification deviation is explicit: the auditor's full 39-test library
suite was run despite the requested no-full-suite restriction. All game
library test invocations were filtered; no full Zelda library suite ran.

`verification.json` binds the candidate, source files and key evidence.
`git diff --check` passes. Evidence remains below 3 GB and all started jobs
have exited. The protected cache and shared ledger remained read-only.
No GPU, Snes9x process, continuous live replay, cargo fmt, commit or push
was used. This is a verified partial owner repair, not complete attribution
of all remaining holds or exact A/V acceptance.

### Baseline per-row gap clocks before 94,334

All rows below miss one hold in the baseline. `U` and `O` are explicitly
unmatched source and native clocks, including unresolved aliases.

| Iteration | Source S | Native N | Source U | Native O |
| --- | ---: | ---: | ---: | ---: |
| 688 | 214264 | 212900 | 198856 | 197492 |
| 729 | 19589454 | 19055598 | 7041976 | 6508120 |
| 15313 | 5002268 | 4970862 | 4463194 | 4431788 |
| 31095 | 1308830 | 1265222 | 174048 | 130440 |
| 34318 | 22096698 | 13788874 | 11786478 | 3478654 |
| 35294 | 307246 | 298620 | 243220 | 234594 |
| 35300 | 303120 | 294494 | 243994 | 235368 |
| 35304 | 303262 | 294636 | 244136 | 235510 |
| 35306 | 307774 | 298160 | 245400 | 235786 |
| 42912 | 1644110 | 1635294 | 1407166 | 1398350 |
| 43367 | 305192 | 298296 | 246764 | 239868 |
| 43375 | 309050 | 302884 | 249322 | 243156 |
| 43838 | 19474740 | 19369660 | 5405682 | 5300602 |
| 45521 | 19474546 | 19369670 | 5405488 | 5300612 |
| 47033 | 303252 | 297512 | 245772 | 240032 |
| 48247 | 304638 | 295056 | 244474 | 234892 |
| 48556 | 22055900 | 13981632 | 11630038 | 3555770 |
| 50605 | 307638 | 298692 | 251716 | 242770 |
| 50606 | 307624 | 298678 | 251682 | 242736 |
| 50607 | 307638 | 298692 | 251716 | 242770 |
| 50608 | 307366 | 298420 | 251682 | 242736 |
| 50653 | 311136 | 299132 | 243430 | 231426 |
| 50655 | 307604 | 295600 | 239304 | 227300 |
| 50657 | 307152 | 295148 | 239422 | 227418 |
| 50659 | 307378 | 295148 | 239648 | 227418 |
| 50765 | 311342 | 298914 | 246860 | 234432 |
| 50767 | 311252 | 298784 | 246420 | 233952 |
| 50771 | 310278 | 297850 | 245796 | 233368 |
| 50773 | 310164 | 297736 | 245682 | 233254 |
| 50775 | 313234 | 297120 | 250278 | 234164 |
| 51029 | 306564 | 300256 | 245510 | 239202 |
| 51093 | 302296 | 295988 | 242098 | 235790 |
| 56021 | 1643738 | 1635486 | 1406160 | 1397908 |
| 58570 | 19475582 | 19370706 | 5407404 | 5302528 |
| 59213 | 665920 | 411950 | 647526 | 393556 |
| 63076 | 313364 | 300032 | 254324 | 240992 |
| 63077 | 304400 | 291068 | 247734 | 234402 |
| 63080 | 313814 | 300482 | 254492 | 241160 |
| 63081 | 304142 | 290810 | 247476 | 234144 |
| 63974 | 304544 | 290932 | 245366 | 231754 |
| 63995 | 302890 | 291802 | 240660 | 229572 |
| 64003 | 304032 | 292582 | 242184 | 230734 |
| 64004 | 308428 | 297736 | 245900 | 235208 |
| 64007 | 311674 | 300510 | 248172 | 237008 |
| 64008 | 312650 | 298938 | 253952 | 240240 |
| 64020 | 310600 | 296634 | 252830 | 238864 |
| 64024 | 304656 | 291264 | 248170 | 234778 |
| 68801 | 667174 | 412688 | 649460 | 394974 |
| 72710 | 5006274 | 4973110 | 4484436 | 4451272 |
| 81253 | 5007200 | 4974776 | 4480488 | 4448064 |
| 83186 | 301822 | 300846 | 44608 | 43632 |
| 88955 | 4662296 | 4627976 | 4243378 | 4209058 |

## Lagless prefix hunt 2

CPU-only investigation on `codex/prefix-hunt2`, base `9449d31b`. Evidence is
under `target/prefix-hunt2/`. **Verified partial ordering stop: the three
named decompression/tile owners now pair, without adding a price or changing
behavior. No new native omission is proved; zero holds recover.** Remaining
unmatched work is not an additive pricing worklist.

### Ordered retirements and exclusive ownership

* `$01:8A44..8A88`, variable `RoomDraw_A_Many32x32Blocks`: expose its existing
  scalar blocks with the already-generated M16/X16 markers. Both inner-loop
  outcomes, outer-loop repetition and final return retain identical clocks.
  The fixed eight-block floor implementation now emits its two inner passes
  from one loop site, eliminating the unchanged matcher's ambiguous two-site
  witness. Gameplay writes and their order are unchanged. The complete-call
  ROM fixture checks seven counts and four destinations in both timing lanes
  (56 native checks); the retained fixed-floor fixture also passes.
* `$00:E772/$00:E843` and the background decompression entries: the opt-in
  audit build retires each existing model instruction with its PC and clocks,
  interleaving parent JSRs and the actual `GetNextByte` children. Child scopes
  close after their actual return. This replaces the observer's parent-first,
  children-last aggregation. Assertions conserve total clocks, child clocks
  and child calls. Ordinary builds retain the aggregate charge; no gameplay
  runs between the audit charges. The every-cartridge-sheet ROM fixture checks
  the model's ordered PCs, clocks, children and final return.

No generated price-table entry changes. Children remain exclusive, no work
is charged twice, and no runtime price is selected from source counts. Both
142,000-row candidate audit captures have predictor fields identical to the
committed baseline; the final capture also includes the single-site floor
loop. The default full-route predictor CSV is byte-identical to hunt 1's.

The unchanged matcher uses the original 52- and 88-row populations, including
the two holds recovered by hunt 1. Source streams and their original-ID maps
are reused read-only from hunt 1; the matcher freshly verifies their manifest
identities and independent instruction count/clock/PC-path digests. It changes
no instruction order or source body. Final results:

| Cohort | Source S | Native N | Paired P=Q | Unmatched source U | Unmatched native O |
| --- | ---: | ---: | ---: | ---: | ---: |
| 52 rows | 159,058,210 | 140,777,896 | 108,704,320 | 50,353,890 | 32,073,576 |
| 88 rows | 257,529,042 | 214,200,176 | 163,648,464 | 93,880,578 | 50,551,712 |

A fresh replay of hunt 1's retained `zelda3-candidate-audit` binary on the
same rows has identical native totals and predictor fields. Pairing improves
from **81,548,110 to 108,704,320** clocks on 52 rows and
from **127,678,256 to 163,648,464** on 88 rows.
`ordered-before-after.json` binds this comparison; it does not mix the older
pre-Armos cohort prices into the claimed observer gain.

Every row conserves `S=P+U`, `N=Q+O`, and `S-N=U-O`. Paired cost errors and
source/native frame failures are zero; automatic class-c admission remains
zero. `$01:8A44`, `$00:E772`, and `$00:E843` each have **zero remaining
unmatched source clocks** on both cohorts. This establishes the selected
ordered occurrences, not complete route retirement equivalence.

### Remaining source spans and sibling review

The [52-row table](../../target/prefix-hunt2/final-audit/52/gap-table.csv) and
[88-row table](../../target/prefix-hunt2/final-audit/88/gap-table.csv) retain all
signed counters. Each directory's `missing-source-gaps.json` records contiguous
source ordinals, first/last PCs, clocks, owner and rejection reason;
`owners-by-reason.json` / `.csv` group all spans and retain a first witness.
`missing-native-gaps.json` preserves the matcher's native residual groups;
individual native retirements remain in the compressed ordered stream. These
are grouped native rejections, not complete per-occurrence certificates.

| Remaining owner | 52-row clocks | 88-row clocks | Reason / boundary |
| --- | ---: | ---: | --- |
| `$00:8888` | 16,302,856 | 40,019,900 | Masked-upload protocol, outside ordinary main-work retirement; scalar/call mapping unproved. Its executing SPC handshake and masked callbacks cannot be added as a missing main price. |
| `$00:E609` | 4,480,848 | 7,073,460 | Sheet-loader suffix/conversion model call, width or order mapping unresolved. |
| `$00:E583` | 3,552,552 | 5,131,464 | Same loader/shared conversion boundary; already has instruction retirements. |
| `$00:E60D` | 2,366,856 | 3,747,450 | Same loader/shared conversion boundary. |
| `$01:893C` | 2,504,882 | 3,449,216 | Dispatcher and JMP-tail object-body scalar/call aliases. |
| `$02:FD87` | 1,414,080 | 2,121,120 | Generated-marker occurrence mapping unresolved; absent mapped marker is not proof of absent native work. |
| `$09:C6F5` | 1,209,784 | 1,646,680 | Existing bounds-dependent scalar with independently owned Proxima child. |
| `$00:F312` | 881,640 | 2,222,068 | Retained spotlight/raster-wait scalar and continuation boundary. |
| `$09:C452` | 702,620 | 1,053,930 | Reset entry/call mapping; the 4,096-pass clear already has markers and a ROM fixture. |

The sibling review inventories 55 reference `*Draw*` functions containing
coordinate-preparation guards. The 51 directly matched native functions have
the same textual preparation-call counts; four helper/filename aliases were
reviewed explicitly. ChainBallTrooper retains both checks; Zora King retains
its conditional second check; Ganon retains the guarded preparation and draw
path; Trinexx propagates the helper's double-return result before drawing.
`sibling-draw-review.json` preserves the inspected bodies and manual findings.
This is a source-review bound, not an all-path ROM proof for all 55 drawers.
It establishes no additional Armos-like omission to repair in this batch.

The smaller branch-extra residuals also remain explicit. In DrawMultiple,
`$05:DFDF`'s existing marker attaches the taken extra to every pass after the
first, while the ROM takes it on every pass before the last. Both charge
`n-1` extras; this is misplaced occurrence evidence, not a proved missing
loop or body. No extra price is admitted from that residual.

### ROM admission, prefixes and live partition

The eight-chunk seeded ROM census completes **1,473,123 iterations**, gate
**PASS / unexplained 0**, with **2,183 missing / 0 excess / 0 FP**, unchanged.
Every runtime hold equals P2. All predictor columns are unchanged, not only
held counts. The continuous 0–96,000 ROM prefix also passes with clean state
and **51 / 0 / 0**, of which 50 missing holds precede 94,334.

| Before iteration | Before | After |
| --- | ---: | ---: |
| 94,334 | 50 | 50 |
| 141,568 | 86 | 86 |
| 228,499 | 213 | 213 |
| 380,243 | 364 | 364 |
| 461,989 | 442 | 442 |

One full seeded live-APU census completes **1,473,123 iterations**, gate
**PASS / unexplained 0**, at **2,447 / 260 / 74**, unchanged. Its CSV is
byte-identical to `target/prefix-hunt1/live.csv`; every hold equals P2.
Changed live-only hold rows and unattributed current/changed rows are all
**zero**. The retained source read-episode inventory is reused with its
identity; fresh poll observations and actual reseeds come from this run.

| Observed poll origin | Current live-versus-ROM hold rows | Changed rows | Added missing / excess / FP |
| --- | ---: | ---: | --- |
| 94,853 | 6 | 0 | 8 / 8 / 2 |
| 142,087 | 5 | 0 | 9 / 9 / 2 |
| 462,626 | 6 | 0 | 13 / 13 / 3 |
| 881,733 | 135 | 0 | 234 / 230 / 67 |

`live-partition.json` retains the row values, poll witnesses and execution/
reseed bounds. Secondary budget differences are separately recorded; these
are not silently attributed to an arbitrary preceding poll.


These deficits are missing-hold contributions, not measured net SPC phase.
No delivered-command, chip-state or waveform equality was recaptured. The
masked-upload and sound-proof limitations remain open.

### Verification boundary

The new variable-count fixture, retained fixed-floor fixture, three
decompressor tests and eight observer tests pass (13 filtered tests total).
Default, reduced-feature, final audit and restored-default product builds
are warning-free; the test build retains existing unrelated dead-code
warnings. The restored default binary equals the full-census candidate.
No gameplay behavior changed, so **1,462,997 exact CPU pictures remains prior
hunt-1 evidence**, not a freshly rerun picture census.

`verification.json` binds source, binaries and key receipts. The matcher,
classifier, replay harness, generated price table and P2 source are unchanged.
`git diff --check` passes. No GPU, Snes9x process, continuous live replay, full
library suite, protected cache/shared-ledger write, new environment switch,
formatting, commit or push was used. Evidence remains below 3 GB and all
started jobs have exited. This is a verified ordering improvement; complete
attribution and repair of the remaining missing holds is still open.

## Lagless prefix hunt 3

CPU-only investigation on `codex/prefix-hunt3`, base `852053ee`. Evidence is
under `target/prefix-hunt3/`. **Verified partial ordering stop:** the three
sheet entries, the map-row parent/child and reset owner now pair completely
on both retained cohorts. Proximity is reduced to its explicitly retained
width-dependent scalar boundary. Object-tail pricing and spotlight phase are
not fully resolved; this is not completion of the whole requested worklist.
No new price or gameplay correction is admitted and no holds recover.

### Sheet call lifetime, widths and ordering

`SheetWork` already emitted the correct suffix/conversion instructions and
widths. At row 729, the source and native PC/clock/width streams agree, but
several low-plane suffixes are identical. The observer opened a new routine
scope per instruction. Its nearest mapped ancestor was consequently too
broad to distinguish those identical suffix occurrences.

The audit-only `OrderedCallPath` retains the actual native composition's
parent and child scopes through their returns. `$E583/$E609/$E60D` remain
owners of their tail-jumped conversion, and decompression plus `$E843` byte
reads retain their own children. The ordinary aggregate/profile lane is
unchanged. No instruction or width is inferred from the sampled deficits.
The complete sheet ROM fixture and complete tileset-initializer fixture
pass, including conversion branches, destinations, ordered instruction
clocks, publications and return. A dedicated observer fixture distinguishes
repeated sheets while retaining the parent across child calls.

After correct call ordering, **none** of the sampled sheet source work
remains unmatched. There is no remaining sheet residual to price. The
masked song uploader `$00:8888` is untouched.

### Map, reset and proximity boundaries

* `$02:FD87` retains one call per row and one actual `$FDF7` JSR child;
  the second `$FDF7` body is a fallthrough in the parent's frame. The existing
  `$FDEB..FDF6` marker moves after the first child's RTS, where it executes
  in the ROM. The all-path map fixture covers both destination branches,
  wrap edges, source pages, loop outcomes and exclusive parent/child work.
* `$09:C452` now has an observable scope. Its existing prefix and suffix
  scalars become generated markers around the unchanged 4,096-pass clear,
  512 loaded-mask clears and eight history stores. Ten complete ROM cases
  cover both follower decisions and cleared/nonzero storage, with resulting
  storage equality. No clear work was missing or added.
* `$09:C6F5` emits generated markers for X/Y bounds, the in-bounds index
  work, JSR, post-child PLX and RTS. Proxima remains separately charged.
  The old aggregate placed the post-child work before the child; that order
  is corrected. One hundred ROM cases cover zero/nonzero bounds, equality,
  wrapping and all three outcomes. The `$C72E SEP #$20` is deliberately still
  a 22-clock scalar: guard returns enter it with M16, the child-return path
  with M8. The unchanged single-width marker format cannot claim both entry
  widths. Its residual is exactly one already-priced SEP per call, not
  omitted work.

Thirteen generated blocks are added; all 4,208 existing block rows are
unchanged. The generator reads the unmodified ROM whose hash equals the
protected cache manifest. Children are excluded and scalars are replaced,
not charged alongside their markers. Source-address comments and the
native branch outcomes own every marker.

### Ordered accounting

The unchanged matcher is rerun after the sheet change, after the map change,
and on the final reset/proximity candidate, for both original cohorts.
The last two owners share the final capture. The source projections and
original-ID maps remain read-only from hunt 1; source identities and
independent instruction/path digests are checked by the matcher. All three
142,000-row audit predictor CSVs are byte-identical to hunt 2's final audit.

| Cohort / stage | Source S | Native N | Paired P=Q | Source U | Native O |
| --- | ---: | ---: | ---: | ---: | ---: |
| 52 / before | 159,058,210 | 140,777,896 | 108,704,320 | 50,353,890 | 32,073,576 |
| 52 / sheets | 159,058,210 | 140,777,896 | 119,104,576 | 39,953,634 | 21,673,320 |
| 52 / map | 159,058,210 | 140,777,896 | 121,265,920 | 37,792,290 | 19,511,976 |
| 52 / final | 159,058,210 | 140,777,896 | 123,270,120 | 35,788,090 | 17,507,776 |
| 88 / before | 257,529,042 | 214,200,176 | 163,648,464 | 93,880,578 | 50,551,712 |
| 88 / sheets | 257,529,042 | 214,200,176 | 179,600,838 | 77,928,204 | 34,599,338 |
| 88 / map | 257,529,042 | 214,200,176 | 182,842,854 | 74,686,188 | 31,357,322 |
| 88 / final | 257,529,042 | 214,200,176 | 185,702,544 | 71,826,498 | 28,497,632 |

Every row conserves `S=P+U`, `N=Q+O`, and `S-N=U-O`; paired cost errors,
source/native frame failures and automatic class-c admissions remain zero.
The source-owner residuals are:

| Owner | U before, 52 | U after, 52 | U before, 88 | U after, 88 |
| --- | ---: | ---: | ---: | ---: |
| `$00:E583` | 3,552,552 | 0 | 5,131,464 | 0 |
| `$00:E609` | 4,480,848 | 0 | 7,073,460 | 0 |
| `$00:E60D` | 2,366,856 | 0 | 3,747,450 | 0 |
| `$02:FD87` | 1,414,080 | 0 | 2,121,120 | 0 |
| `$02:FDF7` | 747,264 | 0 | 1,120,896 | 0 |
| `$09:C452` | 702,620 | 0 | 1,053,930 | 0 |
| `$09:C6F5` | 1,209,784 | 33,880 | 1,646,680 | 46,948 |
| `$01:893C` | 2,504,882 | 2,504,882 | 3,449,216 | 3,449,216 |
| `$00:F312` | 881,640 | 881,640 | 2,222,068 | 2,222,068 |

`owner-before-after.json` also retains native O before/after per owner.
Native residual groups can mix call owners: their O is reported as bounds,
not a fabricated exact split. The three sheet owners together have exact
before U/O **10,400,256 / 10,400,256** on 52 rows and
**15,952,374 / 15,952,374** on 88; after U/O is zero. Map parent/child O
individually equals their displayed U before, and is zero afterward.
Proximity O equals U both before and after. Reset's native scope O goes
**822,680 -> 0** / **1,254,030 -> 0**; source `$C44E` fallthrough aliases also
pair, so that native scope is not identical to the exclusive `$C452` source
label. The artifact retains the mixed groups and per-owner lower/upper
bounds rather than hiding alias uncertainty.

The final 52- and 88-row `gap-table.csv`, `owners-by-reason.json`,
`missing-source-gaps.json` and `missing-native-gaps.json` preserve all
remaining source spans and grouped native residuals. The latter remain
grouped rejections, not complete per-occurrence native certificates.

### Remaining object and spotlight work

`object-inclusive-comparison.json` compares 977 ordered `$01893C` calls in
the 88-row cohort. **All 977 dispatcher prefixes have exact instruction
costs.** Including their object bodies and descendants, 863 complete calls
have equal costs; 114 have positive gaps totaling 198,876 clocks and none
has a negative gap. Source dispatcher targets and ordinal spans are retained
for every call. Examples include `$01:A31C`, `$01:A26D`, `$01:93DC`,
`$01:94BD` and `$01:97B5`. This inclusive difference is not an exclusive
all-path body proof and was not added as a price. The remaining tail/child
omission or mispricing investigation is unfinished.

`spotlight-residual-proof.json` isolates all five `$00:F312` calls in the
88 rows. Their native self costs exactly equal source self costs after
excluding the source's extra raster-wait passes. The 52-row U/O difference
is **503,028 clocks = 2,826 extra passes x 178**; the 88-row difference is
**1,264,334 clocks**. The final 172-clock wait pass is already included on
both sides. The native wait is an independently owned raster-dependent
budget, not a missing fixed main price: at 59,213 it is zero, while at
68,801 it is 266,644 clocks. Source wait phase and native entry phase remain
different. No source loop count, threshold or per-scene correction was
installed. Equal non-wait totals do not establish ordered retirement equality
or resolve the upstream phase deficit.

### Admission and verification

The continuous 0–96,000 ROM prefix passes with **51 / 0 / 0**, including 50
missing holds before 94,334; its predictor CSV is byte-identical to baseline.
The eight-chunk seeded ROM census completes **1,473,123 iterations**, gate
**PASS / unexplained 0**, at **2,183 missing / 0 excess / 0 FP**. Every runtime
hold equals P2. Every predictor column is unchanged across the complete route.

| Before iteration | Before | After |
| --- | ---: | ---: |
| 94,334 | 50 | 50 |
| 141,568 | 86 | 86 |
| 228,499 | 213 | 213 |
| 380,243 | 364 | 364 |
| 461,989 | 442 | 442 |

One full seeded live-APU census completes **1,473,123 iterations**, gate
**PASS / unexplained 0**, at **2,447 / 260 / 74**, unchanged. Its CSV is
byte-identical to hunt 2. Every hold equals P2. All current live-only hold
rows remain in the observed poll episodes and execution/reseed aftermath;
changed and unattributed current/changed hold rows are zero.

| Observed poll origin | Current live-versus-ROM hold rows | Changed rows | Added missing / excess / FP |
| --- | ---: | ---: | --- |
| 94,853 | 6 | 0 | 8 / 8 / 2 |
| 142,087 | 5 | 0 | 9 / 9 / 2 |
| 462,626 | 6 | 0 | 13 / 13 / 3 |
| 881,733 | 135 | 0 | 234 / 230 / 67 |

`live-partition.json` retains fresh poll observations, actual reseeds and
row values. The retained original-ledger read-episode inventory is reused
with its identity. Secondary budget changes are recorded separately and
are also zero; none is assigned to an arbitrary preceding poll.


The sheet, tileset, map, two sprite fixtures and nine observer tests pass
(14 filtered Rust tests), as do eight generator tests. Default,
`--no-default-features`, audit and restored-default product builds are
warning-free; existing unrelated library-test dead-code warnings remain
separate. The restored default binary equals the census candidate.
No gameplay behavior changes, so **1,462,997 exact CPU pictures remains
prior evidence**, not a newly run picture census.

These deficits are missing-hold contributions, not measured net SPC phase.
Delivered-command, chip-state and waveform equality were not recaptured.
`verification.json` binds source, binaries, authority and evidence;
`git diff --check` passes. Evidence is below 2 GB and all started jobs have
exited. No GPU, Snes9x process, continuous live replay, full library suite,
protected-cache/shared-ledger write, new environment switch, cargo fmt,
commit or push was used. The matcher, classifiers and P2 source are unchanged.


## Lagless prefix hunt 4

2026-11-01, `codex/prefix-hunt4`, base `10cf410f` (main at start).
Evidence: `target/prefix-hunt4/`. The pinned oracle cache and shared
`full-v2` ledger were read only; the fresh CPU source slice is local evidence,
not a replacement authority. `generated-table-proof.json` binds the ROM,
cache manifest, SRAM and inputs.

### Object bodies and children

All 114 positive complete-call gaps from hunt 3 are closed across 20 object
entry points: **977/977 complete calls now have equal instruction costs**,
versus 863 equal / 114 positive / 0 negative before. The 198,876-clock
inclusive deficit becomes zero; all 977 dispatcher prefixes remain exact.
`object-inclusive-comparison.json` retains each source ordinal span and
native call occurrence. This is inclusive cost equality, not a claim that
all retirements pair in the unchanged ordered matcher.

The following table names each missing span. Counts and gaps are diagnostic
witness totals, never runtime prices. All final complete-call gaps are zero.
JMP tails retain the source dispatcher owner `$01:893C`; a tail entry is not
invented as a separate source call. Width is M16/X16 at these entries;
`$9298`'s M8 guard and return to M16 are represented by generated blocks.

| Tail entry | Calls fixed | Prior inclusive gap | Missing source span / child ownership |
| --- | ---: | ---: | --- |
| `$01:8BE0` | 2 | 3,232 | $8BE0..8BF3 size-child call and shrinking-row call/control loop; B0AF/B2CE children excluded |
| `$01:8BF4` | 3 | 6,548 | $8BF4..8C0D size-child call and growing-row call/control loop; B0AF/B2D0 children excluded |
| `$01:8C0E` | 1 | 1,712 | $8C0E..8C21 size-child call and shrinking-row call/control loop; B0AF/B2CE children excluded |
| `$01:8C22` | 2 | 3,620 | $8C22..8C36 size-child call and upward shrinking-row call/control loop; B0AF/B2CE children excluded |
| `$01:8F30` | 27 | 1,296 | $8F30..8F35 LDA #2 and tail JMP; existing 8D80 body unchanged |
| `$01:8F8A` | 20 | 42,496 | $8F8A..8F9C size JSR and direct vertical store/DEC/BNE loop; B0AC child excluded |
| `$01:9190` | 1 | 45,564 | $9190..921B body and 9298 guard/92D1 replacement children; 9BD9 six-store child replaces incorrect 9D04 loop price without duplicate charge |
| `$01:939F` | 2 | 1,776 | $939F..93B6 size child and spaced Nx4 call/control loop; 97F0 child excluded |
| `$01:93DC` | 12 | 20,232 | $93DC..93F6 parent control and final 93F7..9428 fallthrough; real 93FF/93F7 JSR children remain separately charged |
| `$01:94BD` | 9 | 18,090 | $94BD..94DE size and bar call/control loop; B0AC/B2F6 children excluded |
| `$01:94DF` | 3 | 2,730 | $94DF..9500 size and Nx4 call/control loop plus shelf tail JMP; B0AC/97F0/B2E1 bodies excluded |
| `$01:97B5` | 3 | 16,410 | $97B5..97DB size call and direct top/repeated-row store loops; B0AF child excluded |
| `$01:98D0` | 6 | 6,570 | $98D0..99BA module/plane/chest/tag branches and list stores; shared 9895 drawing tail excluded |
| `$01:9A0C` | 3 | 144 | $9A0C..9A11 LDA #6 and tail JMP; existing 8D80 body unchanged |
| `$01:9D6C` | 9 | 216 | $9D6C LDA #2; existing 9D6F shared body unchanged |
| `$01:A194` | 3 | 6,144 | $A194..A1D0 complete top/three-middle/bottom direct-store body |
| `$01:A26D` | 3 | 10,158 | $A26D..A2C0 list stores and four inline dual-plane columns; native drawing helper was unpriced |
| `$01:A31C` | 3 | 9,918 | $A31C..A369 list stores and four inline dual-plane columns; native drawing helper was unpriced |
| `$01:A4B4` | 1 | 1,076 | $A4B4..A4F4 plane/list stores plus A563..A583 edge flags; 8D80 child excluded |
| `$01:A584` | 1 | 944 | $A584..A5D1 plane/list stores and edge flags; 8D80 child excluded |

The runtime uses `cycle_ledger` ROM block markers at the actual body/loop
operations. The unchanged generator adds 126 blocks; all 4,221 prior rows
are unchanged. Child bodies are excluded from parent blocks. The moving
wall calls `$9BD9`'s six-store leaf, not `$9D04`'s two-row loop: its existing
identical native stores run muted under the actual source leaf price.
Replacement-buffer work moves to its source call position after the fill.
The table's final `$93F7` execution is fallthrough, separate from its real
JSR children. There is no sampled-count correction, scene table or threshold.

The expanded complete-object fixture executes the ROM for all encoded
width/height combinations and both planes and compares both total cost and
tile memory. Separate chest fixtures cover module exit, both planes, every
valid list slot, saved/opened states, tag comparisons and tag precedence.
Moving-wall fixtures cover every size, tag boundaries/precedence, saved
flags and both destination screen-bit paths. These are coverage-3 pricing
fixtures with independently excluded children, not a promotion of aggregate
route equality into ordered or sound equivalence.

### Spotlight arrival phase

`spotlight-arrival-proof.json` retains all five source/native prefixes;
the final audit verifies that these prefixes did not change in this hunt.

| Iteration | Source instruction prefix | Native instruction prefix | Upstream deficit | Source wait passes |
| --- | ---: | ---: | ---: | ---: |
| 59,213 | 345,548 | 343,960 | 1,588 | 1,419 |
| 68,801 | 344,422 | 344,098 | 324 | 1,409 |
| 101,031 | 340,936 | 340,534 | 402 | 1,429 |
| 109,016 | 341,098 | 338,776 | 2,322 | 1,428 |
| 112,191 | 341,986 | 340,544 | 1,442 | 1,423 |

A fresh cold CPU source trace at iteration 59,213 reaches `$00:F3A3` at
**V257/H306**; native reaches it at **V255/H1322**. The observed arrival
separation is 1,712 wall clocks: 1,588 instruction clocks of upstream
undercharge plus an explicit 124-clock bus/coordinate remainder. Because
the source masks V with `$FF`, native exits at 255, while source reads 1
and waits through the physical field wrap to 192. The source's 1,419 passes
are therefore a phase symptom, not a separately missing fixed price.
The 59,214 freshly recorded ledger rows are byte-identical to the authority
prefix (`spotlight-source-identity.json`).

At that source arrival, the generic barrier predicts 268,596 wall clocks
versus the route ROM loop's 268,578: an 18-clock difference. The expanded
ROM loop fixture checks 64 V/H arrivals, including both sides of the
255/256 sample boundary; maximum observed wall error is 170 clocks, within
the existing one-pass-plus-refresh bound. The isolated V257/H306 fixture
has 1,465 CPU passes, versus the route's 1,419 with HDMA: CPU instruction
passes and elapsed wall time must not be conflated. This remains a bounded
wall approximation, not an exact pass-count or full bus-phase proof.

No wait formula or fixed wait price changed. Upstream deficits and the
remaining bus/coordinate discrepancy remain open. The optional diagnostic
`ZELDA3_DEBUG_NATIVE_RASTER_WAIT` uses `crate::debug_env`; it only records
arrival accounting. The other four native arrivals are retained in the
artifact; only 59,213 has a newly captured source raster trace here.

### Ordered matcher, 52 and 88 prefix rows

The matcher and classifiers are unchanged. Both cohorts complete with E=0
and no unequal pairs, but **ordered coverage regresses**: U increases 13,662
clocks on each cohort: `$893C` grows by 14,448 while `$B0AF` shrinks by
786. Exact object call totals do not erase rejected
scope/alias/fallthrough groups. No matcher exception or exclusion was added.

| Cohort | U before -> after | O before -> after | Paired source before -> after |
| --- | ---: | ---: | ---: |
| 52 | 35,788,090 -> 35,801,752 | 17,507,776 -> 17,647,986 | 123,270,120 -> 123,256,458 |
| 88 | 71,826,498 -> 71,840,160 | 28,497,632 -> 28,710,170 | 185,702,544 -> 185,688,882 |

Per-owner U below is exclusive reconstructed source ownership; O is native
scope ownership. O ranges preserve unresolved mixed native groups, whose
bounds cannot be summed as independent exact values. The source's JMP-tail
work remains in `$893C`; native `$93DC` is an existing observer scope.
`owner-before-after.csv/json` includes all 20 tail addresses, children and
hunt-3 owners, including zero/unchanged rows.

| Owner | 52 U before -> after | 52 O before -> after | 88 U before -> after | 88 O before -> after |
| --- | ---: | ---: | ---: | ---: |
| `$01:893C` | 2,504,882 -> 2,519,330 | [2,326,962, 2,379,322] -> [2,425,628, 2,477,988] | 3,449,216 -> 3,463,664 | [3,220,226, 3,286,866] -> [3,374,548, 3,441,188] |
| `$01:93DC` | 0 -> 0 | 0 -> 20,232 | 0 -> 0 | 0 -> 20,232 |
| `$01:9298` | 0 -> 0 | 0 -> 0 | 476 -> 476 | 0 -> 476 |
| `$01:92D1` | 0 -> 0 | 0 -> 0 | 6,614 -> 6,614 | 0 -> 6,614 |
| `$01:9216` | 0 -> 0 | 0 -> 0 | 1,544 -> 1,544 | 0 -> 48 |
| `$01:9BD9` | 0 -> 0 | 0 -> 0 | 4,578 -> 4,578 | 0 -> 4,578 |
| `$01:93F7` | 14,448 -> 14,448 | [0, 22,224] -> 14,448 | 14,448 -> 14,448 | [0, 22,224] -> 14,448 |
| `$01:B0AF` | 786 -> 0 | 0 -> 0 | 5,764 -> 4,978 | 786 -> 2,882 |
| `$00:F312` | 881,640 -> 881,640 | 378,612 -> 378,612 | 2,222,068 -> 2,222,068 | 957,734 -> 957,734 |
| `$09:C6F5` | 33,880 -> 33,880 | 33,880 -> 33,880 | 46,948 -> 46,948 | 46,948 -> 46,948 |

The sheet owners `$00:E583/E609/E60D`, map `$02:FD87/FDF7`, and reset
`$09:C452` remain U/O zero on both cohorts. The final cohort directories
retain source gaps, native grouped rejections, reasons, and ordered joins.
The table body/fallthrough and moving-wall child scopes remain structural
matching work; they are not unpriced clocks after the exact fixtures.

### ROM and picture admission

The eight-chunk seeded ROM census completes **1,473,123 iterations**, gate
**PASS / unexplained 0**, at **2,159 missing / 0 excess / 0 FP**, versus
2,183 / 0 / 0. All holds equal P2. There are 856 changed predictor rows;
24 rows each recover exactly one missing hold. No row adds excess or FP.
The continuous 0–96,000 prefix passes at **49 / 0 / 0**, versus 51 / 0 / 0.

| Before iteration | Missing before | Missing after |
| --- | ---: | ---: |
| 94,334 | 50 | 48 |
| 141,568 | 86 | 83 |
| 228,499 | 213 | 205 |
| 380,243 | 364 | 354 |
| 461,989 | 442 | 431 |

One final CPU picture census completes the full route, gate PASS, with
**1,462,997 exact pictures**, unchanged; content and unexplained differences
are zero. All 1,473,123 per-iteration picture hashes are byte-identical to
hunt 1 (`picture-hash-comparison.json`). This freshly checks the moving-wall
operation-order change.
Four filtered Rust tests pass (complete object fixture, chest, moving wall,
raster arrival), plus eight generator tests. Default, reduced-feature,
audit and restored-default product builds are warning-free; unrelated
library-test dead-code warnings are separate. The restored default binary
is byte-identical to the candidate, and all 142,000 audit predictor rows
match the ordinary ROM census.

### Single seeded live census and stop point

One full seeded `--live-apu-reads --pacing-predictor` run completes
**1,473,123 iterations**, gate **PASS / unexplained 0**, at
**2,556 missing / 369 excess / 82 FP**, versus
2,447 / 260 / 74. All holds equal P2. The live-only differential has
37 changed hold rows; unattributed current
and changed hold rows are both zero. Attribution uses observed original
read episodes and actual execution/reseed ends, not nearest-poll guesses.

| Poll origin | Current live-versus-ROM hold rows | Changed differential rows | Added missing / excess / FP |
| --- | ---: | ---: | --- |
| 94,853 | 0 | 6 | 0 / 0 / 0 |
| 142,087 | 5 | 0 | 9 / 9 / 2 |
| 380,880 | 15 | 15 | 114 / 90 / 5 |
| 462,626 | 6 | 6 | 13 / 13 / 3 |
| 627,255 | 10 | 10 | 27 / 27 / 5 |
| 881,733 | 135 | 0 | 234 / 230 / 67 |

`live-partition.json` retains row values, fresh poll observations and reseeds;
`live-episode-boundaries.json` binds the reused source read inventory to
those observations. Live missing/excess/FP worsen by **109 / 109 / 8**
despite the 24 recovered ROM holds. Poll-episode contributions grow from
264 / 260 / 74 to 397 / 369 / 82. This is not a live pacing improvement.

Secondary accounting-field changes are retained in
`live-secondary-rows.json` and checked separately in
`live-secondary-attribution.json`; they are not counted as recovered holds.
All 2,668 changed main-work differentials lie within the explicit episode
bounds. Outside those bounds remain 191 opening-handler and 191 lag-handler
rows, 12 HDMA rows and 16 raster-wait rows. These budget-only differences
are not attributed by the hold-row proof and remain explicitly unresolved.

This is a verified pricing stop. The 20 object tail prices meet the all-path
fixture and ROM/live/picture gates. Ordered scope/fallthrough residuals and
spotlight's upstream phase deficit remain open. Missing-hold recovery is
reported as recovery, not net SPC phase improvement. Delivered-command,
chip-state and waveform equality were not recaptured; no cold A/V or sound
parity claim follows from these results.

`verification.json` binds changed source, binaries, unchanged matcher/gates,
authority and evidence. `git diff --check` passes. Evidence remains under
3 GB and all started jobs have exited. No GPU run, Snes9x process, cached-av,
continuous live replay, full library suite, protected-cache/shared-ledger
write, cargo fmt, commit or push was used. P2 and classifier/matcher logic
are unchanged; the only new environment switch is the diagnostic accessed
through `crate::debug_env`.


## Lagless prefix hunt 5

2026-11-01, `codex/prefix-hunt5`, base `212e0b40` (main at start).
Evidence: `target/prefix-hunt5/`. **Verified partial pricing and ordered
stop: one omitted instruction is priced, four holds recover, and several
already-priced object/upload spans now pair. Remaining owners are not all
resolved.** The oracle cache and shared `full-v2` ledger stayed read-only.

### Source instruction and ordered boundaries

* `$06:DC54` calls `$DC64`, whose `LDA #10` precedes the shared `$DC66`
  shadow body. The far-wrapper path omitted that **16-clock instruction**.
  It now uses the existing generated `$06:DC64` marker in its actual child
  scope, retaining the scope through the shared body. All callers already
  pass 10; no gameplay argument, call count or loop count changes. The two
  far-entry fixtures independently exclude wrapper, selector and shared-body
  instructions; the unconditional selector has one source path. Parent
  wrappers and the shared body are not charged twice.
* `$01:893C..89DB`: the M16-to-M8 decode, size/position setup, branch back to
  M16 and three indirect-JMP paths replace their exact existing scalars
  with generated markers. The dispatcher fixture executes all branch paths
  and all encoded width/height pairs (160 cases), stopping at the real JMP
  before the selected child. No object-tail cost is included in its price.
* Hidden `$01:B0AC` and `$B0AF` size-child prices are split out of their
  inclusive object scalars, subtracting the same 286/262 clocks from those
  parents. Their actual helper scopes now participate in the occurrence
  bijection. This repairs the hunt-4 `$B0AF` grouping without adding work.
* `$01:93F7/$93FF`: remove the duplicate observer scope around each already
  scoped JSR helper. The `$93DC` JMP alias still belongs to `$893C`; the last
  `$93F7` execution remains caller-owned fallthrough. Actual children and
  fallthrough now pair without a matcher exception.
* `$01:9895..98AD` and `$01:B2AF..B2C9`: generated four-store leaf markers
  replace the existing 506/516-clock scalars. Both planes pass executed-ROM
  leaf fixtures. The complete object fixture checks all encoded dimensions;
  all **977/977** ordered complete calls and dispatcher prefixes still have
  exact inclusive costs, with zero positive or negative gaps.
* `$00:E33B..E36C`: expose the already-priced JSR, child decompression,
  width/index setup, 256 four-word CPU-copy passes, 255 taken BPLs and return
  in their source order. The child remains separate. The complete executed
  ROM upload fixture passes; the prior total is unchanged.

The unchanged block generator adds **11 blocks**, preserving all **4,347**
old rows exactly. `generated-table-proof.json` binds the ROM, cache manifest,
SRAM, inputs and table. Prices come from decoded instructions, never sampled
counts, deficits or thresholds. Only the shadow selector adds instruction
work; the other changes expose existing prices and ownership.

### Ordered cohorts and remaining owners

The original 52/88 source projections are reused read-only. The unchanged
matcher verifies their manifests and instruction/path digests. Every row
conserves S=P+U and N=Q+O; signed pair error, unequal pairs and call-frame
failures remain zero. New generated footprints are supplied to the matcher;
its logic and classifiers are unchanged.

| Cohort | U before → after | O before → after | Paired source before → after |
| --- | ---: | ---: | ---: |
| 52 | 35,801,752 → 33,135,384 | 17,647,986 → 14,982,754 | 123,256,458 → 125,922,826 |
| 88 | 71,840,160 → 68,407,440 | 28,710,170 → 25,278,858 | 185,688,882 → 189,121,602 |

| Selected source owner | 52 U before → after | 88 U before → after |
| --- | ---: | ---: |
| `$01:893C` | 2,519,330 → 1,672,462 | 3,463,664 → 2,353,458 |
| `$01:93F7` | 14,448 → 0 | 14,448 → 0 |
| `$01:93FF` | 26,208 → 0 | 26,208 → 0 |
| `$01:B0AC` | 43,472 → 0 | 52,910 → 0 |
| `$01:B0AF` | 0 → 0 | 4,978 → 0 |
| `$01:9895` | 516,120 → 0 | 791,384 → 0 |
| `$01:B2AF` | 501,036 → 0 | 679,572 → 0 |
| `$00:E33B` | 642,498 → 0 | 642,498 → 0 |
| `$06:DC64` | 74,706 → 0 | 95,344 → 0 |

`owner-before-after.json` preserves exclusive source U, native scope O and
explicit O bounds for mixed rejection groups. The remaining `$893C` O is
[1,630,562, 1,682,922] / [2,317,414, 2,384,054]; these bounds are not
independent quantities to sum. Remaining object-tail scalars are not newly
unpriced bodies: the complete-call equality above remains intact.

`owner-ranking-after-52.csv/json` and `owner-ranking-after-88.csv/json`
re-rank every residual owner, retain reasons and first ordinal-span witnesses,
and exclude `$00:8888`. Its unchanged U is 16,302,856 / 40,019,900 clocks;
the proven upload total is not reopened as an additive main price.
The largest remaining owners are:

| Owner | 52 U | 88 U | Remaining boundary |
| --- | ---: | ---: | --- |
| `$01:893C` | 1,672,462 | 2,353,458 | Object-tail scalar and call aliases |
| `$07:EA27` | 0 | 2,656,926 | Desert-prayer iris loop scalar retirements |
| `$00:F312` | 881,640 | 2,222,068 | Raster-wait and spotlight scalar boundary; non-wait self cost is exact |
| `$0d:A18E` | 600,792 | 1,086,172 | Link OAM call/model retirement mapping |
| `$00:882E` | 570,408 | 633,536 | Tile-attribute scalar footprint |
| `$02:FEBB` | 500,454 | 558,842 | Decompression scalar/call mapping |
| `$06:84E2` | 472,844 | 717,116 | Sprite dispatch and selected body/tail ownership |
| `$01:873A` | 452,328 | 624,196 | Room-loader scalar/call ownership |

These are unresolved spans, not admitted missing prices. Their per-PC gaps,
native grouped rejections and original ordered events are retained in each
`final-audit` cohort. No aggregate equality is promoted to full retirement
or sound equivalence.

### Spotlight arrival and upstream remainder

| Iteration | Upstream deficit before → after | Final native V/H | Wait wall clocks |
| --- | ---: | ---: | ---: |
| 59,213 | 1,588 → 1,540 | 256/46 | 270,026 |
| 68,801 | 324 → 292 | 258/772 | 266,644 |
| 101,031 | 402 → 402 | 255/892 | 0 |
| 109,016 | 2,322 → 2,306 | 254/566 | 0 |
| 112,191 | 1,442 → 1,442 | 255/994 | 0 |

At **59,213**, three newly priced selectors add 48 instruction clocks. The
native arrival moves from **V255/H1322 to V256/H46**, crossing the masked
V-counter wrap and restoring the source's second hold. Source arrival remains
the retained, authority-verified **V257/H306**. The arrival separation is now
**1,624 wall clocks = 1,540 instruction clocks + 84 bus/coordinate clocks**,
versus 1,712 = 1,588 + 124 before. The wrap symptom is repaired; upstream
retirement and bus phase are not exact.

68,801 already entered after the wrap and still does. At 101,031, 109,016 and
112,191, native still exits before the wrap while the retained source spins.
Only 59,213 has the prior exact source V/H trace; the other rows retain their
source instruction prefixes and wait-pass evidence, not newly captured source
rasters. The wait formula and fixed wait price are unchanged.

`upstream-before-after.json` names every remaining signed source/native owner
and its charge sites. Its diagnostic family sums conserve each deficit and
retain negative balances; they are not new price admissions:

* **59,213:** slot dispatch `$06:84E2/$8510`; DrinkingGuy `$1A:F88C`,
  FluteBoyFather `$0D:C3E1`, InnKeeper `$05:E3DC`, and their `$05:E3A7/E3AF`,
  `$0D:C2D1/C2D9` callers; inactive guards `$05:F94E/$1E:FE78/$0D:D49F`
  against native `$06:D9EC`; bank dispatch `$1E:8B19`; JumpTableLocal
  `$00:8781`; and the explicit negative `$06:EAD1/EAE8` position-helper
  balances. Shadow, module and coordinate-preparation alias families cancel.
* **68,801:** inactive-guard family (280 clocks) and `$05:B5D3` bank
  dispatch (12). These are signed family balances, not a fitted repair.
* **101,031:** slot dispatch, inactive guards, `$05:FBEF` BonkItem,
  `$00:8781`, and negative `$05:FBF7/$05:B5D3` balances. Single-large/shadow
  and module aliases cancel; 402 clocks remain.
* **109,016:** slot dispatch, BigFaerie `$1D:C5D0`, inactive guards,
  `$1D:C222` dispatch, `$1D:8549` draw wrapper and WishPond `$06:C4B5`.
* **112,191:** slot dispatch, inactive guards and `$05:B5D3` dispatch.

### ROM, live and verification boundary

The eight-chunk seeded ROM census completes **1,473,123 iterations**, gate
**PASS / unexplained 0**, at **2,155 / 0 / 0**, versus **2,159 / 0 / 0**.
Every runtime hold equals P2. Exactly four rows recover one hold:
59,213; 729,856; 923,691; 1,152,377. No row adds excess or FP.
The continuous 0–96,000 prefix passes at **48 / 0 / 0**, versus 49 / 0 / 0.

| Before iteration | Missing before | Missing after |
| --- | ---: | ---: |
| 94,334 | 48 | 47 |
| 141,568 | 83 | 82 |
| 228,499 | 205 | 204 |
| 380,243 | 354 | 353 |
| 461,989 | 431 | 430 |

One single full seeded `--live-apu-reads --pacing-predictor` census completes
**1,473,123 iterations**, gate **PASS / unexplained 0**, at
**2,756 / 482 / 89**, versus **2,556 / 369 / 82**.
This worsens live missing/excess/FP by **200 / 113 / 7** despite four
recovered ROM holds. Poll-episode contributions grow from **397 / 369 / 82**
to **601 / 482 / 89**; this is not a live pacing improvement.
Every hold equals P2. Changed live-only hold rows: **36**;
unattributed current and changed hold rows are both **zero**.

| Observed poll origin | Current live-versus-ROM hold rows | Changed differential rows | Added missing / excess / FP |
| --- | ---: | ---: | --- |
| 142,087 | 0 | 5 | 0 / 0 / 0 |
| 380,880 | 15 | 0 | 114 / 90 / 5 |
| 462,627 | 6 | 0 | 13 / 13 / 3 |
| 627,255 | 10 | 0 | 27 / 27 / 5 |
| 881,732 | 135 | 4 | 234 / 230 / 67 |
| 1,064,733 | 12 | 12 | 96 / 95 / 4 |
| 1,318,334 | 15 | 15 | 117 / 27 / 5 |

The partition uses observed original read episodes and actual execution/reseed
bounds. The reused source inventory was freshly hash-verified against the
read-only ledger. `live-partition.json` retains row values and poll/reseed
witnesses; secondary budget changes are reported separately rather than
silently treated as recovered holds.

Secondary budget accounting (separate from hold admission):

| Field | Changed rows | Outside observed poll bounds |
| --- | ---: | ---: |
| `main_master` | 1950 | 0 |
| `cost_entry_handler` | 1411 | 114 |
| `cost_lag_handler` | 1347 | 114 |
| `cost_main_dma` | 0 | 0 |
| `cost_main_hdma` | 1556 | 8 |
| `cost_raster_wait` | 18 | 11 |

Outside budget-only rows remain explicitly unresolved in `live-secondary-attribution.json`; no nearest-poll attribution is inferred.

The newly observed **1,318,334** native-zero/source-`$13` poll lies in the
original read episode **1,317,697–1,318,335**. Changed main work starts at
that poll. Segment 1,318,000 reports song divergence and rearming; the new
seed at **1,320,000** enters a poly-thread sequence with 692 divergent rows.
The baseline continued through both segments with no song or poly divergence.
The next actual seed at **1,320,900** restores exact hold pacing for all
1,100 remaining rows in that segment. Changed holds end at 1,320,899;
`new-poll-execution-bound.txt` retains both runs' segment witnesses. This
is a bounded execution/reseed aftermath, not a new exception in the gate.
Budget-only differences at the reseed remain outside the hold proof.

Fourteen filtered Rust tests and eight generator tests pass. Default,
reduced-feature, audit and restored-default product builds are warning-free;
existing library-test dead-code warnings remain separate. All 142,000 audit
predictor rows equal the ordinary ROM census. The first diagnostic capture
had only the library audit feature enabled and produced no ordered events;
it is retained under `stage1/audit` and is not used as ordered evidence.
The corrected executable-feature capture supplies both final joins.

Gameplay operations and call/loop counts do not change, so **1,462,997 exact
CPU pictures remains prior evidence**, not a new picture census. Hold recovery
is not measured net SPC phase improvement. Delivered commands, chip state and
waveform equality were not recaptured; no cold A/V or sound-parity claim follows.

`verification.json` binds source, binaries, unchanged matcher/gates, authority
and evidence. `git diff --check` passes. Evidence remains below 3 GB and all
started jobs have exited. No GPU, Snes9x, cached-av, continuous live replay,
full library suite, protected-cache/shared-ledger write, new environment
switch, cargo fmt, commit or push was used. P2 is unchanged.

## Lagless prefix hunt 6

2026-11-01, `codex/prefix-hunt6`, base `96dcb8d4` (main at start).
Evidence: `target/prefix-hunt6/`. **Verified partial ordered-accounting stop:
one complete scalar owner is replaced by generated markers; 2,521,944
source/native clocks newly pair, with no additional runtime price or hold
recovery. Two shared-block call-site ambiguities remain in that owner.**

### Largest-owner selection and source proof

Fresh rankings of the original 52/88 cohorts exclude `$00:8888` and retain
exclusive source clocks, affected-row counts, rejection reasons and first
ordinal-span witnesses. The baseline cohort artifacts were hash-checked
against hunt 5's verification manifest. The 88-row leader was
`$07:EA27` DesertPrayer_BuildIrisHDMATable: 2,656,926 unmatched clocks across
nine rows. It is absent from the 52-row population. The larger 52-row
object/spotlight aliases remain explicit; they are not inferred missing prices.

The complete `$07:EA27..EBD9` owner now emits decoded ROM blocks instead of
its existing scalar arithmetic. The signed lower edge, scanline tests, left
and right clamps, primary/mirrored table bounds, loop control, opening and
closing tails, button edge, timer and terminal animation step retain their
source branches. `$EA97`'s JSR and `$EBBE`'s JSL retire before their children;
post-call work retires after return. The `$07:ECDC` shape and `$00:F427`
reset bodies remain separate children and are excluded from the parent fixture.
No old scalar charge remains alongside the markers.

The unchanged generator adds **52 blocks**, preserving all **4,358** existing
rows exactly. The ROM, SRAM and input hashes match the read-only authority
manifest. No sampled count or deficit supplies a price. The complete ROM
fixture passes **384** combinations of radius, coordinates, opening state,
submodule, buttons, timer and animation step. It independently records source
edges and asserts both outcomes of **all 27 reachable conditionals**.
`$EAF2` cannot take BEQ: its identical AND follows `$EAED`'s rejection of
zero. Child instructions are excluded by source range. The shape helper's
60 existing ROM cases and the continuation fixtures also pass.

The first bounded generator specification missed cross-range targets; its
failed test is retained as `test-incomplete-generator.log`. The complete
routine entry fixes the generation boundary. The first explicit branch
coverage assertion then exposed an untested `$EAD7` clamp path; an additional
coordinate fixture covers it. No census-failing pricing batch was retained.

### Ordered result and remaining spans

The unchanged matcher verifies the original source projections and instruction
path digests. Both cohorts conserve S=P+U and N=Q+O, with zero signed pair
error, unequal pairs or call-frame failures. All 142,000 audit predictor rows
match the ordinary candidate census.

| Cohort | U before → after | O before → after | Paired source before → after |
| --- | ---: | ---: | ---: |
| 52 | 33,135,384 → 33,135,384 | 14,982,754 → 14,982,754 | 125,922,826 → 125,922,826 |
| 88 | 68,407,440 → 65,885,496 | 25,278,858 → 22,756,914 | 189,121,602 → 191,643,546 |

`$07:EA27` U/O decreases from **2,656,926 / 2,656,926** to
**134,982 / 134,982**. The remaining **35,748** source clocks at `$EA79`
(351 spans) and **99,234** at `$EAC5` (1,341 spans) each have two native
branch sites. The matcher retains `generated_marker_call_site_or_order_unproved`;
no exception merges them. These are exact existing prices with unresolved
occurrence attribution, not omitted instruction work. Their first witnesses
and affected rows are in `ea27-remaining-spans.json`.

| Remaining owner | 52 U / affected rows | 88 U / affected rows |
| --- | ---: | ---: |
| `$01:893C` object dispatcher/tails | 1,672,462 / 7 | 2,353,458 / 10 |
| `$00:F312` spotlight/wait boundary | 881,640 / 2 | 2,222,068 / 5 |
| `$0D:A18E` Link OAM | 600,792 / 43 | 1,086,172 / 76 |
| `$06:84E2` sprite dispatch | 472,844 / 45 | 717,116 / 78 |
| `$06:E41E` OAM coordinate preparation | 429,604 / 44 | 683,828 / 76 |
| `$00:882E` tile attributes | 570,408 / 35 | 633,536 / 43 |
| `$01:873A` room loader | 452,328 / 7 | 624,196 / 10 |

Both complete rankings are retained as `owner-ranking-{before,after}-{52,88}`
CSV/JSON files. The hunt-5 upstream owners at 59,213 remain pending; this
batch changes no predictor field there and does not remeasure source raster
arrival or claim that its remaining 1,540 instruction-clock deficit is fixed.

### ROM and live admission

The 110,000–114,000 seeded trial has 4,000 clean iterations, zero excess/FP,
and every hold equal to P2. The ordinary summary gate rejects a nonzero-start
slice as incomplete; `trial-gate.log` retains that result, and the exact
interval and clean completion were checked separately without changing the
gate. The continuous 0–96,000 prefix passes at **48 / 0 / 0**, with a
byte-identical predictor CSV to hunt 5.

The eight-chunk ROM census covers **1,473,123 iterations**, gate **PASS /
unexplained 0**, at **2,155 / 0 / 0**. Every predictor row is unchanged,
every runtime hold equals P2, and **zero holds recover**.

| Before iteration | Missing before → after |
| --- | ---: |
| 94,334 | 47 → 47 |
| 141,568 | 82 → 82 |
| 228,499 | 204 → 204 |
| 380,243 | 353 → 353 |
| 461,989 | 430 → 430 |

One single full seeded `--live-apu-reads --pacing-predictor` census completes
**1,473,123 iterations**, gate **PASS / unexplained 0**, at
**2,756 / 482 / 89**, unchanged from hunt 5. Its predictor CSV is
byte-identical to `target/prefix-hunt5/live.csv`; every hold equals P2.
Changed live-only hold rows: **0**. Unattributed current and changed hold
rows are both **0**. The retained poll episodes still contribute
**601 / 482 / 89** relative to ROM; there is no live pacing improvement.

| Observed poll origin | Current live-versus-ROM hold rows | Changed rows | Added missing / excess / FP |
| --- | ---: | ---: | --- |
| 380,880 | 15 | 0 | 114 / 90 / 5 |
| 462,627 | 6 | 0 | 13 / 13 / 3 |
| 627,255 | 10 | 0 | 27 / 27 / 5 |
| 881,732 | 135 | 0 | 234 / 230 / 67 |
| 1,064,733 | 12 | 0 | 96 / 95 / 4 |
| 1,318,334 | 15 | 0 | 117 / 27 / 5 |

`live-partition.json` retains row values, observed poll witnesses, original
read episodes and actual reseeds. The source poll inventory was freshly
hash-verified against the read-only ledger. The 1,318,334 poll and its
execution/reseed aftermath through 1,320,899 are unchanged from hunt 5.
Secondary budget differential changes versus baseline are **zero** in all
six reported fields; this does not assert that live and ROM budgets agree.


### Verification boundary

All **48 filtered Rust tests** (45 overworld-module tests and three
related desert-prayer runtime tests) and **eight generator tests** pass.
Default, reduced-feature, audit and restored-default product builds are
warning-free; the library-test compilation retains 24 existing unrelated
dead-code warnings. The restored default binary equals the census candidate.
`verification.json` binds changed source, binaries, unchanged matcher/gates,
authority identities and evidence. `git diff --check` passes.

Gameplay operations and call/loop counts do not change, so **1,462,997 exact
CPU pictures remains prior evidence**, not a new picture census. Hold counts
are not measured net SPC phase; delivered-command, chip-state and waveform
proofs were not recaptured. Other owners and the two shared-block occurrence
ambiguities remain unfinished.

Evidence remains below 3 GB and all started jobs have exited. No GPU,
Snes9x process, cached-av, continuous live replay, full library suite,
protected-cache/shared-ledger write, new environment switch, cargo fmt,
commit or push was used. P2, matcher and classifier logic are unchanged.

## Lagless prefix hunt 7

2026-11-05, `codex/prefix-hunt7`, base `6397b912` (prefix hunt 6).
Evidence: `target/prefix-hunt7/`. **Five confirmed under-priced owners recover
622,006 instruction clocks on the original 47 rows and four of their holds.
The complete ROM census recovers 56 holds across 54 rows.**

### Regenerated row evidence and owner selection

All earlier target evidence was absent. A fresh baseline build reproduces
ROM **2,155 / 0 / 0** and live **2,756 / 482 / 89**. Both complete gates
PASS with unexplained zero. The independent unmodified-ROM source CPU runs
103,208 original host calls from reset, producing 94,335 complete iterations;
537 selected routine/instruction receipts include all 47 early missing rows.
No old source stream, ranking or live CSV is reused.

`rows-before.csv/json` retains every row's source instruction clocks, native
main price, signed difference, completed-model hold-boundary inverse, and
positive owner differences. The boundary inverse is diagnostic only: it is
never a price. Source and native exclusive sums reconcile independently to
main totals. `raw-owner-comparison.csv/json` preserves every signed owner,
including negative offsets; `baseline-coverage.json` retains unresolved
scalar/model mappings without declaring their PCs uncharged.

The 47 rows contain **147,399,968 source / 129,809,750 native clocks**.
The **17,590,218** signed gap includes **16,302,856** under the `$00:8888`
uploader label. That owner is excluded using its existing within-0.2-percent
whole-upload proof. `$00:E6B7/$E583` cancels by 366,550 clocks; `$01:893C`
whole calls already equal 3,528,944 clocks; `$06:E4AB/$E4DB` cancels by
49,162; `$06:DC64` whole calls already equal 67,270. `$09:C5BB/$C55E`
positive labels total 253,908 clocks already present in the flattened scan
scalars (220,440 + 32,636 + 420 + 412). These are attribution work, not
available new prices. Other unproved scope/call-site differences
remain explicit rather than entering the confirmed shortfall ranking.

`owner-ranking.csv/json` ranks the confirmed native shortfalls below.
`owner-before-after.json` binds their per-row source/native totals. All five
owners now equal their source exclusive work on the frozen cohort.

| Owner / omitted work | Source clocks | Native before → after | Recovered clocks | Affected rows | Recovered hold rows sharing this owner |
| --- | ---: | ---: | ---: | ---: | --- |
| `$02:FEBB` bank-02 decoder | 500,454 | 116,776 → 500,454 | 383,678 | 1 | 729 |
| `$02:FF5F` source-byte reader | 160,170 | 13,300 → 160,170 | 146,870 | 1 | 729 |
| `$09:C327` dungeon record | 66,128 | 0 → 66,128 | 66,128 | 6 | 15,313; 31,095; 88,955 |
| `$09:C290` dungeon list | 16,650 | 0 → 16,650 | 16,650 | 6 | 15,313; 31,095; 88,955 |
| `$09:C3E8` overlord record | 8,680 | 0 → 8,680 | 8,680 | 1 | 31,095 |

Recovered-row memberships overlap and are not additive owner-alone hold
claims. In particular, 31,095 needs the combined list/record/overlord work.
The exact per-row additions are 530,548 at 729; 6,230 at 15,313; 35,236 at
31,095; 7,786 each at 43,838, 45,521 and 58,570; and 26,634 at 88,955.
The other 40 cohort rows gain no work. The post-price non-upload signed
instruction remainder is **665,356 clocks**, not an admitted surcharge.

### ROM pricing and all-path fixtures

The enemy-damage loader's translated asset copy already priced its complete
`$02:FE71..FEBA` wrapper and nibble expansion, but omitted the compressed
stream's decoder and reader. A private native decode of the existing encoded
asset now retires those instructions without publishing different game state.
The existing general bank-02 decoder's scalars are replaced by generated
markers, so quadrant callers retain their exact prices. Reader, literal,
fill, alternating word, incrementing, reference and long-header paths price
actual native iterations and bank wraps. Calls retire before their reader;
reader bodies never enter the parent price.

The complete dungeon-list entry, each executed record, cursor/page crossings,
death-mask gate, previous-slot marker, overlord allocation search, and type
suffixes retire source-address blocks. The existing physical room-pointer
table supplies source addresses for page crossing, not scene prices.
`$09:C345` is a separate child only on its real JSR path; the direct BEQ
retains the parent frame. `$09:C871` initialization remains excluded and
unpriced. Partial field publications do not acquire a complete-record charge.
No prior caller scalar is charged again.

The unchanged generator adds **66 blocks**, preserving all **4,410** old rows
exactly. Four new tests execute **3,758 unmodified-ROM cases**: 158 decoder
cases, 2,880 record cases, 80 overlord cases and all 320 cartridge rooms under
both death masks. Synthetic command/record input bytes live in WRAM; no ROM
instruction or data byte is patched. The fixtures independently observe both
outcomes of all **25 reachable conditional branches**, exclude real children
with dynamic call frames, and compare exclusive source clocks. Decoder output
bytes also agree.

The initial record fixture incorrectly excluded local stack pushes; its
replacement tracks real call frames. A subsequent setup error exported a
room-death mask while outdoors, where that WRAM belongs to the overworld
presence table. The corrected indoor fixture passes. The failed fixture logs
are retained; neither supplied a price correction. An invalid audit invocation
without its required range and a zero-iteration early binary-copy launch are
also retained outside admission evidence.

### ROM and CPU picture admission

The requested eight-slice ROM helper completes **1,473,123 iterations**, gate
**PASS / unexplained 0**, at **2,099 / 0 / 0**, versus **2,155 / 0 / 0**.
Every runtime hold equals P2; **54 changed rows recover 56 holds**, with no
row losing a hold. `rom-before-after.json` contains every changed row.
The complete 0–94,334 seeded prefix passes at **43 / 0 / 0**, versus 47 / 0 / 0.
Its recovered rows are **729; 15,313; 31,095; 88,955**. Each cohort main-price
increase equals its independently regenerated omitted source instructions.

| Before iteration | Missing before → after |
| --- | ---: |
| 94,334 | 47 → 43 |
| 141,568 | 82 → 77 |
| 228,499 | 204 → 196 |
| 380,243 | 353 → 341 |
| 461,989 | 430 → 415 |

A fresh eight-slice seeded **`--cpu-video`** census uses the same candidate,
completes all **1,473,123 iterations**, and passes the logical gate with
unexplained zero. **1,462,997 pictures are exact**, preserving the required
count; 563 are early, 3,952 late, four show content during ROM lag, and zero
have other content differences. All per-iteration CPU hashes are retained.
This is fresh picture evidence, not an inherited count or an A/V claim.

### Seeded live admission and poll attribution

One single full seeded `--live-apu-reads --pacing-predictor` candidate
census completes **1,473,123 iterations**, gate **PASS / unexplained 0**,
at **2,451 / 347 / 76**, versus the independently rerun baseline
**2,756 / 482 / 89**. Every live runtime hold equals P2. Missing/excess/FP
improves by **305 / 135 / 13**; 56 missing holds are the ROM improvement,
while the live-minus-ROM poll contributions change from **601 / 482 / 89**
to **352 / 347 / 76**. This is an aggregate hold result, not source-equal
SPC phase or sound delivery.

Exactly **51 live-only differential hold rows change**. All 51 are assigned
to observed `$08:C606` song-end episodes and their execution/reseed aftermath;
**unattributed current and changed rows are both zero**. There are **158**
current live-versus-ROM hold rows, versus 193 at baseline.
`live-partition.json` and `changed-live-holds.csv` retain every changed row's
source lag, all four baseline/candidate ROM/live hold counts, contributions,
original source-read episode, observed poll origin and actual segment bounds.
`poll-witnesses.json` retains both runs' original port observations, including
removed origins. No nearest-poll distance or new classifier is used.

| Observed poll origin before → after | Current live-versus-ROM hold rows | Changed rows | Current added missing / excess / FP | Execution/reseed end, exclusive |
| --- | ---: | ---: | --- | ---: |
| absent → 142,087 | 5 | 5 | 9 / 9 / 2 | 142,122 (main work); segment 144,000 |
| 380,880 → absent | 0 | 15 | 0 / 0 / 0 | 383,636 |
| 462,627 → 462,626 | 6 | 6 | 13 / 13 / 3 | 464,000 |
| 627,255 → absent | 0 | 10 | 0 / 0 / 0 | 629,845 |
| 881,732 → 881,732 | 135 | 0 | 234 / 230 / 67 | 884,327 |
| 1,064,733 → 1,064,733 | 12 | 0 | 96 / 95 / 4 | 1,066,963 |
| 1,318,334 → absent | 0 | 15 | 0 / 0 / 0 | 1,320,900 |

Every changed live-only row is listed below; these are differential changes,
so a shared ROM-price improvement alone does not enter this table.

| Poll episode | Changed iteration IDs |
| --- | --- |
| absent → 142,087 | 142,088; 142,089; 142,091; 142,092; 142,094 |
| 380,880 → absent | 380,945; 380,946; 380,955; 380,956; 381,425; 381,426; 383,178; 383,181; 383,343; 383,346; 383,631; 383,632; 383,633; 383,634; 383,635 |
| 462,627 → 462,626 | 462,698; 462,699; 462,708; 462,709; 463,178; 463,179 |
| 627,255 → absent | 627,316; 627,317; 627,326; 627,327; 627,799; 627,800; 629,325; 629,327; 629,458; 629,462 |
| 1,318,334 → absent | 1,318,395; 1,318,396; 1,318,405; 1,318,406; 1,318,877; 1,318,878; 1,320,353; 1,320,357; 1,320,542; 1,320,544; 1,320,895; 1,320,896; 1,320,897; 1,320,898; 1,320,899 |

The new 142,087 native-zero/source-`$13` observation is one iteration before
source zero, inside the original **141,568–142,088** read episode. Changed
main work begins at that poll and ends at **142,121** (11 changed budget
rows within that execution span); the 142,000 segment reports 34 song-class
WRAM differences. The next 144,000 segment continues cleanly without reseeding.
The shifted 462,626 poll lies in **461,989–462,628**, now two iterations before
source zero; its changed main work ends at 463,998 and the actual 464,000 seed
bounds that aftermath. All six changed hold rows lie within those bounds.

Removed episodes are witnessed in the fresh baseline, whose actual seeded
polygon aftermath ends at 383,636; 629,845; and 1,320,900 respectively.
The candidate has no song or polygon divergence in those stretches and
continues through their ordinary segment boundaries without those reseeds.
The unchanged 881,732 and 1,064,733 episodes retain all 147 of their current
hold-differential rows. Per-episode contributions conserve each run's global
live-minus-ROM totals exactly. Budget equality outside these hold claims,
APU command transport, chip state and waveforms are not newly certified.


### Verification boundary

All **64 distinct filtered Rust tests** and **eight generator tests** pass.
The four new all-path tests, touched loader tests, existing overworld and
file-selection regressions, retained decompressor tests and enemy-damage
fixtures are named in `test-summary.json`. Default candidate and restored
product builds are warning-free; library-test compilation retains the same
24 unrelated dead-code warnings. The restored default product is byte-identical
to the censused candidate.

`verification.json` binds source, binaries, read-only authorities, helper
identities and evidence. `git diff --check` passes. No P2, matcher, classifier,
GPU path, gameplay operation, original-timing model or environment switch
changes. No GPU, `--video`, cached-av, Snes9x process, continuous live replay,
full library suite, oracle-cache/shared-ledger write, cargo fmt, commit or push
was used. Evidence remains below 3 GB; all launched work is joined before the
final receipt. Hold-count improvements do not establish delivered-command,
chip-state, waveform, net SPC-phase or cold exact A/V equality.
## Lagless prefix hunt 8

CPU-only continuation on `codex/prefix-hunt8`, base `9eaa38b9`. Evidence is
under `target/prefix-hunt8/`. The rebuilt baseline is byte-identical to hunt
7's admitted candidate (`a87cae89…`). Prior artifacts were individually
hash-checked before reuse. ROM, assets, SRAM, input, oracle hashes, ledger
streams and all 1,499 retained seed identities match their protected receipts.
The cold source profiler independently regenerates all 196 missing rows before
228,499; its 43 remaining early rows agree exactly with the retained source
routine profiles.

### The uploader label, reconciled per iteration

The whole-route percentage is not the proof for this cohort. Forty-one of the
43 early rows execute no uploader. The other two have the following complete
instruction-clock reconciliation (`uploader-reconciliation.csv/json`):

| Iteration | Source `$00:8888` | Native main-profile label | Native protocol CPU work | Source minus native protocol |
| --- | ---: | ---: | ---: | ---: |
| 34,318 | 8,269,252 | 0 | 8,049,118 | +220,134 |
| 48,556 | 8,033,604 | 0 | 8,196,410 | −162,806 |
| Total | **16,302,856** | **0** | **16,245,528** | **+57,328** |

The native owner is `SongBankHostTransfer` in `spc_driver_clock.rs`, advanced
through `advance_host_cpu_until` during the upload's masked callbacks. It
schedules real CPU bus accesses outside main's cycle ledger. The original
completion observations bind each transfer to the same logical iteration as
the source; no transfer moves into another iteration. Native command/clear
hosts are 37,579–37,603 and 54,131–54,155. The two runs' physical host numbers
are not an equality claim. Source/native masked-callback counts are 23/23 and
24/23 respectively.

The protocol audit includes the 116 CPU clocks before the first ready read and
76 after the final clear. Source per-PC receipts independently recover each
ready/header/byte loop. Both uploads transfer 13,366 bytes through two data
headers and the terminal header. The signed source-minus-native remainder is:

| Executed polling difference | Clocks |
| --- | ---: |
| Ready loops, 58 clocks each | +57,536 |
| Header acknowledgement loops, 52 clocks each | −260 |
| Byte acknowledgement loops, 52 clocks each | +52 |
| Other instruction work / unexplained remainder | **0** |

At 34,318, the three terms are +220,342 / −260 / +52; at 48,556 they are
−162,806 / 0 / 0. Thus 16,245,528 clocks are present in the separate protocol
owner, and 57,328 are a measured difference in executed waits against the
native SPC. There is no unit-price shortfall or logical-iteration migration.
The wait-count difference remains a physical protocol-phase difference; this
batch does not certify equal SPC phase. Charging the uploader's source label
to main would duplicate the protocol owner. No uploader surcharge is added.

### Confirmed shortfalls before 228,499

`raw-owner-comparison.csv/json` retains every signed label on the complete
196-row cohort: **400,019,508 source / 324,464,024 native main clocks**, with
**71,472,860** source clocks under the separately scheduled upload label.
Source and native exclusive sums conserve their respective totals on every
row. The fresh per-row reconciliations prove exact cancellation for
`$00:E6B7/$E583` and `$06:E4AB/$E4DB`, and whole-call equality for `$06:DC64`.
`$01:893C` has six equal active whole-call rows; four unequal rows retain their
69,672-clock signed difference as unresolved work. Existing flattened scan,
quadrant, VWF and inherited-tail prices do not become omissions merely because
their source labels lack a native scope.

The admitted ranking and per-row source/native proofs are retained in
`owner-ranking.csv/json` and `owner-before-after.json`. Counts below refer to
the frozen 196-row cohort, not all route calls. Each admitted owner now equals
its source exclusive instructions on those rows. The overlay's native-before
column includes its existing 106,420-clock ordinary-quadrant scaffold per call,
even though that work previously had the caller's label.

| Owner / omitted work | Native before → after | Recovered clocks | Affected rows | Recovered hold rows sharing this owner |
| --- | ---: | ---: | ---: | --- |
| `$06:AA48` covered-crab writer | 0 → 63,214 | 63,214 | 23 | none |
| `$06:B867` arrow writer | 0 → 36,512 | 36,512 | 11 | 63,080 |
| `$02:E4A3` dungeon-exit body | 0 → 22,116 | 22,116 | 3 | none |
| `$09:C770` overworld sprite record | 0 → 18,930 | 18,930 | 4 | none |
| `$02:F7CB` overlay-specific setup and traversal | 425,680 → 440,832 | 15,152 | 4 | 56,021 |
| `$02:E5D4` cached-exit restoration | 0 → 3,220 | 3,220 | 1 | none |
| `$09:C871` dungeon overlord initialization | 0 → 226 | 226 | 1 | none |

The additions total **159,370 clocks on 43 affected cohort rows**. Memberships
can overlap; they are not owner-alone counterfactual hold claims. Every row's
main-price increase equals the sum of its independently regenerated omitted
instructions. The non-upload signed remainder is **3,923,254 clocks** and is
not an admitted price.

### ROM paths, fixtures and rejected Moldorm trial

The draw markers retire actual clipping, graphics, type and loop outcomes.
Coordinate-preparation children keep their existing charges. The overlay
entry uses its own pointer setup and 16-by-16 traversal, replacing the ordinary
quadrant scaffold while preserving decoder/interlace/map-definition children.
Its difference is 3,788 clocks per complete call. Dungeon exits price the
actual descending table search, physical indexed page crossings, signed scroll
branches, cached restoration and camera/graphics call instructions.

The overworld record's allocation scan, reusable-slot condition and field
stores are self work. `$09:C80B` is a real child on the overlord path, including
its `$C871` fallthrough; dungeon's JSR to `$C871` gets a separate call frame.
The `$C779` RTS after the real overworld child is priced separately. `$C80B`
has no executions on this frozen cohort, but its complete allocation/type
paths are covered by the same source fixtures. No partial publication acquires
a complete-record price.

The unchanged generator adds **74 blocks**, preserving all **4,476** old rows
exactly. Five new tests execute **1,204 unmodified-ROM cases**: 192 arrow,
480 crab, 281 record/overlord/initialization, 91 dungeon/cached-exit and all
160 cartridge overlay screens. They independently observe both outcomes of
all **31 reachable conditional branches**. `$C78A`'s taken equality is
unreachable after the earlier identical comparison rejected it; the fixture
checks that fact. Synthetic fixture data occupies WRAM; ROM instructions and
data are unpatched.

An additional Mini Moldorm trial passed 960 source cases but failed route
admission at **931,965**: the exact 122-clock out-of-bounds return changed main
from 408,614 to 408,736 clocks, tipped the existing spotlight raster wait from
0 to 270,204, and produced two holds against source lag one. No existing charge
for that return was proved. The complete owner is deferred, with its patch,
binary, fixtures, **2,054 / 1 / 0** rejected census and row evidence retained
under `rejected-moldorm/`. The final source has no Moldorm pricing change;
931,965 returns to 408,614 clocks, zero raster wait and one hold. The admitted
batch does not claim that timing interaction is repaired.

### ROM and CPU picture admission

The requested eight-slice ROM helper completes **1,473,123 iterations**, gate
**PASS / unexplained 0**, at **2,084 / 0 / 0** versus **2,099 / 0 / 0**. All
runtime holds equal P2. Exactly **15 rows recover 15 holds**, with no lost
holds. Route-wide main price increases by **48,719,856 clocks on 16,852 rows**, with
no negative clock differences. `rom-before-after.json` retains every changed
hold row and both clock/count comparisons.

| Before iteration | Missing before → after |
| --- | ---: |
| 94,334 | 43 → 41 |
| 141,568 | 77 → 75 |
| 228,499 | 196 → 194 |
| 380,243 | 341 → 337 |
| 461,989 | 415 → 410 |

A fresh eight-slice **`--cpu-video`** census uses the admitted binary and
completes all **1,473,123 iterations**, gate PASS with unexplained zero.
**1,462,997 pictures are exact**; 563 are early, 3,952 late, four show content
during source lag and zero have other content differences. Every new
per-iteration CPU hash file is byte-identical to its individually hash-verified
main counterpart, including existing mismatch cases. This is fresh CPU picture
evidence, not inherited acceptance or an A/V claim.

### Single seeded live admission and poll attribution

One full, single-process seeded `--live-apu-reads --pacing-predictor` census
completes **1,473,123 iterations**, gate **PASS / unexplained 0**, at
**2,457 / 368 / 81**, versus main's hash-verified **2,451 / 347 / 76**.
Every runtime hold equals P2. Live missing/excess/FP worsens by **6 / 21 / 5**.
The live-minus-ROM contributions change from **352 / 347 / 76** to
**373 / 368 / 81**: the two newly observed poll episodes add 21 missing and
21 excess holds and five FP rows, while ROM pricing recovers 15 missing holds.
The shifted 881,732 episode preserves its aggregate contribution.

Exactly **14 live-only differential hold rows change**. All 14 are bound to
original `$08:C606` read episodes and observed execution/reseed aftermath;
**unattributed current and changed rows are both zero**. There are 168 current
live-versus-ROM hold rows, versus 158 on main. The 29 raw live hold changes
consist of these 14 plus the 15 shared ROM-price recoveries.

Attribution uses native gameplay-poll observations to identify the original
source read episodes. Execution bounds come from independent main instruction
differences, recorded classified execution, and actual census segments/reseeds.
Changed hold rows do not select those bounds. The compressed timing trace
records every iteration and independently confirms all eight actual reseeds:
464,000; 736,000; 738,000; 882,000; 884,000; 884,327; 1,066,000; and 1,066,963.
All main-budget difference groups begin at their observed poll origin; neither
run has an unanchored main-budget group.

| Observed poll origin before → after | Current hold-differential rows | Changed rows | Current added missing / excess / FP | Execution/reseed end, exclusive |
| --- | ---: | ---: | --- | ---: |
| 142,087 → 142,087 | 5 | 0 | 9 / 9 / 2 | 144,000 |
| absent → 229,034 | 4 | 4 | 9 / 9 / 2 | 230,000 |
| 462,626 → 462,626 | 6 | 0 | 13 / 13 / 3 | 466,000 |
| absent → 735,866 | 6 | 6 | 12 / 12 / 3 | 738,000 |
| 881,732 → 881,733 | 135 | 4 | 234 / 230 / 67 | 884,327 |
| 1,064,733 → 1,064,733 | 12 | 0 | 96 / 95 / 4 | 1,066,963 |

Every changed live-only row is listed below. The ROM hold column is identical
before and after on all 14 rows. `changed-live-holds.csv` retains all four
ROM/live counts and their differentials; `live-partition.json` conserves both
runs' global live-minus-ROM totals exactly.

| Iteration | Poll episode | Source lag NMIs | ROM holds | Main live holds → candidate |
| --- | --- | ---: | ---: | ---: |
| 229,035 | new 229,034 | 0 | 0 | 0 → 5 |
| 229,036 | new 229,034 | 5 | 5 | 5 → 0 |
| 229,038 | new 229,034 | 0 | 0 | 0 → 4 |
| 229,041 | new 229,034 | 4 | 4 | 4 → 0 |
| 735,936 | new 735,866 | 0 | 0 | 0 → 1 |
| 735,937 | new 735,866 | 1 | 1 | 1 → 0 |
| 735,946 | new 735,866 | 0 | 0 | 0 → 4 |
| 735,947 | new 735,866 | 5 | 4 | 4 → 0 |
| 736,420 | new 735,866 | 8 | 7 | 7 → 0 |
| 736,422 | new 735,866 | 0 | 0 | 0 → 7 |
| 881,793 | 881,732 → 881,733 | 0 | 0 | 1 → 0 |
| 881,794 | 881,732 → 881,733 | 0 | 0 | 0 → 1 |
| 881,803 | 881,732 → 881,733 | 0 | 0 | 4 → 0 |
| 881,804 | 881,732 → 881,733 | 0 | 0 | 0 → 4 |

The new 229,034 native-zero/source-`$13` observation is one iteration before
source zero, within **228,499–229,035**. Its changed main work ends at
229,068; the 230,000 segment continues cleanly. The new 735,866 observation
is within **735,229–735,867**, again one iteration before source zero. Its
main-budget differences end at 736,633, with classified polygon aftermath
through 737,999; the actual 736,000 and 738,000 reseeds are retained. All six
changed rows lie inside those recorded bounds.

The shifted 881,733 observation remains within **881,095–881,734**. Main's
881,732 observation is two iterations before source zero; the candidate is
one before. The four changed rows move the one- and four-hold FP events one
iteration later. The original 882,000 / 884,000 / 884,327 reseed chain bounds
the aftermath. Original observations from both runs, including absent polls,
are preserved in `poll-witnesses.json`.

The unchanged 462,626 episode still reseeds at 464,000. Both ROM/live
comparisons retain 52 post-seed main-budget differences ending at 464,051;
this receipt therefore records the containing segment's 466,000 endpoint.
Those rows add no changed hold claim. Budget equality outside the reported
hold claims is not certified.

### Verification boundary

All **77 distinct filtered Rust tests** and **eight generator tests** pass.
`test-summary.json` lists the exact tests: the five new all-path fixtures,
touched overworld/loading modules, arrow/crab/Moldorm regressions and the
retained prefix-7 decoder/loader fixtures. Product builds are warning-free;
library-test compilation retains the same 24 unrelated dead-code warnings.
The default product and censused candidate are byte-identical.

`verification.json` and `artifact-hashes.json` bind source, binaries,
authorities, helper identities and evidence. `git diff --check` passes.
No matcher, classifier, P2, original-timing model or environment-switch change
is included. No GPU, `--video`, cached-av, Snes9x process, continuous live
replay, full library suite, oracle-cache/shared-ledger write, cargo fmt,
commit or push occurred. Rebuildable files were removed only from this
worktree's own `target/`; hunt-7 evidence remains read-only. Target stays below
3 GB, and all launched work is joined before the final receipt.

The uploader's remaining wait-count difference and rejected Moldorm timing
interaction remain explicit. Hold counts and CPU picture hashes do not
establish delivered-command, chip-state, waveform, net SPC-phase or cold exact
A/V equality.

## Lagless prefix hunt 9

CPU-only continuation on `codex/prefix-hunt9`, base `61b57842`. Evidence is
under `target/prefix-hunt9/`. The fresh 94,334-iteration baseline matches
every column of hunt 8's hash-verified admitted predictor. Authority hashes
and all 1,499 retained seed identities agree. The cold source profiler
regenerates all 41 early missing rows: its 5,080 routine rows match the
retained source exactly. A second cold pass adds opening-NMI timestamps;
routine, structural and timing-audit files are byte-identical between passes.
No prior binary with an unverified identity supplies a new acceptance claim.

### Classification of the 41 rows

`classification.csv/json` retains every row's diagnostic hold-boundary inverse,
signed instruction reconciliation, physical source opening-NMI and main-entry
raster, native nominal entry, lag-NMI arrivals, executed APU polls, DMA,
refresh and nonrefresh bus terms. `instruction-conservation.json` verifies
exclusive-sum conservation independently on both sides of every row.
The inverse is diagnostic only; no inverse, deficit or sampled loop count is
used as a runtime price.

| Class | Rows | Finding |
| --- | ---: | --- |
| (a) instruction shortfall | 38 | The remaining non-poll instruction gap alone reaches the diagnostic inverse, which already composes refresh and leased HDMA. |
| (b) independently established inherited arrival cause | 0 | Arrival differences are retained, but no row is assigned an exclusive upstream cause without that proof. |
| (c) executed upload poll/return phase | 2 | 34,318 and 48,556 have separately scheduled protocol instructions and unequal executed polling. |
| (d) mixed NMI-entry/bus phase residual | 1 | At 63,995, 7,496 instruction clocks fall 10 short of the 7,506-clock inverse. Source CLI is 82 clocks later than P2's nominal entry and four earlier than the HDMA composer's nominal entry; the source lag NMI arrives at V225/H48 versus the composer's nominal H86. Source nonrefresh main bus time is 252 clocks greater. Their exclusive causal split is unresolved. |

These are primary discrepancy classes, not a claim that an instruction-gap
row has no arrival or bus difference. The source has **zero V-counter reads
on all 41 rows**, and native raster-wait time is zero throughout. Neither the
V255 exit nor the V257-to-1 wrap nor `$00:F312` executes here. There is no
evidence for a spotlight-wait explanation of this early cohort. Opening
handler wall time agrees on 39 rows; source exceeds native by 584 clocks at
688 and 102 at 42,912. Actual source main-entry phase, the nominal P2 V225/H0
origin, and the leased-HDMA V225/H86 origin remain separate columns.

The complete cohort has 114,879,568 source instruction clocks, 98,007,002
native main clocks and 16,245,528 native protocol clocks. The signed remainder
is **627,038**; removing the **57,328** executed-poll difference leaves
**569,710 non-poll instruction clocks**. These totals do not become prices.
Main DMA agrees exactly at **393,840 clocks**. Source nonrefresh main bus
residual is 370,132 versus 364,602 native leased-HDMA clocks; interrupt-edge
refresh attribution remains explicit rather than being renamed a price.

The uploader reconciliation is unchanged and individually hash-verified.
The ready column counts 16-bit read pairs; header/byte columns count actual
acknowledgement-loop observations. The native owner remains
`SongBankHostTransfer`, outside main's routine ledger.

| Iteration | Ready pairs source / native | Header ACK source / native | Byte ACK source / native | Signed total instruction gap after protocol attribution | Gap after executed-poll reconciliation |
| --- | ---: | ---: | ---: | ---: | ---: |
| 34,318 | 11,184 / 7,385 | 52 / 57 | 76,551 / 76,550 | +246,430 | +26,296 |
| 48,556 | 7,122 / 9,929 | 51 / 51 | 76,551 / 76,551 | −136,556 | +26,250 |

The three wait-count terms remain +57,536 / −260 / +52 clocks, with zero
unexplained protocol instruction remainder. Source/native masked callbacks
are 23/23 and 24/23. In particular, 48,556 misses a hold while native executes
more protocol work. Upload wait/return phase is not an omitted uploader unit
price. This batch does not repair or certify equal SPC phase.

### Cucco caller and shared-tail pricing

Cucco is the largest active-sprite partition in the early cohort:
**122 calls, 1,566,342 source clocks / 1,431,508 native before**. The retained
draw prefix and existing coordinate, movement, collision, damage, RNG and
lifted-child bodies already own their instructions. The missing work is in
the caller suffix and shared native tails. `sprite-owner-ranking.json`
retains the disjoint sprite-call ranking; raw routine label differences remain
in `raw-owner-comparison.csv` and are not treated as omissions.

| Source owner/range | Recovered cohort clocks | Affected rows |
| --- | ---: | ---: |
| `$06:A5DA..A67E` caller suffix and actual state dispatch | 67,206 | 19 |
| `$06:A67F..A6AC` calm selection | 644 | 7 |
| `$06:A6AD..A6B0` shared lifted-call opcode and actual return | 11,880 | 19 |
| `$06:A6B1..A6E1` hopping caller | 4,662 | 10 |
| `$06:A6E2..A6FB` subtype increments, graphics publication and tail | 42,780 | 19 |
| `$06:A6FC..A726` flee and shared landing retarget | 3,952 | 13 |
| `$06:A727..A72E` panic wrapper | 1,950 | 13 |
| `$06:A72F..A732` distress entry | 624 | 13 |
| **Total** | **133,698** | **19 distinct rows** |

Markers retire native branch outcomes and actual child-call/return
instructions. Three-, four- and five-increment entries retain their own ROM
PCs, and suspended helpers resume after their retired increments. Graphics
publication does not precharge the pending lifted call. The lifted helper's
double return does not acquire the skipped caller RTS. The carried landing
uses the same real retarget tail as flee; its other caller instructions remain
unpriced. No complete-call price is attached to a partial publication.

Every new per-row main-price addition equals the independent source receipts.
Expanding the executed markers proves exact instruction counts and clocks on
all **1,186 per-PC rows** in the newly owned ranges. Cucco's resulting whole
calls total **1,565,206 clocks**, leaving **1,136** unresolved; no aggregate
whole-call deficit is charged. `per-pc-reconciliation.json`,
`cucco-whole-call-reconciliation.json` and `cohort-before-after.json` retain
the proofs. The generator adds **60 blocks**, preserving all **4,550** old
blocks exactly. The new fixture executes **291 unmodified-ROM cases** and
observes both outcomes of all **18 conditional branches** in these ranges.

### ROM admission

The requested eight-slice helper completes **1,473,123 iterations**, gate
**PASS / unexplained 0**, at **2,069 / 0 / 0** versus **2,084 / 0 / 0**.
All runtime holds equal P2. Exactly 15 rows gain one correct hold, with no
lost hold or new excess anywhere: 43,375; 50,605–50,608; 50,653; 50,655;
50,657; 50,659; 50,765; 50,767; 50,771; 50,773; 50,775; and 51,029.
The route-wide main increase is **50,153,806 clocks on 23,117 rows**, with
zero negative clock differences. `rom-before-after.json` retains every
changed hold and the complete comparisons.

| Before iteration | Missing before → after |
| --- | ---: |
| 94,334 | 41 → 26 |
| 141,568 | 75 → 60 |
| 228,499 | 194 → 179 |
| 380,243 | 337 → 322 |
| 461,989 | 410 → 395 |

The deferred Mini Moldorm price remains deferred. Class (c) does not dominate
this cohort, and none of its rows executes the spotlight wait. Hunt 8's
931,965 witness still identifies a separate wait-model problem; this batch
does not claim to fix it or re-admit that owner.

### CPU picture admission

The fresh eight-slice `--cpu-video` census completes **1,473,123 iterations**,
gate **PASS / unexplained 0**. **1,462,997 pictures remain exact**; 563 are
early, 3,952 late, four show content during source lag and zero have other
content differences. Every new per-iteration CPU hash file is byte-identical
to its individually hash-verified baseline counterpart, including existing
mismatch cases. `cpu-picture-hash-agreement.json` binds all eight comparisons.
This is fresh CPU picture evidence, not an inherited acceptance or A/V claim.

### Single seeded live admission and poll attribution

One full, single-process seeded `--live-apu-reads --pacing-predictor` census
completes **1,473,123 iterations**, gate **PASS / unexplained 0**. SPC state
carries across the actual seeds. All runtime holds equal P2.

| Lane | Missing before → after | Excess before → after | FP before → after |
| --- | ---: | ---: | ---: |
| ROM | 2,084 → 2,069 | 0 → 0 | 0 → 0 |
| Live | 2,457 → 2,547 | 368 → 449 | 81 → 84 |
| Live minus ROM | 373 → 478 | 368 → 449 | 81 → 84 |

Live therefore worsens by **90 / 81 / 3**. Its phase-dependent contribution
adds **105 / 81 / 3**, while the shared ROM prices recover 15 missing holds.
The newly observed 380,880 episode contributes 114 / 90 / 5; removing the
old 229,034 episode subtracts 9 / 9 / 2. The shifted 462,627 episode preserves
its aggregate contribution. No equal-SPC-phase or live-improvement claim is
made.

There are **25 live-only differential hold changes**, all attributed to
original `$08:C606` song-end episodes; **unattributed current and changed
rows are zero**. The 40 raw live hold changes consist of those 25 and the
15 shared ROM recoveries. Current live-versus-ROM hold rows number **179**,
versus **168** on the baseline. `live-partition.json` conserves both runs'
complete global live-minus-ROM totals and checks all four P2/runtime counts.

| Observed poll origin before → after | Current hold-differential rows | Changed rows | Current added missing / excess / FP | Execution/reseed end, exclusive |
| --- | ---: | ---: | --- | ---: |
| 142,087 → 142,087 | 5 | 0 | 9 / 9 / 2 | 144,000 |
| 229,034 → absent | 0 | 4 | 0 / 0 / 0 | baseline 230,000 |
| absent → 380,880 | 15 | 15 | 114 / 90 / 5 | 383,636 |
| 462,626 → 462,627 | 6 | 6 | 13 / 13 / 3 | 466,000 |
| 735,866 → 735,866 | 6 | 0 | 12 / 12 / 3 | 738,000 |
| 881,733 → 881,733 | 135 | 0 | 234 / 230 / 67 | 884,327 |
| 1,064,733 → 1,064,733 | 12 | 0 | 96 / 95 / 4 | 1,066,963 |

Every changed live-only row follows. Both ROM columns agree on all 25 rows.
`changed-live-holds.csv` retains all four counts, signed differentials,
original source interval, observed poll origins and execution endpoints.

| Iteration | Poll episode | Source lag NMIs | ROM holds | Baseline live holds → candidate |
| --- | --- | ---: | ---: | ---: |
| 229,035 | removed 229,034 | 0 | 0 | 5 → 0 |
| 229,036 | removed 229,034 | 5 | 5 | 0 → 5 |
| 229,038 | removed 229,034 | 0 | 0 | 4 → 0 |
| 229,041 | removed 229,034 | 4 | 4 | 0 → 4 |
| 380,945 | new 380,880 | 0 | 0 | 0 → 1 |
| 380,946 | new 380,880 | 1 | 1 | 1 → 0 |
| 380,955 | new 380,880 | 0 | 0 | 0 → 4 |
| 380,956 | new 380,880 | 5 | 4 | 4 → 0 |
| 381,425 | new 380,880 | 0 | 0 | 0 → 8 |
| 381,426 | new 380,880 | 8 | 8 | 8 → 0 |
| 383,178 | new 380,880 | 0 | 0 | 0 → 7 |
| 383,181 | new 380,880 | 7 | 7 | 7 → 0 |
| 383,343 | new 380,880 | 7 | 7 | 7 → 0 |
| 383,346 | new 380,880 | 0 | 0 | 0 → 7 |
| 383,631 | new 380,880 | 59 | 59 | 59 → 0 |
| 383,632 | new 380,880 | 6 | 6 | 6 → 0 |
| 383,633 | new 380,880 | 23 | 22 | 22 → 0 |
| 383,634 | new 380,880 | 1 | 1 | 1 → 59 |
| 383,635 | new 380,880 | 1 | 1 | 1 → 6 |
| 462,698 | 462,626 → 462,627 | 0 | 0 | 1 → 0 |
| 462,699 | 462,626 → 462,627 | 0 | 0 | 0 → 1 |
| 462,708 | 462,626 → 462,627 | 0 | 0 | 4 → 0 |
| 462,709 | 462,626 → 462,627 | 0 | 0 | 0 → 4 |
| 463,178 | 462,626 → 462,627 | 0 | 0 | 8 → 0 |
| 463,179 | 462,626 → 462,627 | 0 | 0 | 0 → 8 |

Bounds are derived from observed poll outcomes, independent main-budget
differences, classified execution and actual census segments/reseeds; changed
hold rows do not select them. The new native-zero/source-`$13` observation
at 380,880 is inside the original **380,243–380,881** read episode, one
iteration before source zero. Its 1,056 main-budget differences begin at
380,880 and end at 383,635. Classified song-poll differences reach 381,999;
the observed 382,000 reseed retains SPC state, and the remaining budget
differences end at the actual 383,636 resync. This post-seed aftermath is
retained explicitly rather than described as another source poll.

The removed 229,034 episode's original baseline witness and 230,000 endpoint
remain intact; the candidate instead observes zero with source at 229,035.
The shifted 462,627 observation remains inside **461,989–462,628**. Its
472 main-budget differences end at 464,051, after the actual 464,000 reseed,
so the containing segment ends at 466,000. The baseline had 661 differences
in that episode, with the same last iteration. All six changed hold rows
are inside the independently retained bounds.

The compressed timing stream records every iteration and confirms all ten
actual reseeds: **382,000; 383,636; 464,000; 736,000; 738,000; 882,000;
884,000; 884,327; 1,066,000; and 1,066,963**. Every main-budget difference
group is anchored to an observed poll origin; unanchored groups are zero in
both runs. `poll-witnesses.json` retains the original observations, including
concordant zero reads and removed baseline episodes. These receipts establish
the stated hold attribution, not equal chip state, waveform or A/V phase.

### Verification boundary

All **50 distinct filtered Rust tests** and **eight generator tests** pass.
The Rust set covers the new all-branch ROM fixture, retained Cucco instruction
and continuation fixtures, chicken hopping, both touched NPC modules and the
source-profiler example. Product and source-profiler builds are warning-free;
library-test compilation retains the same 24 unrelated dead-code warnings.
The default product and censused candidate are byte-identical.

`verification.json` and `artifact-hashes.json` bind source, binaries,
authorities, helper identities and evidence. `git diff --check` passes.
No matcher, classifier, P2, original-timing or environment-switch change is
included. No GPU, `--video`, cached-av, Snes9x process, continuous live replay,
full library suite, oracle-cache/shared-ledger write, cargo fmt, commit or push
occurred. Rebuildable files were removed only from this worktree's own
`target/`; hunt-7/8 evidence remains read-only. Target stays below 3 GB after
the initial oversized cache cleanup, and all launched work is joined.

The mixed 63,995 phase residual, two upload phase cases, remaining unpriced
instructions and deferred Moldorm/wait interaction stay explicit. ROM hold
recovery and exact CPU picture hashes do not establish delivered-command,
chip-state, waveform, net SPC-phase or cold exact A/V equality.

## Lagless prefix hunt 10

Worktree `/Users/missingno/Documents/zelda3-rs-codex`, branch
`codex/prefix-hunt10`, based on **24130104**. Evidence is retained in
`target/prefix-hunt10/`. The full-v2 ledger and pinned oracle cache remain
read-only. Every reused authority and evidence file is hash-checked; the
fresh 94,334-iteration baseline predictor CSV equals hunt 9 byte for byte.

### Reclassification and remaining owners

The classification is refreshed against the new main, including hunt 9's
Cucco additions. Before: **23 instruction-shortfall / 0 independently proved
inherited arrival / 2 upload poll-return / 1 mixed entry-bus** rows. After:
**13 / 0 / 2 / 1**, leaving **16 missing rows before 94,334**. Ten early rows
recover; 49 recover over the complete ROM route. The 26-row cohort gains
**159,536 instruction clocks**. Arrival and bus observations remain
diagnostics; no deficit or inverse becomes a price.

`classification.csv/json` names every remaining row and its specific blocker.
`reconciled-owners-after.json` retains all signed owner buckets, including
zero-sum aliases and the existing -22-clock main-return convention. Sprite
partitions sum exclusive instruction work; raw overlapping inclusive gaps
are never summed as omissions. The source has no V-counter reads on these
26 rows, and native raster-wait time stays zero.

| Iteration | Class | Instruction clocks to diagnostic hold boundary | Remaining non-poll instruction gap | Specific remaining blocker |
| --- | --- | ---: | ---: | --- |
| 34,318 | (c) | 265,206 | 20,730 | Executed upload polling/return phase; equal receiver state and first-poll phase unproved; remaining non-poll loaders/callers are insufficient alone |
| 35,294 | (a) | 1,588 | 5,934 | $42/09 2,824, $ba/09 858; Y movement response 1,090, X movement response 850, Y item caller 150, sword caller 184 |
| 35,300 | (a) | 5,714 | 5,934 | $42/09 2,824, $ba/09 858; Y movement response 1,090, X movement response 850, Y item caller 150, sword caller 184 |
| 35,304 | (a) | 5,572 | 5,934 | $42/09 2,824, $ba/09 858; Y movement response 1,090, X movement response 850, Y item caller 150, sword caller 184 |
| 35,306 | (a) | 2,048 | 6,922 | $42/09 3,812, $ba/09 858; Y movement response 1,090, X movement response 850, Y item caller 150, sword caller 184 |
| 43,367 | (a) | 1,648 | 3,860 | $1d/09 268, $2a/09 64, $3c/09 68, $74/09 482; Y movement response 1,090, X movement response 850, Y item caller 150, sword caller 184, tile-interaction caller 726 |
| 47,033 | (a) | 3,370 | 3,676 | $2a/09 56, $3c/09 60, $74/09 474, $da/09 108; Y movement response 1,090, X movement response 850, Y item caller 150, sword caller 184, tile-interaction caller 726 |
| 48,247 | (a) | 5,888 | 7,580 | $2a/09 56, $3c/09 60, $74/09 474, $da/09 108; Y movement response 1,090, X movement response 850, Y item caller 150, tile-interaction caller 726, 079d72 1,468, 08fb2b 456, 0ff979 2,164 |
| 48,556 | (c) | 70,310 | 20,684 | Executed upload polling/return phase; equal receiver state and first-poll phase unproved; remaining non-poll loaders/callers are insufficient alone |
| 51,093 | (a) | 786 | 3,592 | $2a/09 64, $3c/09 68, $74/09 482; Y movement response 1,090, X movement response 850, Y item caller 150, sword caller 184, tile-interaction caller 598, 02f2a5 128 |
| 63,077 | (a) | 7,412 | 8,912 | $1b/09 1,746, $41/09 816, $42/09 1,214, $46/09 2,844; Y movement response 1,090, X movement response 850, Y item caller 190, sword caller 184 |
| 63,081 | (a) | 7,670 | 8,912 | $1b/09 1,746, $41/09 816, $42/09 1,214, $46/09 2,844; Y movement response 1,090, X movement response 850, Y item caller 190, sword caller 184 |
| 63,974 | (a) | 3,160 | 4,804 | $08/09 2,186, $0a/09 214, $51/09 236; Y movement response 1,090, Y item caller 190, sword caller 184, tile-interaction caller 726 |
| 63,995 | (d) | 2,708 | 2,698 | 2698 non-poll instruction clocks vs 2708 inverse; remaining Octorok and movement/Y-item/sword instructions plus unresolved 10-clock NMI-entry/bus split |
| 64,003 | (a) | 2,228 | 3,360 | $08/09 192, $0a/09 96, $0c/06 780; Y movement response 1,090, X movement response 850, Y item caller 190, sword caller 184 |
| 64,024 | (a) | 3,628 | 5,384 | $08/09 192, $0a/09 96, $51/09 118; Link state aliases 1,434, Y movement response 1,090, X movement response 850, 078926 230, 0792a0 348, 0792b9 112, 07e1be 550, 07e1d7 314, 07e370 3,100 |

Sprite labels are type/state: `$42/$09` is guard; `$BA/$09` whirlpool;
$1D/$09` flute quest; `$2A/$09` sweeping lady; `$3C/$09` Kakariko boy;
$74/$09` running boy; `$DA/$09` rupee; `$1B/$09` arrow; `$41/$09` guard;
$46/$09` blue archer; `$08/$0A` Octoroks; `$0C/$06` dying stone; and
$51/$09` Armos. The movement buckets are the remaining Y/X response tails
after their already-priced probe/setup children. Map-scroll, Y-item, sword
and named hexadecimal caller residues remain unpriced; their per-row
decomposition is retained rather than charged as an aggregate.

### ROM-exact owners and fixtures

| Owner | Added clocks on the 26-row cohort | Affected rows |
| --- | ---: | ---: |
| palette cache continuation | 80,466 | 3 |
| armor/glove caller and armor copy | 27,830 | 5 |
| Armos caller and Z movement | 18,536 | 7 |
| guard collision caller and bank-5 wrapper | 16,216 | 7 |
| Octorok-stone caller | 13,932 | 7 |
| intro dispatch/fade | 1,386 | 1 |
| fairy draw prefix | 1,056 | 3 |
| incremental graphics uploader | 114 | 1 |
| **Total** | **159,536** | **26-row cohort** |

The cache copy now retires each actual LDA/STA pair and each completed
eight-word loop tail, retaining its cursor over suspension. All 258 possible
cuts, including empty/repeated slices and the final CGRAM publication, match
the unmodified ROM. The atomic caller uses the same owner, replacing its
existing 26,822-clock scalar without adding it again. Only three previously
unpriced pre-dungeon continuations contribute the 80,466-clock increase.

Armor/glove caller markers preserve the real 15-word arbitrary-copy child.
The three-word sword and four-word shield calls remain explicit omissions:
2,508 clocks on each of the five selected palette-load rows. The guard path
owns its actual post-parry branches, movement/collision call opcodes and RTS
dispatch, plus its bank-5 B88D wrapper. The blue-archer B88D calls remain
unpriced. Armos owns its caller and actual bank-5 Z mover; selecting the real
bank-5 inactive entry changes attribution without changing its total price.
Octorok stones retain their actual crumbling/active branches. Fairy pricing
ends at the draw/touchability/inactive prefix; it does not precharge the AI.
The intro prices its executed module/JSL dispatcher and background-fade
wrappers. Incremental graphics prices both the completed-counter return and
the actual upload branch, with no scene or iteration selector.

The generator adds **105 blocks** and preserves all **4,610** old blocks.
The eight new tests execute **1,048 unmodified-ROM fixtures**. Expanded
markers agree in count and clocks on **1,010 complete-owner per-PC rows**.
Two shared-PC families retain intentional partial coverage: armor copies
exclude sword/shield calls, and the bank-5 collision wrapper excludes three
blue-archer calls. Those omissions are recorded in
`per-pc-reconciliation.csv/json`; no full-coverage claim is made for them.
`owner-ranking.json` and `cohort-before-after.json` conserve each row's
entire price increase.

### Upload poll/return phase

Fresh recorded-APU prefix diagnostics observe these executed polls:

| Iteration | Ready pairs source / native | Header ACK source / native | Byte ACK source / native | Remaining non-poll gap |
| --- | ---: | ---: | ---: | ---: |
| 34,318 | 11,184 / 10,036 | 52 / 56 | 76,551 / 76,551 | 20,730 |
| 48,556 | 7,122 / 7,078 | 51 / 56 | 76,551 / 76,551 | 20,684 |

`upload-phase-current.json` retains executed access counts, phase clocks,
first reads, clears and return callbacks. Native masked callbacks remain
23/23 versus source 23/24. The new ready counts differ from main's
7,385/9,929 despite an unchanged protocol implementation; upstream restored
holds change the receiver/arrival history even when gameplay reads replay
the cartridge answers. All 24 filtered upload-return tests pass, including
ready-poll bus timestamps, future-store visibility and return deadlines.
This establishes no new protocol unit-price omission or ROM-exact native
scheduling defect. Equal receiver state and first-poll phase are still
unproved; scheduling versus SPC response timing is not exclusively resolved.
No upload behavior change or fitted wait adjustment is admitted.

### ROM admission

The requested eight-slice helper completes **1,473,123 iterations**, gate
**PASS / unexplained 0**, at **2,020 / 0 / 0**, versus main's **2,069 / 0 / 0**.
All runtime holds equal P2. Exactly **49** rows gain one correct hold; no hold
is lost and no excess appears. Route-wide main instruction pricing increases
by **139,228,456 clocks on 200,440 rows**, with zero negative differences.

| Before iteration | Missing before → after |
| --- | ---: |
| 94,334 | 26 → 16 |
| 141,568 | 60 → 50 |
| 228,499 | 179 → 167 |
| 380,243 | 322 → 308 |
| 461,989 | 395 → 376 |

The ten early recoveries are **688; 42,912; 43,838; 45,521; 58,570;
63,076; 64,004; 64,008; 64,020; and 83,186**.
`rom-before-after.json` retains all 49 changed rows and the full comparison.

### CPU picture admission

The fresh eight-slice `--cpu-video` census completes **1,473,123 iterations**,
gate **PASS / unexplained 0**. **1,462,997 pictures remain exact**; 563 are
early, 3,952 late, four show content during source lag, and zero have other
content differences. All eight per-iteration hash files equal their
individually hash-verified hunt-9 baseline counterparts byte for byte.
`cpu-picture-hash-agreement.json` binds these fresh comparisons.

### Single seeded live admission and poll attribution

One full, single-process `--live-apu-reads --pacing-predictor` census completes
**1,473,123 iterations**, gate **PASS / unexplained 0**. SPC state carries
across the actual seeds. All four predictor/runtime count comparisons agree.

| Lane | Missing before → after | Excess before → after | FP before → after |
| --- | ---: | ---: | ---: |
| ROM | 2,069 → 2,020 | 0 → 0 | 0 → 0 |
| Live | 2,547 → 2,516 | 449 → 401 | 84 → 86 |
| Live minus ROM | 478 → 496 | 449 → 401 | 84 → 86 |

Live improves by **31 missing / 48 excess**, while FP increases by **2**.
The shared ROM prices recover 49 missing holds; the net phase contribution
is **+18 / -48 / +2**. The two new episodes add 144 / 54 / 10, removing two
baseline episodes subtracts 126 / 102 / 8, and the shifted episode preserves
its aggregate contribution. No equal-SPC-phase claim is made.

There are **52 live-only differential hold changes**, all attributed to
original `$08:C606` song-end episodes. **Unattributed current/changed rows
and unanchored main-budget differences are zero**. The 101 raw live hold
changes comprise those 52 plus 49 shared ROM recoveries. Current live versus
ROM hold-differential rows number **183**, versus **179** on main.
`live-partition.json` conserves both complete live-minus-ROM totals and all
four P2/runtime counts.

| Observed poll origin before → after | Current hold-differential rows | Changed rows | Current added missing / excess / FP | End before → after, exclusive |
| --- | ---: | ---: | --- | --- |
| 142,087 → 142,087 | 5 | 0 | 9 / 9 / 2 | 144,000 → 144,000 |
| 380,880 → absent | 0 | 15 | 0 / 0 / 0 | 383,636 → absent |
| 462,627 → 462,626 | 6 | 6 | 13 / 13 / 3 | 466,000 → 466,000 |
| absent → 627,255 | 10 | 10 | 27 / 27 / 5 | absent → 629,845 |
| 735,866 → absent | 0 | 6 | 0 / 0 / 0 | 738,000 → absent |
| 881,733 → 881,733 | 135 | 0 | 234 / 230 / 67 | 884,327 → 884,327 |
| 1,064,733 → 1,064,733 | 12 | 0 | 96 / 95 / 4 | 1,066,963 → 1,066,963 |
| absent → 1,318,334 | 15 | 15 | 117 / 27 / 5 | absent → 1,320,900 |

Every changed live-only row follows. Both ROM columns agree on all 52 rows.
`changed-live-holds.csv` retains all four hold counts, source lag and both
observed poll origins, compared directly with main's hash-verified
`target/prefix-hunt9/live.csv.zst` and ROM CSV.

| Iteration | Poll episode before → after | Source lag NMIs | ROM holds | Main live holds → candidate |
| --- | --- | ---: | ---: | ---: |
| 380,945 | 380,880 → absent | 0 | 0 | 1 → 0 |
| 380,946 | 380,880 → absent | 1 | 1 | 0 → 1 |
| 380,955 | 380,880 → absent | 0 | 0 | 4 → 0 |
| 380,956 | 380,880 → absent | 5 | 4 | 0 → 4 |
| 381,425 | 380,880 → absent | 0 | 0 | 8 → 0 |
| 381,426 | 380,880 → absent | 8 | 8 | 0 → 8 |
| 383,178 | 380,880 → absent | 0 | 0 | 7 → 0 |
| 383,181 | 380,880 → absent | 7 | 7 | 0 → 7 |
| 383,343 | 380,880 → absent | 7 | 7 | 0 → 7 |
| 383,346 | 380,880 → absent | 0 | 0 | 7 → 0 |
| 383,631 | 380,880 → absent | 59 | 59 | 0 → 59 |
| 383,632 | 380,880 → absent | 6 | 6 | 0 → 6 |
| 383,633 | 380,880 → absent | 23 | 22 | 0 → 22 |
| 383,634 | 380,880 → absent | 1 | 1 | 59 → 1 |
| 383,635 | 380,880 → absent | 1 | 1 | 6 → 1 |
| 462,698 | 462,627 → 462,626 | 0 | 0 | 0 → 1 |
| 462,699 | 462,627 → 462,626 | 0 | 0 | 1 → 0 |
| 462,708 | 462,627 → 462,626 | 0 | 0 | 0 → 4 |
| 462,709 | 462,627 → 462,626 | 0 | 0 | 4 → 0 |
| 463,178 | 462,627 → 462,626 | 0 | 0 | 0 → 8 |
| 463,179 | 462,627 → 462,626 | 0 | 0 | 8 → 0 |
| 627,316 | absent → 627,255 | 0 | 0 | 0 → 1 |
| 627,317 | absent → 627,255 | 1 | 1 | 1 → 0 |
| 627,326 | absent → 627,255 | 0 | 0 | 0 → 4 |
| 627,327 | absent → 627,255 | 5 | 4 | 4 → 0 |
| 627,799 | absent → 627,255 | 0 | 0 | 0 → 8 |
| 627,800 | absent → 627,255 | 8 | 8 | 8 → 0 |
| 629,325 | absent → 627,255 | 7 | 7 | 7 → 0 |
| 629,327 | absent → 627,255 | 0 | 0 | 0 → 7 |
| 629,458 | absent → 627,255 | 0 | 0 | 0 → 7 |
| 629,462 | absent → 627,255 | 7 | 7 | 7 → 0 |
| 735,936 | 735,866 → absent | 0 | 0 | 1 → 0 |
| 735,937 | 735,866 → absent | 1 | 1 | 0 → 1 |
| 735,946 | 735,866 → absent | 0 | 0 | 4 → 0 |
| 735,947 | 735,866 → absent | 5 | 4 | 0 → 4 |
| 736,420 | 735,866 → absent | 8 | 7 | 0 → 7 |
| 736,422 | 735,866 → absent | 0 | 0 | 7 → 0 |
| 1,318,395 | absent → 1,318,334 | 0 | 0 | 0 → 1 |
| 1,318,396 | absent → 1,318,334 | 1 | 1 | 1 → 0 |
| 1,318,405 | absent → 1,318,334 | 0 | 0 | 0 → 4 |
| 1,318,406 | absent → 1,318,334 | 4 | 4 | 4 → 0 |
| 1,318,877 | absent → 1,318,334 | 0 | 0 | 0 → 8 |
| 1,318,878 | absent → 1,318,334 | 8 | 8 | 8 → 0 |
| 1,320,353 | absent → 1,318,334 | 0 | 0 | 0 → 7 |
| 1,320,357 | absent → 1,318,334 | 7 | 7 | 7 → 0 |
| 1,320,542 | absent → 1,318,334 | 7 | 7 | 7 → 0 |
| 1,320,544 | absent → 1,318,334 | 0 | 0 | 0 → 7 |
| 1,320,895 | absent → 1,318,334 | 58 | 58 | 58 → 0 |
| 1,320,896 | absent → 1,318,334 | 6 | 6 | 6 → 0 |
| 1,320,897 | absent → 1,318,334 | 24 | 24 | 24 → 0 |
| 1,320,898 | absent → 1,318,334 | 1 | 1 | 1 → 0 |
| 1,320,899 | absent → 1,318,334 | 1 | 1 | 1 → 0 |

Bounds come from observed native/source poll outcomes inside original source
read episodes, independent main-budget differences, classified execution and
the actual census segments. Changed hold rows do not select the bounds.
The new native-zero/source-nonzero read at **627,255** lies inside
**626,618–627,256**; the new **1,318,334** read lies inside
**1,317,697–1,318,335**. Both precede the original source zero by one
iteration. Their post-seed aftermath ends at actual resyncs **629,845** and
**1,320,900**, retaining the **628,000** and **1,320,000** reseeds explicitly.
The shifted **462,626** observation remains inside **461,989–462,628**.
Its independent execution bound stays at 466,000. Removed **380,880** and
**735,866** retain their original baseline witnesses and endpoints.

The complete compressed timing stream records all ten actual reseeds:
**464,000; 628,000; 629,845; 882,000; 884,000; 884,327; 1,066,000;
1,066,963; 1,320,000; and 1,320,900**.
`poll-witnesses.json` retains bounded original-episode observations, source
intervals and derived execution bounds. The full `live-timing.jsonl.zst`
remains intact. These receipts establish hold attribution; they do not
establish equal receiver state, delivered-command phase, waveform or A/V.

### Verification boundary

All **137 distinct filtered Rust tests** and **eight generator tests** pass.
The Rust set includes the eight new ROM fixtures, guard, Armos, Octorok,
fairy, inactive/return, intro, palette/cache and graphics tests, plus 24
upload-return scheduling tests. The 64-test intro filter passes with its
required `ZELDA3_ASSETS` input. The final product build is warning-free;
library-test compilation retains the same 24 unrelated dead-code warnings.

`verification.json`, `artifact-hashes.json`, authority readbacks and the
unchanged-control receipt bind source, binaries, helpers and evidence.
`git diff --check` passes. Matcher, classifier, P2, original-timing code and
environment-switch declarations are unchanged. No GPU, `--video`, cached-av,
Snes9x process, continuous live replay, full library suite, authority write,
cargo fmt, commit or push occurred. All launched work is joined.

The inherited target cache started at 5.7 GB and was reduced by deleting
only rebuildable files under this worktree's own `target/`. During report
assembly, an overbroad derived witness export temporarily grew target to
3.6 GB. That export was removed and replaced with bounded episode witnesses;
the complete compressed timing stream was preserved. The final target is
below 3 GB. Thus a strict never-exceeded-3-GB constraint was not met during
that temporary export; `witness-export-cleanup.json` records it. Prior hunt
evidence remains read-only.

Sixteen early missing rows remain, including the two upload phase cases and
the mixed 63,995 entry/bus residual. This batch does not establish continuous
live replay through the second song end or cold exact A/V equality.

## Lagless prefix hunt 11

### Fixture fix-up

Fix-up evidence is in `target/prefix-hunt11-fix/`. The new outdoor guard
tails used selector scopes `$07:BA0A/$07:C4D4`, adding 1,090/850 clocks to
selector self-work. They now use their own `$07:BEF8/$07:C92F` fragment
scopes; their ROM block prices and total work are unchanged.

The transition fixture's missing 26,822 clocks are the complete palette-cache
copy at `$02:C769`. Its hunt-10 marker gate suppressed ordinary-timing
ledger work. The atomic wrapper now retires the same generated ROM blocks
in both timing modes; partial continuations retain their existing gate.
The hash-verified archived hunt-10 test binary, with recorded source hashes
matching **4bcd16bf**, reproduces **4,398 versus 31,220** before this repair.
Thus this omission predates hunt 11; the archived cardinal fixture passes.
Neither existing fixture was edited. No fitted price was introduced.

Both named fixtures and all **21 hunt-11 fixtures** pass in default and
`--no-default-features --features parity-debug` builds. The complete requested
filter unions finish at **1,150 passed / 0 failed / 1 inherited ignore** and
**1,031 passed / 0 failed / 0 ignored**, respectively; the ignored test is the
existing host-2293 retained-trace fixture, not either repaired fixture.
All **eight generator tests** pass. Full names are retained in
`default-test-names.txt` and `reduced-test-names.txt`, with per-filter lists.

All three fresh censuses cover **1,473,123 iterations**, gate
**PASS / unexplained 0**. Eight-slice ROM remains **1,864 / 0 / 0**
missing/excess/FP, with **2 / 34 / 142 / 275 / 334** missing before
**94,334 / 141,568 / 228,499 / 380,243 / 461,989**. Its complete predictor
CSV is byte-identical to original hunt 11; runtime/P2 disagreements are zero.

One single seeded live census remains **2,276 / 318 / 85**, also with a
byte-identical hunt-11 predictor CSV. Versus hunt 10, **180** changed hold
rows partition into **156** shared ROM recoveries and **24** live-only rows:
six at poll **462,626 → 462,627**, six at returning **735,866**, and twelve
at removed **1,064,733**. Every live-only change is attributed to `$08:C606`;
unattributed changed/current differential rows and unanchored main-budget
rows are zero. `changed-live-holds.csv`, `live-partition.json` and
`poll-witnesses.json` retain every row, witness and independently derived bound.

Eight-slice `--cpu-video` retains **1,462,997 exact pictures** and all eight
original hunt-11 picture-hash streams byte for byte. All 13 authority hashes
remain unchanged. Target finishes at **2.33 GB allocated / 2.32 GB apparent**
after cache removal and verified lossless archival confined to this target.

An artifact-discovery error invoked a stale product executable without a
CPU-only flag; it failed at GPU initialization. The error is retained in
`stale-executable-error.log`. Subsequent binaries were selected from Cargo
artifact metadata; every requested test and census used library/CPU modes.

### Authority and instruction ownership

Base **4bcd16bf**, branch `codex/prefix-hunt11`. Evidence is under
`target/prefix-hunt11/`. The regenerated full-v2 ledger has **1,473,123
iterations**. The ledger, oracle, ROM, assets and both requested gate helpers
match their authority hashes before and after verification. All 311 entries
in the hunt-9/hunt-10 artifact manifests were verified before reuse. A fresh
94,334-iteration baseline and its four routine profiles equal main's verified
hunt-10 candidate outputs byte for byte.

Pricing uses generated ROM blocks and the executing semantic branches.
The generator adds **532 blocks**, preserves all **4,715** existing blocks
unchanged, and reproduces the checked-in generated file exactly. No scene,
row deficit, sampled count or classification result supplies an instruction
price. Exclusive routine sums conserve the fresh main budgets on every
26-row diagnostic fixture: **92,924 additional clocks** in that cohort.

| Newly priced owner | Admission and ownership boundary |
| --- | --- |
| Y/X outdoor movement response tails, `$07:BEF8` / `$07:C92F` | Common response tails add **1,090 / 850 clocks per execution** after the existing selector/probe/setup work. Guard prefixes stop at the first unadmitted child on complex water, ledge, stair, slope, gravestone and spike paths. |
| Y-item `$07:9B0E`, sword `$07:9CD9` | Lamp/boomerang callers, return/reset paths, sword swing/cooldown, hold/release and controls `$07:9D72` / `$07:9D9F`; separately priced children retain their owners. |
| Named Link residues | `$07:8926`, `$07:92A0`, `$07:92B9`, `$07:E1BE`, `$07:E1D7`, the admitted recoil-state caller fragments, and the **72-clock** `$07:E370` bank prefix. The other 3,028 clocks of the apparent E370 deficit already belong to movement aliases. |
| Map and ancillary callers | `$02:F273` dispatch and all six direction wrappers, including `$02:F2A5`, `$02:F2B3` and `$02:F2C8`. The prior inventory labels the F273/F2B3/F2C8 group the **tile-interaction caller**; its ROM owner is map scroll. Also `$08:FB2B` OAM caller and `$0F:F979` sword-charge sparkle allocation. Map wrapper blocks split at child returns to retain the actual M16/X16 state. |
| Sprite callers and residues | Guard `$41/$42` chase, facing, speed and parry; whirlpool `$BA`; arrow `$1B` movement/damage and tile-return callers; blue archer `$46` caller/draw/shadow; Octorok fire caller and zero/nonzero recoil entry guards; running-boy dispatcher. |

There are **21 new filtered ROM fixture tests**, including **9,126 movement
guard cases**. They execute unmodified ROM instructions, retain call-stack
ownership, exercise the branch alternatives of each admitted fragment, and
compare instruction clocks; state-changing fixtures also compare their
relevant WRAM results. Complex response continuations, nonzero sprite recoil
bodies and complex recoil landing remain unpriced. No complete coverage claim
is made for every sprite residue named in the initial inventory; the early
instruction-shortfall rows close with the admitted owners above.

Two behavior corrections follow actual ROM branches: Z acceleration tests
handler **5**, matching `CMP #$05`, and parry skips action-hitbox overlap when
B frames are negative, matching `BMI`. Both have ROM fixtures. All **18**
route rows with a lower main budget reproduce in bounded baseline/candidate
profiles. Their only negative exclusive owner is `$06:F836`: **22,684 clocks**
of redundant overlap work disappear, while other new markers add 17,404,
giving a net **−5,280** on those rows. `negative-main-reconciliation.json`
retains every row and its conserving routine deltas.

### Classification before 94,334

| Classification | Before | After |
| --- | ---: | ---: |
| Instruction shortfall (a) | 13 | 0 |
| Upload phase (c) | 2 | 2 |
| Mixed instruction/entry/bus (d) | 1 | 0 |
| Total missing rows | **16** | **2** |

The **14 recovered rows** are **35,294; 35,300; 35,304; 35,306; 43,367;
47,033; 48,247; 51,093; 63,077; 63,081; 63,974; 63,995; 64,003; and
64,024**. Pricing alone closes mixed row **63,995**; no entry or bus phase
adjustment is needed. `classification.csv/json` retain each fresh hold count,
main-price delta and remaining instruction gap.

Rows **34,318** and **48,556** retain **40 versus 41** source lag NMIs.
Their main instruction budgets do not change in this batch. The verified
hunt-10 upload diagnosis remains applicable, with non-poll instruction gaps
of **20,730 / 20,684 clocks**. Native ready/ack counts and first-poll phase
were not remeasured here. Those older observations are explicitly identified
as baseline evidence, not fresh phase measurements. Equal receiver state and
first-poll phase remain unproved; this batch admits no upload scheduling fix.

### ROM and CPU picture admission

The requested eight-slice ROM helper completes **1,473,123 iterations**,
gate **PASS / unexplained 0**, at **1,864 / 0 / 0** missing/excess/FP,
versus main's **2,020 / 0 / 0**. Exactly **156** rows gain one correct hold;
none loses a hold. All runtime holds equal P2. Net main instruction pricing
increases by **760,731,654 clocks**, with positive differences on 1,010,269
rows and the 18 explained negative differences above.

| Before iteration | Missing before → after |
| --- | ---: |
| 94,334 | **16 → 2** |
| 141,568 | **50 → 34** |
| 228,499 | **167 → 142** |
| 380,243 | **308 → 275** |
| 461,989 | **376 → 334** |

The fresh eight-slice `--cpu-video` census also completes **1,473,123
iterations**, gate **PASS / unexplained 0**. **1,462,997 pictures remain
exact**; 563 are early, 3,952 late, four show content during source lag, and
zero have other content differences. All eight per-iteration hash streams
equal main's hash-verified streams byte for byte. The streams are retained
losslessly compressed, with pre/post compression identities.

### Single seeded live admission and poll attribution

One full single-process `--live-apu-reads --pacing-predictor` census completes
**1,473,123 iterations**, gate **PASS / unexplained 0**. It is not chunked or
repeated. All four runtime/P2 comparisons agree.

| Lane | Missing before → after | Excess before → after | FP before → after |
| --- | ---: | ---: | ---: |
| ROM | 2,020 → 1,864 | 0 → 0 | 0 → 0 |
| Live | 2,516 → 2,276 | 401 → 318 | 86 → 85 |
| Live minus ROM | 496 → 412 | 401 → 318 | 86 → 85 |

Live improves by **240 missing / 83 excess / 1 FP**. The shared ROM prices
recover 156 missing holds; the net live phase contribution is **−84 / −83 /
−1**. The returning 735,866 episode contributes 12 / 12 / 3; removing the
1,064,733 episode subtracts 96 / 95 / 4. The shifted 462,627 episode preserves
its aggregate contribution. No equal-SPC-phase claim is made.

There are **24 live-only differential hold changes**, all attributed to
original `$08:C606` song-end poll episodes. **Unattributed changed/current
hold-differential rows and unanchored main-budget differences are zero**.
The **180 raw live changes** partition exactly into these 24 plus **156
shared ROM recoveries**. Current live-versus-ROM hold-differential rows
number **177**, versus main's 183. `live-partition.json` conserves all four
totals and the complete live-minus-ROM contributions.

| Observed poll origin before → after | Current hold-differential rows | Changed rows | Current added missing / excess / FP | End before → after, exclusive |
| --- | ---: | ---: | --- | --- |
| 142,087 → 142,087 | 5 | 0 | 9 / 9 / 2 | 144,000 → 144,000 |
| 462,626 → 462,627 | 6 | 6 | 13 / 13 / 3 | 466,000 → 466,000 |
| 627,255 → 627,255 | 10 | 0 | 27 / 27 / 5 | 629,845 → 629,845 |
| absent → 735,866 | 6 | 6 | 12 / 12 / 3 | absent → 738,000 |
| 881,733 → 881,733 | 135 | 0 | 234 / 230 / 67 | 884,327 → 884,327 |
| 1,064,733 → absent | 0 | 12 | 0 / 0 / 0 | 1,066,963 → absent |
| 1,318,334 → 1,318,334 | 15 | 0 | 117 / 27 / 5 | 1,320,900 → 1,320,900 |

Every changed live-only row follows. Both ROM columns agree on all 24 rows.
`changed-live-holds.csv` retains all four hold counts, source lag and both
poll origins, compared directly with main's hash-verified
`target/prefix-hunt10/live.csv` and `target/prefix-hunt10/candidate/rom.csv`.
`all-changed-live-holds.csv` also retains all 180 raw changes.

| Iteration | Poll episode before → after | Source lag NMIs | ROM holds | Main live holds → candidate |
| --- | --- | ---: | ---: | ---: |
| 462,698 | 462,626 → 462,627 | 0 | 0 | 1 → 0 |
| 462,699 | 462,626 → 462,627 | 0 | 0 | 0 → 1 |
| 462,708 | 462,626 → 462,627 | 0 | 0 | 4 → 0 |
| 462,709 | 462,626 → 462,627 | 0 | 0 | 0 → 4 |
| 463,178 | 462,626 → 462,627 | 0 | 0 | 8 → 0 |
| 463,179 | 462,626 → 462,627 | 0 | 0 | 0 → 8 |
| 735,936 | absent → 735,866 | 0 | 0 | 0 → 1 |
| 735,937 | absent → 735,866 | 1 | 1 | 1 → 0 |
| 735,946 | absent → 735,866 | 0 | 0 | 0 → 4 |
| 735,947 | absent → 735,866 | 5 | 4 | 4 → 0 |
| 736,420 | absent → 735,866 | 8 | 7 | 7 → 0 |
| 736,422 | absent → 735,866 | 0 | 0 | 0 → 7 |
| 1,064,797 | 1,064,733 → absent | 0 | 0 | 1 → 0 |
| 1,064,798 | 1,064,733 → absent | 1 | 1 | 0 → 1 |
| 1,064,807 | 1,064,733 → absent | 0 | 0 | 4 → 0 |
| 1,064,808 | 1,064,733 → absent | 5 | 4 | 0 → 4 |
| 1,065,280 | 1,064,733 → absent | 0 | 0 | 7 → 0 |
| 1,065,281 | 1,064,733 → absent | 8 | 7 | 0 → 7 |
| 1,066,543 | 1,064,733 → absent | 0 | 0 | 7 → 0 |
| 1,066,544 | 1,064,733 → absent | 7 | 7 | 0 → 7 |
| 1,066,958 | 1,064,733 → absent | 59 | 59 | 0 → 59 |
| 1,066,959 | 1,064,733 → absent | 6 | 6 | 59 → 6 |
| 1,066,960 | 1,064,733 → absent | 24 | 24 | 6 → 24 |
| 1,066,961 | 1,064,733 → absent | 1 | 1 | 24 → 1 |

Bounds use native/source poll observations inside the original source read
episodes, all independent live-versus-ROM main-budget differences, and the
actual census segment containing the last such difference. Changed holds
do not select the bounds. The fresh native-zero/source-19 poll at **462,627**
lies inside source **461,989–462,628**; the new **735,866** observation lies
inside **735,229–735,867**. Both precede the original zero by one iteration.
The latter's independent main-budget differences end at **736,633**, inside
the actual **736,000–738,000** segment. The removed 1,064,733 episode retains
its verified main-baseline witness and 1,066,963 endpoint. All ten original
source read episodes and their zero rows are independently rechecked against
the fresh timing stream, including episodes with no current native mismatch.

The complete timing stream records ten actual reseeds: **464,000; 628,000;
629,845; 736,000; 738,000; 882,000; 884,000; 884,327; 1,320,000; and
1,320,900**. `poll-witnesses.json` retains bounded observations, independent
main-work ranges, source intervals and old/current bounds. The complete
timing stream and **9,998,339,945-byte**, 1,473,123-row outcome stream remain
losslessly compressed. The outcome archive's decompressed byte count and
SHA-256 equal the streaming original receipt before its redundant sparse
file is removed. These receipts establish attribution, not equal receiver
state, command phase, waveform or A/V.

### Verification boundary

All **90 distinct filtered Rust tests** and **eight generator tests** pass.
The Rust set includes all 21 new fixtures and touched-module regressions.
The product build is warning-free. Library-test compilation retains the
same **24 pre-existing dead-code warnings**. No new warning is introduced.
Matcher, classifier, P2, original-timing code and environment declarations
are unchanged. No GPU, `--video`, cached-av, Snes9x process, continuous live
replay, full library suite, authority write, cargo fmt, commit or push occurred.

`verification.json`, `artifact-hashes.json`, `source.patch`, the authority
readbacks and unchanged-control hashes bind the final source, binaries and
evidence. `git diff --check` passes. All launched work is joined, including
the single live census and its archival consumer; no background work remains.
All 311 prior artifact identities are rechecked unchanged after the run.

The inherited target was **5.1 GB**. Only this worktree's rebuildable cache
and verified redundant evidence representations were removed. Temporary
build cache and the optional full outcome export exceeded the requested
3 GB cap; therefore the strict never-exceeded resource constraint was not
met. The outcome export was archived losslessly before its consumed extents
were released, preserving the complete single live census without a rerun.
The final target is **2.94 GB allocated**, below 3 GB; cleanup and joined
process receipts are recorded with verification.
These results do not establish equal live SPC phase, waveform or cold exact
A/V equality.

## Lagless prefix hunt 12

Diagnosis on `codex/prefix-hunt12`, base **b061a913**. Evidence is under
`target/prefix-hunt12/`. **The milestone is not reached: no behavior fix is
admitted, neither early upload row closes, and no continuous replay is attempted.**
No owners are newly priced and no instruction clocks are recovered.

### Upload protocol and receiver evidence

The refreshed source CPU run reproduces the retained timing phases for both
rows exactly. Its fresh routine totals are **22,096,698 / 22,055,900** clocks.
Both uploads contain **11,694 bytes to $D000**, **1,672 bytes to $2B00**, and a
zero-length **$0800** terminal header. All **26,748** native protocol writes
equal source in order and value, including all four terminal clears. A missing
transfer block does not explain these two observations.

`source-final/apu-bus.csv.zst` retains every selected CPU APUI access, its PC,
bus-start timestamp, value and receiver observation, together with `$4200`
mask/restore stores. The two `source-final/receiver-*.json.zst` files retain
complete source CPU/APU checkpoints, including the SMP continuation and clock
remainder; the adjacent APU-only JSON files are observations, not resumable
checkpoints. `final-protocol.log.zst` retains native protocol accesses and SPC
port observations. `protocol-comparison.json` and
`fresh-poll-conservation.json` bind the stream and instruction comparisons. The new diagnostic switch
`ZELDA3_DEBUG_SPC_MMIO_CYCLES` selects inclusive local SPC-cycle windows
through `crate::debug_env`; it only emits port observations. `commands.json`
retains the source, native, ROM, live and test invocations.

| Iteration | Ready polls, source / native ($00:8891) | Header polls, source / native ($00:88EC) | Byte polls, source / native ($00:88B3/$88C3) | Source minus native poll clocks | Remaining non-poll gap |
| --- | ---: | ---: | ---: | ---: | ---: |
| 34,318 | 11,184 / 10,036 | 52 / 56 | 76,551 / 76,551 | 66,376 | 20,730 |
| 48,556 | 7,122 / 6,137 | 51 / 50 | 76,551 / 76,552 | 57,130 | 20,684 |

The ready loop executes a 16-bit compare, hence two APUI reads per poll.
Taken ready loops cost 58 instruction clocks; taken header/byte loops cost
52. These remain executed protocol work, not prices added to P2. The older
hunt-10 native observation of **7,078** ready polls at 48,556 is stale against
this base; the fresh diagnostic replay reproduces main's predictor rows exactly.

The resident driver, not a fixed latency response, publishes ready. Its
`MOV !$F4,A` at **$11E8** and `MOV !$F5,A` at **$11ED** publish `$AA/$BB` at the
following SPC clocks. Trace PCs $11EB/$11F0 are the post-operand PCs at those
bus operations. The first `$FF` read is at instruction **$08E5** (observed PC
$08E8).

| Iteration / lane | CPU command: source synchronized SPC clock / native scheduled target | First SPC `$FF` read | `$AA` store | `$BB` store |
| --- | ---: | ---: | ---: | ---: |
| 34,318 source | 642,512,165 | 642,527,398 | 642,544,073 | 642,544,080 |
| 34,318 native | 641,145,312 | 641,157,586 | 641,173,948 | 641,173,955 |
| 48,556 source | 925,119,059 | 925,122,447 | 925,139,379 | 925,139,386 |
| 48,556 native | 923,708,480 | 923,711,096 | 923,725,992 | 923,725,999 |

The clock epochs differ; the absolute values are not equal-phase assertions.
Native command targets also do not prove their exact publication micro-step.
Raw SPC MMIO logging observes return-preview clones as well as execution;
`native-receiver.json` explicitly deduplicates identical timestamped ready
stores. Retired native access counts come from the real transfer's audit.

Source begins its first ready read **364 clocks after $02:855D's command
store**, at **V31/H1196** and **V179/H44** respectively. Native bank 0 still
uses its unpositioned output-window command and 386-clock first-read path.
Source receiver PCs at the command are **$0854 / $08F5**, with timer-0
remaining/divider/counter **91/8/1 / 45/2/0**, and `$43` **168 / 32**.
The first native instruction boundary with `$FF` latched is **$0846** at
**641,145,317**, timer-0 **17/3/2**, `$43=112`; the second is **$0876** at
**923,708,483**, timer-0 **51/8/0**, `$43=80`. `native-command-state.json`
identifies both witnesses. The 508 native before/after audio snapshots, DSP
observations and SPC instruction traces are retained losslessly in
`native-state/`; all **485,604,046** decompressed bytes have verified hashes.
These are neighboring instruction-boundary observations, not matched
source/native CPU/SMP checkpoints. Equal receiver state and an exact common
arrival phase remain unproved.

Each source row accepts **40 lag NMIs before masking**, then **one** in the
post-upload suffix: **V251/H822 at $02:83C6** and **V225/H12 at $00:864F**.
Native returns with zero pending instruction holds and retires its closing
NMI. These witnesses identify a return/arrival ownership boundary to resolve;
they do not establish a source-exact replacement scheduler.

The rejected phase trial retained bank 0's remainder using the existing
positioned-bank path. It changed ready polls to **8,830 / 8,033** and masked
callbacks to **23 / 24**, but retained **40 versus 41** accepted holds on both
rows. It is reverted. `phase-trial.json` and `phase-bus.log.zst` retain the
bounded experiment. No fitted delay, forced hold, acknowledgement injection,
sampled price, or ledger-driven polling is introduced. The upstream cause of
the differing driver phase is still open; part B is unfinished.
The lagless bootstrap restores its saved `logical_audio` after the native
CPU/PPU bootstrap, so it does not retain that bootstrap's executing SPC
state. This is an upstream discontinuity to investigate, not a demonstrated
explanation of the two measured poll deficits or an admitted epoch fix.

### ROM, live and prefix results

The eight-slice ROM helper completes **1,473,123 iterations**, gate
**PASS / unexplained 0**. Its entire predictor CSV is byte-identical to
main's `target/prefix-hunt11-fix/candidate/rom.csv`.

| Before iteration | Missing before → after |
| --- | ---: |
| 94,334 | **2 → 2** |
| 141,568 | **34 → 34** |
| 228,499 | **142 → 142** |
| 380,243 | **275 → 275** |
| 461,989 | **334 → 334** |

One single seeded live census also completes **1,473,123 iterations**, gate
**PASS / unexplained 0**. Its entire predictor CSV is byte-identical to main's
`target/prefix-hunt11-fix/live.csv`.

| Lane | Missing before → after | Excess before → after | FP before → after |
| --- | ---: | ---: | ---: |
| ROM | 1,864 → 1,864 | 0 → 0 | 0 → 0 |
| Live | 2,276 → 2,276 | 318 → 318 | 85 → 85 |

The fresh eight-slice `--cpu-video` census completes the same iteration count,
gate **PASS / unexplained 0**. It retains **1,462,997 exact pictures**, 563
early, 3,952 late, four during source lag, 905 polygon-thread and 4,702 torn,
with zero other content differences. The complete result tally equals main;
new per-picture hash streams were not recorded.

### Second-song-end and poll attribution

Both lanes retain **34 missing / 0 excess / 0 FP before 141,568**. In the
second song-end episode, source first returns zero at **142,088**, while native
returns zero at **142,087**. Through the exclusive 144,000 boundary, live adds
**9 missing / 9 excess / 2 FP** over ROM, unchanged from main.

| Iteration | Poll owner / origin | Source lag | ROM holds | Main live → current live |
| --- | --- | ---: | ---: | ---: |
| 142,088 | $08:C606 / 142,087 | 0 | 0 | 5 → 5 |
| 142,089 | $08:C606 / 142,087 | 5 | 5 | 0 → 0 |
| 142,091 | $08:C606 / 142,087 | 0 | 0 | 4 → 4 |
| 142,092 | $08:C606 / 142,087 | 4 | 4 | 3 → 3 |
| 142,094 | $08:C606 / 142,087 | 3 | 3 | 0 → 0 |

There are **zero changed live hold rows**, **zero live-only changed rows**
and **zero unattributed changed rows** versus main. The changed-row
attribution table is empty, as recorded in `changed-live-holds.csv` and
`live-comparison.json`. This is a numerical control, not a recovery or an
assertion of equal CPU/SPC phase.

### Verification boundary

All **110 distinct filtered tests** pass: **43** SPC-driver, **28** native
pacing, **36** APU and **3** source timing-audit tests. The existing cold-native
bootstrap/source-clock fixture also passes independently. The product build
is warning-free; library-test compilation retains **24 pre-existing dead-code
warnings**, with no new warning message.

The full default library suite runs **once**, after all implementation changes,
with the requested assets path: **SNES 505 passed / 7 ignored**, **Zelda 3,008
passed / 3 ignored**. Its only failure is the explicitly permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There are **zero unaccepted failures**. This suite is not all green;
`tests-full-library.log` and `full-library-result.json` retain the complete
execution receipt.

Only diagnostic observation code and this report change. Classifier, matcher,
P2 and instruction pricing remain unchanged. No GPU, `--video`, cached A/V,
Snes9x process, continuous live replay, protected authority write, cargo fmt,
commit or push occurs. Every product invocation uses `--logical-replay`; the
fresh picture check additionally uses `--cpu-video`.

`authority-before.json`, `authority-after.json`, `prior-hashes.json`,
`prior-hashes-after.json`, `source.patch`, `binary.json` and
`artifact-hashes.json` bind the source, controls and evidence. Final identities,
joined process statuses, test results and resource readbacks are retained in
`verification.json` and `resources.json`.

The inherited target was **5.0 GiB**. Rebuildable caches and verified redundant
representations were removed only inside this worktree's target. The inherited
footprint and later rebuilds exceeded 3 GB, so the strict resource constraint
was not maintained. The final target readback is **2.63 GB allocated**, below 3 GB;
resource receipts retain the actual byte counts and observed over-budget samples. All
prior evidence remains unchanged. These diagnostics do not establish a
ROM-exact mechanism fix, equal live SPC phase, waveform or cold A/V equality.

## Lagless prefix hunt 13

Work on `codex/prefix-hunt13`, base **89ea9fbb**. Evidence is under
`target/prefix-hunt13/`. **The bootstrap state substitution is removed, but
the prefix milestone is not reached: both early upload rows still hold 40
versus 41 source lag NMIs.** No owners are newly priced; no main instruction
clocks are recovered. Every main instruction budget and ROM hold count stays
equal to main. No continuous live replay is attempted.

### Bootstrap owner and cold-source fixture

Previously, lagless initialization saved the standalone `logical_audio`, ran
the real native reset/upload/PPU/first-NMI owners, then restored that saved
audio wholesale. The saved driver had been advanced to its first poll and
normalized to cycle zero. It was a different machine, not the executing cold
receiver with a different presentation label.

| Observation | SPC cycle / PC | A / X / Y / SP, hexadecimal | Timer 0 remaining / divider / target / counter | `$43` | CPU conversion clock |
| --- | --- | --- | --- | ---: | --- |
| Former saved standalone state | 0 / `$0879` | `00 / 45 / 01 / CE` | 118 / 0 / 16 / 0, enabled | 0 | absent |
| Executing bootstrap, latest lazy synchronization | 1,379,486 / `$0876` | `3F / 45 / 00 / CF` | 98 / 7 / 16 / 0, enabled | 136 | reference 28,897,054; remainder 266,356; SMP credit 2 |
| Native and continuous source at common CPU deadline 28,938,832 | 1,381,479 / `$0873` | `1F / 45 / 00 / CF` | 25 / 6 / 16 / 0, enabled | 192 | reference 28,938,832; remainder 67,573; SMP credit 0 |

Timers 1 and 2 remain disabled, with zero targets/dividers/counters. Their
remaining cycles are **118 / 6** in the saved standalone state, **98 / 2** at
the lazy native bootstrap observation, and **25 / 9** at the common deadline.
The differing lazy observation timestamps must not be mistaken for a matched
CPU/SMP comparison.

The new `lag_free_bootstrap_keeps_the_continuous_cold_receiver` regression
starts both sides from reset. At the common CPU deadline it compares the
conversion clock, cycle count, all three timers, SMP registers/flags, input
and output ports, every APU RAM byte, and executed DSP register-write history.
They match. The existing cold-upload fixture also retains **356,174 exact
frozen port accesses** and its **1,355,567-cycle / `$0800`** driver-entry
handoff. Fresh source checkpoints cover reset, cold upload return, opening
iterations, iteration 1,400, and the first runtime upload. In this ledger the
first runtime bank upload is **10,425**, opening at source host **11,479**;
iteration 1,400 precedes it.

Lagless now retains the executing audio owner and its suspended SMP
instruction. Its output cursor starts at **1,381,184 SPC clocks**, computed
as `floor(master * 15664 / 328125 / 32) * 32` from the native bootstrap's
**28,933,252** master-clock return. The sample clock retains remainder
**9,459,328 / 10,500,000**, field **80**, and transport host index **81**.
No SMP state is rewound or replaced, and no instruction is finished merely
to select the handoff. Subsequent execution resumes the retained owner.
The **1,379,486 already-executed bootstrap SPC cycles** are preserved; they
are not an added CPU instruction price.

The native event clock retains its existing write-driven legacy DSP model
and separate modern renderer. The source checkpoint has an exact DSP
sidecar, at phase **18** in `receiver-0-main`. Its legacy `dsp_regs` mirror
is stale by design; it is not the source's live DSP register array. The
fixture compares executed DSP writes, while the checkpoints retain the
source pipeline and native observations separately. Equal DSP pipeline,
readback, waveform and full-route SPC phase are not asserted.

### First remaining receiver phase divergence

After preserving bootstrap state, the two short bus windows contain **36
matching port accesses** before the first differing value. Source publishes
SFX **10** on APUI03 at CPU master **29,254,448**, synchronized SPC cycle
**1,396,549**, in iteration 0's first lag NMI. Native's callback transport
does not deliver that value at the corresponding receiver read.

The first differing bus value is the `$F7` dummy read in SPC `$08E5`, cycle
**1,403,457**: source **10**, native **0**. The semantic load at `$08E8`
then reads **10 / 0** at **1,403,463**, and the stable compare reads the same
values at **1,403,468**. Source logs post-operand PCs; bulk native logging
uses the instruction PC. `first-phase-divergence.json` preserves that
normalization and both neighboring streams.

The first demonstrated phase-changing instruction is **SPC `$08F1: BNE +0`**.
Native's instruction trace has `A=Y=0`, starts it at **1,403,472**, and enters
`$08F3` at **1,403,474**. Source's stable nonzero reads imply the taken
four-clock path and `$08F3` at **1,403,476**. The source branch endpoint is
inferred from the observed reads and unchanged ROM opcode timing, not a
recorded source instruction trace. The next port episode confirms the
two-clock split: native/source `$F5` reads at **1,411,429 / 1,411,431**.
This identifies an inherited post-bootstrap NMI/callback transport boundary;
this batch does not replace that scheduler or claim an instruction-by-
instruction full-state proof of the subsequent route.

### Upload receiver observations and suffix NMI

The source command checkpoints at 34,318 and 48,556 are retained hunt-12
authorities with verified hashes; the fresh source-core behavior is unchanged.
The first-runtime-upload source checkpoint is newly generated. Native command
observations name the actual delivery instruction boundary, **0 / 3 / 5**
SPC clocks after their scheduled targets respectively. These are not matched
source/native CPU access checkpoints.

| Iteration / lane | Command SPC cycle or scheduled target | SPC PC | Timer 0 remaining / divider / counter | `$43` |
| --- | ---: | --- | --- | ---: |
| 10,425 source | 196,850,144 | `$09A0` | 32 / 1 / 2 | 176 |
| 10,425 native | 196,849,052 | `$0846` | 100 / 9 / 1 | 176 |
| 34,318 source | 642,512,165 | `$0854` | 91 / 8 / 1 | 168 |
| 34,318 native | 642,526,496 | `$112D` | 93 / 8 / 0 | 104 |
| 48,556 source | 925,119,059 | `$08F5` | 45 / 2 / 0 | 32 |
| 48,556 native | 925,089,696 | `$08FA` | 91 / 13 / 0 | 216 |

Timer 0 remains enabled with target 16 in every row. At iteration 1,400,
source main-entry and native callback-entry observations already differ:
**28,967,327 / 28,967,092** SPC cycles, PCs **`$0FFE / $10D4`**, and
timer-0 state **97/1/1 / 76/15/0**, with `$43=24` on both sides. Their
different observation boundaries do not establish a 235-clock execution
deficit. `source-early-summary.json`, `native-boundary-summary.json` and
`source-command-summary.json` retain the full register/timer observations.

| Iteration | Ready polls, source / main / candidate | Header polls, source / candidate | Byte polls, source / candidate | Main → candidate holds | Main → candidate masked callbacks |
| --- | ---: | ---: | ---: | ---: | ---: |
| 34,318 | 11,184 / 10,036 / 9,851 | 52 / 53 | 76,551 / 76,550 | 40 → 40 | 23 → 23 |
| 48,556 | 7,122 / 6,137 / 10,952 | 51 / 56 | 76,551 / 76,551 | 40 → 40 | 22 → 23 |

The first runtime upload has **5,876 source / 9,689 native ready polls**;
its **58 holds** remain correct. For the two target rows, native first `$FF`
reads occur at **642,538,241 / 925,106,049**, `$AA` ready stores at
**642,554,603 / 925,120,945**, and `$BB` stores seven clocks later. Source's
corresponding first reads are **642,527,398 / 925,122,447**, with `$AA`
stores at **642,544,073 / 925,139,379**. `receiver-comparison.json` binds
the timing audit, receiver observations and deduplicated bus witnesses.

Conserving hunt 12's verified fixed protocol path against the fresh poll
counts gives source-minus-native poll work **77,314 / -222,400 clocks** and
the same non-poll instruction gaps **20,730 / 20,684**. This is an observation
of executed paths, not new P2 pricing. The increased ready-poll count at
48,556 still produces 40 holds; the missing suffix NMI remains a CPU/return
ownership question.

Source still accepts **40 lag NMIs before masking plus one in the suffix**:
**V251/H822 at `$02:83C6`** and **V225/H12 at `$00:864F`**. Native's measured
return has **zero pending** holds and retires its closing NMI. Neither row
closes. No fitted delay, forced hold, acknowledgement injection, sampled
price or ledger-driven polling is introduced.

### ROM, live and CPU-picture admission

The requested eight-slice ROM helper completes **1,473,123 iterations**,
gate **PASS / unexplained 0**. **Zero ROM hold rows change** and runtime
holds equal P2 throughout. There are 39 changed predictor rows: 13 change
masked callback count, with other changes in NMI entry/lag-handler and
resulting elapsed diagnostics. Every main instruction budget is unchanged.

| Before iteration | Missing before → after |
| --- | ---: |
| 94,334 | **2 → 2** |
| 141,568 | **34 → 34** |
| 228,499 | **142 → 142** |
| 380,243 | **275 → 275** |
| 461,989 | **334 → 334** |

| Lane | Missing before → after | Excess before → after | FP before → after |
| --- | ---: | ---: | ---: |
| ROM | 1,864 → 1,864 | 0 → 0 | 0 → 0 |
| Live | 2,276 → 2,276 | 318 → 318 | 85 → 85 |

The fresh eight-slice `--cpu-video` census also passes all **1,473,123
iterations**, unexplained **0**, with **1,462,997 exact pictures**. Its
563 early, 3,952 late, four during source lag, 905 polygon-thread, 4,702
torn and zero other content differences equal main's tally. Separate
per-picture hash streams were not recorded.

### Single seeded live census and poll attribution

Exactly one full, single-process seeded `--live-apu-reads --pacing-predictor`
census completes **1,473,123 iterations**, gate **PASS / unexplained 0**.
It is neither chunked nor repeated. **Zero live hold rows change**, hence
**zero live-only changed hold rows and zero unattributed changed rows**.
The complete changed-row table is empty in `changed-live-holds.csv`.

All **177 current live-versus-ROM hold-differential rows** remain attributed
to the same observed `$08:C606` episodes. Unattributed current rows,
unanchored main-budget differences and runtime/P2 disagreements are zero.
The two actual full timing traces re-observe the zero decisions; inherited
episode evidence is hash-verified. Fresh differences remain within those
recovery bounds, and the actual reseed vector equals main's. `current-live-rom-holds.csv` enumerates all
177 rows; `live-comparison.json` conserves their contributions.

| Poll origin, main → candidate | Current hold-differential rows | Changed hold rows | Added missing / excess / FP |
| --- | ---: | ---: | ---: |
| 142,087 → 142,087 | 5 | 0 | 9 / 9 / 2 |
| 462,627 → 462,627 | 6 | 0 | 13 / 13 / 3 |
| 627,255 → 627,255 | 10 | 0 | 27 / 27 / 5 |
| 735,866 → 735,866 | 6 | 0 | 12 / 12 / 3 |
| 881,733 → 881,733 | 135 | 0 | 234 / 230 / 67 |
| absent → absent at source episode 1,064,096 | 0 | 0 | 0 / 0 / 0 |
| 1,318,334 → 1,318,334 | 15 | 0 | 117 / 27 / 5 |

Source/native first zero in the second song-end episode remains
**142,088 / 142,087**. Episodes starting 94,334, 228,499 and 380,243 retain
zero added hold differences. Totals for live minus ROM remain **412 / 318 /
85**. The live predictor CSV has 85 changed rows in masked callback,
handler/elapsed, preceding-NMI and two raster-wait diagnostic fields; its
main instruction budgets and hold vector are identical to main. CSV byte
identity and equal executing SPC phase are not claimed.

### Verification boundary

All **206 distinct filtered tests** pass: 43 SPC-driver, 28 native-pacing,
130 audio, two lagless-bootstrap and three source timing-audit tests. The
final product build is warning-free. Default library compilation retains
the same **24 pre-existing warning messages** as hunt 12, with no new one.

The requested full default library suite runs **once**, after all
implementation changes, with the requested assets path: **SNES 505 passed /
7 ignored**, **Zelda 3,009 passed / 3 ignored**. Its sole failure is the
explicitly permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There are **zero unaccepted failures**. This suite is not all green;
`tests-full-library.log` and `full-library-result.json` retain its complete
execution result, including exit 101.

Only bootstrap audio ownership/output epoch, diagnostic observation and its
regression fixture change. Pricing, P2, classifier and matcher are unchanged.
No GPU, `--video`, cached A/V, Snes9x process, continuous live replay,
protected-authority write, cargo fmt, commit or push occurs. Every product
invocation uses `--logical-replay`; the picture census adds `--cpu-video`.
These gates do not establish equal full-route SPC/DSP phase, waveform or
cold A/V parity, nor recovery of the two missing upload suffix NMIs.

`authority-before/after.json`, `prior-hashes.json`,
`supplementary-prior-hashes.json`, `commands.json`, `source.patch`, binary
identities and `artifact-hashes.json` bind the evidence. Receiver/short
instruction archives have complete verified member hashes; merged ROM and
live CSVs are retained losslessly compressed. All launched processes are
joined before completion.

The inherited target occupied **4.45 GB allocated**. Rebuildable
Cargo outputs were removed, but builds/replays also exceeded 3 GB afterward
(an observed **4.03 GB** sample). **The strict resource constraint was not
maintained.** Final readback is **2.77 GB allocated**, below 3 GB. All
prior evidence is unchanged. Only rebuildable caches and verified redundant
representations inside this worktree's own target were removed; authority and
prior-evidence hash checks pass again after verification.

## Lagless prefix hunt 13

Follow-up work on `codex/prefix-hunt14`, base **cca69d0a**. Evidence is under
`target/prefix-hunt14/`. **The missing first SFX command is explained and its
suppression is fixed, but same-CPU-position publication remains unresolved.**
No owners are newly priced and no main instruction clocks are recovered.
Both early upload rows still hold **40 versus 41** source lag NMIs.

### Command producer and completed-NMI transport ownership

The fresh source trace identifies `Intro_Init`'s **`$0C:C16D`** store of
**10** to WRAM **`$012F`**, at CPU master **29,153,570**, in iteration 0's
main work. Its first lag NMI reads that byte and publishes it from
**`$00:8127 STA $2143`**, at CPU master **29,254,448** and synchronized SPC
cycle **1,396,549**. The interrupted main PC at that NMI is **`$0C:C1DF`**.
The source opening NMI had previously published zero at **28,897,054**.

Native generates the command in `intro_init` (`ending.rs:3219`). The
producer records **209,686** main work clocks, P2 wall **216,046**, and
publication slot **0**. The first held callback publishes 10; its NMI
samples 10; both queue push and pop contain 10. The command is neither
absent nor coalesced with a later zero.

The loss happens in `AbsoluteDspEventClock::advance_window`. The executing
bootstrap NMI has already written zero to APUI03, but `cpu_port_actions`
still records that completed store. The first logical callback replaces its
new command with this old store decision and then suppresses its delivery
as already handled. Its computed first transport batch is **all `None`**.
Only the following callback's zero reaches the receiver.

`begin_lagless_output_after_bootstrap` now retires those completed-handler
markers. It preserves the executing receiver, suspended instruction,
conversion clock, ports, timers and RAM. The next callback owns its own
stores. No acknowledgement, hold, command value or deadline is fabricated.
The new `bootstrap_nmi_cannot_suppress_the_first_lagless_effect` fixture
retains a CPU/APU owner, completes a bootstrap port store, performs the
handoff and verifies that the executing SPC reads the next effect and its
subsequent one-shot clear. It also checks unchanged machine state and clock
at handoff. The two existing cold-bootstrap fixtures still pass.

`port-stream-comparison.csv` records source iteration 0 and following
iterations alongside both native streams. Source rows contain actual CPU
bus timestamps. Its callback/held/closing columns normalize the source
logger's closing-NMI label, which names the following opening iteration;
`source_iteration` preserves that raw label. Lagless native rows retain
their scheduled SPC deadlines;
they have **no executing CPU master position**, and the report does not
invent one from an inverse clock conversion. The rebuilt baseline binary
hash is exactly hunt 13's **`1c73f1d4...0e28`**. The initially inherited,
different binary is explicitly excluded from all measurements.

### Next first divergence: output epoch versus CPU NMI phase

| Observation | Source | Main baseline | Candidate |
| --- | ---: | ---: | ---: |
| First logical SFX-10 APUI03 publication, SPC cycle | 1,396,549 | absent | 1,381,221 |
| First differing port read, SPC cycle | — | 1,403,457 | 1,385,026 |
| Source / native value at that read | — | 10 / 0 | 0 / 10 |
| Equal bus events before mismatch in the recorded window | — | 36 | 8 |

The candidate's first mismatch is **SPC `$08E5`'s `$F7` dummy read at
1,385,026**, source **0**, native **10**. The semantic load at **1,385,032**
and stable compare at **1,385,037** also read **0 / 10**. Source logs
post-operand PCs; native logs instruction PCs. The comparison normalizes
that reporting difference and compares cycle, direction, address and value.
`first-phase-divergence.json` retains both neighboring streams and the
freshly reproduced main mismatch.

The callback window is **[1,381,184, 1,398,272)**. The transport computes
APUI01/02/03 targets **1,381,214 / 1,381,218 / 1,381,221**.
`lagless_host_port_target_cycles` preserves NMI-relative store spacing but
translates the NMI epoch to the callback's output-window start. That output
boundary is not the accepted CPU NMI boundary. Clearing stale ownership
therefore exposes **15,328 SPC clocks of early publication**, rather than
establishing ROM-exact phase. This difference is an observed deadline gap,
not a fitted price or a proposed delay. No nominal H-clock adjustment,
window shift, forced hold or ledger-driven port injection is introduced.
**Requirement B's same-CPU-position fix is not achieved by this batch.**

### Upload ready polls and the closing suffix NMI

| Iteration | Ready polls, source / main / candidate | Header polls, source / candidate | Byte polls, source / candidate | Holds, main → candidate / source | Masked callbacks, main → candidate |
| --- | ---: | ---: | ---: | ---: | ---: |
| 34,318 | 11,184 / 9,851 / 9,851 | 52 / 53 | 76,551 / 76,550 | 40 → 40 / 41 | 23 → 23 |
| 48,556 | 7,122 / 10,952 / 10,952 | 51 / 56 | 76,551 / 76,551 | 40 → 40 / 41 | 23 → 23 |

The first runtime upload, iteration **10,425**, has **5,876 source /
9,689 main / 9,690 candidate** ready polls and retains **58 holds**.
Its byte polls change **73,007 → 73,006**; total protocol CPU work changes
by **six clocks**, with no instruction-price change. The two target rows'
ready/header/byte counts are unchanged from main.

Their command deliveries also retain main's five observed core fields:
**642,526,499 / `$112D` / timer-0 93,8,16,0 / `$43=104`**, and
**925,089,701 / `$08FA` / timer-0 91,13,16,0 / `$43=216`**; scheduled targets
are **642,526,496 / 925,089,696**. These are instruction-boundary delivery
observations, not equal source/native CPU checkpoints or a full-state proof.

Their native final port clears occur at timed-transport CPU masters
**13,467,901,212 / 19,387,020,481**; the retained return suffix reaches
**13,467,901,386 / 19,387,020,655**, with **zero pending holds** in both
cases. Each still retires a closing NMI. Source's additional suffix NMI
remains at **V251/H822, `$02:83C6`**, and **V225/H12, `$00:864F`**,
respectively. Neither missing row closes. Verified hunt-13 source protocol
and suffix witnesses remain applicable: the authority hashes and source
code are unchanged. `receiver-comparison.json` joins each transfer by its
actual callback endpoint, rather than assuming adjacent upload records
belong to adjacent target rows. The 256 new receiver checkpoint members
are hash-verified in `receiver-observations.tar.zst`.

### ROM, live and CPU-picture admission

The requested eight-slice ROM helper passes **1,473,123 iterations**, gate
**PASS / unexplained 0**. **Zero predictor rows and zero hold rows change**;
all main instruction budgets remain equal to main. Pricing, P2, classifier
and matcher are unchanged.

| Before iteration | Missing before → after |
| --- | ---: |
| 94,334 | **2 → 2** |
| 141,568 | **34 → 34** |
| 228,499 | **142 → 142** |
| 380,243 | **275 → 275** |
| 461,989 | **334 → 334** |

| Lane | Missing before → after | Excess before → after | FP before → after |
| --- | ---: | ---: | ---: |
| ROM | **1,864 → 1,864** | **0 → 0** | **0 → 0** |
| Live | **2,276 → 2,276** | **318 → 318** | **85 → 85** |

The fresh eight-slice `--cpu-video` census passes **1,473,123 iterations**,
unexplained **0**, with **1,462,997 exact pictures**. Its **563 early,
3,952 late, four during source lag, 905 polygon-thread, 4,702 torn and zero
other content differences** equal main's aggregate tallies. Separate
per-picture hash streams are not retained.

### Single seeded live census and poll attribution

Exactly one full, single-process seeded `--live-apu-reads --pacing-predictor`
census completes **1,473,123 iterations**, gate **PASS / unexplained 0**.
It is neither chunked nor repeated. **Zero live hold rows and zero predictor
rows change**, hence **zero live-only changed holds and zero unattributed
changes**. `changed-live-holds.csv` is the complete, empty changed-row table.

All **177 current live-versus-ROM hold-differential rows** retain their
hash-verified `$08:C606` episode attribution. The fresh **6,048 poll rows**,
all ten first-zero decisions and the actual reseed vector match main.
Current unattributed rows, changed main instruction budgets and runtime/P2
disagreements are all zero.

| Source episode / first zero | Native first zero, main → candidate | Current hold-differential rows | Changed holds | Added missing / excess / FP |
| --- | ---: | ---: | ---: | ---: |
| 94,334 / 94,854 | 94,854 → 94,854 | 0 | 0 | 0 / 0 / 0 |
| 141,568 / 142,088 | 142,087 → 142,087 | 5 | 0 | 9 / 9 / 2 |
| 228,499 / 229,035 | 229,035 → 229,035 | 0 | 0 | 0 / 0 / 0 |
| 380,243 / 380,881 | 380,881 → 380,881 | 0 | 0 | 0 / 0 / 0 |
| 461,989 / 462,628 | 462,627 → 462,627 | 6 | 0 | 13 / 13 / 3 |
| 626,618 / 627,256 | 627,255 → 627,255 | 10 | 0 | 27 / 27 / 5 |
| 735,229 / 735,867 | 735,866 → 735,866 | 6 | 0 | 12 / 12 / 3 |
| 881,095 / 881,734 | 881,733 → 881,733 | 135 | 0 | 234 / 230 / 67 |
| 1,064,096 / 1,064,734 | 1,064,734 → 1,064,734 | 0 | 0 | 0 / 0 / 0 |
| 1,317,697 / 1,318,335 | 1,318,334 → 1,318,334 | 15 | 0 | 117 / 27 / 5 |

Live minus ROM totals remain **412 / 318 / 85**. The live CSV and logical
audio-timing command/poll JSONL are **byte-identical to main**; both streams
are freshly generated and their hashes are checked. Those observation
formats do not contain the complete executing SPC state. Their identity
does not contradict the separately recorded early port-bus difference or
establish equal SPC/DSP phase.

### Verification boundary

All **74 distinct filtered tests** pass: **44 SPC-driver, two bootstrap and
28 native-pacing** tests. Product builds are warning-free. The filtered
library build has the same **24 pre-existing warning headers** as hunt 13,
with zero new warnings.

The required full default library suite runs **once**, after all
implementation changes and candidate replays, with the requested assets
path. **SNES: 505 passed / 7 ignored. Zelda: 3,010 passed / 3 ignored.**
Its sole failure is the explicitly permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There are **zero unaccepted failures**. Exit **101** and the complete
results are retained in `tests-full-library.log` and
`full-library-result.json`; the suite is not all green. Zelda's execution
finishes in **1,278.24 seconds**. No implementation changes follow this run.

Source publication, transport ownership, callback scheduling and CPU-picture
acceptance are separate claims. The stale completed-NMI ownership defect is
fixed; equal CPU/SPC phase is not established, and the next first divergence
is earlier than main's. This is not a completed ROM-exact phase repair or
recovery of either missing upload suffix NMI.

The inherited target occupied **3.5 GiB allocated**. Rebuildable outputs
were removed first, but a later observed sample reached **4.05 GB**.
**The strict 3 GB resource bound was not maintained.** After preserving the
verified product binary and compressed default-test binaries and removing
rebuildable Cargo output, final allocation is **2.85 GB**, below 3 GB.
Prior evidence is untouched. Only this worktree's Cargo cache and verified
redundant new observations are removed; authority and prior-evidence hashes
are checked again after testing.

No GPU, `--video`, cached A/V, Snes9x process, continuous live replay,
protected-authority write, new environment switch, cargo fmt, commit or
push occurs. Every product invocation uses `--logical-replay`; the picture
census adds `--cpu-video`. Authority and prior-evidence hashes, binary
identities, command records, `source.patch` and final artifact hashes bind
the evidence. All launched processes are joined before completion.

## Lagless prefix hunt 13

Follow-up diagnosis on `codex/prefix-hunt15`, base **a1d4754c**. Evidence is
under `target/prefix-hunt15/`. **The first post-bootstrap lagless NMI has no
measured native CPU acceptance position. Requirement B therefore stops at
its explicit no-guess boundary.** No transport behavior is changed, no
owners are newly priced, and **zero main instruction clocks are recovered**.

### Acceptance ownership and the four port stores

The cold opening NMI does have an executing CPU owner. The fresh bounded
replay records acceptance at **28,896,206**, **V225/H26**, interrupted
**`$00:8036`**. Its hardware entry takes **62 CPU master clocks**, reaching
the handler vector at **28,896,268**, H88. `NativeNmiContext` then advances
the actual instruction/bus path, including refresh, through `music_header`
and `audio_and_update_gate`. `native_nmi_header_access` publishes each store
through `native_nmi_port_at` at that CPU timeline's current timestamp.
These native transactions match the hash-verified source bus witness.

| NMI / accepted CPU master / V,H | `$2140` store | `$2141` store | `$2142` store | `$2143` store |
| --- | ---: | ---: | ---: | ---: |
| Cold normal, native and source: **28,896,206 / 225,26** | none | 28,896,908 | 28,896,992 | 28,897,054 = 0 |
| First lag, source: **29,253,600 / 225,52** | none | 29,254,302 | 29,254,386 | 29,254,448 = 10 |
| Following normal, source: **29,610,942 / 225,30** | 29,611,512 | 29,611,690 | 29,611,774 | 29,611,836 = 0 |
| First lag / ordinary following normal, native lagless | **unmeasured** | **unmeasured** | **unmeasured** | **unmeasured** |

The first two paths read `$2140` but do not store it: the executing receiver
echo is `$8B`, different from the last-music latch. Absence of that store is
an executed branch decision, not a missing observation. Their stores to
ports 1/2/3 lie **702 / 786 / 848** clocks after acceptance. The following
normal path clears port 0 and reaches its four stores at **570 / 748 / 832 /
894** clocks after acceptance. The normal-versus-lag update-gate branch is
after the audio stores; it does not justify a different guessed audio epoch.

The discontinuity is explicit in `zelda_initialization_code`: after
`begin_lagless_audio_after_bootstrap(28,933,252)`, it drops
`native_startup_cpu` and `native_ppu`. Ordinary lagless main subsequently
records aggregate instruction work and P2 callback slots. `ApuRequest`
retains a slot, value and producer, but neither an interrupted instruction
continuation nor an absolute CPU acceptance timestamp. `run_native_held_frame`
and the ordinary closing callback call the unclocked `interrupt_nmi` path;
they do not pass a `NativeNmiContext` / `CpuMasterTimeline` into the clocked
header. `ledger_master_at_nmi_acceptance` is an instruction-ledger mark,
not a physical beam/master timestamp. Neither that mark nor a P2 callback
count supplies the missing acceptance point.

This diagnosis is scoped to these ordinary lagless callbacks. Retained
native CPU owners elsewhere already have clocked handler entry machinery;
their existence does not establish an entry position for the first lag
callback after the cold owner has been discarded.

### CPU-to-SPC conversion and the unchanged first split

The conversion contract retains **15,664 / 328,125**, the fractional
remainder, the suspended SPC instruction and signed pseudo-step credit.
For source CPU store **29,254,448**, the pure quotient is **1,396,546** with
remainder **17,222**. Synchronizing the executing SPC reaches **1,396,549**.
The three-clock difference is retained instruction credit; the quotient
must not be mislabeled as the observed receiver bus position. Cold native
stores likewise retain their clock checkpoint and match the source's
observed **1,379,478 / 1,379,483 / 1,379,486** execution cycles.

The ordinary callback instead schedules against output window
**[1,381,184, 1,398,272)**. `lagless_host_port_target_cycles` takes
`max(frame_start_cycle, apu_cycle_origin + apu.cycles)` as its epoch, then
adds NMI-relative store differences computed from the nominal
**V225/H84** epoch. H84 is a legacy handler/vector-position assumption,
not the measured hardware acceptance position. The output window starts
on the DSP sample lattice after the bootstrap main-entry handoff; the
first source lag NMI accepts later, in the following field. Reusing the
relative store spacing cannot repair that epoch distinction or recover
the interrupted CPU instruction's acceptance jitter.

Concretely, bootstrap main entry **28,933,252** converts to **1,381,212**;
the output cursor rounds that down to the sample boundary **1,381,184**.
Source's next accepted lag NMI, **29,253,600**, converts to **1,396,505**
before accounting for execution credit. Thus the callback epoch starts
near bootstrap's returned main, rather than the next NMI's accepted CPU
instruction boundary. These conversions describe the observed mismatch;
they are not replacements for the absent native CPU clock.

The native batch remains **none / 0 / 0 / 10**, scheduled at
**1,381,184 / 1,381,214 / 1,381,218 / 1,381,221**. Port 3 is therefore
**15,328 observed SPC clocks** earlier than source. This is a measured
publication gap, never an inserted offset or a price.

Fresh bus-window comparison reproduces **eight equal events**, then the
first difference at **SPC 1,385,026**, instruction **`$08E5`**, dummy read
of **`$F7`**: **source 0 / native 10**. Its semantic load at **1,385,032**
and compare at **1,385,037** also read **0 / 10**. Source logs post-operand
PCs and native logs instruction PCs; the comparison uses cycle, direction,
address and value. `first-phase-divergence.json` preserves the neighboring
events. There is **no later newly reached first divergence**: B's missing
CPU owner prevents a justified transport repair, so this remains the frontier.

### Ready polls and the post-upload suffix

The fresh bounded, seeded ROM run through iteration 48,556 records:

| Iteration | Ready polls, source / main / current | Header polls, source / current | Byte polls, source / current | Holds, main → current / source | Masked callbacks |
| --- | ---: | ---: | ---: | ---: | ---: |
| 34,318 | **11,184 / 9,851 / 9,851** | 52 / 53 | 76,551 / 76,550 | **40 → 40 / 41** | 23 |
| 48,556 | **7,122 / 10,952 / 10,952** | 51 / 56 | 76,551 / 76,551 | **40 → 40 / 41** | 23 |

The native port clears remain at transport CPU masters **13,467,901,212 /
19,387,020,481**; the 174-clock instruction suffix reaches the `$4200`
restore at **13,467,901,386 / 19,387,020,655**, with zero pending holds.
`queue_upload_return_nmi_at_master` subsequently advances the remaining
store access (6), RTS (42) and hardware entry (62) on that transport
timeline. This suffix mechanism does not restore the discarded main CPU
owner or make the source/native upload command epochs equal.

Source's additional accepted suffix NMIs remain **13,467,680,894,
V251/H822, `$02:83C6`**, and **19,387,412,412, V225/H12, `$00:864F`**.
Both missing holds remain open. Source ready-loop and suffix witnesses are
reused only after their artifact hashes, unchanged ledger/oracle authority
hashes and unchanged source-executor code are checked. The native protocol
counts and return timestamps are freshly reproduced in `upload-comparison.json`.

### Admission and verification boundary

Verification results are recorded with their command, binary, authority and
artifact identities under `target/prefix-hunt15/`. No runtime source change
is proposed by this diagnosis. A callback-window shift, nominal H-clock,
ledger-derived epoch or forced hold would bypass the missing executing CPU
acceptance owner and is deliberately not substituted for it.

The unmodified eight-slice ROM helper passes **1,473,123 iterations**, gate
**PASS / unexplained 0**. All predictor rows, hold rows and main instruction
budgets equal main. The fresh CSV is byte-identical to the verified
`target/prefix-hunt14/candidate/rom.csv.zst` after decompression; its raw
hash and verification receipt are retained, and its redundant new copy is
removed to make room for library compilation. Prior evidence is not changed.

| Before iteration | Missing before → after |
| --- | ---: |
| 94,334 | **2 → 2** |
| 141,568 | **34 → 34** |
| 228,499 | **142 → 142** |
| 380,243 | **275 → 275** |
| 461,989 | **334 → 334** |

The fresh eight-slice `--cpu-video` census also passes **1,473,123
iterations**, unexplained **0**, with **1,462,997 exact pictures**. Its
**563 early, 3,952 late, four during source lag, 905 polygon-thread, 4,702
torn and zero other content differences** match main's aggregate tallies.
These are picture-census results, not proof of equal CPU/SPC phase.

Exactly one complete single-process seeded `--live-apu-reads
--pacing-predictor` census passes **1,473,123 iterations**, gate **PASS /
unexplained 0**. The live CSV is byte-identical to main after decompression;
the complete timing stream is also byte-identical to main. Both fresh
streams are hashed before their redundant new copies are removed. The
preserved identical copies remain under `target/prefix-hunt14/`, and
`live-polls.jsonl.zst` retains the current poll observations separately.

| Lane | Missing before → after | Excess before → after | FP before → after |
| --- | ---: | ---: | ---: |
| ROM | **1,864 → 1,864** | **0 → 0** | **0 → 0** |
| Live | **2,276 → 2,276** | **318 → 318** | **85 → 85** |

All **6,048 native poll rows**, ten first-zero decisions and actual reseeds
match main. There are **zero changed live predictor rows, zero changed live
hold rows, zero live-only changed holds and zero unattributed changes**.
`changed-live-holds.csv` is the complete empty changed-row table. The **177
current live-versus-ROM hold differences** retain their verified `$08:C606`
episode attribution; current unattributed rows and runtime/`cost_p2`
disagreements are both zero. The other `predicted_refresh` column is a
diagnostic predictor, not the runtime P2 result used for that check.

| Source episode / first zero | Native first zero, main → current | Current hold-differential rows | Changed holds | Added missing / excess / FP |
| --- | ---: | ---: | ---: | ---: |
| 94,334 / 94,854 | 94,854 → 94,854 | 0 | 0 | 0 / 0 / 0 |
| 141,568 / 142,088 | 142,087 → 142,087 | 5 | 0 | 9 / 9 / 2 |
| 228,499 / 229,035 | 229,035 → 229,035 | 0 | 0 | 0 / 0 / 0 |
| 380,243 / 380,881 | 380,881 → 380,881 | 0 | 0 | 0 / 0 / 0 |
| 461,989 / 462,628 | 462,627 → 462,627 | 6 | 0 | 13 / 13 / 3 |
| 626,618 / 627,256 | 627,255 → 627,255 | 10 | 0 | 27 / 27 / 5 |
| 735,229 / 735,867 | 735,866 → 735,866 | 6 | 0 | 12 / 12 / 3 |
| 881,095 / 881,734 | 881,733 → 881,733 | 135 | 0 | 234 / 230 / 67 |
| 1,064,096 / 1,064,734 | 1,064,734 → 1,064,734 | 0 | 0 | 0 / 0 / 0 |
| 1,317,697 / 1,318,335 | 1,318,334 → 1,318,334 | 15 | 0 | 117 / 27 / 5 |

An earlier live launch aborted after **777 startup rows**, before any
song-end episode: the analysis collector incorrectly treated the always-
zstd timing stream as plain JSONL and closed its pipe. Its failure and
partial output are retained under `aborted-live/` and excluded from census
results. The corrected launch runs the entire route in one process; there
is no repeated complete live census or repeated live song-end experiment.

The strict decimal **3 GB resource bound is not maintained throughout**.
The inherited target starts at **6,068,224,000 allocated bytes**; removing
only rebuildable Cargo output reduces it to **2,854,899,712**. A later ROM
concatenation sample reaches **3,000,508,416**, and a guarded library link
attempt reaches **3,010,256,896**. Those are recorded breaches, not rounded
down to GiB or omitted because cleanup follows. `resources.json` records
the final allocation and maximum observed samples.

Subsequent library builds disable incremental compilation and use host
optimization `z`, one codegen unit and stripped symbols to reduce artifacts.
Compiler-cache files are transparently compressed. Standalone `.rmeta`
sidecars are temporarily reclaimed only after verifying an exact copy in
each corresponding `.rlib` archive; their original bytes and nanosecond
timestamps are restored after the build. This changes host artifact
storage, not emulated instruction prices, runtime code or test selection.
New replay logs are losslessly compressed, and verified duplicate fresh
CSV/timing copies point to the preserved main artifacts. Prior evidence,
ledger and oracle files remain read-only.

All **81 distinct filtered tests** pass: **seven native NMI stack/header
tests, 44 SPC-driver tests, two cold-bootstrap tests and 28 native-pacing
tests**. In particular, `native_nmi_entry_bus_matches_rom_across_refresh_and_field_boundaries`,
`music_header_matches_rom_commands_echoes_and_bus_timestamps` and
`nmi_audio_and_update_gate_matches_rom_data_bus_and_state` verify the
existing clocked machinery independently of the missing ordinary lagless
acceptance owner. The completed-bootstrap suppression fixture also passes.
These fixtures do not manufacture or validate an unmeasured runtime entry.

The product binary is the hash-verified, unchanged main artifact
**`e68c718a...6728e`**, with its warning-free product-build receipt. No new
product build is claimed. The fresh library test build retains exactly
**24 pre-existing warning headers**, identical to main, with **zero new
warnings**. No warning suppression or unrelated source cleanup is applied.

The required full default library suite runs **once**, after the route
censuses and filtered checks, with the requested assets path and the same
compact host-build settings. **SNES: 505 passed / 7 ignored. Zelda: 3,010
passed / 3 ignored.** Its sole failure is the explicitly permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There are **zero unaccepted failures**. Exit **101**, the full log and the
command/environment receipt are preserved; the suite is not all green.
Zelda's execution finishes in **2,144.98 seconds**. No implementation change
follows this run.

The fresh test executables are losslessly archived and their decoded hashes
verified before removing this worktree's rebuildable Cargo cache. Final
target allocation is **2.87 GB**, below the limit; this does not erase the
earlier resource breaches. All **14 checked authorities and 115 checked
prior artifacts** remain byte-identical. Every launched replay and Cargo
session has completed; no owned replay or build process is left running.

This is a **documentation/evidence diagnosis**, not an admitted ROM-exact
publication repair. The clocked bootstrap owner is measured; the first
ordinary post-bootstrap lag NMI and ordinary intro closing callbacks still
lack measured native acceptance positions. Equal command/poll observations
do not establish equal executing SPC state or repair that missing CPU owner.
No new owner, instruction price, P2 rule, classifier, matcher, engine
environment switch, nominal H-clock adjustment, forced hold, source-driven
port injection, GPU execution, cached A/V, external Snes9x process, cargo
fmt, commit or push is introduced. Every product invocation uses
`--logical-replay`; the picture census additionally uses `--cpu-video`.

## Lagless prefix hunt 16

Follow-up investigation on `codex/prefix-hunt16`, base **6e01b922**.
**Incomplete: no acceptance/publication repair is implemented or admitted.**
Evidence is under `target/prefix-hunt16/`. No runtime file changes, new
prices or recovered instruction clocks are proposed.

### Executing owners and the unresolved handoff

The existing clocked implementation is usable only with a live interrupted
CPU context and its executing `CpuMasterTimeline`. `NativeNmiContext`
already owns hardware entry, `music_header` and `audio_and_update_gate`;
`native_nmi_header_access` sends their actual bus timestamps through
`native_nmi_port_at`. The unresolved part is the ordinary lagless caller's
instruction continuation before that entry, rather than its handler's
port-store offsets.

The retained intro path demonstrates the lifecycle that must survive the
handoff. `try_native_startup_main_entry` consumes the returned caller
context. The intro setup and half-slot owners advance that same timeline;
`Startup::advance_intro_clear` retires individual instructions, observes
vblank and completes the interrupted instruction before returning
`IntroClearBoundary::AcceptNmi`. `resume_native_intro_clear_host` then runs
the accepted handler and resumes the suspended main. The ordinary lagless
path instead finishes main atomically, computes its pending holds, and
runs presentation callbacks afterward. Keeping the bootstrap pointer
alone would enable those interruptible main owners without also providing
their resume lifecycle. It is not a completed callback wiring change.

`NativePacing`'s immutable HDMA lease is an aggregate stall composer, not
an instruction continuation. Its block costs, P2 slots and nominal handler
wall durations cannot identify the instruction interrupted at vblank.
This attempt does not convert any of them, or
`ledger_master_at_nmi_acceptance`, into a hardware acceptance timestamp.
Both ordinary post-bootstrap lag/held NMIs and ordinary closing NMIs
remain unmeasured and retain their existing behavior. The clocked cold
bootstrap and the existing upload-return suffix are unchanged.

### Store frontier and upload rows

The following are **hash-verified hunt-15 observations**, not new native
measurements. All 14 checked authority files and 239 manifest entries for
hunts 14/15 match their recorded hashes. The prior product binary also
matches its recorded SHA-256; the runtime source directories have no diff
between its build base `a1d4754c` and this branch's `6e01b922` base.

Source accepts the first lag NMI at CPU master **29,253,600** and stores
SFX-10 at **29,254,448**, synchronizing the executing SPC at **1,396,549**.
This attempt has **not** reproduced that timestamp on a repaired native
timeline. The retained comparison's first difference remains SPC
**1,385,026**, instruction **`$08E5`**, dummy read **`$F7`**, source **0** /
native **10**, after eight equal events. No newly reached next divergence
is claimed.

| Iteration | Ready polls, source / unchanged main | Holds, unchanged main / source | Current repair outcome |
| --- | ---: | ---: | --- |
| 34,318 | 11,184 / 9,851 | 40 / 41 | Unresolved; no new run |
| 48,556 | 7,122 / 10,952 | 40 / 41 | Unresolved; no new run |

The retained source suffix acceptances are **13,467,680,894**, V251/H822,
`$02:83C6`, and **19,387,412,412**, V225/H12, `$00:864F`. Neither missing
hold is fixed. Upload polls, suffix behavior and later bus windows were
not freshly measured in this attempt.

### Census and poll-attribution boundary

These are the verified existing main results. The right-hand values denote
unchanged runtime source, **not fresh hunt-16 census results**.

| Before iteration | Existing main missing → unchanged source |
| --- | ---: |
| 94,334 | 2 → 2 |
| 141,568 | 34 → 34 |
| 228,499 | 142 → 142 |
| 380,243 | 275 → 275 |
| 461,989 | 334 → 334 |

| Lane | Existing missing / excess / FP | Fresh hunt-16 gate |
| --- | ---: | --- |
| ROM | 1,864 / 0 / 0 | Not run |
| Live | 2,276 / 318 / 85 | Not run |
| CPU pictures | 1,462,997 exact | Not run |

| Poll attribution | Result in this attempt |
| --- | --- |
| Implemented live-only hold changes | None; runtime source unchanged |
| Fresh changed-live-row comparison | Not run |
| Fresh `$08:C606` episode measurements | Not run |
| Fresh unexplained-row admission | Not established |

No product build, filtered tests, full default library suite, ROM census,
CPU-picture census or live replay was launched. Prior successful gates
and the prior suite's permitted spotlight failure are not presented as
fresh verification. The requested warning-free build and final full-suite
requirements remain outstanding along with the implementation. No
background process was launched, and prior evidence and authority files
were not modified. **This section records an unfinished attempt, not a
behavior change or an admitted batch.**

## Lagless prefix hunt 17

Bounded hardware-anchor trial on `codex/prefix-hunt17`, base **e8102237**.
**Withdrawn: the candidate never reached an executing product build.**
The two edited runtime/test files are restored byte-for-byte; only this
diagnosis is proposed. Evidence is under `target/prefix-hunt17/`, including
the withdrawn `trial.patch`. No owners are newly priced and **zero main
instruction clocks are recovered**. Fresh restored-baseline checks below
must not be read as admission of the trial.

### Hardware anchor and its unmodeled remainder

The trial removes `max(frame_start_cycle, execution_cycle)` from ordinary
lagless port scheduling. After the clocked bootstrap, the driver's
`host_frame_index` names the next physical field. Its proposed anchor is
that field's **V225/H12**: V225 publication plus the pinned core's
`SNES9X_NMI_ACCEPTANCE_DELAY_MASTER_CYCLES`, rather than the preceding
output window's sample boundary. The slow-ROM hardware entry then takes
**62 clocks**, and the existing handler's instruction work advances through
physical WRAM refresh before CPU-to-SPC conversion. No observed residual,
source acceptance mark, P2 result or ledger timestamp enters that formula.

This is only an earliest-eligibility anchor. Ordinary atomic main has no
retained interrupted instruction, active-DMA continuation, or measured
phase of the `$00:8034/$8036` wait loop. Knowing that a normal NMI finds
main waiting does not identify which LDA/BEQ instruction crosses H12.
The trial therefore does **not** implement the requested ordinary wait-loop
completion. Existing clocked bootstrap/idle owners keep their executing
continuations; no guessed wait phase is attached to the ordinary path.
Neither an exact acceptance repair nor completion of requirement A is
claimed.

All **14 authority hashes and 239 hunt-14/15 artifact hashes** match before
reuse. The source executor and runtime source have no diff from the verified
product build base `a1d4754c`. `anchor-residuals.csv` records all **30 source
port stores** across the eight compared opening NMIs. The table shows port 3;
the other compared stores in each NMI have the same CPU residual. These are
**calculated trial positions**, not stores observed from a candidate binary.
Each calculation holds the source's observed header branch fixed; candidate
branch decisions and receiver equality are unmeasured.

| Source NMI | Source acceptance | Trial anchor | Source `$2143` store | Calculated trial store | Trial minus source, CPU master |
| --- | ---: | ---: | ---: | ---: | ---: |
| Bootstrap normal | 28,896,206 | 28,896,206, retained CPU | 28,897,054 | 28,897,054 | **0** |
| Iteration 0 lag | 29,253,600 | 29,253,560 | 29,254,448 | 29,254,408 | **−40** |
| Iteration 1 normal | 29,610,942 | 29,610,924 | 29,611,836 | 29,611,818 | **−18** |
| Iteration 1 lag | 29,968,338 | 29,968,292 | 29,969,232 | 29,969,186 | **−46** |
| Iteration 2 normal | 30,325,660 | 30,325,656 | 30,326,554 | 30,326,550 | **−4** |
| Iteration 2 lag | 30,683,066 | 30,683,024 | 30,683,960 | 30,683,918 | **−42** |
| Iteration 3 normal | 31,040,392 | 31,040,388 | 31,041,286 | 31,041,282 | **−4** |
| Iteration 3 lag | 31,397,798 | 31,397,756 | 31,398,692 | 31,398,650 | **−42** |

The first lag's proposed stores are anchor **29,253,560** plus
**702 / 786 / 848**, preserving the verified bootstrap header work. Source
accepts **40 clocks later**, at H52. The proposed SFX-10 store's pure SPC
quotient is **1,396,544**; source's store **29,254,448** has quotient
**1,396,546** and observed receiver position **1,396,549**. No candidate
receiver position is measured. Subtracting the quotient from the observed
source position would mix conversion with retained instruction credit.

The fresh three-iteration **restored-baseline** bus replay matches eight
events, then differs at **SPC 1,385,026**, native instruction **`$08E5`**,
dummy read **`$F7`**, **source 0 / native 10**. This reproduces hunt 15's
frontier. **No next first divergence is established for the trial**, and
later hunt-13/14 upload bus windows are not claimed to match an unbuilt
candidate.

### Upload rows and the suffix NMI after restoration

A fresh bounded, seeded replay through iteration 48,556 reproduces:

| Iteration | Ready polls, source / main / restored | Header polls, source / restored | Byte polls, source / restored | Holds, main → restored / source | Masked callbacks |
| --- | ---: | ---: | ---: | ---: | ---: |
| 34,318 | **11,184 / 9,851 / 9,851** | 52 / 53 | 76,551 / 76,550 | **40 → 40 / 41** | 23 |
| 48,556 | **7,122 / 10,952 / 10,952** | 51 / 56 | 76,551 / 76,551 | **40 → 40 / 41** | 23 |

Native clears remain **13,467,901,212 / 19,387,020,481**; the executed
174-clock suffix restores `$4200` at **13,467,901,386 / 19,387,020,655**,
with zero pending holds. The existing upload-return entry still owns its
remaining store access, RTS and hardware entry. Verified source accepts
the additional suffix NMIs at **13,467,680,894**, V251/H822, `$02:83C6`,
and **19,387,412,412**, V225/H12, `$00:864F`. Both missing holds remain
unresolved. `upload-comparison-restored.json` distinguishes these fresh
native observations from the hash-verified source witnesses. There are
no candidate ready-poll, hold or suffix measurements.

### Admission and verification boundary

The fresh default product build does not complete under the storage budget.
Guarded attempts stop around 2.98 GB. A diagnostic build without the guard
reaches **3,091,857,408 allocated bytes**, and a separate library attempt
is observed at **3,059,339,264** before being stopped. These are actual
**3 GB breaches**. Lossless compression of new Cargo files preserves their
contents; deleting `.rmeta` sidecars during pipelined compilation is unsafe
because Cargo still passes those paths to rustc. That attempt produces a
build error and is excluded from verification. All new rebuildable Cargo
output is subsequently removed; prior evidence remains read-only.

An initial ROM helper completes its eight executions but its concatenation
exceeds the storage guard, so it is excluded as a complete gate. The first
CPU-picture launch is stopped before processing iterations by the same
resource check. The replacement ROM run streams both slice CSVs and the
combined CSV through FIFOs and lossless zstd; the supplied helper itself
is unchanged. Excluded attempts and their process exits remain recorded.

The retained product SHA-256 is **`e68c718a...6728e`**. Its complete runtime
source matches the restored branch. The hash-verified hunt-15 library
executable, **`9fcebacf...5af4`**, passes **81 filtered tests** freshly:
seven native NMI stack/header tests, 44 SPC-driver tests, two bootstrap
tests and 28 native-pacing tests. Those runs reuse a verified executable;
they are not a fresh product build or a test of the withdrawn patch.

The replacement eight-slice ROM helper passes **1,473,123 iterations**,
**PASS / unexplained 0**, with **1,864 missing / 0 excess / 0 FP**. Its
entire CSV is byte-identical to main, including every main instruction
budget and every hold row. The raw SHA-256 is **`ac1aad69...21b5`**;
`rom/result.json` records the identity before the redundant compressed
copy is removed. The verified main copy remains under hunt 14.

| Before iteration | Missing before → restored |
| --- | ---: |
| 94,334 | **2 → 2** |
| 141,568 | **34 → 34** |
| 228,499 | **142 → 142** |
| 380,243 | **275 → 275** |
| 461,989 | **334 → 334** |

The fresh eight-slice **`--cpu-video`** census also passes all **1,473,123
iterations**, **unexplained 0**, with **1,462,997 exact pictures**. Its
563 early, 3,952 late, four during-lag, 905 polygon-thread, 4,702 torn and
zero other content differences reproduce main. All eight processes exit
zero. These are restored-runtime picture results, not evidence that the
proposed port anchor preserves them.

Exactly **one** full single-process seeded **`--live-apu-reads
--pacing-predictor`** census completes: **1,473,123 iterations**, **PASS /
unexplained 0**. Its CSV is byte-identical to main's
`target/prefix-hunt14/live.csv.zst` after decompression, raw SHA-256
**`f11bc4b6...c6f4`**. The complete timing stream is also byte-identical,
raw SHA-256 **`286873e5...17af`**. Both identities are recorded before
removing the redundant fresh copies; `live-polls.jsonl.zst` retains the
fresh poll observations.

| Lane | Missing before → restored | Excess before → restored | FP before → restored |
| --- | ---: | ---: | ---: |
| ROM | **1,864 → 1,864** | **0 → 0** | **0 → 0** |
| Live | **2,276 → 2,276** | **318 → 318** | **85 → 85** |

There are **zero changed live predictor rows, zero changed live hold rows,
zero live-only changed holds and zero unattributed changed rows**.
`changed-live-holds.csv` is the complete empty changed-row table. All
**6,048 native poll rows**, ten first-zero decisions and ten actual reseeds
match main. There is consequently no new hold row requiring an additional
`$08:C606` attribution. This compares the restored runtime with main, not
the withdrawn trial with main.

| `$08:C606` source episode / first zero | Native first zero, main → restored | Changed live holds |
| --- | ---: | ---: |
| 94,334 / 94,854 | 94,854 → 94,854 | 0 |
| 141,568 / 142,088 | 142,087 → 142,087 | 0 |
| 228,499 / 229,035 | 229,035 → 229,035 | 0 |
| 380,243 / 380,881 | 380,881 → 380,881 | 0 |
| 461,989 / 462,628 | 462,627 → 462,627 | 0 |
| 626,618 / 627,256 | 627,255 → 627,255 | 0 |
| 735,229 / 735,867 | 735,866 → 735,866 | 0 |
| 881,095 / 881,734 | 881,733 → 881,733 | 0 |
| 1,064,096 / 1,064,734 | 1,064,734 → 1,064,734 | 0 |
| 1,317,697 / 1,318,335 | 1,318,334 → 1,318,334 | 0 |

Fresh SNES library and test-executable builds complete **without warnings**.
The subsequent full-default test build also completes. Its Zelda and SNES
executables are byte-identical to the verified hunt-15 executables. The
Zelda test build retains exactly **24 pre-existing warning headers**, equal
to main; it is not a warning-free product build. To fit the final link,
the build temporarily reclaims 95 redundant metadata sidecars only after
the final test compiler starts with archive dependencies. Their complete
bytes are verified inside each archive, then restored with their original
timestamps before the required Cargo test invocation.

The requested full default library suite executes **once**, after the
route censuses and all runtime edits/restoration, with
`ZELDA3_ASSETS=/Users/missingno/Documents/zelda3-rs/zelda3_assets.dat`
and `cargo test -q --profile parity -p zelda3 -p snes --lib`.
**SNES: 505 passed / 7 ignored. Zelda: 3,010 passed / 3 ignored.** Its sole
failure is the explicitly permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There are **zero unaccepted failures**; exit **101** is retained rather
than reported as an all-green suite. Zelda finishes in **1,616.01 seconds**.
The distinct, long production spotlight bus fixture passes. Once Cargo
starts test execution, redundant metadata is reclaimed again for storage
headroom; no test executable or runtime source is changed.

After the suite, both fresh executable hashes still match the preserved
hunt-15 archives. Only this worktree's newly built Cargo cache is removed.
Final target allocation is **2,877,382,656 bytes**, below 3 GB; this does
**not** erase the earlier breaches. All **14 authority files and 239 prior
manifest entries** remain byte-identical. Every launched replay, collector,
compressor and Cargo session has finished; no owned background process
remains. `verification.json`, `resources.json` and the full-suite log retain
the commands, hashes, exits and verification boundaries.

No runtime change is admitted without an executing candidate and all its
gates. In particular, calculated small residuals do not establish equal
SPC phase, a wait-loop completion, a later bus frontier, or improved upload
holds. No instruction price, classifier, matcher, P2 rule, forced hold,
runtime environment switch, ledger-driven injection, GPU execution,
cached A/V, external Snes9x process, cargo fmt, commit or push is introduced.

## Lagless prefix hunt 18

Hardware-anchor trial on `codex/prefix-hunt18`, base **6ff335b1**.
**Withdrawn after executing the candidate: three existing SPC-driver
fixtures fail.** Both trial files are restored byte-for-byte. The restored
product SHA-256 is **`e68c718a...6728e`**, exactly the retained main binary;
the independently preserved candidate is **`c7d13bdd...edd7f`** at
`target/prefix-hunt18/zelda3-candidate`. All candidate route measurements
below belong to that candidate, not to the restored final runtime.
Evidence is under `target/prefix-hunt18/`. **No owners are newly priced;
zero main instruction clocks are recovered.**

### Hardware anchor, measured phase and next bus owner

Hunt 17's `trial.patch` applies cleanly. It replaces the output-window
origin with physical-field V225/H12 eligibility, then advances the existing
62-clock slow-ROM hardware entry and handler work through WRAM refresh
before CPU-to-SPC conversion. The interrupted ordinary main instruction,
DMA continuation and `$00:8034/$8036` wait-loop phase remain unmodeled.
No measured residual or ledger acceptance timestamp enters the runtime
formula. The patch is retained as evidence, not as an admitted change.

All **389** initial authority/prior-artifact checks, **854** supplementary
hunt-12/13 artifact checks and **five** additional source-witness checks
match. The source CPU/APU core has no diff from the source witness base
`89ea9fbb`; the example's intervening edits only add observations. Fresh
CPU source executions also cover 2,400 and 11,700 host calls, writing only
new hunt-18 evidence.

The opening table reports the last store, `$2143`. Each of the other
compared stores in the same NMI has the same CPU residual. CPU residuals
are calculated from the implemented anchor and handler path; the candidate
SPC queue deadlines are observed from the executing product. The bootstrap
retains its executing CPU owner. Scheduled deadlines and source synchronized
SPC observations have different conversion-credit boundaries, so their
subtraction is not an instruction price or a measured CPU acceptance.

| Source NMI / field | Candidate minus source, CPU master | Candidate queued SPC / source synchronized SPC |
| --- | --- | --- |
| Bootstrap / 80 | 0 | 1,379,486 / 1,379,486 |
| Iteration 0 lag / 81 | -40 | 1,396,544 / 1,396,549 |
| Iteration 1 normal / 82 | -18 | 1,413,606 / 1,413,608 |
| Iteration 1 lag / 83 | -46 | 1,430,666 / 1,430,669 |
| Iteration 2 normal / 84 | -4 | 1,447,725 / 1,447,729 |
| Iteration 2 lag / 85 | -42 | 1,464,785 / 1,464,787 |
| Iteration 3 normal / 86 | -4 | 1,481,845 / 1,481,847 |
| Iteration 3 lag / 87 | -42 | 1,498,905 / 1,498,910 |

The first SFX-10 deadline moves **1,381,221 → 1,396,544** SPC clocks,
recovering **15,323 clocks of publication phase**, while recovering **zero
main instruction clocks**. Source's CPU store is **29,254,448**; the trial
formula gives **29,254,408**, still **40 CPU master clocks early**. The
source pure CPU-to-SPC quotient is **1,396,546**; the synchronized receiver
observation is **1,396,549**. These quantities remain separately recorded.

The candidate matches all **52** bus events in the inherited opening window
**[1,381,400, 1,420,000]**. A fresh contiguous extended comparison matches
**55,828** events, then first differs at **SPC 38,169,267**: native
instruction **`$08E5`**, dummy read **`$F5`**, **source 0 / candidate 5**.
Source logs post-operand `$08E8`; bus equality compares cycle, direction,
address and value. `first-phase-divergence.json` retains both neighboring
streams and the common acknowledgement writes.

The next missing owner is the **NMI's acknowledgement read and branch
decision**, not an additional fitted store offset. In source iteration
**1,915**, physical field **2,236**, source accepts at CPU master
**799,377,296**, V225/H20, only eight clocks after eligibility. Both SPC
streams publish ambient acknowledgement **5 at 38,158,816**. Source reads
`$2141` as **5 at 38,160,627** and clears it from `$00:8110` at CPU master
**799,377,998**, synchronized SPC **38,160,631**. Candidate computes
`frame_writes` at window **[38,145,280, 38,162,336)** with receiver execution
**38,145,300**, before that acknowledgement. It chooses **no port-1 store**;
its next clear is not queued until **38,177,690**. Its port-2/3 paths are
46 CPU clocks shorter than source's clear branch: their actual queue-formula
residuals are **−54**, versus the eligibility-only **−8**. Anchoring stores
does not move the earlier read/branch decision to the executing CPU NMI.
`next-divergence-transport.log` and `next-divergence-port-neighbors.json`
retain that causal split. Ordinary instruction completion is still required
before exact acceptance can be claimed.

### Every compared NMI and the upload windows

`all-compared-nmi-residuals.csv` records **2,462 unique source NMIs**;
`all-compared-store-residuals.csv` records all **9,515 compared source
stores**, including the extended prefix, first runtime upload and both
inherited upload windows.
Each row retains source acceptance/PC, physical field, eligibility residual,
observed candidate queue deadline/value, and the CPU formula position when
an ordinary queued store exists. **Every reconstructed ordinary CPU position converts
to its observed queue deadline exactly.** These are physical-field joins;
subsequent callback-count or branch differences do not establish equal
logical ownership. Absent stores stay explicit.

The following enumerates all **42 NMIs per upload window**: opening,
40 pre-mask lag NMIs and the source suffix NMI. Eligibility deltas compare
V225/H12 with source acceptance; queued store deltas use the candidate branch
and are ordered by ports 1/2/3. A single number means all three agree.
The suffix uses the clocked upload-return owner, so it is not assigned an
ordinary eligibility anchor. At 48,556 the ordinary stores in the same
physical field belong to a different post-return callback; they are not a
matched source suffix. The full CSVs retain that distinction and all values.

| NMI | 34,318 field | Eligibility delta | Queued store deltas 1/2/3 | 48,556 field | Eligibility delta | Queued store deltas 1/2/3 |
| --- | --- | --- | --- | --- | --- | --- |
| Opening | 37621 | -6 | -6 | 54186 | 0 | 0 |
| Lag 1 | 37622 | -8 | -8 | 54187 | -40 | -40 |
| Lag 2 | 37623 | -2 | -2 | 54188 | 0 | 0 |
| Lag 3 | 37624 | -18 | -18 | 54189 | -8 | -8 |
| Lag 4 | 37625 | -30 | -30 | 54190 | -16 | -16 |
| Lag 5 | 37626 | -44 | -44 | 54191 | -2 | -2 |
| Lag 6 | 37627 | -18 | -18 | 54192 | -36 | -36 |
| Lag 7 | 37628 | -14 | -14 | 54193 | -2 | -2 |
| Lag 8 | 37629 | -30 | -30 | 54194 | -34 | -34 |
| Lag 9 | 37630 | -50 | -50 | 54195 | -8 | -8 |
| Lag 10 | 37631 | -16 | -16 | 54196 | -10 | -10 |
| Lag 11 | 37632 | -30 | -30 | 54197 | -12 | -12 |
| Lag 12 | 37633 | -28 | -28 | 54198 | 0 | 0 |
| Lag 13 | 37634 | 0 | 0 | 54199 | -4 | -4 |
| Lag 14 | 37635 | -12 | -12 | 54200 | -6 | -6 |
| Lag 15 | 37636 | -36 | -36 | 54201 | -30 | -30 |
| Lag 16 | 37637 | -22 | -22 | 54202 | -32 | -32 |
| Lag 17 | 37638 | -2 | -2 | 54203 | -38 | -38 |
| Lag 18 | 37639 | 0 | 0 | 54204 | -18 | -18 |
| Lag 19 | 37640 | -28 | -28 | 54205 | -16 | -16 |
| Lag 20 | 37641 | -10 | -10 | 54206 | -20 | -20 |
| Lag 21 | 37642 | -2 | -2 | 54207 | -10 | -10 |
| Lag 22 | 37643 | -46 | -46 | 54208 | 0 | 0 |
| Lag 23 | 37644 | -30 | -30 | 54209 | -12 | -12 |
| Lag 24 | 37645 | -28 | -28 | 54210 | -20 | -20 |
| Lag 25 | 37646 | -36 | -36 | 54211 | -2 | -2 |
| Lag 26 | 37647 | -28 | -28 | 54212 | -24 | -24 |
| Lag 27 | 37648 | -34 | -34 | 54213 | -38 | -38 |
| Lag 28 | 37649 | -10 | -10 | 54214 | -30 | -30 |
| Lag 29 | 37650 | -2 | -2 | 54215 | -6 | -6 |
| Lag 30 | 37651 | -10 | -10 | 54216 | -20 | -20 |
| Lag 31 | 37652 | -18 | -18 | 54217 | -32 | -32 |
| Lag 32 | 37653 | -16 | -16 | 54218 | -54 | -54 |
| Lag 33 | 37654 | -40 | -40 | 54219 | -10 | -10 |
| Lag 34 | 37655 | -12 | -12 | 54220 | -14 | -14 |
| Lag 35 | 37656 | -12 | -12 | 54221 | -12 | -12 |
| Lag 36 | 37657 | -2 | -2 | 54222 | -2 | -2 |
| Lag 37 | 37658 | -46 | -46 | 54223 | -22 | -22 |
| Lag 38 | 37659 | -6 | -6 | 54224 | -6 | absent |
| Lag 39 | 37660 | -22 | -22 | 54225 | -10 | absent |
| Lag 40 | 37661 | 0 | absent | 54226 | -12 | absent |
| Suffix | 37685 | clocked return | ordinary stores absent | 54250 | clocked return | different post-return callback |

### Executed upload polls, holds and return suffix

| Iteration | Ready polls, source / main / candidate | Header polls, source / candidate | Byte polls, source / candidate | Holds, main → candidate / source | Masked callbacks |
| --- | --- | --- | --- | --- | --- |
| 34,318 | **11,184 / 9,851 / 11,871** | 52 / 52 | 76,551 / 76,551 | **40 → 40 / 41** | 23 |
| 48,556 | **7,122 / 10,952 / 10,471** | 51 / 55 | 76,551 / 76,552 | **40 → 40 / 41** | 23 |

| Iteration / lane | First SPC `$FF` read | `$AA` ready store | `$BB` ready store |
| --- | --- | --- | --- |
| 34,318 source | 642,527,398 | 642,544,073 | 642,544,080 |
| 34,318 candidate | 642,529,972 | 642,543,310 | 642,543,317 |
| 48,556 source | 925,122,447 | 925,139,379 | 925,139,386 |
| 48,556 candidate | 925,087,620 | 925,102,516 | 925,102,523 |

These are retired transfer-audit counts and deduplicated SPC bus observations,
including return-preview clones. They are not matched common-CPU access
checkpoints after callback counts diverge. The first runtime upload at
10,425 changes **9,689 → 6,259** ready polls against **5,876** source,
retains **58 holds**, and changes masked callbacks **23 → 22**. The later
native output epoch consequently advances by one callback; the two target
command targets are **642,509,440 / 925,072,640**.

The two final port clears are **13,467,664,648 / 19,386,634,459** CPU master
clocks. Their executed 174-clock suffixes restore `$4200` at
**13,467,664,822 / 19,386,634,633**, both with **zero pending holds**. Source
accepts its additional suffix NMIs at **13,467,680,894**, V251/H822,
`$02:83C6`, and **19,387,412,412**, V225/H12, `$00:864F`. Neither source
suffix hold is recovered. `upload-comparison.json`,
`upload-receiver-bus.json`, the complete bounded bus log, and per-callback
receiver observations retain the measurements.

The executed return-owner port deliveries are also measured, rather than
substituted with ordinary same-field nominal stores:

| Iteration / port | Source synchronized SPC | Candidate instruction-return SPC | Observed SPC delta |
| --- | --- | --- | --- |
| 34,318 / 1 | 642,918,897 | 642,918,132 | -765 |
| 34,318 / 2 | 642,918,905 | 642,918,142 | -763 |
| 34,318 / 3 | 642,918,905 | 642,918,142 | -763 |
| 48,556 / 1 | 925,514,479 | 925,477,350 | -37,129 |
| 48,556 / 2 | 925,514,483 | 925,477,360 | -37,123 |
| 48,556 / 3 | 925,514,489 | 925,477,360 | -37,129 |

All six values match source (ambient 5, then two zeros), but these are
different receiver epochs. `upload-return-store-residuals.json` preserves
the source CPU stores and actual native return-owner observations without
inventing an inverse-converted native CPU acceptance.

### ROM, pictures and instruction conservation

The supplied eight-slice ROM census helper passes **1,473,123 iterations**,
**PASS / unexplained 0**. ROM holds do not change in any row; all main
instruction budgets are unchanged. **440** predictor rows change handler,
refresh/raster or elapsed diagnostics; **69** change masked callback count.
The handler-price functions and P2 rules are unchanged; observed branches
change their executed durations. `instruction-conservation.json` enumerates
the changed fields.

| Before iteration | Missing main → candidate |
| --- | --- |
| 94,334 | 2 → 2 |
| 141,568 | 34 → 34 |
| 228,499 | 142 → 142 |
| 380,243 | 275 → 275 |
| 461,989 | 334 → 334 |

| Lane | Missing main → candidate | Excess main → candidate | FP main → candidate |
| --- | --- | --- | --- |
| ROM | **1,864 → 1,864** | **0 → 0** | **0 → 0** |
| Live | **2,276 → 2,215** | **318 → 257** | **85 → 72** |

The eight-slice **`--cpu-video`** census passes all **1,473,123 iterations**,
**unexplained 0**, preserving **1,462,997 exact pictures**. The 563 early,
3,952 late, four during-lag, 905 polygon-thread, 4,702 torn and zero other
content differences equal main. All eight processes exit zero.

### One live census and complete changed-row attribution

Exactly **one** full single-process seeded **`--live-apu-reads
--pacing-predictor`** census completes, **PASS / unexplained 0**. The baseline
is main's hash-verified `target/prefix-hunt14/live.csv.zst`. All **33** changed
live hold rows are live-only changes because ROM holds are unchanged; all
are attributed to observed **`$08:C606`** song-end predicate differences and
carried state inside the verified recovery intervals. **Zero changed rows
are unattributed.** The **2,494** live main-budget differences are executed
poll-dependent path differences within those same intervals; zero are
unanchored. Runtime holds equal P2 in both complete route lanes.

| Source poll episode / first zero | Main → candidate first zero | Changed hold rows | Delta missing / excess / FP | Recovery bound, exclusive |
| --- | --- | --- | --- | --- |
| 94,334 / 94,854 | 94,854 → 94,854 | 0 | 0 / 0 / 0 | 94,855 |
| 141,568 / 142,088 | 142,087 → 142,088 | 5 | -9 / -9 / -2 | 144,000 |
| 228,499 / 229,035 | 229,035 → 229,035 | 0 | 0 / 0 / 0 | 229,036 |
| 380,243 / 380,881 | 380,881 → 380,881 | 0 | 0 / 0 / 0 | 380,882 |
| 461,989 / 462,628 | 462,627 → 462,628 | 6 | -13 / -13 / -3 | 466,000 |
| 626,618 / 627,256 | 627,255 → 627,256 | 10 | -27 / -27 / -5 | 629,845 |
| 735,229 / 735,867 | 735,866 → 735,867 | 6 | -12 / -12 / -3 | 738,000 |
| 881,095 / 881,734 | 881,733 → 881,733 | 0 | 0 / 0 / 0 | 884,327 |
| 1,064,096 / 1,064,734 | 1,064,734 → 1,064,734 | 0 | 0 / 0 / 0 | 1,066,963 |
| 1,317,697 / 1,318,335 | 1,318,334 → 1,318,336 | 6 | 0 / 0 / 0 | 1,320,900 |

The four corrected early-zero episodes account for **61 fewer missing,
61 fewer excess and 13 fewer false-positive rows**. The last episode moves
from one iteration early to one late; its six changed hold rows conserve
all three totals. Native poll observations change **6,048 → 6,054** rows.
Actual reseeds change from ten to five: **882,000 / 884,000 / 884,327 /
1,320,000 / 1,320,900**. Both full timing streams are read back to observe
these decisions; baseline raw SHA-256 **`286873e5...17af`** is reproduced.
The current live-versus-ROM differential has **150** rows, all inside the
known poll recovery intervals. `live-comparison.json` conserves the full
counts; `changed-live-holds.json` also includes both timing observations
for each of the following complete changed-row set.

| Iteration | Source lag | Main → candidate holds | Poll episode |
| --- | --- | --- | --- |
| 142,088 | 0 | 5 → 0 | $08:C606 / 141,568 |
| 142,089 | 5 | 0 → 5 | $08:C606 / 141,568 |
| 142,091 | 0 | 4 → 0 | $08:C606 / 141,568 |
| 142,092 | 4 | 3 → 4 | $08:C606 / 141,568 |
| 142,094 | 3 | 0 → 3 | $08:C606 / 141,568 |
| 462,699 | 0 | 1 → 0 | $08:C606 / 461,989 |
| 462,700 | 1 | 0 → 1 | $08:C606 / 461,989 |
| 462,709 | 0 | 4 → 0 | $08:C606 / 461,989 |
| 462,710 | 5 | 0 → 4 | $08:C606 / 461,989 |
| 463,179 | 0 | 8 → 0 | $08:C606 / 461,989 |
| 463,180 | 8 | 0 → 8 | $08:C606 / 461,989 |
| 627,316 | 0 | 1 → 0 | $08:C606 / 626,618 |
| 627,317 | 1 | 0 → 1 | $08:C606 / 626,618 |
| 627,326 | 0 | 4 → 0 | $08:C606 / 626,618 |
| 627,327 | 5 | 0 → 4 | $08:C606 / 626,618 |
| 627,799 | 0 | 8 → 0 | $08:C606 / 626,618 |
| 627,800 | 8 | 0 → 8 | $08:C606 / 626,618 |
| 629,325 | 7 | 0 → 7 | $08:C606 / 626,618 |
| 629,327 | 0 | 7 → 0 | $08:C606 / 626,618 |
| 629,458 | 0 | 7 → 0 | $08:C606 / 626,618 |
| 629,462 | 7 | 0 → 7 | $08:C606 / 626,618 |
| 735,936 | 0 | 1 → 0 | $08:C606 / 735,229 |
| 735,937 | 1 | 0 → 1 | $08:C606 / 735,229 |
| 735,946 | 0 | 4 → 0 | $08:C606 / 735,229 |
| 735,947 | 5 | 0 → 4 | $08:C606 / 735,229 |
| 736,420 | 8 | 0 → 7 | $08:C606 / 735,229 |
| 736,422 | 0 | 7 → 0 | $08:C606 / 735,229 |
| 1,318,395 | 0 | 1 → 0 | $08:C606 / 1,317,697 |
| 1,318,397 | 0 | 0 → 1 | $08:C606 / 1,317,697 |
| 1,318,405 | 0 | 4 → 0 | $08:C606 / 1,317,697 |
| 1,318,407 | 0 | 0 → 4 | $08:C606 / 1,317,697 |
| 1,318,877 | 0 | 8 → 0 | $08:C606 / 1,317,697 |
| 1,318,879 | 0 | 0 → 8 | $08:C606 / 1,317,697 |

### Fixture rejection and final verification boundary

The executing candidate's filtered SPC-driver run is **42 passed / 3 failed**.
The failures are:

- `bootstrap_nmi_cannot_suppress_the_first_lagless_effect`: input remains
  zero instead of 10 after the fixture's eight-sample output window.
- `sound_fix1_first_upload_return_publishes_ambient_after_terminal_clears`:
  subsequent input ports remain `[5, 0, 66]` instead of `[0, 0, 0]`.
- `lagless_nmi_stores_survive_output_boundary_and_cycle_rebase`: the
  absolute field deadline precedes the rebased APU origin and
  `target.checked_sub(apu_cycle_origin)` fails.

These expose the trial's changed contract between physical field index,
output-window duration and retained receiver time. The route censuses do
not cover every existing window/rebase fixture. The trial is rejected
without changing those fixtures' expectations. `runtime-restoration.json`
verifies both files against their pre-trial SHA-256 values. The restored
fresh product is byte-identical to main. Restored focused tests pass:
**44 SPC driver + 7 NMI stack + 2 lag-free bootstrap + 28 native pacing =
81 tests**. Candidate and restored product builds and the fresh source
example build have **zero warnings**. The library-test build re-emits the
same **24 pre-existing warning headers** as main; it is not reported as a
warning-free test build.

The required full default library suite runs **once**, after the route
censuses and all runtime edits/restoration, with
`ZELDA3_ASSETS=/Users/missingno/Documents/zelda3-rs/zelda3_assets.dat`
and `cargo test -q --profile parity -p zelda3 -p snes --lib`.
**SNES: 505 passed / 7 ignored. Zelda: 3,010 passed / 3 ignored.** The sole
failure is the explicitly permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There are **zero unaccepted failures**. Exit **101** is retained, rather
than reported as an all-green suite; Zelda finishes in **810.32 seconds**.

Both runtime files are restored, so the final branch adds this diagnosis
only. The single live census above tests the withdrawn candidate; there is
no second restored live replay. The restored product hash, focused tests
and final default suite establish the final-runtime boundary separately.
The largest observed target allocation is approximately **6.6 GB**, and the
final allocation is approximately **4.6 GB**, below the **8 GB** cap. After
the short source process exits zero, 1,998 unreferenced fresh receiver
observations are reclaimed with a path/size/hash manifest; complete bus,
CPU timing/profile streams and selected source boundaries remain. Prior
evidence is unchanged: all **1,248** rechecked entries match, including all
**14 authority files** rechecked again after the suite. Every launched
replay, source executor, Cargo session and collector has finished; no owned
background process remains. Work uses CPU logical modes and Rust source
executors, with no GPU execution, cached A/V or external Snes9x process.
Instruction prices, classifier, matcher, P2 rules and runtime environment
interfaces are unchanged. No cargo fmt, commit or push is performed.

## Lagless prefix hunt 19

Retained-handler trial on `codex/prefix-hunt19`, base **2458a66f**.
**Rejected and surgically restored.** A new ROM-derived regression exposes
an incorrect new-music store duration in the inherited anchor helper.
All five source/fixture files are restored byte-for-byte; the candidate
patches, binary and measurements remain under `target/prefix-hunt19/`.
**No owners are newly priced; zero main instruction clocks are recovered.**
Candidate measurements below are not measurements of the restored runtime.

### Owners evaluated and the source-based rejection

The trial retains a physical-field NMI audio handler, rather than deciding
its stores from the command batch assembled at the output-window start.
`Interrupt_NMI` captures `$012C..$012F` and `$0133/$0131` before consuming
the latches. V225/H12 eligibility plus the executed 62-clock hardware entry
anchors an ordinary handler. Its `$2140/$2141` acknowledgement reads wait
for the executing receiver's pseudo-op boundary; their comparisons choose
the subsequent stores. The two effect ports are stores in this ROM handler.
Native CPU-owned accesses and upload-return vectors retain their owners.
No ledger acceptance timestamp, scene offset, measured deficit, classifier,
matcher or P2 change enters the runtime. Music branch duration distinguishes
the no-store comparison path, repeated-command skip and the `$F2` control
branch, using the ROM's executed instructions.

The generic SPC host callback also permits a CPU read to enqueue a following
store before the same SPC instruction completes. Deferring that queueing
to whole-instruction return had made some stores late. Handler continuations
and absolute store deadlines carry across sample windows. A deadline too
far ahead for the local 32-bit clock stays absolute until a later rebase;
an already-due store is retired in deadline order, retaining its original
target and an explicit `spc_port_overdue` delivery observation when tracing.
Unsigned subtraction from a later APU origin is avoided.

However, `host_port_target_cycles_at_nmi_entry` subtracts forty clocks from
**every** legacy store offset. That is invalid for a new music command:
`$00:80F3 STA $2140` executes **444 CPU master clocks** after vector entry
and, at the eligibility anchor, precedes refresh. The inherited helper uses
**404**, subtracting a refresh that has not occurred.

The hash-verified source execution gives iteration **194**, physical field
**391**, interrupted `$00:8034`, acceptance **140,037,020**, V225/H12.
There is **zero** instruction-acceptance residual. Entry62 plus work444
places the source store at **140,037,526**, H518, before H534 refresh.
Candidate uses **140,037,486**: **40 CPU clocks early**, with queued SPC
**6,685,096** instead of the CPU-derived **6,685,098**. Source synchronized
SPC is independently also **6,685,098** here. This is a handler-work error,
not the unknown interrupted-instruction remainder seen in the first lag.
`source-store-regression.json` retains the witness and arithmetic.

The additional fixture
`lagless_new_music_store_keeps_the_rom_prefix_before_refresh` fails exactly
**(6,685,096, 0, 1) != (6,685,098, 0, 1)**. Its expected timestamp comes
from the source CPU execution and ROM prefix, not candidate output.
The sole live census had already started before this final branch audit.
Correcting the helper afterward would require another live census to admit
the corrected binary. No replacement live census is launched in this batch.
The patch is withdrawn rather than using an older binary's measurements to
validate a corrected runtime. `trial-with-source-regression.patch` includes
the failing source regression; `trial.patch` identifies the measured runtime.

### The three inherited failures and new transport fixtures

All three original behavioral expectations are retained in the trial;
their output-window observation points encode the old epoch:

- **Bootstrap effect:** source iteration 0's field 81 NMI writes SFX 10, and
  the following NMI clears it. Eight samples after a synthetic master-zero
  bootstrap cannot reach that physical NMI. The fixture completes its
  bootstrap store on the hardware timeline and observes successive full
  field windows. Expected **10 then 0**, and the executing SPC read of 10,
  remain unchanged. Source opening/lag stores are in
  `source-read-store-witnesses.json`.
- **Upload-return ambient:** `$88F3..$88FC` terminal clears precede the
  return NMI's ambient 5 store. Source upload bus evidence and ROM
  `$8108 LDA $2141; CMP $0131; BNE; $8110 STZ $2141` require a following
  NMI with echo 5/last 5 to clear it. Expected `[0,5,0,$42]` after return and
  `[0,0,0]` on ports1..3 after that following NMI remain unchanged. A second
  arbitrary 64-sample window is replaced by observation through the next
  anchored handler's last store; it is not a new expected port value.
- **Rebase/window ordering:** the original fixture paired field 302 with an
  APU timestamp above `2^30`, unrelated physical epochs. Its former
  callback-relative translation made that pairing work. The trial uses a
  coherent hardware field near that absolute clock while retaining the
  rebase, split-window and intermediate-latch assertions. The separate
  pre-origin regression covers already-due absolute stores in original
  order, and a deadline beyond the local `u32` horizon must remain pending.

The acknowledgement regression executes `MOV $F5,#5` after the window
starts but before the anchored `$2141` read. Captured source latches are
zero/last5 even though the preassembled typed ambient batch still says 5;
ROM `$8108/$8110` therefore requires the clear. The new deadline regression
requires 5 then its later clear0 to retire in order before the rebased
origin, while preserving an unrepresentable future SFX 10 deadline.
The prior translation/spacing fixture is likewise replaced with assertions
of physical V225/H12 plus entry/work; its callback-origin translation was
an old scheduler contract, not ROM behavior. These candidate fixtures pass,
but are withdrawn with the rejected runtime so final source stays coherent.

### Residuals, completed acknowledgement and next first bus divergence

The first-lag `$2143` queue deadline moves **1,381,221 → 1,396,544**,
recovering **15,323 SPC clocks of publication phase**, while main instruction
recovery remains **zero**. Candidate CPU store **29,254,408** remains
**40 CPU clocks early** against source **29,254,448**. Here that residual
is the unmodeled interrupted instruction, unlike the music-store defect.
All three source store ports share the CPU residual in these opening NMIs.

| Physical field | Candidate minus source, CPU master | Candidate queued SPC / source synchronized SPC, port3 |
| --- | --- | --- |
| 80 | 0 | 1,379,486 / 1,379,486 |
| 81 | -40 | 1,396,544 / 1,396,549 |
| 82 | -18 | 1,413,606 / 1,413,608 |
| 83 | -46 | 1,430,666 / 1,430,669 |
| 84 | -4 | 1,447,725 / 1,447,729 |
| 85 | -42 | 1,464,785 / 1,464,787 |
| 86 | -4 | 1,481,845 / 1,481,847 |
| 87 | -42 | 1,498,905 / 1,498,910 |

The former first split at **SPC 38,169,267** is crossed. At field 2236,
source ambient acknowledgement5 lands at **38,158,816**. The trial's
anchored read is CPU **799,377,912**, synchronized receiver **38,160,627**,
value5; source reads eight CPU clocks later, at **799,377,920**, also
synchronized **38,160,627**. Candidate now queues the clear at **38,160,630**;
source's `$8110` clear is CPU **799,377,998**, synchronized **38,160,631**.
`acknowledgement-witness.json` preserves the observed candidate decisions.

A contiguous comparison of the inherited opening through SPC200,000,000
matches **90,095 bus events** before the next first difference, **SPC
60,509,114**, `$F7` read: **source 0 / candidate 12**. Fresh source execution
covers the surrounding NMIs in `source-next/`. The physical field is
**3546**, source opening iteration 3081; source acceptance is V225/H28,
CPU **1,267,526,764**, sixteen clocks after eligibility. Candidate queues
SFX12 at **60,509,111**, planned CPU **1,267,527,596**; source CPU store is
**1,267,527,612**, pure quotient **60,509,112**, synchronized **60,509,114**.
Source performs the SPC read returning0 before publishing the CPU's12 at
that same synchronized SPC cycle. Remaining ownership includes actual
instruction acceptance and retained CPU-to-SMP credit/port synchronization.
The residual is not an offset added to the runtime. PC naming differs
between traces; bus equality compares cycle/direction/address/value.

`all-compared-store-residuals.csv` records **9,530 source stores** across
**2,467 unique NMIs**, including the fresh five-NMI source witness.
Every reconstructed ordinary candidate CPU position converts to its
observed queue deadline; missing and clocked-return stores stay explicit.
Physical-field joins do not establish equal logical callback ownership.

### Both upload rows and the post-upload suffix NMI

| Iteration | Ready polls source / main / candidate | Header polls source / candidate | Byte polls source / candidate | Holds main → candidate / source | Masked callbacks |
| --- | --- | --- | --- | --- | --- |
| 34,318 | 11,184 / 9,851 / 11,871 | 52 / 51 | 76,551 / 76,551 | 40 → 40 / 41 | 23 |
| 48,556 | 7,122 / 10,952 / 10,471 | 51 / 55 | 76,551 / 76,552 | 40 → 40 / 41 | 23 |

The first runtime upload at 10,425 retains **6,259** ready polls against
source 5,876/main 9,689, **58 holds**, and **22 masked callbacks**. Its
one-callback epoch advance remains; these are executed transfer counts,
not matched CPU checkpoints after callback counts diverge.

| Iteration / lane | First SPC `$FF` read | Ready `$AA` store | Ready `$BB` store |
| --- | --- | --- | --- |
| 34,318 / source | 642,527,398 | 642,544,073 | 642,544,080 |
| 34,318 / candidate | 642,529,970 | 642,543,308 | 642,543,315 |
| 48,556 / source | 925,122,447 | 925,139,379 | 925,139,386 |
| 48,556 / candidate | 925,087,620 | 925,102,516 | 925,102,523 |

Candidate final clears are CPU **13,467,664,596 / 19,386,634,459**;
the executed 174-clock suffix restores `$4200` at **13,467,664,770 /
19,386,634,633**, both with **zero pending holds**. Source accepts its
additional suffix NMIs at **13,467,680,894**, V251/H822, `$02:83C6`, and
**19,387,412,412**, V225/H12, `$00:864F`. Neither missing source suffix hold
is recovered. The return handler also retains its read/decision phases in
the trial, using its clocked vector instead of an ordinary field anchor.

| Iteration / port | Source synchronized SPC | Candidate queued SPC | Difference |
| --- | --- | --- | --- |
| 34,318 / 1 | 642,918,897 | 642,918,130 | -767 |
| 34,318 / 2 | 642,918,905 | 642,918,134 | -771 |
| 34,318 / 3 | 642,918,905 | 642,918,137 | -768 |
| 48,556 / 1 | 925,514,479 | 925,477,349 | -37,130 |
| 48,556 / 2 | 925,514,483 | 925,477,354 | -37,129 |
| 48,556 / 3 | 925,514,489 | 925,477,359 | -37,130 |

All six values agree (ambient 5, then two zeros); these remain different
receiver epochs. The complete 42-NMI tables per upload are retained in
`upload-34318-nmis.json` and `upload-48556-nmis.json`, including opening,
40 pre-mask lag NMIs and the source suffix. Store residuals and absence
are explicit for every port; no ordinary same-field store substitutes for
an unmatched source suffix.

### ROM, pictures and instruction conservation

The eight-slice supplied ROM helper passes **1,473,123 iterations**,
**PASS / unexplained 0**. ROM holds change in **zero rows**; all main
instruction budgets are unchanged. **550** predictor rows change handler,
raster/refresh or elapsed diagnostics, including 83 elapsed-diagnostic rows.
`instruction-conservation.json` enumerates the changed fields.

| Before iteration | Missing main → candidate |
| --- | --- |
| 94,334 | 2 → 2 |
| 141,568 | 34 → 34 |
| 228,499 | 142 → 142 |
| 380,243 | 275 → 275 |
| 461,989 | 334 → 334 |

The eight-slice `--cpu-video` gate passes all **1,473,123 iterations**,
**unexplained 0**, preserving **1,462,997 exact pictures**. The 563 early,
3,952 late, four during-lag, 905 polygon-thread, 4,702 torn and zero other
content differences equal main. Every CPU-picture process exits zero.

### One live census and complete changed-row attribution

Exactly **one** full single-process seeded `--live-apu-reads
--pacing-predictor` census completes: **1,473,123 iterations, PASS /
unexplained 0**. Main's baseline is the hash-verified
`target/prefix-hunt14/live.csv.zst`; its uncompressed CSV identity is
`f11bc4b6...c6f4`. The trial binary is `090f1d6f...fea0c`, retained separately
from the byte-identical restored main product `e68c718a...6728e`.

| Lane | Missing main → candidate | Excess main → candidate | FP main → candidate |
| --- | --- | --- | --- |
| ROM | 1,864 → 1,864 | 0 → 0 | 0 → 0 |
| Live | 2,276 → 2,104 | 318 → 149 | 85 → 14 |

All **176** changed live hold rows are live-only because ROM holds change
in zero rows. Every one is attributed to an observed **`$08:C606`**
song-end predicate change and its carried state within verified recovery
intervals. **Zero changed hold rows are unassigned.** The **4,744** live
main-budget differences are executed poll-path/carry differences in those
same intervals; zero are unanchored. Runtime holds equal P2 in every route
row. Current live-versus-ROM holds differ in **37** rows, all attributed.
`changed-live-holds.json` retains both full timing observations for every
changed row and asserts that their hold values reproduce the CSVs.

| Poll episode / source first zero | Main → candidate first zero | Changed hold rows | Delta missing / excess / FP | Recovery bound, exclusive |
| --- | --- | --- | --- | --- |
| 94,334 / 94,854 | 94,854 → 94,854 | 0 | 0 / 0 / 0 | 94,855 |
| 141,568 / 142,088 | 142,087 → 142,088 | 5 | -9 / -9 / -2 | 144,000 |
| 228,499 / 229,035 | 229,035 → 229,035 | 0 | 0 / 0 / 0 | 229,036 |
| 380,243 / 380,881 | 380,881 → 380,881 | 0 | 0 / 0 / 0 | 380,882 |
| 461,989 / 462,628 | 462,627 → 462,628 | 6 | -13 / -13 / -3 | 466,000 |
| 626,618 / 627,256 | 627,255 → 627,257 | 6 | 0 / 0 / 0 | 629,845 |
| 735,229 / 735,867 | 735,866 → 735,867 | 6 | -12 / -12 / -3 | 738,000 |
| 881,095 / 881,734 | 881,733 → 881,734 | 135 | -234 / -230 / -67 | 884,327 |
| 1,064,096 / 1,064,734 | 1,064,734 → 1,064,735 | 12 | 96 / 95 / 4 | 1,066,963 |
| 1,317,697 / 1,318,335 | 1,318,334 → 1,318,336 | 6 | 0 / 0 / 0 | 1,320,900 |

The 881,095 episode removes 234 missing, 230 excess and 67 FP rows, but the
1,064,096 episode becomes one iteration late and adds 96 missing, 95 excess
and four FP rows. The 626,618 and 1,317,697 episodes move from one early to
one late while conserving totals. All regressions remain explicit.
The net improvement is **172 missing / 169 excess / 71 FP**, not an
admitted runtime result. Native poll observations change **6,048 → 6,057**.
Actual reseeds change from ten to six: **628,000 / 629,845 / 1,066,000 /
1,066,963 / 1,320,000 / 1,320,900**. Both complete traces are read back;
baseline raw SHA-256 **`286873e5...17af`** is reproduced and candidate raw
SHA-256 is **`184adedc...80fb`**. Source first-zero values are independently
checked against the current trace's retained ledger reads for all ten episodes.

The following is the complete changed-live-hold set; each episode denotes
`$08:C606`. Full row observations and provenance are retained in the JSON
and in `changed-live-holds.csv`.

| Iteration | Source lag | Main → candidate holds | Poll episode |
| --- | --- | --- | --- |
| 142,088 | 0 | 5 → 0 | 141,568 |
| 142,089 | 5 | 0 → 5 | 141,568 |
| 142,091 | 0 | 4 → 0 | 141,568 |
| 142,092 | 4 | 3 → 4 | 141,568 |
| 142,094 | 3 | 0 → 3 | 141,568 |
| 462,699 | 0 | 1 → 0 | 461,989 |
| 462,700 | 1 | 0 → 1 | 461,989 |
| 462,709 | 0 | 4 → 0 | 461,989 |
| 462,710 | 5 | 0 → 4 | 461,989 |
| 463,179 | 0 | 8 → 0 | 461,989 |
| 463,180 | 8 | 0 → 8 | 461,989 |
| 627,316 | 0 | 1 → 0 | 626,618 |
| 627,318 | 0 | 0 → 1 | 626,618 |
| 627,326 | 0 | 4 → 0 | 626,618 |
| 627,328 | 0 | 0 → 4 | 626,618 |
| 627,799 | 0 | 8 → 0 | 626,618 |
| 627,801 | 0 | 0 → 8 | 626,618 |
| 735,936 | 0 | 1 → 0 | 735,229 |
| 735,937 | 1 | 0 → 1 | 735,229 |
| 735,946 | 0 | 4 → 0 | 735,229 |
| 735,947 | 5 | 0 → 4 | 735,229 |
| 736,420 | 8 | 0 → 7 | 735,229 |
| 736,422 | 0 | 7 → 0 | 735,229 |
| 881,794 | 0 | 1 → 0 | 881,095 |
| 881,795 | 1 | 0 → 1 | 881,095 |
| 881,804 | 0 | 4 → 0 | 881,095 |
| 881,805 | 5 | 0 → 4 | 881,095 |
| 882,275 | 8 | 0 → 8 | 881,095 |
| 882,276 | 0 | 8 → 0 | 881,095 |
| 882,475 | 2 | 0 → 2 | 881,095 |
| 882,476 | 2 | 0 → 2 | 881,095 |
| 882,477 | 2 | 0 → 2 | 881,095 |
| 882,484 | 0 | 2 → 0 | 881,095 |
| 882,485 | 0 | 2 → 0 | 881,095 |
| 882,486 | 0 | 2 → 0 | 881,095 |
| 882,529 | 2 | 0 → 2 | 881,095 |
| 882,530 | 2 | 0 → 2 | 881,095 |
| 882,531 | 2 | 0 → 2 | 881,095 |
| 882,538 | 0 | 2 → 0 | 881,095 |
| 882,539 | 0 | 2 → 0 | 881,095 |
| 882,540 | 0 | 2 → 0 | 881,095 |
| 882,591 | 2 | 0 → 2 | 881,095 |
| 882,592 | 2 | 0 → 2 | 881,095 |
| 882,593 | 2 | 0 → 2 | 881,095 |
| 882,600 | 0 | 2 → 0 | 881,095 |
| 882,601 | 0 | 2 → 0 | 881,095 |
| 882,602 | 0 | 2 → 0 | 881,095 |
| 882,676 | 2 | 0 → 2 | 881,095 |
| 882,677 | 2 | 0 → 2 | 881,095 |
| 882,678 | 2 | 0 → 2 | 881,095 |
| 882,686 | 0 | 2 → 0 | 881,095 |
| 882,687 | 0 | 2 → 0 | 881,095 |
| 882,688 | 0 | 2 → 0 | 881,095 |
| 882,740 | 2 | 0 → 2 | 881,095 |
| 882,741 | 2 | 0 → 2 | 881,095 |
| 882,742 | 2 | 0 → 2 | 881,095 |
| 882,750 | 0 | 2 → 0 | 881,095 |
| 882,751 | 0 | 2 → 0 | 881,095 |
| 882,752 | 0 | 2 → 0 | 881,095 |
| 882,804 | 2 | 0 → 2 | 881,095 |
| 882,805 | 2 | 0 → 2 | 881,095 |
| 882,806 | 2 | 0 → 2 | 881,095 |
| 882,814 | 0 | 2 → 0 | 881,095 |
| 882,815 | 0 | 2 → 0 | 881,095 |
| 882,816 | 0 | 2 → 0 | 881,095 |
| 882,893 | 2 | 0 → 2 | 881,095 |
| 882,894 | 2 | 0 → 2 | 881,095 |
| 882,895 | 2 | 0 → 2 | 881,095 |
| 882,919 | 0 | 2 → 0 | 881,095 |
| 882,920 | 0 | 2 → 0 | 881,095 |
| 882,921 | 0 | 2 → 0 | 881,095 |
| 882,957 | 2 | 0 → 2 | 881,095 |
| 882,958 | 2 | 0 → 2 | 881,095 |
| 882,959 | 2 | 0 → 2 | 881,095 |
| 882,983 | 0 | 2 → 0 | 881,095 |
| 882,984 | 0 | 2 → 0 | 881,095 |
| 882,985 | 0 | 2 → 0 | 881,095 |
| 883,053 | 2 | 0 → 2 | 881,095 |
| 883,054 | 2 | 0 → 2 | 881,095 |
| 883,055 | 2 | 0 → 2 | 881,095 |
| 883,071 | 0 | 2 → 0 | 881,095 |
| 883,072 | 0 | 2 → 0 | 881,095 |
| 883,073 | 0 | 2 → 0 | 881,095 |
| 883,117 | 2 | 0 → 2 | 881,095 |
| 883,118 | 2 | 0 → 2 | 881,095 |
| 883,119 | 2 | 0 → 2 | 881,095 |
| 883,135 | 0 | 2 → 0 | 881,095 |
| 883,136 | 0 | 2 → 0 | 881,095 |
| 883,137 | 0 | 2 → 0 | 881,095 |
| 883,181 | 2 | 0 → 2 | 881,095 |
| 883,182 | 2 | 0 → 2 | 881,095 |
| 883,183 | 2 | 0 → 2 | 881,095 |
| 883,199 | 0 | 2 → 0 | 881,095 |
| 883,200 | 0 | 2 → 0 | 881,095 |
| 883,201 | 0 | 2 → 0 | 881,095 |
| 883,272 | 2 | 0 → 2 | 881,095 |
| 883,273 | 2 | 0 → 2 | 881,095 |
| 883,274 | 2 | 0 → 2 | 881,095 |
| 883,298 | 0 | 2 → 0 | 881,095 |
| 883,299 | 0 | 2 → 0 | 881,095 |
| 883,300 | 0 | 2 → 0 | 881,095 |
| 883,338 | 2 | 0 → 2 | 881,095 |
| 883,339 | 2 | 0 → 2 | 881,095 |
| 883,340 | 2 | 0 → 2 | 881,095 |
| 883,364 | 0 | 2 → 0 | 881,095 |
| 883,365 | 0 | 2 → 0 | 881,095 |
| 883,366 | 0 | 2 → 0 | 881,095 |
| 883,398 | 2 | 0 → 2 | 881,095 |
| 883,399 | 2 | 0 → 2 | 881,095 |
| 883,400 | 2 | 0 → 2 | 881,095 |
| 883,424 | 0 | 2 → 0 | 881,095 |
| 883,425 | 0 | 2 → 0 | 881,095 |
| 883,426 | 0 | 2 → 0 | 881,095 |
| 883,483 | 2 | 0 → 2 | 881,095 |
| 883,484 | 2 | 0 → 2 | 881,095 |
| 883,485 | 2 | 0 → 2 | 881,095 |
| 883,509 | 0 | 2 → 0 | 881,095 |
| 883,510 | 0 | 2 → 0 | 881,095 |
| 883,511 | 0 | 2 → 0 | 881,095 |
| 883,539 | 2 | 0 → 2 | 881,095 |
| 883,540 | 2 | 0 → 2 | 881,095 |
| 883,541 | 2 | 0 → 2 | 881,095 |
| 883,565 | 0 | 2 → 0 | 881,095 |
| 883,566 | 0 | 2 → 0 | 881,095 |
| 883,567 | 0 | 2 → 0 | 881,095 |
| 883,601 | 2 | 0 → 2 | 881,095 |
| 883,602 | 2 | 0 → 2 | 881,095 |
| 883,603 | 2 | 0 → 2 | 881,095 |
| 883,627 | 0 | 2 → 0 | 881,095 |
| 883,628 | 0 | 2 → 0 | 881,095 |
| 883,629 | 0 | 2 → 0 | 881,095 |
| 883,697 | 2 | 0 → 2 | 881,095 |
| 883,698 | 2 | 0 → 2 | 881,095 |
| 883,699 | 2 | 0 → 2 | 881,095 |
| 883,724 | 0 | 2 → 0 | 881,095 |
| 883,725 | 0 | 2 → 0 | 881,095 |
| 883,726 | 0 | 2 → 0 | 881,095 |
| 883,727 | 2 | 0 → 2 | 881,095 |
| 883,728 | 2 | 0 → 2 | 881,095 |
| 883,729 | 2 | 0 → 2 | 881,095 |
| 883,754 | 0 | 2 → 0 | 881,095 |
| 883,755 | 0 | 2 → 0 | 881,095 |
| 883,756 | 0 | 2 → 0 | 881,095 |
| 883,765 | 2 | 0 → 2 | 881,095 |
| 883,766 | 2 | 0 → 2 | 881,095 |
| 883,767 | 2 | 0 → 2 | 881,095 |
| 883,792 | 0 | 2 → 0 | 881,095 |
| 883,793 | 0 | 2 → 0 | 881,095 |
| 883,794 | 0 | 2 → 0 | 881,095 |
| 883,842 | 7 | 0 → 7 | 881,095 |
| 883,881 | 0 | 7 → 0 | 881,095 |
| 883,977 | 7 | 0 → 7 | 881,095 |
| 884,319 | 0 | 59 → 0 | 881,095 |
| 884,320 | 0 | 6 → 0 | 881,095 |
| 884,321 | 0 | 25 → 0 | 881,095 |
| 884,322 | 59 | 1 → 59 | 881,095 |
| 884,323 | 6 | 1 → 6 | 881,095 |
| 884,324 | 25 | 1 → 25 | 881,095 |
| 1,064,798 | 1 | 1 → 0 | 1,064,096 |
| 1,064,799 | 0 | 0 → 1 | 1,064,096 |
| 1,064,808 | 5 | 4 → 0 | 1,064,096 |
| 1,064,809 | 0 | 0 → 4 | 1,064,096 |
| 1,065,281 | 8 | 7 → 0 | 1,064,096 |
| 1,065,282 | 0 | 0 → 7 | 1,064,096 |
| 1,066,543 | 0 | 0 → 7 | 1,064,096 |
| 1,066,544 | 7 | 7 → 0 | 1,064,096 |
| 1,066,958 | 59 | 59 → 0 | 1,064,096 |
| 1,066,959 | 6 | 6 → 59 | 1,064,096 |
| 1,066,960 | 24 | 24 → 6 | 1,064,096 |
| 1,066,961 | 1 | 1 → 24 | 1,064,096 |
| 1,318,395 | 0 | 1 → 0 | 1,317,697 |
| 1,318,397 | 0 | 0 → 1 | 1,317,697 |
| 1,318,405 | 0 | 4 → 0 | 1,317,697 |
| 1,318,407 | 0 | 0 → 4 | 1,317,697 |
| 1,318,877 | 0 | 8 → 0 | 1,317,697 |
| 1,318,879 | 0 | 0 → 8 | 1,317,697 |

### Final verification boundary

The candidate's focused checks pass **230 tests**: 46 SPC driver, 130 audio,
12 NMI, seven NMI stack, two lag-free bootstrap, 28 native pacing, four SNES
host-port and one scheduled-input-port test. The subsequently added,
ROM-derived new-music prefix fixture fails; it is the explicit source-work
rejection, not an expectation updated to candidate output. After withdrawal,
the restored SPC-driver filter passes all **44 tests**.

Candidate and restored product builds, and the fresh source-example build,
have **zero warnings**. Library-test compilation repeats the same **24
pre-existing warning headers** as main, with zero new headers. Test builds
are therefore not described as warning-free.

The required full default library suite runs **once**, after all runtime
and fixture edits/restoration, with
`ZELDA3_ASSETS=/Users/missingno/Documents/zelda3-rs/zelda3_assets.dat`
and `cargo test -q --profile parity -p zelda3 -p snes --lib`.
**SNES: 505 passed / seven ignored. Zelda: 3,010 passed / three ignored.**
Its sole failure is the explicitly permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There are **zero unaccepted failures**. Exit **101** is retained; Zelda
finishes in **827.09 seconds**. This suite measures the restored main
runtime, not an admitted candidate.

All **519** checked hunt-14/18 artifacts and **14** authority hashes still
match after the work. The pinned base's product source has no difference
from hunt18's restored base. `runtime-restoration.json` verifies all five
restored files; `restored-binary-identity.json` proves the final product is
byte-identical to main. No pricing/P2/classifier/matcher change, ledger or
oracle write, new environment switch, GPU invocation, commit or push is
made. All launched processes are joined. Final `target/` usage is **4.9 GiB**.

The next implementation must derive unrefreshed CPU work per store/path,
including **444** for the new-music `$80F3` store, and pass the retained
source regression before launching its live admission census. Neither the
later bus frontier nor improved live totals validates the withdrawn helper.

## Lagless prefix hunt 20

Physical-field NMI transport on `codex/prefix-hunt20`, base **3ec69b94**.
**Rejected and surgically restored.** The hunt 19 patch, including its
source regression, was reapplied. The measured trial fixes the pre-refresh
store but fails newly executed ROM read/store counterexamples at an exact
refresh endpoint. Candidate measurements below do not describe the restored
runtime. Its five
baseline source hashes and the reused source/baseline artifacts are verified;
the ledger and oracle identities match the regenerated authorities.
**No new main owners are priced; zero main instruction clocks are recovered.**

### Executed work and conditional hardware refresh

`host_port_target_cycles_at_nmi_entry` now takes executed ROM work, rather
than removing a stall from every elapsed store offset. The new-music
`$00:80F3 STA $2140` prefix is **444** CPU master clocks; the clear prefix is
**468**. Later ambient/effect paths likewise exclude hardware stalls.
Every store, as already every retained acknowledgement read, advances from
its vector position through `advance_snes_cpu_master_clock`. A strictly crossed
refresh event adds the pinned **40** clocks; the exact-endpoint omission
below rejects the trial. The existing M1SNES/v2 schedule
carries **H534/H538** through scanlines, fields and the odd short line.
There is no fitted subtraction, per-scene price, threshold, forced hold,
window translation, classifier/matcher/P2 change or runtime use of
`ledger_master_at_nmi_acceptance`.

The source regression at iteration **194**, field **391**, accepts at CPU
**140,037,020**, V225/H12, with zero instruction-acceptance residual.
Entry62 plus work444 gives **140,037,526**, H518, queued SPC **6,685,098**;
the fresh trace observes exactly those values. Source synchronized SPC is
also **6,685,098**. Hunt 19's **140,037,486 / 6,685,096** is corrected:
**40 CPU clocks of executed prefix are restored**, with zero CPU/SPC
residual. H518 precedes even the earliest H534 refresh; the carried model's
field 391/V225 refresh is H538. The new boundary fixture checks accesses
before, across and after refresh, including a next scanline.

The retained handler captures source command/last-command latches before
semantic consumption, reads the executing receiver at anchored bus times,
and runs the ROM comparisons before choosing stores. Ordinary eligibility
is V225/H12 plus the executed **62-clock** hardware entry. Actual interrupted
instruction/DMA acceptance remains unresolved. Clocked upload-return vectors
and native CPU bus owners remain separate. The generic SPC callback can
queue a CPU-selected store before the current SPC instruction returns.
Absolute deadlines survive output windows and rebases, including overdue
stores and deadlines outside the local `u32` horizon. New CPU/store tracing
uses the existing port-trace switch; no environment switch is added.

Hunt 19's fixture justifications remain valid: the bootstrap effect is
observed through the physical NMI (still **10 then 0**); ambient upload
return is observed through the following acknowledgement NMI (still
`[0,5,0,$42]` then zero on ports1..3); the rebase fixture uses coherent
physical epochs. No behavioral expected port value is relaxed.

### Source rejection: an access exactly at refresh

A further CPU-only `RomCpuTimingProbe` executes the actual ROM from
`$00:80C9`, field 391/V225, command1/last0 (store) or command0/echo1 (read).
It independently reproduces the real H74 witness before probing both sides
of the carried H538 boundary. The pinned core revision is
`921f9f7b83660eb44ad263022a57a4a029057c37`. Its `getset.h` memory-access
macro drains `while (CPU.Cycles >= CPU.NextEvent)`; `cpuexec.cpp` adds
`SNES_WRAM_REFRESH_CYCLES` at `HC_WRAM_REFRESH_EVENT`.

| Access | Vector H | Executed work | Candidate CPU | ROM CPU | Candidate minus source |
| --- | --- | --- | --- | --- | --- |
| New music STA | 74 | 444 | 140,037,526 | 140,037,526 | 0 |
| New music STA | 90 | 444 | 140,037,542 | 140,037,542 | 0 |
| New music STA | 94 | 444 | 140,037,546 | 140,037,586 | -40 |
| New music STA | 98 | 444 | 140,037,590 | 140,037,590 | 0 |
| Music LDA | 148 | 390 | 140,037,546 | 140,037,586 | -40 |

Both failing prefixes end **exactly at H538** before event processing.
The helper's `cpu_master_clocks <= clocks_until_refresh` returns that
pre-drain endpoint. The source processes refresh before the next bus access,
so that access is at **H578**. Candidate queued quotient is **6,685,099**;
source CPU-derived quotient is **6,685,100**, for both read and store.
This is an executed-access boundary defect, not interrupted-instruction
acceptance or a fitted residual. Exact-point semantics differ from the
existing work-end helper fixture, which deliberately exposes the pre-drain
endpoint. A repair must distinguish a work endpoint from the next access;
it must not relax that fixture or add a measured offset.

`exact-refresh-endpoint-regression.json` retains all ten executed source
cases, source/header hashes and pinned revision. `source-refresh-probe.log`
records the executions. `trial-with-endpoint-regressions.patch` adds separate
source-derived read/store assertions to the measured trial. These expected
values come from ROM execution, not candidate behavior. An isolated test
binary compiled from the trial's verbatim helpers reproduces both failures
(`tests-exact-refresh-endpoints.log`), without changing the restored source.
The one live census was already in flight when this audit found the defect.
No second live census is launched. All five runtime/fixture files are restored
byte-for-byte to **3ec69b94**; the trial binary, patch and evidence remain.

### Residuals and next first bus divergence

The first-lag port3 queue deadline is **1,381,221 → 1,396,544**,
recovering **15,323 SPC clocks of publication phase**, separate from zero
main instruction recovery. Its planned CPU store is **29,254,408**, still
40 clocks before source **29,254,448** because interrupted-instruction
acceptance is not yet owned.

| Physical field | Candidate minus source, CPU master | Candidate queued / source synchronized SPC, port3 |
| --- | --- | --- |
| 80 | 0 | 1,379,486 / 1,379,486 |
| 81 | -40 | 1,396,544 / 1,396,549 |
| 82 | -18 | 1,413,606 / 1,413,608 |
| 83 | -46 | 1,430,666 / 1,430,669 |
| 84 | -4 | 1,447,725 / 1,447,729 |
| 85 | -42 | 1,464,785 / 1,464,787 |
| 86 | -4 | 1,481,845 / 1,481,847 |
| 87 | -42 | 1,498,905 / 1,498,910 |

The acknowledgement at field 2236 is read at CPU **799,377,912**, receiver
**38,160,627**, value5; source reads eight CPU clocks later, synchronized
to the same receiver cycle and value. Candidate clear0 is CPU
**799,377,990**, queued **38,160,630**; source is **799,377,998**, synchronized
**38,160,631**. The earlier split at SPC 38,169,267 stays crossed.

The freshly compared contiguous bus prefix matches **90,095 events** before
**SPC 60,509,114**, `$F7` read: **source 0 / candidate 12**. Source field 3546
accepts at **1,267,526,764**, H28, sixteen clocks after eligibility.
Candidate port3 is CPU **1,267,527,596**, queue **60,509,111**; source is
CPU **1,267,527,612**, pure quotient **60,509,112**, synchronized
**60,509,114**. Source reads0 before publishing 12 at that synchronized
cycle. This remains interrupted-CPU acceptance and CPU-to-SMP credit/bus
ordering work, not a runtime offset. PC naming differs between trace lanes;
bus equality compares cycle/direction/address/value.

Fresh `spc_nmi_store` observations rebuild **9,530 source-store residuals**
across **2,467 NMIs**, with **zero ambiguous joins**. All **182,575** traced
candidate store positions convert to their observed queue deadlines by the
exact CPU-to-SPC quotient. Twelve absent source stores and six separately
matched clocked-return stores remain explicit. Same physical field does not
prove same logical callback ownership. The full tables and the separately
corrected new-music witness are in `target/prefix-hunt20/`.

### Upload rows and the post-upload suffix NMI

| Iteration | Ready polls source / main / candidate | Header polls source / candidate | Byte polls source / candidate | Holds main → candidate / source | Masked callbacks |
| --- | --- | --- | --- | --- | --- |
| 34,318 | 11,184 / 9,851 / 11,871 | 52 / 51 | 76,551 / 76,551 | 40 → 40 / 41 | 23 |
| 48,556 | 7,122 / 10,952 / 10,471 | 51 / 55 | 76,551 / 76,552 | 40 → 40 / 41 | 23 |

The first upload at 10,425 retains **6,259** ready polls, source 5,876,
**58 holds** and **22 masked callbacks**. These are executed transfer counts,
not matched checkpoints after callback counts diverge.


| Iteration / lane | First SPC `$FF` read | Ready `$AA` store | Ready `$BB` store |
| --- | --- | --- | --- |
| 34,318 / source | 642,527,398 | 642,544,073 | 642,544,080 |
| 34,318 / candidate | 642,529,970 | 642,543,308 | 642,543,315 |
| 48,556 / source | 925,122,447 | 925,139,379 | 925,139,386 |
| 48,556 / candidate | 925,087,620 | 925,102,516 | 925,102,523 |

The two final clears are CPU **13,467,664,596 / 19,386,634,459**;
the executed 174-clock suffix restores `$4200` at **13,467,664,770 /
19,386,634,633**, with **zero pending holds**. Source accepts its additional
suffix NMIs at **13,467,680,894**, V251/H822, `$02:83C6`, and
**19,387,412,412**, V225/H12, `$00:864F`. Neither suffix hold is recovered.

| Iteration / port | Source synchronized SPC | Candidate queued SPC | Difference |
| --- | --- | --- | --- |
| 34,318 / 1 | 642,918,897 | 642,918,130 | -767 |
| 34,318 / 2 | 642,918,905 | 642,918,134 | -771 |
| 34,318 / 3 | 642,918,905 | 642,918,137 | -768 |
| 48,556 / 1 | 925,514,479 | 925,477,349 | -37,130 |
| 48,556 / 2 | 925,514,483 | 925,477,354 | -37,129 |
| 48,556 / 3 | 925,514,489 | 925,477,359 | -37,130 |

All six values match (ambient 5, then two zeros), but the receiver epochs
remain different. The complete 42-NMI tables per upload retain opening,
40 pre-mask lag NMIs and the source suffix, with port residuals/absence
explicit. No same-field ordinary NMI substitutes for an unmatched suffix.

### Diagnostic access-boundary follow-up

`corrected-trial.patch` retains a concrete follow-up, **unapplied and
unadmitted**. A separate `nmi_audio_access_master_clock` resolves a completed
work endpoint, then drains a refresh exactly due before the next bus access.
Both acknowledgement reads and every handler store use it. The existing
work-endpoint helper and its fixture remain byte-for-byte unchanged; the
repair does not reinterpret that fixture as an access timestamp.

Verbatim extracted follow-up helpers pass **all ten** independent ROM
positions: stores from H74/H90/H94/H98/H600 and reads from
H74/H144/H148/H152/H600. `tests-corrected-endpoints.log` retains the results.
The combined patch applies cleanly to the restored base, includes both new
endpoint source regressions and the original H518 regression, but has **no
full integrated build, ROM/picture census, default library suite or live
census**. These diagnostic passes do not admit that patch. A fresh complete
evaluation belongs to a subsequent batch; no second live run is used here.

### ROM, CPU pictures and instruction conservation

The supplied eight-slice ROM helper passes **1,473,123 iterations**, **PASS /
unexplained 0**. ROM holds change in **zero rows**; main instruction budgets
change in **zero rows**. The 550 changed predictor rows retain only handler,
raster/refresh and elapsed diagnostics; `instruction-conservation.json`
enumerates them. No pricing, classifier, matcher or P2 code changes.

| Before iteration | Missing main → measured trial |
| --- | --- |
| 94,334 | 2 → 2 |
| 141,568 | 34 → 34 |
| 228,499 | 142 → 142 |
| 380,243 | 275 → 275 |
| 461,989 | 334 → 334 |

The eight-slice `--cpu-video` gate also passes **1,473,123 iterations**,
**unexplained 0**, with all eight processes exiting zero. It preserves
**1,462,997 exact pictures**, 563 early, 3,952 late, 4 during-lag, 905 polygon-thread,
4,702 torn and 0 other content differences, all equal to main.

### One fresh live census and complete changed-row attribution

Exactly **one** full, single-process seeded `--live-apu-reads
--pacing-predictor` census completes: **1,473,123 iterations**, **PASS /
unexplained 0**. The measured binary is **`9d9998a9…8173da`**, not the later
restored runtime or the diagnostic follow-up patch.

| Lane | Missing main → trial | Excess main → trial | FP main → trial |
| --- | --- | --- | --- |
| ROM | 1,864 → 1,864 | 0 → 0 | 0 → 0 |
| Live | 2,276 → 2,104 | 318 → 149 | 85 → 14 |

All **176** changed live hold rows are live-only, because ROM holds do not
change. Every row is attributed to a **`$08:C606`** song-end decision and
its carried state; **zero rows are unassigned**. Both complete timing traces
are read back. CSV holds are checked against both observations on every
changed row. Changed episodes have an observed first-zero predicate change;
the recovery boundary and following row agree in native context, callback,
held/masked and poll observations. Full boundary observations remain in
`live-comparison.json`, and full changed-row observations in
`changed-live-holds.json`.

The **4,744** live main-budget differences are executed poll-path/carry
changes inside those same verified intervals, with zero unanchored rows.
Runtime holds equal P2 throughout. Current live-versus-ROM holds differ in
**37** rows, all attributed. This is receiver-driven branch behavior, not
new main instruction prices. The net trial change is **172 fewer missing,
169 fewer excess and 71 fewer FP**, and remains a rejected trial result.

| Poll episode / source first zero | Main → trial first zero | Changed hold rows | Delta missing / excess / FP | Recovery bound, exclusive |
| --- | --- | --- | --- | --- |
| 94,334 / 94,854 | 94,854 → 94,854 | 0 | 0 / 0 / 0 | 94,855 |
| 141,568 / 142,088 | 142,087 → 142,088 | 5 | -9 / -9 / -2 | 144,000 |
| 228,499 / 229,035 | 229,035 → 229,035 | 0 | 0 / 0 / 0 | 229,036 |
| 380,243 / 380,881 | 380,881 → 380,881 | 0 | 0 / 0 / 0 | 380,882 |
| 461,989 / 462,628 | 462,627 → 462,628 | 6 | -13 / -13 / -3 | 466,000 |
| 626,618 / 627,256 | 627,255 → 627,257 | 6 | 0 / 0 / 0 | 629,845 |
| 735,229 / 735,867 | 735,866 → 735,867 | 6 | -12 / -12 / -3 | 738,000 |
| 881,095 / 881,734 | 881,733 → 881,734 | 135 | -234 / -230 / -67 | 884,327 |
| 1,064,096 / 1,064,734 | 1,064,734 → 1,064,735 | 12 | 96 / 95 / 4 | 1,066,963 |
| 1,317,697 / 1,318,335 | 1,318,334 → 1,318,336 | 6 | 0 / 0 / 0 | 1,320,900 |

The 881,095 episode improves 234 missing/230 excess/67 FP; the 1,064,096
episode regresses 96 missing/95 excess/4 FP as the poll becomes one iteration
late. The 626,618 and 1,317,697 episodes cross from early to late while
conserving totals. All regressions remain explicit.

Observed native poll rows are **6,048 → 6,057**. The six actual trial reseeds are **628,000 / 629,845 / 1,066,000 / 1,066,963 / 1,320,000 / 1,320,900**.
Baseline CSV raw SHA-256 is `f11bc4b607fc228fb6e2a181b332cf4932772889c742be8d2985ed6ce9e7c6f4`; baseline timing raw SHA-256 is `286873e50117590b175154c1bb1d14265832c78b9b7cc3259ee1e5f6130217af`; trial timing raw SHA-256 is `184adedcf95d5176b3dff5350ae1774f821ed6407d1d1881d167ca803ac480fb`. Source first-zero values are independently reproduced from retained ledger reads in the fresh trace for all ten episodes.

The following is the complete changed-live-hold set. Every episode denotes
`$08:C606`; the CSV/JSON retain all rows and their provenance.

| Iteration | Source lag | Main → trial holds | Poll episode |
| --- | --- | --- | --- |
| 142,088 | 0 | 5 → 0 | 141,568 |
| 142,089 | 5 | 0 → 5 | 141,568 |
| 142,091 | 0 | 4 → 0 | 141,568 |
| 142,092 | 4 | 3 → 4 | 141,568 |
| 142,094 | 3 | 0 → 3 | 141,568 |
| 462,699 | 0 | 1 → 0 | 461,989 |
| 462,700 | 1 | 0 → 1 | 461,989 |
| 462,709 | 0 | 4 → 0 | 461,989 |
| 462,710 | 5 | 0 → 4 | 461,989 |
| 463,179 | 0 | 8 → 0 | 461,989 |
| 463,180 | 8 | 0 → 8 | 461,989 |
| 627,316 | 0 | 1 → 0 | 626,618 |
| 627,318 | 0 | 0 → 1 | 626,618 |
| 627,326 | 0 | 4 → 0 | 626,618 |
| 627,328 | 0 | 0 → 4 | 626,618 |
| 627,799 | 0 | 8 → 0 | 626,618 |
| 627,801 | 0 | 0 → 8 | 626,618 |
| 735,936 | 0 | 1 → 0 | 735,229 |
| 735,937 | 1 | 0 → 1 | 735,229 |
| 735,946 | 0 | 4 → 0 | 735,229 |
| 735,947 | 5 | 0 → 4 | 735,229 |
| 736,420 | 8 | 0 → 7 | 735,229 |
| 736,422 | 0 | 7 → 0 | 735,229 |
| 881,794 | 0 | 1 → 0 | 881,095 |
| 881,795 | 1 | 0 → 1 | 881,095 |
| 881,804 | 0 | 4 → 0 | 881,095 |
| 881,805 | 5 | 0 → 4 | 881,095 |
| 882,275 | 8 | 0 → 8 | 881,095 |
| 882,276 | 0 | 8 → 0 | 881,095 |
| 882,475 | 2 | 0 → 2 | 881,095 |
| 882,476 | 2 | 0 → 2 | 881,095 |
| 882,477 | 2 | 0 → 2 | 881,095 |
| 882,484 | 0 | 2 → 0 | 881,095 |
| 882,485 | 0 | 2 → 0 | 881,095 |
| 882,486 | 0 | 2 → 0 | 881,095 |
| 882,529 | 2 | 0 → 2 | 881,095 |
| 882,530 | 2 | 0 → 2 | 881,095 |
| 882,531 | 2 | 0 → 2 | 881,095 |
| 882,538 | 0 | 2 → 0 | 881,095 |
| 882,539 | 0 | 2 → 0 | 881,095 |
| 882,540 | 0 | 2 → 0 | 881,095 |
| 882,591 | 2 | 0 → 2 | 881,095 |
| 882,592 | 2 | 0 → 2 | 881,095 |
| 882,593 | 2 | 0 → 2 | 881,095 |
| 882,600 | 0 | 2 → 0 | 881,095 |
| 882,601 | 0 | 2 → 0 | 881,095 |
| 882,602 | 0 | 2 → 0 | 881,095 |
| 882,676 | 2 | 0 → 2 | 881,095 |
| 882,677 | 2 | 0 → 2 | 881,095 |
| 882,678 | 2 | 0 → 2 | 881,095 |
| 882,686 | 0 | 2 → 0 | 881,095 |
| 882,687 | 0 | 2 → 0 | 881,095 |
| 882,688 | 0 | 2 → 0 | 881,095 |
| 882,740 | 2 | 0 → 2 | 881,095 |
| 882,741 | 2 | 0 → 2 | 881,095 |
| 882,742 | 2 | 0 → 2 | 881,095 |
| 882,750 | 0 | 2 → 0 | 881,095 |
| 882,751 | 0 | 2 → 0 | 881,095 |
| 882,752 | 0 | 2 → 0 | 881,095 |
| 882,804 | 2 | 0 → 2 | 881,095 |
| 882,805 | 2 | 0 → 2 | 881,095 |
| 882,806 | 2 | 0 → 2 | 881,095 |
| 882,814 | 0 | 2 → 0 | 881,095 |
| 882,815 | 0 | 2 → 0 | 881,095 |
| 882,816 | 0 | 2 → 0 | 881,095 |
| 882,893 | 2 | 0 → 2 | 881,095 |
| 882,894 | 2 | 0 → 2 | 881,095 |
| 882,895 | 2 | 0 → 2 | 881,095 |
| 882,919 | 0 | 2 → 0 | 881,095 |
| 882,920 | 0 | 2 → 0 | 881,095 |
| 882,921 | 0 | 2 → 0 | 881,095 |
| 882,957 | 2 | 0 → 2 | 881,095 |
| 882,958 | 2 | 0 → 2 | 881,095 |
| 882,959 | 2 | 0 → 2 | 881,095 |
| 882,983 | 0 | 2 → 0 | 881,095 |
| 882,984 | 0 | 2 → 0 | 881,095 |
| 882,985 | 0 | 2 → 0 | 881,095 |
| 883,053 | 2 | 0 → 2 | 881,095 |
| 883,054 | 2 | 0 → 2 | 881,095 |
| 883,055 | 2 | 0 → 2 | 881,095 |
| 883,071 | 0 | 2 → 0 | 881,095 |
| 883,072 | 0 | 2 → 0 | 881,095 |
| 883,073 | 0 | 2 → 0 | 881,095 |
| 883,117 | 2 | 0 → 2 | 881,095 |
| 883,118 | 2 | 0 → 2 | 881,095 |
| 883,119 | 2 | 0 → 2 | 881,095 |
| 883,135 | 0 | 2 → 0 | 881,095 |
| 883,136 | 0 | 2 → 0 | 881,095 |
| 883,137 | 0 | 2 → 0 | 881,095 |
| 883,181 | 2 | 0 → 2 | 881,095 |
| 883,182 | 2 | 0 → 2 | 881,095 |
| 883,183 | 2 | 0 → 2 | 881,095 |
| 883,199 | 0 | 2 → 0 | 881,095 |
| 883,200 | 0 | 2 → 0 | 881,095 |
| 883,201 | 0 | 2 → 0 | 881,095 |
| 883,272 | 2 | 0 → 2 | 881,095 |
| 883,273 | 2 | 0 → 2 | 881,095 |
| 883,274 | 2 | 0 → 2 | 881,095 |
| 883,298 | 0 | 2 → 0 | 881,095 |
| 883,299 | 0 | 2 → 0 | 881,095 |
| 883,300 | 0 | 2 → 0 | 881,095 |
| 883,338 | 2 | 0 → 2 | 881,095 |
| 883,339 | 2 | 0 → 2 | 881,095 |
| 883,340 | 2 | 0 → 2 | 881,095 |
| 883,364 | 0 | 2 → 0 | 881,095 |
| 883,365 | 0 | 2 → 0 | 881,095 |
| 883,366 | 0 | 2 → 0 | 881,095 |
| 883,398 | 2 | 0 → 2 | 881,095 |
| 883,399 | 2 | 0 → 2 | 881,095 |
| 883,400 | 2 | 0 → 2 | 881,095 |
| 883,424 | 0 | 2 → 0 | 881,095 |
| 883,425 | 0 | 2 → 0 | 881,095 |
| 883,426 | 0 | 2 → 0 | 881,095 |
| 883,483 | 2 | 0 → 2 | 881,095 |
| 883,484 | 2 | 0 → 2 | 881,095 |
| 883,485 | 2 | 0 → 2 | 881,095 |
| 883,509 | 0 | 2 → 0 | 881,095 |
| 883,510 | 0 | 2 → 0 | 881,095 |
| 883,511 | 0 | 2 → 0 | 881,095 |
| 883,539 | 2 | 0 → 2 | 881,095 |
| 883,540 | 2 | 0 → 2 | 881,095 |
| 883,541 | 2 | 0 → 2 | 881,095 |
| 883,565 | 0 | 2 → 0 | 881,095 |
| 883,566 | 0 | 2 → 0 | 881,095 |
| 883,567 | 0 | 2 → 0 | 881,095 |
| 883,601 | 2 | 0 → 2 | 881,095 |
| 883,602 | 2 | 0 → 2 | 881,095 |
| 883,603 | 2 | 0 → 2 | 881,095 |
| 883,627 | 0 | 2 → 0 | 881,095 |
| 883,628 | 0 | 2 → 0 | 881,095 |
| 883,629 | 0 | 2 → 0 | 881,095 |
| 883,697 | 2 | 0 → 2 | 881,095 |
| 883,698 | 2 | 0 → 2 | 881,095 |
| 883,699 | 2 | 0 → 2 | 881,095 |
| 883,724 | 0 | 2 → 0 | 881,095 |
| 883,725 | 0 | 2 → 0 | 881,095 |
| 883,726 | 0 | 2 → 0 | 881,095 |
| 883,727 | 2 | 0 → 2 | 881,095 |
| 883,728 | 2 | 0 → 2 | 881,095 |
| 883,729 | 2 | 0 → 2 | 881,095 |
| 883,754 | 0 | 2 → 0 | 881,095 |
| 883,755 | 0 | 2 → 0 | 881,095 |
| 883,756 | 0 | 2 → 0 | 881,095 |
| 883,765 | 2 | 0 → 2 | 881,095 |
| 883,766 | 2 | 0 → 2 | 881,095 |
| 883,767 | 2 | 0 → 2 | 881,095 |
| 883,792 | 0 | 2 → 0 | 881,095 |
| 883,793 | 0 | 2 → 0 | 881,095 |
| 883,794 | 0 | 2 → 0 | 881,095 |
| 883,842 | 7 | 0 → 7 | 881,095 |
| 883,881 | 0 | 7 → 0 | 881,095 |
| 883,977 | 7 | 0 → 7 | 881,095 |
| 884,319 | 0 | 59 → 0 | 881,095 |
| 884,320 | 0 | 6 → 0 | 881,095 |
| 884,321 | 0 | 25 → 0 | 881,095 |
| 884,322 | 59 | 1 → 59 | 881,095 |
| 884,323 | 6 | 1 → 6 | 881,095 |
| 884,324 | 25 | 1 → 25 | 881,095 |
| 1,064,798 | 1 | 1 → 0 | 1,064,096 |
| 1,064,799 | 0 | 0 → 1 | 1,064,096 |
| 1,064,808 | 5 | 4 → 0 | 1,064,096 |
| 1,064,809 | 0 | 0 → 4 | 1,064,096 |
| 1,065,281 | 8 | 7 → 0 | 1,064,096 |
| 1,065,282 | 0 | 0 → 7 | 1,064,096 |
| 1,066,543 | 0 | 0 → 7 | 1,064,096 |
| 1,066,544 | 7 | 7 → 0 | 1,064,096 |
| 1,066,958 | 59 | 59 → 0 | 1,064,096 |
| 1,066,959 | 6 | 6 → 59 | 1,064,096 |
| 1,066,960 | 24 | 24 → 6 | 1,064,096 |
| 1,066,961 | 1 | 1 → 24 | 1,064,096 |
| 1,318,395 | 0 | 1 → 0 | 1,317,697 |
| 1,318,397 | 0 | 0 → 1 | 1,317,697 |
| 1,318,405 | 0 | 4 → 0 | 1,317,697 |
| 1,318,407 | 0 | 0 → 4 | 1,317,697 |
| 1,318,877 | 0 | 8 → 0 | 1,317,697 |
| 1,318,879 | 0 | 0 → 8 | 1,317,697 |

### Verification boundary and final state

The measured trial's production build is warning-free. Filtered SPC-driver,
audio, NMI stack/bus, bootstrap, pacing and host-port groups pass **232 tests**
across filters; the real `nmi_stack` group passes 7 (the inherited
`native_nmi_stack_bus` filter matches zero tests). The new before-refresh
source regression passes. Test builds retain the same 24 baseline dead-code
warning headers, with no new headers.

Exactly one full default library suite is run with the required assets and
packages: **Snes: 505 passed / 7 ignored; Zelda: 3,014 passed / 1 failed / 3 ignored**.
Its only failure is the permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
It tests the already compiled measured candidate; source restoration during
that run does not turn its result into a restored-runtime measurement.
The two separate exact-refresh source regressions fail on the measured
trial, overriding the route census results and rejecting admission.

The five runtime/fixture files are restored byte-for-byte to 3ec69b94.
The restored production build is warning-free; its binary SHA-256 is
`e68c718a5a5fdbbd65b793bbffae5a4357c0de9202652589da68c5608d36728e`,
byte-identical to the recorded restored main product. All **44** restored
SPC-driver tests pass. All **14 authorities and 140 reused artifacts** remain
hash-identical. Every launched process is joined; the target directory is
**5.87 GB (5.47 GiB)**, below 8 GB. Only this documentation section remains in the source diff.
There is no commit/push, GPU run, external Snes9x process, cached-A/V replay,
second live census or write to the ledger/oracle/helper authorities.
These are logical/CPU-picture and receiver-timing diagnostics, not cold A/V
acceptance. All retained trial measurements identify their own binary.

## Lagless prefix hunt 21

Corrected physical-field NMI transport on `codex/prefix-hunt21`, base
**541c21fd**. The hunt 20 `corrected-trial.patch` is integrated unchanged
in its four runtime files. Additional source-derived fixtures audit the
access boundary and every music/ambient path before any census launches.
**Kept and admitted in this worktree.** All requested gates pass, with
only the explicitly permitted default-suite spotlight failure. **No new main
instruction owners are priced; zero main instruction clocks are recovered.**

### Hardware law, executed work and the pre-census audit

Ordinary audio handlers begin at physical V225/H12 eligibility plus the
executed 62-clock hardware entry. Captured `$012C..$012F` and `$0133/$0131`
latches preserve the ROM command branches; acknowledgement reads observe
the executing receiver at the handler's bus times. Clocked upload returns
and existing native CPU bus accesses retain their separate owners.
Interrupted-instruction/DMA acceptance is still unresolved.

Pinned Snes9x revision `921f9f7b83660eb44ad263022a57a4a029057c37`
uses `while (CPU.Cycles >= CPU.NextEvent)` in `getset.h` memory-access
completion. `cpuexec.cpp` adds 40 master clocks at
`HC_WRAM_REFRESH_EVENT`. Consequently work ending exactly at refresh must
wait for that drain before the following bus access. The existing
`advance_snes_cpu_master_clock` work-endpoint helper and its fixture stay
unchanged. `nmi_audio_access_master_clock` adds the due refresh at equality;
both acknowledgement reads and every retained handler store use it.
The carried M1SNES/v2 H534/H538 phase includes field boundaries and the odd
short line. No sampled residual supplies a runtime price.

The new-music prefix is 444 executed clocks, the clear prefix 468; music
branch completion at `$8103` is 450/496/426/578/552 for no-clear, clear,
repeated, new-below-F2 and new-F2-plus respectively. Ambient reads add 72;
ambient stores add 150 for clear or 110 for direct. Effect-store suffixes
are 234/296 (clear), 204/266 (direct), or 188/250 (no ambient store).
Refresh is charged from each vector through its executed prefix, including
a refresh at the exact access boundary. These are ROM branch costs, not
route-dependent prices.

The pre-census audit is retained in `target/prefix-hunt21/pre-census-audit.json`.
All ten independent ROM positions are integrated fixtures. A further
135-case `RomCpuTimingProbe` fixture executes the real `$00:80C9` handler:
five music branches crossed with ambient skip/clear/direct and nine vector
positions, including both exact endpoints, already-passed refresh and a
next-line crossing. It checks reads, stores, values and each access's
refresh against executed source instruction work. All checks pass before
ROM, live or picture census launch. The full default library suite remains
the final test command after evaluation.

The field 391 new-music source witness is freshly reproduced: acceptance
140,037,020, vector 140,037,082, store **140,037,526**, queued SPC
**6,685,098**, zero CPU/quotient residual. Hunt 19's blanket subtraction
had removed 40 clocks before refresh. At the exact H538 endpoints, the
corrected read and store are **140,037,586**, rather than 140,037,546:
the missing 40-clock hardware drain is restored.

### Before, equal and after the refresh boundary

The executed-source timestamps below are also checked by the integrated
fixtures. Field 391/V225 refresh is H538; the equal endpoints drain to H578.

| Access | Vector H | Executed work | Source CPU = corrected CPU | CPU residual |
| --- | --- | --- | --- | --- |
| store | 74 | 444 | 140,037,526 | 0 |
| store | 90 | 444 | 140,037,542 | 0 |
| store | 94 | 444 | 140,037,586 | 0 |
| store | 98 | 444 | 140,037,590 | 0 |
| store | 600 | 444 | 140,038,052 | 0 |
| read | 74 | 390 | 140,037,472 | 0 |
| read | 144 | 390 | 140,037,542 | 0 |
| read | 148 | 390 | 140,037,586 | 0 |
| read | 152 | 390 | 140,037,590 | 0 |
| read | 600 | 390 | 140,037,998 | 0 |

### Fixture epochs and deadline carry

Only superseded output-window epochs change in existing fixtures. The
bootstrap effect is observed through source field 81 and the following
physical NMI, preserving **10 then 0** and the SPC's actual read of 10.
The upload-return ambient fixture observes the following acknowledgement
NMI's final store, preserving `[0,5,0,$42]` then zero on ports1..3.
The rebase fixture uses a coherent hardware field near its absolute APU
clock instead of pairing field 302 with an unrelated clock above 2^30.
The old callback-translation fixture is replaced with physical eligibility
plus entry/work assertions. No behavioral expected port value is relaxed.

NMI continuations and stores retain absolute u64 deadlines across windows.
Already-due stores retire in original deadline order before any unsigned
origin subtraction. A future deadline outside local u32 range remains
pending until representable after rebase; scheduled local events rebase
with the APU. Existing split-window, rebase, pre-origin ordering and far
future fixtures pass. The generic SPC host callback can enqueue a
CPU-selected store before the same SPC instruction completes.

### Residuals and the next first bus divergence

Fresh bus replay through iteration 48,558 exits zero with clean WRAM.
Its raw log and decompressed timing trace are byte-identical to the
hash-verified hunt20 trial, independently establishing the following
unchanged measurements on the corrected product. The source authorities
and reused baseline artifacts are hash-checked before reuse.

First-lag port3 publication moves **1,381,221 → 1,396,544**, recovering
**15,323 SPC clocks of publication phase**, separate from zero recovered
main instruction clocks. Planned CPU store 29,254,408 remains 40 clocks
before source 29,254,448 because actual interrupted-instruction acceptance
is not owned.

| Physical field | Candidate minus source, CPU master | Candidate queued / source synchronized SPC, port3 |
| --- | --- | --- |
| 80 | 0 | 1,379,486 / 1,379,486 |
| 81 | -40 | 1,396,544 / 1,396,549 |
| 82 | -18 | 1,413,606 / 1,413,608 |
| 83 | -46 | 1,430,666 / 1,430,669 |
| 84 | -4 | 1,447,725 / 1,447,729 |
| 85 | -42 | 1,464,785 / 1,464,787 |
| 86 | -4 | 1,481,845 / 1,481,847 |
| 87 | -42 | 1,498,905 / 1,498,910 |

At field 2236 the candidate ambient read is CPU 799,377,912, receiver
38,160,627, value5; source reads eight CPU clocks later, synchronized to
the same cycle and value. Candidate clear0 is CPU 799,377,990, queued
38,160,630; source is CPU 799,377,998, synchronized 38,160,631. The former
SPC 38,169,267 split remains crossed.

The contiguous comparison matches **90,095 bus events** before
**SPC 60,509,114**, `$F7` read: **source 0 / candidate 12**. Source field 3546
accepts at CPU 1,267,526,764/H28, sixteen clocks after eligibility.
Candidate port3 is CPU 1,267,527,596, queue 60,509,111; source is
CPU 1,267,527,612, quotient60,509,112, synchronized 60,509,114. Source
reads0 before publishing12 at that synchronized cycle. Remaining work is
actual CPU acceptance and CPU-to-SMP credit/bus ordering, not a fitted
runtime offset. Bus equality compares cycle/direction/address/value.

The regenerated residual CSV covers **9,530 source stores / 2,467 NMIs**,
with **zero ambiguous joins**. All **182,575** fresh candidate store
observations convert to their queue deadlines by the exact CPU/SPC
quotient. Twelve absent source stores and six clocked-return stores stay
explicit. Returns are joined to their fresh clocked vector, not a
same-field ordinary NMI. Physical-field joins do not establish logical
callback identity. Complete tables are retained under `target/prefix-hunt21/`.

### Upload rows and post-upload suffix NMIs

| Iteration | Ready polls source / main / corrected | Header polls source / corrected | Byte polls source / corrected | Holds main → corrected / source | Masked callbacks |
| --- | --- | --- | --- | --- | --- |
| 34,318 | 11,184 / 9,851 / 11,871 | 52 / 51 | 76,551 / 76,551 | 40 → 40 / 41 | 23 |
| 48,556 | 7,122 / 10,952 / 10,471 | 51 / 55 | 76,551 / 76,552 | 40 → 40 / 41 | 23 |

The first upload at 10,425 retains 6,259 ready polls against source 5,876,
58 holds and 22 masked callbacks. These are executed transfer counts;
callback counts diverge, so they are not matched CPU checkpoints.

| Iteration / lane | First SPC FF read | Ready AA store | Ready BB store |
| --- | --- | --- | --- |
| 34,318 / source | 642,527,398 | 642,544,073 | 642,544,080 |
| 34,318 / corrected | 642,529,970 | 642,543,308 | 642,543,315 |
| 48,556 / source | 925,122,447 | 925,139,379 | 925,139,386 |
| 48,556 / corrected | 925,087,620 | 925,102,516 | 925,102,523 |

Final clears are CPU 13,467,664,596 /19,386,634,459. The executed 174-clock
suffix restores `$4200` at 13,467,664,770 /19,386,634,633, with zero pending
holds. Source's additional suffix NMIs accept at 13,467,680,894,
V251/H822, `$02:83C6`, and 19,387,412,412, V225/H12, `$00:864F`.
Neither missing source suffix hold is recovered.

| Iteration / port | Source synchronized SPC | Corrected queued SPC | Difference |
| --- | --- | --- | --- |
| 34,318 / 1 | 642,918,897 | 642,918,130 | -767 |
| 34,318 / 2 | 642,918,905 | 642,918,134 | -771 |
| 34,318 / 3 | 642,918,905 | 642,918,137 | -768 |
| 48,556 / 1 | 925,514,479 | 925,477,349 | -37,130 |
| 48,556 / 2 | 925,514,483 | 925,477,354 | -37,129 |
| 48,556 / 3 | 925,514,489 | 925,477,359 | -37,130 |

All six values agree (ambient 5, then two zeros), while receiver epochs
remain different. The complete 42-NMI tables per upload retain opening,
40 pre-mask lag NMIs and source suffix, including every absent port.

### ROM, CPU pictures and instruction conservation

The supplied eight-slice ROM helper passes **1,473,123 iterations**,
**PASS / unexplained 0**. ROM holds change in **zero rows**; main and VWF
instruction budgets change in **zero rows**. The same 550 predictor rows
change only handler, elapsed, raster, refresh or HDMA diagnostics; their
complete field counts are in `instruction-conservation.json`. No pricing,
classifier, matcher or P2 implementation changes are made.

The five requested ROM prefixes remain unchanged. Live prefixes are also
shown to distinguish receiver-driven branch improvements from ROM pricing.

| Before iteration | ROM missing main → corrected | Live missing main → corrected |
| --- | --- | --- |
| 94,334 | 2 → 2 | 2 → 2 |
| 141,568 | 34 → 34 | 34 → 34 |
| 228,499 | 142 → 142 | 151 → 142 |
| 380,243 | 275 → 275 | 284 → 275 |
| 461,989 | 334 → 334 | 343 → 334 |

The eight-slice `--cpu-video` gate passes **1,473,123 iterations**,
**unexplained 0**, with all eight processes exiting zero. It preserves
**1,462,997 exact pictures**, 563 early, 3,952 late, 4 during-lag,
905 polygon-thread, 4,702 torn and 0 other content differences, matching
main in every category.

### One live census and complete poll attribution

Exactly **one** full, single-process seeded `--live-apu-reads
--pacing-predictor` census completes: **1,473,123 iterations**,
**PASS / unexplained 0**, exit zero. Commands, launch count, binary identity
and timing readback are retained under `target/prefix-hunt21/`.

| Lane | Missing main → corrected | Excess main → corrected | FP main → corrected |
| --- | --- | --- | --- |
| ROM | 1,864 → 1,864 | 0 → 0 | 0 → 0 |
| Live | 2,276 → 2,104 | 318 → 149 | 85 → 14 |

All **176 changed live hold rows** are live-only, because ROM holds change
in zero rows across builds. Every changed row is attributed to an observed
**`$08:C606`** song-end predicate change and its carried state; **zero
rows are unassigned**. The complete baseline and corrected timing traces
are read back. Every changed CSV hold value is checked against its actual
timing observation. Each changed episode has an observed first-zero
predicate change; its recovery boundary and following row agree in native
context, callbacks, held/masked counts, poll outcome and published port
values. Full boundary observations remain in `live-comparison.json`.

The **4,744** live main-budget differences are executed poll-path/carry
changes within those same verified intervals; zero are unanchored. Main
instruction prices stay unchanged. Runtime holds equal P2 in every row.
Current live-versus-ROM holds differ in **37** rows, all attributed.
The net live improvement is **172 missing / 169 excess / 71 FP**.

| Poll episode / source first zero | Main → corrected first zero | Changed hold rows | Delta missing / excess / FP | Recovery bound, exclusive |
| --- | --- | --- | --- | --- |
| 94,334 / 94,854 | 94,854 → 94,854 | 0 | 0 / 0 / 0 | 94,855 |
| 141,568 / 142,088 | 142,087 → 142,088 | 5 | -9 / -9 / -2 | 144,000 |
| 228,499 / 229,035 | 229,035 → 229,035 | 0 | 0 / 0 / 0 | 229,036 |
| 380,243 / 380,881 | 380,881 → 380,881 | 0 | 0 / 0 / 0 | 380,882 |
| 461,989 / 462,628 | 462,627 → 462,628 | 6 | -13 / -13 / -3 | 466,000 |
| 626,618 / 627,256 | 627,255 → 627,257 | 6 | 0 / 0 / 0 | 629,845 |
| 735,229 / 735,867 | 735,866 → 735,867 | 6 | -12 / -12 / -3 | 738,000 |
| 881,095 / 881,734 | 881,733 → 881,734 | 135 | -234 / -230 / -67 | 884,327 |
| 1,064,096 / 1,064,734 | 1,064,734 → 1,064,735 | 12 | 96 / 95 / 4 | 1,066,963 |
| 1,317,697 / 1,318,335 | 1,318,334 → 1,318,336 | 6 | 0 / 0 / 0 | 1,320,900 |

The 881,095 episode removes 234 missing, 230 excess and 67 FP, while
1,064,096 becomes one iteration late and adds 96 missing, 95 excess and
4 FP. The 626,618 and 1,317,697 episodes cross from early to late while
conserving totals. These regressions remain explicit.

Observed native poll rows are **6,048 → 6,057**. The six actual corrected
reseeds are **628,000 / 629,845 / 1,066,000 / 1,066,963 / 1,320,000 /
1,320,900**. Baseline CSV raw SHA-256 is
`f11bc4b607fc228fb6e2a181b332cf4932772889c742be8d2985ed6ce9e7c6f4`;
baseline timing raw SHA-256 is
`286873e50117590b175154c1bb1d14265832c78b9b7cc3259ee1e5f6130217af`;
corrected timing raw SHA-256 is
`184adedcf95d5176b3dff5350ae1774f821ed6407d1d1881d167ca803ac480fb`.
The latter is byte-identical to the hash-verified hunt20 measured trace,
now produced by the fully integrated corrected product. Source first-zero
values are independently reproduced from retained ledger reads in the
fresh trace for all ten episodes.

The following is the complete changed-live-hold set. Every episode denotes
`$08:C606`; `changed-live-holds.csv/json` retain every row and both timing
observations. No live/ROM hold-equality assumption is used for attribution.

| Iteration | Source lag | Main → corrected holds | Poll episode |
| --- | --- | --- | --- |
| 142,088 | 0 | 5 → 0 | 141,568 |
| 142,089 | 5 | 0 → 5 | 141,568 |
| 142,091 | 0 | 4 → 0 | 141,568 |
| 142,092 | 4 | 3 → 4 | 141,568 |
| 142,094 | 3 | 0 → 3 | 141,568 |
| 462,699 | 0 | 1 → 0 | 461,989 |
| 462,700 | 1 | 0 → 1 | 461,989 |
| 462,709 | 0 | 4 → 0 | 461,989 |
| 462,710 | 5 | 0 → 4 | 461,989 |
| 463,179 | 0 | 8 → 0 | 461,989 |
| 463,180 | 8 | 0 → 8 | 461,989 |
| 627,316 | 0 | 1 → 0 | 626,618 |
| 627,318 | 0 | 0 → 1 | 626,618 |
| 627,326 | 0 | 4 → 0 | 626,618 |
| 627,328 | 0 | 0 → 4 | 626,618 |
| 627,799 | 0 | 8 → 0 | 626,618 |
| 627,801 | 0 | 0 → 8 | 626,618 |
| 735,936 | 0 | 1 → 0 | 735,229 |
| 735,937 | 1 | 0 → 1 | 735,229 |
| 735,946 | 0 | 4 → 0 | 735,229 |
| 735,947 | 5 | 0 → 4 | 735,229 |
| 736,420 | 8 | 0 → 7 | 735,229 |
| 736,422 | 0 | 7 → 0 | 735,229 |
| 881,794 | 0 | 1 → 0 | 881,095 |
| 881,795 | 1 | 0 → 1 | 881,095 |
| 881,804 | 0 | 4 → 0 | 881,095 |
| 881,805 | 5 | 0 → 4 | 881,095 |
| 882,275 | 8 | 0 → 8 | 881,095 |
| 882,276 | 0 | 8 → 0 | 881,095 |
| 882,475 | 2 | 0 → 2 | 881,095 |
| 882,476 | 2 | 0 → 2 | 881,095 |
| 882,477 | 2 | 0 → 2 | 881,095 |
| 882,484 | 0 | 2 → 0 | 881,095 |
| 882,485 | 0 | 2 → 0 | 881,095 |
| 882,486 | 0 | 2 → 0 | 881,095 |
| 882,529 | 2 | 0 → 2 | 881,095 |
| 882,530 | 2 | 0 → 2 | 881,095 |
| 882,531 | 2 | 0 → 2 | 881,095 |
| 882,538 | 0 | 2 → 0 | 881,095 |
| 882,539 | 0 | 2 → 0 | 881,095 |
| 882,540 | 0 | 2 → 0 | 881,095 |
| 882,591 | 2 | 0 → 2 | 881,095 |
| 882,592 | 2 | 0 → 2 | 881,095 |
| 882,593 | 2 | 0 → 2 | 881,095 |
| 882,600 | 0 | 2 → 0 | 881,095 |
| 882,601 | 0 | 2 → 0 | 881,095 |
| 882,602 | 0 | 2 → 0 | 881,095 |
| 882,676 | 2 | 0 → 2 | 881,095 |
| 882,677 | 2 | 0 → 2 | 881,095 |
| 882,678 | 2 | 0 → 2 | 881,095 |
| 882,686 | 0 | 2 → 0 | 881,095 |
| 882,687 | 0 | 2 → 0 | 881,095 |
| 882,688 | 0 | 2 → 0 | 881,095 |
| 882,740 | 2 | 0 → 2 | 881,095 |
| 882,741 | 2 | 0 → 2 | 881,095 |
| 882,742 | 2 | 0 → 2 | 881,095 |
| 882,750 | 0 | 2 → 0 | 881,095 |
| 882,751 | 0 | 2 → 0 | 881,095 |
| 882,752 | 0 | 2 → 0 | 881,095 |
| 882,804 | 2 | 0 → 2 | 881,095 |
| 882,805 | 2 | 0 → 2 | 881,095 |
| 882,806 | 2 | 0 → 2 | 881,095 |
| 882,814 | 0 | 2 → 0 | 881,095 |
| 882,815 | 0 | 2 → 0 | 881,095 |
| 882,816 | 0 | 2 → 0 | 881,095 |
| 882,893 | 2 | 0 → 2 | 881,095 |
| 882,894 | 2 | 0 → 2 | 881,095 |
| 882,895 | 2 | 0 → 2 | 881,095 |
| 882,919 | 0 | 2 → 0 | 881,095 |
| 882,920 | 0 | 2 → 0 | 881,095 |
| 882,921 | 0 | 2 → 0 | 881,095 |
| 882,957 | 2 | 0 → 2 | 881,095 |
| 882,958 | 2 | 0 → 2 | 881,095 |
| 882,959 | 2 | 0 → 2 | 881,095 |
| 882,983 | 0 | 2 → 0 | 881,095 |
| 882,984 | 0 | 2 → 0 | 881,095 |
| 882,985 | 0 | 2 → 0 | 881,095 |
| 883,053 | 2 | 0 → 2 | 881,095 |
| 883,054 | 2 | 0 → 2 | 881,095 |
| 883,055 | 2 | 0 → 2 | 881,095 |
| 883,071 | 0 | 2 → 0 | 881,095 |
| 883,072 | 0 | 2 → 0 | 881,095 |
| 883,073 | 0 | 2 → 0 | 881,095 |
| 883,117 | 2 | 0 → 2 | 881,095 |
| 883,118 | 2 | 0 → 2 | 881,095 |
| 883,119 | 2 | 0 → 2 | 881,095 |
| 883,135 | 0 | 2 → 0 | 881,095 |
| 883,136 | 0 | 2 → 0 | 881,095 |
| 883,137 | 0 | 2 → 0 | 881,095 |
| 883,181 | 2 | 0 → 2 | 881,095 |
| 883,182 | 2 | 0 → 2 | 881,095 |
| 883,183 | 2 | 0 → 2 | 881,095 |
| 883,199 | 0 | 2 → 0 | 881,095 |
| 883,200 | 0 | 2 → 0 | 881,095 |
| 883,201 | 0 | 2 → 0 | 881,095 |
| 883,272 | 2 | 0 → 2 | 881,095 |
| 883,273 | 2 | 0 → 2 | 881,095 |
| 883,274 | 2 | 0 → 2 | 881,095 |
| 883,298 | 0 | 2 → 0 | 881,095 |
| 883,299 | 0 | 2 → 0 | 881,095 |
| 883,300 | 0 | 2 → 0 | 881,095 |
| 883,338 | 2 | 0 → 2 | 881,095 |
| 883,339 | 2 | 0 → 2 | 881,095 |
| 883,340 | 2 | 0 → 2 | 881,095 |
| 883,364 | 0 | 2 → 0 | 881,095 |
| 883,365 | 0 | 2 → 0 | 881,095 |
| 883,366 | 0 | 2 → 0 | 881,095 |
| 883,398 | 2 | 0 → 2 | 881,095 |
| 883,399 | 2 | 0 → 2 | 881,095 |
| 883,400 | 2 | 0 → 2 | 881,095 |
| 883,424 | 0 | 2 → 0 | 881,095 |
| 883,425 | 0 | 2 → 0 | 881,095 |
| 883,426 | 0 | 2 → 0 | 881,095 |
| 883,483 | 2 | 0 → 2 | 881,095 |
| 883,484 | 2 | 0 → 2 | 881,095 |
| 883,485 | 2 | 0 → 2 | 881,095 |
| 883,509 | 0 | 2 → 0 | 881,095 |
| 883,510 | 0 | 2 → 0 | 881,095 |
| 883,511 | 0 | 2 → 0 | 881,095 |
| 883,539 | 2 | 0 → 2 | 881,095 |
| 883,540 | 2 | 0 → 2 | 881,095 |
| 883,541 | 2 | 0 → 2 | 881,095 |
| 883,565 | 0 | 2 → 0 | 881,095 |
| 883,566 | 0 | 2 → 0 | 881,095 |
| 883,567 | 0 | 2 → 0 | 881,095 |
| 883,601 | 2 | 0 → 2 | 881,095 |
| 883,602 | 2 | 0 → 2 | 881,095 |
| 883,603 | 2 | 0 → 2 | 881,095 |
| 883,627 | 0 | 2 → 0 | 881,095 |
| 883,628 | 0 | 2 → 0 | 881,095 |
| 883,629 | 0 | 2 → 0 | 881,095 |
| 883,697 | 2 | 0 → 2 | 881,095 |
| 883,698 | 2 | 0 → 2 | 881,095 |
| 883,699 | 2 | 0 → 2 | 881,095 |
| 883,724 | 0 | 2 → 0 | 881,095 |
| 883,725 | 0 | 2 → 0 | 881,095 |
| 883,726 | 0 | 2 → 0 | 881,095 |
| 883,727 | 2 | 0 → 2 | 881,095 |
| 883,728 | 2 | 0 → 2 | 881,095 |
| 883,729 | 2 | 0 → 2 | 881,095 |
| 883,754 | 0 | 2 → 0 | 881,095 |
| 883,755 | 0 | 2 → 0 | 881,095 |
| 883,756 | 0 | 2 → 0 | 881,095 |
| 883,765 | 2 | 0 → 2 | 881,095 |
| 883,766 | 2 | 0 → 2 | 881,095 |
| 883,767 | 2 | 0 → 2 | 881,095 |
| 883,792 | 0 | 2 → 0 | 881,095 |
| 883,793 | 0 | 2 → 0 | 881,095 |
| 883,794 | 0 | 2 → 0 | 881,095 |
| 883,842 | 7 | 0 → 7 | 881,095 |
| 883,881 | 0 | 7 → 0 | 881,095 |
| 883,977 | 7 | 0 → 7 | 881,095 |
| 884,319 | 0 | 59 → 0 | 881,095 |
| 884,320 | 0 | 6 → 0 | 881,095 |
| 884,321 | 0 | 25 → 0 | 881,095 |
| 884,322 | 59 | 1 → 59 | 881,095 |
| 884,323 | 6 | 1 → 6 | 881,095 |
| 884,324 | 25 | 1 → 25 | 881,095 |
| 1,064,798 | 1 | 1 → 0 | 1,064,096 |
| 1,064,799 | 0 | 0 → 1 | 1,064,096 |
| 1,064,808 | 5 | 4 → 0 | 1,064,096 |
| 1,064,809 | 0 | 0 → 4 | 1,064,096 |
| 1,065,281 | 8 | 7 → 0 | 1,064,096 |
| 1,065,282 | 0 | 0 → 7 | 1,064,096 |
| 1,066,543 | 0 | 0 → 7 | 1,064,096 |
| 1,066,544 | 7 | 7 → 0 | 1,064,096 |
| 1,066,958 | 59 | 59 → 0 | 1,064,096 |
| 1,066,959 | 6 | 6 → 59 | 1,064,096 |
| 1,066,960 | 24 | 24 → 6 | 1,064,096 |
| 1,066,961 | 1 | 1 → 24 | 1,064,096 |
| 1,318,395 | 0 | 1 → 0 | 1,317,697 |
| 1,318,397 | 0 | 0 → 1 | 1,317,697 |
| 1,318,405 | 0 | 4 → 0 | 1,317,697 |
| 1,318,407 | 0 | 0 → 4 | 1,317,697 |
| 1,318,877 | 0 | 8 → 0 | 1,317,697 |
| 1,318,879 | 0 | 0 → 8 | 1,317,697 |

### Verification boundary and final state

The production build is warning-free. Its SHA-256 is
`3ba1c05717cf2573fa7d55ce617c90682a64729ec384cc44f184f22bbafcb826`;
the measured binary is retained as `target/prefix-hunt21/zelda3-candidate`.
The four runtime files match the corrected hunt20 retained sources
byte-for-byte; added test coverage is in `upload_return_tests.rs`.

Filtered groups pass: **52 SPC-driver**, the separate **135-case handler
audit**, **195 audio**, **7 NMI-stack**, **28 native-pacing**, **2 bootstrap**,
**6 host-port** and **1 scheduled-input-port** tests. The broader NMI filter
passes **17 Snes / 258 Zelda tests**, with its existing ignored cases.
Groups overlap. Test builds retain the same **24 baseline dead-code warning
headers**, with no new headers. `filtered-results.json` and
`warning-comparison.json` retain the exact results.

Exactly **one** full default library suite runs, as the final test command,
with the required assets, packages, profile and `--lib` selection:
**Snes 505 passed / 7 ignored; Zelda 3,018 passed / 1 failed / 3 ignored**.
Its sole failure is the permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
The command exits 101 for that failure; it is not described as an all-green
suite. `full-library-result.json` records the exact command and confirms
that every audited runtime/fixture source hash stayed unchanged.

The access, refresh, deadline-carry and fixture audit completes **before any
census launch**. No runtime or fixture source changes follow the census.
The report-only live/ROM-equality guard is corrected and the complete CSV
comparison recomputed; this changes no runtime and launches no additional
live replay. All 176 changed live holds remain attributed and every ROM,
picture, instruction-conservation and library admission requirement holds.

The corrected handler remains in the source diff on `codex/prefix-hunt21`;
there is no restoration, commit or push. All launched processes are joined.
Ledger, oracle, source ROM/assets and helper authorities remain
hash-identical after evaluation. Target usage is **6.89 GB
(6.42 GiB)**, below 8 GB; only completed build incremental caches
are removed, within this worktree's own target directory. Prior evidence is
preserved read-only. Commands, binary/source identities, raw timings,
complete residuals, attribution and gate results are retained under
`target/prefix-hunt21/`.

These are complete seeded logical/CPU-picture gates and receiver-timing
measurements. No GPU run, external Snes9x process, cached-A/V replay,
second live census or write to the ledger/oracle/helper authorities occurs.
Cold exact A/V acceptance remains a separate verification boundary.

## Lagless prefix hunt 22

Investigated NMI acceptance on `codex/prefix-hunt22`, base **a8aea661**.
**Diagnosis only: no runtime acceptance change is admitted.** There is no
runtime patch to revert. The ordinary handler, P2, classifier, matcher,
instruction prices and hold decisions remain unchanged. **Zero new main
owners are priced; zero main instruction clocks are recovered.** Actual
acceptance for the ordinary wait loop is still unresolved in the lagless
runtime; this section does not claim completion of that ownership work.

### Eligibility owners and residual distributions

The hash-verified hunt 21 table covers **2,467 NMIs / 9,530 source stores**.
`target/prefix-hunt22/nmi-classes.csv` assigns every row. The distinction is
the work executing on the CPU, rather than whether a module has an existing
instruction-cost model. Ordinary lagless main runs those models atomically;
their existence does not establish a retained physical CPU phase. The
bootstrap NMI belongs to the wait-loop class by its interrupted work and
already has an executing CPU owner. No `$08:C606` song-end poll occurs in
this residual table, whose last sampled iteration is 48,556.

| Eligibility class | NMIs | Source stores / compared / absent | CPU store residual before → after |
| --- | ---: | ---: | --- |
| (i) Main completed, `$00:8034/$00:8036` wait | 1,938 | 7,656 / 7,656 / 0 | [-22, 0] → [-22, 0] |
| (ii) Retained polygon worker and upload suffix | 79 | 314 / 314 / 0 | [-777,771, 0] → same |
| (iii) Atomic lagless main | 450 | 1,560 / 1,548 / 12 | [-2,540, 0] → [-2,540, 0] |

All values are candidate minus source, in CPU master clocks. Complete
before/after store and acceptance histograms are in `residual-classes.json`.
The wait-loop per-NMI store residual histogram is below; all ports agree.
The bootstrap NMI already owns actual acceptance and has zero store residual,
although its physical eligibility precedes acceptance by fourteen clocks.

| CPU residual | 0 | -2 | -4 | -6 | -8 | -10 | -12 | -14 | -16 | -18 | -20 | -22 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| NMIs | 173 | 173 | 184 | 171 | 167 | 161 | 179 | 150 | 159 | 161 | 178 | 82 |

Class (ii) contains **77 polygon-worker NMIs**, residual **[-46, 0]**,
and the two upload suffix NMIs below. The fresh instruction-edge trace
distinguishes the worker from atomic main even when main has already
completed. All 2,467 residual acceptances join the freshly executed
source instruction trace, with zero missing or conflicting joins.

Class (iii)'s ordinary instruction-boundary residual is **[-54, 0]** for
448 NMIs. Two NMIs during iteration 1,915, accepted at `$00:8CDE`, retain
DMA-delayed residuals **-2,540 / -1,808**. Thus -46 is not a valid bound
for the complete atomic-main class. The suffix class also needs its actual
enable/arrival edge; subtracting the same field's V225 eligibility gives
-36,270 / 0, which must not be called ordinary instruction jitter.
For the 448 ordinary atomic-main NMIs the store bound is **[-88, 0]**:
one music access crosses a different refresh boundary from its source.

### Executed wait law and the missing native phase

`MainWait::step_clocked` already executes the real slow-ROM `LDA $12`
(24 clocks) and taken `BEQ` (22 clocks), including their bus boundaries.
The new `nmi_acceptance_follows_the_executed_wait_phase` fixture crosses
all 23 even phases of this 46-clock period at two starting scanlines in
each of two physical fields. It compares native instructions, registers,
WRAM, bus timestamps and refresh against `RomCpuTimingProbe`, then checks
the actual first eligible instruction boundary and interrupted PC. The
same hardware eligibility has multiple valid acceptance outcomes depending
on the retained entry phase; eligibility alone cannot select one.

The ledger example's timing-audit mode now writes
`nmi-instruction-edges.csv`: the retired instruction, instruction start,
acceptance, next PC, executed work, last wait entry and last `$4200` store.
These are diagnostic observations only. No observation is imported into
the runtime or the read-only ledger authority.

The next bus split's source phase is now explicit. Field 3,546 entered
wait at CPU **1,267,303,034**. Its final `BEQ` starts at **1,267,526,742**,
crosses eligibility **1,267,526,748**, and finishes at **1,267,526,764**,
interrupting `$00:8034`: exactly **16** clocks of acceptance remainder.
That is executed source provenance, not permission to inject those clocks.

The lagless runtime discards `native_startup_cpu` after bootstrap, and
foreign-memory seeds contain no CPU execution phase. Its cold-intro
polygon worker retains instructions, but `lag_free_intro_poly_work`
recreates a zero-origin bus cursor for each scheduled work slice;
instruction counts select that legacy slice. This is not an absolute
acceptance phase. Reusing those sampled counts as timing prices is excluded.
Subsequent
`native_finish_main_cost` retains aggregate P2 work/elapsed estimates,
not a completed instruction timeline. For the preceding iteration 3,080,
source opening-to-wait time is **133,640**, while native P2 reports
**133,724**. Across the first 2,000 sampled iterations every P2 endpoint
differs from the source endpoint, with differences **[-121,758, 16,322]**.
These differences are diagnostic residuals, never runtime prices. Using
that endpoint as an actual wait-loop phase would introduce an unproved
phase reset. No such trial is admitted, and no ledger acceptance timestamp
is read by a new runtime owner.

### Upload arrival, receiver work and suffix ordering

The fresh candidate bus/timing replay is byte-identical to hunt 21.
Hash-verified source APUI transactions retain the complete command, final
clear and enable edges. `upload-mechanism.json` decomposes the measured
epochs using executed transfer work; none of these residuals is charged.

| Iteration | Command CPU residual | Transfer wall difference | Final clear / enable CPU residual | Source enable → acceptance |
| --- | ---: | ---: | ---: | ---: |
| 34,318 | -57,092 | +41,016 | -16,076 | 48 |
| 48,556 | -972,361 | +200,404 | -771,957 | 5,822 |

Both rows load bank 0. `native_defer_song_upload` captures a P2 phase but
deliberately marks bank 0 unpositioned; `start_deferred_song_bank_transfer`
therefore passes no phase. The command is queued at the audio window's end,
SPC **642,509,440 / 925,072,640**, rather than at an executing caller's
CPU store. The exact ceiling conversion gives CPU **13,459,104,316 /
19,378,157,559**. This legacy transfer also uses a **386-clock** first-read
prefix; the positioned overworld caller and source use **364**. The missing
command epoch and legacy caller path are upstream of acceptance jitter.

At 34,318, source clears the last port at **13,467,680,672**, restores
`$4200` at **13,467,680,846**, V251/H774, and accepts after the remaining
store bus cycle and `RTS` at **13,467,680,894**, `$02:83C6`.
Candidate restore is **13,467,664,770**, V239/H1062. Both are in VBlank;
the 48-clock store-completion/return interval is correct in this case.
The **16,076-clock** upstream completion error already exists before
acceptance. Ready polls remain **11,184 source / 11,871 candidate**,
header polls **52 / 51**, byte polls **76,551 / 76,551**.

At 48,556, source clears at **19,387,406,416**, restores at
**19,387,406,590**, V220/H1010, and continues the main suffix until the
next V225/H12 acceptance at **19,387,412,412**, `$00:864F`.
Candidate restore is **19,386,634,633**, field 54,248 V178/H1073;
source restore is in field 54,250. The current queue assumes a return NMI
after 48 clocks even outside VBlank. A visible-line enable must instead
wait for the hardware arrival and execute the intervening caller suffix.
Ready polls remain **7,122 / 10,471**, header polls **51 / 55** and byte
polls **76,551 / 76,552**. The command already arrives **972,361 CPU clocks
early**, so correcting only enable-edge acceptance cannot reconcile the
upload epochs.

The source's two suffix NMIs occur before main clears `$12`; both are
additional lag NMIs. Native atomic main has already completed that suffix
when its deferred transfer returns, restores its completed latch and
uses the return callback as a closing NMI with pending holds zero. This
explains why an extra source suffix hold is absent. Recovering it requires
the real upload caller/suffix continuation and its hardware arrival; an
unconditional extra hold would not establish that ownership.

| Iteration / port | Source synchronized SPC | Candidate queued SPC | Residual |
| --- | ---: | ---: | ---: |
| 34,318 / 1 | 642,918,897 | 642,918,130 | -767 |
| 34,318 / 2 | 642,918,905 | 642,918,134 | -771 |
| 34,318 / 3 | 642,918,905 | 642,918,137 | -768 |
| 48,556 / 1 | 925,514,479 | 925,477,349 | -37,130 |
| 48,556 / 2 | 925,514,483 | 925,477,354 | -37,129 |
| 48,556 / 3 | 925,514,489 | 925,477,359 | -37,130 |

Both rows retain **40 native holds / 41 source lag NMIs**, with 23 masked
native callbacks each. The six values agree: ambient 5 and two zero effect
ports. CPU/SPC quotient deadlines and source synchronized receiver cycles
remain separate observations.

The contiguous source/candidate bus comparison still agrees for **90,095
events**, then differs at **SPC 60,509,114**, `$F7`, source **0** /
candidate **12**. The source wait phase above establishes the interrupted
instruction, but the native phase is not owned and this frontier is not
claimed crossed. Its CPU store is 1,267,527,596 versus source
1,267,527,612. Adding the executed 16-clock remainder would move the
quotient deadline from SPC **60,509,111 to 60,509,112**, still before
source synchronization at **60,509,114**. Thus acceptance ownership alone
does not prove this bus split crossed; the CPU-to-SMP credit and bus order
also need their executed owner. No two-cycle shift is applied.

### Verification boundary

The pre-census audit admits only the source observation tool; the additional
wait fixture is test-only. There are no production runtime or instruction
budget changes, fixture epoch updates, new environment switches, classifier,
matcher or P2 changes.

The eight-slice ROM helper passes **1,473,123 iterations**, **PASS /
unexplained 0**, at **1,864 missing / 0 excess / 0 FP**. Every predictor row
is byte-identical to hunt 21: zero changed main/VWF instruction budgets,
zero changed ROM hold rows and zero changed P2 diagnostics. The five
prefixes are confirmed in both completed ROM and live-prefix readback:

| Before iteration | ROM missing before → after | Live missing before → after |
| --- | --- | --- |
| 94,334 | 2 → 2 | 2 → 2 |
| 141,568 | 34 → 34 | 34 → 34 |
| 228,499 | 142 → 142 | 142 → 142 |
| 380,243 | 275 → 275 | 275 → 275 |
| 461,989 | 334 → 334 | 334 → 334 |

The eight CPU-picture processes all exit zero: **1,473,123 iterations**,
**PASS / unexplained 0**, **1,462,997 exact pictures**, 563 early, 3,952
late, 4 during-lag, 905 polygon-thread, 4,702 torn and 0 other content
differences. Every category matches hunt 21.

Filtered tests pass: **6 main-wait tests**, including the 92-case acceptance
fixture, and **6 ledger-example tests**. Production and source-example
builds are warning-free; test warnings match the pre-existing baseline.
No existing fixture expectation or epoch is changed. Slow live-census and
final default-suite results are recorded under `target/prefix-hunt22/`.

Exactly **one** single-process seeded `--live-apu-reads
--pacing-predictor` census completes: **1,473,123 iterations**, **PASS /
unexplained 0**, exit zero. The entire live CSV and decompressed timing
trace are byte-identical to hunt 21. Both lanes retain every instruction
budget and hold row; there are **zero changed live hold rows**, therefore
zero live-only changed rows and zero unattributed changes.

| Lane | Missing before → after | Excess before → after | FP before → after |
| --- | --- | --- | --- |
| ROM | 1,864 → 1,864 | 0 → 0 | 0 → 0 |
| Live | 2,104 → 2,104 | 149 → 149 | 14 → 14 |

The remaining live-versus-ROM differential is **37 hold rows**, all
attributed to `$08:C606` poll episodes and their carried state. Fresh trace
readback reproduces all **6,057 poll rows** and six actual reseeds.
`current-live-rom-holds.csv/json` retains every differential row;
`live-rom-row-observations.json` retains their fresh timing observations.
The following deltas are live minus ROM, not changes made by this hunt.

| Poll episode | Differential hold rows | Delta missing / excess / FP |
| --- | ---: | --- |
| 626,618 | 10 | +27 / +27 / +5 |
| 1,064,096 | 12 | +96 / +95 / +4 |
| 1,317,697 | 15 | +117 / +27 / +5 |
| Total | 37 | +240 / +149 / +14 |

The final default library suite is launched **once**, after all source and
fixture edits: **505 SNES tests pass / 7 ignored**, **3,019 Zelda tests
pass / 3 ignored**, with exactly the permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`
failure. There are no other failures and no new test warnings. Source
hashes remain unchanged across that run. `git diff --check` passes.

All source, census and test processes are joined. The ledger, oracle,
ROM/assets and gate helpers remain hash-identical; all 399 checked
read-only retained artifacts/inputs are unchanged. Only this report is
updated among the files captured in the previous artifact registry. No
commit, push, GPU, external Snes9x process, cached-A/V replay, second live
census, classifier/matcher/P2 change or authority write occurs. Only
rebuildable incremental output and redundant new receiver snapshots were
removed within this worktree's own `target/`; snapshot hashes are retained.
Final target usage is **7.49 GB**, below the 8 GB limit.

These are completed seeded logical, receiver and CPU-picture measurements.
Actual runtime wait/polygon/suffix acceptance ownership is **not completed**;
the eligibility anchor remains. Cold exact A/V acceptance is a separate
verification boundary.

## Lagless prefix hunt 23

Worktree `/Users/missingno/Documents/zelda3-rs-codex`, branch
`codex/prefix-hunt23`, base **ffb56bb3**. Returned to main instruction
pricing. **The candidate is not admitted:** nine live-only changed hold rows
belong to the pendant poll at `$08:C3FD`, outside the required `$08:C606`
boundary. The candidate's nine owner groups add **60,080 instruction clocks on the 34-row
cohort** and recover **20 of its missing holds**. Across the full ROM route,
**21 holds recover**, with none lost. Source prices come from executed ROM
blocks, branch outcomes and indexed-address crossings. No measured deficit,
sampled poll count or scene selector becomes a runtime price.

### Fresh classification and ownership

All 20 selected retained artifacts match their recorded hashes. The fresh
141,568-iteration baseline agrees with hash-verified hunt 22 on every
predictor field. Hunt 10/11 classifications are not transplanted into the
new window. A fresh unmodified-ROM CPU run covers **154,300 host calls /
141,600 complete iterations**, collecting routine, sprite, per-PC,
structural, NMI and APUI receipts for all 34 missing rows. The ledger,
oracle and helpers remain read-only.

| Classification | Before | Remaining after |
| --- | ---: | ---: |
| (a) Nonpoll instruction shortfall | 26 | 7 |
| (b) Independently established inherited arrival | 0 | 0 |
| (c) Upload polling/return phase | 5 | 5 |
| (d) Mixed instruction / spotlight poll phase | 3 | 2 |
| Recovered holds | 0 | 20 |

`classification.csv/json` retains every cohort row, its before/after hold
counts, diagnostic clocks short, signed instruction gaps and specific
remaining owners. `raw-owner-residuals.csv` retains all signed routine
residuals, including negative aliases. Sprite call-frame partitions and
separate protocol ownership prevent overlapping raw parent/child gaps from
being counted as missing instructions. The frozen P2 inverse is a diagnostic
boundary, not an instruction price or a promise about a changed raster wait.

The reconciled ranking starts with vulture, statue and Leever groups, each
closing four rows independently, then pit callers/wrappers (three), prayer
dispatch/case callers (two), and cannon entry (one). These closure counts
overlap. Bit scan, plaque and Debirando entries complete combinations of
owners rather than independently crossing an original boundary.
`owner-ranking.json` also retains every measured sprite-family deficit and
the deferred overlay/item-tile work; raw routine rankings remain separately
available for auditing aliases.

### ROM-exact prices and fixtures

| Owner group | Cohort clocks added | Unmodified-ROM fixture |
| --- | ---: | --- |
| Vulture wrappers, draw and inactive prefix | 13,524 | `prefix23_vulture_wrappers_draw_and_inactive_prefix_match_rom` |
| Desert statue caller and bank-5 direction wrappers | 12,012 | `prefix23_desert_statue_caller_all_paths_match_rom`; shared wrappers in dungeon-entry fixture |
| Leever entry through inactive child | 7,824 | `prefix23_dungeon_sprite_entries_cover_all_prefix_paths_in_rom` |
| Debirando pit entry and direction wrappers | 4,240 | same dungeon-entry fixture |
| Prayer dispatcher and case callers | 10,836 | `prefix23_prayer_dispatch_and_case_callers_match_all_rom_targets` |
| Wall-cannon entry through inactive child | 2,352 | dungeon-entry fixture |
| Most-significant-bit scan | 6,860 | `prefix23_bit_scan_matches_every_byte_in_unmodified_rom` |
| Plaque two-call entry | 1,512 | `prefix23_plaque_two_call_prefix_matches_active_and_unwinding_paths` |
| Debirando two-call entry | 920 | dungeon-entry fixture |
| **Total** | **60,080** | **8,284 ROM owner cases in six tests** |

The statue fixture covers both outcomes of all ten caller conditionals,
including damage carry preserved without applying damage, both floors,
solid/passable attribute pages, direction-table crossings and terminal
movement counters. Its 4,608 cases exclude draw, collision, movement and
damage children. The 3,072 dungeon owner cases cover all four prefix branch
pairs, both ends of the sprite-slot range, active/unwinding returns and all
four directions. The vulture fixture checks five disjoint owners in 320
cases. Prayer covers all five actual dispatch targets, two iris radii and
both dispatcher/caller ownerships; bit scan exhausts all 256 byte inputs.

The existing bank-5 near wrapper and its real long trampoline are now
explicitly selected by both statue draw and pit entry. Their direction body
keeps its existing child charge. Inactive stack-unwind work is not charged
again. Vulture's active AI, plaque's interaction suffix and the dungeon
entries' AI/movement/spawn suffixes remain unpriced. These are explicit
prefix boundaries, not complete-sprite coverage claims.

All **5,247 old generated blocks are preserved**; 62 are added. Expanded
before/after markers recover exactly **60,080 clocks on 1,108 per-PC rows**.
Every added count is nonnegative and every affected PC agrees with its full
source count and clocks: zero excess/partial rows. Main-budget, routine-total
and block-marker deltas agree on every cohort row.
`per-pc-conservation.csv/json`, `instruction-conservation.json` and
`generated-block-preservation.json` retain the mechanical checks.

All 14 prayer rows recover: 111,397; 111,403; 111,405; 111,409; 111,419;
111,430; 111,432; 111,436; 111,437; 111,441; 111,444; 111,453; 111,454;
and 111,456. The first five gain 3,088 clocks each, the other nine 3,256.
Their remaining signed instruction difference is the unchanged **-22-clock
main-return convention**. Dungeon rows 112,877 / 112,879 gain 2,192 each,
121,549 gains 2,248, and 123,618 / 123,620 gain 3,304 each. Their holds
recover while 1,024 / 1,024 / 120 / 456 / 456 instruction clocks remain
unpriced, respectively.

The twentieth cohort recovery, **112,191**, remains a mixed-mechanism
observation. Its 2,096 new Leever clocks change the existing raster-wait
decision from 0 to **269,136 elapsed clocks**; main instructions change
487,168 to 489,264, P2 elapsed 555,044 to 828,208, and holds 1 to 2.
The original frozen inverse was 150,082, which does not predict this coupled
wait change. Source retains 1,423 V-counter reads and **253,116 additional
executed poll-instruction clocks**; the after main-instruction gap is still
253,386, including 270 nonpoll clocks. No poll instructions or physical
acceptance phase are newly owned by this recovery. The only later ROM
recovery is **1,403,789**, gaining 1,176 instruction clocks and one hold.

### Remaining rows and their blockers

All fourteen remaining rows retain their original diagnostic boundary.
The nonpoll column excludes the separately executed upload protocol, or
the additional source spotlight polling, as appropriate. It is a measured
instruction residual, not a price or an inferred arrival correction.

| Iteration | Class | Diagnostic clocks short | Remaining nonpoll instruction gap | Blocker |
| --- | --- | ---: | ---: | --- |
| 34,318 | c | 265,206 | 20,730 | Unpositioned upload caller, receiver polling and return/suffix phase |
| 48,556 | c | 70,310 | 20,684 | Same upload/enable/suffix ownership gap |
| 95,127 | c | 156,042 | 20,846 | Bank-0 upload polling/return phase |
| 97,658 | a | 2,654 | 4,022 | Overlay caller `$02:AF1E` 3,488; loader `$02:FD0D` 344; map16 overlay entry 212; -22 return convention |
| 99,949 | a | 694 | 4,868 | Sprite `$41/$08` 858, `$41/$09` 62, `$d8/$09` 46; other signed instruction remainder 3,902 |
| 101,031 | d | 160,288 | 14,600 | 1,429 source V-counter reads; spotlight poll placement and remaining iris/caller work |
| 106,624 | a | 5,320 | 10,356 | Raven 580, blue archer 3,026, green bush guard 1,678; other remainder 5,072 |
| 106,956 | a | 4,606 | 10,110 | Raven/guard/archer/bush-guard bodies 4,074; other remainder 6,036 |
| 106,958 | a | 2,894 | 9,186 | Same sprite bodies 4,074; other remainder 5,112 |
| 106,960 | a | 282 | 9,186 | Same sprite bodies 4,074; other remainder 5,112 |
| 106,964 | a | 7,232 | 8,016 | Same sprite bodies 4,074; other remainder 3,942 |
| 109,000 | c | 112,868 | 28,972 | Bank-1 upload polling/return phase and remaining dungeon loaders |
| 109,016 | d | 151,160 | 2,302 | 1,428 source V-counter reads; spotlight poll placement and remaining caller instructions |
| 129,746 | c | 157,658 | 33,720 | Bank-1 upload polling/return phase and remaining dungeon loaders |

The seven (a) rows need further ROM ownership fixtures. The guard-area
remainders include item/tile callers and execution-chain work; the complete
signed scope and sprite partitions remain in the evidence rather than
silently treating aliases as additive prices. The five (c) rows each retain
2,508 unpriced sword/shield-copy clocks among their nonprotocol omissions.
Fresh source protocol work and current native protocol observations are in
`upload-phase-current.json`. Their differing executed polls are not imported
as prices. Nonprotocol instruction restoration alone is smaller than every
upload row's frozen boundary. The two remaining (d) rows retain 254,184 /
254,006 additional source poll-instruction clocks; acceptance and actual
executed waiting still require their owners.

### Full ROM admission

The required eight-slice seeded helper completes **1,473,123 iterations**,
**PASS / unexplained 0**, at **1,843 missing / 0 excess / 0 FP** versus
**1,864 / 0 / 0**. All runtime holds equal P2. Exactly 21 rows gain one
correct hold; zero holds are lost. Main instruction work increases by
**57,151,544 clocks on 125,216 rows**, with zero negative changes.
`rom-comparison.json` retains all 21 changed holds and the five prefixes.

| Before iteration | ROM missing before → candidate | Live missing before → candidate |
| --- | ---: | ---: |
| 94,334 | 2 → 2 | 2 → 2 |
| 141,568 | 34 → 14 | 34 → 14 |
| 228,499 | 142 → 122 | 142 → 131 |
| 380,243 | 275 → 255 | 275 → 273 |
| 461,989 | 334 → 314 | 334 → 446 |

The fresh eight-slice `--cpu-video` census also passes **1,473,123
iterations / unexplained 0**, with **1,462,997 exact pictures**, 563 early,
3,952 late, four during-lag, 905 polygon-thread, 4,702 torn and zero other
content differences. Every category and every per-segment video report
agrees with its hash-verified hunt-22 counterpart. All eight processes exit
zero; fresh per-iteration hashes and their identities are retained in
`cpu-pictures/` and `cpu-picture-agreement.json`.

Filtered Rust groups pass: **248 sprite-draw, 129 sprite-core, 7 player-OAM,
41 messaging**, and the six new fixture tests; groups overlap. All eight
generator tests pass. Product builds are warning-free. Library-test builds
retain the same 24 baseline dead-code warning headers, with no new headers.

Exactly **one** final default library suite runs after all runtime/fixture
edits, using the requested assets, packages, profile and `--lib` selection:
**SNES 505 passed / 7 ignored; Zelda 3,025 passed / 1 failed / 3 ignored**.
The sole failure is the permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
Exit 101 is recorded as that allowed failure, not an all-green suite.
Runtime and fixture hashes remain unchanged across the run.

### Single seeded live census and source-PC admission

Exactly **one** single-process seeded `--live-apu-reads --pacing-predictor`
census completes **1,473,123 iterations**, exit zero, **PASS / unexplained
0**. It carries SPC state across seeds. Its trace contains **6,060 native
poll rows**, versus 6,057 on the hash-verified main baseline. All four
ROM/live before/after runtime hold columns equal P2.

| Lane | Missing before → candidate | Excess before → candidate | FP before → candidate |
| --- | ---: | ---: | ---: |
| ROM | 1,864 → 1,843 | 0 → 0 | 0 → 0 |
| Live | 2,104 → 2,215 | 149 → 257 | 14 → 23 |
| Live minus ROM | 240 → 372 | 149 → 257 | 14 → 23 |

**Live worsens by 111 missing / 108 excess / 9 FP.** The 21 shared ROM
recoveries are offset by 132 / 108 / 9 of additional poll-phase differential.
There are **45 changed live hold rows**: 21 shared ROM recoveries and
**24 live-only changes**. The current live-versus-ROM differential has
61 rows, comprising the baseline's unchanged 37 rows plus those 24.

Every changed row has a song-end-poll attribution, but **nine do not have
the required source PC**. A second fresh cold CPU observation run covers
**249,400 host calls / 229,108 complete iterations**. Its APUI bus receipts
directly establish the first three episodes' read instruction:

| Episode | First sampled read / first zero iteration | Executed source read PC | Values |
| --- | --- | --- | --- |
| 94,334 | 94,334 / 94,854 | `$08:C3FD` / `$08:C3FD` | 19 / 0 |
| 141,568 | 141,568 / 142,088 | `$08:C3FD` / `$08:C3FD` | 19 / 0 |
| 228,499 | 228,499 / 229,035 | `$08:C3FD` / `$08:C3FD` | 19 / 0 |

These are pendant polls. The crystal path is `$08:C606`. The retained
all-episode `$08:C606` label is corrected for the first three episodes;
matching artifact hashes do not make that old label a valid source-PC
attribution. `poll-source-identity.json` and `source-polls/apu-bus.csv`
retain all six direct observations. No runtime classifier or matcher is
changed to relabel the pendant path.

| Poll episode / source PC | First native zero before → candidate | Changed live-only rows | Added missing / excess / FP | Recovery bound, exclusive |
| --- | --- | ---: | --- | ---: |
| 141,568 / `$08:C3FD` | 142,088 → 142,089 | 5 | +9 / +9 / +2 | 144,000 |
| 228,499 / `$08:C3FD` | 229,035 → 229,036 | 4 | +9 / +9 / +2 | 230,000 |
| 380,243 / `$08:C606` | 380,881 → 380,882 | 15 | +114 / +90 / +5 | 383,636 |
| Total | | **24** | **+132 / +108 / +9** | |

Bounds come from observed boolean poll divergence, independent main/VWF
budget differences and completed replay segments; changed hold rows do not
select the endpoints. The pendant main-budget differences end at 142,122 /
229,069, within the segments ending at 144,000 / 230,000; neither endpoint
is claimed as an actual reseed. The crystal episode's 1,062 differing main
budgets begin at 380,881 and end at 383,635. It crosses the observed 382,000
reseed and continues until the actual 383,636 resync. All eight actual
candidate reseeds are observed: **382,000; 383,636; 628,000; 629,845;
1,066,000; 1,066,963; 1,320,000; 1,320,900**.

Every live-only changed hold row follows. The ROM column is identical
before and after on all 24 rows; both counts are also retained separately
in `changed-live-holds.csv/json`.

| Iteration | Poll episode / actual source PC | Source lag | ROM holds | Live holds before → candidate |
| --- | --- | ---: | ---: | ---: |
| 142,089 | 141,568 / `$08:C3FD` | 5 | 5 | 5 → 0 |
| 142,090 | 141,568 / `$08:C3FD` | 0 | 0 | 0 → 5 |
| 142,092 | 141,568 / `$08:C3FD` | 4 | 4 | 4 → 0 |
| 142,093 | 141,568 / `$08:C3FD` | 3 | 3 | 3 → 4 |
| 142,095 | 141,568 / `$08:C3FD` | 0 | 0 | 0 → 3 |
| 229,036 | 228,499 / `$08:C3FD` | 5 | 5 | 5 → 0 |
| 229,037 | 228,499 / `$08:C3FD` | 0 | 0 | 0 → 5 |
| 229,039 | 228,499 / `$08:C3FD` | 4 | 4 | 4 → 0 |
| 229,042 | 228,499 / `$08:C3FD` | 0 | 0 | 0 → 4 |
| 380,946 | 380,243 / `$08:C606` | 1 | 1 | 1 → 0 |
| 380,947 | 380,243 / `$08:C606` | 0 | 0 | 0 → 1 |
| 380,956 | 380,243 / `$08:C606` | 5 | 4 | 4 → 0 |
| 380,957 | 380,243 / `$08:C606` | 0 | 0 | 0 → 4 |
| 381,426 | 380,243 / `$08:C606` | 8 | 8 | 8 → 0 |
| 381,427 | 380,243 / `$08:C606` | 0 | 0 | 0 → 8 |
| 383,178 | 380,243 / `$08:C606` | 0 | 0 | 0 → 7 |
| 383,181 | 380,243 / `$08:C606` | 7 | 7 | 7 → 0 |
| 383,343 | 380,243 / `$08:C606` | 7 | 7 | 7 → 0 |
| 383,346 | 380,243 / `$08:C606` | 0 | 0 | 0 → 7 |
| 383,631 | 380,243 / `$08:C606` | 59 | 59 | 59 → 0 |
| 383,632 | 380,243 / `$08:C606` | 6 | 6 | 6 → 0 |
| 383,633 | 380,243 / `$08:C606` | 23 | 22 | 22 → 0 |
| 383,634 | 380,243 / `$08:C606` | 1 | 1 | 1 → 59 |
| 383,635 | 380,243 / `$08:C606` | 1 | 1 | 1 → 6 |

Thus **15 changed rows satisfy the C606 source-PC boundary; nine fail it**.
There are zero causally unattributed song-end rows, but calling the nine
pendant rows C606 would be inaccurate. `outside-C606-changed-live-holds.csv`
isolates those nine. `poll-attribution.json` conserves every global and
episode missing/excess/FP delta and records the independent bounds.

### Verification boundary and retained candidate

**The batch fails the literal C606-only admission requirement.** The ROM,
CPU-picture, fixture, warning and default-library requirements pass, while
the live census is worse and nine live-only rows have source PC C3FD.
The pricing implementation is retained uncommitted as an **unadmitted
candidate**, with `candidate-runtime.patch` and the measured
`zelda3-candidate` binary under `target/prefix-hunt23/`. No commit or push
occurs, and no accepted-pricing claim is made.

No runtime or fixture edits follow the admission launches. No P2,
classifier, matcher, timing-epoch expectation or environment-switch change
is included. Runtime prices do not use per-scene values or measured
deficits. No injected holds, interpreter fallback, GPU run, `--video`,
cached-A/V replay, external
Snes9x process, second live census or second full default suite occurs.
Every Zelda binary invocation uses `--logical-replay` with CPU-only modes.

Final hash checks preserve all **35 reused prior artifacts**, **14 authority
files**, and **620 source/fixture files** frozen before the censuses. The
worktree target occupies approximately **5.55 GB**, below the
8,000,000,000-byte limit. Only rebuildable files within this worktree's
`target/` were removed by this hunt; both cleanup lists are retained.
All launched processes have completed, with no background process left.
`verification.json`, `resources.json`, `prior-final-verification.json`
and `artifact-hashes.json` retain the final checks and evidence identities.

These are seeded logical, CPU-picture and source-instruction measurements.
Actual physical main/wait acceptance phase, complete spotlight polling and
upload arrival/receiver/return ownership remain unresolved. Cold exact A/V
acceptance remains a separate boundary.

### Admission note (reviewer)

Admitted after review. The C606-only wording of the admission rule came from the
earlier mislabeling of all song-end episodes as crystal polls; the rule's intent
is "every live-only changed hold row belongs to a song-end poll episode". The
nine rows above belong to the pendant song-end poll `$08:C3FD`, whose source PC
this hunt established directly, and each is a one-iteration shift of the native
first-zero read (142,088 → 142,089; 229,035 → 229,036). Independent light gate:
eight-slice ROM census PASS 1,843 / 0 / 0, live recount 2,215 / 257 / 23,
filtered tests pass. Later hunts admit `$08:C3FD` or `$08:C606` attributions.

## Lagless prefix hunt 24

Worktree `/Users/missingno/Documents/zelda3-rs-codex`, branch
`codex/prefix-hunt24`, base **6e6d34cc**. Instruction pricing restores
**219,868 clocks on the 122-row cohort**, recovering all seven early (a)
rows and **25 of the next 108 rows**. Across the full ROM route, **150
missing holds recover**, with none lost; main work increases by
**670,454,218 instruction clocks on 257,225 iterations**, with no negative
budget changes. Prices follow executed unmodified-ROM blocks and actual
branch/page-crossing outcomes. All receipts below are under
`target/prefix-hunt24/`.

### Fresh classification and ownership

The 36 reused hunt-23 artifacts match their recorded hashes. The authorities
and current source agree with the measured main baseline. A new cold
unmodified-ROM CPU run covers **249,400 host calls / 229,108 complete
iterations**, collecting per-PC, routine, sprite, structural, timing, NMI
and APUI receipts for every missing row in both cohorts. The new native
baseline agrees with main's ROM holds and main budgets throughout the
window. The single-process versus eight-slice comparison has 23 rows with NMI,
wait or elapsed differences, beginning at the 186,000 slice boundary.
`baseline-agreement.json` retains them; they are not reused as physical
arrival evidence.

| Class | Before 141,568, before → remaining | 141,568–228,499, before → remaining |
| --- | ---: | ---: |
| (a) Nonpoll instruction shortfall | 7 → 0 | 89 → 65 |
| (b) Independently established inherited arrival | 0 → 0 | 0 → 0 |
| (c) Upload protocol / receiver / return phase | 5 → 5 | 4 → 3 |
| (d) Spotlight poll or insufficient instruction-only closure | 2 → 2 | 15 → 15 |
| Recovered holds | 0 → 7 | 0 → 25 |

`classification-before.json/csv`, `classification-after.json/csv` and the
combined `classification.json/csv` retain every row's owners, signed
instruction gaps, protocol/poll exclusions, before/after holds and clocks.
`raw-owner-residuals.csv` preserves negative routine residuals as well as
positive ones. Parent/child aliases are not additive missing work. Class
(b) stays empty because no independent physical arrival proof was obtained;
a diagnostic P2 inverse is not that proof.

The next-cohort ranking, computed before pricing, starts with Giant
Moldorm `$09/$09` (**11 rows / 10 independent frozen-boundary closures**),
Mini Moldorm `$18/$09` (**7 / 6**), walking Zora `$56/$09` (**12 / 6**),
Keese `$6f/$09` (**6 / 3**), bumper `$93/$09` (**10 / 3**) and fake sword
`$e8/$09` (**4 / 3**). Closure counts overlap and do not promise closure
after the coupled raster wait is recomputed. `owner-ranking-before.json`
keeps the two cohorts separate; `owner-ranking-after.json` ranks the
remaining sprite families. Keese/fake-sword caller and draw omissions, plus bumper draw/helper
omissions, remain deferred.

### ROM-exact owners and fixtures

The table partitions the cohort's *net added work* by actual sprite call
frames and non-sprite owners. Shared children and routine-scope transfers
are reconciled before summing; raw routine increases alone would overcount.
The row column gives rows with new work / recovered rows containing that
owner. These contributions overlap; they are not isolated counterfactual
closures. `priced-owner-rows.json` retains each contribution.

| Owner | Cohort clocks added | Rows with work / recovered | Unmodified-ROM fixture (prefix `prefix24_`) |
| --- | ---: | ---: | --- |
| Overlay caller `$02:AF1E`, loader `$02:FD0D`, overlay map8 entry `$02:FD26` | 8,088 | 2 / 2 | `overlay_caller_loader_and_map8_entry_match_all_rom_paths` |
| Sword/shield palette-copy child `$1B:EF7B` | 20,064 | 8 / 1 | `sword_and_shield_copy_children_match_unmodified_rom` |
| Raven bounce / long / recoil / predicate wrappers | 34,098 | 28 / 7 | `raven_wrappers_and_recoil_prefix_match_rom` |
| Guard preparation/parry, bush caller/states/draw, archer scan/movement, heart call and other shared sprite children | 18,810 | 15 / 7 | `guard_preparation_all_subtypes_and_floors_match_rom`; `guard_parry_call_and_heart_first_call_match_rom`; `green_bush_guard_caller_and_states_match_rom`; `bush_soldier_draw_all_graphics_and_clipping_match_rom`; `archer_scan_and_clear_movement_match_all_rom_paths` |
| Item/tile selection `$1B:BD7A/BD82`, execution callers `$07:DC4A/DC2E` | 26,598 | 15 / 9 | `item_tile_selection_prefix_matches_every_rom_comparison_path`; `item_tile_execution_chain_dispatch_matches_rom`; `item_execution_dispatch_and_existing_effect_prices_match_all_bytes` |
| Dash control `$07:8F86..915D` | 33,890 | 15 / 9 | `dash_control_callers_match_rom_paths` |
| Mini Moldorm on-screen draw `$1D:F829..F942` | 48,812 | 6 / 4 | `mini_moldorm_on_screen_body_all_paths_match_rom` |
| Giant Moldorm bounce, entry, segment-explosion gates and coordinate children | 10,274 | 11 / 10 | `giant_moldorm_entry_and_coordinate_owners_match_rom`; `giant_moldorm_segment_explosion_all_gates_match_rom` |
| Walking Zora dispatcher, wait/surface and preparation children | 19,234 | 12 / 4 | `walking_zora_dispatch_wait_and_surface_match_rom` |
| **Total** | **219,868** | **80 / 32 distinct rows** | **16 fixture tests** |

All-path fixtures cover every priced conditional; dash asserts both outcomes
of all 25 branches, overlay covers all caller comparisons, bush draw covers
all seven graphics values and clipping, and the shared bush-state fixture
covers both `$47` and `$49`. Mini Moldorm covers all 16 directions, legal
ROM slots 0–3, history phases, coordinate clipping/carry and table crossing.
The item execution fixture covers all 256 input bytes, supported handlers,
cheat and platform cases. Children are excluded from owner scopes, and
new caller charges select their actual near/long ROM entry rather than
charging the same child twice. The guard parry JSL belongs to guard callers,
not the shared Bluesain child.

The copy is exactly **1,092 sword + 1,416 shield = 2,508 clocks** wherever
both calls execute, including each of the five early upload rows. Upload
protocol and spotlight-poll prices are unchanged. Item/tile mutation and
secret suffixes, Zora walking/thrown bodies, and other explicitly unpriced
suffixes remain outside the admitted scopes.

All **5,309 old generated blocks are unchanged**, with **255 new blocks**.
Expanded marker deltas add **214,432 clocks / 7,414 instruction executions
on 4,973 per-PC rows**, with zero negative counts or source-count/clock
excesses. The remaining **5,436 clocks** are existing exact children:
**3,278 coordinate clocks** (eleven 298-clock calls) and **2,158 tile-effect
clocks**. Other scalar movements cancel as routine attribution changes.
Every row's main-budget delta equals its routine-total delta.
`per-pc-conservation.csv`, `instruction-conservation.json`,
`scalar-conservation.json`, `sprite-owner-deltas.csv` and
`priced-owner-clocks.json` retain these reconciliations. Regeneration is
byte-identical to `rom_block_costs.rs`; all eight generator tests pass.

An initial candidate also priced Mini Moldorm's `$1D:F822..F828` entry.
Those exact 122 clocks at iteration **931,965** moved the unchanged P2
raster wait from zero to **270,204 elapsed clocks**, creating one excess
hold (candidate totals 1,691 / 1 / 0). That candidate is rejected. The final
owner starts uniformly at the natural **on-screen body F829**; the entry,
carry check and off-screen return remain unpriced on *every* path. No
scene selector, measured deficit or spotlight adjustment is introduced.
`entry-prefix-exclusion.json` retains the rejected row and both budgets.
The final ROM result is **1,693 / 0 / 0**.

### Remaining rows and their blockers

The seven early (a) rows gain **4,044 / 4,768 / 8,054 / 7,774 / 7,774 /
7,774 / 7,300** clocks at iterations **97,658 / 99,949 / 106,624 /
106,956 / 106,958 / 106,960 / 106,964**, respectively. All holds recover.
Their after nonpoll residuals are **-22 / 100 / 2,302 / 2,336 / 1,412 /
1,412 / 716**. Recovering a hold does not claim complete instruction ownership.

The following table covers every remaining row in both cohorts. The
nonpoll gap removes executed source upload protocol and extra spotlight
poll instructions. Signed sprite gaps use disjoint call-frame partitions;
the other remainder can be negative because existing return conventions
and attribution differences are retained. The diagnostic clocks short are
recomputed by unchanged P2 and are never used as prices.

| Iteration | Class | Diagnostic clocks short | Nonpoll instruction gap | Remaining owner / blocker |
| --- | --- | ---: | ---: | --- |
| 34,318 | c | 262,698 | 18,222 | Upload receiver/poll/return and unpriced callers/loaders; copy gain 2,508 |
| 48,556 | c | 67,802 | 18,176 | Upload receiver/poll/return and unpriced callers/loaders; copy gain 2,508 |
| 95,127 | c | 153,534 | 18,338 | Upload receiver/poll/return and unpriced callers/loaders; copy gain 2,508 |
| 101,031 | d | 160,288 | 14,600 | Spotlight: 1,429 V-counter reads; dash item `$3b/$09` 2,588; remaining iris/caller work |
| 109,000 | c | 110,360 | 26,464 | Upload receiver/poll/return and unpriced callers/loaders; copy gain 2,508 |
| 109,016 | d | 151,160 | 2,302 | Spotlight: 1,428 V-counter reads; big fairy `$c8/$09` 1,600; fairy `$72/$09` 724; remaining iris/caller work |
| 129,746 | c | 155,150 | 31,212 | Upload receiver/poll/return and unpriced callers/loaders; copy gain 2,508 |
| 146,148 | c | 38,076 | 75,156 | Upload receiver/poll/return and unpriced callers/loaders; no copy on this path |
| 146,786 | d | 2,700 | -22 | Main/NMI/HDMA arrival and executed-wait ownership; instruction work alone is insufficient |
| 146,790 | d | 2,602 | -22 | Main/NMI/HDMA arrival and executed-wait ownership; instruction work alone is insufficient |
| 146,794 | d | 5,012 | -22 | Main/NMI/HDMA arrival and executed-wait ownership; instruction work alone is insufficient |
| 146,798 | d | 7,822 | -22 | Main/NMI/HDMA arrival and executed-wait ownership; instruction work alone is insufficient |
| 146,802 | d | 10,414 | -22 | Main/NMI/HDMA arrival and executed-wait ownership; instruction work alone is insufficient |
| 146,806 | d | 12,286 | -22 | Main/NMI/HDMA arrival and executed-wait ownership; instruction work alone is insufficient |
| 146,810 | d | 14,732 | -22 | Main/NMI/HDMA arrival and executed-wait ownership; instruction work alone is insufficient |
| 150,378 | a | 22,484 | 50,152 | sprite $ec `$ec/$06` 10,178; sprite $0d `$0d/$09` 776; other signed caller/child remainder 39,198 |
| 154,637 | a | 1,334 | 5,674 | sprite $55 `$55/$09` 2,004; sprite $eb `$eb/$09` 1,470; sprite $52 `$52/$09` 902; other signed caller/child remainder 1,298 |
| 158,519 | a | 7,586 | 10,258 | walking Zora `$56/$09` 6,408; sprite $eb `$eb/$09` 1,470; sprite $55 `$55/$09` 1,076; other signed caller/child remainder 1,304 |
| 158,538 | a | 2,878 | 12,034 | walking Zora `$56/$09` 8,224; sprite $eb `$eb/$09` 1,470; sprite $55 `$55/$09` 1,036; other signed caller/child remainder 1,304 |
| 158,558 | a | 10,088 | 10,892 | walking Zora `$56/$09` 7,336; sprite $eb `$eb/$09` 1,470; sprite $55 `$55/$09` 782; other signed caller/child remainder 1,304 |
| 158,719 | a | 248 | 1,198 | walking Zora `$56/$09` 540; sprite $eb `$eb/$09` 456; sprite $55 `$55/$09` 218; other signed caller/child remainder -16 |
| 158,729 | a | 840 | 1,198 | walking Zora `$56/$09` 540; sprite $eb `$eb/$09` 456; sprite $55 `$55/$09` 218; other signed caller/child remainder -16 |
| 160,109 | a | 720 | 918 | sprite $37 `$37/$09` 848; sprite $58 `$58/$09` 92; other signed caller/child remainder -22 |
| 160,154 | a | 906 | 1,428 | sprite $55 `$55/$09` 500; other signed caller/child remainder 928 |
| 161,123 | a | 6,540 | 7,132 | sprite $55 `$55/$09` 3,544; sprite $eb `$eb/$09` 1,672; other signed caller/child remainder 1,916 |
| 162,301 | a | 1,548 | 7,376 | sprite $52 `$52/$09` 3,636; sprite $eb `$eb/$09` 456; other signed caller/child remainder 3,284 |
| 162,819 | a | 6,686 | 7,110 | sprite $52 `$52/$09` 3,636; sprite $eb `$eb/$09` 456; other signed caller/child remainder 3,018 |
| 168,346 | d | 83,650 | 8,290 | Main/NMI/HDMA arrival and executed-wait ownership; sprite $55 `$55/$09` 500; instruction work alone is insufficient |
| 171,309 | d | 21,222 | 8,060 | Main/NMI/HDMA arrival and executed-wait ownership; sprite $2b `$2b/$09` 1,454; instruction work alone is insufficient |
| 171,310 | d | 63,222 | 1,432 | Main/NMI/HDMA arrival and executed-wait ownership; sprite $2b `$2b/$09` 1,454; instruction work alone is insufficient |
| 172,380 | d | 143,228 | -116 | Main/NMI/HDMA arrival and executed-wait ownership; instruction work alone is insufficient |
| 174,033 | a | 665 | 2,210 | sprite $d4 `$d4/$09` 888; sprite $d9 `$d9/$09` 526; other signed caller/child remainder 796 |
| 174,034 | a | 1,185 | 2,498 | sprite $d4 `$d4/$09` 888; sprite $d9 `$d9/$09` 526; other signed caller/child remainder 1,084 |
| 174,280 | c | 112,958 | 26,464 | Upload receiver/poll/return and unpriced callers/loaders; copy gain 2,508 |
| 174,296 | d | 152,714 | 2,302 | Spotlight: 1,427 V-counter reads; big fairy `$c8/$09` 1,600; fairy `$72/$09` 724; remaining iris/caller work |
| 180,178 | a | 24,900 | 29,614 | sprite $48 `$48/$09` 16,866; sprite $ec `$ec/$06` 1,428; sprite $c5 `$c5/$09` 208; other signed caller/child remainder 11,112 |
| 180,182 | a | 18,464 | 24,596 | sprite $48 `$48/$09` 9,762; sprite $ec `$ec/$06` 1,152; sprite $c5 `$c5/$09` 208; other signed caller/child remainder 13,474 |
| 181,567 | a | 2,746 | 5,652 | fake sword `$e8/$09` 3,064; sprite $c4 `$c4/$09` 920; sprite $d9 `$d9/$09` 600; sprite $dc `$dc/$09` 600; sprite $e1 `$e1/$09` 300; other signed caller/child remainder 168 |
| 181,568 | a | 3,684 | 5,652 | fake sword `$e8/$09` 3,064; sprite $c4 `$c4/$09` 920; sprite $dc `$dc/$09` 600; sprite $d9 `$d9/$09` 600; sprite $e1 `$e1/$09` 300; other signed caller/child remainder 168 |
| 181,569 | a | 2,874 | 5,652 | fake sword `$e8/$09` 3,064; sprite $c4 `$c4/$09` 920; sprite $d9 `$d9/$09` 600; sprite $dc `$dc/$09` 600; sprite $e1 `$e1/$09` 300; other signed caller/child remainder 168 |
| 181,575 | a | 2,452 | 5,652 | fake sword `$e8/$09` 3,064; sprite $c4 `$c4/$09` 920; sprite $dc `$dc/$09` 600; sprite $d9 `$d9/$09` 600; sprite $e1 `$e1/$09` 300; other signed caller/child remainder 168 |
| 186,606 | a | 1,348 | 2,166 | sprite $17 `$17/$09` 892; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,608 | a | 1,936 | 2,166 | sprite $17 `$17/$09` 892; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,610 | a | 1,104 | 2,166 | sprite $17 `$17/$09` 892; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,614 | a | 1,498 | 2,166 | sprite $17 `$17/$09` 892; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,616 | a | 2,048 | 2,166 | sprite $17 `$17/$09` 892; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,618 | a | 1,156 | 2,166 | sprite $17 `$17/$09` 892; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,620 | a | 1,594 | 2,166 | sprite $17 `$17/$09` 892; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,622 | a | 1,218 | 2,166 | sprite $17 `$17/$09` 892; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,624 | a | 1,812 | 2,166 | sprite $17 `$17/$09` 892; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,626 | a | 1,280 | 2,166 | sprite $17 `$17/$09` 892; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,664 | a | 2,120 | 4,004 | sprite $33 `$33/$09` 1,234; sprite $17 `$17/$09` 892; sprite $79 `$79/$09` 604; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,666 | a | 1,090 | 4,004 | sprite $33 `$33/$09` 1,234; sprite $17 `$17/$09` 892; sprite $79 `$79/$09` 604; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,668 | a | 2,070 | 4,004 | sprite $33 `$33/$09` 1,234; sprite $17 `$17/$09` 892; sprite $79 `$79/$09` 604; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,670 | a | 1,588 | 4,004 | sprite $33 `$33/$09` 1,234; sprite $17 `$17/$09` 892; sprite $79 `$79/$09` 604; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,672 | a | 1,998 | 4,004 | sprite $33 `$33/$09` 1,234; sprite $17 `$17/$09` 892; sprite $79 `$79/$09` 604; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,684 | a | 3,108 | 4,004 | sprite $33 `$33/$09` 1,234; sprite $17 `$17/$09` 892; sprite $79 `$79/$09` 604; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,686 | a | 2,996 | 4,004 | sprite $33 `$33/$09` 1,234; sprite $17 `$17/$09` 892; sprite $79 `$79/$09` 604; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,690 | a | 2,212 | 4,004 | sprite $33 `$33/$09` 1,234; sprite $17 `$17/$09` 892; sprite $79 `$79/$09` 604; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,694 | a | 2,926 | 4,004 | sprite $33 `$33/$09` 1,234; sprite $17 `$17/$09` 892; sprite $79 `$79/$09` 604; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,696 | a | 1,302 | 4,004 | sprite $33 `$33/$09` 1,234; sprite $17 `$17/$09` 892; sprite $79 `$79/$09` 604; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 186,698 | a | 2,340 | 4,004 | sprite $33 `$33/$09` 1,234; sprite $17 `$17/$09` 892; sprite $79 `$79/$09` 604; sprite $0d `$0d/$09` 582; raven `$00/$09` 380; sprite $ac `$ac/$09` 144; other signed caller/child remainder 168 |
| 187,109 | d | 138,608 | 8,070 | Spotlight: 1,408 V-counter reads; Keese `$6f/$09` 6,912; sprite $ad `$ad/$09` 1,180; remaining iris/caller work |
| 187,779 | a | 7,010 | 7,200 | Keese `$6f/$09` 3,840; other signed caller/child remainder 3,360 |
| 187,781 | a | 82 | 5,098 | Keese `$6f/$09` 3,840; other signed caller/child remainder 1,258 |
| 187,782 | a | 1,442 | 5,098 | Keese `$6f/$09` 3,840; other signed caller/child remainder 1,258 |
| 188,654 | a | 1,956 | 4,710 | Keese `$6f/$09` 3,840; other signed caller/child remainder 870 |
| 188,962 | d | 230,604 | 4,768 | Spotlight: 1,414 V-counter reads; Keese `$6f/$09` 3,840; remaining iris/caller work |
| 189,344 | a | 5,586 | 6,014 | sprite $c2 `$c2/$09` 1,192; sprite $27 `$27/$09` 192; other signed caller/child remainder 4,630 |
| 189,368 | a | 3,306 | 4,630 | sprite $c2 `$c2/$09` 1,192; sprite $ad `$ad/$09` 610; other signed caller/child remainder 2,828 |
| 189,923 | d | 224,566 | 1,170 | Spotlight: 1,423 V-counter reads; sprite $c2 `$c2/$09` 1,192; remaining iris/caller work |
| 190,815 | a | 1,836 | 2,220 | sprite $c2 `$c2/$09` 894; sprite $27 `$27/$09` 192; other signed caller/child remainder 1,134 |
| 194,151 | c | 19,768 | 43,808 | Upload receiver/poll/return and unpriced callers/loaders; copy gain 2,508 |
| 194,994 | a | 2,356 | 5,222 | sprite $87 `$87/$09` 3,024; Mini Moldorm `$18/$09` 230; sprite $86 `$86/$09` 216; other signed caller/child remainder 2,904 |
| 196,051 | a | 7,518 | 8,090 | other signed caller/child remainder 8,090 |
| 198,239 | a | 540 | 3,990 | sprite $a7 `$a7/$09` 1,760; Mini Moldorm `$18/$09` 882; sprite $e4 `$e4/$09` 248; other signed caller/child remainder 1,100 |
| 198,240 | a | 124 | 3,872 | sprite $a7 `$a7/$09` 1,760; Mini Moldorm `$18/$09` 882; sprite $e4 `$e4/$09` 248; other signed caller/child remainder 982 |
| 209,808 | a | 4,708 | 14,724 | sprite $26 `$26/$09` 96; other signed caller/child remainder 15,104 |
| 215,856 | a | 1,016 | 9,814 | bumper `$93/$09` 1,236; sprite $7e `$7e/$09` 282; other signed caller/child remainder 8,296 |
| 215,857 | a | 2,232 | 2,500 | bumper `$93/$09` 1,236; sprite $7e `$7e/$09` 282; other signed caller/child remainder 982 |
| 215,858 | a | 2,234 | 2,500 | bumper `$93/$09` 1,236; sprite $7e `$7e/$09` 282; other signed caller/child remainder 982 |
| 215,859 | a | 2,232 | 2,500 | bumper `$93/$09` 1,236; sprite $7e `$7e/$09` 282; other signed caller/child remainder 982 |
| 217,218 | a | 2,550 | 8,676 | sprite $26 `$26/$08` 2,772; bumper `$93/$08` 1,800; sprite $7e `$7e/$08` 170; other signed caller/child remainder 3,934 |
| 217,219 | a | 1,016 | 9,696 | bumper `$93/$09` 1,236; sprite $7e `$7e/$09` 282; other signed caller/child remainder 8,178 |
| 217,220 | a | 2,232 | 2,618 | bumper `$93/$09` 1,236; sprite $7e `$7e/$09` 282; other signed caller/child remainder 1,100 |
| 217,221 | a | 1,816 | 2,500 | bumper `$93/$09` 1,236; sprite $7e `$7e/$09` 282; other signed caller/child remainder 982 |
| 217,222 | a | 2,232 | 2,500 | bumper `$93/$09` 1,236; sprite $7e `$7e/$09` 282; other signed caller/child remainder 982 |
| 219,129 | a | 54 | 82 | other signed caller/child remainder 82 |
| 219,689 | a | 194 | 6,410 | bumper `$93/$09` 1,236; sprite $7e `$7e/$09` 92; other signed caller/child remainder 5,082 |
| 225,257 | a | 4,570 | 6,292 | bumper `$93/$09` 1,236; sprite $7e `$7e/$09` 92; other signed caller/child remainder 4,964 |

The eight remaining upload rows have separate protocol owners in the fresh
APUI and routine receipts. Some retain enough *unpriced nonprotocol* work
for a future instruction batch; that does not establish upload acceptance
or authorize importing a measured poll count as a price. The seven
146,786–146,810 rows already have a **-22-clock** signed main-instruction
residual: their phase cannot be repaired by inventing more main work.
`classification.json` names every remaining sprite family/state and keeps
raw positive routine aliases separate from additive owners. The per-slot,
per-routine `remaining-sprite-residuals.csv` retains every signed child
and wrapper difference. The next scopes include Keese caller `$05:AA8B`
and draw entry `$06:DC10`, bumper draw `$1E:AA8B` (the bumper caller is
already priced), fake-sword caller/draw `$05:EEAF/EEE6` and thrown
interaction `$06:E02A`, plus Zora walking/thrown, draw and shadow suffixes.
Those are owner candidates, not sums of raw positive aliases.

### Full ROM, CPU pictures and live census

The eight-slice seeded ROM helper completes **1,473,123 iterations**,
**PASS / unexplained 0**, at **1,843 → 1,693 missing**, **0 → 0 excess**,
**0 → 0 FP**. All 150 changed holds recover one missing hold, with zero
losses and zero runtime-hold/P2 disagreements.

| Before iteration | ROM missing before → after | Live missing before → after |
| --- | ---: | ---: |
| 94,334 | 2 → 2 | 2 → 2 |
| 141,568 | 14 → 7 | 14 → 7 |
| 228,499 | 122 → 90 | 131 → 90 |
| 380,243 | 255 → 214 | 273 → 223 |
| 461,989 | 314 → 270 | 446 → 279 |

The eight-slice `--cpu-video` census also passes **1,473,123 iterations /
unexplained 0**, retaining **1,462,997 exact pictures**, 563 early, 3,952
late, four during-lag, 905 polygon-thread, 4,702 torn and zero other content
misses. Every picture hash and every segment's video categories are
identical to main, independently of changed pacing budgets.
`cpu-picture-agreement.json` and `cpu-picture-gate.txt` retain the proof.

Exactly **one** continuous seeded `--live-apu-reads --pacing-predictor`
process completes all **1,473,123 iterations**; SPC state carries across
its seeds. Live totals are **2,215 → 2,105 missing**,
**257 → 318 excess**, **23 → 85 FP**.
`live-command.json` records the single launch, completed exit and final
binary hash. `live-comparison.json` retains the totals and prefixes;
`live-timing.jsonl.zst` retains the continuous native/source poll trace.
Missing improves, while live excess and FP increase by **61 / 62**. The
881,095 episode contributes **+234 / +230 / +67** missing/excess/FP across
135 changed rows after its first native zero moves one iteration early.
The net live-only delta is **+40 / +61 / +62**; the 150 shared ROM
recoveries make the overall live missing delta **-110**. These poll-phase
regressions are retained and attributed, not described as solved pacing.


### Song-end attribution

All **179 live-only changed hold rows** are attributed:
**5 pendant `$08:C3FD`**, **174 crystal `$08:C606`**,
**zero unattributed**. The comparison has **329 total changed live rows**,
of which **150 share the exact ROM before/after hold pair**.
The conservative live-only set includes every other changed pair.

The first cold source pass recorded routine profiles without bus instrumentation;
its command and output are retained. A corrected cold unmodified-ROM CPU
source replay, with the existing block/bus and timing-audit switches enabled,
observes **20 direct APUI
receipts**: a nonzero and first-zero `$2140` read in each of ten song-end
episodes. `poll-source-identity.json` establishes the actual source PCs;
sprite labels are not used to infer pendant versus crystal ownership.
Routine, sprite and structural profiles from the two source passes are
byte-identical (`source-poll-profile-agreement.json`).
The episode bounds below are chosen *without hold changes*: actual
native/source boolean polls, native/source context divergence, four-lane
main/VWF budget differences, then actual reseeding or the containing
2,000-iteration segment boundary. The four lanes remove the intended ROM
price delta from the live instruction delta. `poll-attribution.json`
conserves every missing/excess/FP delta, including shared ROM recoveries.

| Episode | Direct source PC | Source first zero | Main live first zero | Hunt 24 first zero | Independent recovery end | Live-only changed rows |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 94,334 | `$08:C3FD` | 94,854 | 94,854 | 94,854 | 96,000 | 0 |
| 141,568 | `$08:C3FD` | 142,088 | 142,089 | 142,088 | 144,000 | 5 |
| 228,499 | `$08:C3FD` | 229,035 | 229,036 | 229,036 | 230,000 | 0 |
| 380,243 | `$08:C606` | 380,881 | 380,882 | 380,881 | 383,636 | 15 |
| 461,989 | `$08:C606` | 462,628 | 462,628 | 462,627 | 466,000 | 6 |
| 626,618 | `$08:C606` | 627,256 | 627,257 | 627,257 | 629,845 | 0 |
| 735,229 | `$08:C606` | 735,867 | 735,867 | 735,868 | 738,000 | 6 |
| 881,095 | `$08:C606` | 881,734 | 881,734 | 881,733 | 884,327 | 135 |
| 1,064,096 | `$08:C606` | 1,064,734 | 1,064,735 | 1,064,734 | 1,066,963 | 12 |
| 1,317,697 | `$08:C606` | 1,318,335 | 1,318,336 | 1,318,336 | 1,320,900 | 0 |

Every live-only changed row follows. `changed-live-holds.csv/json` retains
its metric deltas and the independently observed bounds as well.

| Iteration | Source lag | ROM holds before → after | Live holds before → after | Episode / direct source PC |
| --- | ---: | ---: | ---: | --- |
| 142,089 | 5 | 5 → 5 | 0 → 5 | 141,568 / `$08:C3FD` |
| 142,090 | 0 | 0 → 0 | 5 → 0 | 141,568 / `$08:C3FD` |
| 142,092 | 4 | 4 → 4 | 0 → 4 | 141,568 / `$08:C3FD` |
| 142,093 | 3 | 3 → 3 | 4 → 3 | 141,568 / `$08:C3FD` |
| 142,095 | 0 | 0 → 0 | 3 → 0 | 141,568 / `$08:C3FD` |
| 380,946 | 1 | 1 → 1 | 0 → 1 | 380,243 / `$08:C606` |
| 380,947 | 0 | 0 → 0 | 1 → 0 | 380,243 / `$08:C606` |
| 380,956 | 5 | 4 → 4 | 0 → 4 | 380,243 / `$08:C606` |
| 380,957 | 0 | 0 → 0 | 4 → 0 | 380,243 / `$08:C606` |
| 381,426 | 8 | 8 → 8 | 0 → 8 | 380,243 / `$08:C606` |
| 381,427 | 0 | 0 → 0 | 8 → 0 | 380,243 / `$08:C606` |
| 383,178 | 0 | 0 → 0 | 7 → 0 | 380,243 / `$08:C606` |
| 383,181 | 7 | 7 → 7 | 0 → 7 | 380,243 / `$08:C606` |
| 383,343 | 7 | 7 → 7 | 0 → 7 | 380,243 / `$08:C606` |
| 383,346 | 0 | 0 → 0 | 7 → 0 | 380,243 / `$08:C606` |
| 383,631 | 59 | 59 → 59 | 0 → 59 | 380,243 / `$08:C606` |
| 383,632 | 6 | 6 → 6 | 0 → 6 | 380,243 / `$08:C606` |
| 383,633 | 23 | 22 → 22 | 0 → 22 | 380,243 / `$08:C606` |
| 383,634 | 1 | 1 → 1 | 59 → 1 | 380,243 / `$08:C606` |
| 383,635 | 1 | 1 → 1 | 6 → 1 | 380,243 / `$08:C606` |
| 462,699 | 0 | 0 → 0 | 0 → 1 | 461,989 / `$08:C606` |
| 462,700 | 1 | 1 → 1 | 1 → 0 | 461,989 / `$08:C606` |
| 462,709 | 0 | 0 → 0 | 0 → 4 | 461,989 / `$08:C606` |
| 462,710 | 5 | 4 → 4 | 4 → 0 | 461,989 / `$08:C606` |
| 463,179 | 0 | 0 → 0 | 0 → 8 | 461,989 / `$08:C606` |
| 463,180 | 8 | 8 → 8 | 8 → 0 | 461,989 / `$08:C606` |
| 735,937 | 1 | 1 → 1 | 1 → 0 | 735,229 / `$08:C606` |
| 735,938 | 0 | 0 → 0 | 0 → 1 | 735,229 / `$08:C606` |
| 735,947 | 5 | 4 → 4 | 4 → 0 | 735,229 / `$08:C606` |
| 735,948 | 0 | 0 → 0 | 0 → 4 | 735,229 / `$08:C606` |
| 736,420 | 8 | 7 → 7 | 7 → 0 | 735,229 / `$08:C606` |
| 736,422 | 0 | 0 → 0 | 0 → 7 | 735,229 / `$08:C606` |
| 881,794 | 0 | 0 → 0 | 0 → 1 | 881,095 / `$08:C606` |
| 881,795 | 1 | 1 → 1 | 1 → 0 | 881,095 / `$08:C606` |
| 881,804 | 0 | 0 → 0 | 0 → 4 | 881,095 / `$08:C606` |
| 881,805 | 5 | 4 → 4 | 4 → 0 | 881,095 / `$08:C606` |
| 882,275 | 8 | 8 → 8 | 8 → 0 | 881,095 / `$08:C606` |
| 882,276 | 0 | 0 → 0 | 0 → 8 | 881,095 / `$08:C606` |
| 882,475 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,476 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,477 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,484 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,485 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,486 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,529 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,530 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,531 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,538 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,539 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,540 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,591 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,592 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,593 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,600 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,601 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,602 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,676 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,677 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,678 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,686 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,687 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,688 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,740 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,741 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,742 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,750 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,751 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,752 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,804 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,805 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,806 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,814 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,815 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,816 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,893 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,894 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,895 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,919 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,920 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,921 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,957 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,958 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,959 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 882,983 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,984 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 882,985 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,053 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,054 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,055 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,071 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,072 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,073 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,117 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,118 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,119 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,135 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,136 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,137 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,181 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,182 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,183 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,199 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,200 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,201 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,272 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,273 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,274 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,298 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,299 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,300 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,338 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,339 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,340 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,364 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,365 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,366 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,398 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,399 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,400 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,424 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,425 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,426 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,483 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,484 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,485 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,509 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,510 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,511 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,539 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,540 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,541 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,565 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,566 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,567 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,601 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,602 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,603 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,627 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,628 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,629 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,697 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,698 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,699 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,724 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,725 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,726 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,727 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,728 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,729 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,754 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,755 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,756 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,765 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,766 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,767 | 2 | 2 → 2 | 2 → 0 | 881,095 / `$08:C606` |
| 883,792 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,793 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,794 | 0 | 0 → 0 | 0 → 2 | 881,095 / `$08:C606` |
| 883,842 | 7 | 7 → 7 | 7 → 0 | 881,095 / `$08:C606` |
| 883,881 | 0 | 0 → 0 | 0 → 7 | 881,095 / `$08:C606` |
| 883,977 | 7 | 7 → 7 | 7 → 0 | 881,095 / `$08:C606` |
| 884,319 | 0 | 0 → 0 | 0 → 59 | 881,095 / `$08:C606` |
| 884,320 | 0 | 0 → 0 | 0 → 6 | 881,095 / `$08:C606` |
| 884,321 | 0 | 0 → 0 | 0 → 25 | 881,095 / `$08:C606` |
| 884,322 | 59 | 59 → 59 | 59 → 1 | 881,095 / `$08:C606` |
| 884,323 | 6 | 6 → 6 | 6 → 1 | 881,095 / `$08:C606` |
| 884,324 | 25 | 25 → 25 | 25 → 1 | 881,095 / `$08:C606` |
| 1,064,798 | 1 | 1 → 1 | 0 → 1 | 1,064,096 / `$08:C606` |
| 1,064,799 | 0 | 0 → 0 | 1 → 0 | 1,064,096 / `$08:C606` |
| 1,064,808 | 5 | 4 → 4 | 0 → 4 | 1,064,096 / `$08:C606` |
| 1,064,809 | 0 | 0 → 0 | 4 → 0 | 1,064,096 / `$08:C606` |
| 1,065,281 | 8 | 7 → 7 | 0 → 7 | 1,064,096 / `$08:C606` |
| 1,065,282 | 0 | 0 → 0 | 7 → 0 | 1,064,096 / `$08:C606` |
| 1,066,543 | 0 | 0 → 0 | 7 → 0 | 1,064,096 / `$08:C606` |
| 1,066,544 | 7 | 7 → 7 | 0 → 7 | 1,064,096 / `$08:C606` |
| 1,066,958 | 59 | 59 → 59 | 0 → 59 | 1,064,096 / `$08:C606` |
| 1,066,959 | 6 | 6 → 6 | 59 → 6 | 1,064,096 / `$08:C606` |
| 1,066,960 | 24 | 24 → 24 | 6 → 24 | 1,064,096 / `$08:C606` |
| 1,066,961 | 1 | 1 → 1 | 24 → 1 | 1,064,096 / `$08:C606` |


### Verification boundary

**Admission passes the specified seeded logical gates.** The 16 new ROM
fixture tests pass, together with the filtered sprite-draw (258), player
(62), sprite (129), overworld (103), graphics (40), guard (13), prep (39)
and Mothula (7) groups. Generator tests pass (8). Production builds are
warning-free; library builds retain only the same pre-existing warning
headers as main. `test-verification.json` records the final checks and
supersedes the retained failed development attempts.

After runtime and fixture changes were frozen, the exact full default
library command ran **once**:

```sh
ZELDA3_ASSETS=/Users/missingno/Documents/zelda3-rs/zelda3_assets.dat cargo test -q --profile parity -p zelda3 -p snes --lib
```

SNES: **505 passed / 7 ignored**. Zelda: **3,041 passed / 3 ignored / one
permitted failure**, exactly
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There are no other failures and no full-suite rerun.

Final hashes preserve all **36 reused prior artifacts**, **14 authority
files** and **931 frozen source/fixture files**. The final binary is
`zelda3-admission`; its source and fixtures are unchanged across the final
ROM, CPU-picture, live and full-library admission runs. All launched
processes have completed, with no background process left.

No P2, classifier, matcher, polling price or environment-switch change is
included. There is no measured-deficit price, per-scene pricing, injected
hold or interpreter fallback. Every Zelda binary invocation is CPU-only
with `--logical-replay`; no GPU, `--video`, cached-A/V run or external
Snes9x process occurs. The ledger, oracle cache and gate helpers remain
read-only. No commit, push or `cargo fmt` occurs.

Only rebuildable files in this worktree's `target/` were removed. Earlier
candidate slice CSVs were losslessly compressed and checked before their
raw copies were removed; both cache cleanup lists are retained. The target
size is recorded in `resources.json` and remains below **8,000,000,000
bytes**. `verification.json`, final identity checks, `runtime.patch` and
`artifact-hashes.json` retain the result and its evidence identities.

These are seeded logical, CPU-picture and source-instruction proofs. Full
upload arrival/receiver/return ownership, actual spotlight acceptance phase
and complete unpriced instruction suffixes remain unresolved as listed
above. Cold exact A/V acceptance is a separate boundary.

## Lagless prefix hunt 25

Instruction pricing on `codex/prefix-hunt25`, based on main **`eb7cddcc`**.
Evidence is in `target/prefix-hunt25/`. The immutable admission binary is
`zelda3-admission`, SHA-256
`a7c88195805fbf26c3743d7413de7d38cec965ab80f2abcfd811ebf7ab959839`.
The final ROM lane restores **320,086,182 instruction master clocks on
205,510 iterations** and recovers **38 missing holds**, with no lost hold
or decreased main-instruction budget. Of those recoveries, **15** are in
141,568–228,499, **20** in 228,499–380,243 and **three** later in the route.

### Fresh classification and owner ranking

The regenerated ledger has **1,473,123 complete iterations**. Its stream,
iteration, seed, ROM, SRAM and oracle hashes agree with the verified prior
authorities. The pristine native baseline was regenerated through 380,243;
all 228,499 earlier predictor rows agree byte-for-byte with main's prior
admitted prefix. Hash-verified current source receipts cover the 90 earlier
missing rows. A new cold unmodified-ROM CPU pass completes **381,401
iterations / 414,000 host calls**, producing 171 selected complete profiles
and covering **all 124** new-cohort missing rows. No new-cohort classification
is inherited from an old run. Aborted and superseded development attempts
are retained separately and are not admission evidence.

Class (a) is remaining instruction ownership capable of crossing the
current frozen diagnostic boundary; (c) has an executed upload-protocol
owner; (d) retains extra source spotlight polling or a complete signed
nonpoll instruction remainder smaller than that boundary. No independent
inherited-arrival proof establishes class (b) in this batch. These are
evidence classifications; no runtime classifier is changed.

| Cohort | (a) before → after | (b) | (c) before → after | (d) before → after | Recovered |
| --- | ---: | ---: | ---: | ---: | ---: |
| Before 141,568 | 0 → 0 | 0 | 5 → 5 | 2 → 2 | 0 |
| 141,568–228,499 | 65 → 52 | 0 | 3 → 3 | 15 → 13 | 15 |
| 228,499–380,243 | 109 → 89 | 0 | 4 → 4 | 11 → 11 | 20 |

All **214** baseline missing rows have before/after source, routine,
sprite and object partitions in `classification.json`,
`raw-owner-residuals.csv`, `sprite-owner-residuals.csv` and
`raw-object-residuals.csv`. Main instruction totals already include VWF.
Every signed child and wrapper is retained; inclusive totals and raw
positive aliases are never added to produce a price.

The fresh new-cohort ranking below uses each complete signed sprite-family
owner, including all slots and child scopes. A frozen-boundary closure is
a prioritization diagnostic, not a price or an additive recovery forecast.

| New-cohort owner | Instruction rows | Frozen-boundary closures | Signed clocks |
| --- | ---: | ---: | ---: |
| Helmasaur King $92/$09 | 24 | 15 | 405,594 |
| red spear guard $48/$09 | 16 | 14 | 228,260 |
| sprite 62 master sword $62/$09 | 5 | 5 | 73,844 |
| sprite 45 hog spear man $45/$09 | 7 | 4 | 13,036 |
| sprite $26 $26/$09 | 6 | 3 | 27,648 |
| sprite 5 a lost woods squirrel $5a/$09 | 5 | 3 | 4,490 |
| sprite $79 $79/$09 | 3 | 3 | 1,812 |
| sprite $ec $ec/$09 | 6 | 2 | 6,420 |
| sprite 59 lost woods bird $59/$09 | 5 | 2 | 3,010 |
| sprite 15 antifairy $15/$09 | 5 | 2 | 1,908 |

The top two families are priced through their actual draw/caller scopes.
The guard scopes recover **13** new-cohort rows (12 red-guard rows and one
shared `$44/$09` row); Helmasaur draw restores **six**. The earlier owner batch
also recovers 242,077, making **20** total new-cohort recoveries. These
are actual final holds. They do not claim complete Helmasaur AI or mask
coverage. After this batch, Helmasaur still ranks first with 18 remaining
instruction rows / nine frozen-boundary opportunities / 125,714 signed
clocks, followed by Master Sword (five / five / 73,844), hog spear man
(seven / four / 13,036), Hardhat Beetle (five / three / 24,576) and the
lost-woods squirrel (five / three / 4,490). `owner-ranking.json` retains
both complete rankings.

### Owners priced and ROM fixtures

Every new price comes from decoded ROM blocks, actual branch/index
outcomes and routine scopes. The fixtures execute unmodified ROM, select
only the named owner's PCs and exclude called children. The **18** new
fixtures cover both ends of the slot range, clipping and real table
crossings; the thrown-interaction fixture checks both outcomes of all 22
owned conditional branches. Keese, guard, Zora and landing fixtures replay
the actual ROM random results when a child requires them.

| Owner / actual ROM entries | Fixture names |
| --- | --- |
| Keese caller and actual draw/inactive/recoil/facing entries: `05aa8b`, `06dbf0`, `06dc10`, `05f94e`, `05f971` | `prefix25_keese_caller_all_paths_match_rom` |
| Hardhat and firebar preparation: `069122`, `0691b4`, `068ba7` | `prefix25_beetle_and_firebar_preparation_match_rom` |
| Firebar caller and correction wrapper: `1ed01a`, `06feb4` | `prefix25_firebar_correction_wrapper_matches_rom`<br>`prefix25_firebar_caller_all_paths_match_rom` |
| Bumper draw and shared JMP/fallthrough tail: `1eaa8b`, `1e87f0` | `prefix25_bumper_draw_and_trampoline_match_rom` |
| Fake sword bank wrapper, caller and draw: `05eea7`, `05eeaf`, `05eee6` | `prefix25_fake_sword_caller_and_draw_match_rom` |
| Thrown prefix, complete landing branch owner, and peer suffix: `06dff2`, `06e02a`, `06e07a`, `06e149` | `prefix25_thrown_interaction_prefix_and_peer_suffix_match_rom` |
| Water terminal helpers, shared flags return and transmute guard: `06e0ab`, `06e0f6`, `06e095`, `06e22f`, `06e26d` | `prefix25_water_terminal_helpers_and_transmute_guard_match_rom` |
| Zora walking, thrown, shadow and animation caller: `059dd6`, `059e66`, `059edb`, `05bd52` | `prefix25_walking_zora_walking_all_paths_match_rom`<br>`prefix25_walking_zora_thrown_all_paths_match_rom`<br>`prefix25_zora_shadow_suffix_all_paths_match_rom` |
| Zora speed wrapper and body excluding project-speed child: `06ea12`, `06ea04` | `prefix25_zora_speed_wrapper_and_child_match_rom` |
| Falling palette-filter JMP tail owned by actual dispatcher frame: `028e27:02a2a0`, `028e27:02a2a4` | `prefix25_falling_palette_tail_matches_all_rom_filter_paths` |
| Helmasaur draw caller, eyes, body, legs, mouth and correction entry: `1e853b`, `1e856b`, `1e87e5`, `1e8805`, `1e88bc`, `06feb4` | `prefix25_helmasaur_draw_eyes_body_legs_mouth_and_correction_match_rom` |
| Guard head, body and two-tile spear writers: `05b160`, `05b3cd`, `05cd54` | `prefix25_guard_head_and_body_all_graphics_clips_and_crossings_match_rom`<br>`prefix25_guard_spear_directions_types_clips_and_crossings_match_rom` |
| Javelin draw caller, fresh spear entry and far shadow entry: `05d192`, `05cd48`, `05c68c`, `06dc5c` | `prefix25_javelin_draw_fresh_spear_and_shadow_entries_match_rom` |
| Red javelin caller and shared dispatch prefix: `05cde1`, `05ce1b` | `prefix25_red_javelin_caller_and_shared_prefix_all_paths_match_rom` |

Keese's `$06:DC10` draw continuation already retires its instructions;
this batch selects the actual `$06:DBF0` bank entry and prices the caller,
inactive/recoil guards and actual facing entries without charging that
draw body again. The ROM's second facing call is executed at its real
call site. The existing `prefix11_recoil_entry_guards_match_zero_and_nonzero_paths`
fixture also passes for the selected recoil guards.

The bumper JMP tail, Helmasaur body fallthrough and mask JSR all reach
`$1E:87F0`; each real invocation owns that shared tail once. Fake sword
uses its real `$05:EEA7` bank wrapper, caller and draw entry. Thrown
interaction has separate prefix, landing, peer suffix and terminal-child
owners. For example, the apparent `$06:E02A` +1,146 raw residual at
181,567 cancels the new `$06:E07A` -878 and `$06:E149` -268 aliases;
it is not another 1,146 clocks to charge.

Zora walking/thrown/shadow, speed and animation entries now own their
actual instructions. Hardhat and firebar preparation and firebar's caller
and correction wrapper are priced separately. The falling transition's
`$02:A2A0` JSL / `$02:A2A4` RTS tail belongs to the real `$02:8E27`
dispatch frame: **104** clocks recover 219,129. Its previous signed
residual was only 82 because of the unchanged -22 main-return convention.

The following are disjoint native instruction deltas on the **214 selected
baseline missing rows**, grouped across all child scopes and slots. These
clocks are not the full-route 320,086,182 total.

| Sprite family / state | Earlier cohort clocks | New cohort clocks |
| --- | ---: | ---: |
| sprite $26 `$26/$08` | 2,772 | 0 |
| sprite $26 `$26/$09` | 96 | 3,072 |
| sprite 44 bluesain bolt `$44/$09` | 0 | 15,536 |
| red spear guard `$48/$08` | 0 | 18,782 |
| red spear guard `$48/$09` | 17,892 | 216,030 |
| sprite 4 a bomb guard `$4a/$09` | 0 | 1,380 |
| walking Zora `$56/$09` | 23,048 | 0 |
| Keese `$6f/$09` | 26,112 | 6,144 |
| sprite $7e `$7e/$08` | 170 | 0 |
| sprite $7e `$7e/$09` | 2,440 | 0 |
| Helmasaur King `$92/$09` | 0 | 238,454 |
| bumper `$93/$09` | 12,360 | 2,472 |
| fake sword `$e8/$09` | 12,760 | 530 |
| sprite $ec `$ec/$0a` | 0 | 1,420 |
| Falling dispatcher tail | 104 | 0 |
| **Total** | **97,754** | **503,820** |

The selected-row total is **601,574**. All 214 main-budget deltas equal
the routine-total and expanded block-marker deltas, independently per row;
the remaining scalar delta is zero in every row. Across **9,413 changed
per-PC rows**, count and clock deltas are nonnegative, and neither the
added work nor affected after-marker totals exceed the source. **178**
rows retain partial source coverage from other callers; they are kept in
`per-pc-conservation.json/csv`, not described as complete-PC coverage.
`instruction-conservation.json` retains every iteration's equality.

The generator adds **199** blocks (5,571 → 5,770). All but one old block
are unchanged. `$06:E25A`'s old 250 static clocks are split into 208
plus the separately owned 42-clock `$06:E26D` RTS; its indexed crossing
is preserved and the existing damage caller emits both exactly once.
`generated-block-preservation.json` records this source split.

The signed audit also retains an untouched allocator debt. Four fake-sword
rows (181,567 / 181,568 / 181,569 / 181,575) have a **-126-clock**
family remainder: a +104 signed `$06:84E2` caller-scope residual and an
unchanged -230 `$06:EAD1` right-of-Link predicate difference. The OAM
defer allocator still calls the already marked predicate and retains its
older raw charge: source reports one call / 230 clocks, and both baseline
and candidate report two / 460. Neither that old child charge nor the
caller remainder is changed here. `fake-sword-signed-remainder.json`
keeps the complete vector. The new-marker proof does not certify untouched
raw prices, and a raw positive caller alias is not promoted into a price.

### Remaining rows and their blockers

All **179** remaining ROM rows through 380,243 are listed below. The
nonpoll remainder excludes the separately executed source upload protocol
and additional source spotlight polls. Signed family and object owners
include negative offsets; the other caller/child column is the remaining
net, not a sum of positive routine aliases. Upload and the seven -22 rows
are reported as mechanisms only. `remaining-blockers.csv/json` and
`remaining-sprite-residuals.csv` retain the full per-row evidence.

| Iteration | Class | Diagnostic clocks short | Signed nonpoll gap | Remaining owner / mechanism |
| --- | --- | ---: | ---: | --- |
| 34,318 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 48,556 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 95,127 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 101,031 | d | 160,288 | 14,600 | Spotlight placement/actual wait: 1,429 source V-counter reads and 254,184 extra source poll-instruction clocks; remaining nonpoll owners: dash item $3b/$09 +2,588; other signed caller/child remainder +12,012 |
| 109,000 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 109,016 | d | 151,160 | 2,302 | Spotlight placement/actual wait: 1,428 source V-counter reads and 254,006 extra source poll-instruction clocks; remaining nonpoll owners: big fairy $c8/$09 +1,600; fairy $72/$09 +724; other signed caller/child remainder -22 |
| 129,746 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 146,148 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 146,786 | d | — | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,790 | d | — | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,794 | d | — | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,798 | d | — | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,802 | d | — | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,806 | d | — | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,810 | d | — | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 150,378 | a | 22,484 | 50,152 | Remaining instruction owners: sprite $ec $ec/$06 +10,178; Buzzblob $0d/$09 +776; other signed caller/child remainder +39,198 |
| 154,637 | a | 1,334 | 5,674 | Remaining instruction owners: sprite $55 $55/$09 +2,004; sprite $eb $eb/$09 +1,470; sprite $52 $52/$09 +902; other signed caller/child remainder +1,298 |
| 158,519 | a | 1,178 | 3,850 | Remaining instruction owners: sprite $eb $eb/$09 +1,470; sprite $55 $55/$09 +1,076; other signed caller/child remainder +1,304 |
| 158,558 | a | 2,752 | 3,556 | Remaining instruction owners: sprite $eb $eb/$09 +1,470; sprite $55 $55/$09 +782; other signed caller/child remainder +1,304 |
| 158,729 | a | 300 | 658 | Remaining instruction owners: sprite $eb $eb/$09 +456; sprite $55 $55/$09 +218; other signed caller/child remainder -16 |
| 160,109 | a | 720 | 918 | Remaining instruction owners: sprite $37 $37/$09 +848; sprite $58 $58/$09 +92; other signed caller/child remainder -22 |
| 160,154 | a | 906 | 1,428 | Remaining instruction owners: sprite $55 $55/$09 +500; other signed caller/child remainder +928 |
| 161,123 | a | 6,540 | 7,132 | Remaining instruction owners: sprite $55 $55/$09 +3,544; sprite $eb $eb/$09 +1,672; other signed caller/child remainder +1,916 |
| 162,301 | a | 1,548 | 7,376 | Remaining instruction owners: sprite $52 $52/$09 +3,636; sprite $eb $eb/$09 +456; other signed caller/child remainder +3,284 |
| 162,819 | a | 6,686 | 7,110 | Remaining instruction owners: sprite $52 $52/$09 +3,636; sprite $eb $eb/$09 +456; other signed caller/child remainder +3,018 |
| 168,346 | d | 83,650 | 8,290 | Arrival/acceptance or actual-wait owner unresolved: the complete signed nonpoll instruction remainder is below the current frozen boundary; remaining signed owners: sprite $55 $55/$09 +500; other signed caller/child remainder +7,790 |
| 171,309 | d | 21,222 | 8,060 | Arrival/acceptance or actual-wait owner unresolved: the complete signed nonpoll instruction remainder is below the current frozen boundary; remaining signed owners: sprite $2b $2b/$09 +1,454; other signed caller/child remainder +6,606 |
| 171,310 | d | 63,222 | 1,432 | Arrival/acceptance or actual-wait owner unresolved: the complete signed nonpoll instruction remainder is below the current frozen boundary; remaining signed owners: sprite $2b $2b/$09 +1,454; other signed caller/child remainder -22 |
| 172,380 | d | 143,228 | -116 | Arrival/acceptance or actual-wait owner unresolved: the complete signed nonpoll instruction remainder is below the current frozen boundary; remaining signed owners: blue archer $46/$09 -94; other signed caller/child remainder -22 |
| 174,033 | a | 665 | 2,210 | Remaining instruction owners: sprite $d4 $d4/$09 +888; sprite $d9 $d9/$09 +526; other signed caller/child remainder +796 |
| 174,034 | a | 1,185 | 2,498 | Remaining instruction owners: sprite $d4 $d4/$09 +888; sprite $d9 $d9/$09 +526; other signed caller/child remainder +1,084 |
| 174,280 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 174,296 | d | 152,714 | 2,302 | Spotlight placement/actual wait: 1,427 source V-counter reads and 253,828 extra source poll-instruction clocks; remaining nonpoll owners: big fairy $c8/$09 +1,600; fairy $72/$09 +724; other signed caller/child remainder -22 |
| 180,178 | a | 15,954 | 20,668 | Remaining instruction owners: red spear guard $48/$09 +7,920; sprite $ec $ec/$06 +1,428; sprite $c5 $c5/$09 +208; other signed caller/child remainder +11,112 |
| 180,182 | a | 9,518 | 15,650 | Remaining instruction owners: sprite $ec $ec/$06 +1,152; red spear guard $48/$09 +816; sprite $c5 $c5/$09 +208; other signed caller/child remainder +13,474 |
| 181,568 | a | 494 | 2,462 | Remaining instruction owners: sprite $c4 $c4/$09 +920; sprite $d9 $d9/$09 +600; sprite $dc $dc/$09 +600; sprite $e1 $e1/$09 +300; fake sword $e8/$09 -126; other signed caller/child remainder +168 |
| 186,606 | a | 1,348 | 2,166 | Remaining instruction owners: sprite $17 $17/$09 +892; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,608 | a | 1,936 | 2,166 | Remaining instruction owners: sprite $17 $17/$09 +892; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,610 | a | 1,104 | 2,166 | Remaining instruction owners: sprite $17 $17/$09 +892; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,614 | a | 1,498 | 2,166 | Remaining instruction owners: sprite $17 $17/$09 +892; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,616 | a | 2,048 | 2,166 | Remaining instruction owners: sprite $17 $17/$09 +892; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,618 | a | 1,156 | 2,166 | Remaining instruction owners: sprite $17 $17/$09 +892; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,620 | a | 1,594 | 2,166 | Remaining instruction owners: sprite $17 $17/$09 +892; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,622 | a | 1,218 | 2,166 | Remaining instruction owners: sprite $17 $17/$09 +892; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,624 | a | 1,812 | 2,166 | Remaining instruction owners: sprite $17 $17/$09 +892; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,626 | a | 1,280 | 2,166 | Remaining instruction owners: sprite $17 $17/$09 +892; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,664 | a | 2,120 | 4,004 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,666 | a | 1,090 | 4,004 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,668 | a | 2,070 | 4,004 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,670 | a | 1,588 | 4,004 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,672 | a | 1,998 | 4,004 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,684 | a | 3,108 | 4,004 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,686 | a | 2,996 | 4,004 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,690 | a | 2,212 | 4,004 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,694 | a | 2,926 | 4,004 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,696 | a | 1,302 | 4,004 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 186,698 | a | 2,340 | 4,004 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; Buzzblob $0d/$09 +582; raven $00/$09 +380; sprite $ac $ac/$09 +144; other signed caller/child remainder +168 |
| 187,779 | a | 3,170 | 3,360 | Remaining instruction owners: other signed caller/child remainder +3,360 |
| 189,344 | a | 5,586 | 6,014 | Remaining instruction owners: sprite $c2 $c2/$09 +1,192; sprite $27 $27/$09 +192; other signed caller/child remainder +4,630 |
| 189,368 | a | 3,306 | 4,630 | Remaining instruction owners: sprite $c2 $c2/$09 +1,192; sprite $ad $ad/$09 +610; other signed caller/child remainder +2,828 |
| 189,923 | d | 224,566 | 1,170 | Spotlight placement/actual wait: 1,423 source V-counter reads and 253,116 extra source poll-instruction clocks; remaining nonpoll owners: sprite $c2 $c2/$09 +1,192; other signed caller/child remainder -22 |
| 190,815 | a | 1,836 | 2,220 | Remaining instruction owners: sprite $c2 $c2/$09 +894; sprite $27 $27/$09 +192; other signed caller/child remainder +1,134 |
| 194,151 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 194,994 | a | 2,356 | 5,222 | Remaining instruction owners: sprite $87 $87/$09 +3,024; Mini Moldorm $18/$09 +230; sprite $86 $86/$09 +216; crystal switch $1e/$09 -1,152; other signed caller/child remainder +2,904 |
| 196,051 | a | 7,518 | 8,090 | Remaining instruction owners: other signed caller/child remainder +8,090 |
| 198,239 | a | 540 | 3,990 | Remaining instruction owners: sprite $a7 $a7/$09 +1,760; Mini Moldorm $18/$09 +882; sprite $e4 $e4/$09 +248; other signed caller/child remainder +1,100 |
| 198,240 | a | 124 | 3,872 | Remaining instruction owners: sprite $a7 $a7/$09 +1,760; Mini Moldorm $18/$09 +882; sprite $e4 $e4/$09 +248; other signed caller/child remainder +982 |
| 209,808 | a | 4,612 | 14,628 | Remaining instruction owners: crystal switch $1e/$09 -476; other signed caller/child remainder +15,104 |
| 215,857 | a | 714 | 982 | Remaining instruction owners: other signed caller/child remainder +982 |
| 215,858 | a | 716 | 982 | Remaining instruction owners: other signed caller/child remainder +982 |
| 215,859 | a | 714 | 982 | Remaining instruction owners: other signed caller/child remainder +982 |
| 217,220 | a | 714 | 1,100 | Remaining instruction owners: other signed caller/child remainder +1,100 |
| 217,221 | a | 298 | 982 | Remaining instruction owners: other signed caller/child remainder +982 |
| 217,222 | a | 714 | 982 | Remaining instruction owners: other signed caller/child remainder +982 |
| 225,257 | a | 3,242 | 4,964 | Remaining instruction owners: dungeon object $232 +352; dungeon object $239 +854; other signed caller/child remainder +3,758 |
| 239,545 | a | 12 | 368 | Remaining instruction owners: sprite_kakariko_boy $3c/$09 +68; sprite_running_boy $74/$09 +68; sprite_sweeping_lady $2a/$09 +64; other signed caller/child remainder +168 |
| 242,100 | d | 46,004 | 1,478 | Arrival/acceptance or actual-wait owner unresolved: the complete signed nonpoll instruction remainder is below the current frozen boundary; remaining signed owners: sprite_5_a_lost_woods_squirrel $5a/$09 +898; sprite_59_lost_woods_bird $59/$09 +602; other signed caller/child remainder -22 |
| 242,103 | a | 1,976 | 2,428 | Remaining instruction owners: sprite_5_a_lost_woods_squirrel $5a/$09 +898; sprite_59_lost_woods_bird $59/$09 +602; other signed caller/child remainder +928 |
| 242,118 | a | 762 | 2,532 | Remaining instruction owners: sprite_5_a_lost_woods_squirrel $5a/$09 +898; sprite_59_lost_woods_bird $59/$09 +602; other signed caller/child remainder +1,032 |
| 242,122 | a | 2,228 | 2,532 | Remaining instruction owners: sprite_5_a_lost_woods_squirrel $5a/$09 +898; sprite_59_lost_woods_bird $59/$09 +602; other signed caller/child remainder +1,032 |
| 242,124 | a | 418 | 2,532 | Remaining instruction owners: sprite_5_a_lost_woods_squirrel $5a/$09 +898; sprite_59_lost_woods_bird $59/$09 +602; other signed caller/child remainder +1,032 |
| 242,906 | a | 4,864 | 20,360 | Remaining instruction owners: sprite_62_master_sword $62/$09 +14,888; other signed caller/child remainder +5,472 |
| 242,934 | a | 9,768 | 15,800 | Remaining instruction owners: sprite_62_master_sword $62/$09 +14,192; other signed caller/child remainder +1,608 |
| 242,938 | a | 9,116 | 19,730 | Remaining instruction owners: sprite_62_master_sword $62/$09 +14,192; other signed caller/child remainder +5,538 |
| 242,940 | a | 412 | 17,988 | Remaining instruction owners: sprite_62_master_sword $62/$09 +16,380; other signed caller/child remainder +1,608 |
| 242,942 | a | 9,986 | 21,078 | Remaining instruction owners: sprite_62_master_sword $62/$09 +14,192; other signed caller/child remainder +6,886 |
| 244,486 | a | 436 | 1,478 | Remaining instruction owners: sprite_5_a_lost_woods_squirrel $5a/$09 +898; sprite_59_lost_woods_bird $59/$09 +602; other signed caller/child remainder -22 |
| 253,040 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 254,154 | d | 228,700 | 434 | Spotlight placement/actual wait: 1,428 source V-counter reads and 254,006 extra source poll-instruction clocks; remaining nonpoll owners: sprite $eb $eb/$09 +456; other signed caller/child remainder -22 |
| 255,517 | a | 1,000 | 2,770 | Remaining instruction owners: red spear guard $48/$09 +1,646; other signed caller/child remainder +1,124 |
| 255,521 | a | 1,608 | 2,968 | Remaining instruction owners: red spear guard $48/$09 +1,646; other signed caller/child remainder +1,322 |
| 255,525 | a | 1,294 | 2,968 | Remaining instruction owners: red spear guard $48/$09 +1,646; other signed caller/child remainder +1,322 |
| 255,529 | a | 1,526 | 2,266 | Remaining instruction owners: red spear guard $48/$09 +944; other signed caller/child remainder +1,322 |
| 255,572 | a | 1,022 | 2,556 | Remaining instruction owners: sprite $33 $33/$09 +1,234; other signed caller/child remainder +1,322 |
| 256,490 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 256,786 | a | 7,284 | 43,632 | Remaining instruction owners: dungeon object $12f +4,514; dungeon object $130 +4,514; dungeon object $170 +2,020; dungeon object $171 +12,940; dungeon object $214 +2,518; dungeon object $215 +2,518; dungeon object $216 +2,518; dungeon object $217 +2,518; dungeon object $220 +124; other signed caller/child remainder +9,448 |
| 281,028 | a | 6,862 | 8,362 | Remaining instruction owners: sprite_1_c_statue $1c/$09 +460; blue archer $46/$09 -188; other signed caller/child remainder +8,090 |
| 282,475 | a | 160 | 740 | Remaining instruction owners: green guard $41/$05 +718; sprite_1_c_statue $1c/$09 +460; other signed caller/child remainder -438 |
| 282,497 | a | 8,584 | 47,980 | Remaining instruction owners: green guard $41/$05 +718; sprite_1_c_statue $1c/$09 +460; dungeon object $170 +1,880; dungeon object $171 +21,488; dungeon object $321 +2,538; dungeon object $32d +17,816; other signed caller/child remainder +3,080 |
| 294,833 | a | 9,914 | 13,668 | Remaining instruction owners: sprite_4_a_bomb_guard $4a/$06 +7,492; sprite_11_hinox $11/$09 +2,634; sprite_22_ropa $22/$09 +1,918; other signed caller/child remainder +1,624 |
| 294,835 | a | 10,642 | 18,236 | Remaining instruction owners: sprite_4_a_bomb_guard $4a/$06 +7,594; green guard $41/$09 +2,728; sprite_11_hinox $11/$09 +2,634; sprite_22_ropa $22/$09 +1,918; other signed caller/child remainder +3,362 |
| 294,837 | a | 12,492 | 15,022 | Remaining instruction owners: sprite_4_a_bomb_guard $4a/$06 +6,420; green guard $41/$09 +2,728; sprite_11_hinox $11/$09 +2,634; sprite_22_ropa $22/$09 +1,918; other signed caller/child remainder +1,322 |
| 294,844 | d | 7,862 | 7,802 | Arrival/acceptance or actual-wait owner unresolved: the complete signed nonpoll instruction remainder is below the current frozen boundary; remaining signed owners: sprite_4_a_bomb_guard $4a/$06 +7,372; sprite_11_hinox $11/$09 +452; other signed caller/child remainder -22 |
| 295,089 | d | 147,092 | 2,302 | Spotlight placement/actual wait: 1,427 source V-counter reads and 253,828 extra source poll-instruction clocks; remaining nonpoll owners: big fairy $c8/$09 +1,600; fairy $72/$09 +724; other signed caller/child remainder -22 |
| 297,579 | a | 860 | 6,040 | Remaining instruction owners: sprite $eb $eb/$09 +1,470; sprite_45_hog_spear_man $45/$09 +1,296; sprite $55 $55/$09 +676; Buzzblob $0d/$09 +194; other signed caller/child remainder +2,404 |
| 297,585 | a | 9,296 | 10,228 | Remaining instruction owners: sprite $55 $55/$09 +3,920; sprite_45_hog_spear_man $45/$09 +2,240; sprite $eb $eb/$09 +1,470; Buzzblob $0d/$09 +194; other signed caller/child remainder +2,404 |
| 297,638 | a | 2,782 | 8,046 | Remaining instruction owners: sprite $55 $55/$09 +1,776; sprite $eb $eb/$09 +1,470; sprite_45_hog_spear_man $45/$09 +1,296; Buzzblob $0d/$09 +194; other signed caller/child remainder +3,310 |
| 297,863 | d | 129,956 | 4,942 | Spotlight placement/actual wait: 1,426 source V-counter reads and 253,650 extra source poll-instruction clocks; remaining nonpoll owners: sprite_bb_shopkeeper $bb/$09 +5,352; sprite_28_dark_world_hint_npc $28/$09 -388; other signed caller/child remainder -22 |
| 299,434 | d | 147,848 | 2,302 | Spotlight placement/actual wait: 1,427 source V-counter reads and 253,828 extra source poll-instruction clocks; remaining nonpoll owners: big fairy $c8/$09 +1,600; fairy $72/$09 +724; other signed caller/child remainder -22 |
| 299,510 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 304,890 | a | 11,756 | 15,634 | Remaining instruction owners: sprite_0_f_octoballoon $0f/$06 +11,456; sprite_10_octoballoon_baby $10/$09 +4,010; other signed caller/child remainder +168 |
| 307,022 | d | 234,416 | 2,666 | Spotlight placement/actual wait: 1,405 source V-counter reads and 249,912 extra source poll-instruction clocks; remaining nonpoll owners: sprite $58 $58/$09 +2,582; other signed caller/child remainder +84 |
| 307,040 | c | — | — | Upload caller, protocol receiver/acknowledgement and return/suffix arrival ownership remain unpositioned; no upload or acceptance-timing change. |
| 310,875 | a | 10,058 | 20,736 | Remaining instruction owners: sprite_0_f_octoballoon $0f/$06 +11,518; sprite_10_octoballoon_baby $10/$09 +4,010; sprite $55 $55/$09 +676; other signed caller/child remainder +4,532 |
| 310,980 | d | 171,138 | 7,790 | Arrival/acceptance or actual-wait owner unresolved: the complete signed nonpoll instruction remainder is below the current frozen boundary; remaining signed owners: other signed caller/child remainder +7,790 |
| 310,982 | d | 84,542 | 710 | Arrival/acceptance or actual-wait owner unresolved: the complete signed nonpoll instruction remainder is below the current frozen boundary; remaining signed owners: other signed caller/child remainder +710 |
| 313,078 | a | 32,062 | 32,694 | Remaining instruction owners: sprite_19_poe $19/$09 +8,560; green guard $41/$09 +716; other signed caller/child remainder +23,418 |
| 313,255 | a | 35,952 | 36,056 | Remaining instruction owners: sprite_19_poe $19/$09 +9,850; green guard $41/$09 +2,704; other signed caller/child remainder +23,502 |
| 314,323 | a | 574 | 3,098 | Remaining instruction owners: sprite_45_hog_spear_man $45/$09 +2,704; sprite $79 $79/$09 +604; blue archer $46/$09 -188; other signed caller/child remainder -22 |
| 314,327 | a | 474 | 3,098 | Remaining instruction owners: sprite_45_hog_spear_man $45/$09 +2,704; sprite $79 $79/$09 +604; blue archer $46/$09 -188; other signed caller/child remainder -22 |
| 314,335 | a | 14 | 3,098 | Remaining instruction owners: sprite_45_hog_spear_man $45/$09 +2,704; sprite $79 $79/$09 +604; blue archer $46/$09 -188; other signed caller/child remainder -22 |
| 317,588 | a | 3,970 | 8,098 | Remaining instruction owners: sprite_45_hog_spear_man $45/$09 +92; other signed caller/child remainder +8,006 |
| 322,193 | a | 10,344 | 11,236 | Remaining instruction owners: sprite_22_ropa $22/$09 +2,170; sprite_11_hinox $11/$09 +1,510; sprite_4_a_bomb_guard $4a/$09 +1,082; sprite_12_moblin $12/$09 +976; sprite_b6_kiki $b6/$09 +864; other signed caller/child remainder +4,634 |
| 327,434 | a | 720 | 786 | Remaining instruction owners: sprite_15_antifairy $15/$09 +212; other signed caller/child remainder +574 |
| 336,024 | a | 58 | 894 | Remaining instruction owners: sprite_15_antifairy $15/$09 +424; other signed caller/child remainder +470 |
| 336,035 | a | 452 | 998 | Remaining instruction owners: sprite_15_antifairy $15/$09 +424; other signed caller/child remainder +574 |
| 336,039 | a | 424 | 998 | Remaining instruction owners: sprite_15_antifairy $15/$09 +424; other signed caller/child remainder +574 |
| 336,046 | a | 576 | 894 | Remaining instruction owners: sprite_15_antifairy $15/$09 +424; other signed caller/child remainder +470 |
| 337,919 | a | 1,192 | 17,372 | Remaining instruction owners: sprite $26 $26/$09 +4,072; crystal switch $1e/$09 +146; other signed caller/child remainder +13,154 |
| 337,925 | a | 2,338 | 5,208 | Remaining instruction owners: sprite $26 $26/$09 +1,876; crystal switch $1e/$09 +146; other signed caller/child remainder +3,186 |
| 338,240 | a | 1,312 | 6,772 | Remaining instruction owners: sprite $26 $26/$09 +2,340; crystal switch $1e/$09 +1,246; other signed caller/child remainder +3,186 |
| 338,673 | a | 9,852 | 10,696 | Remaining instruction owners: sprite $26 $26/$09 +8,144; crystal switch $1e/$09 +146; other signed caller/child remainder +2,406 |
| 344,246 | a | 4,258 | 5,390 | Remaining instruction owners: spike $8a/$08 +702; Pokey $8e/$08 +168; sprite_1_c_statue $1c/$08 +142; other signed caller/child remainder +4,378 |
| 347,334 | a | 5,338 | 10,696 | Remaining instruction owners: sprite $26 $26/$09 +8,144; crystal switch $1e/$09 +146; other signed caller/child remainder +2,406 |
| 348,203 | a | 3,532 | 5,404 | Remaining instruction owners: Pokey $8e/$09 +308; other signed caller/child remainder +5,096 |
| 353,572 | a | 3,198 | 5,276 | Remaining instruction owners: Pokey $8e/$09 +308; other signed caller/child remainder +4,968 |
| 354,592 | a | 1,510 | 1,848 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +470 |
| 354,594 | a | 48 | 1,848 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +470 |
| 354,595 | d | 1,966 | 1,952 | Arrival/acceptance or actual-wait owner unresolved: the complete signed nonpoll instruction remainder is below the current frozen boundary; remaining signed owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +574 |
| 354,604 | a | 1,226 | 1,848 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +470 |
| 354,611 | a | 344 | 1,952 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +574 |
| 354,616 | a | 1,272 | 1,848 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +470 |
| 354,619 | a | 1,692 | 1,952 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +574 |
| 356,806 | a | 2,714 | 7,966 | Remaining instruction owners: sprite $ec $ec/$0a +4,084; crystal switch $1e/$09 +1,254; sprite $d9 $d9/$09 +300; other signed caller/child remainder +2,328 |
| 367,254 | a | 1,342 | 9,646 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +394; spike $8a/$09 +156; crystal switch $1e/$09 -668; other signed caller/child remainder +8,914 |
| 367,255 | a | 2,052 | 9,646 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +394; spike $8a/$09 +156; crystal switch $1e/$09 -668; other signed caller/child remainder +8,914 |
| 367,256 | a | 5,532 | 9,646 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +394; spike $8a/$09 +156; crystal switch $1e/$09 -668; other signed caller/child remainder +8,914 |
| 367,257 | a | 3,064 | 9,646 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +394; spike $8a/$09 +156; crystal switch $1e/$09 -668; other signed caller/child remainder +8,914 |
| 367,286 | a | 30,104 | 36,046 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +394; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +34,876 |
| 367,291 | a | 20,924 | 33,378 | Remaining instruction owners: green Eyegore $83/$09 +4,782; eye statue $38/$09 +450; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +28,020 |
| 367,313 | a | 38,446 | 43,440 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +42,464 |
| 367,327 | a | 35,246 | 35,880 | Remaining instruction owners: green Eyegore $83/$09 +1,374; eye statue $38/$09 +450; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +33,930 |
| 367,337 | a | 37,994 | 43,472 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +42,496 |
| 367,347 | a | 33,266 | 44,110 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +43,134 |
| 367,361 | a | 38,924 | 44,422 | Remaining instruction owners: green Eyegore $83/$09 +1,374; eye statue $38/$09 +450; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +42,472 |
| 367,372 | a | 37,698 | 44,308 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +43,332 |
| 367,394 | a | 23,774 | 33,506 | Remaining instruction owners: green Eyegore $83/$09 +4,782; eye statue $38/$09 +450; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +28,148 |
| 370,567 | a | 2,424 | 2,658 | Remaining instruction owners: Pokey $8e/$09 +1,386; crystal switch $1e/$09 -238; other signed caller/child remainder +1,510 |
| 372,814 | a | 16,820 | 18,356 | Remaining instruction owners: Pokey $8e/$09 +14,210; Pokey $8e/$06 +154; sprite_d9_green_rupee $da/$09 +108; crystal switch $1e/$09 -238; other signed caller/child remainder +4,122 |
| 375,434 | a | 24,120 | 29,948 | Remaining instruction owners: crystal switch $1e/$09 -238; dungeon object $170 +2,830; dungeon object $171 +22,704; dungeon object $312 +768; dungeon object $32a +330; other signed caller/child remainder +3,554 |
| 378,070 | a | 3,560 | 7,998 | Remaining instruction owners: Helmasaur King $92/$09 +6,620; other signed caller/child remainder +1,378 |
| 378,071 | a | 9,192 | 10,482 | Remaining instruction owners: Helmasaur King $92/$09 +8,354; other signed caller/child remainder +2,128 |
| 378,077 | a | 6,742 | 8,258 | Remaining instruction owners: Helmasaur King $92/$09 +6,612; other signed caller/child remainder +1,646 |
| 378,078 | a | 6,290 | 7,698 | Remaining instruction owners: Helmasaur King $92/$09 +6,612; other signed caller/child remainder +1,086 |
| 378,079 | a | 4,812 | 8,688 | Remaining instruction owners: Helmasaur King $92/$09 +7,602; other signed caller/child remainder +1,086 |
| 378,080 | a | 6,698 | 7,698 | Remaining instruction owners: Helmasaur King $92/$09 +6,612; other signed caller/child remainder +1,086 |
| 378,092 | a | 7,312 | 7,834 | Remaining instruction owners: Helmasaur King $92/$09 +6,748; other signed caller/child remainder +1,086 |
| 378,093 | d | 7,874 | 7,834 | Arrival/acceptance or actual-wait owner unresolved: the complete signed nonpoll instruction remainder is below the current frozen boundary; remaining signed owners: Helmasaur King $92/$09 +6,748; other signed caller/child remainder +1,086 |
| 378,094 | a | 7,490 | 7,834 | Remaining instruction owners: Helmasaur King $92/$09 +6,748; other signed caller/child remainder +1,086 |
| 378,095 | a | 6,852 | 12,052 | Remaining instruction owners: Helmasaur King $92/$09 +7,932; other signed caller/child remainder +4,120 |
| 378,096 | a | 9,198 | 10,264 | Remaining instruction owners: Helmasaur King $92/$09 +6,778; other signed caller/child remainder +3,486 |
| 378,098 | a | 8,948 | 9,490 | Remaining instruction owners: Helmasaur King $92/$09 +6,972; other signed caller/child remainder +2,518 |
| 378,099 | a | 4,574 | 9,326 | Remaining instruction owners: Helmasaur King $92/$09 +6,808; other signed caller/child remainder +2,518 |
| 378,103 | a | 382 | 11,984 | Remaining instruction owners: Helmasaur King $92/$09 +9,618; other signed caller/child remainder +2,366 |
| 378,104 | a | 5,860 | 9,700 | Remaining instruction owners: Helmasaur King $92/$09 +7,334; other signed caller/child remainder +2,366 |
| 378,105 | a | 8,644 | 9,688 | Remaining instruction owners: Helmasaur King $92/$09 +7,170; other signed caller/child remainder +2,518 |
| 378,541 | a | 9,726 | 11,182 | Remaining instruction owners: Helmasaur King $92/$09 +7,714; other signed caller/child remainder +3,468 |
| 378,780 | a | 176 | 6,122 | Remaining instruction owners: Helmasaur King $92/$09 +2,316; sprite_70_king_helmasaur_fireball $70/$09 +2,260; other signed caller/child remainder +1,546 |
| 378,830 | a | 5,336 | 9,682 | Remaining instruction owners: Helmasaur King $92/$09 +7,164; other signed caller/child remainder +2,518 |

All twelve upload rows retain caller, receiver/acknowledgement and
return/suffix phase ownership. `upload-phase-current.json` combines current
source bus-PC and phase receipts for both cohorts. Those executed access
counts are diagnostics, never prices. The seven 146,786–146,810 rows retain
the same -22 signed main-instruction residual and zero new clocks; actual
main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved.
No upload or acceptance timing changes occur.

King Helmasaur's remaining family residual includes root/AI, mask-body
and swing work beyond the priced draw scopes. Remaining guard-family
AI and state bodies, other named sprite families and signed non-sprite
caller/child remainders need their own source-entry/return fixtures. The
complete signed scope vectors are retained to establish those owners
before any further price is admitted.

### Full ROM and CPU pictures

The specified eight-slice ROM helper passes **1,473,123 iterations /
unexplained 0**, at **1,693 → 1,655 missing**, **0 → 0 excess**,
**0 → 0 FP**. All 38 changed holds recover exactly one missing hold;
no hold is lost and no runtime hold disagrees with the actual `cost_p2`
model. The simpler `predicted` CSV column is not the P2 admission model.
`rom-admission-command.json`, `rom-admission-helper.log` and
`rom-comparison.json` retain the complete run and comparison.

| Before iteration | ROM missing before → after | Live missing before → after |
| --- | ---: | ---: |
| 94,334 | 2 → 2 | 2 → 2 |
| 141,568 | 7 → 7 | 7 → 7 |
| 228,499 | 90 → 75 | 90 → 75 |
| 380,243 | 214 → 179 | 223 → 179 |
| 461,989 | 270 → 235 | 279 → 349 |

Three recoveries are mixed instruction/wait observations, not acceptance
phase repairs. The restored ROM instructions change the existing raster
wait decision in these rows:

| Iteration | New instruction clocks | Raster wait before → after | Holds before → after |
| --- | ---: | ---: | ---: |
| 187,109 | 6,912 | 0 → 267,000 | 1 → 2 |
| 188,962 | 3,840 | 0 → 268,424 | 1 → 2 |
| 549,411 | 824 | 0 → 269,492 | 1 → 2 |

No new poll instruction or physical acceptance phase is owned by those
hold recoveries. The later recovered rows are 525,097 (+3,068 instruction
clocks), 549,411 (+824) and 775,682 (+196).
`recovered-row-mechanisms.json` retains all 38 rows.

The independent eight-slice `--cpu-video` census also passes all
**1,473,123 iterations / unexplained 0**. It retains **1,462,997 exact
pictures**, 563 early, 3,952 late, four during-lag, 905 polygon-thread,
4,702 torn and zero other content misses. All eight picture-hash files
are byte-identical to main. `cpu-gate.txt`,
`cpu-picture-agreement.json` and `cpu-admission-commands.json` record the
CPU-only proof.

### Single live census and song-end attribution

Exactly **one continuous seeded** `--live-apu-reads --pacing-predictor`
process completes all **1,473,123 iterations**. SPC state carries across
its seeds. The logical gate passes with **unexplained 0**. Live totals
**worsen**: **2,105 → 2,172 missing**, **318 → 399 excess**,
**85 → 88 FP**. `live-command.json` records the single launch and
completed exit; `live-comparison.json`, `live-counts.txt` and
`live-timing.jsonl.zst` retain the complete data.

There are **57 changed live hold rows**. **38** share the exact ROM
before/after hold pair. All other **19 live-only changes** are attributed:
**four pendant `$08:C3FD`**, **15 crystal `$08:C606`**, **zero
unattributed**. The conservative live-only set includes every changed
pair that does not exactly match a changed ROM pair.

The 228,499 pendant episode moves its first native zero from **229,036
→ 229,035**, matching the direct source zero; it contributes **-9 / -9 /
-2** missing/excess/FP. The 380,243 crystal episode moves from **380,881
→ 380,882**, one iteration late, contributing **+114 / +90 / +5**.
The net live-only delta is therefore **+105 / +81 / +3**. The **-38 /
0 / 0** shared ROM improvement makes the total live delta **+67 / +81 /
+3**. The crystal regression is retained and attributed; its timing is
not repaired in this batch.

The actual source PCs come from **20 direct APUI receipts** (nonzero and
first-zero `$2140` reads in ten episodes) in the verified prior cold
unmodified-ROM CPU source replay. `poll-source-provenance.json` records
the reuse; ROM, ledger and source authorities are unchanged. These PCs
are established directly, never inferred from sprite labels. The complete
source-context/read payload hash of the new live trace equals baseline.

Episode bounds are derived **without changed holds**: contiguous native
poll clusters overlap the directly observed source-poll intervals; actual
boolean polls, native/source contexts and four-lane main/VWF differences
locate execution divergence; actual reseeding or the containing
2,000-iteration boundary ends recovery. The four lanes remove the intended
ROM price changes. Main already includes VWF; VWF is only a separate
observation component here. `live-execution-events.json` and
`poll-attribution.json` retain the bounds and evidence.

| Episode | Direct source PC | Source first zero | Native first zero before → after | Last observed execution divergence | Recovery end | Live-only rows | Delta missing / excess / FP |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 94,334 | `$08:C3FD` | 94,854 | 94,854 → 94,854 | — | 96,000 | 0 | +0 / +0 / +0 |
| 141,568 | `$08:C3FD` | 142,088 | 142,088 → 142,088 | — | 144,000 | 0 | +0 / +0 / +0 |
| 228,499 | `$08:C3FD` | 229,035 | 229,036 → 229,035 | 229,069 | 230,000 | 4 | -9 / -9 / -2 |
| 380,243 | `$08:C606` | 380,881 | 380,881 → 380,882 | 383,635 | 383,636 | 15 | +114 / +90 / +5 |
| 461,989 | `$08:C606` | 462,628 | 462,627 → 462,627 | 463,179 | 464,000 | 0 | +0 / +0 / +0 |
| 626,618 | `$08:C606` | 627,256 | 627,257 → 627,257 | 629,461 | 629,845 | 0 | +0 / +0 / +0 |
| 735,229 | `$08:C606` | 735,867 | 735,868 → 735,868 | 736,421 | 738,000 | 0 | +0 / +0 / +0 |
| 881,095 | `$08:C606` | 881,734 | 881,733 → 881,733 | 884,325 | 884,327 | 0 | +0 / +0 / +0 |
| 1,064,096 | `$08:C606` | 1,064,734 | 1,064,734 → 1,064,734 | — | 1,066,000 | 0 | +0 / +0 / +0 |
| 1,317,697 | `$08:C606` | 1,318,335 | 1,318,336 → 1,318,336 | 1,320,899 | 1,320,900 | 0 | +0 / +0 / +0 |

Every live-only changed row is listed below. Full before/after budgets and
metric deltas are in `changed-live-holds.csv/json`;
`all-changed-live-holds.csv` also retains the 38 shared ROM changes.

| Iteration | Source lag | ROM holds before → after | Live holds before → after | Episode / direct source PC |
| --- | ---: | ---: | ---: | --- |
| 229,036 | 5 | 5 → 5 | 0 → 5 | 228,499 / `$08:C3FD` |
| 229,037 | 0 | 0 → 0 | 5 → 0 | 228,499 / `$08:C3FD` |
| 229,039 | 4 | 4 → 4 | 0 → 4 | 228,499 / `$08:C3FD` |
| 229,042 | 0 | 0 → 0 | 4 → 0 | 228,499 / `$08:C3FD` |
| 380,946 | 1 | 1 → 1 | 1 → 0 | 380,243 / `$08:C606` |
| 380,947 | 0 | 0 → 0 | 0 → 1 | 380,243 / `$08:C606` |
| 380,956 | 5 | 4 → 4 | 4 → 0 | 380,243 / `$08:C606` |
| 380,957 | 0 | 0 → 0 | 0 → 4 | 380,243 / `$08:C606` |
| 381,426 | 8 | 8 → 8 | 8 → 0 | 380,243 / `$08:C606` |
| 381,427 | 0 | 0 → 0 | 0 → 8 | 380,243 / `$08:C606` |
| 383,178 | 0 | 0 → 0 | 0 → 7 | 380,243 / `$08:C606` |
| 383,181 | 7 | 7 → 7 | 7 → 0 | 380,243 / `$08:C606` |
| 383,343 | 7 | 7 → 7 | 7 → 0 | 380,243 / `$08:C606` |
| 383,346 | 0 | 0 → 0 | 0 → 7 | 380,243 / `$08:C606` |
| 383,631 | 59 | 59 → 59 | 59 → 0 | 380,243 / `$08:C606` |
| 383,632 | 6 | 6 → 6 | 6 → 0 | 380,243 / `$08:C606` |
| 383,633 | 23 | 22 → 22 | 22 → 0 | 380,243 / `$08:C606` |
| 383,634 | 1 | 1 → 1 | 1 → 59 | 380,243 / `$08:C606` |
| 383,635 | 1 | 1 → 1 | 1 → 6 | 380,243 / `$08:C606` |

### Verification boundary

The final filtered sprite-draw (**275**), sprite (**129**), guard (**13**),
Helmasaur (**nine**), preparation (**39**) and dungeon (**62**) groups
pass, including all **18 new ROM fixtures**. Generator tests pass (**eight**).
Production builds are warning-free. Library builds retain the same
pre-existing warning headers as main. `test-verification.json` preserves
all final checks and the retained development attempts.

After runtime changes were frozen, the requested full default command
ran **once**, with `CARGO_INCREMENTAL=0` only to limit rebuildable cache size:

```sh
ZELDA3_ASSETS=/Users/missingno/Documents/zelda3-rs/zelda3_assets.dat cargo test -q --profile parity -p zelda3 -p snes --lib
```

That full run records SNES **505 passed / seven ignored**, and Zelda
**3,058 passed / two failed / three ignored**. One is the permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
The additional failure is
`zelda_rtl::sprite_main::tests::mantle_and_oam_correction_cycles_match_rom_for_clipping`:
its legacy setup constructed a non-lagless state, disabling the new
lagless far-wrapper markers while expecting their clocks.

The fixture now explicitly enables `lag_free`. Every original unmodified-ROM
clock and OAM assertion is unchanged. The **entire 18-test sprite-main
group passes** after that repair. It is the only post-freeze source edit,
in a module imported exclusively under `#[cfg(test)]`. The subsequent
production rebuild is **byte-identical** to the frozen ROM/CPU/live binary,
and all runtime source remains unchanged. `fixture-mode-repair.json`,
`mantle-fixture-command.json`, `final-build-command.json` and
`runtime-source-agreement.json` retain that proof.

**Only spotlight remains unresolved.** The original full-suite transcript
contains the additional, now repaired setup failure; it is not rewritten.
No second full-suite run is made under the once-only rule. Library coverage
is therefore one full run plus the passing fixture-group recheck, rather
than a fresh full-suite transcript after the setup repair.

The continuous selected profile and chunked ROM census agree on main/VWF
prices, source lag, native/P2 holds and classification shortfalls throughout
all **380,243** prefix rows. Fifty rows differ only in incoming-handler,
physical raster/refresh/HDMA or elapsed diagnostics. Among selected rows,
only the elapsed counter at upload rows 194,151 / 256,490 differs between
continuous and fresh-slice runs. `final-profile-agreement.json` retains
every difference. Those phase observations are not prices or upload fixes.

Final checks preserve **36 reused prior artifacts** and all **14 authority
files**. The final source manifest tracks **934 files**; 933 retain their
original runtime-freeze hashes, with only the explicitly recorded
`cfg(test)` fixture repair above. The normal production rebuild and every
final runtime census use the same binary bytes. All launched source,
native, test, build and trace-decoder workloads have completed; no
background work remains.

No runtime P2, classifier, matcher, polling-price or environment-switch
change is included. There is no measured-deficit price, per-scene table,
injected hold, acceptance compensation or interpreter fallback. All game
binary invocations use CPU-only `--logical-replay`; no GPU, `--video`,
cached-A/V run or external Snes9x process occurs. The ledger, oracle cache
and gate helpers remain read-only. No commit, push or `cargo fmt` occurs.

Only rebuildable files inside this worktree's `target/` are removed.
Superseded diagnostic CSVs are losslessly compressed and checked before
removing raw copies; cleanup and compression receipts are retained.
`resources.json` records target size below **8,000,000,000 bytes**.
`verification.json`, `runtime.patch` and `artifact-hashes.json` retain
the final state and identities.

These are seeded logical, CPU-picture and source-instruction proofs.
Upload arrival/receiver/return ownership, actual spotlight acceptance and
waiting, retained raw child/caller offsets and the complete unpriced
suffixes remain unresolved as recorded. Cold exact A/V acceptance is a
separate boundary.

## Lagless prefix hunt 26

Instruction pricing on `codex/prefix-hunt26`, based on main **`cf15ffd4`**.
Evidence is in `target/prefix-hunt26/`. The immutable admission binary is
`zelda3-admission`, SHA-256
`596d1e87477d77b691400bcb51cbd5d6c76503df81854a58f01ca82b921078a1`.

The full ROM lane restores **78,472,200 instruction master clocks on
114,323 iterations** and recovers **29 missing holds**, with no lost hold
or decreased main-instruction budget. **Two** recoveries precede 228,499,
**24** are in 228,499–380,243 and **three** are later in the route.
Every recovery retains its prior raster-wait charge. No arrival, acceptance,
spotlight-wait, upload, polling-price, P2 or matcher change is included.

### Classification and source identity

The starting worktree is clean at the requested base. All **934** retained
source hashes agree with main; the regenerated **1,473,123-iteration** ledger,
ROM, SRAM, oracle and helper authorities agree with the retained hashes.
Source artifacts are verified before final reuse. A fresh seeded native
profile completes all **380,243** prefix iterations with no unexplained
state difference. The hash-verified cold unmodified-ROM CPU receipts cover
all **214** retained profile rows; their independent per-PC sums equal
every source instruction total. No stale native classification is inherited.
`baseline-source-verification.json`, `source-profile-verification.json`,
`source-instruction-agreement.json` and `final-profile-command.json` retain
the evidence.

Class (a) retains a complete signed nonpoll instruction remainder that
can reach the current frozen diagnostic boundary. Class (c) retains an
executed upload-protocol owner. Class (d) retains extra source spotlight
polling or a signed nonpoll remainder below the boundary. No independent
inherited-arrival proof establishes class (b). These classifications are
diagnostics; runtime classification is unchanged.

| Cohort | (a) before → after | (b) | (c) before → after | (d) before → after | Recovered |
| --- | ---: | ---: | ---: | ---: | ---: |
| Before 141,568 | 0 → 0 | 0 | 5 → 5 | 2 → 2 | 0 |
| 141,568–228,499 | 52 → 50 | 0 | 3 → 3 | 13 → 13 | 2 |
| 228,499–380,243 | 89 → 66 | 0 | 4 → 4 | 11 → 10 | 24 |

There are **179** starting rows: **141 (a), 12 (c), 26 (d)**. After
pricing, **153** remain: **116 (a), 12 (c), 25 (d)**. The current missing
row set equals the canonical ROM census exactly. All rows and signed
source/native partitions are in `classification.csv/json`,
`raw-owner-residuals.csv`, `sprite-owner-residuals.csv` and
`raw-object-residuals.csv`. The CSV `total` summary is excluded from
instruction-owner vectors. Main instruction totals already include VWF. Inclusive totals and positive
aliases are never added to produce a price.

The signed starting ranking places Helmasaur first (18 instruction rows /
nine frozen-boundary opportunities), then Master Sword (five / five),
heart piece and hog spear man (four opportunities each). This batch prices
Helmasaur root/AI, mask caller and swing, Master Sword prop/dispatch/light
owners, the actual hog/bolt child entries and frequent smaller families.
The frequent Buzzblob, apple and raven remainders do not individually cross
the old boundaries; their proved combination recovers two rows. Crystal
switch retains its existing negative child remainders while its actual
F154 entry is restored. `owner-ranking-before.json` and
`owner-ranking-after.json` retain complete signed rankings.

No starting or remaining class-(d) row has a complete signed nonpoll
remainder at or above its boundary. Shared Helmasaur instruction prices
nonetheless recover **378,093** through the existing cost model. This is
an instruction-only recovery, with unchanged raster wait; it is not an
arrival or acceptance repair. The seven **146,786–146,810** rows retain
the same **-22** residual and zero added clocks. Both early spotlight rows
**101,031 / 109,016** remain unchanged. All twelve upload rows retain
caller, receiver/acknowledgement and return/suffix phase blockers;
`upload-phase-current.json` retains current native phase receipts for all
twelve rows alongside the hash-verified source phase authority. These
access counts and elapsed clocks are diagnostics, never prices; protocol
mechanisms and physical arrival/acceptance ownership are unchanged.

The furthest classified cohort remains **380,243**: its 116 instruction
rows are not exhausted, so the conditional move into 380,243–461,989 is
not taken. The next signed ranking begins with sprite `$17/$09`
(19 rows / 11 opportunities), `$33/$09` (11 / eight), heart piece
(ten / four), red spear guard (six / three) and Hardhat Beetle (five / three).
Their bodies, the remaining spike/Eyegore/eye-statue/Pokey/thrown-item
owners and signed non-sprite caller/child remainders remain unpriced as
listed below.

### Owners and unmodified-ROM fixtures

Every price uses decoded ROM block markers, native branch/index outcomes
and actual routine entries. All **14 new fixtures** start from
`marker29_state`, which explicitly constructs a `lag_free` state. Their
PC predicates exclude children. Caller coverage assertions check both
branch outcomes; helper fixtures exercise their fixed loops, table indices
and actual entry guards. ROM random results are replayed where a child
needs them. No measured deficit, sampled instruction count, threshold
or scene table becomes a price.

| Owner / actual entries | Fixture names |
| --- | --- |
| Helmasaur root and four AI states; Helmasaur fireball gate and caller-owned popped tail: `1e8039`, `1e819c`, `1e81d5`, `1e8210`, `1e8242`, `1e8253`, `1e825e` | `prefix26_helmasaur_root_ai_and_child_returns_match_rom` |
| Helmasaur reinitialization: `1e8019` | `prefix26_helmasaur_reinitialize_all_table_indices_match_rom` |
| Helmasaur swing: `1e82a0` | `prefix26_helmasaur_swing_all_branches_match_rom` |
| Helmasaur mask caller: `1e8686` | `prefix26_helmasaur_mask_all_caller_branches_match_rom` |
| Helmasaur movement and animation child; Helmasaur fireball gate and caller-owned popped tail: `1e81e6`, `1e8200`, `1e8253`, `1e825e` | `prefix26_helmasaur_movement_and_fireball_child_scopes_match_rom` |
| Master Sword subtype dispatch: `0588c5`, `008781` | `prefix26_sword_dispatch_excludes_all_five_children` |
| Master Sword prop caller and states; Master Sword light fountain; Master Sword light well; Master Sword fountain drawing: `058d29`, `058d40`, `058d57`, `058d7a`, `0589dc`, `058a16`, `058a94` | `prefix26_sword_prop_and_fountain_all_branches_match_rom` |
| Cukeman caller: `1afa0c` | `prefix26_cukeman_all_caller_branches_match_rom` |
| Buzzblob actual recoil guard: `06ff78` | `prefix26_buzzblob_actual_recoil_entry_matches_rom` |
| Raven actual projection wrapper: `06ea1a` | `prefix26_raven_project_speed_far_wrapper_matches_rom` |
| Apple seed handler: `1ef515` | `prefix26_apple_seed_handler_all_branches_match_rom` |
| Lost Woods bird caller and states; Lost Woods squirrel caller: `05940e`, `059468` | `prefix26_lost_woods_callers_all_branches_match_rom` |
| Hog spear man actual movement and recoil entries; Blue bolt guard actual movement and recoil entries: `05cdd4`, `05f971`, `05f9ed` | `prefix26_hog_and_bolt_actual_movement_and_recoil_entries_match_rom` |
| Crystal switch actual same-layer entry: `06f154`, `06f15c` | `prefix26_crystal_switch_actual_same_layer_entry_matches_rom` |

The guarded-movement and prop callers select the existing bank-5 XY
entries; `marker31_bank5_move_xy_matches_rom`,
`marker31_bank5_move_x_matches_rom` and
`marker31_bank5_move_y_matches_rom` also pass in the complete sprite
group. The nonzero recoil bodies remain unpriced beyond their already
proved entry guards. The apple seed handler is priced independently of
the remaining moving-apple body.

Helmasaur MaybeFireball closes its child frame at the first PLA. The
second PLA, random/sound tail and return execute in the actual caller
frame; both trigger paths and the ordinary return have ROM fixtures.
Reinitialization, animation ticks, movement, swing and mask callers exclude
their children. The mask retains the already priced shared `$1E:87F0`
wrapper once per real invocation. Master Sword subtype and prop dispatches
exclude their child handlers and keep the JumpTableLocal frame split.

The generator adds **207 blocks (5,770 → 5,977)**; every old block pricing tuple is
byte-for-byte unchanged. `generated-block-preservation.json` records the
comparison. Across the **214** retained profile rows, all main-budget,
routine-total and expanded block-marker deltas equal **296,010 clocks**.
The total scalar delta is **zero**; the existing 2,100 caller-return clocks
move from the old dispatch owner into bird/squirrel scopes without being
charged again.

Across **4,448 changed per-PC rows**, added counts/clocks are nonnegative,
and both additions and affected after-marker totals stay at or below
source counts/clocks. **24** affected rows retain partial coverage from
other callers. `instruction-conservation.json` and
`per-pc-conservation.csv/json` retain the proof. Existing negative signed
remainders are retained, including those that cancel other caller/child
work; they are not new prices or a claim of complete instruction coverage.

| Sprite family / state | Added clocks on 214 retained rows |
| --- | ---: |
| `$00/$09` | 7,980 |
| `$0d/$09` | 13,580 |
| `$1e/$09` | 2,304 |
| `$45/$09` | 6,320 |
| `$59/$09` | 3,612 |
| `$5a/$09` | 5,388 |
| `$62/$09` | 63,024 |
| `$92/$09` | 190,778 |
| `$ac/$09` | 3,024 |
| **Total** | **296,010** |

The actual 29 hold recoveries group as **13 Helmasaur**, **five Master
Sword**, **three lost-woods wildlife**, **six hog/bolt entry selections**
(three after the classified cohort) and **two combined Buzzblob/raven/apple**
rows. `recovered-owner-groups.json` and `recovered-row-mechanisms.json`
retain each row and instruction delta. None changes raster-wait work.

### Remaining rows and their blockers

Signed families below include every slot and child alias. The other
remainder is signed as well; raw positive scopes are not summed as prices.
`remaining-blockers.csv/json` and `remaining-sprite-residuals.csv` retain
the complete machine-readable vectors.

| Iteration | Class | Frozen boundary | Signed nonpoll gap | Blocker |
| --- | --- | ---: | ---: | --- |
| 34,318 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 48,556 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 95,127 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 101,031 | d | 160,288 | 14,600 | Spotlight placement/actual wait: 1,429 source V-counter reads and 254,184 extra source poll-instruction clocks; remaining nonpoll owners: dash item $3b/$09 +2,588; other signed caller/child remainder +12,012 |
| 109,000 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 109,016 | d | 151,160 | 2,302 | Spotlight placement/actual wait: 1,428 source V-counter reads and 254,006 extra source poll-instruction clocks; remaining nonpoll owners: big fairy $c8/$09 +1,600; fairy $72/$09 +724; other signed caller/child remainder -22 |
| 129,746 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 146,148 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 146,786 | d | 2,700 | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,790 | d | 2,602 | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,794 | d | 5,012 | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,798 | d | 7,822 | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,802 | d | 10,414 | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,806 | d | 12,286 | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 146,810 | d | 14,732 | -22 | Unchanged -22-clock main-instruction convention; actual main/NMI/HDMA arrival and wait/acceptance ownership remain unresolved. |
| 150,378 | a | 21,708 | 49,376 | Remaining instruction owners: sprite $ec $ec/$06 +10,178; other signed caller/child remainder +39,198 |
| 154,637 | a | 1,334 | 5,674 | Remaining instruction owners: sprite $55 $55/$09 +2,004; sprite $eb $eb/$09 +1,470; sprite $52 $52/$09 +902; other signed caller/child remainder +1,298 |
| 158,519 | a | 1,178 | 3,850 | Remaining instruction owners: sprite $eb $eb/$09 +1,470; sprite $55 $55/$09 +1,076; other signed caller/child remainder +1,304 |
| 158,558 | a | 2,752 | 3,556 | Remaining instruction owners: sprite $eb $eb/$09 +1,470; sprite $55 $55/$09 +782; other signed caller/child remainder +1,304 |
| 158,729 | a | 300 | 658 | Remaining instruction owners: sprite $eb $eb/$09 +456; sprite $55 $55/$09 +218; other signed caller/child remainder -16 |
| 160,109 | a | 720 | 918 | Remaining instruction owners: sprite $37 $37/$09 +848; sprite $58 $58/$09 +92; other signed caller/child remainder -22 |
| 160,154 | a | 906 | 1,428 | Remaining instruction owners: sprite $55 $55/$09 +500; other signed caller/child remainder +928 |
| 161,123 | a | 6,540 | 7,132 | Remaining instruction owners: sprite $55 $55/$09 +3,544; sprite $eb $eb/$09 +1,672; other signed caller/child remainder +1,916 |
| 162,301 | a | 1,548 | 7,376 | Remaining instruction owners: sprite $52 $52/$09 +3,636; sprite $eb $eb/$09 +456; other signed caller/child remainder +3,284 |
| 162,819 | a | 6,686 | 7,110 | Remaining instruction owners: sprite $52 $52/$09 +3,636; sprite $eb $eb/$09 +456; other signed caller/child remainder +3,018 |
| 168,346 | d | 83,650 | 8,290 | Arrival/acceptance or actual wait unresolved: complete signed nonpoll remainder is below the frozen boundary; remaining signed owners: sprite $55 $55/$09 +500; other signed caller/child remainder +7,790 |
| 171,309 | d | 21,222 | 8,060 | Arrival/acceptance or actual wait unresolved: complete signed nonpoll remainder is below the frozen boundary; remaining signed owners: sprite $2b $2b/$09 +1,454; other signed caller/child remainder +6,606 |
| 171,310 | d | 63,222 | 1,432 | Arrival/acceptance or actual wait unresolved: complete signed nonpoll remainder is below the frozen boundary; remaining signed owners: sprite $2b $2b/$09 +1,454; other signed caller/child remainder -22 |
| 172,380 | d | 143,228 | -116 | Arrival/acceptance or actual wait unresolved: complete signed nonpoll remainder is below the frozen boundary; remaining signed owners: blue archer $46/$09 -94; other signed caller/child remainder -22 |
| 174,033 | a | 665 | 2,210 | Remaining instruction owners: sprite $d4 $d4/$09 +888; sprite $d9 $d9/$09 +526; other signed caller/child remainder +796 |
| 174,034 | a | 1,185 | 2,498 | Remaining instruction owners: sprite $d4 $d4/$09 +888; sprite $d9 $d9/$09 +526; other signed caller/child remainder +1,084 |
| 174,280 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 174,296 | d | 152,714 | 2,302 | Spotlight placement/actual wait: 1,427 source V-counter reads and 253,828 extra source poll-instruction clocks; remaining nonpoll owners: big fairy $c8/$09 +1,600; fairy $72/$09 +724; other signed caller/child remainder -22 |
| 180,178 | a | 15,954 | 20,668 | Remaining instruction owners: red spear guard $48/$09 +7,920; sprite $ec $ec/$06 +1,428; sprite $c5 $c5/$09 +208; other signed caller/child remainder +11,112 |
| 180,182 | a | 9,518 | 15,650 | Remaining instruction owners: sprite $ec $ec/$06 +1,152; red spear guard $48/$09 +816; sprite $c5 $c5/$09 +208; other signed caller/child remainder +13,474 |
| 181,568 | a | 494 | 2,462 | Remaining instruction owners: sprite $c4 $c4/$09 +920; sprite $d9 $d9/$09 +600; sprite $dc $dc/$09 +600; sprite $e1 $e1/$09 +300; fake sword $e8/$09 -126; other signed caller/child remainder +168 |
| 186,606 | a | 242 | 1,060 | Remaining instruction owners: sprite $17 $17/$09 +892; other signed caller/child remainder +168 |
| 186,608 | a | 830 | 1,060 | Remaining instruction owners: sprite $17 $17/$09 +892; other signed caller/child remainder +168 |
| 186,614 | a | 392 | 1,060 | Remaining instruction owners: sprite $17 $17/$09 +892; other signed caller/child remainder +168 |
| 186,616 | a | 942 | 1,060 | Remaining instruction owners: sprite $17 $17/$09 +892; other signed caller/child remainder +168 |
| 186,618 | a | 50 | 1,060 | Remaining instruction owners: sprite $17 $17/$09 +892; other signed caller/child remainder +168 |
| 186,620 | a | 488 | 1,060 | Remaining instruction owners: sprite $17 $17/$09 +892; other signed caller/child remainder +168 |
| 186,622 | a | 112 | 1,060 | Remaining instruction owners: sprite $17 $17/$09 +892; other signed caller/child remainder +168 |
| 186,624 | a | 706 | 1,060 | Remaining instruction owners: sprite $17 $17/$09 +892; other signed caller/child remainder +168 |
| 186,626 | a | 174 | 1,060 | Remaining instruction owners: sprite $17 $17/$09 +892; other signed caller/child remainder +168 |
| 186,664 | a | 1,014 | 2,898 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; other signed caller/child remainder +168 |
| 186,668 | a | 964 | 2,898 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; other signed caller/child remainder +168 |
| 186,670 | a | 482 | 2,898 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; other signed caller/child remainder +168 |
| 186,672 | a | 892 | 2,898 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; other signed caller/child remainder +168 |
| 186,684 | a | 2,002 | 2,898 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; other signed caller/child remainder +168 |
| 186,686 | a | 1,890 | 2,898 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; other signed caller/child remainder +168 |
| 186,690 | a | 1,106 | 2,898 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; other signed caller/child remainder +168 |
| 186,694 | a | 1,820 | 2,898 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; other signed caller/child remainder +168 |
| 186,696 | a | 196 | 2,898 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; other signed caller/child remainder +168 |
| 186,698 | a | 1,234 | 2,898 | Remaining instruction owners: sprite $33 $33/$09 +1,234; sprite $17 $17/$09 +892; sprite $79 $79/$09 +604; other signed caller/child remainder +168 |
| 187,779 | a | 3,170 | 3,360 | Remaining instruction owners: other signed caller/child remainder +3,360 |
| 189,344 | a | 5,586 | 6,014 | Remaining instruction owners: sprite $c2 $c2/$09 +1,192; sprite $27 $27/$09 +192; other signed caller/child remainder +4,630 |
| 189,368 | a | 3,306 | 4,630 | Remaining instruction owners: sprite $c2 $c2/$09 +1,192; sprite $ad $ad/$09 +610; other signed caller/child remainder +2,828 |
| 189,923 | d | 224,566 | 1,170 | Spotlight placement/actual wait: 1,423 source V-counter reads and 253,116 extra source poll-instruction clocks; remaining nonpoll owners: sprite $c2 $c2/$09 +1,192; other signed caller/child remainder -22 |
| 190,815 | a | 1,836 | 2,220 | Remaining instruction owners: sprite $c2 $c2/$09 +894; sprite $27 $27/$09 +192; other signed caller/child remainder +1,134 |
| 194,151 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 194,994 | a | 2,356 | 5,222 | Remaining instruction owners: sprite $87 $87/$09 +3,024; Mini Moldorm $18/$09 +230; sprite $86 $86/$09 +216; crystal switch $1e/$09 -1,152; other signed caller/child remainder +2,904 |
| 196,051 | a | 7,518 | 8,090 | Remaining instruction owners: other signed caller/child remainder +8,090 |
| 198,239 | a | 540 | 3,990 | Remaining instruction owners: sprite $a7 $a7/$09 +1,760; Mini Moldorm $18/$09 +882; sprite $e4 $e4/$09 +248; other signed caller/child remainder +1,100 |
| 198,240 | a | 124 | 3,872 | Remaining instruction owners: sprite $a7 $a7/$09 +1,760; Mini Moldorm $18/$09 +882; sprite $e4 $e4/$09 +248; other signed caller/child remainder +982 |
| 209,808 | a | 4,612 | 14,628 | Remaining instruction owners: crystal switch $1e/$09 -476; other signed caller/child remainder +15,104 |
| 215,857 | a | 714 | 982 | Remaining instruction owners: other signed caller/child remainder +982 |
| 215,858 | a | 716 | 982 | Remaining instruction owners: other signed caller/child remainder +982 |
| 215,859 | a | 714 | 982 | Remaining instruction owners: other signed caller/child remainder +982 |
| 217,220 | a | 714 | 1,100 | Remaining instruction owners: other signed caller/child remainder +1,100 |
| 217,221 | a | 298 | 982 | Remaining instruction owners: other signed caller/child remainder +982 |
| 217,222 | a | 714 | 982 | Remaining instruction owners: other signed caller/child remainder +982 |
| 225,257 | a | 3,242 | 4,964 | Remaining instruction owners: dungeon object $239 +854; dungeon object $232 +352; other signed caller/child remainder +3,758 |
| 239,545 | a | 12 | 368 | Remaining instruction owners: sprite_kakariko_boy $3c/$09 +68; sprite_running_boy $74/$09 +68; sprite_sweeping_lady $2a/$09 +64; other signed caller/child remainder +168 |
| 242,100 | d | 44,504 | -22 | Arrival/acceptance or actual wait unresolved: complete signed nonpoll remainder is below the frozen boundary; remaining signed owners: other signed caller/child remainder -22 |
| 242,103 | a | 476 | 928 | Remaining instruction owners: other signed caller/child remainder +928 |
| 242,122 | a | 728 | 1,032 | Remaining instruction owners: other signed caller/child remainder +1,032 |
| 253,040 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 254,154 | d | 228,700 | 434 | Spotlight placement/actual wait: 1,428 source V-counter reads and 254,006 extra source poll-instruction clocks; remaining nonpoll owners: sprite $eb $eb/$09 +456; other signed caller/child remainder -22 |
| 255,517 | a | 1,000 | 2,770 | Remaining instruction owners: red spear guard $48/$09 +1,646; other signed caller/child remainder +1,124 |
| 255,521 | a | 1,608 | 2,968 | Remaining instruction owners: red spear guard $48/$09 +1,646; other signed caller/child remainder +1,322 |
| 255,525 | a | 1,294 | 2,968 | Remaining instruction owners: red spear guard $48/$09 +1,646; other signed caller/child remainder +1,322 |
| 255,529 | a | 1,526 | 2,266 | Remaining instruction owners: red spear guard $48/$09 +944; other signed caller/child remainder +1,322 |
| 255,572 | a | 1,022 | 2,556 | Remaining instruction owners: sprite $33 $33/$09 +1,234; other signed caller/child remainder +1,322 |
| 256,490 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 256,786 | a | 7,284 | 43,632 | Remaining instruction owners: dungeon object $171 +12,940; dungeon object $12f +4,514; dungeon object $130 +4,514; dungeon object $214 +2,518; dungeon object $215 +2,518; dungeon object $216 +2,518; dungeon object $217 +2,518; dungeon object $170 +2,020; dungeon object $220 +124; other signed caller/child remainder +9,448 |
| 281,028 | a | 6,862 | 8,362 | Remaining instruction owners: sprite_1_c_statue $1c/$09 +460; blue archer $46/$09 -188; other signed caller/child remainder +8,090 |
| 282,475 | a | 160 | 740 | Remaining instruction owners: green guard $41/$05 +718; sprite_1_c_statue $1c/$09 +460; other signed caller/child remainder -438 |
| 282,497 | a | 8,584 | 47,980 | Remaining instruction owners: green guard $41/$05 +718; sprite_1_c_statue $1c/$09 +460; dungeon object $171 +21,488; dungeon object $32d +17,816; dungeon object $321 +2,538; dungeon object $170 +1,880; other signed caller/child remainder +3,080 |
| 294,833 | a | 9,914 | 13,668 | Remaining instruction owners: sprite_4_a_bomb_guard $4a/$06 +7,492; sprite_11_hinox $11/$09 +2,634; sprite_22_ropa $22/$09 +1,918; other signed caller/child remainder +1,624 |
| 294,835 | a | 10,642 | 18,236 | Remaining instruction owners: sprite_4_a_bomb_guard $4a/$06 +7,594; green guard $41/$09 +2,728; sprite_11_hinox $11/$09 +2,634; sprite_22_ropa $22/$09 +1,918; other signed caller/child remainder +3,362 |
| 294,837 | a | 12,492 | 15,022 | Remaining instruction owners: sprite_4_a_bomb_guard $4a/$06 +6,420; green guard $41/$09 +2,728; sprite_11_hinox $11/$09 +2,634; sprite_22_ropa $22/$09 +1,918; other signed caller/child remainder +1,322 |
| 294,844 | d | 7,862 | 7,802 | Arrival/acceptance or actual wait unresolved: complete signed nonpoll remainder is below the frozen boundary; remaining signed owners: sprite_4_a_bomb_guard $4a/$06 +7,372; sprite_11_hinox $11/$09 +452; other signed caller/child remainder -22 |
| 295,089 | d | 147,092 | 2,302 | Spotlight placement/actual wait: 1,427 source V-counter reads and 253,828 extra source poll-instruction clocks; remaining nonpoll owners: big fairy $c8/$09 +1,600; fairy $72/$09 +724; other signed caller/child remainder -22 |
| 297,579 | a | 474 | 5,654 | Remaining instruction owners: sprite $eb $eb/$09 +1,470; sprite_45_hog_spear_man $45/$09 +1,104; sprite $55 $55/$09 +676; other signed caller/child remainder +2,404 |
| 297,585 | a | 7,966 | 8,898 | Remaining instruction owners: sprite $55 $55/$09 +3,920; sprite $eb $eb/$09 +1,470; sprite_45_hog_spear_man $45/$09 +1,104; other signed caller/child remainder +2,404 |
| 297,638 | a | 2,396 | 7,660 | Remaining instruction owners: sprite $55 $55/$09 +1,776; sprite $eb $eb/$09 +1,470; sprite_45_hog_spear_man $45/$09 +1,104; other signed caller/child remainder +3,310 |
| 297,863 | d | 129,956 | 4,942 | Spotlight placement/actual wait: 1,426 source V-counter reads and 253,650 extra source poll-instruction clocks; remaining nonpoll owners: sprite_bb_shopkeeper $bb/$09 +5,352; sprite_28_dark_world_hint_npc $28/$09 -388; other signed caller/child remainder -22 |
| 299,434 | d | 147,848 | 2,302 | Spotlight placement/actual wait: 1,427 source V-counter reads and 253,828 extra source poll-instruction clocks; remaining nonpoll owners: big fairy $c8/$09 +1,600; fairy $72/$09 +724; other signed caller/child remainder -22 |
| 299,510 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 304,890 | a | 11,756 | 15,634 | Remaining instruction owners: sprite_0_f_octoballoon $0f/$06 +11,456; sprite_10_octoballoon_baby $10/$09 +4,010; other signed caller/child remainder +168 |
| 307,022 | d | 234,416 | 2,666 | Spotlight placement/actual wait: 1,405 source V-counter reads and 249,912 extra source poll-instruction clocks; remaining nonpoll owners: sprite $58 $58/$09 +2,582; other signed caller/child remainder +84 |
| 307,040 | c | — | — | Upload caller, receiver/acknowledgement and return/suffix arrival ownership remain unresolved; mechanisms unchanged. |
| 310,875 | a | 10,058 | 20,736 | Remaining instruction owners: sprite_0_f_octoballoon $0f/$06 +11,518; sprite_10_octoballoon_baby $10/$09 +4,010; sprite $55 $55/$09 +676; other signed caller/child remainder +4,532 |
| 310,980 | d | 171,138 | 7,790 | Arrival/acceptance or actual wait unresolved: complete signed nonpoll remainder is below the frozen boundary; remaining signed owners: other signed caller/child remainder +7,790 |
| 310,982 | d | 84,542 | 710 | Arrival/acceptance or actual wait unresolved: complete signed nonpoll remainder is below the frozen boundary; remaining signed owners: other signed caller/child remainder +710 |
| 313,078 | a | 32,062 | 32,694 | Remaining instruction owners: sprite_19_poe $19/$09 +8,560; green guard $41/$09 +716; other signed caller/child remainder +23,418 |
| 313,255 | a | 35,952 | 36,056 | Remaining instruction owners: sprite_19_poe $19/$09 +9,850; green guard $41/$09 +2,704; other signed caller/child remainder +23,502 |
| 317,588 | a | 3,970 | 8,098 | Remaining instruction owners: sprite_45_hog_spear_man $45/$09 +92; other signed caller/child remainder +8,006 |
| 322,193 | a | 10,344 | 11,236 | Remaining instruction owners: sprite_22_ropa $22/$09 +2,170; sprite_11_hinox $11/$09 +1,510; sprite_4_a_bomb_guard $4a/$09 +1,082; sprite_12_moblin $12/$09 +976; sprite_b6_kiki $b6/$09 +864; other signed caller/child remainder +4,634 |
| 327,434 | a | 720 | 786 | Remaining instruction owners: sprite_15_antifairy $15/$09 +212; other signed caller/child remainder +574 |
| 336,024 | a | 58 | 894 | Remaining instruction owners: sprite_15_antifairy $15/$09 +424; other signed caller/child remainder +470 |
| 336,035 | a | 452 | 998 | Remaining instruction owners: sprite_15_antifairy $15/$09 +424; other signed caller/child remainder +574 |
| 336,039 | a | 424 | 998 | Remaining instruction owners: sprite_15_antifairy $15/$09 +424; other signed caller/child remainder +574 |
| 336,046 | a | 576 | 894 | Remaining instruction owners: sprite_15_antifairy $15/$09 +424; other signed caller/child remainder +470 |
| 337,919 | a | 808 | 16,988 | Remaining instruction owners: sprite $26 $26/$09 +4,072; crystal switch $1e/$09 -238; other signed caller/child remainder +13,154 |
| 337,925 | a | 1,954 | 4,824 | Remaining instruction owners: sprite $26 $26/$09 +1,876; crystal switch $1e/$09 -238; other signed caller/child remainder +3,186 |
| 338,240 | a | 928 | 6,388 | Remaining instruction owners: sprite $26 $26/$09 +2,340; crystal switch $1e/$09 +862; other signed caller/child remainder +3,186 |
| 338,673 | a | 9,468 | 10,312 | Remaining instruction owners: sprite $26 $26/$09 +8,144; crystal switch $1e/$09 -238; other signed caller/child remainder +2,406 |
| 344,246 | a | 4,258 | 5,390 | Remaining instruction owners: spike $8a/$08 +702; Pokey $8e/$08 +168; sprite_1_c_statue $1c/$08 +142; other signed caller/child remainder +4,378 |
| 347,334 | a | 4,954 | 10,312 | Remaining instruction owners: sprite $26 $26/$09 +8,144; crystal switch $1e/$09 -238; other signed caller/child remainder +2,406 |
| 348,203 | a | 3,532 | 5,404 | Remaining instruction owners: Pokey $8e/$09 +308; other signed caller/child remainder +5,096 |
| 353,572 | a | 3,198 | 5,276 | Remaining instruction owners: Pokey $8e/$09 +308; other signed caller/child remainder +4,968 |
| 354,592 | a | 1,510 | 1,848 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +470 |
| 354,594 | a | 48 | 1,848 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +470 |
| 354,595 | d | 1,966 | 1,952 | Arrival/acceptance or actual wait unresolved: complete signed nonpoll remainder is below the frozen boundary; remaining signed owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +574 |
| 354,604 | a | 1,226 | 1,848 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +470 |
| 354,611 | a | 344 | 1,952 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +574 |
| 354,616 | a | 1,272 | 1,848 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +470 |
| 354,619 | a | 1,692 | 1,952 | Remaining instruction owners: sprite $ec $ec/$09 +1,070; Pokey $8e/$09 +308; other signed caller/child remainder +574 |
| 356,806 | a | 2,330 | 7,582 | Remaining instruction owners: sprite $ec $ec/$0a +4,084; crystal switch $1e/$09 +870; sprite $d9 $d9/$09 +300; other signed caller/child remainder +2,328 |
| 367,254 | a | 1,342 | 9,646 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +394; spike $8a/$09 +156; crystal switch $1e/$09 -668; other signed caller/child remainder +8,914 |
| 367,255 | a | 2,052 | 9,646 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +394; spike $8a/$09 +156; crystal switch $1e/$09 -668; other signed caller/child remainder +8,914 |
| 367,256 | a | 5,532 | 9,646 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +394; spike $8a/$09 +156; crystal switch $1e/$09 -668; other signed caller/child remainder +8,914 |
| 367,257 | a | 3,064 | 9,646 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +394; spike $8a/$09 +156; crystal switch $1e/$09 -668; other signed caller/child remainder +8,914 |
| 367,286 | a | 30,104 | 36,046 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +394; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +34,876 |
| 367,291 | a | 20,924 | 33,378 | Remaining instruction owners: green Eyegore $83/$09 +4,782; eye statue $38/$09 +450; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +28,020 |
| 367,313 | a | 38,446 | 43,440 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +42,464 |
| 367,327 | a | 35,246 | 35,880 | Remaining instruction owners: green Eyegore $83/$09 +1,374; eye statue $38/$09 +450; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +33,930 |
| 367,337 | a | 37,994 | 43,472 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +42,496 |
| 367,347 | a | 33,266 | 44,110 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +43,134 |
| 367,361 | a | 38,924 | 44,422 | Remaining instruction owners: green Eyegore $83/$09 +1,374; eye statue $38/$09 +450; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +42,472 |
| 367,372 | a | 37,698 | 44,308 | Remaining instruction owners: eye statue $38/$09 +450; green Eyegore $83/$09 +400; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +43,332 |
| 367,394 | a | 23,774 | 33,506 | Remaining instruction owners: green Eyegore $83/$09 +4,782; eye statue $38/$09 +450; red Eyegore $84/$09 +200; spike $8a/$09 +156; crystal switch $1e/$09 -230; other signed caller/child remainder +28,148 |
| 370,567 | a | 2,424 | 2,658 | Remaining instruction owners: Pokey $8e/$09 +1,386; crystal switch $1e/$09 -238; other signed caller/child remainder +1,510 |
| 372,814 | a | 16,820 | 18,356 | Remaining instruction owners: Pokey $8e/$09 +14,210; Pokey $8e/$06 +154; sprite_d9_green_rupee $da/$09 +108; crystal switch $1e/$09 -238; other signed caller/child remainder +4,122 |
| 375,434 | a | 24,120 | 29,948 | Remaining instruction owners: crystal switch $1e/$09 -238; dungeon object $171 +22,704; dungeon object $170 +2,830; dungeon object $312 +768; dungeon object $32a +330; other signed caller/child remainder +3,554 |
| 378,071 | a | 846 | 2,136 | Remaining instruction owners: Helmasaur King $92/$09 +8; other signed caller/child remainder +2,128 |
| 378,096 | a | 1,104 | 2,170 | Remaining instruction owners: Helmasaur King $92/$09 -1,316; other signed caller/child remainder +3,486 |
| 378,098 | a | 660 | 1,202 | Remaining instruction owners: Helmasaur King $92/$09 -1,316; other signed caller/child remainder +2,518 |
| 378,105 | a | 158 | 1,202 | Remaining instruction owners: Helmasaur King $92/$09 -1,316; other signed caller/child remainder +2,518 |
| 378,541 | a | 4,944 | 6,400 | Remaining instruction owners: Helmasaur King $92/$09 +2,932; other signed caller/child remainder +3,468 |
| 378,830 | a | 1,104 | 5,450 | Remaining instruction owners: Helmasaur King $92/$09 +2,932; other signed caller/child remainder +2,518 |

### Full ROM and CPU pictures

The specified eight-slice ROM helper passes **1,473,123 iterations /
unexplained 0**, at **1,655 → 1,626 missing**, **0 → 0 excess** and
**0 → 0 false positives**. All 29 changed ROM holds recover one missing
hold each; none is lost. Every native hold agrees with the actual `cost_p2`
model, and no main-instruction budget decreases. The simpler `predicted`
column is not the P2 admission model. `rom-admission-helper.log`,
`rom-gate.txt` and `rom-comparison.json` retain the complete evidence.

| Before iteration | ROM missing before → after | Live missing before → after |
| --- | ---: | ---: |
| 94,334 | 2 → 2 | 2 → 2 |
| 141,568 | 7 → 7 | 7 → 7 |
| 228,499 | 75 → 73 | 75 → 73 |
| 380,243 | 179 → 153 | 179 → 153 |
| 461,989 | 235 → 209 | 349 → 209 |

The three later ROM recoveries are **893,988, 894,008 and 1,068,891**,
each with **1,648** restored instruction clocks from actual hog/bolt
entry selections. They are included in the full-route totals but do not
extend the classified cohort.

The independent eight-slice `--cpu-video` census also passes all
**1,473,123 iterations / unexplained 0** and retains **1,462,997 exact
pictures**: 563 early, 3,952 late, four during-lag, 905 polygon-thread,
4,702 torn and zero other content misses. All eight picture-hash files
are byte-identical to the hash-verified baseline. `cpu-gate.txt`,
`cpu-picture-agreement.json` and `cpu-admission-commands.json` retain
the CPU-only proof.

### One continuous live census and direct song-end attribution

Exactly **one continuous seeded** `--live-apu-reads --pacing-predictor`
process completes all **1,473,123 iterations**, with SPC state carried
across seeds. Its logical gate passes with **unexplained 0**. Live totals
improve from **2,172 → 1,899 missing**, **399 → 269 excess** and
**88 → 75 false positives**. `live-command.json` records the single
launch and completed exit; `live-gate.txt`, `live-counts.txt` and
`live-timing.jsonl.zst` retain the complete run.

There are **65 changed live hold rows**. **29** share the exact changed
ROM before/after pair. Every other **36** is attributed to a crystal
song-end poll at **`$08:C606`**: **15** in the 380,243 episode, **six**
in 461,989 and **15** in 1,317,697. There are **zero pendant-only changes**
and **zero unattributed rows**. The conservative live-only set includes
every changed pair that is not exactly a changed ROM pair.

The shared ROM contribution is **-29 / 0 / 0** missing/excess/FP.
The three crystal episodes contribute **-244 / -130 / -13**, making
the complete live delta **-273 / -130 / -13**. Their first native zero
now agrees with the directly observed source zero. No polling-price,
arrival, acceptance or upload mechanism is changed to produce this result.

The source PCs are established by **20 direct APUI bus receipts**
(nonzero and first-zero reads in ten episodes) from the hash-verified
cold unmodified-ROM CPU source replay. Every receipt was checked again
against `target/prefix-hunt24/source-polls-direct/apu-bus.csv`; the ROM,
ledger and source authorities remain unchanged. `poll-source-identity.json`
and `poll-source-verification.json` record this reuse. The complete source
context/read payload hash agrees between baseline and candidate traces.
PCs are never inferred from sprite labels.

Bounds are derived **without changed holds**. Contiguous native poll
clusters overlap the directly observed source-poll intervals. Actual
boolean polls, native contexts and four-lane main-without-VWF/VWF
differences locate execution divergence while removing intended ROM
price changes. The first actual reseed after the last observed execution
difference ends recovery, or the containing 2,000-iteration boundary
provides the fallback. `live-execution-events.json` and
`poll-attribution.json` retain the complete evidence and bounds. Main
instruction totals already include VWF; it is separated only for this
diagnostic comparison.

| Episode | Direct source PC | Source first zero | Native first zero before → after | Last observed execution difference | Recovery end | Live-only rows | Delta missing / excess / FP |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 94,334 | `$08:C3FD` | 94,854 | 94,854 → 94,854 | — | 96,000 | 0 | +0 / +0 / +0 |
| 141,568 | `$08:C3FD` | 142,088 | 142,088 → 142,088 | — | 144,000 | 0 | +0 / +0 / +0 |
| 228,499 | `$08:C3FD` | 229,035 | 229,035 → 229,035 | — | 230,000 | 0 | +0 / +0 / +0 |
| 380,243 | `$08:C606` | 380,881 | 380,882 → 380,881 | 383,635 | 383,636 | 15 | -114 / -90 / -5 |
| 461,989 | `$08:C606` | 462,628 | 462,627 → 462,628 | 464,051 | 466,000 | 6 | -13 / -13 / -3 |
| 626,618 | `$08:C606` | 627,256 | 627,257 → 627,257 | — | 628,000 | 0 | +0 / +0 / +0 |
| 735,229 | `$08:C606` | 735,867 | 735,868 → 735,868 | — | 736,000 | 0 | +0 / +0 / +0 |
| 881,095 | `$08:C606` | 881,734 | 881,733 → 881,733 | — | 882,000 | 0 | +0 / +0 / +0 |
| 1,064,096 | `$08:C606` | 1,064,734 | 1,064,734 → 1,064,734 | — | 1,066,000 | 0 | +0 / +0 / +0 |
| 1,317,697 | `$08:C606` | 1,318,335 | 1,318,336 → 1,318,335 | 1,320,899 | 1,320,900 | 15 | -117 / -27 / -5 |

Every live-only changed row is listed below. `changed-live-holds.csv/json`
also retains its budgets and metric deltas; `all-changed-live-holds.csv`
includes the 29 shared ROM changes.

| Iteration | Source lag | ROM holds before → after | Live holds before → after | Episode / direct source PC |
| --- | ---: | ---: | ---: | --- |
| 380,946 | 1 | 1 → 1 | 0 → 1 | 380,243 / `$08:C606` |
| 380,947 | 0 | 0 → 0 | 1 → 0 | 380,243 / `$08:C606` |
| 380,956 | 5 | 4 → 4 | 0 → 4 | 380,243 / `$08:C606` |
| 380,957 | 0 | 0 → 0 | 4 → 0 | 380,243 / `$08:C606` |
| 381,426 | 8 | 8 → 8 | 0 → 8 | 380,243 / `$08:C606` |
| 381,427 | 0 | 0 → 0 | 8 → 0 | 380,243 / `$08:C606` |
| 383,178 | 0 | 0 → 0 | 7 → 0 | 380,243 / `$08:C606` |
| 383,181 | 7 | 7 → 7 | 0 → 7 | 380,243 / `$08:C606` |
| 383,343 | 7 | 7 → 7 | 0 → 7 | 380,243 / `$08:C606` |
| 383,346 | 0 | 0 → 0 | 7 → 0 | 380,243 / `$08:C606` |
| 383,631 | 59 | 59 → 59 | 0 → 59 | 380,243 / `$08:C606` |
| 383,632 | 6 | 6 → 6 | 0 → 6 | 380,243 / `$08:C606` |
| 383,633 | 23 | 22 → 22 | 0 → 22 | 380,243 / `$08:C606` |
| 383,634 | 1 | 1 → 1 | 59 → 1 | 380,243 / `$08:C606` |
| 383,635 | 1 | 1 → 1 | 6 → 1 | 380,243 / `$08:C606` |
| 462,699 | 0 | 0 → 0 | 1 → 0 | 461,989 / `$08:C606` |
| 462,700 | 1 | 1 → 1 | 0 → 1 | 461,989 / `$08:C606` |
| 462,709 | 0 | 0 → 0 | 4 → 0 | 461,989 / `$08:C606` |
| 462,710 | 5 | 4 → 4 | 0 → 4 | 461,989 / `$08:C606` |
| 463,179 | 0 | 0 → 0 | 8 → 0 | 461,989 / `$08:C606` |
| 463,180 | 8 | 8 → 8 | 0 → 8 | 461,989 / `$08:C606` |
| 1,318,396 | 1 | 1 → 1 | 0 → 1 | 1,317,697 / `$08:C606` |
| 1,318,397 | 0 | 0 → 0 | 1 → 0 | 1,317,697 / `$08:C606` |
| 1,318,406 | 4 | 4 → 4 | 0 → 4 | 1,317,697 / `$08:C606` |
| 1,318,407 | 0 | 0 → 0 | 4 → 0 | 1,317,697 / `$08:C606` |
| 1,318,878 | 8 | 8 → 8 | 0 → 8 | 1,317,697 / `$08:C606` |
| 1,318,879 | 0 | 0 → 0 | 8 → 0 | 1,317,697 / `$08:C606` |
| 1,320,353 | 0 | 0 → 0 | 7 → 0 | 1,317,697 / `$08:C606` |
| 1,320,357 | 7 | 7 → 7 | 0 → 7 | 1,317,697 / `$08:C606` |
| 1,320,542 | 7 | 7 → 7 | 0 → 7 | 1,317,697 / `$08:C606` |
| 1,320,544 | 0 | 0 → 0 | 7 → 0 | 1,317,697 / `$08:C606` |
| 1,320,895 | 58 | 58 → 58 | 0 → 58 | 1,317,697 / `$08:C606` |
| 1,320,896 | 6 | 6 → 6 | 0 → 6 | 1,317,697 / `$08:C606` |
| 1,320,897 | 24 | 24 → 24 | 0 → 24 | 1,317,697 / `$08:C606` |
| 1,320,898 | 1 | 1 → 1 | 0 → 1 | 1,317,697 / `$08:C606` |
| 1,320,899 | 1 | 1 → 1 | 0 → 1 | 1,317,697 / `$08:C606` |

### Verification boundary

The complete touched-module groups pass: sprite-draw **289**, sprite
**120**, guard **13**, Helmasaur **nine**, world **17** and sprite-main
**16**, including all **14 new ROM fixtures**. Generator tests pass
(**eight**). Production builds are warning-free; library compilation
retains the same 24 warning headers as baseline, with no additions.
`test-verification.json`, `filtered-summary.json` and
`warning-agreement.json` retain the results.

After all runtime and fixture changes were frozen, the requested full
default library command ran **once**, with `CARGO_INCREMENTAL=0` solely
to bound rebuildable cache size:

```sh
ZELDA3_ASSETS=/Users/missingno/Documents/zelda3-rs/zelda3_assets.dat cargo test -q --profile parity -p zelda3 -p snes --lib
```

SNES records **505 passed / seven ignored**. Zelda records **3,073
passed / one failed / three ignored**. The sole failure is the permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There is no additional fixture failure, post-full repair or second full
run. `full-library-command.json` and `full-library.log` preserve the
complete transcript.

All **935** frozen source files and the admission binary retain their
hashes across the final ROM, CPU-picture, selected-profile, live and
library checks. Final checks also preserve **45 reused artifacts** and
all **14 authority files**. Current source and binary agreement is in
`runtime-source-agreement.json`; authority and prior-artifact checks are
in `authorities-after.json` and `prior-final-verification.json`.

All 18 game-binary invocations use CPU-only `--logical-replay`, and
every launched game, build, test and trace-decoder process has completed.
No background work remains. There is no GPU, `--video`, cached-A/V run,
external Snes9x process, new environment switch, runtime classifier, matcher
or P2 edit. The ledger, oracle cache and gate helpers remain read-only.
No commit, push or `cargo fmt` occurs. The worktree target remains below
**8,000,000,000 bytes**, with evidence in `target/prefix-hunt26/`;
`resources.json`, `verification.json`, `runtime.patch` and
`artifact-hashes.json` retain the final state.

These are seeded logical, CPU-picture and source-instruction proofs.
The remaining instruction owners, upload caller/receiver/return placement,
spotlight waiting, signed child/caller offsets and actual arrival/acceptance
ownership remain unresolved as listed. Cold exact A/V acceptance is a
separate boundary.

## Lagless prefix hunt 27

Instruction pricing on `codex/prefix-hunt27`, based on main **`5bfe6ca7`**.
Evidence is in `target/prefix-hunt27/`. The immutable final admission binary is
`zelda3-admission-final`, SHA-256
`0f0d985a88a9c12a6761ae6b15c3455ed8e9217a9b0d297a2f086f2ab1b5a840`.

The complete ROM census improves from **1,626 to 1,530 missing**, with **0 excess
and 0 false positives**. There are **116 recovered rows and 20 newly exposed
rows**, a net reduction of **96**. Main instruction work changes by a net
**+997,233,074 master clocks** across the route: positive per-iteration deltas
sum to 1,103,416,314 clocks, and negative deltas from the overcount corrections
and their combinations sum to 106,183,240. These are measured consequences,
never prices. The 209 baseline missing rows through 461,989 gain a net
**517,344 instruction clocks**; 26 recover and seven previously exact rows
become missing. `rom-comparison.json` and `recovered-prefix-rows.csv/json`
retain every change.

### Classification and source identity

The initial worktree is clean at the requested base. All 935 retained base
source hashes agree. Reused hunt-24/25 source profiles, hunt-26 baselines and
the direct song-poll bus receipts are hash-verified before reuse. The regenerated
ledger contains **1,473,123 iterations**. ROM, assets, SRAM, ledger, oracle and
gate-helper authorities remain unchanged.

Fresh native baseline profiles agree with all 22,282 retained routine rows
on the original 214 selected iterations. Cold unmodified-ROM CPU extensions
cover the additional classified cohort and newly exposed rows; there are
**282 profiled prefix rows** in all. Every new source routine total equals
its independently summed per-PC instruction receipts. Final native profiles
are regenerated on the final binary. `baseline-profile-agreement.json`,
`source-new-instruction-agreement.json`,
`source-supplement-instruction-agreement.json`, `analytical-basis.json`
and `profile-clean-agreement.json` retain the evidence.

Class (a) has a complete signed nonpoll instruction remainder at or above the
frozen diagnostic boundary. Class (c) has an executed upload-protocol owner.
Class (d) retains source spotlight polling or insufficient signed instruction
work to reach that boundary. There is no independent arrival proof for class
(b). This classification is diagnostic; the runtime classifier, matcher and P2
are unchanged. Main instruction totals already include VWF. Inclusive totals,
positive aliases, sampled counts, deficits and boundary sizes are never prices.

| Cohort | (a) before → after | (b) | (c) before → after | (d) before → after | Old rows recovered | Newly exposed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Before 141,568 | 0 → 0 | 0 | 5 → 5 | 2 → 2 | 0 | 0 |
| 141,568–228,499 | 50 → 50 | 0 | 3 → 3 | 13 → 14 | 0 | 1 |
| 228,499–380,243 | 66 → 46 | 0 | 4 → 4 | 10 → 11 | 24 | 5 |
| 380,243–461,989 | 47 → 46 | 0 | 2 → 2 | 7 → 7 | 2 | 1 |
| **Total** | **163 → 142** | **0** | **14 → 14** | **32 → 34** | **26** | **7** |

At 380,243, the previous **116 (a) / 12 (c) / 25 (d)** becomes
**96 / 12 / 27**. Through 461,989, **209 → 190** rows remain:
**142 (a), 14 (c), 34 (d)**. The classification equals the canonical ROM
missing-row set exactly. All 282 signed partitions conserve the complete
source-minus-native instruction total.
`classification.csv/json`, `signed-partition-agreement.csv/json`,
`routine-vectors.csv/json`, `non-sprite-vectors.csv/json`,
`object-vectors.csv/json` and `remaining-sprite-residuals.csv`
retain the full signed vectors.

Upload and spotlight mechanisms are byte-identical. Shared instruction prices
incidentally recover spotlight row **406,065** with **162** added instruction
clocks; the overcount corrections expose spotlight row **403,249**. Neither is
a wait, arrival or acceptance repair. All fourteen upload rows remain blocked.
The seven **146,786–146,810** rows retain identical instruction budgets and
holds, including their **-22** residual. `protected-minus22-rows.csv/json`
records that check.

### Concrete ownership of the other signed remainder

At **367,256**, the old **+8,914** non-sprite remainder decomposes exactly:

- Nearby tiles `$07:CF12`: **+3,124**.
- Moving-wall torch scan `$01:CA17`: **+2,814**.
- Y cardinal caller `$07:BA0A` and its signed child offsets: **+1,200**.
- X cardinal caller `$07:C4D4` and its signed child offsets: **+920**.
- Moving-wall east `$01:C8D4`: **+524**; tag `$01:C43E`: **+354**.
- Main-loop offset: **-22**.

The apparent +3,164 module dispatcher cancels -2,752 Dungeon, -108 floor
indicator and -304 HUD work. Link_ControlHandler +2,318 cancels
LinkState_Default -2,318. Velocity +2,416 cancels movement -1,870 and sand
drag -546. Those aliases are not independently priced. Nearby-tile work gains
3,116 clocks and the torch scan gains 2,814, leaving **+2,984** signed
non-sprite work. The complete iteration gains **6,076** clocks and recovers.

At **367,347**, the **+43,134** non-sprite remainder includes sword-beam
caller **6,008**, collision scan **4,202**, radial projection **3,224**,
nearby tiles **3,124**, torches **2,814**, radial sparkle OAM **2,624**,
initial tile work **2,610**, OAM XY **1,664**, AddSwordBeam **1,662**,
Y clipping **1,424**, plus the actual far wrappers, axis entries and
beam-hit caller. Signed parent/child offsets are retained. Pricing reduces
that non-sprite remainder to **+3,610**; the complete iteration gains
**40,108** clocks and recovers.

A new **-6,946** routine residual at BeamHit `$08:8D19` is an attribution
alias: the source RTS-selected body remains in Ancilla_ExecuteOne
`$08:833C`, whose residual is **+7,368**. Together they retain **+422**.
The physical per-PC audit proves the affected native counts and clocks do not
exceed source. This routine-level negative is not an overcharged sprite family.

At **375,434**, concrete object gains are **22,704** for `$171`,
**2,830** for `$170`, **768** for `$312` and **330** for `$32a`:
**26,632** clocks before the independent 238-clock crystal-switch correction.
The actual main-budget gain is **26,394**, recovering the row. LampCones'
old parent **-107,034** and child **+107,364** sum to only **+330**; the child
is already priced and is not charged again. The remaining **+3,554** non-sprite
vector includes spiral adjustment **+1,656**, follower initialization **+770**
and warp-pad work **+492**, with the door, torch and HUD aliases still signed.

The twelve recovered 367,254–367,394 rows and all other prefix recoveries are
listed in `recovered-prefix-rows.csv/json`. Row **367,327** remains blocked.
The remaining `$8e` family at **372,814** is **+1,584**, within a complete
signed gap of **+5,814**. The ROM dispatch is **Terrorpin** at `$1E:B26F`;
Pokey is `$c7`. Pricing follows the actual dispatch, not the inherited label.

### Owners and unmodified-ROM fixtures

All **24 new fixtures** explicitly construct a `lag_free` state. New caller
markers exclude children; overcount fixtures also compare complete routine
totals to detect duplicate child charges. Fixtures execute unmodified ROM
across the supported caller branches, loop bounds, directions, guards,
graphics and clip/page-crossing paths. The existing
`marker32_nearby_tiles_matches_rom` proves the nearby-tile body now selected
by the five cardinal call/resume sites.

| Owner / actual entries | Fixture names |
| --- | --- |
| Radial projection and far wrapper: `08fadd`, `08fb23` | `prefix27_radial_projection_all_angles_radii_and_far_entry_match_rom` |
| Radial sparkle OAM: `08da17`; OAM far wrapper: `08f6fe` | `prefix27_radial_oam_all_sign_and_coordinate_paths_match_rom`; sword-beam caller fixture |
| Actual sword-beam X/Y entries: `0ffa37`, `0ffa43` | `prefix27_sword_beam_axis_wrappers_all_slots_and_velocity_signs_match_rom` |
| Sword-beam caller: `0ff764`; scan/tile far entries `088da2`, `08923b` | `prefix27_sword_beam_all_caller_branches_exclude_children` |
| BeamHit caller/BRL tail and coordinate/clipping helpers: `088d19`, `08fb44`, `08fba6`, `08fbc9`, `08fbd6` | `prefix27_beam_hit_all_timer_clipping_sign_and_oam_crossing_paths_match_rom` |
| Initial-tile caller: `099dd3` | `prefix27_initial_tile_all_directions_collision_and_clear_paths_exclude_children` |
| AddSwordBeam and actual failure return: `0ff67b`, `0ff64d` | `prefix27_add_sword_beam_all_spawn_facing_and_initial_collision_paths_exclude_children` |
| Nearby tiles: `07cf12`; moving-wall torches: `01ca17` | `marker32_nearby_tiles_matches_rom`; `prefix27_moving_wall_torch_scan_all_count_and_guard_paths_match_rom` |
| Objects `$170/$171/$312/$32a`: `0190f9`, `019120`, size children `01b0ac/01b0af`, `019aa9`, `019de5` | `prefix27_room_object_callers_all_sizes_planes_and_rupee_guard_exclude_children`; extended `object_tail_prices_match_executed_rom_paths` |
| Spike caller and states: `1ebce8` | `prefix27_spike_all_stationary_transient_and_motion_states_exclude_children` |
| Red/green Eyegore drawing: `1ecacf`, actual multiple/shadow wrappers | `prefix27_eyegore_drawing_all_graphics_pause_and_clips_exclude_children` |
| Eye-statue caller and arrow trigger: `05e62b`, `05e633` | `prefix27_eye_statue_all_arrow_trigger_paths_exclude_children` |
| Terrorpin caller/states and hammer guards: `1eb26f`, `1eb2a7`, `1eb30e`, `1eb3a3` | `prefix27_terrorpin_both_states_all_caller_paths_exclude_children`; `prefix27_terrorpin_hammer_all_guards_overlap_and_toggle_paths_exclude_children` |
| Hinox draw prefix/JMP and default shadow load: `06a1f9` | `prefix27_hinox_draw_all_graphics_and_clips_exclude_children` |
| Thrown item `$ec` caller: `06aae0` | `prefix27_thrown_item_all_sorting_size_state_and_world_paths_exclude_children` |
| Heart piece `$eb` prefix through `05f058` | `prefix27_heart_piece_prefix_all_initialization_active_and_collision_paths_exclude_children` |
| Antifairy caller: `06a50c`; actual Goriya/antifairy draw wrapper | `prefix27_antifairy_all_damage_and_magic_paths_exclude_children`; existing Goriya draw fixtures |
| Shared red-spear/archer walking and animation: `05ceaa`, `05cee2` | `prefix27_guard_walking_all_timer_collision_direction_and_animation_paths_exclude_children` |
| NPC/shop draw callers: `05f91b`, `0dc3e1`, `05e3dc`, `1af88c`, `1ef4ce` | `prefix27_npc_and_shop_draw_callers_all_frames_and_clips_exclude_children` |
| Sweeping-lady, Kakariko-boy and running-boy bounces; actual bank-5 direction entries | `prefix27_npc_bank_six_bounces_and_sweeping_wrapper_exclude_children`; existing bank-5 direction fixtures |

Sword-beam movement now follows ROM Y-then-X order. Helmasaur's correction
count is a ROM-exact behavior correction. Both retain the baseline CPU
picture hashes. No arrival, upload, spotlight, polling-price or P2 change is
used. The legacy non-lag-free predicate charge is retained where the marker
body is disabled; an obsolete unmarked collision scan is removed.

The generator adds **297 blocks (5,977 → 6,274)**, preserving every old tuple.
Across all 282 selected rows, expanded marker work changes by **+610,078**
clocks, scalar work by **-102,480**, and complete instruction work by
**+507,598**. Every affected after-marker count and clock total is at or below
the source at all **8,655 affected per-PC rows**. Signed routine partitions
conserve the same totals. `generated-block-preservation.json`,
`instruction-conservation.json` and `per-pc-conservation.csv` retain the proof.

### Negative-family overcount audit

The audit covers every negative family residual on all 282 profiled rows:
**58** in total, including the **32** on the original missing-row set.
All are real overcharges, not family attribution aliases. No negative family
residual remains. `negative-family-audit.csv/json` records every row, its
before/after residual, finding and fixture.

| Family | Negative rows | ROM finding and correction | Fixture |
| --- | ---: | --- | --- |
| Crystal switch `$1e/$09` | 25 | Oam_AllocateDeferToPlayer duplicated predicate work already owned by `06ead1/06eae8`. All audited residuals become zero. | `prefix27_defer_all_relative_branches_charge_each_helper_once` |
| Fake sword `$e8/$09` | 5 | Same duplicate; **-126 → +104**. Remaining caller work is unpriced. | Same defer fixture |
| Storyteller `$28/$09` | 1 | Same duplicate; **-388 → +948**. | Same defer fixture |
| Blue/green archer `$46/$47, $09` | 7 | Actual entries are `05c6e0/05ca10`; the full entries add an unexecuted 16+78 clocks per draw. All audited residuals become zero. | `prefix27_archer_actual_head_and_body_entries_skip_only_the_rom_prefixes` |
| Helmasaur `$92/$09` | 20 | ROM `1e8a7b` loads **15**, for **16** correction visits. Native visited 17. Savings follow actual clip paths; at 378,096/098/105 the **1,554**-clock correction changes **-1,316 → +238**. | `prefix27_helmasaur_tail_correction_visits_exactly_sixteen_entries` |

The overcount corrections expose seven prefix rows:
172,413; 378,077/080/092/093/094; and 403,249.
At **172,413**, the corrected native instruction price is **298,550** against
source **298,528**, leaving **-22** and only a **two-clock** diagnostic shortfall.
There is no missing instruction owner to price. This explains the one-count
increase before 228,499; it is left unfitted. At **378,093**, the remaining
1,324 instruction clocks are below the 1,364-clock boundary, so the exposed
row is (d). `newly-exposed-prefix-rows.csv/json` retains all seven findings.

### Remaining blockers through 461,989

| Family | Remaining rows with nonzero family work | Signed sum | Largest row |
| --- | ---: | ---: | ---: |
| Crystal switch `$1e` | 2 | +2,200 | +1,100 |
| Spike `$8a` | 1 | +702 | +702 |
| Red Eyegore `$84` | 1 | +96 | +96 |
| Green Eyegore `$83` | 1 | +1,166 | +1,166 |
| Eye statue `$38` | 1 | +104 | +104 |
| Terrorpin (actual dispatch) `$8e` | 2 | +1,752 | +1,584 |
| Heart piece `$eb` | 16 | +7,570 | +718 |
| Thrown item `$ec` | 4 | +15,684 | +10,178 |
| Red spear guard `$48` | 5 | +10,174 | +7,338 |
| Hinox `$11` | 10 | +14,928 | +2,298 |
| Antifairy `$15` | 0 | +0 | +0 |

These are remaining *signed* family totals on missing rows, not new prices.
The heart-piece pickup/bounce suffix, thrown-item children, Hinox behavior,
remaining guard states, Eyegore recoil/damage and Terrorpin damage children
are not claimed complete. Further cohort-C owners, including Arrghus/Arrgi,
also remain. The table below gives every remaining row.
`remaining-blockers.csv/json` contains the same inventory; the full concrete
routine/object/sprite vectors are retained beside it.

| Iteration | Class | Diagnostic clocks short | Signed nonpoll gap | Blocker |
| --- | --- | ---: | ---: | --- |
| 34,318 | c | 262,698 | +18,222 | Upload caller/receiver/return placement unresolved; mechanism unchanged. other signed caller/child work +18,222. |
| 48,556 | c | 67,802 | +18,176 | Upload caller/receiver/return placement unresolved; mechanism unchanged. other signed caller/child work +18,176. |
| 95,127 | c | 153,534 | +18,338 | Upload caller/receiver/return placement unresolved; mechanism unchanged. other signed caller/child work +18,338. |
| 101,031 | d | 160,288 | +14,600 | Spotlight wait/arrival unresolved (1,429 source V-counter reads); mechanism unchanged. sprite $3b/$09 +2,588; other signed caller/child work +12,012. |
| 109,000 | c | 110,360 | +26,464 | Upload caller/receiver/return placement unresolved; mechanism unchanged. other signed caller/child work +26,464. |
| 109,016 | d | 151,160 | +2,302 | Spotlight wait/arrival unresolved (1,428 source V-counter reads); mechanism unchanged. sprite $c8/$09 +1,600; sprite $72/$09 +724; other signed caller/child work -22. |
| 129,746 | c | 155,150 | +31,212 | Upload caller/receiver/return placement unresolved; mechanism unchanged. other signed caller/child work +31,212. |
| 146,148 | c | 38,076 | +75,156 | Upload caller/receiver/return placement unresolved; mechanism unchanged. other signed caller/child work +75,156. |
| 146,786 | d | 2,700 | -22 | Instruction work cannot cover the pacing boundary: -22 nonpoll gap vs 2,700 short. other signed caller/child work -22. |
| 146,790 | d | 2,602 | -22 | Instruction work cannot cover the pacing boundary: -22 nonpoll gap vs 2,602 short. other signed caller/child work -22. |
| 146,794 | d | 5,012 | -22 | Instruction work cannot cover the pacing boundary: -22 nonpoll gap vs 5,012 short. other signed caller/child work -22. |
| 146,798 | d | 7,822 | -22 | Instruction work cannot cover the pacing boundary: -22 nonpoll gap vs 7,822 short. other signed caller/child work -22. |
| 146,802 | d | 10,414 | -22 | Instruction work cannot cover the pacing boundary: -22 nonpoll gap vs 10,414 short. other signed caller/child work -22. |
| 146,806 | d | 12,286 | -22 | Instruction work cannot cover the pacing boundary: -22 nonpoll gap vs 12,286 short. other signed caller/child work -22. |
| 146,810 | d | 14,732 | -22 | Instruction work cannot cover the pacing boundary: -22 nonpoll gap vs 14,732 short. other signed caller/child work -22. |
| 150,378 | a | 21,708 | +49,376 | Remaining instruction owners: sprite $ec/$06 +10,178; other signed caller/child work +39,198. |
| 154,637 | a | 380 | +4,720 | Remaining instruction owners: sprite $55/$09 +2,004; sprite $52/$09 +902; sprite $eb/$09 +516; other signed caller/child work +1,298. |
| 158,519 | a | 224 | +2,896 | Remaining instruction owners: sprite $55/$09 +1,076; sprite $eb/$09 +516; other signed caller/child work +1,304. |
| 158,558 | a | 1,798 | +2,602 | Remaining instruction owners: sprite $55/$09 +782; sprite $eb/$09 +516; other signed caller/child work +1,304. |
| 158,729 | a | 138 | +496 | Remaining instruction owners: sprite $eb/$09 +294; sprite $55/$09 +218; other signed caller/child work -16. |
| 160,109 | a | 720 | +918 | Remaining instruction owners: sprite $37/$09 +848; sprite $58/$09 +92; other signed caller/child work -22. |
| 160,154 | a | 906 | +1,428 | Remaining instruction owners: sprite $55/$09 +500; other signed caller/child work +928. |
| 161,123 | a | 5,586 | +6,178 | Remaining instruction owners: sprite $55/$09 +3,544; sprite $eb/$09 +718; other signed caller/child work +1,916. |
| 162,301 | a | 1,386 | +7,214 | Remaining instruction owners: sprite $52/$09 +3,636; sprite $eb/$09 +294; other signed caller/child work +3,284. |
| 162,819 | a | 6,524 | +6,948 | Remaining instruction owners: sprite $52/$09 +3,636; sprite $eb/$09 +294; other signed caller/child work +3,018. |
| 168,346 | d | 83,650 | +8,290 | Instruction work cannot cover the pacing boundary: +8,290 nonpoll gap vs 83,650 short. sprite $55/$09 +500; other signed caller/child work +7,790. |
| 171,309 | d | 21,452 | +8,290 | Instruction work cannot cover the pacing boundary: +8,290 nonpoll gap vs 21,452 short. sprite $2b/$09 +1,684; other signed caller/child work +6,606. |
| 171,310 | d | 63,452 | +1,662 | Instruction work cannot cover the pacing boundary: +1,662 nonpoll gap vs 63,452 short. sprite $2b/$09 +1,684; other signed caller/child work -22. |
| 172,380 | d | 143,322 | -22 | Instruction work cannot cover the pacing boundary: -22 nonpoll gap vs 143,322 short. other signed caller/child work -22. |
| 172,413 | d | 2 | -22 | Instruction work cannot cover the pacing boundary: -22 nonpoll gap vs 2 short. other signed caller/child work -22. |
| 174,033 | a | 665 | +2,210 | Remaining instruction owners: sprite $d4/$09 +888; sprite $d9/$09 +526; other signed caller/child work +796. |
| 174,034 | a | 1,185 | +2,498 | Remaining instruction owners: sprite $d4/$09 +888; sprite $d9/$09 +526; other signed caller/child work +1,084. |
| 174,280 | c | 112,958 | +26,464 | Upload caller/receiver/return placement unresolved; mechanism unchanged. other signed caller/child work +26,464. |
| 174,296 | d | 152,714 | +2,302 | Spotlight wait/arrival unresolved (1,427 source V-counter reads); mechanism unchanged. sprite $c8/$09 +1,600; sprite $72/$09 +724; other signed caller/child work -22. |
| 180,178 | a | 15,372 | +20,086 | Remaining instruction owners: sprite $48/$09 +7,338; sprite $ec/$06 +1,428; sprite $c5/$09 +208; other signed caller/child work +11,112. |
| 180,182 | a | 9,084 | +15,216 | Remaining instruction owners: sprite $ec/$06 +1,152; sprite $48/$09 +382; sprite $c5/$09 +208; other signed caller/child work +13,474. |
| 181,568 | a | 724 | +2,692 | Remaining instruction owners: sprite $c4/$09 +920; sprite $dc/$09 +600; sprite $d9/$09 +600; other signed caller/child work +168. |
| 186,606 | a | 242 | +1,060 | Remaining instruction owners: sprite $17/$09 +892; other signed caller/child work +168. |
| 186,608 | a | 830 | +1,060 | Remaining instruction owners: sprite $17/$09 +892; other signed caller/child work +168. |
| 186,614 | a | 392 | +1,060 | Remaining instruction owners: sprite $17/$09 +892; other signed caller/child work +168. |
| 186,616 | a | 942 | +1,060 | Remaining instruction owners: sprite $17/$09 +892; other signed caller/child work +168. |
| 186,618 | a | 50 | +1,060 | Remaining instruction owners: sprite $17/$09 +892; other signed caller/child work +168. |
| 186,620 | a | 488 | +1,060 | Remaining instruction owners: sprite $17/$09 +892; other signed caller/child work +168. |
| 186,622 | a | 112 | +1,060 | Remaining instruction owners: sprite $17/$09 +892; other signed caller/child work +168. |
| 186,624 | a | 706 | +1,060 | Remaining instruction owners: sprite $17/$09 +892; other signed caller/child work +168. |
| 186,626 | a | 174 | +1,060 | Remaining instruction owners: sprite $17/$09 +892; other signed caller/child work +168. |
| 186,664 | a | 1,014 | +2,898 | Remaining instruction owners: sprite $33/$09 +1,234; sprite $17/$09 +892; sprite $79/$09 +604; other signed caller/child work +168. |
| 186,668 | a | 964 | +2,898 | Remaining instruction owners: sprite $33/$09 +1,234; sprite $17/$09 +892; sprite $79/$09 +604; other signed caller/child work +168. |
| 186,670 | a | 482 | +2,898 | Remaining instruction owners: sprite $33/$09 +1,234; sprite $17/$09 +892; sprite $79/$09 +604; other signed caller/child work +168. |
| 186,672 | a | 892 | +2,898 | Remaining instruction owners: sprite $33/$09 +1,234; sprite $17/$09 +892; sprite $79/$09 +604; other signed caller/child work +168. |
| 186,684 | a | 2,002 | +2,898 | Remaining instruction owners: sprite $33/$09 +1,234; sprite $17/$09 +892; sprite $79/$09 +604; other signed caller/child work +168. |
| 186,686 | a | 1,890 | +2,898 | Remaining instruction owners: sprite $33/$09 +1,234; sprite $17/$09 +892; sprite $79/$09 +604; other signed caller/child work +168. |
| 186,690 | a | 1,106 | +2,898 | Remaining instruction owners: sprite $33/$09 +1,234; sprite $17/$09 +892; sprite $79/$09 +604; other signed caller/child work +168. |
| 186,694 | a | 1,820 | +2,898 | Remaining instruction owners: sprite $33/$09 +1,234; sprite $17/$09 +892; sprite $79/$09 +604; other signed caller/child work +168. |
| 186,696 | a | 196 | +2,898 | Remaining instruction owners: sprite $33/$09 +1,234; sprite $17/$09 +892; sprite $79/$09 +604; other signed caller/child work +168. |
| 186,698 | a | 1,234 | +2,898 | Remaining instruction owners: sprite $33/$09 +1,234; sprite $17/$09 +892; sprite $79/$09 +604; other signed caller/child work +168. |
| 187,779 | a | 3,170 | +3,360 | Remaining instruction owners: other signed caller/child work +3,360. |
| 189,344 | a | 5,586 | +6,014 | Remaining instruction owners: sprite $c2/$09 +1,192; sprite $27/$09 +192; other signed caller/child work +4,630. |
| 189,368 | a | 3,536 | +4,860 | Remaining instruction owners: sprite $c2/$09 +1,192; sprite $ad/$09 +840; other signed caller/child work +2,828. |
| 189,923 | d | 224,566 | +1,170 | Spotlight wait/arrival unresolved (1,423 source V-counter reads); mechanism unchanged. sprite $c2/$09 +1,192; other signed caller/child work -22. |
| 190,815 | a | 1,836 | +2,220 | Remaining instruction owners: sprite $c2/$09 +894; sprite $27/$09 +192; other signed caller/child work +1,134. |
| 194,151 | c | 19,768 | +43,808 | Upload caller/receiver/return placement unresolved; mechanism unchanged. object $1c1 +3,316; object $1aa +2,368; object $1ab +2,288; other signed caller/child work +31,014. |
| 194,994 | a | 3,508 | +6,374 | Remaining instruction owners: sprite $87/$09 +3,024; sprite $18/$09 +230; sprite $86/$09 +216; other signed caller/child work +2,904. |
| 196,051 | a | 7,518 | +8,090 | Remaining instruction owners: other signed caller/child work +8,090. |
| 198,239 | a | 540 | +3,990 | Remaining instruction owners: sprite $a7/$09 +1,760; sprite $18/$09 +882; sprite $e4/$09 +248; other signed caller/child work +1,100. |
| 198,240 | a | 124 | +3,872 | Remaining instruction owners: sprite $a7/$09 +1,760; sprite $18/$09 +882; sprite $e4/$09 +248; other signed caller/child work +982. |
| 209,808 | a | 5,088 | +15,104 | Remaining instruction owners: other signed caller/child work +15,104. |
| 215,857 | a | 714 | +982 | Remaining instruction owners: other signed caller/child work +982. |
| 215,858 | a | 716 | +982 | Remaining instruction owners: other signed caller/child work +982. |
| 215,859 | a | 714 | +982 | Remaining instruction owners: other signed caller/child work +982. |
| 217,220 | a | 714 | +1,100 | Remaining instruction owners: other signed caller/child work +1,100. |
| 217,221 | a | 298 | +982 | Remaining instruction owners: other signed caller/child work +982. |
| 217,222 | a | 714 | +982 | Remaining instruction owners: other signed caller/child work +982. |
| 225,257 | a | 3,242 | +4,964 | Remaining instruction owners: object $239 +854; object $232 +352; other signed caller/child work +3,758. |
| 242,100 | d | 44,504 | -22 | Instruction work cannot cover the pacing boundary: -22 nonpoll gap vs 44,504 short. other signed caller/child work -22. |
| 242,103 | a | 476 | +928 | Remaining instruction owners: other signed caller/child work +928. |
| 242,122 | a | 728 | +1,032 | Remaining instruction owners: other signed caller/child work +1,032. |
| 253,040 | c | 3,568 | +28,930 | Upload caller/receiver/return placement unresolved; mechanism unchanged. other signed caller/child work +28,930. |
| 254,154 | d | 228,538 | +272 | Spotlight wait/arrival unresolved (1,428 source V-counter reads); mechanism unchanged. sprite $eb/$09 +294; other signed caller/child work -22. |
| 255,517 | a | 172 | +1,942 | Remaining instruction owners: sprite $48/$09 +818; other signed caller/child work +1,124. |
| 255,521 | a | 780 | +2,140 | Remaining instruction owners: sprite $48/$09 +818; other signed caller/child work +1,322. |
| 255,525 | a | 466 | +2,140 | Remaining instruction owners: sprite $48/$09 +818; other signed caller/child work +1,322. |
| 255,529 | a | 582 | +1,322 | Remaining instruction owners: other signed caller/child work +1,322. |
| 255,572 | a | 1,022 | +2,556 | Remaining instruction owners: sprite $33/$09 +1,234; other signed caller/child work +1,322. |
| 256,490 | c | 56,980 | +41,164 | Upload caller/receiver/return placement unresolved; mechanism unchanged. object $1db +4,476; object $31e +2,800; object $232 +1,056; other signed caller/child work +32,336. |
| 281,028 | a | 7,050 | +8,550 | Remaining instruction owners: sprite $1c/$09 +460; other signed caller/child work +8,090. |
| 282,475 | a | 160 | +740 | Remaining instruction owners: sprite $41/$05 +718; sprite $1c/$09 +460; other signed caller/child work -438. |
| 294,833 | a | 9,554 | +13,308 | Remaining instruction owners: sprite $4a/$06 +7,492; sprite $11/$09 +2,274; sprite $22/$09 +1,918; other signed caller/child work +1,624. |
| 294,835 | a | 10,282 | +17,876 | Remaining instruction owners: sprite $4a/$06 +7,594; sprite $41/$09 +2,728; sprite $11/$09 +2,274; other signed caller/child work +3,362. |
| 294,837 | a | 12,132 | +14,662 | Remaining instruction owners: sprite $4a/$06 +6,420; sprite $41/$09 +2,728; sprite $11/$09 +2,274; other signed caller/child work +1,322. |
| 294,844 | d | 7,502 | +7,442 | Instruction work cannot cover the pacing boundary: +7,442 nonpoll gap vs 7,502 short. sprite $4a/$06 +7,372; sprite $11/$09 +92; other signed caller/child work -22. |
| 295,089 | d | 147,092 | +2,302 | Spotlight wait/arrival unresolved (1,427 source V-counter reads); mechanism unchanged. sprite $c8/$09 +1,600; sprite $72/$09 +724; other signed caller/child work -22. |
| 297,585 | a | 7,012 | +7,944 | Remaining instruction owners: sprite $55/$09 +3,920; sprite $45/$09 +1,104; sprite $eb/$09 +516; other signed caller/child work +2,404. |
| 297,638 | a | 1,442 | +6,706 | Remaining instruction owners: sprite $55/$09 +1,776; sprite $45/$09 +1,104; sprite $eb/$09 +516; other signed caller/child work +3,310. |
| 297,863 | d | 130,988 | +5,974 | Spotlight wait/arrival unresolved (1,426 source V-counter reads); mechanism unchanged. sprite $bb/$09 +5,048; sprite $28/$09 +948; other signed caller/child work -22. |
| 299,434 | d | 147,848 | +2,302 | Spotlight wait/arrival unresolved (1,427 source V-counter reads); mechanism unchanged. sprite $c8/$09 +1,600; sprite $72/$09 +724; other signed caller/child work -22. |
| 299,510 | c | 232,210 | +18,442 | Upload caller/receiver/return placement unresolved; mechanism unchanged. other signed caller/child work +18,442. |
| 304,890 | a | 11,756 | +15,634 | Remaining instruction owners: sprite $0f/$06 +11,456; sprite $10/$09 +4,010; other signed caller/child work +168. |
| 307,022 | d | 234,416 | +2,666 | Spotlight wait/arrival unresolved (1,405 source V-counter reads); mechanism unchanged. sprite $58/$09 +2,582; other signed caller/child work +84. |
| 307,040 | c | 123,564 | +46,014 | Upload caller/receiver/return placement unresolved; mechanism unchanged. object $1dc +12,810; other signed caller/child work +33,204. |
| 310,875 | a | 10,058 | +20,736 | Remaining instruction owners: sprite $0f/$06 +11,518; sprite $10/$09 +4,010; sprite $55/$09 +676; other signed caller/child work +4,532. |
| 310,980 | d | 171,138 | +7,790 | Instruction work cannot cover the pacing boundary: +7,790 nonpoll gap vs 171,138 short. other signed caller/child work +7,790. |
| 310,982 | d | 84,542 | +710 | Instruction work cannot cover the pacing boundary: +710 nonpoll gap vs 84,542 short. other signed caller/child work +710. |
| 313,078 | a | 9,672 | +10,304 | Remaining instruction owners: sprite $19/$09 +8,560; sprite $41/$09 +716; other signed caller/child work +1,028. |
| 313,255 | a | 13,568 | +13,672 | Remaining instruction owners: sprite $19/$09 +9,850; sprite $41/$09 +2,704; other signed caller/child work +1,118. |
| 317,588 | a | 3,970 | +8,098 | Remaining instruction owners: sprite $45/$09 +92; other signed caller/child work +8,006. |
| 322,193 | a | 9,984 | +10,876 | Remaining instruction owners: sprite $22/$09 +2,170; sprite $11/$09 +1,150; sprite $4a/$09 +1,082; other signed caller/child work +4,634. |
| 327,434 | a | 508 | +574 | Remaining instruction owners: other signed caller/child work +574. |
| 336,035 | a | 28 | +574 | Remaining instruction owners: other signed caller/child work +574. |
| 336,046 | a | 152 | +470 | Remaining instruction owners: other signed caller/child work +470. |
| 337,925 | a | 2,192 | +5,062 | Remaining instruction owners: sprite $26/$09 +1,876; other signed caller/child work +3,186. |
| 338,240 | a | 1,166 | +6,626 | Remaining instruction owners: sprite $26/$09 +2,340; sprite $1e/$09 +1,100; other signed caller/child work +3,186. |
| 338,673 | a | 8,148 | +8,992 | Remaining instruction owners: sprite $26/$09 +8,144; other signed caller/child work +848. |
| 344,246 | a | 4,258 | +5,390 | Remaining instruction owners: sprite $8a/$08 +702; sprite $8e/$08 +168; sprite $1c/$08 +142; other signed caller/child work +4,378. |
| 347,334 | a | 3,634 | +8,992 | Remaining instruction owners: sprite $26/$09 +8,144; other signed caller/child work +848. |
| 348,203 | a | 3,224 | +5,096 | Remaining instruction owners: other signed caller/child work +5,096. |
| 353,572 | a | 2,890 | +4,968 | Remaining instruction owners: other signed caller/child work +4,968. |
| 354,592 | a | 132 | +470 | Remaining instruction owners: other signed caller/child work +470. |
| 354,595 | d | 588 | +574 | Instruction work cannot cover the pacing boundary: +574 nonpoll gap vs 588 short. other signed caller/child work +574. |
| 354,619 | a | 314 | +574 | Remaining instruction owners: other signed caller/child work +574. |
| 356,806 | a | 1,402 | +6,654 | Remaining instruction owners: sprite $ec/$0a +2,926; sprite $1e/$09 +1,100; sprite $d9/$09 +300; other signed caller/child work +2,328. |
| 367,327 | a | 4,494 | +5,128 | Remaining instruction owners: sprite $83/$09 +1,166; sprite $38/$09 +104; sprite $84/$09 +96; other signed caller/child work +3,762. |
| 370,567 | a | 1,276 | +1,510 | Remaining instruction owners: other signed caller/child work +1,510. |
| 372,814 | a | 4,278 | +5,814 | Remaining instruction owners: sprite $8e/$09 +1,584; sprite $da/$09 +108; other signed caller/child work +4,122. |
| 378,071 | a | 842 | +2,132 | Remaining instruction owners: sprite $92/$09 +1,562; other signed caller/child work +570. |
| 378,077 | a | 232 | +1,748 | Remaining instruction owners: sprite $92/$09 +102; other signed caller/child work +1,646. |
| 378,080 | a | 188 | +1,188 | Remaining instruction owners: sprite $92/$09 +102; other signed caller/child work +1,086. |
| 378,092 | a | 802 | +1,324 | Remaining instruction owners: sprite $92/$09 +238; other signed caller/child work +1,086. |
| 378,093 | d | 1,364 | +1,324 | Instruction work cannot cover the pacing boundary: +1,324 nonpoll gap vs 1,364 short. sprite $92/$09 +238; other signed caller/child work +1,086. |
| 378,094 | a | 980 | +1,324 | Remaining instruction owners: sprite $92/$09 +238; other signed caller/child work +1,086. |
| 378,096 | a | 1,100 | +2,166 | Remaining instruction owners: sprite $92/$09 +238; other signed caller/child work +1,928. |
| 378,098 | a | 656 | +1,198 | Remaining instruction owners: sprite $92/$09 +238; other signed caller/child work +960. |
| 378,105 | a | 154 | +1,198 | Remaining instruction owners: sprite $92/$09 +238; other signed caller/child work +960. |
| 378,541 | a | 4,908 | +6,364 | Remaining instruction owners: sprite $92/$09 +4,454; other signed caller/child work +1,910. |
| 378,830 | a | 1,068 | +5,414 | Remaining instruction owners: sprite $92/$09 +4,454; other signed caller/child work +960. |
| 380,956 | d | 19,320 | +766 | Instruction work cannot cover the pacing boundary: +766 nonpoll gap vs 19,320 short. other signed caller/child work +766. |
| 383,633 | a | 5,726 | +7,198 | Remaining instruction owners: other signed caller/child work +7,198. |
| 389,654 | a | 38,208 | +47,408 | Remaining instruction owners: other signed caller/child work +47,408. |
| 390,579 | a | 14,912 | +22,044 | Remaining instruction owners: sprite $12/$09 +9,046; sprite $4a/$06 +7,474; sprite $11/$09 +2,274; other signed caller/child work +1,720. |
| 390,877 | a | 5,784 | +12,380 | Remaining instruction owners: sprite $4a/$06 +6,540; sprite $11/$09 +2,298; sprite $22/$09 +1,918; other signed caller/child work +1,624. |
| 390,887 | d | 6,968 | +6,490 | Instruction work cannot cover the pacing boundary: +6,490 nonpoll gap vs 6,968 short. sprite $4a/$06 +6,420; sprite $11/$09 +92; other signed caller/child work -22. |
| 400,064 | a | 9,414 | +13,398 | Remaining instruction owners: sprite $41/$09 +6,098; sprite $22/$09 +4,322; sprite $11/$09 +1,100; other signed caller/child work +796. |
| 401,366 | a | 5,188 | +5,380 | Remaining instruction owners: sprite $aa/$09 +3,132; sprite $d3/$09 +1,530; sprite $41/$09 +344; other signed caller/child work +374. |
| 401,471 | d | 137,418 | +5,410 | Spotlight wait/arrival unresolved (1,421 source V-counter reads); mechanism unchanged. sprite $bb/$09 +5,432; other signed caller/child work -22. |
| 401,612 | d | 230,738 | +2,694 | Spotlight wait/arrival unresolved (1,426 source V-counter reads); mechanism unchanged. sprite $bb/$09 +2,716; other signed caller/child work -22. |
| 401,764 | a | 2,044 | +2,962 | Remaining instruction owners: sprite $aa/$09 +2,588; other signed caller/child work +374. |
| 403,249 | d | 228,489 | +2,070 | Spotlight wait/arrival unresolved (1,417 source V-counter reads); mechanism unchanged. sprite $b5/$09 +2,092; other signed caller/child work -22. |
| 403,304 | d | 237,646 | +9,198 | Spotlight wait/arrival unresolved (1,380 source V-counter reads); mechanism unchanged. sprite $0e/$09 +3,526; sprite $12/$09 +2,556; sprite $22/$09 +1,932; other signed caller/child work +84. |
| 405,837 | c | 338,940 | +28,930 | Upload caller/receiver/return placement unresolved; mechanism unchanged. other signed caller/child work +28,930. |
| 407,899 | d | 243,696 | +8,402 | Spotlight wait/arrival unresolved (1,425 source V-counter reads); mechanism unchanged. sprite $aa/$09 +3,132; sprite $a9/$09 +3,090; sprite $12/$09 +1,116; other signed caller/child work +84. |
| 410,098 | c | 280,306 | +27,798 | Upload caller/receiver/return placement unresolved; mechanism unchanged. object $237 +328; other signed caller/child work +27,470. |
| 411,310 | a | 2,082 | +2,720 | Remaining instruction owners: sprite $d2/$09 +2,076; sprite $eb/$09 +516; other signed caller/child work +128. |
| 411,514 | a | 4,368 | +5,364 | Remaining instruction owners: sprite $d2/$09 +4,474; sprite $eb/$09 +516; other signed caller/child work +374. |
| 411,545 | a | 464 | +2,720 | Remaining instruction owners: sprite $d2/$09 +2,076; sprite $eb/$09 +516; other signed caller/child work +128. |
| 411,549 | a | 1,218 | +3,012 | Remaining instruction owners: sprite $d2/$09 +2,076; sprite $eb/$09 +516; sprite $0c/$09 +292; other signed caller/child work +128. |
| 411,555 | a | 532 | +2,720 | Remaining instruction owners: sprite $d2/$09 +2,076; sprite $eb/$09 +516; other signed caller/child work +128. |
| 411,556 | a | 2,478 | +2,720 | Remaining instruction owners: sprite $d2/$09 +2,076; sprite $eb/$09 +516; other signed caller/child work +128. |
| 427,806 | a | 706 | +6,728 | Remaining instruction owners: sprite $9a/$08 +322; sprite $81/$08 +126; sprite $c6/$09 +108; object $232 +704; object $31d +544; object $331 +484; other signed caller/child work +4,126. |
| 435,694 | a | 1,248 | +6,098 | Remaining instruction owners: sprite $21/$09 +5,568; sprite $23/$09 +232; sprite $9a/$09 +178; other signed caller/child work +120. |
| 435,695 | a | 452 | +6,098 | Remaining instruction owners: sprite $21/$09 +5,568; sprite $23/$09 +232; sprite $9a/$09 +178; other signed caller/child work +120. |
| 435,696 | a | 692 | +6,098 | Remaining instruction owners: sprite $21/$09 +5,568; sprite $23/$09 +232; sprite $9a/$09 +178; other signed caller/child work +120. |
| 440,926 | a | 2,612 | +6,484 | Remaining instruction owners: sprite $9a/$08 +322; sprite $81/$08 +126; sprite $c6/$09 +108; object $232 +704; object $31d +544; object $331 +484; other signed caller/child work +3,882. |
| 451,842 | a | 187,580 | +222,524 | Remaining instruction owners: sprite $21/$09 +5,568; sprite $24/$09 +232; sprite $9a/$09 +178; other signed caller/child work +216,546. |
| 452,255 | a | 1,998 | +2,270 | Remaining instruction owners: sprite $37/$09 +848; sprite $9a/$09 +730; sprite $24/$09 +464; other signed caller/child work +120. |
| 456,721 | a | 15,294 | +98,616 | Remaining instruction owners: sprite $8d/$08 +91,954; sprite $8c/$08 +3,480; object $301 +184; other signed caller/child work +2,998. |
| 456,849 | a | 70,992 | +78,326 | Remaining instruction owners: sprite $8c/$09 +46,602; sprite $8d/$09 +12,194; other signed caller/child work +19,530. |
| 456,855 | a | 80,820 | +82,566 | Remaining instruction owners: sprite $8c/$09 +46,616; sprite $8d/$09 +12,194; other signed caller/child work +23,756. |
| 456,856 | a | 83,004 | +84,556 | Remaining instruction owners: sprite $8c/$09 +46,384; sprite $8d/$09 +12,194; other signed caller/child work +25,978. |
| 456,857 | a | 83,004 | +84,478 | Remaining instruction owners: sprite $8c/$09 +46,210; sprite $8d/$09 +12,194; other signed caller/child work +26,074. |
| 456,858 | a | 82,820 | +85,904 | Remaining instruction owners: sprite $8c/$09 +46,296; sprite $8d/$09 +12,194; other signed caller/child work +27,414. |
| 456,859 | a | 81,256 | +86,980 | Remaining instruction owners: sprite $8c/$09 +46,412; sprite $8d/$09 +12,194; other signed caller/child work +28,374. |
| 456,860 | a | 83,004 | +88,204 | Remaining instruction owners: sprite $8c/$09 +46,296; sprite $8d/$09 +12,194; other signed caller/child work +29,714. |
| 456,861 | a | 83,004 | +89,382 | Remaining instruction owners: sprite $8c/$09 +46,514; sprite $8d/$09 +12,194; other signed caller/child work +30,674. |
| 456,862 | a | 83,004 | +89,252 | Remaining instruction owners: sprite $8c/$09 +46,296; sprite $8d/$09 +12,194; other signed caller/child work +30,762. |
| 456,863 | a | 80,788 | +91,358 | Remaining instruction owners: sprite $8c/$09 +46,560; sprite $8d/$09 +12,194; other signed caller/child work +32,604. |
| 456,864 | a | 82,586 | +89,284 | Remaining instruction owners: sprite $8c/$09 +46,328; sprite $8d/$09 +12,194; other signed caller/child work +30,762. |
| 456,865 | a | 78,702 | +96,816 | Remaining instruction owners: sprite $8c/$09 +46,658; sprite $8d/$09 +12,194; other signed caller/child work +37,964. |
| 456,900 | a | 64,096 | +78,384 | Remaining instruction owners: sprite $8c/$09 +46,560; sprite $8d/$09 +12,194; other signed caller/child work +19,630. |
| 456,901 | a | 75,900 | +77,268 | Remaining instruction owners: sprite $8c/$09 +46,778; sprite $8d/$09 +12,194; other signed caller/child work +18,296. |
| 456,902 | a | 76,368 | +77,018 | Remaining instruction owners: sprite $8c/$09 +46,560; sprite $8d/$09 +12,194; other signed caller/child work +18,264. |
| 456,903 | a | 74,462 | +79,294 | Remaining instruction owners: sprite $8c/$09 +46,760; sprite $8d/$09 +12,194; other signed caller/child work +20,340. |
| 456,904 | a | 76,368 | +79,030 | Remaining instruction owners: sprite $8c/$09 +46,528; sprite $8d/$09 +12,194; other signed caller/child work +20,308. |
| 456,905 | a | 76,368 | +81,212 | Remaining instruction owners: sprite $8c/$09 +46,634; sprite $8d/$09 +12,194; other signed caller/child work +22,384. |
| 456,906 | a | 76,368 | +80,962 | Remaining instruction owners: sprite $8c/$09 +46,416; sprite $8d/$09 +12,194; other signed caller/child work +22,352. |
| 456,907 | a | 74,620 | +83,670 | Remaining instruction owners: sprite $8c/$09 +46,220; sprite $8d/$09 +12,194; other signed caller/child work +25,256. |
| 457,143 | a | 58,342 | +59,788 | Remaining instruction owners: sprite $8c/$09 +46,680; sprite $8d/$09 +11,256; sprite $8d/$06 +194; other signed caller/child work +1,658. |
| 457,147 | a | 58,508 | +59,560 | Remaining instruction owners: sprite $8c/$09 +46,452; sprite $8d/$09 +11,256; sprite $8d/$06 +194; other signed caller/child work +1,658. |
| 457,151 | a | 58,402 | +59,948 | Remaining instruction owners: sprite $8c/$09 +46,568; sprite $8d/$09 +11,256; sprite $8d/$06 +194; other signed caller/child work +1,930. |
| 457,309 | a | 73,956 | +76,992 | Remaining instruction owners: sprite $8c/$09 +46,472; sprite $8d/$09 +11,256; other signed caller/child work +19,264. |
| 457,445 | a | 68,470 | +75,738 | Remaining instruction owners: sprite $8c/$09 +46,626; sprite $8d/$09 +10,318; other signed caller/child work +18,794. |

### Complete ROM and CPU-picture admission

The supplied eight-slice ROM helper passes **1,473,123 iterations** with
**unexplained 0**, **1,530 missing / 0 excess / 0 false positives**.
`clean-admission-commands.json`, `rom-gate-final.txt`,
`rom-counts-final.txt` and `admission-clean/rom.csv` retain the final run.
Both final binary and final source identities are checked across all gates.

| Missing before iteration | ROM before → after | Live before → after |
| --- | ---: | ---: |
| 94,334 | 2 → 2 | 2 → 2 |
| 141,568 | 7 → 7 | 7 → 7 |
| 228,499 | 73 → 74 | 73 → 74 |
| 380,243 | 153 → 135 | 153 → 144 |
| 461,989 | 209 → 190 | 209 → 199 |

The independent eight-slice `--cpu-video` gate passes all **1,473,123
iterations / unexplained 0** and retains **1,462,997 exact pictures**.
All eight picture-hash files are byte-identical to main.
`cpu-picture-agreement.json`, `cpu-gate.txt` and
`cpu-admission-final/` retain the CPU-only proof.

### One continuous live census and direct poll attribution

Exactly **one continuous seeded** `--live-apu-reads --pacing-predictor`
process completes **1,473,123 iterations**, carrying SPC state across seeds.
Its gate passes with **unexplained 0**. Live totals improve from
**1,899 → 1,566 missing**, **269 → 36 excess** and **75 → 7 false positives**.
`live-command.json`, `live-gate.txt`, `live-counts.txt`, `live.csv` and
`live-timing.jsonl.zst` retain the single launch and complete run.

There are **281 changed live hold rows**. **136** have exactly the same
changed ROM before/after pair. Every other **145** is directly attributed:
**four** to the pendant episode at 228,499 (`$08:C3FD`), **six** to the
crystal episode at 735,229 and **135** to the crystal episode at 881,095
(both `$08:C606`). **Zero rows are unattributed.**
The shared ROM contribution is **-96 / 0 / 0** missing/excess/FP;
the song-poll contribution is **-237 / -233 / -68**, giving the full
**-333 / -233 / -68** live delta.

The two crystal first-zero polls now agree with source. The pendant first
zero moves one iteration later and contributes **+9 / +9 / +2**; this
regression is retained, not hidden or fitted away. No polling-price or
polling mechanism changes are made.

The source PCs are established by **20 direct APUI bus receipts**: start
and first-zero reads in ten episodes from the hash-verified cold unmodified-ROM
CPU replay. Every receipt is rechecked against the retained source bus file.
The sampled source main-loop status reads at `$00:80E1` are separate from
these game song-end predicates. PCs are not inferred from sprite labels.
`poll-source-identity.json`, `poll-source-verification.json` and
`source-poll-episode-proof.json` retain the source evidence. The complete
source context/read payload hashes agree across both traces, with zero
source payload differences.

Bounds are derived **without changed holds**. Actual native boolean-poll
clusters overlap the direct source intervals. Live-versus-ROM differences
are measured separately for main-without-VWF and VWF in each version,
removing intended common instruction prices. Actual native/source contexts
and before/after boolean-poll differences also locate divergent execution.
Recovery ends at the first actual reseed before the containing 2,000-iteration
boundary, otherwise at that boundary after the final independently observed
event. `live-execution-events.json` and `poll-attribution.json` retain these
cues and bounds.

| Episode | Direct source PC | Source first zero | Native first zero before → after | Last execution event | Recovery end | Live-only rows | Delta missing / excess / FP |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 94,334 | `$08:C3FD` | 94,854 | 94,854 → 94,854 | — | 96,000 | 0 | +0 / +0 / +0 |
| 141,568 | `$08:C3FD` | 142,088 | 142,088 → 142,088 | — | 144,000 | 0 | +0 / +0 / +0 |
| 228,499 | `$08:C3FD` | 229,035 | 229,035 → 229,036 | 229,069 | 230,000 | 4 | +9 / +9 / +2 |
| 380,243 | `$08:C606` | 380,881 | 380,881 → 380,881 | — | 382,000 | 0 | +0 / +0 / +0 |
| 461,989 | `$08:C606` | 462,628 | 462,628 → 462,628 | — | 464,000 | 0 | +0 / +0 / +0 |
| 626,618 | `$08:C606` | 627,256 | 627,257 → 627,257 | 629,601 | 629,845 | 0 | +0 / +0 / +0 |
| 735,229 | `$08:C606` | 735,867 | 735,868 → 735,867 | 736,633 | 738,000 | 6 | -12 / -12 / -3 |
| 881,095 | `$08:C606` | 881,734 | 881,733 → 881,734 | 884,326 | 884,327 | 135 | -234 / -230 / -67 |
| 1,064,096 | `$08:C606` | 1,064,734 | 1,064,734 → 1,064,734 | — | 1,066,000 | 0 | +0 / +0 / +0 |
| 1,317,697 | `$08:C606` | 1,318,335 | 1,318,335 → 1,318,335 | — | 1,320,000 | 0 | +0 / +0 / +0 |

Every live-only changed hold row is listed below. `changed-live-holds.csv/json`
contains the same budgets, metric deltas and direct-PC attribution;
`all-changed-live-holds.csv` includes all 136 shared ROM changes.

| Iteration | Source lag | ROM holds before → after | Live holds before → after | Episode / direct source PC |
| --- | ---: | ---: | ---: | --- |
| 229,036 | 5 | 5 → 5 | 5 → 0 | 228,499 / `$08:C3FD` |
| 229,037 | 0 | 0 → 0 | 0 → 5 | 228,499 / `$08:C3FD` |
| 229,039 | 4 | 4 → 4 | 4 → 0 | 228,499 / `$08:C3FD` |
| 229,042 | 0 | 0 → 0 | 0 → 4 | 228,499 / `$08:C3FD` |
| 735,937 | 1 | 1 → 1 | 0 → 1 | 735,229 / `$08:C606` |
| 735,938 | 0 | 0 → 0 | 1 → 0 | 735,229 / `$08:C606` |
| 735,947 | 5 | 4 → 4 | 0 → 4 | 735,229 / `$08:C606` |
| 735,948 | 0 | 0 → 0 | 4 → 0 | 735,229 / `$08:C606` |
| 736,420 | 8 | 7 → 7 | 0 → 7 | 735,229 / `$08:C606` |
| 736,422 | 0 | 0 → 0 | 7 → 0 | 735,229 / `$08:C606` |
| 881,794 | 0 | 0 → 0 | 1 → 0 | 881,095 / `$08:C606` |
| 881,795 | 1 | 1 → 1 | 0 → 1 | 881,095 / `$08:C606` |
| 881,804 | 0 | 0 → 0 | 4 → 0 | 881,095 / `$08:C606` |
| 881,805 | 5 | 4 → 4 | 0 → 4 | 881,095 / `$08:C606` |
| 882,275 | 8 | 8 → 8 | 0 → 8 | 881,095 / `$08:C606` |
| 882,276 | 0 | 0 → 0 | 8 → 0 | 881,095 / `$08:C606` |
| 882,475 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,476 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,477 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,484 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,485 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,486 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,529 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,530 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,531 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,538 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,539 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,540 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,591 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,592 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,593 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,600 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,601 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,602 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,676 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,677 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,678 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,686 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,687 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,688 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,740 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,741 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,742 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,750 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,751 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,752 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,804 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,805 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,806 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,814 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,815 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,816 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,893 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,894 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,895 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,919 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,920 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,921 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,957 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,958 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,959 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 882,983 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,984 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 882,985 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,053 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,054 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,055 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,071 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,072 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,073 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,117 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,118 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,119 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,135 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,136 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,137 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,181 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,182 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,183 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,199 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,200 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,201 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,272 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,273 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,274 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,298 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,299 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,300 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,338 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,339 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,340 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,364 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,365 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,366 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,398 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,399 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,400 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,424 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,425 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,426 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,483 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,484 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,485 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,509 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,510 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,511 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,539 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,540 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,541 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,565 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,566 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,567 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,601 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,602 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,603 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,627 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,628 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,629 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,697 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,698 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,699 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,724 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,725 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,726 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,727 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,728 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,729 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,754 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,755 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,756 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,765 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,766 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,767 | 2 | 2 → 2 | 0 → 2 | 881,095 / `$08:C606` |
| 883,792 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,793 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,794 | 0 | 0 → 0 | 2 → 0 | 881,095 / `$08:C606` |
| 883,842 | 7 | 7 → 7 | 0 → 7 | 881,095 / `$08:C606` |
| 883,881 | 0 | 0 → 0 | 7 → 0 | 881,095 / `$08:C606` |
| 883,977 | 7 | 7 → 7 | 0 → 7 | 881,095 / `$08:C606` |
| 884,319 | 0 | 0 → 0 | 59 → 0 | 881,095 / `$08:C606` |
| 884,320 | 0 | 0 → 0 | 6 → 0 | 881,095 / `$08:C606` |
| 884,321 | 0 | 0 → 0 | 25 → 0 | 881,095 / `$08:C606` |
| 884,322 | 59 | 59 → 59 | 1 → 59 | 881,095 / `$08:C606` |
| 884,323 | 6 | 6 → 6 | 1 → 6 | 881,095 / `$08:C606` |
| 884,324 | 25 | 25 → 25 | 1 → 25 | 881,095 / `$08:C606` |


### Verification boundary

The touched-module groups pass: ancilla **50**, dungeon **64**, sprite draw
**304**, sprite **120**, sprite main **16**, guard **13**, Mothula/spike
**seven**, Hinox/shop **eight**, and player **50**, including all **24 new
ROM fixtures**. Generator tests pass (**eight**). Production builds are
warning-free. Library compilation retains the baseline **16 distinct warning
headers**, with no additions. `filtered-summary.json`, `cleanup-commands.json`,
`generator-tests.log` and `warning-agreement.json` retain the results.

After all runtime and fixture changes, the full default library suite ran
**once**, with `CARGO_INCREMENTAL=0` solely to bound rebuildable cache size:

```sh
ZELDA3_ASSETS=/Users/missingno/Documents/zelda3-rs/zelda3_assets.dat cargo test -q --profile parity -p zelda3 -p snes --lib
```

SNES records **505 passed / seven ignored**. Zelda records **3,097 passed /
one failed / three ignored**. The sole failure is the explicitly permitted
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There is no other failure and no post-full repair or repeat.
`full-library-command.json` and `full-library.log` preserve the complete run.

All **942** frozen source files and the final binary retain their hashes
across the final ROM, CPU-picture, selected-profile, live and library checks.
The previous documentation is preserved byte-for-byte before this appended
section. Final authority and reused-artifact checks are unchanged.


Final source, binary, authority, reused-artifact and resource checks are in
`runtime-source-agreement.json`, `authorities-after.json`,
`prior-final-verification.json`, `resources.json`, `verification.json`,
`runtime.patch` and `artifact-hashes.json`.
Every launched process has completed. Every game-binary invocation uses
CPU-only `--logical-replay`. There is no GPU, `--video`, cached-A/V or
external Snes9x run, new runtime environment switch, classifier, matcher or
P2 edit. The ledger, oracle cache and gate helpers remain read-only.
No commit, push or `cargo fmt` occurs. The worktree target remains below
**12,000,000,000 bytes**.

These are seeded logical, CPU-picture and unmodified-ROM instruction proofs.
The 190 listed instruction/protocol/wait/phase blockers remain as classified.
Cold exact A/V acceptance is a separate boundary.

## Lagless prefix hunt 28

Instruction pricing on `codex/prefix-hunt28`, based on main **`f5279bb8`**.
Evidence is in `target/prefix-hunt28/`. The frozen admission binary is
`zelda3-final`, SHA-256
`b735eda5a0f8cc9f69617795ee6f957ff10cd18788e01eec23f3cac1b5767e06`.

The ROM census improves **1,530/0/0 → 1,500/0/0**, recovering **30 rows**
with **zero exposed**. Main work gains **332,105,232 instruction clocks**.
Live totals worsen **1,566/36/7 → 1,735/144/14**; all **43 live-only changed
holds** are attributed to direct `$08:C606` crystal-poll episodes. CPU pictures
remain **1,462,997 exact**. The final full library suite has only the permitted
spotlight failure.

### Diagnosis: return boundary and interrupt-phase ownership

The seven 146,786–146,810 rows do **not** establish an accumulating
inter-iteration hardware deficit. On every row, `main_master + cost_shortfall`
is **230,464**. Main CPU work falls while the inverse diagnostic shortfall
grows. The source main CPU changes from 227,742 to 215,710 clocks; native
changes from 227,764 to 215,732. Main refresh falls from 6,840 to 6,480 source
clocks. No per-iteration debt is added to the runtime.

| Iteration | Source main CPU | Native main CPU | Signed gap | Diagnostic shortfall | Main + shortfall |
| --- | ---: | ---: | ---: | ---: | ---: |
| 146,786 | 227,742 | 227,764 | -22 | 2,700 | 230,464 |
| 146,790 | 227,840 | 227,862 | -22 | 2,602 | 230,464 |
| 146,794 | 225,430 | 225,452 | -22 | 5,012 | 230,464 |
| 146,798 | 222,620 | 222,642 | -22 | 7,822 | 230,464 |
| 146,802 | 220,028 | 220,050 | -22 | 10,414 | 230,464 |
| 146,806 | 218,156 | 218,178 | -22 | 12,286 | 230,464 |
| 146,810 | 215,710 | 215,732 | -22 | 14,732 | 230,464 |

The recurring **-22** is an executed return-boundary convention. The source
main profile closes at `STZ $12` at `$00:805D`. Native also charges the
following `BRA $8034` at `$00:805F`, exactly 22 clocks. The unmodified-ROM
fixture `prefix28_profile_cut_excludes_only_the_executed_return_branch`
executes the 24-clock store and complete 46-clock suffix with multiple latch
and frame values. This explains the same convention at 97,658 and 172,413;
it does not justify subtracting 22 from physical execution.

The existing source example gains `--checkpoint-hardware-audit` to retain
actual register writes, hardware events, ROM identity and boundary WRAM
hashes. The first opening NMI is replayed from the earlier 146,148 receiver;
the following six openings reproduce their retained checkpoint WRAM hashes.
These are resumed diagnostics, not a new cold picture or A/V acceptance run.

| Actual opening NMI write | Channels / mode | A-bus source | Transfer bytes | Existing hardware-law clocks |
| --- | --- | --- | ---: | ---: |
| `$00:8BCA` → `$420B=1` | one, channel 0 / mode 0 | `$00:0800` | 544 | 4,378 |
| `$00:9374` → `$420B=1` | one, channel 0 / mode 1 | `$7E:E800` | 2,048 | 16,410 |
| `$00:8135`, `$00:82C4` → `$420C=0` | zero active HDMA channels | none | 0 | 0 transfer clocks |

There are 225 scheduled HDMA-start events per field, **zero active transfer
lines**, and no nonzero HDMA-enable write in the seven audited windows.
DMA already uses **18 per run + 8 per channel + 8 per byte**. Fixture
`prefix28_polygon_dma_all_pending_paths_match_unmodified_rom_and_existing_hardware_charge`
checks pending values 0, 1 and 255 against executed ROM and proves the existing
2,048-byte upload charge; with OAM the two runs total **20,788** clocks.

The source opening-handler profile contains **5,230 CPU + 20,788 DMA + 800
refresh = 26,818** clocks. Opening edge to main entry is 26,858 or 26,880,
including the actual hardware-entry phase. The source IRQ contains **1,528
CPU + 80 refresh = 1,608**; lag NMI **3,226 + 120 = 3,346**. The polygon
lane contains about **212,992 CPU + 6,400 refresh = 219,392** clocks.
The first audit also contains **222,264 CPU + 6,720 refresh** clocks of
main-loop idle polling, outside main work; idle padding is not a new work
price. Main wall time falls **234,582 → 222,190**, while source idle
padding rises **228,984 → 241,374** clocks. This is existing field-time
partitioning, not an accumulating new transfer charge. These separately profiled lanes must not all be added to main: their
boundaries cross main/idle/interrupt intervals.

Native assigns the seven rows the **9,344-clock** entry handler and their
adjacent entries **30,812 clocks**. Source main-entry to main-done is
348,414 down to 336,022 clocks, unlike the native phase estimate. The
unrepresented source work belongs to **NMI polygon upload and polygon/IRQ
phase ownership**, rather than an absent DMA byte/channel law, active HDMA,
growing refresh, or the 22-clock return. `nmi_update_irqgfx` already charges
the upload when the pending generation actually executes. Moving that work
between handlers would change phase ownership; this batch leaves it and P2
unchanged; an exact interrupt-phase repair remains unresolved in class (d).
All seven protected rows retain their instruction budgets and
holds. `part-a-components.json`, `part-a-summary.csv/json`,
`hardware-first-opening.json`, `hardware-boundary-agreement.json` and
`native-hardware-window.csv` retain the observations.

### Owners and unmodified-ROM fixtures

All new lagless marker fixtures construct `lag_free` states. Markers price
executed ROM blocks, exclude child work and preserve actual call/return
ownership. The existing prefix-25/27 fixtures are updated where the newly
priced suffix/child now belongs to a separate scope. The sprite-family audit
finds **zero negative families** after the changes.

| Owner / actual entries | Fixture names (common `prefix28_` prefix) |
| --- | --- |
| Heart-piece suffix `$05:F059–F0CC`, retained fourth-piece receipt | `heart_piece_all_bounce_delay_and_pickup_paths_exclude_children`; `fourth_heart_piece_retained_return_prices_only_its_suffix` |
| Hinox `$06:9F05`, idle `9FBC`, walk `A029`, throw `9F4A`, face/direction `9FE1/A004` | `hinox_all_states_timers_random_faces_and_spawn_paths_exclude_children` |
| Guard idle/alert/approach/attack `$05:CE74/CF44/CF85/D045` | `guard_idle_alert_approach_and_throw_all_paths_exclude_children` |
| Bank-1E recoil `$1E:FE9B–FF1D`, actual collision wrapper `$1E:8B2E` | `bank1e_recoil_all_returns_masks_and_collision_paths_exclude_children` |
| Post-overlap damage `$06:F2D3–F313` and `F2A9`, excluding type dispatch | `damage_post_overlap_all_action_and_transmutation_paths_exclude_dispatch` |
| Thrown debris `$06:AC41–AD02`, four spawn attempts and garnish scan | `thrown_scatter_all_sizes_slots_allocations_and_clip_paths_exclude_children` |
| Breakage `$06:E25A/E26D`; remove duplicate leaf charges in its callers | `breakage_all_flags_and_slots_excludes_callers` |
| Arrghus puffs `$1E:B6E9–B817`, all 512 angles, signed wave/wrap and actual 13-entry loop | `arrghus_puffs_all_angles_quadrants_waves_and_wraps_exclude_children` |
| Arrghus caller `$1E:B433–B4BD`, hit transition `B5EE` | `arrghus_prefix_all_states_hit_recoil_and_collision_paths_exclude_children`; `arrghus_hit_transition_all_slots_excludes_sound` |
| Arrghus draw `$1E:B840–B8B3`, actual `$1E:87F0` entry | `arrghus_draw_all_states_waves_clips_and_page_crossings_exclude_children` |
| Arrgi `$1E:B8C4–B9A8` | `arrgi_all_tether_home_damage_and_return_paths_exclude_children` |
| Big-shadow entry `$1D:D1A8`, actual `$1D:8549/$06:84BD` wrappers | `big_shadow_entry_all_z_and_clip_paths_excludes_draw_and_coordinates` |
| Swamp overlay clear `$01:EFEC–F045`, 256 × 16 word stores in ROM order | `swamp_overlay_clear_all_stores_and_width_restoration_match_rom` |

The heart bounce now follows the ROM's final `INC` after the signed shift:
shifted X velocity `$FF` becomes zero. This is the only behavior correction
in the suffix; its admission is the complete CPU-picture census below.
The swamp clear gains **215,798** instruction clocks at 451,842, including
its actual loop and width restoration. No wait price is introduced.

An initial candidate inadvertently shared 194 newly priced wrapper clocks
with the already priced `$1D:D1AF` large-shadow entry. The correction confines
those wrappers to the actual new `$1D:D1A8` entry and restores existing
`D1AF` accounting. The final profiles, gates and live census use the corrected
binary. The superseded full-suite log is retained separately; it is not
claimed as final-source validation.

### Classification, signed remainders and admission

Classification remains diagnostic: executed upload-protocol work is (c);
source VCounter polling or insufficient complete signed nonpoll work is (d);
otherwise signed nonpoll work at or above the frozen boundary is (a). There
is no independent arrival proof for (b). Main totals include VWF.

Retained source/authority hashes are checked before reuse. All 942 base
source hashes agree with the main baseline; fresh baseline profiling agrees
with all 29,500 retained routine rows. Final native profiles cover all **282**
selected rows. Complete sprite/object/caller partitions, including the VWF
continuation and negative attribution aliases, conserve every full signed
instruction gap. `source-profile-selection.json` records which source copy
owns each iteration; duplicated retained source receipts are counted once.
`routine-vectors.csv/json`, `non-sprite-vectors.csv/json`,
`object-vectors.csv/json`, `remaining-sprite-residuals.csv/json`,
`blocker-decomposition.json` and `signed-partition-agreement.csv/json`
retain the concrete owners and offsets, rather than an undifferentiated
"other caller" remainder.

The complete ROM route gains **332,105,232 main instruction clocks**, all
positive deltas, recovering **30 rows** and exposing **zero**. The 190 old
missing rows through 461,989 gain **1,756,220** clocks; five recover:
**255,517 / 255,521 / 255,525 / 451,842 / 456,721**. The 282-row selected
set gains **1,763,370** clocks. These measured gains are consequences of ROM
prices, never inputs to a price.

| Cohort | (a) before → after | (b) | (c) before → after | (d) before → after | Recovered | Exposed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Before 141,568 | 0 → 0 | 0 | 5 → 5 | 2 → 2 | 0 | 0 |
| 141,568–228,499 | 50 → 50 | 0 | 3 → 3 | 14 → 14 | 0 | 0 |
| 228,499–380,243 | 46 → 43 | 0 | 4 → 4 | 11 → 11 | 3 | 0 |
| 380,243–461,989 | 46 → 44 | 0 | 2 → 2 | 7 → 7 | 2 | 0 |
| **Total** | **142 → 137** | **0** | **14 → 14** | **34 → 34** | **5** | **0** |

The canonical missing-row set and classification agree exactly. All fourteen
upload rows retain identical instruction budgets and holds. All 28 classified
upload/source-VCounter rows retain their holds, and **no full-route hold
change has a native raster-wait owner**. The seven -22 rows are byte-identical
in the before/after predictor CSVs. There is no upload, spotlight-wait,
classifier, matcher or P2 modification. Shared requested prices add **1,100**
instruction clocks to spotlight row 403,304 (Hinox) and **192** to 407,899
(damage); their holds stay unchanged. These are instruction prices, with no
spotlight wait or arrival correction. Class (a) remains nonempty, so this
batch does not extend source classification into 461,989–600,000.

| Concrete remaining owner | Signed decomposition / result |
| --- | --- |
| Heart-piece `$eb` | Suffix now priced; **294** remains: outer `$05:F018` wrapper **190** and bank-6 `$06:C0A3` bounce **104**. Parent draw/damage aliases cancel. |
| Hinox `$11` | All **ten selected family instances** have **zero** signed family remainder; total gain **14,928** clocks. Any remaining row blocker belongs to another owner. |
| Guard `$48`, 180,178 | State work gains **6,878**; remaining **460 = 230 `$06:EB0A` Sprite_IsRightOfLocation + 230 `$06:EB1D` Sprite_IsBelowLocation**. Other guard parent/child offsets cancel. |
| Thrown `$ec`, 356,806 | Remaining **2,926**: CarriedSprite_CheckForThrow `$06:DF6D` **1,300**, scenery interaction `$06:E164` **1,140**, single-sprite hit `$06:E1B2` **682**, sound pan `$0D:BBA1/BB8A/BB98` **408/220/152**, with actual `$06:E149/E02A` offsets **-268/-708**. Draw, active-dispatch and collision aliases cancel separately. |
| Eyegore `$83/$84` | Recoil and post-overlap children are priced. `$84` facing remainder **194 = 104 `$1E:FE69` + 90 `$06:EAA0`** where executed. At 367,291 and 367,394, `$83` retains **4,216 = 2,486 sword damage `$06:ED3F` + 842 damage dispatch `$06:F2B4` + 584 zap `$06:EC02` + 470 pan `$0D:BBA1` - 166 post-overlap scope offset**. |
| Terrorpin `$8e`, 372,814 | **1,584 → 938**, gain **646**. Remaining **938 = 1,008 type-damage dispatch `$06:F2B4` - 166 priced prelude alias + 96 weapon tink `$06:F69F`**. Other signed caller/child pairs cancel. |
| Arrghus active `$8c` | Puff/draw/caller/hit work gains **1,154,440** clocks across selected instances. Normal active-state remainders **274/292/356** include the actual `$1E:87F4` **42-clock RTS** plus still-unpriced state bodies. At 456,900, **356 = 1,538 `$1E:8B19` - 1,224 `$1E:B433` + 42**. |
| Arrgi `$8d` | Active paths are exact in 28 instances; selected family gain **341,410**. The state-08 instance at 456,721 retains **50,348 = 43,394 Sprite_ReturnIfBossFinished `$06:8F1C` + 14,174 execute attribution `$06:84E2` - 7,410 initialization `$06:864D` + 190 Arrghus_HandlePuffs `$1E:B8B4`**. The accompanying `$8c` state-08 remainder is **3,480 = 3,338 + 712 - 570** across those same first three owners. |

At 297,585, the **2,404** non-sprite remainder is concrete: Y movement
caller/children `$07:BA0A/BDB1/BEF8` sum to **1,716**, CancelDash **96**,
ResetSwimmingState **286**, HandleYItem **150**, UnequipCape **178**, and
main return **-22**. Main/overworld, Link/default, X movement and velocity/
sand-drag attribution pairs cancel. At 456,900, the **19,630** non-sprite
vector explicitly contains hookshot RTS-dispatch `$08:833C` **5,388**,
single-sprite collision `$08:8DAE` **5,320**, AddHookshot `$09:9B10` **1,210**,
tile/camera/OAM/sound entries and their signed dispatcher offsets. Every entry
is retained in `remaining-blockers.csv/json`; none is priced from its residual.

| Missing before iteration | ROM before → after |
| --- | ---: |
| 94,334 | 2 → 2 |
| 141,568 | 7 → 7 |
| 228,499 | 74 → 74 |
| 380,243 | 135 → 132 |
| 461,989 | 190 → 185 |

The final eight-slice seeded ROM helper reports **PASS**, **unexplained 0**,
complete **1,473,123 iterations**, **1,530/0/0 → 1,500/0/0** missing/excess/FP.
`rom-comparison.json`, `rom-gate.txt`, `rom-counts.txt` and
`final-replay-commands.json` identify the exact binary and every changed row.

### Single seeded live census and direct poll attribution

Exactly **one** full seeded `--live-apu-reads --pacing-predictor` census runs
on the frozen binary. Its totals **worsen** from **1,566/36/7 to
1,735/144/14** missing/excess/FP. There are **73 changed live holds**:
**30 shared ROM changes** and **43 live-only changes**. All 43 belong to four
crystal song-end episodes, directly sourced at **`$08:C606`**;
**zero changed rows are unattributed**. The pendant `$08:C3FD` episodes have
no changed live-only holds.

Source context, APUI reads and APUI writes agree on **every one of the
1,473,123 trace rows**. Their normalized SHA-256 is
`a85696ffff0f15dd054297a8a36d421583ccb319440192fc3e16c90fc35307d5`
on both sides. Cartridge APUI answers are disabled throughout. The source
PC identities come from the hash-verified direct cold-ROM APUI bus receipts,
read again in `poll-source-identity.json`, rather than inferred from held rows.

Episode recovery bounds use actual native boolean-poll clusters, **3,676
changed four-lane live-vs-ROM work events**, **278 context/poll events** and
actual reseeds. The combined event union has 3,677 iterations. Bounds are
established before assigning changed holds: last execution event, then the
first actual reseed before the containing 2,000-iteration boundary, otherwise
that boundary. They do not use the changed-hold set as a boundary.
`live-execution-events.json`, `live-reseeds.json`, `live-poll-clusters.json`
and `poll-attribution.json` retain those independent observations.

| Source episode | Direct source PC | Source first zero | Native first zero before → after | Last changed execution event | Recovery end | Live-only rows | Δ missing/excess/FP |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| 94,334 | `$08:C3FD` | 94,854 | 94,854 → 94,854 | — | 96,000 | 0 | +0/+0/+0 |
| 141,568 | `$08:C3FD` | 142,088 | 142,088 → 142,088 | — | 144,000 | 0 | +0/+0/+0 |
| 228,499 | `$08:C3FD` | 229,035 | 229,036 → 229,036 | — | 230,000 | 0 | +0/+0/+0 |
| 380,243 | `$08:C606` | 380,881 | 380,881 → 380,881 | — | 382,000 | 0 | +0/+0/+0 |
| 461,989 | `$08:C606` | 462,628 | 462,628 → 462,627 | 464,051 | 466,000 | 6 | +13/+13/+3 |
| 626,618 | `$08:C606` | 627,256 | 627,257 → 627,256 | 629,601 | 629,845 | 10 | -27/-27/-5 |
| 735,229 | `$08:C606` | 735,867 | 735,867 → 735,867 | — | 736,000 | 0 | +0/+0/+0 |
| 881,095 | `$08:C606` | 881,734 | 881,734 → 881,734 | — | 882,000 | 0 | +0/+0/+0 |
| 1,064,096 | `$08:C606` | 1,064,734 | 1,064,734 → 1,064,735 | 1,066,962 | 1,066,963 | 12 | +96/+95/+4 |
| 1,317,697 | `$08:C606` | 1,318,335 | 1,318,335 → 1,318,336 | 1,320,899 | 1,320,900 | 15 | +117/+27/+5 |

Every live-only changed hold is listed below. The source PC in every row is
**`$08:C606`**; each episode's directly observed first zero and independent
recovery bound are in the table above. `live-only-changed-rows.csv/json`
additionally retain the ROM before/after holds, source lag, native first-zero
observations and recovery end for each individual row.

| Iteration | Source episode / PC | Source lag | Live hold before → after |
| --- | --- | ---: | ---: |
| 462,699 | 461,989 / `$08:C606` | 0 | 0 → 1 |
| 462,700 | 461,989 / `$08:C606` | 1 | 1 → 0 |
| 462,709 | 461,989 / `$08:C606` | 0 | 0 → 4 |
| 462,710 | 461,989 / `$08:C606` | 5 | 4 → 0 |
| 463,179 | 461,989 / `$08:C606` | 0 | 0 → 8 |
| 463,180 | 461,989 / `$08:C606` | 8 | 8 → 0 |
| 627,317 | 626,618 / `$08:C606` | 1 | 0 → 1 |
| 627,318 | 626,618 / `$08:C606` | 0 | 1 → 0 |
| 627,327 | 626,618 / `$08:C606` | 5 | 0 → 4 |
| 627,328 | 626,618 / `$08:C606` | 0 | 4 → 0 |
| 627,800 | 626,618 / `$08:C606` | 8 | 0 → 8 |
| 627,801 | 626,618 / `$08:C606` | 0 | 8 → 0 |
| 629,325 | 626,618 / `$08:C606` | 7 | 0 → 7 |
| 629,327 | 626,618 / `$08:C606` | 0 | 7 → 0 |
| 629,458 | 626,618 / `$08:C606` | 0 | 7 → 0 |
| 629,462 | 626,618 / `$08:C606` | 7 | 0 → 7 |
| 1,064,798 | 1,064,096 / `$08:C606` | 1 | 1 → 0 |
| 1,064,799 | 1,064,096 / `$08:C606` | 0 | 0 → 1 |
| 1,064,808 | 1,064,096 / `$08:C606` | 5 | 4 → 0 |
| 1,064,809 | 1,064,096 / `$08:C606` | 0 | 0 → 4 |
| 1,065,281 | 1,064,096 / `$08:C606` | 8 | 7 → 0 |
| 1,065,282 | 1,064,096 / `$08:C606` | 0 | 0 → 7 |
| 1,066,543 | 1,064,096 / `$08:C606` | 0 | 0 → 7 |
| 1,066,544 | 1,064,096 / `$08:C606` | 7 | 7 → 0 |
| 1,066,958 | 1,064,096 / `$08:C606` | 59 | 59 → 0 |
| 1,066,959 | 1,064,096 / `$08:C606` | 6 | 6 → 59 |
| 1,066,960 | 1,064,096 / `$08:C606` | 24 | 24 → 6 |
| 1,066,961 | 1,064,096 / `$08:C606` | 1 | 1 → 24 |
| 1,318,396 | 1,317,697 / `$08:C606` | 1 | 1 → 0 |
| 1,318,397 | 1,317,697 / `$08:C606` | 0 | 0 → 1 |
| 1,318,406 | 1,317,697 / `$08:C606` | 4 | 4 → 0 |
| 1,318,407 | 1,317,697 / `$08:C606` | 0 | 0 → 4 |
| 1,318,878 | 1,317,697 / `$08:C606` | 8 | 8 → 0 |
| 1,318,879 | 1,317,697 / `$08:C606` | 0 | 0 → 8 |
| 1,320,353 | 1,317,697 / `$08:C606` | 0 | 0 → 7 |
| 1,320,357 | 1,317,697 / `$08:C606` | 7 | 7 → 0 |
| 1,320,542 | 1,317,697 / `$08:C606` | 7 | 7 → 0 |
| 1,320,544 | 1,317,697 / `$08:C606` | 0 | 0 → 7 |
| 1,320,895 | 1,317,697 / `$08:C606` | 58 | 58 → 0 |
| 1,320,896 | 1,317,697 / `$08:C606` | 6 | 6 → 0 |
| 1,320,897 | 1,317,697 / `$08:C606` | 24 | 24 → 0 |
| 1,320,898 | 1,317,697 / `$08:C606` | 1 | 1 → 0 |
| 1,320,899 | 1,317,697 / `$08:C606` | 1 | 1 → 0 |

| Missing before iteration | Live missing/excess/FP before → after |
| --- | --- |
| 94,334 | 2/0/0 → 2/0/0 |
| 141,568 | 7/0/0 → 7/0/0 |
| 228,499 | 74/0/0 → 74/0/0 |
| 380,243 | 144/9/2 → 141/9/2 |
| 461,989 | 199/9/2 → 194/9/2 |

The four changed crystal episodes contribute **+199 missing, +108 excess,
+7 FP**; the 30 shared ROM recoveries contribute **-30 missing**. These sum
exactly to the full live change **+169/+108/+7**. No hold, poll matcher,
source answer, acceptance threshold or per-episode runtime price is changed
to compensate for the song-end timing shifts. `live-command.json` records
one launch, and `live-counts.txt` independently checks the final metrics.

### CPU-picture and verification boundary

The final seeded `--cpu-video` census retains **1,462,997 / 1,473,123 exact
pictures**. All eight complete per-iteration picture-hash streams are
**byte-for-byte identical to main**; all slices are WRAM-clean and exit zero.
`cpu-picture-agreement.json`, `cpu-gate.txt` and `cpu-commands.json` retain the
counts, hashes and exact CPU-only commands. This admits the heart velocity
correction without changing the established image result. It is not a GPU,
continuous cold A/V or live audio acceptance claim.

Filtered touched-module groups pass: draw **318** (rerun after the shadow
entry correction), sprite **120**, guard **13**, Mothula/Arrghus **7**, Hinox
**8**, dungeon **65**, main wait **7**, NMI **10**, and cycle ledger **9**.
All **17 new fixtures**, **8 generator tests**, and **6 source-example tests**
pass. The final production binary and source example build without warnings.
The default library build retains exactly the baseline warning-header sequence
(24 emitted headers, 16 distinct), with no added warnings.

The final-source full default suite executes the requested command exactly:

```sh
ZELDA3_ASSETS=/Users/missingno/Documents/zelda3-rs/zelda3_assets.dat cargo test -q --profile parity -p zelda3 -p snes --lib
```

SNES: **505 passed, 7 ignored**. Zelda: **3,114 passed, 3 ignored**, with the
sole permitted failure:
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There are **zero unexpected failures**. One earlier full-suite invocation was
already running when the shared-shadow census effect required correction; its
superseded log is retained, and the corrected-source full suite is run once.
Both runs have only that permitted failure. `test-verification.json`,
`full-library-command.json`, `full-library.log`,
`full-library-superseded-result.json` and `warning-agreement.json` preserve
that distinction.

The final source freeze covers **943 files**. All **14 authority hashes** and
**496 reused evidence receipts** remain unchanged. Source fixtures and resumed
hardware diagnostics establish ownership; seeded ROM/live prediction and CPU
pictures establish their separate admission results. No cached-A/V, GPU,
`--video`, external Snes9x process, classifier/matcher/P2 change, commit or push
is part of this batch. All Zelda executable commands use CPU-only logical
replay modes; all launched processes complete. Only this worktree's build
incremental cache is removed to maintain the target-size limit.

The worktree target occupies **9.75 GB allocated**, below the **16 GB** limit. All gate, test, diagnostic and trace-analysis processes have exited.

## Lagless prefix hunt 29

This batch starts from **`cd4e1fbe`**, on `codex/prefix-hunt29`. The frozen
production executable is `target/prefix-hunt29/zelda3-final`, SHA-256
`b16f3b997af2460d4a308c9f01b8b16756d4abd4798b7bec052341d2eea37332`.
All evidence below is in `target/prefix-hunt29/` unless otherwise stated.
The regenerated full-v2 ledger contains **1,473,123 iterations**. Before reuse,
all 387 hunt-28 artifact receipts, 341 retained source receipts and 14 authority
hashes match; `baseline-hash-verification.json`, `prior-artifact-verification.json`,
`prior-source-verification.json` and `authorities-before.json` retain that check.
Missing prior hunts 10/11/21/22 are not consulted.

### Concrete owners, source prices and fixtures

The original 50 class-(a) rows in 141,568–228,499 rank by closable source
families as follows. Family coverage overlaps; these are not independent NMI
recoveries or fitted prices. All listed family vectors close exactly, including
the already-priced children. The cohort recovers **22 whole NMI rows**.

| Owner family | Original cohort rows closed | Source-owned clocks newly priced in those rows |
| --- | ---: | ---: |
| Hoarder `$17`: covered body, lift and shared leaves | 19 | 16,948 |
| Rupee pull `$33`: bounce, pause, busy and prize paths | 10 | 12,340 |
| Bee `$79`: selector and dormant hive | 10 | 6,040 |
| Heart piece `$eb`: bounce and caller wrappers | 7 | 2,058 |
| Guard `$48`: shared damage leaves | 1 | 460 |

`cohort-a-family-closure-ranking.json` retains the original-row ranking,
iteration lists, signed before/after work and actual closures. Every source,
main and final routine/object/non-sprite/sprite vector is regenerated for all
282 selected rows. The old sparse vectors agree on every retained key; omitted
keys are exact before this batch. Exact cells are retained in the new vectors,
so the signed partitions and negative-family audit are directly reproducible.

Prices use generated unmodified-ROM block markers and routine scopes. The
fixtures execute the actual unmodified ROM across their branch/path matrices,
exclude nested children, compare exclusive clocks and explicitly construct a
`lag_free` state when checking lagless markers. No residual, sampled deficit,
scene table, threshold, classifier, matcher or P2 change supplies a price.

| Owner | ROM entries | Exact-ROM fixture |
| --- | --- | --- |
| location comparisons | `$06:EB0A`, `$06:EB1D` | `prefix29_location_predicates_all_signed_borrow_and_wrap_paths` |
| dash | `$07:9195` | `prefix29_cancel_dash_all_slots_and_clear_paths` |
| swim and cape | `$07:983A`, `$07:AE54` | `prefix29_swimming_reset_and_cape_direct_entries_exclude_acceleration_child` |
| heart callers | `$05:F018`, `$06:C0A3` | `prefix29_heart_piece_callers_all_early_bounce_and_pickup_paths_exclude_children` |
| sound pan | `$0D:BBA1`, `$0D:BB7C`, `$0D:BB8A`, `$0D:BB98` | `prefix29_sound_pan_all_regions_boundaries_and_busy_channels_exclude_children` |
| boss finished | `$06:8F1C` | `prefix29_boss_finished_all_slots_flags_and_property_paths` |
| tink | `$06:F69F` | `prefix29_weapon_tink_busy_and_coordinate_carry_paths_exclude_pan` |
| sword and calculated damage | `$06:ED3F`, `$06:ED89` | `prefix29_sword_all_flags_running_frames_items_and_damage_lookup_paths_exclude_give_damage` |
| give damage and stun | `$06:EDC5`, `$06:EE9F` | `prefix29_give_damage_all_classes_special_types_and_stun_paths_exclude_children` |
| zap | `$06:EC02` | `prefix29_zap_all_types_electric_state_blink_and_sword_paths_exclude_children` |
| damage type dispatch | `$06:F314` | `prefix29_damage_dispatch_all_types_faces_deflections_and_timer_paths_exclude_children` |
| throw | `$06:DF6D` | `prefix29_throw_all_return_gate_button_and_facing_paths_exclude_sound` |
| thrown collision | `$06:E164`, `$06:E1B2` | `prefix29_thrown_collision_all_e_flags_and_single_hit_paths_exclude_children` |
| Hoarder covered and lift | `$06:A86C`, `$06:A874`, `$06:AA0C`, `$06:AA14` | `prefix29_hoarder_covered_all_gates_proximity_lift_and_spawn_paths_exclude_children` |
| rupee pull | `$06:C017`, `$05:E24D`, `$05:E255`, `$06:F129`, `$06:F154`, `$06:F15C` | `prefix29_rupee_pull_all_pause_busy_overlap_and_prize_paths_exclude_children` |
| bee | `$1E:DC5B`, `$1E:DC68` | `prefix29_bee_dispatch_and_hive_all_states_and_allocation_paths_exclude_children` |
| Arrghus states | `$1E:B593`, `$1E:B5C8`, `$1E:B63D`, `$1E:B4CA`, `$1E:B4EF`, `$1E:B532` | `prefix29_arrghus_states_all_timers_velocity_z_rng_and_garnish_paths_exclude_children` |
| Arrghus draw return | `$1E:87F0` | `prefix28_arrghus_draw_all_states_waves_clips_and_page_crossings_exclude_children` |
| AddHookshot | `$09:9B10` | `prefix29_add_hookshot_all_allocation_directions_and_coordinate_wraps_exclude_children` |
| Y pushing and bonking | `$07:BDB1`, `$07:BDEB`, `$07:BE6B` | `prefix29_y_bonk_all_collision_swim_snap_direction_and_drag_paths_exclude_children` |
| Y item bow body | `$07:A006` | `prefix29_bow_all_button_animation_allocation_ammo_refund_and_cleanup_paths_exclude_children` |
| Y item hookshot body | `$07:AB25` | `prefix29_hookshot_item_all_button_door_defense_press_and_allocation_paths_exclude_children` |

The movement entries cover CancelDash, direct ResetSwimmingState/quiet-cape
calls, and Y-axis pushing/bonking including swim, direction, snap, drag and
return paths. The Y-item selector was already priced: moving its semantic body
behind the marked entry adds **zero selector clocks**. Bow and hookshot bodies,
and AddHookshot's allocation/direction/coordinate paths, supply their own ROM
blocks. The new `$07:BEA3` block split retains the existing clear-collision
return work rather than dropping or double-charging it.

Shared sprite leaves cover both location comparisons, weapon tink, zap, damage
type dispatch, sword/calculated damage, give-damage/stun, throwing, thrown
scenery/sprite collision and sound pan/queues. The projection helper reuses its
already-computed location predicates; lift callers omit the shared return
when the lifted tail already owns it. Damage-dispatch tail/SEC and recoil-timer
branches are covered explicitly. Nested damage, transform, stun, pan, allocation
and other children remain excluded from caller fixtures and charged once.

The Hoarder frantic body and active-bee body remain unpriced; their selectors
and covered/dormant branches are measured independently. Rupee pull routes its
existing long damage child through its marked entry. All six Arrghus state
bodies use their physical source scopes, including speed, RNG and garnish-scan
branches; the draw wrapper now includes its missing **42-clock `$1E:87F4` RTS**.
BossFinished closes Arrgi's repeated shared calls without pricing its residual.
No gameplay behavior correction is introduced.

Across the full ROM census, main instruction work increases by
**183,199,982 master clocks**, with **zero negative main deltas** and **zero
VWF changes**. The 282 selected rows gain **184,716** clocks, including
**163,508** in sprite families. The original 185 missing prefix rows gain
**114,016** clocks: early **0**, cohort A **47,228**, cohort B **33,978**,
cohort C **32,810**. These are observed recoveries after source-derived pricing,
not inputs to the prices. `rom-comparison.json`, `profile-budget-agreement.json`,
`family-deltas.csv/json` and `cohort-clock-recovery.json` retain the measurements.

### Classification and remaining concrete work

| Cohort | (a) instruction before → after | (c) upload before → after | (d) phase/wait before → after | Missing rows recovered / exposed |
| --- | ---: | ---: | ---: | ---: |
| Before 141,568 | 0 → 0 | 5 → 5 | 2 → 2 | 0 / 0 |
| 141,568–228,499 | 50 → 28 | 3 → 3 | 14 → 14 | 22 / 0 |
| 228,499–380,243 | 43 → 39 | 4 → 4 | 11 → 10 | 5 / 0 |
| 380,243–461,989 | 44 → 41 | 2 → 2 | 7 → 7 | 3 / 0 |
| Through 461,989 | **137 → 108** | **14 → 14** | **34 → 33** | **30 / 0** |

There are no class-(b) rows. The complete ROM recovers **50 rows**, exposing
**none**. `recovered-rom-rows.csv` lists all 50; `recovered-prefix-rows.csv/json`
lists the 30 prefix rows; `newly-exposed-prefix-rows.csv/json` is empty.
`classification-before.json`, `classification-all.json`, `classification.json`
and `remaining-blockers.csv/json` preserve before/after classification and
concrete signed routine, object and sprite owners. `blocker-decomposition.json`
and `signed-partition-agreement.json` conserve every selected row's partitions.

The later thrown `$ec` family at 356,806 closes **2,926 → 0**. Its distinct
spawn wrapper at 150,378 remains **64**. Eyegore `$83` closes **8,432 → 0**
across 367,291/367,394; `$84` retains **194** per instance, split between
`$1E:FE69` (**104**) and `$06:EAA0` (**90**). Terrorpin 372,814 closes
**938 → 0**; 344,246 retains **168**. Normal active Arrghus work is exact;
456,721's state-08 execute/init difference remains **142**. Arrgi at that row
moves **50,348 → 6,954**, recovering **43,394** BossFinished clocks; its
remaining execute/init/puff terms remain separately visible. At 297,585,
non-sprite work still names Y movement **1,716** and Mirror **150**, with the
retained **−22** main attribution; dash **96**, swimming **286** and cape
**178** close. AddHookshot's **1,210** closes from its source fixture.
Unpriced hookshot `$08:833C`, collision `$08:8DAE`, remaining movement and
other cohort owners retain their concrete source vectors; they are not inferred
from aggregate shortfalls.

Upload and wait/interrupt pricing are unchanged. Shared instruction owners
still affect two previously classified rows and are reported explicitly:
**146,148 (c)** gains **286** ResetSwimmingState clocks, hold **46 → 46**;
**254,154 (d, spotlight)** gains **294** heart caller/bounce clocks, hold
**1 → 2**, recovering its source lag of 2. The latter's nonpoll gap closes
**272 → −22**; the existing P2 model consequently includes its existing raster
wait (**269,848**). No wait price or phase rule is changed. Thus the batch does
not claim that every c/d predictor field is byte-identical.

The original (d) rows split into **9 polygon/IRQ-phase, 14 spotlight-wait,
11 other**; the remaining split is **9 / 13 / 11** after the shared-instruction
recovery. The seven protected −22 polygon rows have identical complete predictor
records before/after, with **main + shortfall = 230,464** throughout (source
main + the same shortfall = **230,442**). All other (d) rows and their measured
main/shortfall sums are below; no residual is promoted to an interrupt price.

| Iteration | Source-evidenced (d) kind | Native main before → after | Shortfall before → after | Main + shortfall before → after | Still missing |
| --- | --- | ---: | ---: | ---: | --- |
| 101,031 | spotlight wait | 477,054 → 477,054 | 160,288 → 160,288 | 637,342 → 637,342 | yes |
| 109,016 | spotlight wait | 486,090 → 486,090 | 151,160 → 151,160 | 637,250 → 637,250 | yes |
| 146,786 | polygon/IRQ phase | 227,764 → 227,764 | 2,700 → 2,700 | 230,464 → 230,464 | yes |
| 146,790 | polygon/IRQ phase | 227,862 → 227,862 | 2,602 → 2,602 | 230,464 → 230,464 | yes |
| 146,794 | polygon/IRQ phase | 225,452 → 225,452 | 5,012 → 5,012 | 230,464 → 230,464 | yes |
| 146,798 | polygon/IRQ phase | 222,642 → 222,642 | 7,822 → 7,822 | 230,464 → 230,464 | yes |
| 146,802 | polygon/IRQ phase | 220,050 → 220,050 | 10,414 → 10,414 | 230,464 → 230,464 | yes |
| 146,806 | polygon/IRQ phase | 218,178 → 218,178 | 12,286 → 12,286 | 230,464 → 230,464 | yes |
| 146,810 | polygon/IRQ phase | 215,732 → 215,732 | 14,732 → 14,732 | 230,464 → 230,464 | yes |
| 168,346 | other | 6,056,144 → 6,056,144 | 83,650 → 83,650 | 6,139,794 → 6,139,794 | yes |
| 171,309 | other | 5,047,274 → 5,047,274 | 21,452 → 21,452 | 5,068,726 → 5,068,726 | yes |
| 171,310 | other | 5,005,274 → 5,005,274 | 63,452 → 63,452 | 5,068,726 → 5,068,726 | yes |
| 172,380 | other | 5,269,332 → 5,269,332 | 143,322 → 143,322 | 5,412,654 → 5,412,654 | yes |
| 172,413 | other | 298,550 → 298,550 | 2 → 2 | 298,552 → 298,552 | yes |
| 174,296 | spotlight wait | 484,536 → 484,536 | 152,714 → 152,714 | 637,250 → 637,250 | yes |
| 189,923 | spotlight wait | 415,724 → 415,724 | 224,566 → 224,566 | 640,290 → 640,290 | yes |
| 242,100 | other | 5,023,212 → 5,023,212 | 44,504 → 44,504 | 5,067,716 → 5,067,716 | yes |
| 254,154 | spotlight wait | 408,548 → 408,842 | 228,538 → 0 | 637,086 → 408,842 | recovered |
| 294,844 | other | 3,970,322 → 3,970,322 | 7,410 → 7,410 | 3,977,732 → 3,977,732 | yes |
| 295,089 | spotlight wait | 490,158 → 490,158 | 147,092 → 147,092 | 637,250 → 637,250 | yes |
| 297,863 | spotlight wait | 506,262 → 506,262 | 130,988 → 130,988 | 637,250 → 637,250 | yes |
| 299,434 | spotlight wait | 489,402 → 489,402 | 147,848 → 147,848 | 637,250 → 637,250 | yes |
| 307,022 | spotlight wait | 421,850 → 421,850 | 234,416 → 234,416 | 656,266 → 656,266 | yes |
| 310,980 | other | 5,968,656 → 5,968,656 | 171,138 → 171,138 | 6,139,794 → 6,139,794 | yes |
| 310,982 | other | 3,994,972 → 3,994,972 | 84,542 → 84,542 | 4,079,514 → 4,079,514 | yes |
| 354,595 | other | 297,924 → 297,924 | 588 → 588 | 298,512 → 298,512 | yes |
| 378,093 | other | 301,536 → 301,536 | 1,364 → 1,364 | 302,900 → 302,900 | yes |
| 380,956 | polygon/IRQ phase | 1,236,436 → 1,236,436 | 19,320 → 19,320 | 1,255,756 → 1,255,756 | yes |
| 390,887 | polygon/IRQ phase | 3,969,936 → 3,969,936 | 6,876 → 6,876 | 3,976,812 → 3,976,812 | yes |
| 401,471 | spotlight wait | 499,832 → 499,832 | 137,418 → 137,418 | 637,250 → 637,250 | yes |
| 401,612 | spotlight wait | 409,552 → 409,552 | 230,738 → 230,738 | 640,290 → 640,290 | yes |
| 403,249 | spotlight wait | 411,848 → 411,848 | 228,489 → 228,489 | 640,337 → 640,337 | yes |
| 403,304 | spotlight wait | 419,812 → 419,812 | 236,546 → 236,546 | 656,358 → 656,358 | yes |
| 407,899 | spotlight wait | 412,762 → 412,762 | 243,504 → 243,504 | 656,266 → 656,266 | yes |

`d-classification.csv/json` also retains source polygon/IRQ CPU work and
V-counter reads. `protected-minus22-rows.csv/json` proves complete-record
identity for the seven protected rows. The signed sprite-family,
total-main-beyond−22 and nonpoll-main-beyond−22 negative audits are all empty.

| Missing before iteration | ROM missing/excess/FP before → after |
| --- | --- |
| 94,334 | 2/0/0 → 2/0/0 |
| 141,568 | 7/0/0 → 7/0/0 |
| 228,499 | 74/0/0 → 52/0/0 |
| 380,243 | 132/0/0 → 105/0/0 |
| 461,989 | 185/0/0 → 155/0/0 |

The final eight-slice seeded ROM helper reports **PASS**, **unexplained 0**,
complete **1,473,123 iterations**, **1,500/0/0 → 1,450/0/0** missing/excess/FP.
`admission-final/rom.csv`, `admission-final/rom.log`, `rom-final-helper.log`
and `final-replay-commands.json` identify the exact binary and slice commands.

### Single seeded live census and direct poll attribution

Exactly **one** full seeded `--live-apu-reads --pacing-predictor` census runs
on the frozen production binary and exits zero. Totals change from
**1,735/144/14 → 1,715/151/18** missing/excess/FP: missing improves by 20,
while excess rises by 7 and FP by 4. The fifth live prefix worsens and is
reported without changing poll logic or acceptance rules.

There are **83 changed live holds**: **50 shared ROM recoveries** and
**33 live-only changes**. Every shared row has exactly the same hold delta as
ROM; there are **zero mixed ROM/live deltas**. All 33 live-only changes belong
to three crystal song-end episodes at directly observed source PC **`$08:C606`**;
**zero rows are unattributed**. The pendant **`$08:C3FD`** episodes have no
changed live-only holds.

The source PC identities are read directly from the retained cold unmodified-ROM
APUI bus, after verifying its SHA-256
`948567a2b5caa2201d51db3c719dacb5934c69299a344f73dff6922429b4798b`
and its command receipt
`c0d84a921c1eecd07d539acc01eb7f185a14121726324ddbca98501e213c2efb`.
`poll-source-identity.json` contains the 20 reread observations at episode entry
and first source zero, and `poll-source-verification.json` retains the hash
check. No held-row pattern supplies a source-PC identity.

Source context, APUI reads and APUI writes agree on **all 1,473,123 trace rows**;
source start/end counters also agree throughout. The normalized source SHA-256
is `a85696ffff0f15dd054297a8a36d421583ccb319440192fc3e16c90fc35307d5`
on both sides, matching main's prior receipt. Cartridge APUI answers remain
disabled throughout. `live-trace-agreement.json` records the exact serialization
and equality checks.

Attribution uses **2,450 changed four-lane live-vs-ROM work events** and
**53 context/boolean-poll events**, a union of **2,454 iterations**. Bounds are
established independently of the changed-hold set: the actual source/native
poll cluster, last execution event, then the first actual reseed before the
containing 2,000-iteration boundary, otherwise that boundary. Some episodes
have work-price differences without a changed exit or hold; they retain their
observed event bounds too. `live-work-events.json`, `live-context-poll-events.json`,
`live-execution-events.json`, `live-poll-clusters.json`, `live-reseeds.json` and
`poll-attribution.json` preserve the complete derivation.

| Source episode | Direct source PC | Source first zero | Native first zero before → after | Last changed execution event | Recovery end | Live-only rows | Δ missing/excess/FP |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| 94,334 | `$08:C3FD` | 94,854 | 94,854 → 94,854 | — | 96,000 | 0 | +0/+0/+0 |
| 141,568 | `$08:C3FD` | 142,088 | 142,088 → 142,088 | — | 144,000 | 0 | +0/+0/+0 |
| 228,499 | `$08:C3FD` | 229,035 | 229,036 → 229,036 | — | 230,000 | 0 | +0/+0/+0 |
| 380,243 | `$08:C606` | 380,881 | 380,881 → 380,882 | 383,635 | 383,636 | 15 | +114/+90/+5 |
| 461,989 | `$08:C606` | 462,628 | 462,627 → 462,627 | 463,179 | 464,000 | 0 | +0/+0/+0 |
| 626,618 | `$08:C606` | 627,256 | 627,256 → 627,256 | — | 628,000 | 0 | +0/+0/+0 |
| 735,229 | `$08:C606` | 735,867 | 735,867 → 735,868 | 736,633 | 738,000 | 6 | +12/+12/+3 |
| 881,095 | `$08:C606` | 881,734 | 881,734 → 881,734 | — | 882,000 | 0 | +0/+0/+0 |
| 1,064,096 | `$08:C606` | 1,064,734 | 1,064,735 → 1,064,734 | 1,066,962 | 1,066,963 | 12 | -96/-95/-4 |
| 1,317,697 | `$08:C606` | 1,318,335 | 1,318,336 → 1,318,336 | 1,320,899 | 1,320,900 | 0 | +0/+0/+0 |

Every changed live-only hold is listed below. `live-only-changed-rows.csv/json`
also retains both ROM holds, source/native first-zero observations and the
independently established recovery end for each row. `live-changed-rows.csv`
includes all 83 changed live holds, including the 50 shared ROM recoveries.

| Iteration | Source episode / direct PC | Source lag | Live hold before → after |
| --- | --- | ---: | ---: |
| 380,946 | 380,243 / `$08:C606` | 1 | 1 → 0 |
| 380,947 | 380,243 / `$08:C606` | 0 | 0 → 1 |
| 380,956 | 380,243 / `$08:C606` | 5 | 4 → 0 |
| 380,957 | 380,243 / `$08:C606` | 0 | 0 → 4 |
| 381,426 | 380,243 / `$08:C606` | 8 | 8 → 0 |
| 381,427 | 380,243 / `$08:C606` | 0 | 0 → 8 |
| 383,178 | 380,243 / `$08:C606` | 0 | 0 → 7 |
| 383,181 | 380,243 / `$08:C606` | 7 | 7 → 0 |
| 383,343 | 380,243 / `$08:C606` | 7 | 7 → 0 |
| 383,346 | 380,243 / `$08:C606` | 0 | 0 → 7 |
| 383,631 | 380,243 / `$08:C606` | 59 | 59 → 0 |
| 383,632 | 380,243 / `$08:C606` | 6 | 6 → 0 |
| 383,633 | 380,243 / `$08:C606` | 23 | 22 → 0 |
| 383,634 | 380,243 / `$08:C606` | 1 | 1 → 59 |
| 383,635 | 380,243 / `$08:C606` | 1 | 1 → 6 |
| 735,937 | 735,229 / `$08:C606` | 1 | 1 → 0 |
| 735,938 | 735,229 / `$08:C606` | 0 | 0 → 1 |
| 735,947 | 735,229 / `$08:C606` | 5 | 4 → 0 |
| 735,948 | 735,229 / `$08:C606` | 0 | 0 → 4 |
| 736,420 | 735,229 / `$08:C606` | 8 | 7 → 0 |
| 736,422 | 735,229 / `$08:C606` | 0 | 0 → 7 |
| 1,064,798 | 1,064,096 / `$08:C606` | 1 | 0 → 1 |
| 1,064,799 | 1,064,096 / `$08:C606` | 0 | 1 → 0 |
| 1,064,808 | 1,064,096 / `$08:C606` | 5 | 0 → 4 |
| 1,064,809 | 1,064,096 / `$08:C606` | 0 | 4 → 0 |
| 1,065,281 | 1,064,096 / `$08:C606` | 8 | 0 → 7 |
| 1,065,282 | 1,064,096 / `$08:C606` | 0 | 7 → 0 |
| 1,066,543 | 1,064,096 / `$08:C606` | 0 | 7 → 0 |
| 1,066,544 | 1,064,096 / `$08:C606` | 7 | 0 → 7 |
| 1,066,958 | 1,064,096 / `$08:C606` | 59 | 0 → 59 |
| 1,066,959 | 1,064,096 / `$08:C606` | 6 | 59 → 6 |
| 1,066,960 | 1,064,096 / `$08:C606` | 24 | 6 → 24 |
| 1,066,961 | 1,064,096 / `$08:C606` | 1 | 24 → 1 |

| Missing before iteration | Live missing/excess/FP before → after |
| --- | --- |
| 94,334 | 2/0/0 → 2/0/0 |
| 141,568 | 7/0/0 → 7/0/0 |
| 228,499 | 74/0/0 → 52/0/0 |
| 380,243 | 141/9/2 → 114/9/2 |
| 461,989 | 194/9/2 → 278/99/7 |

The three changed crystal episodes contribute **+30 missing, +7 excess, +4 FP**;
the 50 shared ROM recoveries contribute **−50 missing**. These sum exactly to
the full live change **−20/+7/+4**. No runtime compensation, per-episode price,
source answer, poll matcher, classifier, wait rule or P2 change is added.
`live-command.json` records **one launch**, `live-comparison.json` preserves
all row deltas, and the independent gate-tools `live-counts.txt` confirms the
final **1,715/151/18** totals.

### CPU-picture and verification boundary

The final seeded `--cpu-video` census retains **1,462,997 exact pictures out
of 1,473,123 iterations**. All eight complete per-iteration picture-hash streams
are **byte-for-byte identical to main**; every slice is WRAM-clean and exits
zero. `cpu-picture-agreement.json`, `cpu-gate.txt`, `cpu-commands.json` and
`cpu-final/` retain the counts, hashes and exact CPU-only commands. This is
seeded CPU-picture agreement, not a GPU or continuous cold A/V claim.

Filtered verification passes: **21 prefix-29 fixtures**, the damage-dispatch
fixture rerun after its final branch-coverage expansion, **17 prefix-28 fixtures**,
**16 marker-37 fixtures**, and **8 generator tests**. The owner table includes
21 new fixtures plus the corrected existing Arrghus draw-return fixture.
`filtered-results.json` contains the exact logs and results. Final production
builds are **warning-free**; the default library build's **24 warning headers**
are identical to main, with no added warnings (`warning-agreement.json`).

After all code and test changes, the full default library suite runs exactly
**once**, using the requested command:

```sh
ZELDA3_ASSETS=/Users/missingno/Documents/zelda3-rs/zelda3_assets.dat cargo test -q --profile parity -p zelda3 -p snes --lib
```

SNES: **505 passed, 7 ignored**. Zelda: **3,135 passed, 3 ignored**, with the
sole permitted failure:
`cycle_models::spotlight_caller_return::tests::closing_dispatch_and_outdoor_movement_keep_the_actual_child_frames`.
There are **zero unexpected failures**, so no corrective full-suite rerun is
needed. The suite exits 101 for that permitted failure. `test-verification.json`,
`full-library-command.json` and `full-library.log` preserve the result.

The final source freeze covers **944 files**, all unchanged through final
verification. The current Cargo-built production binary and frozen census
binary have identical SHA-256. All **14 authority hashes**, **387 hunt-28
artifact receipts** and **341 retained source receipts** remain unchanged;
`authorities-after.json`, `read-only-agreement.json`, `tested-source-hashes.json`
and `source-freeze-agreement.json` retain those checks. `verification.json`
consolidates the admission results; `artifact-hashes.json` inventories this
batch's evidence.

Only ROM-derived instruction ownership and its source fixtures are changed.
Upload/wait/interrupt rules, classifier, matcher and P2 remain unchanged, with
the two shared c/d instruction effects reported explicitly above. Source fixtures,
seeded ROM prediction, live poll attribution and seeded CPU pictures establish
their separate proof boundaries; **108 instruction, 14 upload and 33 phase/wait
prefix blockers remain**. This batch is not full lagless or continuous A/V
acceptance. No GPU, `--video`, cached-A/V, external Snes9x process, new environment
switch, commit or push is part of this batch. Every Zelda executable command
uses CPU-only logical replay; every launched replay, test and analysis process
has exited. Only this worktree's reproducible compiler incremental cache was
removed to maintain the target-size limit; prior evidence is retained.

The final worktree target occupies **9.68 GB allocated**, below the **16 GB** limit. All launched gate, test, replay and trace-analysis processes have completed.
